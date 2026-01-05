use rusqlite::{Connection, Error as SqliteError, params};

pub struct DeckWithCollection {
    pub id: i64,
    pub collection_id: i64,
    pub name: String,
    pub created_at: String,
    pub collection_name: String,
    pub game_name: String,
    pub game_image_url: Option<String>,
    pub card_count: i64,
}

#[derive(Debug)]
pub enum CreateDeckError {
    CollectionNotFound,
    NotOwner,
    DatabaseError,
}

#[derive(Debug)]
pub enum GetDeckError {
    NotFound,
    NotOwner,
    DatabaseError,
}

pub fn init_table(conn: &Connection) -> Result<(), SqliteError> {
    conn.execute(
        "CREATE TABLE IF NOT EXISTS decks (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            collection_id INTEGER NOT NULL,
            name TEXT NOT NULL,
            created_at TEXT NOT NULL,
            FOREIGN KEY (collection_id) REFERENCES collections(id)
        )",
        [],
    )?;
    Ok(())
}

pub fn create(
    conn: &Connection,
    user_id: i64,
    collection_id: i64,
    name: &str,
) -> Result<i64, CreateDeckError> {
    // Check if collection exists and get its owner
    let collection_check: Result<i64, _> = conn.query_row(
        "SELECT user_id FROM collections WHERE id = ?1",
        params![collection_id],
        |row| row.get(0),
    );

    match collection_check {
        Ok(owner_id) => {
            if owner_id != user_id {
                return Err(CreateDeckError::NotOwner);
            }
        }
        Err(rusqlite::Error::QueryReturnedNoRows) => {
            return Err(CreateDeckError::CollectionNotFound);
        }
        Err(_) => {
            return Err(CreateDeckError::DatabaseError);
        }
    }

    let created_at = chrono::Utc::now().to_rfc3339();

    conn.execute(
        "INSERT INTO decks (collection_id, name, created_at) VALUES (?1, ?2, ?3)",
        params![collection_id, name, created_at],
    )
    .map_err(|_| CreateDeckError::DatabaseError)?;

    Ok(conn.last_insert_rowid())
}

pub fn list_by_user(
    conn: &Connection,
    user_id: i64,
) -> Result<Vec<DeckWithCollection>, SqliteError> {
    let mut stmt = conn.prepare(
        "SELECT d.id, d.collection_id, d.name, d.created_at, c.name, g.name, g.image_url
         FROM decks d
         JOIN collections c ON d.collection_id = c.id
         JOIN games g ON c.game_id = g.id
         WHERE c.user_id = ?1
         ORDER BY d.created_at DESC",
    )?;

    let decks = stmt
        .query_map(params![user_id], |row| {
            Ok(DeckWithCollection {
                id: row.get(0)?,
                collection_id: row.get(1)?,
                name: row.get(2)?,
                created_at: row.get(3)?,
                collection_name: row.get(4)?,
                game_name: row.get(5)?,
                game_image_url: row.get(6)?,
                card_count: 0, // Always 0 for now
            })
        })?
        .collect::<Result<Vec<_>, _>>()?;

    Ok(decks)
}

pub fn get(
    conn: &Connection,
    user_id: i64,
    deck_id: i64,
) -> Result<DeckWithCollection, GetDeckError> {
    let result = conn.query_row(
        "SELECT d.id, d.collection_id, d.name, d.created_at, c.name, g.name, g.image_url, c.user_id
         FROM decks d
         JOIN collections c ON d.collection_id = c.id
         JOIN games g ON c.game_id = g.id
         WHERE d.id = ?1",
        params![deck_id],
        |row| {
            Ok((
                DeckWithCollection {
                    id: row.get(0)?,
                    collection_id: row.get(1)?,
                    name: row.get(2)?,
                    created_at: row.get(3)?,
                    collection_name: row.get(4)?,
                    game_name: row.get(5)?,
                    game_image_url: row.get(6)?,
                    card_count: 0, // Always 0 for now
                },
                row.get::<_, i64>(7)?, // owner_id
            ))
        },
    );

    match result {
        Ok((deck, owner_id)) => {
            if owner_id != user_id {
                Err(GetDeckError::NotOwner)
            } else {
                Ok(deck)
            }
        }
        Err(rusqlite::Error::QueryReturnedNoRows) => Err(GetDeckError::NotFound),
        Err(_) => Err(GetDeckError::DatabaseError),
    }
}
