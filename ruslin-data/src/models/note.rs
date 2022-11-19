use std::hash::{Hash, Hasher};

use crate::{schema::notes, DateTime};
use diesel::prelude::*;

use super::ids::{FolderID, NoteID};

#[derive(Clone, Identifiable, Insertable, Queryable, Eq, Debug)]
#[diesel(primary_key(id))]
#[diesel(table_name = notes)]
pub struct AbbrNote {
    pub id: NoteID,
    pub parent_id: FolderID,
    pub title: String,
    pub created_time: DateTime,
    pub updated_time: DateTime,
}

#[derive(Clone, Identifiable, Insertable, Queryable, Eq, Debug)]
#[diesel(primary_key(id))]
#[diesel(table_name = notes)]
pub struct Note {
    pub id: NoteID,
    pub parent_id: FolderID,
    pub title: String,
    pub body: String,
    pub created_time: DateTime,
    pub updated_time: DateTime,
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
