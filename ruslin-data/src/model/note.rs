pub struct Note {
    pub id: String,
    pub parent_id: String, // note_id
    pub title: String,
    pub body: String,
    pub created_time: i64,
    pub updated_time: i64,
    pub is_conflict: bool,
    pub latitude: Option<f64>,
    pub longitude: Option<f64>,
    pub altitude: Option<f64>,
    pub author: Option<String>,
    pub source_url: Option<String>,
    pub is_todo: bool,
    pub todo_due: bool,
    pub todo_completed: bool,
    pub source: String,             // app enum?
    pub source_application: String, // app bundle id ?
    pub application_data: Option<String>,
    pub order: Option<i64>,     // ?
    pub user_created_time: i64, // ?
    pub user_updated_time: i64, // ?
    pub encryption_cipher_text: Option<String>,
    pub encryption_applied: Option<bool>,
    pub markup_language: bool, // true
    // pub is_shared: bool,
    pub share_id: Option<String>,
    pub conflict_original_id: Option<String>,
    pub master_key_id: Option<String>, // ?
                                       // pub type_: i64, ?
}
