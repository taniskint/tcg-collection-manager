use std::collections::HashMap;

use rusqlite::{Connection, Error as SqliteError, params};

pub struct Card {
    pub id: i64,
    pub name: String,
    pub collector_number: String,
    pub image_url: Option<String>,
    pub attributes: HashMap<String, String>,
}

#[derive(Debug)]
pub enum CreateCardError {
    SetNotFound,
    DatabaseError,
}

pub struct CreateCardInput<'a> {
    pub name: &'a str,
    pub collector_number: &'a str,
    pub image_url: Option<&'a str>,
    pub attributes: &'a HashMap<String, String>,
}

fn serialize_attributes(attrs: &HashMap<String, String>) -> Option<String> {
    if attrs.is_empty() {
        None
    } else {
        Some(serde_json::to_string(attrs).unwrap())
    }
}

fn deserialize_attributes(json: Option<String>) -> HashMap<String, String> {
    json.and_then(|s| serde_json::from_str(&s).ok())
        .unwrap_or_default()
}

pub fn create_many(
    conn: &Connection,
    set_id: i64,
    cards: &[CreateCardInput],
) -> Result<Vec<i64>, CreateCardError> {
    // Verify the set exists
    let set_exists: bool = conn
        .query_row(
            "SELECT EXISTS(SELECT 1 FROM sets WHERE id = ?1)",
            params![set_id],
            |row| row.get(0),
        )
        .unwrap_or(false);

    if !set_exists {
        return Err(CreateCardError::SetNotFound);
    }

    let mut ids = Vec::with_capacity(cards.len());
    for card in cards {
        let attrs_json = serialize_attributes(card.attributes);
        conn.execute(
            "INSERT INTO cards (set_id, name, collector_number, image_url, attributes) VALUES (?1, ?2, ?3, ?4, ?5)",
            params![set_id, card.name, card.collector_number, card.image_url, attrs_json],
        )
        .map_err(|_| CreateCardError::DatabaseError)?;

        ids.push(conn.last_insert_rowid());
    }

    Ok(ids)
}

pub fn get(conn: &Connection, set_id: i64, card_id: i64) -> Result<Option<Card>, SqliteError> {
    let mut stmt =
        conn.prepare("SELECT id, name, collector_number, image_url, attributes FROM cards WHERE set_id = ?1 AND id = ?2")?;

    let mut rows = stmt.query(params![set_id, card_id])?;
    match rows.next()? {
        Some(row) => Ok(Some(Card {
            id: row.get(0)?,
            name: row.get(1)?,
            collector_number: row.get(2)?,
            image_url: row.get(3)?,
            attributes: deserialize_attributes(row.get(4)?),
        })),
        None => Ok(None),
    }
}

pub fn list(conn: &Connection, set_id: i64) -> Result<Vec<Card>, SqliteError> {
    let mut stmt = conn.prepare(
        "SELECT id, name, collector_number, image_url, attributes FROM cards WHERE set_id = ?1",
    )?;
    let mut rows = stmt.query(params![set_id])?;
    let mut cards = Vec::new();

    while let Some(row) = rows.next()? {
        cards.push(Card {
            id: row.get(0)?,
            name: row.get(1)?,
            collector_number: row.get(2)?,
            image_url: row.get(3)?,
            attributes: deserialize_attributes(row.get(4)?),
        });
    }

    Ok(cards)
}

pub fn init_table(conn: &Connection) -> Result<(), SqliteError> {
    conn.execute(
        "CREATE TABLE IF NOT EXISTS cards (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            set_id INTEGER NOT NULL,
            name TEXT NOT NULL,
            collector_number TEXT NOT NULL,
            image_url TEXT,
            attributes TEXT,
            FOREIGN KEY (set_id) REFERENCES sets(id)
        )",
        [],
    )?;
    Ok(())
}
