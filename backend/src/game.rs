use rusqlite::{params, Connection, Error as SqliteError};

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
