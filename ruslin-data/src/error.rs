pub use rusqlite::Error as SqliteError;
use thiserror::Error;

pub type Result<T> = std::result::Result<T, DataError>;

#[derive(Error, Debug)]
pub enum DataError {
    #[error("sqlite")]
    Sqlite(#[from] SqliteError),
}
