mod database;
mod error;
pub mod model;

pub use database::Database;
pub use error::{DataError, Result};
pub use model::*;
