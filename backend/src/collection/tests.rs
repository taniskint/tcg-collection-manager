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

fn create_set(client: &Client, game_id: i64, name: &str, code: &str) -> i64 {
    let response = client
        .post(format!("/api/games/{}/sets", game_id))
        .header(ContentType::JSON)
        .header(Header::new("Authorization", admin_auth_header()))
        .body(json!({ "name": name, "code": code }).to_string())
        .dispatch();

    assert_eq!(response.status(), Status::Ok);

    let body: Value = serde_json::from_str(&response.into_string().unwrap()).unwrap();
    body["id"].as_i64().unwrap()
}

fn create_cards(client: &Client, game_id: i64, set_id: i64, cards: Vec<(&str, &str)>) -> Vec<i64> {
    let cards_json: Vec<Value> = cards
        .iter()
        .map(|(name, number)| {
            json!({
                "name": name,
                "collector_number": number
            })
        })
        .collect();

    let response = client
        .post(format!("/api/games/{}/sets/{}/cards", game_id, set_id))
        .header(ContentType::JSON)
        .header(Header::new("Authorization", admin_auth_header()))
        .body(Value::Array(cards_json).to_string())
        .dispatch();

    assert_eq!(response.status(), Status::Ok);

    let body: Value = serde_json::from_str(&response.into_string().unwrap()).unwrap();
    body["ids"]
        .as_array()
        .unwrap()
        .iter()
        .map(|v| v.as_i64().unwrap())
        .collect()
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

// ============================================================================
// GET /api/collections/<id>/cards (List Collection Cards)
// ============================================================================

#[test]
fn test_list_collection_cards_empty() {
    let client = create_test_client();
    create_user(&client, "testuser", "test@example.com", "password123");
    let session_id = login(&client, "testuser", "password123");
    let game_id = create_game(&client, "Pokemon TCG");
    let collection_id = create_collection(&client, &session_id, game_id, "My Collection");

    let response = client
        .get(format!("/api/collections/{}/cards", collection_id))
        .cookie(Cookie::new("session_id", session_id))
        .dispatch();

    assert_eq!(response.status(), Status::Ok);

    let body: Value = serde_json::from_str(&response.into_string().unwrap()).unwrap();
    let cards = body.as_array().unwrap();
    assert_eq!(cards.len(), 0);
}

#[test]
fn test_list_collection_cards_with_cards() {
    let client = create_test_client();
    create_user(&client, "testuser", "test@example.com", "password123");
    let session_id = login(&client, "testuser", "password123");
    let game_id = create_game(&client, "Pokemon TCG");
    let set_id = create_set(&client, game_id, "Base Set", "BS");
    let card_ids = create_cards(
        &client,
        game_id,
        set_id,
        vec![("Pikachu", "025"), ("Charizard", "006")],
    );
    let collection_id = create_collection(&client, &session_id, game_id, "My Collection");

    // Add cards to collection
    client
        .patch(format!("/api/collections/{}/cards", collection_id))
        .header(ContentType::JSON)
        .cookie(Cookie::new("session_id", session_id.clone()))
        .body(
            json!([
                { "card_id": card_ids[0], "quantity": 2 },
                { "card_id": card_ids[1], "quantity": 1 }
            ])
            .to_string(),
        )
        .dispatch();

    let response = client
        .get(format!("/api/collections/{}/cards", collection_id))
        .cookie(Cookie::new("session_id", session_id))
        .dispatch();

    assert_eq!(response.status(), Status::Ok);

    let body: Value = serde_json::from_str(&response.into_string().unwrap()).unwrap();
    let cards = body.as_array().unwrap();
    assert_eq!(cards.len(), 2);

    // Check first card details
    let first_card = &cards[0];
    assert!(first_card["id"].as_i64().is_some());
    assert!(first_card["name"].as_str().is_some());
    assert!(first_card["collector_number"].as_str().is_some());
    assert!(first_card["set_name"].as_str().is_some());
    assert!(first_card["quantity"].as_i64().is_some());
}

#[test]
fn test_list_collection_cards_without_auth() {
    let client = create_test_client();

    let response = client.get("/api/collections/1/cards").dispatch();

    assert_eq!(response.status(), Status::Unauthorized);
}

#[test]
fn test_list_collection_cards_not_owner() {
    let client = create_test_client();
    let game_id = create_game(&client, "Pokemon TCG");

    // Create user 1 with a collection
    create_user(&client, "user1", "user1@example.com", "password123");
    let session1 = login(&client, "user1", "password123");
    let collection_id = create_collection(&client, &session1, game_id, "User1 Collection");

    // Create user 2 and try to access user 1's collection cards
    create_user(&client, "user2", "user2@example.com", "password123");
    let session2 = login(&client, "user2", "password123");

    let response = client
        .get(format!("/api/collections/{}/cards", collection_id))
        .cookie(Cookie::new("session_id", session2))
        .dispatch();

    assert_eq!(response.status(), Status::Forbidden);
}

#[test]
fn test_list_collection_cards_not_found() {
    let client = create_test_client();
    create_user(&client, "testuser", "test@example.com", "password123");
    let session_id = login(&client, "testuser", "password123");

    let response = client
        .get("/api/collections/99999/cards")
        .cookie(Cookie::new("session_id", session_id))
        .dispatch();

    assert_eq!(response.status(), Status::NotFound);
}

// ============================================================================
// PATCH /api/collections/<id>/cards (Update Collection Cards)
// ============================================================================

#[test]
fn test_update_collection_cards_add() {
    let client = create_test_client();
    create_user(&client, "testuser", "test@example.com", "password123");
    let session_id = login(&client, "testuser", "password123");
    let game_id = create_game(&client, "Pokemon TCG");
    let set_id = create_set(&client, game_id, "Base Set", "BS");
    let card_ids = create_cards(&client, game_id, set_id, vec![("Pikachu", "025")]);
    let collection_id = create_collection(&client, &session_id, game_id, "My Collection");

    let response = client
        .patch(format!("/api/collections/{}/cards", collection_id))
        .header(ContentType::JSON)
        .cookie(Cookie::new("session_id", session_id.clone()))
        .body(json!([{ "card_id": card_ids[0], "quantity": 3 }]).to_string())
        .dispatch();

    assert_eq!(response.status(), Status::NoContent);

    // Verify the card was added
    let list_response = client
        .get(format!("/api/collections/{}/cards", collection_id))
        .cookie(Cookie::new("session_id", session_id))
        .dispatch();

    let body: Value = serde_json::from_str(&list_response.into_string().unwrap()).unwrap();
    let cards = body.as_array().unwrap();
    assert_eq!(cards.len(), 1);
    assert_eq!(cards[0]["quantity"], 3);
}

#[test]
fn test_update_collection_cards_remove() {
    let client = create_test_client();
    create_user(&client, "testuser", "test@example.com", "password123");
    let session_id = login(&client, "testuser", "password123");
    let game_id = create_game(&client, "Pokemon TCG");
    let set_id = create_set(&client, game_id, "Base Set", "BS");
    let card_ids = create_cards(&client, game_id, set_id, vec![("Pikachu", "025")]);
    let collection_id = create_collection(&client, &session_id, game_id, "My Collection");

    // First add the card
    client
        .patch(format!("/api/collections/{}/cards", collection_id))
        .header(ContentType::JSON)
        .cookie(Cookie::new("session_id", session_id.clone()))
        .body(json!([{ "card_id": card_ids[0], "quantity": 3 }]).to_string())
        .dispatch();

    // Then remove it by setting quantity to 0
    let response = client
        .patch(format!("/api/collections/{}/cards", collection_id))
        .header(ContentType::JSON)
        .cookie(Cookie::new("session_id", session_id.clone()))
        .body(json!([{ "card_id": card_ids[0], "quantity": 0 }]).to_string())
        .dispatch();

    assert_eq!(response.status(), Status::NoContent);

    // Verify the card was removed
    let list_response = client
        .get(format!("/api/collections/{}/cards", collection_id))
        .cookie(Cookie::new("session_id", session_id))
        .dispatch();

    let body: Value = serde_json::from_str(&list_response.into_string().unwrap()).unwrap();
    let cards = body.as_array().unwrap();
    assert_eq!(cards.len(), 0);
}

#[test]
fn test_update_collection_cards_update_quantity() {
    let client = create_test_client();
    create_user(&client, "testuser", "test@example.com", "password123");
    let session_id = login(&client, "testuser", "password123");
    let game_id = create_game(&client, "Pokemon TCG");
    let set_id = create_set(&client, game_id, "Base Set", "BS");
    let card_ids = create_cards(&client, game_id, set_id, vec![("Pikachu", "025")]);
    let collection_id = create_collection(&client, &session_id, game_id, "My Collection");

    // First add the card with quantity 1
    client
        .patch(format!("/api/collections/{}/cards", collection_id))
        .header(ContentType::JSON)
        .cookie(Cookie::new("session_id", session_id.clone()))
        .body(json!([{ "card_id": card_ids[0], "quantity": 1 }]).to_string())
        .dispatch();

    // Update to quantity 5
    let response = client
        .patch(format!("/api/collections/{}/cards", collection_id))
        .header(ContentType::JSON)
        .cookie(Cookie::new("session_id", session_id.clone()))
        .body(json!([{ "card_id": card_ids[0], "quantity": 5 }]).to_string())
        .dispatch();

    assert_eq!(response.status(), Status::NoContent);

    // Verify the quantity was updated
    let list_response = client
        .get(format!("/api/collections/{}/cards", collection_id))
        .cookie(Cookie::new("session_id", session_id))
        .dispatch();

    let body: Value = serde_json::from_str(&list_response.into_string().unwrap()).unwrap();
    let cards = body.as_array().unwrap();
    assert_eq!(cards.len(), 1);
    assert_eq!(cards[0]["quantity"], 5);
}

#[test]
fn test_update_collection_cards_without_auth() {
    let client = create_test_client();

    let response = client
        .patch("/api/collections/1/cards")
        .header(ContentType::JSON)
        .body(json!([{ "card_id": 1, "quantity": 1 }]).to_string())
        .dispatch();

    assert_eq!(response.status(), Status::Unauthorized);
}

#[test]
fn test_update_collection_cards_not_owner() {
    let client = create_test_client();
    let game_id = create_game(&client, "Pokemon TCG");
    let set_id = create_set(&client, game_id, "Base Set", "BS");
    let card_ids = create_cards(&client, game_id, set_id, vec![("Pikachu", "025")]);

    // Create user 1 with a collection
    create_user(&client, "user1", "user1@example.com", "password123");
    let session1 = login(&client, "user1", "password123");
    let collection_id = create_collection(&client, &session1, game_id, "User1 Collection");

    // Create user 2 and try to modify user 1's collection
    create_user(&client, "user2", "user2@example.com", "password123");
    let session2 = login(&client, "user2", "password123");

    let response = client
        .patch(format!("/api/collections/{}/cards", collection_id))
        .header(ContentType::JSON)
        .cookie(Cookie::new("session_id", session2))
        .body(json!([{ "card_id": card_ids[0], "quantity": 1 }]).to_string())
        .dispatch();

    assert_eq!(response.status(), Status::Forbidden);
}

#[test]
fn test_update_collection_cards_card_not_found() {
    let client = create_test_client();
    create_user(&client, "testuser", "test@example.com", "password123");
    let session_id = login(&client, "testuser", "password123");
    let game_id = create_game(&client, "Pokemon TCG");
    let collection_id = create_collection(&client, &session_id, game_id, "My Collection");

    let response = client
        .patch(format!("/api/collections/{}/cards", collection_id))
        .header(ContentType::JSON)
        .cookie(Cookie::new("session_id", session_id))
        .body(json!([{ "card_id": 99999, "quantity": 1 }]).to_string())
        .dispatch();

    assert_eq!(response.status(), Status::NotFound);

    let body: Value = serde_json::from_str(&response.into_string().unwrap()).unwrap();
    assert_eq!(body["error"], "Card not found");
}

#[test]
fn test_update_collection_cards_game_mismatch() {
    let client = create_test_client();
    create_user(&client, "testuser", "test@example.com", "password123");
    let session_id = login(&client, "testuser", "password123");

    // Create two different games
    let pokemon_id = create_game(&client, "Pokemon TCG");
    let mtg_id = create_game(&client, "Magic: The Gathering");

    // Create a card for MTG
    let mtg_set_id = create_set(&client, mtg_id, "Alpha", "LEA");
    let mtg_card_ids = create_cards(&client, mtg_id, mtg_set_id, vec![("Black Lotus", "001")]);

    // Create a Pokemon collection
    let collection_id = create_collection(&client, &session_id, pokemon_id, "My Pokemon Collection");

    // Try to add MTG card to Pokemon collection
    let response = client
        .patch(format!("/api/collections/{}/cards", collection_id))
        .header(ContentType::JSON)
        .cookie(Cookie::new("session_id", session_id))
        .body(json!([{ "card_id": mtg_card_ids[0], "quantity": 1 }]).to_string())
        .dispatch();

    assert_eq!(response.status(), Status::BadRequest);

    let body: Value = serde_json::from_str(&response.into_string().unwrap()).unwrap();
    assert_eq!(body["error"], "Card does not belong to collection's game");
}

#[test]
fn test_update_collection_cards_collection_not_found() {
    let client = create_test_client();
    create_user(&client, "testuser", "test@example.com", "password123");
    let session_id = login(&client, "testuser", "password123");

    let response = client
        .patch("/api/collections/99999/cards")
        .header(ContentType::JSON)
        .cookie(Cookie::new("session_id", session_id))
        .body(json!([{ "card_id": 1, "quantity": 1 }]).to_string())
        .dispatch();

    assert_eq!(response.status(), Status::NotFound);
}

// ============================================================================
// Card Count in Collection
// ============================================================================

#[test]
fn test_collection_card_count_updates() {
    let client = create_test_client();
    create_user(&client, "testuser", "test@example.com", "password123");
    let session_id = login(&client, "testuser", "password123");
    let game_id = create_game(&client, "Pokemon TCG");
    let set_id = create_set(&client, game_id, "Base Set", "BS");
    let card_ids = create_cards(
        &client,
        game_id,
        set_id,
        vec![("Pikachu", "025"), ("Charizard", "006")],
    );
    let collection_id = create_collection(&client, &session_id, game_id, "My Collection");

    // Initially card_count should be 0
    let response = client
        .get(format!("/api/collections/{}", collection_id))
        .cookie(Cookie::new("session_id", session_id.clone()))
        .dispatch();
    let body: Value = serde_json::from_str(&response.into_string().unwrap()).unwrap();
    assert_eq!(body["card_count"], 0);

    // Add cards: 2 Pikachu + 3 Charizard = 5 total
    client
        .patch(format!("/api/collections/{}/cards", collection_id))
        .header(ContentType::JSON)
        .cookie(Cookie::new("session_id", session_id.clone()))
        .body(
            json!([
                { "card_id": card_ids[0], "quantity": 2 },
                { "card_id": card_ids[1], "quantity": 3 }
            ])
            .to_string(),
        )
        .dispatch();

    // card_count should now be 5
    let response = client
        .get(format!("/api/collections/{}", collection_id))
        .cookie(Cookie::new("session_id", session_id.clone()))
        .dispatch();
    let body: Value = serde_json::from_str(&response.into_string().unwrap()).unwrap();
    assert_eq!(body["card_count"], 5);

    // Remove one card
    client
        .patch(format!("/api/collections/{}/cards", collection_id))
        .header(ContentType::JSON)
        .cookie(Cookie::new("session_id", session_id.clone()))
        .body(json!([{ "card_id": card_ids[0], "quantity": 0 }]).to_string())
        .dispatch();

    // card_count should now be 3
    let response = client
        .get(format!("/api/collections/{}", collection_id))
        .cookie(Cookie::new("session_id", session_id))
        .dispatch();
    let body: Value = serde_json::from_str(&response.into_string().unwrap()).unwrap();
    assert_eq!(body["card_count"], 3);
}
