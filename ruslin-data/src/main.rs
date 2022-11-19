use std::path::Path;

// use crate::{Dat}
use ruslin_data::{Database, DatabaseResult, Folder};

fn main() -> DatabaseResult<()> {
    let data_dir = Path::new("./");
    let db = Database::new(data_dir)?;
    let folder = Folder::new("folder1".to_string());
    db.replace_folder(&folder)?;
    let folders = db.load_folders()?;
    println!("{:?}", folders);
    Ok(())
}
