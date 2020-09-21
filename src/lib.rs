#![deny(missing_docs)]
//! A simple key/value store.

#[macro_use]
extern crate log;

pub use client::KvsClient;
pub use engines::{KvStore, KvsEngine, SledKvsEngine};
pub use error::{KvsError, Result};
pub use server::{run_with, KvsServer, ADDRESS_FORMAT, DEFAULT_LISTENING_ADDRESS};
pub use thread_pool::{NaiveThreadPool, RayonThreadPool, SharedQueueThreadPool, ThreadPool};

mod client;
mod common;
mod engines;
mod error;
mod server;
mod thread_pool;
