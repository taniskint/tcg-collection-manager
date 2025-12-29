use rocket::http::{ContentType, Status};
use rocket::local::blocking::Client;
use serde_json::{json, Value};

use crate::test_helpers::create_test_client;

// ============================================================================
// Helper Functions
// ============================================================================

fn create_user(client: &Client, username: &str, email: &str, password: &str) -> Value {
    let response = client
        .post("/api/users")
        .header(ContentType::JSON)
        .body(
            json!({
                "username": username,
                "email": email,
                "password": password
            })
            .to_string(),
        )
        .dispatch();

    assert_eq!(response.status(), Status::Ok);
    serde_json::from_str(&response.into_string().unwrap()).unwrap()
}

// ============================================================================
// POST /api/users (Create User)
// ============================================================================

#[test]
fn test_create_user_success() {
    let client = create_test_client();

    let response = client
        .post("/api/users")
        .header(ContentType::JSON)
        .body(
            json!({
                "username": "testuser",
                "email": "test@example.com",
                "password": "password123"
            })
            .to_string(),
        )
        .dispatch();

    assert_eq!(response.status(), Status::Ok);

    let body: Value = serde_json::from_str(&response.into_string().unwrap()).unwrap();
    assert!(body["id"].as_i64().is_some());
    assert!(body["id"].as_i64().unwrap() > 0);
}

#[test]
fn test_create_user_duplicate_username() {
    let client = create_test_client();

    // Create first user
    create_user(&client, "testuser", "test1@example.com", "password123");

    // Try to create user with same username
    let response = client
        .post("/api/users")
        .header(ContentType::JSON)
        .body(
            json!({
                "username": "testuser",
                "email": "test2@example.com",
                "password": "password123"
            })
            .to_string(),
        )
        .dispatch();

    assert_eq!(response.status(), Status::Conflict);

    let body: Value = serde_json::from_str(&response.into_string().unwrap()).unwrap();
    assert_eq!(body["error"], "Username already exists");
}

#[test]
fn test_create_user_duplicate_email() {
    let client = create_test_client();

    // Create first user
    create_user(&client, "testuser1", "test@example.com", "password123");

    // Try to create user with same email
    let response = client
        .post("/api/users")
        .header(ContentType::JSON)
        .body(
            json!({
                "username": "testuser2",
                "email": "test@example.com",
                "password": "password123"
            })
            .to_string(),
        )
        .dispatch();

    assert_eq!(response.status(), Status::Conflict);

    let body: Value = serde_json::from_str(&response.into_string().unwrap()).unwrap();
    assert_eq!(body["error"], "Email already exists");
}

#[test]
fn test_create_user_no_auth_required() {
    // User creation is public - no admin auth needed
    let client = create_test_client();

    let response = client
        .post("/api/users")
        .header(ContentType::JSON)
        .body(
            json!({
                "username": "testuser",
                "email": "test@example.com",
                "password": "password123"
            })
            .to_string(),
        )
        .dispatch();

    assert_eq!(response.status(), Status::Ok);
}

#[test]
fn test_create_multiple_users() {
    let client = create_test_client();

    let user1 = create_user(&client, "user1", "user1@example.com", "password1");
    let user2 = create_user(&client, "user2", "user2@example.com", "password2");
    let user3 = create_user(&client, "user3", "user3@example.com", "password3");

    // All should have different IDs
    assert_ne!(user1["id"], user2["id"]);
    assert_ne!(user2["id"], user3["id"]);
    assert_ne!(user1["id"], user3["id"]);
}

#[test]
fn test_create_user_response_only_contains_id() {
    let client = create_test_client();

    let response = client
        .post("/api/users")
        .header(ContentType::JSON)
        .body(
            json!({
                "username": "testuser",
                "email": "test@example.com",
                "password": "password123"
            })
            .to_string(),
        )
        .dispatch();

    let body: Value = serde_json::from_str(&response.into_string().unwrap()).unwrap();

    // Response should only contain id, not username, email, or password
    assert!(body.get("id").is_some());
    assert!(body.get("username").is_none());
    assert!(body.get("email").is_none());
    assert!(body.get("password").is_none());
    assert!(body.get("password_hash").is_none());
}
