pub mod database;
mod models;
mod schema;
pub mod sync;

use std::path::Path;

pub use database::{Database, DatabaseError, DatabaseResult};
pub use models::*;

#[derive(Debug)]
pub struct RuslinData {
    pub db: Database,
}

impl RuslinData {
    pub fn new(data_dir: &Path) -> DatabaseResult<Self> {
        let db = Database::new(data_dir)?;
        Ok(Self { db })
    }
}
