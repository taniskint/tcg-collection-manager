use std::sync::Mutex;

use rocket::local::blocking::Client;
use rusqlite::Connection;

use crate::{build_rocket, init_db, Config, DbConn};

/// Default test admin API key
pub const TEST_ADMIN_KEY: &str = "test-admin-key-12345";

/// Create an in-memory SQLite database with all tables initialized
pub fn create_test_db() -> DbConn {
    let conn = Connection::open(":memory:").expect("Failed to create in-memory database");
    init_db(&conn);
    DbConn(Mutex::new(conn))
}

/// Create a test configuration
pub fn create_test_config() -> Config {
    Config::test_config(TEST_ADMIN_KEY)
}

/// Create a complete test client with fresh database and default config
pub fn create_test_client() -> Client {
    let db = create_test_db();
    let config = create_test_config();
    Client::tracked(build_rocket(db, config)).expect("Failed to create test client")
}

/// Helper to create Authorization header value
pub fn admin_auth_header() -> String {
    format!("Bearer {}", TEST_ADMIN_KEY)
}
