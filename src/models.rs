mod date_time;
mod folder;
mod ids;
mod note;
mod sync_item;

pub use date_time::*;
use diesel::{
    backend::RawValue,
    deserialize::{self, FromSql},
    serialize::{self, IsNull, Output, ToSql},
    sql_types::Integer,
    sqlite::Sqlite,
    AsExpression, FromSqlRow,
};
pub use folder::Folder;
pub use ids::*;
pub use note::{AbbrNote, Note};
use serde_repr::{Deserialize_repr, Serialize_repr};
pub use sync_item::{NewSyncItem, SyncItem, SyncTarget};

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
    // Resource = 4,
    // Tag = 5,
    // NoteTag = 6,
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
}

impl FromSql<Integer, Sqlite> for ModelType {
    fn from_sql(bytes: RawValue<Sqlite>) -> deserialize::Result<Self> {
        match i32::from_sql(bytes)? {
            1 => Ok(ModelType::Note),
            2 => Ok(ModelType::Folder),
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
