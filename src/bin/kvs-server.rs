#![deny(missing_docs)]
//! kvs-server

#[macro_use]
extern crate log;

use std::env::current_dir;
use std::fs;
use std::net::SocketAddr;
use std::process::exit;
use std::str::FromStr;

use clap::{ArgEnum, Clap};
use derive_more::Display;
use log::LevelFilter;

use kvs::*;

const DEFAULT_LISTENING_ADDRESS: &str = "127.0.0.1:4000";
const DEFAULT_ENGINE: Engine = Engine::kvs;

#[derive(Clap, Debug)]
#[clap(name = "kvs-server", version, author, about)]
struct Opt {
    #[clap(
        long,
        about = "Sets the listening address",
        value_name = "IP:PORT",
        default_value = "DEFAULT_LISTENING_ADDRESS",
        parse(try_from_str)
    )]
    addr: SocketAddr,
    #[clap(
        long,
        about = "Sets the storage engine",
        value_name = "ENGINE-NAME",
        possible_values = &Engine::VARIANTS,
        arg_enum
    )]
    engine: Option<Engine>,
}

#[allow(non_camel_case_types)]
#[derive(ArgEnum, Debug, Copy, Clone, PartialEq, Eq, Display)]
enum Engine {
    #[display(fmt = "kvs")]
    kvs,
    #[display(fmt = "sled")]
    sled,
}

impl FromStr for Engine {
    type Err = ();
    fn from_str(s: &str) -> std::result::Result<Self, ()> {
        match s {
            "kvs" => Ok(Engine::kvs),
            "sled" => Ok(Engine::sled),
            _ => Err(()),
        }
    }
}

fn main() {
    env_logger::builder().filter_level(LevelFilter::Info).init();
    let mut opt = Opt::parse();
    let res = current_engine().and_then(move |curr_engine| {
        if opt.engine.is_none() {
            opt.engine = curr_engine;
        }
        if curr_engine.is_some() && opt.engine != curr_engine {
            error!("Wrong engine!");
            exit(1);
        }
        run(opt)
    });
    if let Err(e) = res {
        error!("{}", e);
        exit(1);
    }
}

fn run(opt: Opt) -> Result<()> {
    let engine = opt.engine.unwrap_or(DEFAULT_ENGINE);
    info!("kvs-server {}", env!("CARGO_PKG_VERSION"));
    info!("Storage engine: {}", engine);
    info!("Listening on {}", opt.addr);

    // write engine to engine file
    fs::write(current_dir()?.join("engine"), format!("{}", engine))?;

    match engine {
        Engine::kvs => run_with_engine(KvStore::open(current_dir()?)?, opt.addr),
        Engine::sled => run_with_engine(SledKvsEngine::new(sled::open(current_dir()?)?), opt.addr),
    }
}

fn run_with_engine<E: KvsEngine>(engine: E, addr: SocketAddr) -> Result<()> {
    let server = KvsServer::new(engine);
    server.run(addr)
}

fn current_engine() -> Result<Option<Engine>> {
    let engine = current_dir()?.join("engine");
    if !engine.exists() {
        return Ok(None);
    }

    match fs::read_to_string(engine)?.parse() {
        Ok(engine) => Ok(Some(engine)),
        Err(e) => {
            warn!("The content of engine file is invalid: {:?}", e);
            Ok(None)
        }
    }
}
