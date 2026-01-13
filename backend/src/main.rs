#[macro_use]
extern crate rocket;

mod atlas;
mod booster;
mod card;
mod collection;
mod deck;
mod game;
mod session;
mod set;
mod user;

#[cfg(test)]
pub mod test_helpers;

use std::fs;
use std::sync::Mutex;

use rocket::fs::FileServer;
use rocket::http::Status;
use rocket::request::{FromRequest, Outcome, Request};
use rusqlite::Connection;
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Clone)]
pub struct Config {
    pub admin_api_key: String,
    pub s3: Option<S3Config>,
}

#[derive(Deserialize, Clone)]
pub struct S3Config {
    pub bucket: String,
    pub region: String,
}

impl Config {
    pub fn test_config(admin_api_key: &str) -> Self {
        Self {
            admin_api_key: admin_api_key.to_string(),
            s3: None,
        }
    }
}

#[derive(Serialize)]
pub struct ErrorResponse {
    error: String,
}

impl ErrorResponse {
    pub fn new(error: &str) -> Self {
        Self {
            error: error.to_string(),
        }
    }
}

pub struct DbConn(Mutex<Connection>);

pub struct AdminAuth;

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

pub struct SessionAuth(pub session::SessionUser);

#[rocket::async_trait]
impl<'r> FromRequest<'r> for SessionAuth {
    type Error = &'static str;

    async fn from_request(request: &'r Request<'_>) -> Outcome<Self, Self::Error> {
        let db = request.rocket().state::<DbConn>();
        let session_cookie = request.cookies().get("session_id");

        match (db, session_cookie) {
            (Some(db), Some(cookie)) => {
                let conn = db.0.lock().unwrap();
                match session::get_user_by_session(&conn, cookie.value()) {
                    Some(user) => Outcome::Success(SessionAuth(user)),
                    None => Outcome::Error((Status::Unauthorized, "Invalid session")),
                }
            }
            _ => Outcome::Error((Status::Unauthorized, "No session")),
        }
    }
}

pub fn init_db(conn: &Connection) {
    user::init_table(conn).expect("Failed to initialize users table");
    session::init_table(conn).expect("Failed to initialize sessions table");
    game::init_table(conn).expect("Failed to initialize games table");
    set::init_table(conn).expect("Failed to initialize sets table");
    card::init_table(conn).expect("Failed to initialize cards table");
    collection::init_table(conn).expect("Failed to initialize collections table");
    collection::init_collection_cards_table(conn)
        .expect("Failed to initialize collection_cards table");
    deck::init_table(conn).expect("Failed to initialize decks table");
    deck::init_deck_cards_table(conn).expect("Failed to initialize deck_cards table");
    booster::init_table(conn).expect("Failed to initialize boosters table");
}

fn load_config() -> Config {
    let content = fs::read_to_string("config.toml").expect("Failed to read config.toml");
    toml::from_str(&content).expect("Failed to parse config.toml")
}

pub fn build_rocket(db_conn: DbConn, config: Config) -> rocket::Rocket<rocket::Build> {
    rocket::build()
        .manage(config)
        .manage(db_conn)
        .mount("/", FileServer::from("../frontend"))
        .mount("/api/users", user::routes::routes())
        .mount("/api/sessions", session::routes::routes())
        .mount("/api/games", game::routes::routes())
        .mount("/api/games", set::routes::routes())
        .mount("/api/games", card::routes::routes())
        .mount("/api/collections", collection::routes::routes())
        .mount("/api/decks", deck::routes::routes())
        .mount("/api/games", booster::routes::routes())
        .mount("/api/boosters", booster::routes::open_routes())
}

#[launch]
fn rocket() -> _ {
    let config = load_config();
    let conn = Connection::open("tcg.db").expect("Failed to open database");
    init_db(&conn);

    build_rocket(DbConn(Mutex::new(conn)), config)
}
