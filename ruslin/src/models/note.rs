use std::hash::{Hash, Hasher};

use crate::{schema::notes, DateTimeTimestamp};
use diesel::prelude::*;

use super::ids::{FolderID, NoteID};

#[derive(Clone, Identifiable, Insertable, Queryable, Eq, Debug)]
#[diesel(primary_key(id))]
#[diesel(table_name = notes)]
pub struct AbbrNote {
    pub id: NoteID,
    pub parent_id: FolderID,
    pub title: String,
    pub created_time: DateTimeTimestamp,
    pub updated_time: DateTimeTimestamp,
}

#[derive(Clone, Identifiable, Insertable, Queryable, Eq, Debug)]
#[diesel(primary_key(id))]
#[diesel(table_name = notes)]
pub struct Note {
    pub id: NoteID,
    pub parent_id: FolderID,
    pub title: String,
    pub body: String,
    pub created_time: DateTimeTimestamp,
    pub updated_time: DateTimeTimestamp,
}

impl Note {
    pub fn updated(&self) -> Self {
        let mut note = self.clone();
        note.updated_time = DateTimeTimestamp::now();
        note
    }
}

impl Hash for AbbrNote {
    fn hash<H: Hasher>(&self, hasher: &mut H) {
        self.id.hash(hasher);
    }
}

impl PartialEq for AbbrNote {
    fn eq(&self, other: &AbbrNote) -> bool {
        self.id == other.id
    }
}

impl Hash for Note {
    fn hash<H: Hasher>(&self, hasher: &mut H) {
        self.id.hash(hasher);
    }
}

impl PartialEq for Note {
    fn eq(&self, other: &Note) -> bool {
        self.id == other.id
    }
}
