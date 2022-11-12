use std::io;
use thiserror::Error;

pub type Result<T> = std::result::Result<T, SyncError>;

#[derive(Error, Debug)]
pub enum SyncError {
    #[error("io error")]
    IOError(#[from] io::Error),
    #[error("file not exists")]
    FileNotExists,
    #[error("unknown")]
    Unknown,
}
