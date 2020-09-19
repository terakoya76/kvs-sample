use std::env::current_dir;
use std::process::exit;

use clap::{load_yaml, App};

use kvs::{KvStore, KvsError, Result};

fn main() -> Result<()> {
    let yaml = load_yaml!("cli.yml");
    let matches = App::from_yaml(yaml)
        .version(env!("CARGO_PKG_VERSION"))
        .get_matches();

    match matches.subcommand() {
        ("set", Some(m)) => {
            let k = m.value_of("key").expect("KEY argument missing");
            let v = m.value_of("value").expect("KEY argument missing");

            let mut store = KvStore::open(current_dir()?)?;
            store.set(k.to_string(), v.to_string())?;
        }
        ("get", Some(m)) => {
            let k = m.value_of("key").expect("KEY argument missing");

            let mut store = KvStore::open(current_dir()?)?;
            if let Some(v) = store.get(k.to_string())? {
                println!("{}", v);
            } else {
                println!("Key not found");
            }
        }
        ("rm", Some(m)) => {
            let k = m.value_of("key").expect("KEY argument missing");

            let mut store = KvStore::open(current_dir()?)?;
            match store.remove(k.to_string()) {
                Ok(()) => {}
                Err(KvsError::KeyNotFound) => {
                    println!("Key not found");
                    exit(1);
                }
                Err(e) => return Err(e),
            }
        }
        _ => unreachable!(),
    }
    Ok(())
}
