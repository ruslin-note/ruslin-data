use std::hash::{Hash, Hasher};

use crate::{schema::deleted_items, DateTimeTimestamp, ModelType};
use diesel::prelude::*;

#[derive(Clone, Identifiable, Queryable, Insertable, Eq, Debug)]
#[diesel(primary_key(id))]
#[diesel(table_name = deleted_items)]
pub struct DeletedItem {
    pub id: i32,
    pub item_type: ModelType,
    pub item_id: String,
    pub deleted_time: DateTimeTimestamp,
}

impl DeletedItem {
    pub fn filepath(&self) -> String {
        format!("{}.md", self.item_id)
    }
}

impl Hash for DeletedItem {
    fn hash<H: Hasher>(&self, hasher: &mut H) {
        self.id.hash(hasher);
    }
}

impl PartialEq for DeletedItem {
    fn eq(&self, other: &DeletedItem) -> bool {
        self.id == other.id
    }
}

#[derive(Debug, Insertable)]
#[diesel(table_name = deleted_items)]
pub struct NewDeletedItem<'a> {
    pub item_type: ModelType,
    pub item_id: &'a str,
    pub deleted_time: DateTimeTimestamp,
}

impl<'a> NewDeletedItem<'a> {
    pub fn new(item_type: ModelType, item_id: &'a str) -> Self {
        Self {
            item_type,
            item_id,
            deleted_time: DateTimeTimestamp::now(),
        }
    }
}
