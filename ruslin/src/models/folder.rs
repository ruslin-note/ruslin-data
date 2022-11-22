use std::hash::{Hash, Hasher};

use crate::{schema::folders, DateTimeTimestamp};
use diesel::prelude::*;

use super::ids::FolderID;

#[derive(Clone, Identifiable, Insertable, Queryable, Eq, Debug)]
#[diesel(primary_key(id))]
#[diesel(table_name = folders)]
pub struct Folder {
    pub id: FolderID,
    pub title: String,
    pub created_time: DateTimeTimestamp,
    pub updated_time: DateTimeTimestamp,
}

impl Folder {
    pub fn new(title: String) -> Self {
        let time = DateTimeTimestamp::now();
        Self {
            id: FolderID::new(),
            title,
            created_time: time,
            updated_time: time,
        }
    }

    pub fn updated(&self) -> Self {
        let mut folder = self.clone();
        folder.updated_time = DateTimeTimestamp::now();
        folder
    }
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
