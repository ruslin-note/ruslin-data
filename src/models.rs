mod date_time;
mod deleted_item;
mod folder;
mod note;
mod resource;
mod setting;
mod status;
mod sync_item;
mod tag;

pub use date_time::*;
pub use deleted_item::{DeletedItem, NewDeletedItem};
use diesel::{
    backend::RawValue,
    deserialize::{self, FromSql},
    serialize::{self, IsNull, Output, ToSql},
    sql_types::Integer,
    sqlite::Sqlite,
    AsExpression, FromSqlRow,
};
pub use folder::Folder;
pub use note::{notes_fts, AbbrNote, Note, NoteFts};
pub use resource::Resource;
use serde_repr::{Deserialize_repr, Serialize_repr};
pub use setting::{NewSetting, Setting};
pub use status::Status;
pub use sync_item::{NewSyncItem, SyncItem, SyncTarget};
pub use tag::{NoteTag, Tag};

#[derive(
    Eq,
    PartialEq,
    Hash,
    Clone,
    Copy,
    Debug,
    Serialize_repr,
    Deserialize_repr,
    AsExpression,
    FromSqlRow,
)]
#[diesel(sql_type = Integer)]
#[repr(i32)]
pub enum ModelType {
    Note = 1,
    Folder = 2,
    // Setting = 3,
    Resource = 4,
    Tag = 5,
    NoteTag = 6,
    // Search = 7,
    // Alarm = 8,
    // MasterKey = 9,
    // ItemChange = 10,
    // NoteResource = 11,
    // ResourceLocalState = 12,
    // Revision = 13,
    // Migration = 14,
    // SmartFilter = 15,
    // Command = 16,
    Unsupported = -1,
}

impl From<i32> for ModelType {
    fn from(val: i32) -> Self {
        match val {
            1 => ModelType::Note,
            2 => ModelType::Folder,
            4 => ModelType::Resource,
            5 => ModelType::Tag,
            6 => ModelType::NoteTag,
            _ => ModelType::Unsupported,
        }
    }
}

impl FromSql<Integer, Sqlite> for ModelType {
    fn from_sql(bytes: RawValue<Sqlite>) -> deserialize::Result<Self> {
        match i32::from_sql(bytes)? {
            1 => Ok(ModelType::Note),
            2 => Ok(ModelType::Folder),
            4 => Ok(ModelType::Resource),
            5 => Ok(ModelType::Tag),
            6 => Ok(ModelType::NoteTag),
            x => Err(format!("Unrecognized variant {}", x).into()),
        }
    }
}

impl ToSql<Integer, Sqlite> for ModelType {
    fn to_sql<'b>(&'b self, out: &mut Output<'b, '_, Sqlite>) -> serialize::Result {
        out.set_value(*self as i32);
        Ok(IsNull::No)
    }
}

pub fn new_id() -> String {
    uuid::Uuid::now_v7().simple().to_string()
}
