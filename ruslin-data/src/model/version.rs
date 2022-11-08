// use rusqlite::Connection;

use rusqlite::Statement;

use crate::{Database, Result};

// CREATE TABLE version (version INT NOT NULL, table_fields_version INT NOT NULL DEFAULT 0)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Version {
    pub version: i32,
    pub table_fields_version: i32,
}

impl Version {
    pub fn get_or_create_version(db: &Database) -> Result<Version> {
        let mut create_version_table_stmt = db.conn.prepare("CREATE TABLE IF NOT EXISTS version (version INT NOT NULL, table_fields_version INT NOT NULL DEFAULT 0)")?;
        create_version_table_stmt.execute([])?;
        let mut query_version_stmt = db.conn.prepare("SELECT * FROM version LIMIT 1")?;
        if !query_version_stmt.exists([])? {
            db.conn.execute(
                "INSERT INTO version (version, table_fields_version) VALUES (0, 0)",
                [],
            )?;
        }
        Self::get_version(&mut query_version_stmt)
    }

    fn get_version(query_version_stmt: &mut Statement) -> Result<Self> {
        Ok(query_version_stmt.query_row([], |row| Ok(Version::new(row.get(0)?, row.get(1)?)))?)
    }

    pub(crate) fn new(version: i32, table_fields_version: i32) -> Version {
        Version {
            version,
            table_fields_version,
        }
    }

    pub(crate) fn update_version_sql(version: i32) -> String {
        format!("UPDATE version SET version = {version}")
    }

    pub(crate) fn update_table_fields_version_sql(table_fields_version: i32) -> String {
        format!("UPDATE version SET table_fields_version = {table_fields_version}")
    }
}

#[cfg(test)]
mod tests {
    use crate::{Database, Result};

    use super::Version;

    #[test]
    fn test_get_or_create_version() -> Result<()> {
        let db = Database::open_in_memory()?;
        let version = Version::get_or_create_version(&db)?;
        assert_eq!(version, Version::new(0, 0));
        let version = Version::get_or_create_version(&db)?;
        assert_eq!(version, Version::new(0, 0));
        let mut sqls: Vec<String> = Vec::new();
        sqls.push(Version::update_version_sql(1));
        sqls.push(Version::update_table_fields_version_sql(1));
        db.execute_batch(sqls)?;
        let version = Version::get_or_create_version(&db)?;
        assert_eq!(version, Version::new(1, 1));
        Ok(())
    }
}
