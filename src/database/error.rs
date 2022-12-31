use std::io;

use thiserror::Error;

// pub type Result<T> = std::result::Result<T, DatabaseError>;

#[derive(Error, Debug)]
pub enum DatabaseError {
    #[error("Error connecting to the database")]
    Open,
    #[error("Invalid data directory")]
    InvalidPath(#[from] io::Error),
    #[error("Error updating data")]
    Update,
    #[error("Error migrating db schema {0}")]
    Migration(Box<dyn std::error::Error + Send + Sync>),
    #[error("Error deleting data")]
    Delete,
    #[error("Error retrieving data")]
    Select,
    #[error("Error inserting data")]
    Insert,
    #[error("Error setting options")]
    Options(#[from] diesel::result::Error),
    #[error("Error cleaning DB file")]
    Vacuum,
    #[error("r2d2 error")]
    R2d2Error(#[from] r2d2::Error),
    #[error("Unknown Error")]
    Unknown,
}
