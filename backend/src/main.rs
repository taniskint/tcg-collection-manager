#[macro_use]
extern crate rocket;

mod card;
mod game;
mod session;
mod set;
mod user;

#[cfg(test)]
pub mod test_helpers;

use std::fs;
use std::sync::Mutex;

use rocket::http::Status;
use rocket::request::{FromRequest, Outcome, Request};
use rusqlite::Connection;
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Clone)]
pub struct Config {
    pub admin_api_key: String,
}

impl Config {
    pub fn test_config(admin_api_key: &str) -> Self {
        Self {
            admin_api_key: admin_api_key.to_string(),
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

pub fn init_db(conn: &Connection) {
    user::init_table(conn).expect("Failed to initialize users table");
    session::init_table(conn).expect("Failed to initialize sessions table");
    game::init_table(conn).expect("Failed to initialize games table");
    set::init_table(conn).expect("Failed to initialize sets table");
    card::init_table(conn).expect("Failed to initialize cards table");
}

fn load_config() -> Config {
    let content = fs::read_to_string("config.toml").expect("Failed to read config.toml");
    toml::from_str(&content).expect("Failed to parse config.toml")
}

#[get("/")]
fn index() -> &'static str {
    "Hello, world!"
}

pub fn build_rocket(db_conn: DbConn, config: Config) -> rocket::Rocket<rocket::Build> {
    rocket::build()
        .manage(config)
        .manage(db_conn)
        .mount("/", routes![index])
        .mount("/users", user::routes::routes())
        .mount("/sessions", session::routes::routes())
        .mount("/games", game::routes::routes())
        .mount("/games", set::routes::routes())
        .mount("/games", card::routes::routes())
}

#[launch]
fn rocket() -> _ {
    let config = load_config();
    let conn = Connection::open("tcg.db").expect("Failed to open database");
    init_db(&conn);

    build_rocket(DbConn(Mutex::new(conn)), config)
}
