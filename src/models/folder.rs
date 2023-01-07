use std::hash::{Hash, Hasher};

use crate::{
    new_id,
    schema::folders,
    sync::{DeserializeForSync, ForSyncSerializer, SerializeForSync, SyncResult},
    DateTimeTimestamp, ModelType,
};
use diesel::prelude::*;

// #[repr(i32)]
// enum FolderIconType {
//     Emoji = 1,
//     DataUrl = 2,
//     FontAwesome = 3,
// }

// struct FolderIcon {
//     r#type: FolderIconType,
//     emoji: String,
//     name: String,
//     dataUrl: String,
// }

pub type SelectionType = (
    folders::columns::id,
    folders::columns::title,
    folders::columns::created_time,
    folders::columns::updated_time,
    folders::columns::user_created_time,
    folders::columns::user_updated_time,
    folders::columns::encryption_cipher_text,
    folders::columns::encryption_applied,
    folders::columns::parent_id,
    folders::columns::is_shared,
    folders::columns::share_id,
    folders::columns::master_key_id,
    folders::columns::icon,
);

#[derive(Clone, Identifiable, Insertable, Queryable, Eq, Debug)]
#[diesel(primary_key(id))]
#[diesel(table_name = folders)]
pub struct Folder {
    pub id: String,
    pub title: String,
    pub created_time: DateTimeTimestamp,
    pub updated_time: DateTimeTimestamp,
    pub user_created_time: DateTimeTimestamp,
    pub user_updated_time: DateTimeTimestamp,
    pub encryption_cipher_text: String,
    pub encryption_applied: bool,
    pub parent_id: Option<String>,
    pub is_shared: bool,
    pub share_id: String,
    pub master_key_id: String,
    pub icon: String,
}

impl Folder {
    pub fn new(title: impl Into<String>, parent_id: Option<String>) -> Self {
        let time = DateTimeTimestamp::now();
        Self {
            id: new_id(),
            title: title.into(),
            created_time: time,
            updated_time: time,
            user_created_time: time,
            user_updated_time: time,
            encryption_cipher_text: String::new(),
            encryption_applied: false,
            parent_id,
            is_shared: false,
            share_id: String::new(),
            master_key_id: String::new(),
            icon: String::new(),
        }
    }

    pub fn new_with_parent(title: impl Into<String>, parent_id: impl Into<String>) -> Self {
        Self::new(title, Some(parent_id.into()))
    }

    pub fn new_root(title: impl Into<String>) -> Self {
        Self::new(title, None)
    }

    pub fn updated(&self) -> Self {
        let mut folder = self.clone();
        folder.updated_time = DateTimeTimestamp::now();
        folder
    }

    pub fn get_title(&self) -> &str {
        &self.title
    }

    pub fn set_title(&mut self, title: &str) {
        self.title = title.replace('\n', " ");
    }

    pub fn md_file_path(&self) -> String {
        format!("{}.md", self.id.as_str())
    }

    pub const SELECTION: SelectionType = (
        folders::id,
        folders::title,
        folders::created_time,
        folders::updated_time,
        folders::user_created_time,
        folders::user_updated_time,
        folders::encryption_cipher_text,
        folders::encryption_applied,
        folders::parent_id,
        folders::is_shared,
        folders::share_id,
        folders::master_key_id,
        folders::icon,
    );
}

impl Hash for Folder {
    fn hash<H: Hasher>(&self, hasher: &mut H) {
        self.id.hash(hasher);
    }
}

impl PartialEq for Folder {
    fn eq(&self, other: &Folder) -> bool {
        self.id == other.id
    }
}

impl SerializeForSync for Folder {
    fn serialize(&self) -> ForSyncSerializer {
        let mut ser = ForSyncSerializer::new(Some(&self.title), None);
        ser.serialize_str("id", self.id.as_str());
        ser.serialize_datetime("created_time", self.created_time);
        ser.serialize_datetime("updated_time", self.updated_time);
        ser.serialize_datetime("user_created_time", self.user_created_time);
        ser.serialize_datetime("user_updated_time", self.user_updated_time);
        ser.serialize_str("encryption_cipher_text", &self.encryption_cipher_text);
        ser.serialize_bool("encryption_applied", self.encryption_applied);
        ser.serialize_opt_str("parent_id", self.parent_id.as_deref());
        ser.serialize_bool("is_shared", self.is_shared);
        ser.serialize_str("share_id", &self.share_id);
        ser.serialize_str("master_key_id", &self.master_key_id);
        ser.serialize_str("icon", &self.icon);
        ser.serialize_type("type_", ModelType::Folder);
        ser
    }
}

impl DeserializeForSync for Folder {
    fn dserialize(des: &crate::sync::ForSyncDeserializer) -> SyncResult<Self> {
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
            parent_id: des.get_opt_string("parent_id"),
            is_shared: des.get_bool("is_shared")?,
            share_id: des.get_opt_string("share_id").unwrap_or_default(),
            master_key_id: des.get_opt_string("share_id").unwrap_or_default(),
            icon: des.get_opt_string("share_id").unwrap_or_default(),
        })
    }
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use crate::{
        sync::{DeserializeForSync, ForSyncDeserializer, SerializeForSync},
        DateTimeRFC333, DateTimeTimestamp, Folder,
    };

    #[test]
    fn test_serialize_and_dserialize_folder() {
        let dt = DateTimeRFC333::from_raw_str("2022-11-20T05:27:50.593Z");
        let dt: DateTimeTimestamp = dt.into();
        let folder = Folder {
            id: "fd7d741357e2451283166354c512df3b".to_string(),
            title: "Folder1".to_string(),
            created_time: dt,
            updated_time: dt,
            user_created_time: dt,
            user_updated_time: dt,
            encryption_cipher_text: String::new(),
            encryption_applied: false,
            parent_id: None,
            is_shared: false,
            share_id: "".to_string(),
            master_key_id: "".to_string(),
            icon: "".to_string(),
        };
        let binding = folder.serialize();
        let serialize_result = binding.as_str();
        let expected_str = "Folder1

id: fd7d741357e2451283166354c512df3b
created_time: 2022-11-20T05:27:50.593Z
updated_time: 2022-11-20T05:27:50.593Z
user_created_time: 2022-11-20T05:27:50.593Z
user_updated_time: 2022-11-20T05:27:50.593Z
encryption_cipher_text: 
encryption_applied: 0
parent_id: 
is_shared: 0
share_id: 
master_key_id: 
icon: 
type_: 2";
        assert_eq!(expected_str, serialize_result);
        let des = ForSyncDeserializer::from_str(expected_str)
            .unwrap_or_else(|_| panic!("unwrap error in {}:{}", file!(), line!()));
        let des_folder = Folder::dserialize(&des)
            .unwrap_or_else(|_| panic!("unwrap error in {}:{}", file!(), line!()));
        assert_eq!(folder, des_folder);
    }
}
