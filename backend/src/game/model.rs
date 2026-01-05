use rusqlite::{Connection, Error as SqliteError, params};

pub struct Game {
    pub id: i64,
    pub name: String,
    pub image_url: Option<String>,
    pub set_count: i64,
}

#[derive(Debug)]
pub enum CreateGameError {
    NameExists,
    DatabaseError,
}

pub fn create(
    conn: &Connection,
    name: &str,
    image_url: Option<&str>,
) -> Result<i64, CreateGameError> {
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

// TODO: Add card_count field when that feature is implemented
pub fn get(conn: &Connection, id: i64) -> Result<Option<Game>, SqliteError> {
    let mut stmt = conn.prepare(
        "SELECT g.id, g.name, g.image_url, COUNT(s.id) as set_count
         FROM games g
         LEFT JOIN sets s ON g.id = s.game_id
         WHERE g.id = ?1
         GROUP BY g.id",
    )?;

    let mut rows = stmt.query(params![id])?;
    match rows.next()? {
        Some(row) => Ok(Some(Game {
            id: row.get(0)?,
            name: row.get(1)?,
            image_url: row.get(2)?,
            set_count: row.get(3)?,
        })),
        None => Ok(None),
    }
}

pub fn list(conn: &Connection) -> Result<Vec<Game>, SqliteError> {
    let mut stmt = conn.prepare(
        "SELECT g.id, g.name, g.image_url, COUNT(s.id) as set_count
         FROM games g
         LEFT JOIN sets s ON g.id = s.game_id
         GROUP BY g.id",
    )?;
    let games = stmt
        .query_map([], |row| {
            Ok(Game {
                id: row.get(0)?,
                name: row.get(1)?,
                image_url: row.get(2)?,
                set_count: row.get(3)?,
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
