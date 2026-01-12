use bcrypt::{DEFAULT_COST, hash, verify};
use rusqlite::{Connection, Error as SqliteError, params};

#[derive(Debug)]
pub enum CreateUserError {
    UsernameExists,
    EmailExists,
    HashError,
    DatabaseError,
}

#[derive(Debug)]
pub enum UpdateUserError {
    NotFound,
    InvalidPassword,
    UsernameExists,
    EmailExists,
    NoFieldsProvided,
    HashError,
    VerifyError,
    DatabaseError,
}

#[derive(Debug)]
pub enum DeleteUserError {
    NotFound,
    InvalidPassword,
    VerifyError,
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

pub fn update(
    conn: &Connection,
    user_id: i64,
    current_password: &str,
    new_username: Option<&str>,
    new_email: Option<&str>,
    new_password: Option<&str>,
) -> Result<(), UpdateUserError> {
    // Validate at least one field is provided
    if new_username.is_none() && new_email.is_none() && new_password.is_none() {
        return Err(UpdateUserError::NoFieldsProvided);
    }

    // Get current user and verify password
    let password_hash: String = conn
        .query_row(
            "SELECT password_hash FROM users WHERE id = ?1",
            params![user_id],
            |row| row.get(0),
        )
        .map_err(|e| match e {
            SqliteError::QueryReturnedNoRows => UpdateUserError::NotFound,
            _ => UpdateUserError::DatabaseError,
        })?;

    // Verify current password
    let valid = verify(current_password, &password_hash).map_err(|_| UpdateUserError::VerifyError)?;
    if !valid {
        return Err(UpdateUserError::InvalidPassword);
    }

    // Build UPDATE statement dynamically
    let mut updates = Vec::new();
    let mut params: Vec<Box<dyn rusqlite::ToSql>> = Vec::new();

    if let Some(username) = new_username {
        updates.push("username = ?");
        params.push(Box::new(username.to_string()));
    }

    if let Some(email) = new_email {
        updates.push("email = ?");
        params.push(Box::new(email.to_string()));
    }

    if let Some(password) = new_password {
        let new_hash = hash(password, DEFAULT_COST).map_err(|_| UpdateUserError::HashError)?;
        updates.push("password_hash = ?");
        params.push(Box::new(new_hash));
    }

    params.push(Box::new(user_id));

    let sql = format!("UPDATE users SET {} WHERE id = ?", updates.join(", "));

    // Execute update
    conn.execute(
        &sql,
        rusqlite::params_from_iter(params.iter().map(|p| p.as_ref())),
    )
    .map_err(|e| {
        if let SqliteError::SqliteFailure(err, _) = &e
            && err.extended_code == 2067
        {
            let msg = e.to_string();
            if msg.contains("username") {
                return UpdateUserError::UsernameExists;
            } else if msg.contains("email") {
                return UpdateUserError::EmailExists;
            }
        }
        UpdateUserError::DatabaseError
    })?;

    Ok(())
}

pub fn delete(
    conn: &Connection,
    user_id: i64,
    password: &str,
) -> Result<(), DeleteUserError> {
    // Get user and verify password
    let password_hash: String = conn
        .query_row(
            "SELECT password_hash FROM users WHERE id = ?1",
            params![user_id],
            |row| row.get(0),
        )
        .map_err(|e| match e {
            SqliteError::QueryReturnedNoRows => DeleteUserError::NotFound,
            _ => DeleteUserError::DatabaseError,
        })?;

    // Verify password
    let valid = verify(password, &password_hash).map_err(|_| DeleteUserError::VerifyError)?;
    if !valid {
        return Err(DeleteUserError::InvalidPassword);
    }

    // Cascade delete all related data in transaction
    conn.execute(
        "DELETE FROM deck_cards WHERE deck_id IN (
            SELECT d.id FROM decks d
            JOIN collections c ON d.collection_id = c.id
            WHERE c.user_id = ?1
        )",
        params![user_id],
    )
    .map_err(|_| DeleteUserError::DatabaseError)?;

    conn.execute(
        "DELETE FROM decks WHERE collection_id IN (
            SELECT id FROM collections WHERE user_id = ?1
        )",
        params![user_id],
    )
    .map_err(|_| DeleteUserError::DatabaseError)?;

    conn.execute(
        "DELETE FROM collection_cards WHERE collection_id IN (
            SELECT id FROM collections WHERE user_id = ?1
        )",
        params![user_id],
    )
    .map_err(|_| DeleteUserError::DatabaseError)?;

    conn.execute(
        "DELETE FROM collections WHERE user_id = ?1",
        params![user_id],
    )
    .map_err(|_| DeleteUserError::DatabaseError)?;

    conn.execute(
        "DELETE FROM sessions WHERE user_id = ?1",
        params![user_id],
    )
    .map_err(|_| DeleteUserError::DatabaseError)?;

    conn.execute(
        "DELETE FROM users WHERE id = ?1",
        params![user_id],
    )
    .map_err(|_| DeleteUserError::DatabaseError)?;

    Ok(())
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
