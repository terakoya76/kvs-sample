use std::convert::TryInto;
use std::net::{SocketAddr, TcpListener, TcpStream};

use smol::io::{BufReader, BufWriter};
use smol::prelude::{AsyncReadExt, AsyncWriteExt};
use smol::Async;

use crate::common::{PacketSize, Request, Response};
use crate::{KvsEngine, Result};

/// The default listening ADDRESS of KvsServer
pub const DEFAULT_LISTENING_ADDRESS: &str = "127.0.0.1:4000";

/// The format of listening ADDRESS
pub const ADDRESS_FORMAT: &str = "IP:PORT";

/// create new KvsEngine instance and run it
pub async fn run_with<E: KvsEngine>(engine: E, addr: SocketAddr) -> Result<()> {
    let server = KvsServer::new(engine);
    server.run(addr).await
}

/// The server of a key value store.
pub struct KvsServer<E: KvsEngine> {
    engine: E,
}

impl<E: KvsEngine> KvsServer<E> {
    /// Create a `KvsServer` with a given storage engine.
    pub fn new(engine: E) -> Self {
        KvsServer { engine }
    }

    /// Run the server listening on the given address
    pub async fn run<A>(&self, addr: A) -> Result<()>
    where
        A: Into<SocketAddr>,
    {
        let listener = Async::<TcpListener>::bind(addr)?;
        loop {
            let (stream, _) = listener.accept().await?;
            let engine = self.engine.clone();
            smol::spawn(serve(engine, stream)).detach();
        }
    }
}

async fn serve<E: KvsEngine>(engine: E, stream: Async<TcpStream>) -> Result<()> {
    let peer_addr = stream.get_ref().peer_addr()?;
    let mut reader = BufReader::new(&stream);
    let mut writer = BufWriter::new(&stream);

    let mut size = vec![0_u8; 8];
    reader.read_exact(&mut size).await?;
    let n = PacketSize::from_bytes(&mut size.as_ref()).get_size();
    info!("Receiving next {} bytes as request", n);

    let mut contents = vec![0; n.try_into()?];
    reader.read_exact(&mut contents).await?;
    let req: Request = serde_json::from_slice(&contents)?;
    info!("Receive request from {}: {:?}", peer_addr, req);

    let res = match req {
        Request::Get { key } => match engine.get(key).await {
            Ok(value) => Response::Get(value),
            Err(e) => Response::Err(format!("{}", e)),
        },
        Request::Set { key, value } => match engine.set(key, value).await {
            Ok(_) => Response::Set,
            Err(e) => Response::Err(format!("{}", e)),
        },
        Request::Remove { key } => match engine.remove(key).await {
            Ok(_) => Response::Remove,
            Err(e) => Response::Err(format!("{}", e)),
        },
    };
    let j = serde_json::to_vec(&res)?;
    writer.write(&j).await?;
    writer.flush().await?;

    Ok(())
}
