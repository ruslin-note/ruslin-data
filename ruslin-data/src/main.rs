use ruslin_data::{model, Database, Folder, Result};

fn main() -> Result<()> {
    let db = Database::open("test.sqlite")?;
    let mut folder = Folder::new("title".to_string(), None, "icon".to_string());
    folder.save(&db)?;
    assert_eq!(folder, Folder::query_one_by_id(&db, folder.get_id())?);
    folder.title = "title2".to_string();
    folder.save(&db)?;
    assert_eq!(folder, Folder::query_one_by_id(&db, folder.get_id())?);
    folder.delete(&db)?;
    Ok(())
}
