mod error;
mod file_api;
pub mod remote_api;
mod serializer;

pub use error::{SyncError, SyncResult};
pub use file_api::*;
pub use serializer::{ForSyncSerializer, SerializeForSync};

pub struct Synchronizer {}

pub struct ForSyncDeserializer(String);

pub trait DeserializeForSync {}
