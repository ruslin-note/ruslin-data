use thiserror::Error;

pub type Result<T> = std::result::Result<T, SyncError>;

#[derive(Error, Debug)]
pub enum SyncError {
    #[error("unknown")]
    Unknown,
}
