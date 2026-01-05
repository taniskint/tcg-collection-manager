use rocket::http::{ContentType, Cookie, Header, Status};
use rocket::local::blocking::Client;
use serde_json::{Value, json};

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

fn create_collection(client: &Client, session_id: &str, game_id: i64, name: &str) -> i64 {
    let response = client
        .post("/api/collections")
        .header(ContentType::JSON)
        .cookie(Cookie::new("session_id", session_id.to_string()))
        .body(json!({ "game_id": game_id, "name": name }).to_string())
        .dispatch();

    assert_eq!(response.status(), Status::Ok);

    let body: Value = serde_json::from_str(&response.into_string().unwrap()).unwrap();
    body["id"].as_i64().unwrap()
}

fn create_deck(client: &Client, session_id: &str, collection_id: i64, name: &str) -> i64 {
    let response = client
        .post("/api/decks")
        .header(ContentType::JSON)
        .cookie(Cookie::new("session_id", session_id.to_string()))
        .body(json!({ "collection_id": collection_id, "name": name }).to_string())
        .dispatch();

    assert_eq!(response.status(), Status::Ok);

    let body: Value = serde_json::from_str(&response.into_string().unwrap()).unwrap();
    body["id"].as_i64().unwrap()
}

// ============================================================================
// POST /api/decks (Create Deck)
// ============================================================================

#[test]
fn test_create_deck_success() {
    let client = create_test_client();
    create_user(&client, "testuser", "test@example.com", "password123");
    let session_id = login(&client, "testuser", "password123");
    let game_id = create_game(&client, "Pokemon TCG");
    let collection_id = create_collection(&client, &session_id, game_id, "My Collection");

    let response = client
        .post("/api/decks")
        .header(ContentType::JSON)
        .cookie(Cookie::new("session_id", session_id))
        .body(
            json!({
                "collection_id": collection_id,
                "name": "My Deck"
            })
            .to_string(),
        )
        .dispatch();

    assert_eq!(response.status(), Status::Ok);

    let body: Value = serde_json::from_str(&response.into_string().unwrap()).unwrap();
    assert!(body["id"].as_i64().is_some());
}

#[test]
fn test_create_deck_without_auth() {
    let client = create_test_client();

    let response = client
        .post("/api/decks")
        .header(ContentType::JSON)
        .body(
            json!({
                "collection_id": 1,
                "name": "My Deck"
            })
            .to_string(),
        )
        .dispatch();

    assert_eq!(response.status(), Status::Unauthorized);
}

#[test]
fn test_create_deck_collection_not_found() {
    let client = create_test_client();
    create_user(&client, "testuser", "test@example.com", "password123");
    let session_id = login(&client, "testuser", "password123");

    let response = client
        .post("/api/decks")
        .header(ContentType::JSON)
        .cookie(Cookie::new("session_id", session_id))
        .body(
            json!({
                "collection_id": 99999,
                "name": "My Deck"
            })
            .to_string(),
        )
        .dispatch();

    assert_eq!(response.status(), Status::NotFound);

    let body: Value = serde_json::from_str(&response.into_string().unwrap()).unwrap();
    assert_eq!(body["error"], "Collection not found");
}

#[test]
fn test_create_deck_not_owner() {
    let client = create_test_client();
    let game_id = create_game(&client, "Pokemon TCG");

    // Create user 1 with a collection
    create_user(&client, "user1", "user1@example.com", "password123");
    let session1 = login(&client, "user1", "password123");
    let collection_id = create_collection(&client, &session1, game_id, "User1 Collection");

    // Create user 2 and try to create a deck in user 1's collection
    create_user(&client, "user2", "user2@example.com", "password123");
    let session2 = login(&client, "user2", "password123");

    let response = client
        .post("/api/decks")
        .header(ContentType::JSON)
        .cookie(Cookie::new("session_id", session2))
        .body(
            json!({
                "collection_id": collection_id,
                "name": "My Deck"
            })
            .to_string(),
        )
        .dispatch();

    assert_eq!(response.status(), Status::Forbidden);

    let body: Value = serde_json::from_str(&response.into_string().unwrap()).unwrap();
    assert_eq!(body["error"], "Access denied");
}

// ============================================================================
// GET /api/decks (List Decks)
// ============================================================================

#[test]
fn test_list_decks_success() {
    let client = create_test_client();
    create_user(&client, "testuser", "test@example.com", "password123");
    let session_id = login(&client, "testuser", "password123");
    let game_id = create_game(&client, "Pokemon TCG");
    let collection_id = create_collection(&client, &session_id, game_id, "My Collection");

    // Create two decks
    create_deck(&client, &session_id, collection_id, "Deck 1");
    create_deck(&client, &session_id, collection_id, "Deck 2");

    let response = client
        .get("/api/decks")
        .cookie(Cookie::new("session_id", session_id))
        .dispatch();

    assert_eq!(response.status(), Status::Ok);

    let body: Value = serde_json::from_str(&response.into_string().unwrap()).unwrap();
    let decks = body.as_array().unwrap();
    assert_eq!(decks.len(), 2);

    // Should include collection and game info
    assert!(decks[0]["collection_name"].as_str().is_some());
    assert!(decks[0]["game_name"].as_str().is_some());
    assert!(decks[0]["name"].as_str().is_some());
    assert!(decks[0]["created_at"].as_str().is_some());
}

#[test]
fn test_list_decks_without_auth() {
    let client = create_test_client();

    let response = client.get("/api/decks").dispatch();

    assert_eq!(response.status(), Status::Unauthorized);
}

#[test]
fn test_list_decks_empty() {
    let client = create_test_client();
    create_user(&client, "testuser", "test@example.com", "password123");
    let session_id = login(&client, "testuser", "password123");

    let response = client
        .get("/api/decks")
        .cookie(Cookie::new("session_id", session_id))
        .dispatch();

    assert_eq!(response.status(), Status::Ok);

    let body: Value = serde_json::from_str(&response.into_string().unwrap()).unwrap();
    let decks = body.as_array().unwrap();
    assert_eq!(decks.len(), 0);
}

#[test]
fn test_list_decks_only_shows_own() {
    let client = create_test_client();
    let game_id = create_game(&client, "Pokemon TCG");

    // Create user 1 with a deck
    create_user(&client, "user1", "user1@example.com", "password123");
    let session1 = login(&client, "user1", "password123");
    let collection1 = create_collection(&client, &session1, game_id, "User1 Collection");
    create_deck(&client, &session1, collection1, "User1 Deck");

    // Create user 2 with a deck
    create_user(&client, "user2", "user2@example.com", "password123");
    let session2 = login(&client, "user2", "password123");
    let collection2 = create_collection(&client, &session2, game_id, "User2 Collection");
    create_deck(&client, &session2, collection2, "User2 Deck");

    // User 1 should only see their deck
    let response = client
        .get("/api/decks")
        .cookie(Cookie::new("session_id", session1))
        .dispatch();

    let body: Value = serde_json::from_str(&response.into_string().unwrap()).unwrap();
    let decks = body.as_array().unwrap();
    assert_eq!(decks.len(), 1);
    assert_eq!(decks[0]["name"], "User1 Deck");
}

// ============================================================================
// GET /api/decks/<id> (Get Deck)
// ============================================================================

#[test]
fn test_get_deck_success() {
    let client = create_test_client();
    create_user(&client, "testuser", "test@example.com", "password123");
    let session_id = login(&client, "testuser", "password123");
    let game_id = create_game(&client, "Pokemon TCG");
    let collection_id = create_collection(&client, &session_id, game_id, "My Collection");
    let deck_id = create_deck(&client, &session_id, collection_id, "My Deck");

    let response = client
        .get(format!("/api/decks/{}", deck_id))
        .cookie(Cookie::new("session_id", session_id))
        .dispatch();

    assert_eq!(response.status(), Status::Ok);

    let body: Value = serde_json::from_str(&response.into_string().unwrap()).unwrap();
    assert_eq!(body["id"], deck_id);
    assert_eq!(body["name"], "My Deck");
    assert_eq!(body["collection_name"], "My Collection");
    assert_eq!(body["game_name"], "Pokemon TCG");
    assert_eq!(body["card_count"], 0);
}

#[test]
fn test_get_deck_without_auth() {
    let client = create_test_client();

    let response = client.get("/api/decks/1").dispatch();

    assert_eq!(response.status(), Status::Unauthorized);
}

#[test]
fn test_get_deck_not_found() {
    let client = create_test_client();
    create_user(&client, "testuser", "test@example.com", "password123");
    let session_id = login(&client, "testuser", "password123");

    let response = client
        .get("/api/decks/99999")
        .cookie(Cookie::new("session_id", session_id))
        .dispatch();

    assert_eq!(response.status(), Status::NotFound);

    let body: Value = serde_json::from_str(&response.into_string().unwrap()).unwrap();
    assert_eq!(body["error"], "Deck not found");
}

#[test]
fn test_get_deck_not_owner() {
    let client = create_test_client();
    let game_id = create_game(&client, "Pokemon TCG");

    // Create user 1 with a deck
    create_user(&client, "user1", "user1@example.com", "password123");
    let session1 = login(&client, "user1", "password123");
    let collection_id = create_collection(&client, &session1, game_id, "User1 Collection");
    let deck_id = create_deck(&client, &session1, collection_id, "User1 Deck");

    // Create user 2 and try to access user 1's deck
    create_user(&client, "user2", "user2@example.com", "password123");
    let session2 = login(&client, "user2", "password123");

    let response = client
        .get(format!("/api/decks/{}", deck_id))
        .cookie(Cookie::new("session_id", session2))
        .dispatch();

    assert_eq!(response.status(), Status::Forbidden);

    let body: Value = serde_json::from_str(&response.into_string().unwrap()).unwrap();
    assert_eq!(body["error"], "Access denied");
}
