use rusqlite::{Connection, Error as SqliteError, params};

pub struct Set {
    pub id: i64,
    pub name: String,
    pub image_url: Option<String>,
}

#[derive(Debug)]
pub enum CreateSetError {
    GameNotFound,
    NameExists,
    DatabaseError,
}

pub fn create(
    conn: &Connection,
    game_id: i64,
    name: &str,
    image_url: Option<&str>,
) -> Result<i64, CreateSetError> {
    // Verify the game exists
    let game_exists: bool = conn
        .query_row(
            "SELECT EXISTS(SELECT 1 FROM games WHERE id = ?1)",
            params![game_id],
            |row| row.get(0),
        )
        .unwrap_or(false);

    if !game_exists {
        return Err(CreateSetError::GameNotFound);
    }

    conn.execute(
        "INSERT INTO sets (game_id, name, image_url) VALUES (?1, ?2, ?3)",
        params![game_id, name, image_url],
    )
    .map_err(|e| {
        if let SqliteError::SqliteFailure(err, _) = &e
            && err.extended_code == 2067
        {
            return CreateSetError::NameExists;
        }
        CreateSetError::DatabaseError
    })?;

    Ok(conn.last_insert_rowid())
}

// TODO: Add card_count field when that feature is implemented
pub fn get(conn: &Connection, game_id: i64, set_id: i64) -> Result<Option<Set>, SqliteError> {
    let mut stmt =
        conn.prepare("SELECT id, name, image_url FROM sets WHERE game_id = ?1 AND id = ?2")?;

    let mut rows = stmt.query(params![game_id, set_id])?;
    match rows.next()? {
        Some(row) => Ok(Some(Set {
            id: row.get(0)?,
            name: row.get(1)?,
            image_url: row.get(2)?,
        })),
        None => Ok(None),
    }
}

pub fn list(conn: &Connection, game_id: i64) -> Result<Vec<Set>, SqliteError> {
    let mut stmt = conn.prepare("SELECT id, name, image_url FROM sets WHERE game_id = ?1")?;
    let sets = stmt
        .query_map(params![game_id], |row| {
            Ok(Set {
                id: row.get(0)?,
                name: row.get(1)?,
                image_url: row.get(2)?,
            })
        })?
        .collect::<Result<Vec<_>, _>>()?;

    Ok(sets)
}

pub fn init_table(conn: &Connection) -> Result<(), SqliteError> {
    conn.execute(
        "CREATE TABLE IF NOT EXISTS sets (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            game_id INTEGER NOT NULL,
            name TEXT NOT NULL,
            image_url TEXT,
            FOREIGN KEY (game_id) REFERENCES games(id),
            UNIQUE(game_id, name)
        )",
        [],
    )?;
    Ok(())
}
