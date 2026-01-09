use rocket::http::{ContentType, Header, Status};
use rocket::local::blocking::Client;
use serde_json::{Value, json};

use crate::test_helpers::{admin_auth_header, create_test_client};

// ============================================================================
// Helper Functions
// ============================================================================

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

fn create_cards(client: &Client, game_id: i64, set_id: i64, cards: Value) -> Value {
    let response = client
        .post(format!("/api/games/{}/sets/{}/cards", game_id, set_id))
        .header(ContentType::JSON)
        .header(Header::new("Authorization", admin_auth_header()))
        .body(cards.to_string())
        .dispatch();

    assert_eq!(response.status(), Status::Ok);
    serde_json::from_str(&response.into_string().unwrap()).unwrap()
}

// ============================================================================
// POST /api/games/<game_id>/sets/<set_id>/cards (Create Cards - Batch)
// ============================================================================

#[test]
fn test_create_single_card() {
    let client = create_test_client();
    let game_id = create_game(&client, "Pokemon TCG");
    let set_id = create_set(&client, game_id, "Shrouded Fable");

    let response = client
        .post(format!("/api/games/{}/sets/{}/cards", game_id, set_id))
        .header(ContentType::JSON)
        .header(Header::new("Authorization", admin_auth_header()))
        .body(json!([{ "name": "Pikachu", "collector_number": "025" }]).to_string())
        .dispatch();

    assert_eq!(response.status(), Status::Ok);

    let body: Value = serde_json::from_str(&response.into_string().unwrap()).unwrap();
    let ids = body["ids"].as_array().unwrap();
    assert_eq!(ids.len(), 1);
    assert!(ids[0].as_i64().unwrap() > 0);
}

#[test]
fn test_create_multiple_cards() {
    let client = create_test_client();
    let game_id = create_game(&client, "Pokemon TCG");
    let set_id = create_set(&client, game_id, "Shrouded Fable");

    let cards = json!([
        { "name": "Pikachu", "collector_number": "025" },
        { "name": "Charizard", "collector_number": "006" },
        { "name": "Bulbasaur", "collector_number": "001" }
    ]);

    let response = client
        .post(format!("/api/games/{}/sets/{}/cards", game_id, set_id))
        .header(ContentType::JSON)
        .header(Header::new("Authorization", admin_auth_header()))
        .body(cards.to_string())
        .dispatch();

    assert_eq!(response.status(), Status::Ok);

    let body: Value = serde_json::from_str(&response.into_string().unwrap()).unwrap();
    let ids = body["ids"].as_array().unwrap();
    assert_eq!(ids.len(), 3);
}

#[test]
fn test_create_card_with_all_fields() {
    let client = create_test_client();
    let game_id = create_game(&client, "Pokemon TCG");
    let set_id = create_set(&client, game_id, "Shrouded Fable");

    let cards = json!([{
        "name": "Pikachu",
        "collector_number": "025",
        "image_url": "https://example.com/pikachu.png",
        "attributes": {
            "rarity": "common",
            "type": "electric"
        }
    }]);

    let body = create_cards(&client, game_id, set_id, cards);
    let card_id = body["ids"][0].as_i64().unwrap();

    // Verify the card was created with all fields
    let response = client
        .get(format!(
            "/api/games/{}/sets/{}/cards/{}",
            game_id, set_id, card_id
        ))
        .dispatch();

    assert_eq!(response.status(), Status::Ok);

    let card: Value = serde_json::from_str(&response.into_string().unwrap()).unwrap();
    assert_eq!(card["name"], "Pikachu");
    assert_eq!(card["collector_number"], "025");
    assert_eq!(card["image_url"], "https://example.com/pikachu.png");
    assert_eq!(card["attributes"]["rarity"], "common");
    assert_eq!(card["attributes"]["type"], "electric");
}

#[test]
fn test_create_cards_set_not_found() {
    let client = create_test_client();
    let game_id = create_game(&client, "Pokemon TCG");

    let response = client
        .post(format!("/api/games/{}/sets/99999/cards", game_id))
        .header(ContentType::JSON)
        .header(Header::new("Authorization", admin_auth_header()))
        .body(json!([{ "name": "Pikachu", "collector_number": "025" }]).to_string())
        .dispatch();

    assert_eq!(response.status(), Status::NotFound);

    let body: Value = serde_json::from_str(&response.into_string().unwrap()).unwrap();
    assert_eq!(body["error"], "Set not found");
}

#[test]
fn test_create_cards_without_auth() {
    let client = create_test_client();
    let game_id = create_game(&client, "Pokemon TCG");
    let set_id = create_set(&client, game_id, "Shrouded Fable");

    let response = client
        .post(format!("/api/games/{}/sets/{}/cards", game_id, set_id))
        .header(ContentType::JSON)
        .body(json!([{ "name": "Pikachu", "collector_number": "025" }]).to_string())
        .dispatch();

    assert_eq!(response.status(), Status::Unauthorized);
}

#[test]
fn test_create_card_empty_attributes() {
    let client = create_test_client();
    let game_id = create_game(&client, "Pokemon TCG");
    let set_id = create_set(&client, game_id, "Shrouded Fable");

    // Card without attributes specified
    let cards = json!([{ "name": "Pikachu", "collector_number": "025" }]);
    let body = create_cards(&client, game_id, set_id, cards);
    let card_id = body["ids"][0].as_i64().unwrap();

    let response = client
        .get(format!(
            "/api/games/{}/sets/{}/cards/{}",
            game_id, set_id, card_id
        ))
        .dispatch();

    let card: Value = serde_json::from_str(&response.into_string().unwrap()).unwrap();
    // Empty attributes should be an empty object
    assert!(card["attributes"].as_object().unwrap().is_empty());
    // collector_number should still be present
    assert_eq!(card["collector_number"], "025");
}

// ============================================================================
// GET /api/games/<game_id>/sets/<set_id>/cards/<card_id> (Get Single Card)
// ============================================================================

#[test]
fn test_get_card_success() {
    let client = create_test_client();
    let game_id = create_game(&client, "Pokemon TCG");
    let set_id = create_set(&client, game_id, "Shrouded Fable");

    let cards = json!([{
        "name": "Pikachu",
        "collector_number": "025",
        "image_url": "https://example.com/pikachu.png",
        "attributes": { "rarity": "common" }
    }]);
    let body = create_cards(&client, game_id, set_id, cards);
    let card_id = body["ids"][0].as_i64().unwrap();

    let response = client
        .get(format!(
            "/api/games/{}/sets/{}/cards/{}",
            game_id, set_id, card_id
        ))
        .dispatch();

    assert_eq!(response.status(), Status::Ok);

    let card: Value = serde_json::from_str(&response.into_string().unwrap()).unwrap();
    assert_eq!(card["id"], card_id);
    assert_eq!(card["name"], "Pikachu");
    assert_eq!(card["collector_number"], "025");
    assert_eq!(card["image_url"], "https://example.com/pikachu.png");
    assert_eq!(card["attributes"]["rarity"], "common");
}

#[test]
fn test_get_card_not_found() {
    let client = create_test_client();
    let game_id = create_game(&client, "Pokemon TCG");
    let set_id = create_set(&client, game_id, "Shrouded Fable");

    let response = client
        .get(format!(
            "/api/games/{}/sets/{}/cards/99999",
            game_id, set_id
        ))
        .dispatch();

    assert_eq!(response.status(), Status::NotFound);

    let body: Value = serde_json::from_str(&response.into_string().unwrap()).unwrap();
    assert_eq!(body["error"], "Card not found");
}

#[test]
fn test_get_card_no_auth_required() {
    let client = create_test_client();
    let game_id = create_game(&client, "Pokemon TCG");
    let set_id = create_set(&client, game_id, "Shrouded Fable");

    let cards = json!([{ "name": "Pikachu", "collector_number": "025" }]);
    let body = create_cards(&client, game_id, set_id, cards);
    let card_id = body["ids"][0].as_i64().unwrap();

    // No auth header - should still work
    let response = client
        .get(format!(
            "/api/games/{}/sets/{}/cards/{}",
            game_id, set_id, card_id
        ))
        .dispatch();

    assert_eq!(response.status(), Status::Ok);
}

// ============================================================================
// GET /api/games/<game_id>/sets/<set_id>/cards (List Cards)
// ============================================================================

#[test]
fn test_list_cards_empty() {
    let client = create_test_client();
    let game_id = create_game(&client, "Pokemon TCG");
    let set_id = create_set(&client, game_id, "Shrouded Fable");

    let response = client
        .get(format!("/api/games/{}/sets/{}/cards", game_id, set_id))
        .dispatch();

    assert_eq!(response.status(), Status::Ok);

    let body: Value = serde_json::from_str(&response.into_string().unwrap()).unwrap();
    assert!(body.as_array().unwrap().is_empty());
}

#[test]
fn test_list_cards_multiple() {
    let client = create_test_client();
    let game_id = create_game(&client, "Pokemon TCG");
    let set_id = create_set(&client, game_id, "Shrouded Fable");

    let cards = json!([
        { "name": "Pikachu", "collector_number": "025" },
        { "name": "Charizard", "collector_number": "006" },
        { "name": "Bulbasaur", "collector_number": "001" }
    ]);
    create_cards(&client, game_id, set_id, cards);

    let response = client
        .get(format!("/api/games/{}/sets/{}/cards", game_id, set_id))
        .dispatch();

    assert_eq!(response.status(), Status::Ok);

    let body: Value = serde_json::from_str(&response.into_string().unwrap()).unwrap();
    let cards = body.as_array().unwrap();

    assert_eq!(cards.len(), 3);

    let names: Vec<&str> = cards.iter().map(|c| c["name"].as_str().unwrap()).collect();
    assert!(names.contains(&"Pikachu"));
    assert!(names.contains(&"Charizard"));
    assert!(names.contains(&"Bulbasaur"));
}

#[test]
fn test_list_cards_only_for_specified_set() {
    let client = create_test_client();
    let game_id = create_game(&client, "Pokemon TCG");
    let set1_id = create_set(&client, game_id, "Shrouded Fable");
    let set2_id = create_set(&client, game_id, "Surging Sparks");

    create_cards(
        &client,
        game_id,
        set1_id,
        json!([{ "name": "Pikachu", "collector_number": "025" }]),
    );
    create_cards(
        &client,
        game_id,
        set2_id,
        json!([{ "name": "Charizard", "collector_number": "006" }]),
    );

    // List cards for set 1
    let response = client
        .get(format!("/api/games/{}/sets/{}/cards", game_id, set1_id))
        .dispatch();
    let body: Value = serde_json::from_str(&response.into_string().unwrap()).unwrap();
    let cards = body.as_array().unwrap();

    assert_eq!(cards.len(), 1);
    assert_eq!(cards[0]["name"], "Pikachu");
}

// ============================================================================
// Attributes Tests
// ============================================================================

#[test]
fn test_card_attributes_preserved() {
    let client = create_test_client();
    let game_id = create_game(&client, "Pokemon TCG");
    let set_id = create_set(&client, game_id, "Shrouded Fable");

    let cards = json!([{
        "name": "Pikachu",
        "collector_number": "025",
        "attributes": {
            "rarity": "common",
            "type": "electric",
            "hp": "60"
        }
    }]);
    let body = create_cards(&client, game_id, set_id, cards);
    let card_id = body["ids"][0].as_i64().unwrap();

    let response = client
        .get(format!(
            "/api/games/{}/sets/{}/cards/{}",
            game_id, set_id, card_id
        ))
        .dispatch();

    let card: Value = serde_json::from_str(&response.into_string().unwrap()).unwrap();
    let attrs = card["attributes"].as_object().unwrap();

    assert_eq!(attrs.len(), 3);
    assert_eq!(attrs["rarity"], "common");
    assert_eq!(attrs["type"], "electric");
    assert_eq!(attrs["hp"], "60");
}
