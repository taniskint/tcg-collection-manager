use bcrypt::{DEFAULT_COST, hash};
use rusqlite::{Connection, Error as SqliteError, params};

#[derive(Debug)]
pub enum CreateUserError {
    UsernameExists,
    EmailExists,
    HashError,
    DatabaseError,
}

pub fn create(
    conn: &Connection,
    username: &str,
    email: &str,
    password: &str,
) -> Result<i64, CreateUserError> {
    let password_hash = hash(password, DEFAULT_COST).map_err(|_| CreateUserError::HashError)?;

    conn.execute(
        "INSERT INTO users (username, email, password_hash) VALUES (?1, ?2, ?3)",
        params![username, email, &password_hash],
    )
    .map_err(|e| {
        if let SqliteError::SqliteFailure(err, _) = &e
            && err.extended_code == 2067
        {
            let msg = e.to_string();
            if msg.contains("username") {
                return CreateUserError::UsernameExists;
            } else if msg.contains("email") {
                return CreateUserError::EmailExists;
            }
        }
        CreateUserError::DatabaseError
    })?;

    Ok(conn.last_insert_rowid())
}

pub fn init_table(conn: &Connection) -> Result<(), SqliteError> {
    conn.execute(
        "CREATE TABLE IF NOT EXISTS users (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            username TEXT NOT NULL UNIQUE,
            email TEXT NOT NULL UNIQUE,
            password_hash TEXT NOT NULL
        )",
        [],
    )?;
    Ok(())
}
