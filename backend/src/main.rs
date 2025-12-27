#[macro_use]
extern crate rocket;

mod game;
mod session;
mod user;

use std::fs;
use std::sync::Mutex;

use rocket::http::{Cookie, CookieJar, Status};
use rocket::request::{FromRequest, Outcome, Request};
use rocket::serde::json::Json;
use rocket::State;
use rusqlite::Connection;
use serde::{Deserialize, Serialize};

#[derive(Deserialize)]
struct Config {
    admin_api_key: String,
}

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

#[derive(Deserialize)]
struct CreateGameRequest {
    name: String,
    image_url: Option<String>,
}

#[derive(Serialize)]
struct GameResponse {
    id: i64,
}

struct DbConn(Mutex<Connection>);

struct AdminAuth;

#[rocket::async_trait]
impl<'r> FromRequest<'r> for AdminAuth {
    type Error = &'static str;

    async fn from_request(request: &'r Request<'_>) -> Outcome<Self, Self::Error> {
        let config = request.rocket().state::<Config>();
        let auth_header = request.headers().get_one("Authorization");

        match (config, auth_header) {
            (Some(config), Some(header)) => {
                if let Some(token) = header.strip_prefix("Bearer ")
                    // Strictly speaking, this is vulnerable to a timing attack because it uses a
                    // basic "==" for comparison.
                    && token == config.admin_api_key
                {
                    return Outcome::Success(AdminAuth);
                }
                Outcome::Error((Status::Unauthorized, "Invalid API key"))
            }
            _ => Outcome::Error((Status::Unauthorized, "Missing API key")),
        }
    }
}

fn init_db(conn: &Connection) {
    user::init_table(conn).expect("Failed to initialize users table");
    session::init_table(conn).expect("Failed to initialize sessions table");
    game::init_table(conn).expect("Failed to initialize games table");
}

fn load_config() -> Config {
    let content = fs::read_to_string("config.toml").expect("Failed to read config.toml");
    toml::from_str(&content).expect("Failed to parse config.toml")
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

#[post("/games", format = "json", data = "<req>")]
fn create_game(
    _auth: AdminAuth,
    db: &State<DbConn>,
    req: Json<CreateGameRequest>,
) -> Result<Json<GameResponse>, (Status, Json<ErrorResponse>)> {
    let conn = db.0.lock().unwrap();

    let id = game::create(&conn, &req.name, req.image_url.as_deref()).map_err(|e| {
        let (status, error) = match e {
            game::CreateGameError::NameExists => (Status::Conflict, "Game already exists"),
            game::CreateGameError::DatabaseError => (Status::InternalServerError, "Failed to create game"),
        };
        (status, Json(ErrorResponse { error: error.to_string() }))
    })?;

    Ok(Json(GameResponse { id }))
}

#[launch]
fn rocket() -> _ {
    let config = load_config();
    let conn = Connection::open("tcg.db").expect("Failed to open database");
    init_db(&conn);

    rocket::build()
        .manage(config)
        .manage(DbConn(Mutex::new(conn)))
        .mount("/", routes![index, create_user, create_session, delete_session, create_game])
}
