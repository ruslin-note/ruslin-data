use crate::{model, Folder, ModelUpgrade, Note, Result, Version};
use rusqlite::Connection;
use std::{ops::Deref, path::Path};

pub struct Database {
    pub(crate) conn: Connection,
}

pub(crate) const DATABASE_VERSION: usize = 1;

impl Database {
    pub fn open<P: AsRef<Path>>(path: P) -> Result<Self> {
        let mut db = Self {
            conn: Connection::open(path)?,
        };
        db.upgrade()?;
        Ok(db)
    }

    #[cfg(test)]
    pub fn open_in_memory() -> Result<Self> {
        let mut db = Self {
            conn: Connection::open_in_memory()?,
        };
        db.upgrade()?;
        Ok(db)
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

    pub fn upgrade(&mut self) -> Result<()> {
        let sp = self.conn.savepoint()?;
        let mut upgrade_manager = UpgradeManager::new();
        upgrade_manager.add::<Folder>();
        upgrade_manager.add::<Note>();
        upgrade_manager.upgrade(&sp)?;
        Ok(sp.commit()?)
    }
}

struct UpgradeManager {
    queue: Vec<Vec<fn(&Connection) -> Result<()>>>,
}

impl UpgradeManager {
    fn new() -> Self {
        let mut queue: Vec<Vec<fn(&Connection) -> Result<()>>> =
            Vec::with_capacity(DATABASE_VERSION);
        for _ in 0..=DATABASE_VERSION {
            queue.push(Vec::new());
        }
        Self { queue }
    }

    fn add<T: ModelUpgrade>(&mut self) {
        self.queue[1].push(T::upgrade_target_v1);
    }

    fn upgrade(&self, conn: &Connection) -> Result<()> {
        let version = Version::get_or_create_version(conn)?;
        let version = version.version as usize;
        if version < DATABASE_VERSION {
            for v in version..=DATABASE_VERSION {
                for u in &self.queue[v] {
                    u(conn)?
                }
            }
        }
        Version::upgrade_version(conn, DATABASE_VERSION, DATABASE_VERSION)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use crate::{Database, Result};

    #[test]
    fn test_database_upgrade() -> Result<()> {
        let mut db = Database::open_in_memory()?;
        db.upgrade()?;
        Ok(())
    }
}
