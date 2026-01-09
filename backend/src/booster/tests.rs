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

fn create_set(client: &Client, game_id: i64, name: &str) -> i64 {
    let response = client
        .post(format!("/api/games/{}/sets", game_id))
        .header(ContentType::JSON)
        .header(Header::new("Authorization", admin_auth_header()))
        .body(json!({ "name": name, "publish_date": "2024-01-15" }).to_string())
        .dispatch();

    assert_eq!(response.status(), Status::Ok);

    let body: Value = serde_json::from_str(&response.into_string().unwrap()).unwrap();
    body["id"].as_i64().unwrap()
}

fn create_cards_with_attributes(
    client: &Client,
    game_id: i64,
    set_id: i64,
    cards: Vec<(&str, &str, Value)>,
) -> Vec<i64> {
    let cards_json: Vec<Value> = cards
        .iter()
        .map(|(name, number, attrs)| {
            json!({
                "name": name,
                "collector_number": number,
                "attributes": attrs
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

fn create_booster(client: &Client, game_id: i64, set_id: i64, name: &str, spec: Value) -> i64 {
    let response = client
        .post(format!("/api/games/{}/sets/{}/boosters", game_id, set_id))
        .header(ContentType::JSON)
        .header(Header::new("Authorization", admin_auth_header()))
        .body(json!({ "name": name, "spec": spec }).to_string())
        .dispatch();

    assert_eq!(response.status(), Status::Ok);

    let body: Value = serde_json::from_str(&response.into_string().unwrap()).unwrap();
    body["id"].as_i64().unwrap()
}

// ============================================================================
// POST /api/games/<game_id>/sets/<set_id>/boosters (Create Booster)
// ============================================================================

#[test]
fn test_create_booster_success() {
    let client = create_test_client();
    let game_id = create_game(&client, "Pokemon TCG");
    let set_id = create_set(&client, game_id, "Base Set");

    let spec = json!([
        [{"attributes": {"Rarity": ["Common"]}, "chance": 1.0}]
    ]);

    let response = client
        .post(format!("/api/games/{}/sets/{}/boosters", game_id, set_id))
        .header(ContentType::JSON)
        .header(Header::new("Authorization", admin_auth_header()))
        .body(json!({ "name": "Booster Pack", "spec": spec }).to_string())
        .dispatch();

    assert_eq!(response.status(), Status::Ok);

    let body: Value = serde_json::from_str(&response.into_string().unwrap()).unwrap();
    assert!(body["id"].as_i64().is_some());
}

#[test]
fn test_create_booster_without_auth() {
    let client = create_test_client();
    let game_id = create_game(&client, "Pokemon TCG");
    let set_id = create_set(&client, game_id, "Base Set");

    let spec = json!([
        [{"attributes": {"Rarity": ["Common"]}, "chance": 1.0}]
    ]);

    let response = client
        .post(format!("/api/games/{}/sets/{}/boosters", game_id, set_id))
        .header(ContentType::JSON)
        .body(json!({ "name": "Booster Pack", "spec": spec }).to_string())
        .dispatch();

    assert_eq!(response.status(), Status::Unauthorized);
}

#[test]
fn test_create_booster_set_not_found() {
    let client = create_test_client();
    let game_id = create_game(&client, "Pokemon TCG");

    let spec = json!([
        [{"attributes": {"Rarity": ["Common"]}, "chance": 1.0}]
    ]);

    let response = client
        .post(format!("/api/games/{}/sets/99999/boosters", game_id))
        .header(ContentType::JSON)
        .header(Header::new("Authorization", admin_auth_header()))
        .body(json!({ "name": "Booster Pack", "spec": spec }).to_string())
        .dispatch();

    assert_eq!(response.status(), Status::NotFound);

    let body: Value = serde_json::from_str(&response.into_string().unwrap()).unwrap();
    assert_eq!(body["error"], "Set not found");
}

#[test]
fn test_create_booster_duplicate_name() {
    let client = create_test_client();
    let game_id = create_game(&client, "Pokemon TCG");
    let set_id = create_set(&client, game_id, "Base Set");

    let spec = json!([
        [{"attributes": {"Rarity": ["Common"]}, "chance": 1.0}]
    ]);

    // Create first booster
    create_booster(&client, game_id, set_id, "Booster Pack", spec.clone());

    // Try to create duplicate
    let response = client
        .post(format!("/api/games/{}/sets/{}/boosters", game_id, set_id))
        .header(ContentType::JSON)
        .header(Header::new("Authorization", admin_auth_header()))
        .body(json!({ "name": "Booster Pack", "spec": spec }).to_string())
        .dispatch();

    assert_eq!(response.status(), Status::Conflict);

    let body: Value = serde_json::from_str(&response.into_string().unwrap()).unwrap();
    assert_eq!(body["error"], "Booster name already exists for this set");
}

// ============================================================================
// GET /api/games/<game_id>/sets/<set_id>/boosters (List Boosters)
// ============================================================================

#[test]
fn test_list_boosters_empty() {
    let client = create_test_client();
    let game_id = create_game(&client, "Pokemon TCG");
    let set_id = create_set(&client, game_id, "Base Set");

    let response = client
        .get(format!("/api/games/{}/sets/{}/boosters", game_id, set_id))
        .dispatch();

    assert_eq!(response.status(), Status::Ok);

    let body: Value = serde_json::from_str(&response.into_string().unwrap()).unwrap();
    let boosters = body.as_array().unwrap();
    assert_eq!(boosters.len(), 0);
}

#[test]
fn test_list_boosters_with_data() {
    let client = create_test_client();
    let game_id = create_game(&client, "Pokemon TCG");
    let set_id = create_set(&client, game_id, "Base Set");

    let spec = json!([
        [{"attributes": {"Rarity": ["Common"]}, "chance": 1.0}]
    ]);

    create_booster(&client, game_id, set_id, "Booster Pack 1", spec.clone());
    create_booster(&client, game_id, set_id, "Booster Pack 2", spec.clone());

    let response = client
        .get(format!("/api/games/{}/sets/{}/boosters", game_id, set_id))
        .dispatch();

    assert_eq!(response.status(), Status::Ok);

    let body: Value = serde_json::from_str(&response.into_string().unwrap()).unwrap();
    let boosters = body.as_array().unwrap();
    assert_eq!(boosters.len(), 2);

    let names: Vec<&str> = boosters
        .iter()
        .map(|b| b["name"].as_str().unwrap())
        .collect();
    assert!(names.contains(&"Booster Pack 1"));
    assert!(names.contains(&"Booster Pack 2"));
}

#[test]
fn test_list_boosters_only_for_specified_set() {
    let client = create_test_client();
    let game_id = create_game(&client, "Pokemon TCG");
    let set1_id = create_set(&client, game_id, "Base Set");
    let set2_id = create_set(&client, game_id, "Jungle");

    let spec = json!([
        [{"attributes": {"Rarity": ["Common"]}, "chance": 1.0}]
    ]);

    create_booster(&client, game_id, set1_id, "Base Booster", spec.clone());
    create_booster(&client, game_id, set2_id, "Jungle Booster", spec.clone());

    // List boosters for set 1
    let response = client
        .get(format!("/api/games/{}/sets/{}/boosters", game_id, set1_id))
        .dispatch();

    let body: Value = serde_json::from_str(&response.into_string().unwrap()).unwrap();
    let boosters = body.as_array().unwrap();
    assert_eq!(boosters.len(), 1);
    assert_eq!(boosters[0]["name"], "Base Booster");
}

// ============================================================================
// POST /api/boosters/<booster_id>/open (Open Packs)
// ============================================================================

#[test]
fn test_open_packs_success() {
    let client = create_test_client();
    let game_id = create_game(&client, "Pokemon TCG");
    let set_id = create_set(&client, game_id, "Base Set");

    // Create cards with attributes
    create_cards_with_attributes(
        &client,
        game_id,
        set_id,
        vec![
            ("Pikachu", "025", json!({"Rarity": "Common"})),
            ("Bulbasaur", "001", json!({"Rarity": "Common"})),
            ("Charizard", "006", json!({"Rarity": "Rare"})),
        ],
    );

    // Create booster with 1 common slot and 1 rare slot
    let spec = json!([
        [{"attributes": {"Rarity": ["Common"]}, "chance": 1.0}],
        [{"attributes": {"Rarity": ["Rare"]}, "chance": 1.0}]
    ]);
    let booster_id = create_booster(&client, game_id, set_id, "Booster Pack", spec);

    // Create user and collection
    create_user(&client, "testuser", "test@example.com", "password123");
    let session_id = login(&client, "testuser", "password123");
    let collection_id = create_collection(&client, &session_id, game_id, "My Collection");

    // Open 1 pack
    let response = client
        .post(format!("/api/boosters/{}/open", booster_id))
        .header(ContentType::JSON)
        .cookie(Cookie::new("session_id", session_id.clone()))
        .body(json!({ "collection_id": collection_id, "count": 1 }).to_string())
        .dispatch();

    assert_eq!(response.status(), Status::Ok);

    let body: Value = serde_json::from_str(&response.into_string().unwrap()).unwrap();
    let cards = body["cards"].as_array().unwrap();

    // Should have 2 cards (1 common + 1 rare)
    let total_quantity: i64 = cards.iter().map(|c| c["quantity"].as_i64().unwrap()).sum();
    assert_eq!(total_quantity, 2);

    // Verify cards were added to collection
    let collection_response = client
        .get(format!("/api/collections/{}/cards", collection_id))
        .cookie(Cookie::new("session_id", session_id))
        .dispatch();

    let collection_body: Value =
        serde_json::from_str(&collection_response.into_string().unwrap()).unwrap();
    let collection_cards = collection_body.as_array().unwrap();

    let collection_total: i64 = collection_cards
        .iter()
        .map(|c| c["quantity"].as_i64().unwrap())
        .sum();
    assert_eq!(collection_total, 2);
}

#[test]
fn test_open_packs_multiple() {
    let client = create_test_client();
    let game_id = create_game(&client, "Pokemon TCG");
    let set_id = create_set(&client, game_id, "Base Set");

    // Create cards
    create_cards_with_attributes(
        &client,
        game_id,
        set_id,
        vec![
            ("Pikachu", "025", json!({"Rarity": "Common"})),
            ("Bulbasaur", "001", json!({"Rarity": "Common"})),
            ("Squirtle", "007", json!({"Rarity": "Common"})),
        ],
    );

    // Create booster with 2 common slots
    let spec = json!([
        [{"attributes": {"Rarity": ["Common"]}, "chance": 1.0}],
        [{"attributes": {"Rarity": ["Common"]}, "chance": 1.0}]
    ]);
    let booster_id = create_booster(&client, game_id, set_id, "Booster Pack", spec);

    create_user(&client, "testuser", "test@example.com", "password123");
    let session_id = login(&client, "testuser", "password123");
    let collection_id = create_collection(&client, &session_id, game_id, "My Collection");

    // Open 3 packs
    let response = client
        .post(format!("/api/boosters/{}/open", booster_id))
        .header(ContentType::JSON)
        .cookie(Cookie::new("session_id", session_id.clone()))
        .body(json!({ "collection_id": collection_id, "count": 3 }).to_string())
        .dispatch();

    assert_eq!(response.status(), Status::Ok);

    let body: Value = serde_json::from_str(&response.into_string().unwrap()).unwrap();
    let cards = body["cards"].as_array().unwrap();

    // Should have 6 cards total (2 per pack * 3 packs)
    let total_quantity: i64 = cards.iter().map(|c| c["quantity"].as_i64().unwrap()).sum();
    assert_eq!(total_quantity, 6);
}

#[test]
fn test_open_packs_without_auth() {
    let client = create_test_client();

    let response = client
        .post("/api/boosters/1/open")
        .header(ContentType::JSON)
        .body(json!({ "collection_id": 1, "count": 1 }).to_string())
        .dispatch();

    assert_eq!(response.status(), Status::Unauthorized);
}

#[test]
fn test_open_packs_booster_not_found() {
    let client = create_test_client();
    let game_id = create_game(&client, "Pokemon TCG");

    create_user(&client, "testuser", "test@example.com", "password123");
    let session_id = login(&client, "testuser", "password123");
    let collection_id = create_collection(&client, &session_id, game_id, "My Collection");

    let response = client
        .post("/api/boosters/99999/open")
        .header(ContentType::JSON)
        .cookie(Cookie::new("session_id", session_id))
        .body(json!({ "collection_id": collection_id, "count": 1 }).to_string())
        .dispatch();

    assert_eq!(response.status(), Status::NotFound);

    let body: Value = serde_json::from_str(&response.into_string().unwrap()).unwrap();
    assert_eq!(body["error"], "Booster not found");
}

#[test]
fn test_open_packs_collection_not_found() {
    let client = create_test_client();
    let game_id = create_game(&client, "Pokemon TCG");
    let set_id = create_set(&client, game_id, "Base Set");

    create_cards_with_attributes(
        &client,
        game_id,
        set_id,
        vec![("Pikachu", "025", json!({"Rarity": "Common"}))],
    );

    let spec = json!([
        [{"attributes": {"Rarity": ["Common"]}, "chance": 1.0}]
    ]);
    let booster_id = create_booster(&client, game_id, set_id, "Booster Pack", spec);

    create_user(&client, "testuser", "test@example.com", "password123");
    let session_id = login(&client, "testuser", "password123");

    let response = client
        .post(format!("/api/boosters/{}/open", booster_id))
        .header(ContentType::JSON)
        .cookie(Cookie::new("session_id", session_id))
        .body(json!({ "collection_id": 99999, "count": 1 }).to_string())
        .dispatch();

    assert_eq!(response.status(), Status::NotFound);

    let body: Value = serde_json::from_str(&response.into_string().unwrap()).unwrap();
    assert_eq!(body["error"], "Collection not found");
}

#[test]
fn test_open_packs_not_owner() {
    let client = create_test_client();
    let game_id = create_game(&client, "Pokemon TCG");
    let set_id = create_set(&client, game_id, "Base Set");

    create_cards_with_attributes(
        &client,
        game_id,
        set_id,
        vec![("Pikachu", "025", json!({"Rarity": "Common"}))],
    );

    let spec = json!([
        [{"attributes": {"Rarity": ["Common"]}, "chance": 1.0}]
    ]);
    let booster_id = create_booster(&client, game_id, set_id, "Booster Pack", spec);

    // User 1 creates a collection
    create_user(&client, "user1", "user1@example.com", "password123");
    let session1 = login(&client, "user1", "password123");
    let collection_id = create_collection(&client, &session1, game_id, "User1 Collection");

    // User 2 tries to open packs into user 1's collection
    create_user(&client, "user2", "user2@example.com", "password123");
    let session2 = login(&client, "user2", "password123");

    let response = client
        .post(format!("/api/boosters/{}/open", booster_id))
        .header(ContentType::JSON)
        .cookie(Cookie::new("session_id", session2))
        .body(json!({ "collection_id": collection_id, "count": 1 }).to_string())
        .dispatch();

    assert_eq!(response.status(), Status::Forbidden);

    let body: Value = serde_json::from_str(&response.into_string().unwrap()).unwrap();
    assert_eq!(body["error"], "Access denied");
}

#[test]
fn test_open_packs_game_mismatch() {
    let client = create_test_client();
    let pokemon_id = create_game(&client, "Pokemon TCG");
    let mtg_id = create_game(&client, "Magic: The Gathering");

    let pokemon_set_id = create_set(&client, pokemon_id, "Base Set");

    create_cards_with_attributes(
        &client,
        pokemon_id,
        pokemon_set_id,
        vec![("Pikachu", "025", json!({"Rarity": "Common"}))],
    );

    let spec = json!([
        [{"attributes": {"Rarity": ["Common"]}, "chance": 1.0}]
    ]);
    let booster_id = create_booster(&client, pokemon_id, pokemon_set_id, "Booster Pack", spec);

    create_user(&client, "testuser", "test@example.com", "password123");
    let session_id = login(&client, "testuser", "password123");

    // Create MTG collection (different game)
    let collection_id = create_collection(&client, &session_id, mtg_id, "My MTG Collection");

    // Try to open Pokemon booster into MTG collection
    let response = client
        .post(format!("/api/boosters/{}/open", booster_id))
        .header(ContentType::JSON)
        .cookie(Cookie::new("session_id", session_id))
        .body(json!({ "collection_id": collection_id, "count": 1 }).to_string())
        .dispatch();

    assert_eq!(response.status(), Status::BadRequest);

    let body: Value = serde_json::from_str(&response.into_string().unwrap()).unwrap();
    assert_eq!(body["error"], "Collection does not match booster's game");
}

#[test]
fn test_open_packs_adds_to_existing_collection_cards() {
    let client = create_test_client();
    let game_id = create_game(&client, "Pokemon TCG");
    let set_id = create_set(&client, game_id, "Base Set");

    // Create only one card so we know which one will be pulled
    let card_ids = create_cards_with_attributes(
        &client,
        game_id,
        set_id,
        vec![("Pikachu", "025", json!({"Rarity": "Common"}))],
    );

    let spec = json!([
        [{"attributes": {"Rarity": ["Common"]}, "chance": 1.0}]
    ]);
    let booster_id = create_booster(&client, game_id, set_id, "Booster Pack", spec);

    create_user(&client, "testuser", "test@example.com", "password123");
    let session_id = login(&client, "testuser", "password123");
    let collection_id = create_collection(&client, &session_id, game_id, "My Collection");

    // Add 2 Pikachu to collection manually
    client
        .patch(format!("/api/collections/{}/cards", collection_id))
        .header(ContentType::JSON)
        .cookie(Cookie::new("session_id", session_id.clone()))
        .body(json!([{ "card_id": card_ids[0], "quantity": 2 }]).to_string())
        .dispatch();

    // Open 1 pack (will get 1 more Pikachu)
    client
        .post(format!("/api/boosters/{}/open", booster_id))
        .header(ContentType::JSON)
        .cookie(Cookie::new("session_id", session_id.clone()))
        .body(json!({ "collection_id": collection_id, "count": 1 }).to_string())
        .dispatch();

    // Check collection now has 3 Pikachu
    let response = client
        .get(format!("/api/collections/{}/cards", collection_id))
        .cookie(Cookie::new("session_id", session_id))
        .dispatch();

    let body: Value = serde_json::from_str(&response.into_string().unwrap()).unwrap();
    let cards = body.as_array().unwrap();

    assert_eq!(cards.len(), 1);
    assert_eq!(cards[0]["name"], "Pikachu");
    assert_eq!(cards[0]["quantity"], 3);
}

#[test]
fn test_open_packs_zero_count() {
    let client = create_test_client();
    let game_id = create_game(&client, "Pokemon TCG");
    let set_id = create_set(&client, game_id, "Base Set");

    create_cards_with_attributes(
        &client,
        game_id,
        set_id,
        vec![("Pikachu", "025", json!({"Rarity": "Common"}))],
    );

    let spec = json!([
        [{"attributes": {"Rarity": ["Common"]}, "chance": 1.0}]
    ]);
    let booster_id = create_booster(&client, game_id, set_id, "Booster Pack", spec);

    create_user(&client, "testuser", "test@example.com", "password123");
    let session_id = login(&client, "testuser", "password123");
    let collection_id = create_collection(&client, &session_id, game_id, "My Collection");

    let response = client
        .post(format!("/api/boosters/{}/open", booster_id))
        .header(ContentType::JSON)
        .cookie(Cookie::new("session_id", session_id))
        .body(json!({ "collection_id": collection_id, "count": 0 }).to_string())
        .dispatch();

    assert_eq!(response.status(), Status::BadRequest);

    let body: Value = serde_json::from_str(&response.into_string().unwrap()).unwrap();
    assert_eq!(body["error"], "Count must be positive");
}

#[test]
fn test_open_packs_with_weighted_chances() {
    let client = create_test_client();
    let game_id = create_game(&client, "Pokemon TCG");
    let set_id = create_set(&client, game_id, "Base Set");

    create_cards_with_attributes(
        &client,
        game_id,
        set_id,
        vec![
            ("Common Card", "001", json!({"Rarity": "Common"})),
            ("Rare Card", "002", json!({"Rarity": "Rare"})),
            ("Ultra Rare Card", "003", json!({"Rarity": "Ultra Rare"})),
        ],
    );

    // Spec with weighted chances: 80% common, 15% rare, 5% ultra rare
    let spec = json!([
        [
            {"attributes": {"Rarity": ["Common"]}, "chance": 0.8},
            {"attributes": {"Rarity": ["Rare"]}, "chance": 0.15},
            {"attributes": {"Rarity": ["Ultra Rare"]}, "chance": 0.05}
        ]
    ]);
    let booster_id = create_booster(&client, game_id, set_id, "Booster Pack", spec);

    create_user(&client, "testuser", "test@example.com", "password123");
    let session_id = login(&client, "testuser", "password123");
    let collection_id = create_collection(&client, &session_id, game_id, "My Collection");

    // Open many packs to test weighted distribution
    let response = client
        .post(format!("/api/boosters/{}/open", booster_id))
        .header(ContentType::JSON)
        .cookie(Cookie::new("session_id", session_id))
        .body(json!({ "collection_id": collection_id, "count": 100 }).to_string())
        .dispatch();

    assert_eq!(response.status(), Status::Ok);

    let body: Value = serde_json::from_str(&response.into_string().unwrap()).unwrap();
    let cards = body["cards"].as_array().unwrap();

    // Just verify we got cards - exact distribution is random
    let total_quantity: i64 = cards.iter().map(|c| c["quantity"].as_i64().unwrap()).sum();
    assert_eq!(total_quantity, 100);
}
