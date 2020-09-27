use std::convert::TryInto;

use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub enum Request {
    Get { key: String },
    Set { key: String, value: String },
    Remove { key: String },
}

#[derive(Debug, Serialize, Deserialize)]
pub enum Response {
    Get(Option<String>),
    Set,
    Remove,
    Err(String),
}

pub struct PacketSize(u64);

impl PacketSize {
    pub fn new(size: u64) -> Self {
        PacketSize(size)
    }

    pub fn to_bytes(&self) -> [u8; 8] {
        self.0.to_be_bytes()
    }

    pub fn from_bytes(bytes: &mut &[u8]) -> Self {
        let (int_bytes, rest) = bytes.split_at(std::mem::size_of::<u64>());
        *bytes = rest;
        PacketSize(u64::from_be_bytes(int_bytes.try_into().unwrap()))
    }

    pub fn get_size(&self) -> u64 {
        self.0
    }
}
