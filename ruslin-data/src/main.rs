use ruslin_data::model;

use rusqlite::{Connection, Result};

#[derive(Debug)]
struct Note {
    id: String,
    parent_id: String,
    title: String,
}

fn main() -> Result<()> {
    let conn = Connection::open("./database.sqlite")?;
    let mut stmt = conn.prepare("SELECT * FROM notes")?;
    let note_iter = stmt.query_map([], |row| {
        Ok(Note {
            id: row.get(0)?,
            parent_id: row.get(1)?,
            title: row.get(2)?,
        })
    })?;

    for note in note_iter {
        let note = note.unwrap();
        println!("Found note {} {} {}", note.id, note.parent_id, note.title);
    }
    Ok(())
}
