use crate::Result;
use rusqlite::Connection;
use std::path::Path;

pub struct Database {
    pub(crate) conn: Connection,
}

impl Database {
    pub fn open<P: AsRef<Path>>(path: P) -> Result<Self> {
        Ok(Self {
            conn: Connection::open(path)?,
        })
    }

    #[cfg(test)]
    pub fn open_in_memory() -> Result<Self> {
        Ok(Self {
            conn: Connection::open_in_memory()?,
        })
    }

    pub(crate) fn execute_batch(&self, sqls: Vec<String>) -> Result<()> {
        let mut batch_sqls: Vec<String> = vec!["BEGIN;".to_string()];
        let sqls_iter = sqls.into_iter().map(|mut sql| {
            sql.push(';');
            sql
        });
        batch_sqls.extend(sqls_iter);
        batch_sqls.push("COMMIT;".to_string());
        let batch_sqls = batch_sqls.join("\n");
        self.conn.execute_batch(&batch_sqls)?;
        Ok(())
    }
}
