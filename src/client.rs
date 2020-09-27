use std::convert::TryInto;
use std::net::{SocketAddr, TcpStream};

use smol::io::{BufReader, BufWriter};
use smol::prelude::{AsyncReadExt, AsyncWriteExt};
use smol::Async;

use crate::common::{PacketSize, Request, Response};
use crate::{KvsError, Result};

/// Key value store client
pub struct KvsClient {
    reader: BufReader<Async<TcpStream>>,
    writer: BufWriter<Async<TcpStream>>,
}

impl KvsClient {
    /// Connect to `addr` to access `KvsServer`.
    pub async fn connect<A>(addr: A) -> Result<Self>
    where
        A: Into<SocketAddr>,
    {
        let tcp_reader = Async::<TcpStream>::connect(addr).await?;
        let tcp_writer = Async::new(tcp_reader.get_ref().try_clone()?)?;
        Ok(KvsClient {
            reader: BufReader::new(tcp_reader),
            writer: BufWriter::new(tcp_writer),
        })
    }

    /// Get the value of a given key from the server.
    pub async fn get(&mut self, key: String) -> Result<Option<String>> {
        let b = serde_json::to_vec(&Request::Get { key })?;
        let size = PacketSize::new(b.len().try_into()?);
        self.writer.write(&size.to_bytes()).await?;
        self.writer.write(&b).await?;
        self.writer.flush().await?;

        let mut contents = Vec::new();
        self.reader.read_to_end(&mut contents).await?;
        let resp: Response = serde_json::from_slice(&contents)?;
        match resp {
            Response::Get(value) => Ok(value),
            Response::Err(msg) => Err(KvsError::StringError(msg)),
            _ => Err(KvsError::StringError("Invalid response".to_owned())),
        }
    }

    /// Set the value of a string key in the server.
    pub async fn set(&mut self, key: String, value: String) -> Result<()> {
        let b = serde_json::to_vec(&Request::Set { key, value })?;
        let size = PacketSize::new(b.len().try_into()?);
        self.writer.write(&size.to_bytes()).await?;
        self.writer.write(&b).await?;
        self.writer.flush().await?;

        let mut contents = Vec::new();
        self.reader.read_to_end(&mut contents).await?;
        let resp: Response = serde_json::from_slice(&contents)?;
        match resp {
            Response::Set => Ok(()),
            Response::Err(msg) => Err(KvsError::StringError(msg)),
            _ => Err(KvsError::StringError("Invalid response".to_owned())),
        }
    }

    /// Remove a string key in the server.
    pub async fn remove(&mut self, key: String) -> Result<()> {
        let b = serde_json::to_vec(&Request::Remove { key })?;
        let size = PacketSize::new(b.len().try_into()?);
        self.writer.write(&size.to_bytes()).await?;
        self.writer.write(&b).await?;
        self.writer.flush().await?;

        let mut contents = Vec::new();
        self.reader.read_to_end(&mut contents).await?;
        self.writer.close().await?;
        let resp: Response = serde_json::from_slice(&contents)?;
        match resp {
            Response::Remove => Ok(()),
            Response::Err(msg) => Err(KvsError::StringError(msg)),
            _ => Err(KvsError::StringError("Invalid response".to_owned())),
        }
    }
}
