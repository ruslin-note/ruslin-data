mod database;
mod models;
mod schema;

// mod error;
// pub mod model;

// pub use database::Database;
// pub use error::{DataError, Result};
// pub use model::*;

pub use database::{Database, DatabaseError, DatabaseResult};
pub use models::*;
