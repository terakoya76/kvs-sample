#[macro_use]
extern crate clap;

use clap::{crate_version, App};
use std::process::exit;

fn main() {
    let yaml = load_yaml!("cli.yml");
    let matches = App::from_yaml(yaml).version(crate_version!()).get_matches();

    match matches.subcommand() {
        ("set", Some(m)) => {
            let k = m.value_of("key").unwrap();
            println!("key: {}", k);

            let v = m.value_of("value").unwrap();
            println!("value: {}", v);

            eprintln!("unimplemented");
            exit(1);
        }
        ("get", Some(m)) => {
            let k = m.value_of("key").unwrap();
            println!("key: {}", k);

            eprintln!("unimplemented");
            exit(1);
        }
        ("rm", Some(m)) => {
            let k = m.value_of("key").unwrap();
            println!("key: {}", k);

            eprintln!("unimplemented");
            exit(1);
        }
        _ => unreachable!(),
    }
}
