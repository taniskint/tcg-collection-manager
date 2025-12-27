#[macro_use]
extern crate rocket;

use std::sync::Mutex;

use bcrypt::{hash, DEFAULT_COST};
use rocket::http::Status;
use rocket::serde::json::Json;
use rocket::State;
use rusqlite::{params, Connection, Error as SqliteError};
use serde::{Deserialize, Serialize};

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

#[launch]
fn rocket() -> _ {
    let conn = Connection::open("tcg.db").expect("Failed to open database");
    init_db(&conn).expect("Failed to initialize database");

    rocket::build()
        .manage(DbConn(Mutex::new(conn)))
        .mount("/", routes![index, create_user])
}
