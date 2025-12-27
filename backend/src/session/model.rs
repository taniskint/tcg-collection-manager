use bcrypt::verify;
use rusqlite::{params, Connection, Error as SqliteError};
use uuid::Uuid;

#[derive(Debug)]
pub enum CreateSessionError {
    InvalidCredentials,
    VerifyError,
    DatabaseError,
}

pub fn create(
    conn: &Connection,
    email_or_username: &str,
    password: &str,
) -> Result<String, CreateSessionError> {
    let result: Result<(i64, String), _> = conn.query_row(
        "SELECT id, password_hash FROM users WHERE email = ?1 OR username = ?1",
        params![email_or_username],
        |row| Ok((row.get(0)?, row.get(1)?)),
    );

    let (user_id, password_hash) = result.map_err(|_| CreateSessionError::InvalidCredentials)?;

    let valid = verify(password, &password_hash).map_err(|_| CreateSessionError::VerifyError)?;

    if !valid {
        return Err(CreateSessionError::InvalidCredentials);
    }

    let session_id = Uuid::new_v4().to_string();
    let expires_at = chrono::Utc::now()
        .checked_add_signed(chrono::Duration::days(7))
        .unwrap()
        .to_rfc3339();

    conn.execute(
        "INSERT INTO sessions (id, user_id, expires_at) VALUES (?1, ?2, ?3)",
        params![&session_id, user_id, &expires_at],
    )
    .map_err(|_| CreateSessionError::DatabaseError)?;

    Ok(session_id)
}

pub fn delete(conn: &Connection, session_id: &str) -> bool {
    let rows_affected = conn
        .execute("DELETE FROM sessions WHERE id = ?1", params![session_id])
        .unwrap_or(0);

    rows_affected > 0
}

pub fn init_table(conn: &Connection) -> Result<(), SqliteError> {
    conn.execute(
        "CREATE TABLE IF NOT EXISTS sessions (
            id TEXT PRIMARY KEY,
            user_id INTEGER NOT NULL,
            expires_at TEXT NOT NULL,
            FOREIGN KEY (user_id) REFERENCES users(id)
        )",
        [],
    )?;
    Ok(())
}
