use diesel::{
    backend::RawValue,
    deserialize::{self, FromSql},
    serialize::{self, IsNull, Output, ToSql},
    sql_types::Text,
    sqlite::Sqlite,
    AsExpression, FromSqlRow,
};
use serde::{Deserialize, Serialize};

fn get_id() -> String {
    uuid::Uuid::now_v7().simple().to_string()
}

#[derive(
    Eq, PartialEq, Hash, Clone, Debug, Serialize, Deserialize, AsExpression, FromSqlRow, Default,
)]
#[diesel(sql_type = Text)]
pub struct FolderID(String);

impl FolderID {
    pub fn new() -> Self {
        Self(get_id())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl FromSql<Text, Sqlite> for FolderID {
    fn from_sql(bytes: RawValue<Sqlite>) -> deserialize::Result<Self> {
        let bytes = <Vec<u8>>::from_sql(bytes)?;
        let string = String::from_utf8(bytes)?;
        Ok(FolderID(string))
    }
}

impl ToSql<Text, Sqlite> for FolderID {
    fn to_sql<'b>(&'b self, out: &mut Output<'b, '_, Sqlite>) -> serialize::Result {
        out.set_value(self.as_str());
        Ok(IsNull::No)
    }
}

#[derive(
    Eq, PartialEq, Hash, Clone, Debug, Serialize, Deserialize, AsExpression, FromSqlRow, Default,
)]
#[diesel(sql_type = Text)]
pub struct NoteID(String);

impl NoteID {
    pub fn new() -> Self {
        Self(get_id())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl FromSql<Text, Sqlite> for NoteID {
    fn from_sql(bytes: RawValue<Sqlite>) -> deserialize::Result<Self> {
        let bytes = <Vec<u8>>::from_sql(bytes)?;
        let string = String::from_utf8(bytes)?;
        Ok(NoteID(string))
    }
}

impl ToSql<Text, Sqlite> for NoteID {
    fn to_sql<'b>(&'b self, out: &mut Output<'b, '_, Sqlite>) -> serialize::Result {
        out.set_value(self.as_str());
        Ok(IsNull::No)
    }
}
