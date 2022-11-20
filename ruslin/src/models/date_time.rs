use chrono::Utc;
use diesel::{
    backend::RawValue,
    deserialize::{self, FromSql},
    serialize::{self, IsNull, Output, ToSql},
    sql_types::BigInt,
    sqlite::Sqlite,
    AsExpression, FromSqlRow,
};
use serde::{Deserialize, Serialize};

#[derive(
    Eq,
    PartialEq,
    Hash,
    Clone,
    Copy,
    Debug,
    Serialize,
    Deserialize,
    AsExpression,
    FromSqlRow,
    PartialOrd,
    Ord,
)]
#[diesel(sql_type = BigInt)]
pub struct DateTime(i64);

impl DateTime {
    pub fn now() -> Self {
        Self(Utc::now().naive_utc().timestamp_millis())
    }
}

impl FromSql<BigInt, Sqlite> for DateTime {
    fn from_sql(bytes: RawValue<Sqlite>) -> deserialize::Result<Self> {
        let timestamp: i64 = FromSql::<BigInt, Sqlite>::from_sql(bytes)?;
        Ok(DateTime(timestamp))
    }
}

impl ToSql<BigInt, Sqlite> for DateTime {
    fn to_sql<'b>(&'b self, out: &mut Output<'b, '_, Sqlite>) -> serialize::Result {
        ToSql::<BigInt, Sqlite>::to_sql(&self.0, out)?;
        Ok(IsNull::No)
    }
}
