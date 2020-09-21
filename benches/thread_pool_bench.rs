#[macro_use]
extern crate criterion;

use std::iter;
use std::net::{IpAddr, Ipv4Addr, SocketAddr};

use criterion::{BatchSize, Criterion, ParameterizedBenchmark};
use kvs::{
    run_with, KvStore, KvsClient, NaiveThreadPool, RayonThreadPool, SharedQueueThreadPool,
    ThreadPool,
};
use rand::prelude::*;
use tempfile::TempDir;

fn set_bench(c: &mut Criterion) {
    let bench = ParameterizedBenchmark::new(
        "naive",
        |b, _| {
            b.iter_batched(
                || {
                    let temp_dir = TempDir::new().unwrap();
                    let store = KvStore::open(temp_dir.path()).unwrap();
                    let pool = NaiveThreadPool::new(num_cpus::get() as u32).unwrap();
                    let addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 3001);

                    let server = run_with(store, pool, addr).unwrap();
                    let client = KvsClient::connect(addr).unwrap();
                    (client, server)
                },
                |(mut client, _server)| {
                    for i in 1..10 {
                        client
                            .set(format!("key{}", i), "value".to_string())
                            .unwrap();
                    }
                },
                BatchSize::SmallInput,
            )
        },
        iter::once(()),
    )
    .with_function("shared_queue", |b, _| {
        b.iter_batched(
            || {
                let temp_dir = TempDir::new().unwrap();
                let store = KvStore::open(temp_dir.path()).unwrap();
                let pool = SharedQueueThreadPool::new(num_cpus::get() as u32).unwrap();
                let addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 3002);

                let server = run_with(store, pool, addr).unwrap();
                let client = KvsClient::connect(addr).unwrap();
                (client, server)
            },
            |(mut client, _server)| {
                for i in 1..10 {
                    client
                        .set(format!("key{}", i), "value".to_string())
                        .unwrap();
                }
            },
            BatchSize::SmallInput,
        )
    })
    .with_function("rayon", |b, _| {
        b.iter_batched(
            || {
                let temp_dir = TempDir::new().unwrap();
                let store = KvStore::open(temp_dir.path()).unwrap();
                let pool = RayonThreadPool::new(num_cpus::get() as u32).unwrap();
                let addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 3003);

                let server = run_with(store, pool, addr).unwrap();
                let client = KvsClient::connect(addr).unwrap();
                (client, server)
            },
            |(mut client, _server)| {
                for i in 1..10 {
                    client
                        .set(format!("key{}", i), "value".to_string())
                        .unwrap();
                }
            },
            BatchSize::SmallInput,
        )
    });
    c.bench("set_bench", bench);
}

criterion_group!(benches, set_bench);
criterion_main!(benches);
