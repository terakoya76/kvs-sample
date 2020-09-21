#![deny(missing_docs)]
//! kvs-client

use std::net::SocketAddr;
use std::process::exit;

use clap::Clap;

use kvs::{KvsClient, Result, ADDRESS_FORMAT, DEFAULT_LISTENING_ADDRESS};

#[derive(Debug, Clap)]
#[clap(name = "kvs-client", version, author, about)]
struct Opt {
    #[clap(subcommand)]
    command: Command,
}

#[derive(Debug, Clap)]
enum Command {
    #[clap(name = "get", about = "Get the string value of a given string key")]
    Get {
        #[clap(name = "KEY", about = "A string key")]
        key: String,
        #[clap(
            long,
            about = "Sets the server address",
            value_name = ADDRESS_FORMAT,
            default_value = DEFAULT_LISTENING_ADDRESS,
            parse(try_from_str)
        )]
        addr: SocketAddr,
    },
    #[clap(name = "set", about = "Set the value of a string key to a string")]
    Set {
        #[clap(name = "KEY", about = "A string key")]
        key: String,
        #[clap(name = "VALUE", about = "The string value of the key")]
        value: String,
        #[clap(
            long,
            about = "Sets the server address",
            value_name = ADDRESS_FORMAT,
            default_value = DEFAULT_LISTENING_ADDRESS,
            parse(try_from_str)
        )]
        addr: SocketAddr,
    },
    #[clap(name = "rm", about = "Remove a given string key")]
    Remove {
        #[clap(name = "KEY", about = "A string key")]
        key: String,
        #[clap(
            long,
            about = "Sets the server address",
            value_name = ADDRESS_FORMAT,
            default_value = DEFAULT_LISTENING_ADDRESS,
            parse(try_from_str)
        )]
        addr: SocketAddr,
    },
}

fn main() {
    let opt: Opt = Opt::parse();
    if let Err(e) = run(opt) {
        eprintln!("{}", e);
        exit(1);
    }
}

fn run(opt: Opt) -> Result<()> {
    match opt.command {
        Command::Get { key, addr } => {
            let mut client = KvsClient::connect(addr)?;
            if let Some(value) = client.get(key)? {
                println!("{}", value);
            } else {
                println!("Key not found");
            }
        }
        Command::Set { key, value, addr } => {
            let mut client = KvsClient::connect(addr)?;
            client.set(key, value)?;
        }
        Command::Remove { key, addr } => {
            let mut client = KvsClient::connect(addr)?;
            client.remove(key)?;
        }
    }
    Ok(())
}
