#[macro_use]
extern crate rocket;

mod session;
mod user;

use std::sync::Mutex;

use rocket::http::{Cookie, CookieJar, Status};
use rocket::serde::json::Json;
use rocket::State;
use rusqlite::Connection;
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

#[derive(Deserialize)]
struct LoginRequest {
    email_or_username: String,
    password: String,
}

struct DbConn(Mutex<Connection>);

fn init_db(conn: &Connection) {
    user::init_table(conn).expect("Failed to initialize users table");
    session::init_table(conn).expect("Failed to initialize sessions table");
}

#[get("/")]
fn index() -> &'static str {
    "Hello, world!"
}

#[post("/users", format = "json", data = "<req>")]
fn create_user(
    db: &State<DbConn>,
    req: Json<CreateUserRequest>,
) -> Result<Json<UserResponse>, (Status, Json<ErrorResponse>)> {
    let conn = db.0.lock().unwrap();

    let id = user::create(&conn, &req.username, &req.email, &req.password).map_err(|e| {
        let (status, error) = match e {
            user::CreateUserError::UsernameExists => (Status::Conflict, "Username already exists"),
            user::CreateUserError::EmailExists => (Status::Conflict, "Email already exists"),
            user::CreateUserError::HashError => (Status::InternalServerError, "Failed to hash password"),
            user::CreateUserError::DatabaseError => (Status::InternalServerError, "Failed to create user"),
        };
        (status, Json(ErrorResponse { error: error.to_string() }))
    })?;

    Ok(Json(UserResponse { id }))
}

#[post("/sessions", format = "json", data = "<req>")]
fn create_session(
    db: &State<DbConn>,
    cookies: &CookieJar<'_>,
    req: Json<LoginRequest>,
) -> Result<Status, (Status, Json<ErrorResponse>)> {
    let conn = db.0.lock().unwrap();

    let session_id = session::create(&conn, &req.email_or_username, &req.password).map_err(|e| {
        let (status, error) = match e {
            session::CreateSessionError::InvalidCredentials => (Status::Unauthorized, "Invalid credentials"),
            session::CreateSessionError::VerifyError => (Status::InternalServerError, "Failed to verify password"),
            session::CreateSessionError::DatabaseError => (Status::InternalServerError, "Failed to create session"),
        };
        (status, Json(ErrorResponse { error: error.to_string() }))
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

    cookies.remove(Cookie::from("session_id"));

    if session::delete(&conn, session_id) {
        Status::Ok
    } else {
        Status::NotFound
    }
}

#[launch]
fn rocket() -> _ {
    let conn = Connection::open("tcg.db").expect("Failed to open database");
    init_db(&conn);

    rocket::build()
        .manage(DbConn(Mutex::new(conn)))
        .mount("/", routes![index, create_user, create_session, delete_session])
}
