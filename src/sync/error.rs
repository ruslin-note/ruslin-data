use std::io;
use thiserror::Error;

use crate::DatabaseError;

pub type SyncResult<T> = std::result::Result<T, SyncError>;

#[derive(Error, Debug)]
pub enum SyncError {
    #[error("io error: {0}")]
    IOError(#[from] io::Error),
    #[error("file not exists: {0}")]
    FileNotExists(String),
    #[error("cannot handle conflitc for two different notes")]
    HandleConflictForDiffNote,
    #[error("unknown")]
    Unknown,
    #[error("serialize error: {0}")]
    SerializeError(String),
    #[error("api error: {0}")]
    APIError(Box<dyn std::error::Error + Send + Sync>),
    #[error("misconfiguration")]
    Misconfiguration,
    #[error("join error: {0}")]
    JoinError(#[from] tokio::task::JoinError),
    #[error("database error: {0}")]
    DatabaseError(#[from] DatabaseError),
    #[error("deserialize error: {key} -> {val}")]
    DeserializeError { key: String, val: String },
    #[error("serde json error: {0}")]
    SerdeJsonError(#[from] serde_json::Error),
    #[error("sync config not exists")]
    SyncConfigNotExists,
    #[error("not supported sync target info {0}")]
    NotSupportedSyncTargetInfo(String),
}

impl serde::ser::Error for SyncError {
    fn custom<T>(msg: T) -> Self
    where
        T: std::fmt::Display,
    {
        Self::SerializeError(msg.to_string())
    }
}
