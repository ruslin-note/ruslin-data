use rusqlite::named_params;

use crate::{get_id, Database, DateTime, ModelUpgrade, Result};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AbbrNote {
    pub id: String,
    pub folder_id: String,
    pub title: String,
    // pub abbr: String,
    pub created_time: DateTime,
    pub updated_time: DateTime,
    // pub user_created_time: DateTime,
    // pub user_updated_time: DateTime,
    pub conflict_original_id: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Note {
    pub id: String,
    pub folder_id: String,
    pub title: String,
    pub body: String,
    pub created_time: DateTime,
    pub updated_time: DateTime,
    // pub is_conflict: bool,
    // pub latitude: Option<f64>,
    // pub longitude: Option<f64>,
    // pub altitude: Option<f64>,
    // pub author: Option<String>,
    // pub source_url: Option<String>,
    // pub is_todo: bool,
    // pub todo_due: bool,
    // pub todo_completed: bool,
    // pub source: String,             // app enum?
    // pub source_application: String, // app bundle id ?
    // pub application_data: Option<String>,
    // pub order: Option<i64>,
    // pub user_created_time: DateTime,
    // pub user_updated_time: DateTime,
    // pub encryption_cipher_text: Option<String>,
    // pub encryption_applied: Option<bool>,
    // pub markup_language: bool, // true
    // pub is_shared: bool,
    // pub share_id: Option<String>,
    pub conflict_original_id: Option<String>,
    // pub master_key_id: Option<String>, // ?
    row_id: i64,
}

impl ModelUpgrade for Note {
    fn upgrade_target_v1(conn: &rusqlite::Connection) -> crate::Result<()> {
        let sql = r#"CREATE TABLE notes(
            id TEXT PRIMARY KEY,
            folder_id TEXT NOT NULL DEFAULT "",
            title TEXT NOT NULL DEFAULT "",
            body TEXT NOT NULL DEFAULT "",
            created_time INT NOT NULL,
            updated_time INT NOT NULL,
            is_conflict INT NOT NULL DEFAULT 0,
            latitude NUMERIC NOT NULL DEFAULT 0,
            longitude NUMERIC NOT NULL DEFAULT 0,
            altitude NUMERIC NOT NULL DEFAULT 0,
            author TEXT NOT NULL DEFAULT "",
            source_url TEXT NOT NULL DEFAULT "",
            is_todo INT NOT NULL DEFAULT 0,
            todo_due INT NOT NULL DEFAULT 0,
            todo_completed INT NOT NULL DEFAULT 0,
            source TEXT NOT NULL DEFAULT "",
            source_application TEXT NOT NULL DEFAULT "",
            application_data TEXT NOT NULL DEFAULT "",
            custom_order NUMERIC NOT NULL DEFAULT 0,
            user_created_time INT NOT NULL DEFAULT 0,
            user_updated_time INT NOT NULL DEFAULT 0,
            encryption_cipher_text TEXT NOT NULL DEFAULT "",
            encryption_applied INT NOT NULL DEFAULT 0,
            markup_language INT NOT NULL DEFAULT 1,
            is_shared INT NOT NULL DEFAULT 0, 
            share_id TEXT NOT NULL DEFAULT "", 
            conflict_original_id TEXT DEFAULT NULL,
            master_key_id TEXT NOT NULL DEFAULT "");

            CREATE INDEX notes_title ON notes (title);
            CREATE INDEX notes_updated_time ON notes (updated_time);
            CREATE INDEX notes_is_conflict ON notes (is_conflict);
            CREATE INDEX notes_is_todo ON notes (is_todo);
            CREATE INDEX notes_custom_order ON notes (custom_order);"#;
        conn.execute(sql, [])?;
        Ok(())
    }
}

impl Note {
    pub fn query_abbr_notes(db: &Database, folder_id: Option<&str>) -> Result<Vec<AbbrNote>> {
        let mut stmt = if folder_id.is_some() {
            db.conn.prepare("SELECT id, folder_id, title, created_time, updated_time, conflict_original_id FROM notes WHERE folder_id = ?1;")?
        } else {
            db.conn.prepare("SELECT id, folder_id, title, created_time, updated_time, conflict_original_id FROM notes;")?
        };
        let rows = if let Some(folder_id) = folder_id {
            assert!(!folder_id.is_empty());
            stmt.query([folder_id])?
        } else {
            stmt.query([])?
        };
        use fallible_iterator::FallibleIterator;
        let abbr_notes: Vec<AbbrNote> = rows
            .map(|row| {
                Ok(AbbrNote {
                    id: row.get(0)?,
                    folder_id: row.get(1)?,
                    title: row.get(2)?,
                    created_time: row.get(3)?,
                    updated_time: row.get(4)?,
                    conflict_original_id: row.get(5)?,
                })
            })
            .collect()?;
        Ok(abbr_notes)
    }

    fn is_insert(&self) -> bool {
        self.row_id == 0
    }

    pub fn new(title: String, body: String, folder_id: String) -> Self {
        let time = DateTime::now_utc();
        Self {
            id: get_id(),
            folder_id,
            title,
            body,
            created_time: time,
            updated_time: time,
            conflict_original_id: None,
            row_id: 0,
        }
    }

    pub fn save(&mut self, db: &Database) -> Result<()> {
        self.insert(db)
    }

    fn insert(&mut self, db: &Database) -> Result<()> {
        let sql = r#"INSERT INTO "notes" (
            "id",
            "folder_id",
            "title",
            "body",
            "created_time", 
            "updated_time"
        ) VALUES (:id, :folder_id, :title, :body, :created_time, :updated_time);"#;
        let mut stmt = db.conn.prepare(sql)?;
        self.row_id = stmt.insert(named_params! {
            ":id": self.id,
            ":folder_id": self.folder_id,
            ":title": self.title,
            ":body": self.body,
            ":created_time": self.created_time,
            ":updated_time": self.updated_time,
        })?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use crate::{Database, Note, Result};

    #[test]
    fn test_create_notes() -> Result<()> {
        let db = Database::open_in_memory()?;
        let mut note = Note::new(
            "title1".to_string(),
            "body1".to_string(),
            "folder_id1".to_string(),
        );
        note.save(&db)?;
        let abbr_notes = Note::query_abbr_notes(&db, Some("folder_id1"))?;
        assert_eq!(1, abbr_notes.len());
        Ok(())
    }
}
