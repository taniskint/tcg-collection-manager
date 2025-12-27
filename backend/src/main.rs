#[macro_use]
extern crate rocket;

use std::sync::Mutex;

use bcrypt::{hash, verify, DEFAULT_COST};
use rocket::http::{Cookie, CookieJar, Status};
use rocket::serde::json::Json;
use rocket::State;
use rusqlite::{params, Connection, Error as SqliteError};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Deserialize)]
struct CreateUserRequest {
    username: String,
    email: String,
    password: String,
}

#[derive(Serialize)]
struct UserResponse {
    id: i64,
}

#[derive(Serialize)]
struct ErrorResponse {
    error: String,
}

#[derive(Deserialize)]
struct LoginRequest {
    email_or_username: String,
    password: String,
}

struct DbConn(Mutex<Connection>);

fn init_db(conn: &Connection) -> Result<(), SqliteError> {
    conn.execute(
        "CREATE TABLE IF NOT EXISTS users (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            username TEXT NOT NULL UNIQUE,
            email TEXT NOT NULL UNIQUE,
            password_hash TEXT NOT NULL
        )",
        [],
    )?;
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

#[get("/")]
fn index() -> &'static str {
    "Hello, world!"
}

#[post("/users", format = "json", data = "<user>")]
fn create_user(
    db: &State<DbConn>,
    user: Json<CreateUserRequest>,
) -> Result<Json<UserResponse>, (Status, Json<ErrorResponse>)> {
    let password_hash = hash(&user.password, DEFAULT_COST).map_err(|_| {
        (
            Status::InternalServerError,
            Json(ErrorResponse {
                error: "Failed to hash password".to_string(),
            }),
        )
    })?;

    let conn = db.0.lock().unwrap();

    conn.execute(
        "INSERT INTO users (username, email, password_hash) VALUES (?1, ?2, ?3)",
        params![&user.username, &user.email, &password_hash],
    )
    .map_err(|e| {
        if let SqliteError::SqliteFailure(err, _) = &e
            && err.extended_code == 2067
        {
            // SQLITE_CONSTRAINT_UNIQUE
            let msg = e.to_string();
            if msg.contains("username") {
                return (
                    Status::Conflict,
                    Json(ErrorResponse {
                        error: "Username already exists".to_string(),
                    }),
                );
            } else if msg.contains("email") {
                return (
                    Status::Conflict,
                    Json(ErrorResponse {
                        error: "Email already exists".to_string(),
                    }),
                );
            }
        }
        (
            Status::InternalServerError,
            Json(ErrorResponse {
                error: "Failed to create user".to_string(),
            }),
        )
    })?;

    let id = conn.last_insert_rowid();

    Ok(Json(UserResponse { id }))
}

#[post("/sessions", format = "json", data = "<login>")]
fn create_session(
    db: &State<DbConn>,
    cookies: &CookieJar<'_>,
    login: Json<LoginRequest>,
) -> Result<Status, (Status, Json<ErrorResponse>)> {
    let conn = db.0.lock().unwrap();

    // Find user by email or username
    let result: Result<(i64, String), _> = conn.query_row(
        "SELECT id, password_hash FROM users WHERE email = ?1 OR username = ?1",
        params![&login.email_or_username],
        |row| Ok((row.get(0)?, row.get(1)?)),
    );

    let (user_id, password_hash) = result.map_err(|_| {
        (
            Status::Unauthorized,
            Json(ErrorResponse {
                error: "Invalid credentials".to_string(),
            }),
        )
    })?;

    // Verify password
    let valid = verify(&login.password, &password_hash).map_err(|_| {
        (
            Status::InternalServerError,
            Json(ErrorResponse {
                error: "Failed to verify password".to_string(),
            }),
        )
    })?;

    if !valid {
        return Err((
            Status::Unauthorized,
            Json(ErrorResponse {
                error: "Invalid credentials".to_string(),
            }),
        ));
    }

    // Create session
    let session_id = Uuid::new_v4().to_string();
    let expires_at = chrono::Utc::now()
        .checked_add_signed(chrono::Duration::days(7))
        .unwrap()
        .to_rfc3339();

    conn.execute(
        "INSERT INTO sessions (id, user_id, expires_at) VALUES (?1, ?2, ?3)",
        params![&session_id, user_id, &expires_at],
    )
    .map_err(|_| {
        (
            Status::InternalServerError,
            Json(ErrorResponse {
                error: "Failed to create session".to_string(),
            }),
        )
    })?;

    cookies.add(Cookie::new("session_id", session_id));

    Ok(Status::Ok)
}

#[delete("/sessions/<session_id>")]
fn delete_session(
    db: &State<DbConn>,
    cookies: &CookieJar<'_>,
    session_id: &str,
) -> Status {
    let conn = db.0.lock().unwrap();

    let rows_affected = conn
        .execute("DELETE FROM sessions WHERE id = ?1", params![session_id])
        .unwrap_or(0);

    cookies.remove(Cookie::from("session_id"));

    if rows_affected > 0 {
        Status::Ok
    } else {
        Status::NotFound
    }
}

#[launch]
fn rocket() -> _ {
    let conn = Connection::open("tcg.db").expect("Failed to open database");
    init_db(&conn).expect("Failed to initialize database");

    rocket::build()
        .manage(DbConn(Mutex::new(conn)))
        .mount("/", routes![index, create_user, create_session, delete_session])
}
