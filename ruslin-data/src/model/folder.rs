use crate::{get_id, DataError, Database, DateTime, ModelUpgrade, Result};
use rusqlite::{named_params, params, Connection};

#[derive(Debug, PartialEq, Eq)]
pub struct Folder {
    id: String,
    pub title: String,
    created_time: DateTime,
    updated_time: DateTime,
    user_created_time: DateTime,
    user_updated_time: DateTime,
    // pub encryption_cipher_text: String,
    // pub encryption_applied: bool,
    pub parent_id: Option<String>,
    // is_shared: bool,
    // share_id: String,
    // master_key_id: String,
    pub icon: String,
    row_id: i64,
}

impl ModelUpgrade for Folder {
    fn upgrade_target_v1(conn: &Connection) -> Result<()> {
        let sql = r#"CREATE TABLE folders (
            id TEXT PRIMARY KEY, 
            title TEXT NOT NULL DEFAULT "", 
            created_time INT NOT NULL, 
            updated_time INT NOT NULL, 
            user_created_time INT NOT NULL DEFAULT 0, 
            user_updated_time INT NOT NULL DEFAULT 0, 
            encryption_cipher_text TEXT NOT NULL DEFAULT "", 
            encryption_applied INT NOT NULL DEFAULT 0, 
            parent_id TEXT DEFAULT NULL, 
            is_shared INT NOT NULL DEFAULT 0, 
            share_id TEXT NOT NULL DEFAULT "", 
            master_key_id TEXT NOT NULL DEFAULT "", 
            icon TEXT NOT NULL DEFAULT "");

            CREATE INDEX folders_title ON folders (title);
            CREATE INDEX folders_updated_time ON folders (updated_time);"#;
        conn.execute(sql, [])?;
        Ok(())
    }
}

impl Folder {
    pub fn new(title: String, parent_id: Option<String>, icon: String) -> Self {
        let time = DateTime::now_utc();
        Self {
            id: get_id(),
            title,
            created_time: time,
            updated_time: time,
            user_created_time: time,
            user_updated_time: time,
            parent_id,
            icon,
            row_id: 0,
        }
    }

    fn is_insert(&self) -> bool {
        self.row_id == 0
    }

    pub fn get_id(&self) -> &str {
        &self.id
    }

    pub fn save(&mut self, db: &Database) -> Result<()> {
        if let Some(parent_id) = &self.parent_id {
            assert_ne!(parent_id, &self.id);
        }
        if self.is_insert() {
            self.insert(db)
        } else {
            self.update(db)
        }
    }

    fn update(&mut self, db: &Database) -> Result<()> {
        let sql = r#"UPDATE "folders"
        SET
            "title" = :title,
            "created_time" = :created_time,
            "updated_time" = :updated_time,
            "user_created_time" = :user_created_time,
            "user_updated_time" = :user_updated_time,
            "parent_id" = :parent_id,
            "icon" = :icon
        WHERE
            ROWID = :rowid;"#;
        let mut stmt = db.conn.prepare(sql)?;
        stmt.execute(named_params! {
            ":rowid": self.row_id,
            ":title": self.title,
            ":created_time": self.created_time,
            ":updated_time": self.updated_time,
            ":user_created_time": self.user_created_time,
            ":user_updated_time": self.user_updated_time,
            ":parent_id": self.parent_id,
            ":icon": self.icon,
        })?;
        Ok(())
    }

    fn insert(&mut self, db: &Database) -> Result<()> {
        let sql = r#"INSERT INTO "folders" (
            "id",
            "title", 
            "created_time", 
            "updated_time", 
            "user_created_time", 
            "user_updated_time", 
            "parent_id", 
            "icon"
        ) VALUES (:id, :title, :created_time, :updated_time, :user_created_time, :user_updated_time, :parent_id, :icon);"#;
        let mut stmt = db.conn.prepare(sql)?;
        self.row_id = stmt.insert(named_params! {
            ":id": self.id,
            ":title": self.title,
            ":created_time": self.created_time,
            ":updated_time": self.updated_time,
            ":user_created_time": self.user_created_time,
            ":user_updated_time": self.user_updated_time,
            ":parent_id": self.parent_id,
            ":icon": self.icon,
        })?;
        Ok(())
    }

    pub fn exists_by_id(db: &Database, id: &str) -> Result<bool> {
        let sql = r#"SELECT 1 FROM folders WHERE id = ?1;"#;
        let mut stmt = db.conn.prepare(sql)?;
        Ok(stmt.exists([id])?)
    }

    pub fn query_one_by_id(db: &Database, id: &str) -> Result<Folder> {
        let sql = r#"SELECT 
            "id",
            "title", 
            "created_time", 
            "updated_time", 
            "user_created_time", 
            "user_updated_time", 
            "parent_id", 
            "icon",
            "rowid"
            FROM folders
            WHERE id = ?1;"#;
        let mut stmt = db.conn.prepare(sql)?;
        Ok(stmt.query_row([id], |row| {
            Ok(Self {
                id: row.get(0)?,
                title: row.get(1)?,
                created_time: row.get(2)?,
                updated_time: row.get(3)?,
                user_created_time: row.get(4)?,
                user_updated_time: row.get(5)?,
                parent_id: row.get(6)?,
                icon: row.get(7)?,
                row_id: row.get(8)?,
            })
        })?)
    }

    pub fn delete(self, db: &Database) -> Result<()> {
        if self.is_insert() {
            return Err(DataError::ModelNotSaved("deleting folder"));
        }
        db.conn
            .execute("DELETE FROM folders WHERE rowid = ?1", [self.row_id])?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use crate::{Database, Folder, Result};

    #[test]
    fn test_create_update_folder() -> Result<()> {
        let db = Database::open_in_memory()?;
        let mut folder = Folder::new("title".to_string(), None, "icon".to_string());
        folder.save(&db)?;
        assert_eq!(folder, Folder::query_one_by_id(&db, folder.get_id())?);
        folder.title = "title2".to_string();
        folder.save(&db)?;
        assert_eq!(folder, Folder::query_one_by_id(&db, folder.get_id())?);
        Ok(())
    }

    #[test]
    fn test_delete_folder() -> Result<()> {
        let db = Database::open_in_memory()?;
        let folder = Folder::new("title".to_string(), None, "icon".to_string());
        assert!(folder.delete(&db).is_err());
        let mut folder = Folder::new("title".to_string(), None, "icon".to_string());
        let folder_id = folder.get_id().to_string();
        folder.save(&db)?;
        folder.delete(&db)?;
        assert!(Folder::query_one_by_id(&db, &folder_id).is_err());
        assert_eq!(false, Folder::exists_by_id(&db, &folder_id)?);
        Ok(())
    }
}
