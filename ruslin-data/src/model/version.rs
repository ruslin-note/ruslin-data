// use rusqlite::Connection;

use rusqlite::{params, Connection, Statement};

use crate::Result;

// CREATE TABLE version (version INT NOT NULL, table_fields_version INT NOT NULL DEFAULT 0)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Version {
    pub version: usize,
    pub table_fields_version: usize,
}

impl Version {
    pub fn get_or_create_version(conn: &Connection) -> Result<Version> {
        let mut create_version_table_stmt = conn.prepare("CREATE TABLE IF NOT EXISTS version (version INT NOT NULL, table_fields_version INT NOT NULL DEFAULT 0)")?;
        create_version_table_stmt.execute([])?;
        let mut query_version_stmt = conn.prepare("SELECT * FROM version LIMIT 1")?;
        if !query_version_stmt.exists([])? {
            conn.execute(
                "INSERT INTO version (version, table_fields_version) VALUES (0, 0)",
                [],
            )?;
        }
        Self::get_version(&mut query_version_stmt)
    }

    fn get_version(query_version_stmt: &mut Statement) -> Result<Self> {
        Ok(query_version_stmt.query_row([], |row| Ok(Version::new(row.get(0)?, row.get(1)?)))?)
    }

    pub(crate) fn new(version: usize, table_fields_version: usize) -> Version {
        Version {
            version,
            table_fields_version,
        }
    }

    pub(crate) fn upgrade_version(
        conn: &Connection,
        version: usize,
        table_fields_version: usize,
    ) -> Result<()> {
        conn.execute(
            "UPDATE version SET version = ?1, table_fields_version = ?2",
            params![version, table_fields_version],
        )?;
        Ok(())
    }
}
