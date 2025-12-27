use rusqlite::{params, Connection, Error as SqliteError};

pub struct Game {
    pub id: i64,
    pub name: String,
    pub image_url: Option<String>,
}

#[derive(Debug)]
pub enum CreateGameError {
    NameExists,
    DatabaseError,
}

pub fn create(conn: &Connection, name: &str, image_url: Option<&str>) -> Result<i64, CreateGameError> {
    conn.execute(
        "INSERT INTO games (name, image_url) VALUES (?1, ?2)",
        params![name, image_url],
    )
    .map_err(|e| {
        if let SqliteError::SqliteFailure(err, _) = &e
            && err.extended_code == 2067
        {
            return CreateGameError::NameExists;
        }
        CreateGameError::DatabaseError
    })?;

    Ok(conn.last_insert_rowid())
}

// TODO: Add set_count and card_count fields when those features are implemented
pub fn list(conn: &Connection) -> Result<Vec<Game>, SqliteError> {
    let mut stmt = conn.prepare("SELECT id, name, image_url FROM games")?;
    let games = stmt
        .query_map([], |row| {
            Ok(Game {
                id: row.get(0)?,
                name: row.get(1)?,
                image_url: row.get(2)?,
            })
        })?
        .collect::<Result<Vec<_>, _>>()?;

    Ok(games)
}

pub fn init_table(conn: &Connection) -> Result<(), SqliteError> {
    conn.execute(
        "CREATE TABLE IF NOT EXISTS games (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            name TEXT NOT NULL UNIQUE,
            image_url TEXT
        )",
        [],
    )?;
    Ok(())
}
