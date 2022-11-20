use chrono::Duration;
use diesel::connection::SimpleConnection;
use diesel::prelude::*;
use diesel::sqlite::SqliteConnection;

#[derive(Debug)]
pub struct ConnectionOptions {
    pub busy_timeout: Option<Duration>,
}

impl ConnectionOptions {
    pub fn apply(&self, conn: &mut SqliteConnection) -> QueryResult<()> {
        conn.batch_execute("PRAGMA foreign_keys = ON;")?;

        if let Some(duration) = self.busy_timeout {
            conn.batch_execute(&format!(
                "PRAGMA busy_timeout = {};",
                duration.num_milliseconds()
            ))?;
        }
        Ok(())
    }
}

impl Default for ConnectionOptions {
    fn default() -> Self {
        Self {
            busy_timeout: Some(Duration::milliseconds(500)),
        }
    }
}

impl diesel::r2d2::CustomizeConnection<SqliteConnection, diesel::r2d2::Error>
    for ConnectionOptions
{
    fn on_acquire(&self, conn: &mut SqliteConnection) -> Result<(), diesel::r2d2::Error> {
        self.apply(conn).map_err(diesel::r2d2::Error::QueryError)
    }
}