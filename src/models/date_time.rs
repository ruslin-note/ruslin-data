use chrono::{DateTime, NaiveDateTime, TimeZone, Utc};
use diesel::{
    backend::RawValue,
    deserialize::{self, FromSql},
    serialize::{self, IsNull, Output, ToSql},
    sql_types::BigInt,
    sqlite::Sqlite,
    AsExpression, FromSqlRow,
};
use serde::{Deserialize, Deserializer, Serialize};

#[derive(
    Eq,
    PartialEq,
    Hash,
    Clone,
    Copy,
    Debug,
    AsExpression,
    FromSqlRow,
    PartialOrd,
    Ord,
    Serialize,
    Deserialize,
)]
#[diesel(sql_type = BigInt)]
pub struct DateTimeTimestamp(i64);

impl DateTimeTimestamp {
    pub fn now() -> Self {
        Self(Utc::now().naive_utc().timestamp_millis())
    }

    pub fn from_timestamp_millis(t: i64) -> Self {
        Self(t)
    }

    pub fn timestamp_millis(&self) -> i64 {
        self.0
    }

    pub fn format_ymd_hms(&self) -> String {
        let utc = NaiveDateTime::from_timestamp_millis(self.0).unwrap();
        chrono::Local
            .from_utc_datetime(&utc)
            .format("%Y-%m-%d %H:%M:%S")
            .to_string()
    }
}

impl FromSql<BigInt, Sqlite> for DateTimeTimestamp {
    fn from_sql(bytes: RawValue<Sqlite>) -> deserialize::Result<Self> {
        let timestamp: i64 = FromSql::<BigInt, Sqlite>::from_sql(bytes)?;
        Ok(DateTimeTimestamp(timestamp))
    }
}

impl ToSql<BigInt, Sqlite> for DateTimeTimestamp {
    fn to_sql<'b>(&'b self, out: &mut Output<'b, '_, Sqlite>) -> serialize::Result {
        ToSql::<BigInt, Sqlite>::to_sql(&self.0, out)?;
        Ok(IsNull::No)
    }
}

impl From<DateTimeRFC333> for DateTimeTimestamp {
    fn from(dt: DateTimeRFC333) -> Self {
        Self(dt.0.timestamp_millis())
    }
}

#[derive(Eq, PartialEq, Hash, Clone, Debug)]
pub struct DateTimeRFC333(DateTime<Utc>);

impl DateTimeRFC333 {
    pub fn now() -> Self {
        Self(Utc::now())
    }

    pub fn from_timestamp_millis(t: i64) -> Self {
        let t = NaiveDateTime::from_timestamp_millis(t).unwrap();
        let time = chrono::DateTime::<Utc>::from_utc(t, Utc);
        Self(time)
    }

    pub fn timestamp_millis(&self) -> i64 {
        self.0.timestamp_millis()
    }

    pub fn as_string(&self) -> String {
        self.0.to_rfc3339_opts(chrono::SecondsFormat::Millis, true)
    }

    pub fn from_raw_str(s: &str) -> Self {
        let dt = chrono::DateTime::parse_from_rfc3339(s).unwrap();
        let dt = dt.with_timezone(&Utc);
        Self(dt)
    }
}

impl From<DateTimeTimestamp> for DateTimeRFC333 {
    fn from(dt: DateTimeTimestamp) -> Self {
        DateTimeRFC333::from_timestamp_millis(dt.0)
    }
}

impl Serialize for DateTimeRFC333 {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let time = self.0.to_rfc3339_opts(chrono::SecondsFormat::Millis, true);
        serializer.serialize_str(&time)
    }
}

impl<'de> Deserialize<'de> for DateTimeRFC333 {
    fn deserialize<D>(deserializer: D) -> Result<DateTimeRFC333, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        let dt = chrono::DateTime::parse_from_rfc3339(&s).unwrap();
        let dt = dt.with_timezone(&Utc);
        Ok(DateTimeRFC333(dt))
    }
}

#[cfg(test)]
mod tests {
    use crate::{DateTimeRFC333, DateTimeTimestamp};

    #[test]
    fn test_date_time_serialize() {
        let dt = DateTimeRFC333::from_timestamp_millis(1668922083344);
        let serialized_str = serde_json::to_string(&dt).unwrap();
        assert_eq!(r#""2022-11-20T05:28:03.344Z""#, serialized_str);
    }

    #[test]
    fn test_date_time_deserialize() {
        let dt: DateTimeRFC333 = serde_json::from_str(r#""2022-11-20T05:28:03.344Z""#).unwrap();
        assert_eq!(1668922083344, dt.timestamp_millis());
    }

    #[test]
    fn test_rfc333_with_timestamp() {
        let dt_timestamp = DateTimeTimestamp::from_timestamp_millis(1668922083344);
        let dt_rfc333: DateTimeRFC333 = dt_timestamp.into();
        let dt_timestamp_2: DateTimeTimestamp = dt_rfc333.into();
        assert_eq!(dt_timestamp, dt_timestamp_2);
    }
}
