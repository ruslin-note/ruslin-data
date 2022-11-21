use chrono::{NaiveDateTime, Utc};
use diesel::{
    backend::RawValue,
    deserialize::{self, FromSql},
    serialize::{self, IsNull, Output, ToSql},
    sql_types::BigInt,
    sqlite::Sqlite,
    AsExpression, FromSqlRow,
};
use serde::{Deserialize, Deserializer, Serialize};

#[derive(Eq, PartialEq, Hash, Clone, Copy, Debug, AsExpression, FromSqlRow, PartialOrd, Ord)]
#[diesel(sql_type = BigInt)]
pub struct DateTime(i64);

impl DateTime {
    pub fn now() -> Self {
        Self(Utc::now().naive_utc().timestamp_millis())
    }

    pub fn from_timestamp_millis(t: i64) -> Self {
        Self(t)
    }

    pub fn timestamp_millis(&self) -> i64 {
        self.0
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

impl Serialize for DateTime {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let t = NaiveDateTime::from_timestamp_millis(self.0).unwrap();
        let time = chrono::DateTime::<Utc>::from_utc(t, Utc)
            .to_rfc3339_opts(chrono::SecondsFormat::Millis, true);
        serializer.serialize_str(&time)
    }
}

impl<'de> Deserialize<'de> for DateTime {
    fn deserialize<D>(deserializer: D) -> Result<DateTime, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        let dt = chrono::DateTime::parse_from_rfc3339(&s).unwrap();
        Ok(DateTime::from_timestamp_millis(dt.timestamp_millis()))
    }
}

#[cfg(test)]
mod tests {
    use crate::DateTime;

    #[test]
    fn test_date_time_serialize() {
        let dt = DateTime::from_timestamp_millis(1668922083344);
        let serialized_str = serde_json::to_string(&dt).unwrap();
        assert_eq!(r#""2022-11-20T05:28:03.344Z""#, serialized_str);
    }

    #[test]
    fn test_date_time_deserialize() {
        let dt: DateTime = serde_json::from_str(r#""2022-11-20T05:28:03.344Z""#).unwrap();
        assert_eq!(1668922083344, dt.timestamp_millis());
    }
}
