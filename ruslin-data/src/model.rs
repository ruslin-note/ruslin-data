mod folder;
mod note;
mod version;

pub use folder::Folder;
pub use note::Note;
pub use version::Version;

use crate::{Database, Result};
use rusqlite::{
    types::{FromSql, FromSqlError},
    Connection, Savepoint, ToSql,
};
use time::OffsetDateTime;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DateTime(i64);

impl DateTime {
    pub fn now_utc() -> Self {
        Self(OffsetDateTime::now_utc().unix_timestamp())
    }

    pub fn from_unix_timestamp(timestamp: i64) -> Self {
        Self(timestamp)
    }

    pub fn to_unix_timestamp(&self) -> i64 {
        self.0
    }
}

// https://github.com/rusqlite/rusqlite/blob/0bd6d6322c365960f18f2e6d27c39fffda893c3c/src/types/mod.rs#L49
impl FromSql for DateTime {
    fn column_result(value: rusqlite::types::ValueRef<'_>) -> rusqlite::types::FromSqlResult<Self> {
        i64::column_result(value)
            .map(DateTime::from_unix_timestamp)
            .map_err(|err| FromSqlError::Other(Box::new(err)))
    }
}

impl ToSql for DateTime {
    fn to_sql(&self) -> rusqlite::Result<rusqlite::types::ToSqlOutput<'_>> {
        Ok(self.to_unix_timestamp().into())
    }
}

pub(crate) trait ModelUpgrade {
    fn upgrade_target_v1(conn: &Connection) -> Result<()>;
}

pub(crate) fn get_id() -> String {
    uuid::Uuid::now_v7().simple().to_string()
}

#[cfg(test)]
mod tests {
    use crate::DateTime;

    #[test]
    fn test_date_time() {
        let date_time = DateTime::now_utc();
        // date_time.to_unix_timestamp()
        assert_eq!(
            date_time,
            DateTime::from_unix_timestamp(date_time.to_unix_timestamp())
        );
    }
}
