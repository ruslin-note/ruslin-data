use std::hash::{Hash, Hasher};

use crate::{schema::sync_items, DateTime, ModelType};
use diesel::{
    backend::RawValue,
    deserialize::{self, FromSql},
    prelude::*,
    serialize::{self, IsNull, Output, ToSql},
    sql_types::Integer,
    sqlite::Sqlite,
    AsExpression, FromSqlRow,
};
use serde_repr::{Deserialize_repr, Serialize_repr};

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
pub enum SyncTarget {
    // None = 0,
    // Memory = 1,
    FileSystem = 2,
    // Nextcloud = 5,
    // WebDav = 6,
    JoplinServer = 9,
}

impl SyncTarget {
    pub fn name(&self) -> &'static str {
        match self {
            // SyncTarget::None => "None",
            // SyncTarget::Memory => "Memory",
            SyncTarget::FileSystem => "FileSystem",
            // SyncTarget::Nextcloud => "Nextcloud",
            // SyncTarget::WebDav => "WebDAV",
            SyncTarget::JoplinServer => "JoplinServer",
        }
    }
}

// https://docs.diesel.rs/master/diesel/deserialize/trait.FromSql.html
impl FromSql<Integer, Sqlite> for SyncTarget {
    fn from_sql(bytes: RawValue<Sqlite>) -> deserialize::Result<Self> {
        match i32::from_sql(bytes)? {
            2 => Ok(SyncTarget::FileSystem),
            9 => Ok(SyncTarget::JoplinServer),
            x => Err(format!("Unrecognized variant {}", x).into()),
        }
    }
}

// https://docs.diesel.rs/master/diesel/serialize/trait.ToSql.html
impl ToSql<Integer, Sqlite> for SyncTarget {
    fn to_sql<'b>(&'b self, out: &mut Output<'b, '_, Sqlite>) -> serialize::Result {
        out.set_value(*self as i32);
        Ok(IsNull::No)
    }
}

#[derive(Clone, Identifiable, Queryable, Eq, Debug)]
#[diesel(primary_key(id))]
#[diesel(table_name = sync_items)]
pub struct SyncItem {
    pub id: i64,
    pub sync_target: SyncTarget,
    pub sync_time: DateTime,
    pub item_type: ModelType,
    pub item_id: String,
}

impl Hash for SyncItem {
    fn hash<H: Hasher>(&self, hasher: &mut H) {
        self.id.hash(hasher);
    }
}

impl PartialEq for SyncItem {
    fn eq(&self, other: &SyncItem) -> bool {
        self.id == other.id
    }
}

#[derive(Debug, Insertable)]
#[diesel(table_name = sync_items)]
pub struct NewSyncItem<'a> {
    pub sync_target: SyncTarget,
    pub item_type: ModelType,
    pub item_id: &'a str,
}
