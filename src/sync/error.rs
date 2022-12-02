use std::io;
use thiserror::Error;

use crate::DatabaseError;

pub type SyncResult<T> = std::result::Result<T, SyncError>;

#[derive(Error, Debug)]
pub enum SyncError {
    #[error("io error")]
    IOError(#[from] io::Error),
    #[error("file not exists")]
    FileNotExists,
    #[error("unknown")]
    Unknown,
    #[error("serialize error")]
    SerializeError(String),
    #[error("api error")]
    APIError(String),
    #[error("join error")]
    JoinError(#[from] tokio::task::JoinError),
    #[error("database error")]
    DatabaseError(#[from] DatabaseError),
    #[error("deserialize error")]
    DeserializeError { key: String, val: String },
    #[error("serde json error")]
    SerdeJsonError(#[from] serde_json::Error),
    #[error("sync config not exists")]
    SyncConfigNotExists,
}

impl serde::ser::Error for SyncError {
    fn custom<T>(msg: T) -> Self
    where
        T: std::fmt::Display,
    {
        Self::SerializeError(msg.to_string())
    }
}
