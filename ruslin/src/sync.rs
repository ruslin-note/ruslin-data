mod error;
mod file_api;
pub mod remote_api;

pub use error::{SyncError, SyncResult};
pub use file_api::*;

pub struct Synchronizer {}
