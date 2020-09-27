use std::num::TryFromIntError;
use std::string::FromUtf8Error;

//use crossbeam::crossbeam_channel::RecvError;
use smol::channel::RecvError;
use thiserror::Error;

/// Error type for kvs.
#[derive(Error, Debug)]
pub enum KvsError {
    /// IO error.
    #[error("{}", .0)]
    Io(#[from] std::io::Error),

    /// Serialization or deserialization error.
    #[error("{}", .0)]
    Serde(#[from] serde_json::Error),

    /// Removing non-existent key error.
    #[error("Key not found")]
    KeyNotFound,

    /// Unexpected command type error.
    /// It indicated a corrupted log or a program bug.
    #[error("Unexpected command type")]
    UnexpectedCommandType,

    /// Key or value is invalid UTF-8 sequence
    #[error("UTF-8 error: {}", .0)]
    Utf8(#[from] FromUtf8Error),

    /// Sled error
    #[error( "sled error: {}", .0)]
    Sled(#[from] sled::Error),

    /// recv error
    #[error( "recv error: {}", .0)]
    Recv(#[from] RecvError),

    /// try from int error
    #[error( "try from int error: {}", .0)]
    TryFromInt(#[from] TryFromIntError),

    /// Error with a string message
    #[error("{}", .0)]
    StringError(String),

    /// anonymous error
    #[error(transparent)]
    Other(#[from] anyhow::Error),
}

/// Result type for kvs.
pub type Result<T> = std::result::Result<T, KvsError>;
