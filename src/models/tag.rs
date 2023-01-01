use std::hash::{Hash, Hasher};

use crate::{
    new_id,
    schema::{note_tags, tags},
    sync::{DeserializeForSync, ForSyncSerializer, SerializeForSync, SyncResult},
    DateTimeTimestamp, ModelType,
};
use diesel::prelude::*;

#[derive(Clone, Identifiable, Insertable, Queryable, Eq, Debug)]
#[diesel(primary_key(id))]
#[diesel(table_name = tags)]
pub struct Tag {
    pub id: String,
    pub title: String,
    pub created_time: DateTimeTimestamp,
    pub updated_time: DateTimeTimestamp,
    pub user_created_time: DateTimeTimestamp,
    pub user_updated_time: DateTimeTimestamp,
    pub encryption_cipher_text: String,
    pub encryption_applied: bool,
    pub is_shared: bool,
    pub parent_id: Option<String>,
}

impl Tag {
    pub fn new(title: &str) -> Self {
        let dt = DateTimeTimestamp::now();
        Self {
            id: new_id(),
            title: title.to_string(),
            created_time: dt,
            updated_time: dt,
            user_created_time: dt,
            user_updated_time: dt,
            encryption_cipher_text: String::new(),
            encryption_applied: false,
            is_shared: false,
            parent_id: None,
        }
    }

    pub fn updated(&self) -> Self {
        let mut tag = self.clone();
        tag.updated_time = DateTimeTimestamp::now();
        tag
    }
}

impl Hash for Tag {
    fn hash<H: Hasher>(&self, hasher: &mut H) {
        self.id.hash(hasher);
    }
}

impl PartialEq for Tag {
    fn eq(&self, other: &Tag) -> bool {
        self.id == other.id
    }
}

#[derive(Clone, Identifiable, Insertable, Queryable, Eq, Debug)]
#[diesel(primary_key(id))]
#[diesel(table_name = note_tags)]
pub struct NoteTag {
    pub id: String,
    pub note_id: String,
    pub tag_id: String,
    pub created_time: DateTimeTimestamp,
    pub updated_time: DateTimeTimestamp,
    pub user_created_time: DateTimeTimestamp,
    pub user_updated_time: DateTimeTimestamp,
    pub encryption_cipher_text: String,
    pub encryption_applied: bool,
    pub is_shared: bool,
}

impl NoteTag {
    pub fn new(note_id: &str, tag_id: &str) -> Self {
        let dt = DateTimeTimestamp::now();
        Self {
            id: new_id(),
            note_id: note_id.to_string(),
            tag_id: tag_id.to_string(),
            created_time: dt,
            updated_time: dt,
            user_created_time: dt,
            user_updated_time: dt,
            encryption_cipher_text: String::new(),
            encryption_applied: false,
            is_shared: false,
        }
    }

    pub fn updated(&self) -> Self {
        let mut note_tag = self.clone();
        note_tag.updated_time = DateTimeTimestamp::now();
        note_tag
    }
}

impl Hash for NoteTag {
    fn hash<H: Hasher>(&self, hasher: &mut H) {
        self.id.hash(hasher);
    }
}

impl PartialEq for NoteTag {
    fn eq(&self, other: &NoteTag) -> bool {
        self.id == other.id
    }
}

impl SerializeForSync for Tag {
    fn serialize(&self) -> ForSyncSerializer {
        let mut ser = ForSyncSerializer::new(Some(&self.title), None);
        ser.serialize_str("id", &self.id);
        ser.serialize_datetime("created_time", self.created_time);
        ser.serialize_datetime("updated_time", self.updated_time);
        ser.serialize_datetime("user_created_time", self.user_created_time);
        ser.serialize_datetime("user_updated_time", self.user_updated_time);
        ser.serialize_str("encryption_cipher_text", &self.encryption_cipher_text);
        ser.serialize_bool("encryption_applied", self.encryption_applied);
        ser.serialize_bool("is_shared", self.is_shared);
        ser.serialize_opt_str("parent_id", self.parent_id.as_deref());
        ser.serialize_type("type_", ModelType::Tag);
        ser
    }
}

impl DeserializeForSync for Tag {
    fn dserialize(des: &crate::sync::ForSyncDeserializer) -> SyncResult<Self> {
        assert!(des.r#type == ModelType::Tag);
        Ok(Self {
            id: des.get_string("id")?,
            title: des.title.to_string(),
            created_time: des.get_date_time_timestamp("created_time")?,
            updated_time: des.get_date_time_timestamp("updated_time")?,
            user_created_time: des.get_date_time_timestamp("user_created_time")?,
            user_updated_time: des.get_date_time_timestamp("user_updated_time")?,
            encryption_cipher_text: des
                .get_opt_string("encryption_cipher_text")
                .unwrap_or_default(),
            encryption_applied: des.get_bool("encryption_applied")?,
            is_shared: des.get_bool("is_shared")?,
            parent_id: des.get_opt_string("parent_id"),
        })
    }
}

impl SerializeForSync for NoteTag {
    fn serialize(&self) -> ForSyncSerializer {
        let mut ser = ForSyncSerializer::new(None, None);
        ser.serialize_str("id", &self.id);
        ser.serialize_str("note_id", &self.note_id);
        ser.serialize_str("tag_id", &self.tag_id);
        ser.serialize_datetime("created_time", self.created_time);
        ser.serialize_datetime("updated_time", self.updated_time);
        ser.serialize_datetime("user_created_time", self.user_created_time);
        ser.serialize_datetime("user_updated_time", self.user_updated_time);
        ser.serialize_str("encryption_cipher_text", &self.encryption_cipher_text);
        ser.serialize_bool("encryption_applied", self.encryption_applied);
        ser.serialize_bool("is_shared", self.is_shared);
        ser.serialize_type("type_", ModelType::NoteTag);
        ser
    }
}

impl DeserializeForSync for NoteTag {
    fn dserialize(des: &crate::sync::ForSyncDeserializer) -> SyncResult<Self> {
        assert!(des.r#type == ModelType::NoteTag);
        Ok(Self {
            id: des.get_string("id")?,
            note_id: des.get_string("note_id")?,
            tag_id: des.get_string("tag_id")?,
            created_time: des.get_date_time_timestamp("created_time")?,
            updated_time: des.get_date_time_timestamp("updated_time")?,
            user_created_time: des.get_date_time_timestamp("user_created_time")?,
            user_updated_time: des.get_date_time_timestamp("user_updated_time")?,
            encryption_cipher_text: des
                .get_opt_string("encryption_cipher_text")
                .unwrap_or_default(),
            encryption_applied: des.get_bool("encryption_applied")?,
            is_shared: des.get_bool("is_shared")?,
        })
    }
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use crate::{
        models::tag::{NoteTag, Tag},
        sync::{DeserializeForSync, ForSyncDeserializer, SerializeForSync},
        DateTimeRFC333, DateTimeTimestamp,
    };

    #[test]
    fn test_serialize_and_dserialize_tag() {
        let dt = DateTimeRFC333::from_raw_str("2023-01-01T02:33:24.006Z");
        let dt: DateTimeTimestamp = dt.into();
        let tag = Tag {
            id: "e3ab346860af417fa7b891ea08d44682".to_string(),
            title: "Tag1".to_string(),
            created_time: dt,
            updated_time: dt,
            user_created_time: dt,
            user_updated_time: dt,
            encryption_cipher_text: String::new(),
            encryption_applied: false,
            is_shared: false,
            parent_id: None,
        };
        let binding = tag.serialize();
        let serialize_result = binding.as_str();
        let expected_str = "Tag1

id: e3ab346860af417fa7b891ea08d44682
created_time: 2023-01-01T02:33:24.006Z
updated_time: 2023-01-01T02:33:24.006Z
user_created_time: 2023-01-01T02:33:24.006Z
user_updated_time: 2023-01-01T02:33:24.006Z
encryption_cipher_text: 
encryption_applied: 0
is_shared: 0
parent_id: 
type_: 5";
        assert_eq!(expected_str, serialize_result);
        let des = ForSyncDeserializer::from_str(expected_str)
            .unwrap_or_else(|_| panic!("unwrap error in {}:{}", file!(), line!()));
        let des_tag = Tag::dserialize(&des)
            .unwrap_or_else(|_| panic!("unwrap error in {}:{}", file!(), line!()));
        assert_eq!(tag, des_tag);
    }

    #[test]
    fn test_serialize_and_dserialize_note_tag() {
        let dt = DateTimeRFC333::from_raw_str("2023-01-01T02:33:24.063Z");
        let dt: DateTimeTimestamp = dt.into();
        let note_tag = NoteTag {
            id: "d4221a8a9b1c4c75ac09f77537db72b6".to_string(),
            note_id: "dbefb5d892534f878196976368275557".to_string(),
            tag_id: "e3ab346860af417fa7b891ea08d44682".to_string(),
            created_time: dt,
            updated_time: dt,
            user_created_time: dt,
            user_updated_time: dt,
            encryption_cipher_text: String::new(),
            encryption_applied: false,
            is_shared: false,
        };
        let binding = note_tag.serialize();
        let serialize_result = binding.as_str();
        let expected_str = "id: d4221a8a9b1c4c75ac09f77537db72b6
note_id: dbefb5d892534f878196976368275557
tag_id: e3ab346860af417fa7b891ea08d44682
created_time: 2023-01-01T02:33:24.063Z
updated_time: 2023-01-01T02:33:24.063Z
user_created_time: 2023-01-01T02:33:24.063Z
user_updated_time: 2023-01-01T02:33:24.063Z
encryption_cipher_text: 
encryption_applied: 0
is_shared: 0
type_: 6";
        assert_eq!(expected_str, serialize_result);
        let des = ForSyncDeserializer::from_str(expected_str)
            .unwrap_or_else(|_| panic!("unwrap error in {}:{}", file!(), line!()));
        let des_note_tag = NoteTag::dserialize(&des)
            .unwrap_or_else(|_| panic!("unwrap error in {}:{}", file!(), line!()));
        assert_eq!(note_tag, des_note_tag);
    }
}
