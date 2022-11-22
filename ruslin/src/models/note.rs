use std::hash::{Hash, Hasher};

use crate::{
    schema::notes,
    sync::{ForSyncSerializer, SerializeForSync, SyncResult},
    DateTimeTimestamp, ModelType,
};
use diesel::prelude::*;

use super::ids::{FolderID, NoteID};

#[derive(Clone, Identifiable, Insertable, Queryable, Eq, Debug)]
#[diesel(primary_key(id))]
#[diesel(table_name = notes)]
pub struct AbbrNote {
    pub id: NoteID,
    pub parent_id: FolderID,
    pub title: String,
    pub created_time: DateTimeTimestamp,
    pub updated_time: DateTimeTimestamp,
}

#[derive(Clone, Identifiable, Insertable, Queryable, Debug)]
#[diesel(primary_key(id))]
#[diesel(table_name = notes)]
pub struct Note {
    pub id: NoteID,
    pub parent_id: FolderID,
    pub title: String,
    pub body: String,
    pub created_time: DateTimeTimestamp,
    pub updated_time: DateTimeTimestamp,
    pub is_conflict: bool,
    pub latitude: f64,
    pub longitude: f64,
    pub altitude: f64,
    pub author: String,
    pub source_url: String,
    pub is_todo: bool,
    pub todo_due: bool,
    pub todo_completed: bool,
    pub source: String,
    pub source_application: String,
    pub application_data: String,
    pub custom_order: i64,
    pub user_created_time: DateTimeTimestamp,
    pub user_updated_time: DateTimeTimestamp,
    pub encryption_cipher_text: String,
    pub encryption_applied: bool,
    pub markup_language: bool,
    pub is_shared: bool,
    pub share_id: String,
    pub conflict_original_id: Option<String>,
    pub master_key_id: String,
}

// source: joplin
// source_application: net.cozic.joplin-cli

impl Note {
    pub fn new(parent_id: FolderID, title: String, body: String) -> Self {
        let dt = DateTimeTimestamp::now();
        Self {
            id: NoteID::new(),
            parent_id,
            title,
            body,
            created_time: dt,
            updated_time: dt,
            is_conflict: false,
            latitude: 0.0,
            longitude: 0.0,
            altitude: 0.0,
            author: "".to_string(),
            source_url: "".to_string(),
            is_todo: false,
            todo_due: false,
            todo_completed: false,
            source: "ruslin".to_string(),
            source_application: "app.ruslin.default".to_string(),
            application_data: "".to_string(),
            custom_order: dt.timestamp_millis(),
            user_created_time: dt,
            user_updated_time: dt,
            encryption_cipher_text: "".to_string(),
            encryption_applied: false,
            markup_language: true,
            is_shared: false,
            share_id: "".to_string(),
            conflict_original_id: None,
            master_key_id: "".to_string(),
        }
    }

    pub fn md_file_path(&self) -> String {
        format!("{}.md", self.id.as_str())
    }
}

impl Note {
    pub fn updated(&self) -> Self {
        let mut note = self.clone();
        note.updated_time = DateTimeTimestamp::now();
        note
    }
}

impl Hash for AbbrNote {
    fn hash<H: Hasher>(&self, hasher: &mut H) {
        self.id.hash(hasher);
    }
}

impl PartialEq for AbbrNote {
    fn eq(&self, other: &AbbrNote) -> bool {
        self.id == other.id
    }
}

impl Hash for Note {
    fn hash<H: Hasher>(&self, hasher: &mut H) {
        self.id.hash(hasher);
    }
}

impl PartialEq for Note {
    fn eq(&self, other: &Note) -> bool {
        self.id == other.id
    }
}

impl SerializeForSync for Note {
    fn serialize(&self) -> SyncResult<ForSyncSerializer> {
        let mut ser = ForSyncSerializer::new(&self.title, Some(&self.body));
        ser.serialize_str("id", self.id.as_str());
        ser.serialize_str("parent_id", self.parent_id.as_str());
        ser.serialize_str("title", &self.title);
        ser.serialize_datetime("created_time", self.created_time);
        ser.serialize_datetime("updated_time", self.updated_time);
        ser.serialize_bool("is_conflict", self.is_conflict);
        ser.serialize_f64("latitude", self.latitude);
        ser.serialize_f64("longitude", self.longitude);
        ser.serialize_f64("altitude", self.altitude);
        ser.serialize_str("author", &self.author);
        ser.serialize_str("source_url", &self.source_url);
        ser.serialize_bool("is_todo", self.is_todo);
        ser.serialize_bool("todo_due", self.todo_due);
        ser.serialize_bool("todo_completed", self.todo_completed);
        ser.serialize_str("source", &self.source);
        ser.serialize_str("source_application", &self.source_application);
        ser.serialize_str("application_data", &self.application_data);
        ser.serialize_i64("custom_order", self.custom_order);
        ser.serialize_datetime("user_created_time", self.user_created_time);
        ser.serialize_datetime("user_updated_time", self.user_updated_time);
        ser.serialize_str("encryption_cipher_text", &self.encryption_cipher_text);
        ser.serialize_bool("encryption_applied", self.encryption_applied);
        ser.serialize_bool("markup_language", self.markup_language);
        ser.serialize_bool("is_shared", self.is_shared);
        ser.serialize_str("share_id", &self.share_id);
        ser.serialize_opt_str("conflict_original_id", self.conflict_original_id.as_deref());
        ser.serialize_str("master_key_id", &self.master_key_id);
        ser.serialize_type("type_", ModelType::Note);
        Ok(ser)
    }
}
