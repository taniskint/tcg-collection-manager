use rocket::http::{ContentType, Cookie, Header, Status};
use rocket::local::blocking::Client;
use serde_json::{json, Value};

use crate::test_helpers::{admin_auth_header, create_test_client};

// ============================================================================
// Helper Functions
// ============================================================================

fn create_user(client: &Client, username: &str, email: &str, password: &str) {
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
}

fn login(client: &Client, email_or_username: &str, password: &str) -> String {
    let response = client
        .post("/api/sessions")
        .header(ContentType::JSON)
        .body(
            json!({
                "email_or_username": email_or_username,
                "password": password
            })
            .to_string(),
        )
        .dispatch();

    assert_eq!(response.status(), Status::Ok);

    response
        .cookies()
        .get("session_id")
        .unwrap()
        .value()
        .to_string()
}

fn create_game(client: &Client, name: &str) -> i64 {
    let response = client
        .post("/api/games")
        .header(ContentType::JSON)
        .header(Header::new("Authorization", admin_auth_header()))
        .body(json!({ "name": name }).to_string())
        .dispatch();

    assert_eq!(response.status(), Status::Ok);

    let body: Value = serde_json::from_str(&response.into_string().unwrap()).unwrap();
    body["id"].as_i64().unwrap()
}

// ============================================================================
// POST /api/collections (Create Collection)
// ============================================================================

#[test]
fn test_create_collection_success() {
    let client = create_test_client();
    create_user(&client, "testuser", "test@example.com", "password123");
    let session_id = login(&client, "testuser", "password123");
    let game_id = create_game(&client, "Pokemon TCG");

    let response = client
        .post("/api/collections")
        .header(ContentType::JSON)
        .cookie(Cookie::new("session_id", session_id))
        .body(
            json!({
                "game_id": game_id,
                "name": "My Collection"
            })
            .to_string(),
        )
        .dispatch();

    assert_eq!(response.status(), Status::Ok);

    let body: Value = serde_json::from_str(&response.into_string().unwrap()).unwrap();
    assert!(body["id"].as_i64().is_some());
}

#[test]
fn test_create_collection_without_auth() {
    let client = create_test_client();
    let game_id = create_game(&client, "Pokemon TCG");

    let response = client
        .post("/api/collections")
        .header(ContentType::JSON)
        .body(
            json!({
                "game_id": game_id,
                "name": "My Collection"
            })
            .to_string(),
        )
        .dispatch();

    assert_eq!(response.status(), Status::Unauthorized);
}

#[test]
fn test_create_collection_game_not_found() {
    let client = create_test_client();
    create_user(&client, "testuser", "test@example.com", "password123");
    let session_id = login(&client, "testuser", "password123");

    let response = client
        .post("/api/collections")
        .header(ContentType::JSON)
        .cookie(Cookie::new("session_id", session_id))
        .body(
            json!({
                "game_id": 99999,
                "name": "My Collection"
            })
            .to_string(),
        )
        .dispatch();

    assert_eq!(response.status(), Status::NotFound);

    let body: Value = serde_json::from_str(&response.into_string().unwrap()).unwrap();
    assert_eq!(body["error"], "Game not found");
}

// ============================================================================
// GET /api/collections (List Collections)
// ============================================================================

#[test]
fn test_list_collections_success() {
    let client = create_test_client();
    create_user(&client, "testuser", "test@example.com", "password123");
    let session_id = login(&client, "testuser", "password123");
    let game_id = create_game(&client, "Pokemon TCG");

    // Create two collections
    client
        .post("/api/collections")
        .header(ContentType::JSON)
        .cookie(Cookie::new("session_id", session_id.clone()))
        .body(json!({ "game_id": game_id, "name": "Collection 1" }).to_string())
        .dispatch();

    client
        .post("/api/collections")
        .header(ContentType::JSON)
        .cookie(Cookie::new("session_id", session_id.clone()))
        .body(json!({ "game_id": game_id, "name": "Collection 2" }).to_string())
        .dispatch();

    let response = client
        .get("/api/collections")
        .cookie(Cookie::new("session_id", session_id))
        .dispatch();

    assert_eq!(response.status(), Status::Ok);

    let body: Value = serde_json::from_str(&response.into_string().unwrap()).unwrap();
    let collections = body.as_array().unwrap();
    assert_eq!(collections.len(), 2);

    // Should include game info
    assert!(collections[0]["game_name"].as_str().is_some());
    assert!(collections[0]["name"].as_str().is_some());
    assert!(collections[0]["created_at"].as_str().is_some());
}

#[test]
fn test_list_collections_without_auth() {
    let client = create_test_client();

    let response = client.get("/api/collections").dispatch();

    assert_eq!(response.status(), Status::Unauthorized);
}

#[test]
fn test_list_collections_empty() {
    let client = create_test_client();
    create_user(&client, "testuser", "test@example.com", "password123");
    let session_id = login(&client, "testuser", "password123");

    let response = client
        .get("/api/collections")
        .cookie(Cookie::new("session_id", session_id))
        .dispatch();

    assert_eq!(response.status(), Status::Ok);

    let body: Value = serde_json::from_str(&response.into_string().unwrap()).unwrap();
    let collections = body.as_array().unwrap();
    assert_eq!(collections.len(), 0);
}

#[test]
fn test_list_collections_only_shows_own() {
    let client = create_test_client();
    let game_id = create_game(&client, "Pokemon TCG");

    // Create user 1 with a collection
    create_user(&client, "user1", "user1@example.com", "password123");
    let session1 = login(&client, "user1", "password123");
    client
        .post("/api/collections")
        .header(ContentType::JSON)
        .cookie(Cookie::new("session_id", session1.clone()))
        .body(json!({ "game_id": game_id, "name": "User1 Collection" }).to_string())
        .dispatch();

    // Create user 2 with a collection
    create_user(&client, "user2", "user2@example.com", "password123");
    let session2 = login(&client, "user2", "password123");
    client
        .post("/api/collections")
        .header(ContentType::JSON)
        .cookie(Cookie::new("session_id", session2.clone()))
        .body(json!({ "game_id": game_id, "name": "User2 Collection" }).to_string())
        .dispatch();

    // User 1 should only see their collection
    let response = client
        .get("/api/collections")
        .cookie(Cookie::new("session_id", session1))
        .dispatch();

    let body: Value = serde_json::from_str(&response.into_string().unwrap()).unwrap();
    let collections = body.as_array().unwrap();
    assert_eq!(collections.len(), 1);
    assert_eq!(collections[0]["name"], "User1 Collection");
}

// ============================================================================
// GET /api/collections/<id> (Get Collection)
// ============================================================================

#[test]
fn test_get_collection_success() {
    let client = create_test_client();
    create_user(&client, "testuser", "test@example.com", "password123");
    let session_id = login(&client, "testuser", "password123");
    let game_id = create_game(&client, "Pokemon TCG");

    // Create collection
    let create_response = client
        .post("/api/collections")
        .header(ContentType::JSON)
        .cookie(Cookie::new("session_id", session_id.clone()))
        .body(json!({ "game_id": game_id, "name": "My Collection" }).to_string())
        .dispatch();

    let create_body: Value =
        serde_json::from_str(&create_response.into_string().unwrap()).unwrap();
    let collection_id = create_body["id"].as_i64().unwrap();

    // Get collection
    let response = client
        .get(format!("/api/collections/{}", collection_id))
        .cookie(Cookie::new("session_id", session_id))
        .dispatch();

    assert_eq!(response.status(), Status::Ok);

    let body: Value = serde_json::from_str(&response.into_string().unwrap()).unwrap();
    assert_eq!(body["id"], collection_id);
    assert_eq!(body["name"], "My Collection");
    assert_eq!(body["game_name"], "Pokemon TCG");
    assert_eq!(body["card_count"], 0);
}

#[test]
fn test_get_collection_without_auth() {
    let client = create_test_client();

    let response = client.get("/api/collections/1").dispatch();

    assert_eq!(response.status(), Status::Unauthorized);
}

#[test]
fn test_get_collection_not_found() {
    let client = create_test_client();
    create_user(&client, "testuser", "test@example.com", "password123");
    let session_id = login(&client, "testuser", "password123");

    let response = client
        .get("/api/collections/99999")
        .cookie(Cookie::new("session_id", session_id))
        .dispatch();

    assert_eq!(response.status(), Status::NotFound);

    let body: Value = serde_json::from_str(&response.into_string().unwrap()).unwrap();
    assert_eq!(body["error"], "Collection not found");
}

#[test]
fn test_get_collection_not_owner() {
    let client = create_test_client();
    let game_id = create_game(&client, "Pokemon TCG");

    // Create user 1 with a collection
    create_user(&client, "user1", "user1@example.com", "password123");
    let session1 = login(&client, "user1", "password123");
    let create_response = client
        .post("/api/collections")
        .header(ContentType::JSON)
        .cookie(Cookie::new("session_id", session1))
        .body(json!({ "game_id": game_id, "name": "User1 Collection" }).to_string())
        .dispatch();

    let create_body: Value =
        serde_json::from_str(&create_response.into_string().unwrap()).unwrap();
    let collection_id = create_body["id"].as_i64().unwrap();

    // Create user 2 and try to access user 1's collection
    create_user(&client, "user2", "user2@example.com", "password123");
    let session2 = login(&client, "user2", "password123");

    let response = client
        .get(format!("/api/collections/{}", collection_id))
        .cookie(Cookie::new("session_id", session2))
        .dispatch();

    assert_eq!(response.status(), Status::Forbidden);

    let body: Value = serde_json::from_str(&response.into_string().unwrap()).unwrap();
    assert_eq!(body["error"], "Access denied");
}
