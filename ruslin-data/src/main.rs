use ruslin_data::{Database, Folder, Note, Result};

fn main() -> Result<()> {
    let db = Database::open("test.sqlite")?;
    let mut folder1 = Folder::new("folder1".to_string(), None, "icon".to_string());
    folder1.save(&db)?;
    let mut folder2 = Folder::new("folder2".to_string(), None, "icon".to_string());
    folder2.save(&db)?;

    let mut note1 = Note::new(
        "title1".to_string(),
        "body1".to_string(),
        folder1.get_id().to_string(),
    );
    note1.save(&db)?;
    let mut note2 = Note::new(
        "title2".to_string(),
        "body2".to_string(),
        folder2.get_id().to_string(),
    );
    note2.save(&db)?;
    let abbr_notes = Note::query_abbr_notes(&db, None)?;
    for abbr_note in abbr_notes {
        println!("{:?}", abbr_note);
    }
    Ok(())
}
