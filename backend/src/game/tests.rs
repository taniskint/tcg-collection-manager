use rocket::http::{ContentType, Header, Status};
use rocket::local::blocking::Client;
use serde_json::{json, Value};

use crate::test_helpers::{admin_auth_header, create_test_client, TEST_ADMIN_KEY};

// ============================================================================
// Helper Functions
// ============================================================================

fn create_game(client: &Client, name: &str, image_url: Option<&str>) -> Value {
    let body = match image_url {
        Some(url) => json!({ "name": name, "image_url": url }),
        None => json!({ "name": name }),
    };

    let response = client
        .post("/api/games")
        .header(ContentType::JSON)
        .header(Header::new("Authorization", admin_auth_header()))
        .body(body.to_string())
        .dispatch();

    assert_eq!(response.status(), Status::Ok);
    serde_json::from_str(&response.into_string().unwrap()).unwrap()
}

// ============================================================================
// POST /api/games (Create Game)
// ============================================================================

#[test]
fn test_create_game_success() {
    let client = create_test_client();

    let response = client
        .post("/api/games")
        .header(ContentType::JSON)
        .header(Header::new("Authorization", admin_auth_header()))
        .body(json!({ "name": "Pokemon TCG" }).to_string())
        .dispatch();

    assert_eq!(response.status(), Status::Ok);

    let body: Value = serde_json::from_str(&response.into_string().unwrap()).unwrap();
    assert!(body["id"].as_i64().is_some());
    assert!(body["id"].as_i64().unwrap() > 0);
}

#[test]
fn test_create_game_with_image_url() {
    let client = create_test_client();

    let response = client
        .post("/api/games")
        .header(ContentType::JSON)
        .header(Header::new("Authorization", admin_auth_header()))
        .body(
            json!({
                "name": "Magic: The Gathering",
                "image_url": "https://example.com/mtg.png"
            })
            .to_string(),
        )
        .dispatch();

    assert_eq!(response.status(), Status::Ok);

    let body: Value = serde_json::from_str(&response.into_string().unwrap()).unwrap();
    let game_id = body["id"].as_i64().unwrap();

    // Verify the game was created with the image URL
    let get_response = client.get(format!("/api/games/{}", game_id)).dispatch();
    let game: Value = serde_json::from_str(&get_response.into_string().unwrap()).unwrap();
    assert_eq!(game["image_url"], "https://example.com/mtg.png");
}

#[test]
fn test_create_game_duplicate_name() {
    let client = create_test_client();

    // Create first game
    create_game(&client, "Pokemon TCG", None);

    // Try to create duplicate
    let response = client
        .post("/api/games")
        .header(ContentType::JSON)
        .header(Header::new("Authorization", admin_auth_header()))
        .body(json!({ "name": "Pokemon TCG" }).to_string())
        .dispatch();

    assert_eq!(response.status(), Status::Conflict);

    let body: Value = serde_json::from_str(&response.into_string().unwrap()).unwrap();
    assert_eq!(body["error"], "Game already exists");
}

// ============================================================================
// Admin Authentication Tests
// ============================================================================

#[test]
fn test_create_game_without_auth() {
    let client = create_test_client();

    let response = client
        .post("/api/games")
        .header(ContentType::JSON)
        .body(json!({ "name": "Pokemon TCG" }).to_string())
        .dispatch();

    assert_eq!(response.status(), Status::Unauthorized);
}

#[test]
fn test_create_game_with_invalid_auth() {
    let client = create_test_client();

    let response = client
        .post("/api/games")
        .header(ContentType::JSON)
        .header(Header::new("Authorization", "Bearer wrong-key"))
        .body(json!({ "name": "Pokemon TCG" }).to_string())
        .dispatch();

    assert_eq!(response.status(), Status::Unauthorized);
}

#[test]
fn test_create_game_with_malformed_auth_header() {
    let client = create_test_client();

    // Missing "Bearer " prefix
    let response = client
        .post("/api/games")
        .header(ContentType::JSON)
        .header(Header::new("Authorization", TEST_ADMIN_KEY))
        .body(json!({ "name": "Pokemon TCG" }).to_string())
        .dispatch();

    assert_eq!(response.status(), Status::Unauthorized);
}

// ============================================================================
// GET /api/games/<id> (Get Single Game)
// ============================================================================

#[test]
fn test_get_game_success() {
    let client = create_test_client();

    // Create a game first
    let create_response = create_game(&client, "Pokemon TCG", Some("https://example.com/pokemon.png"));
    let game_id = create_response["id"].as_i64().unwrap();

    // Get the game
    let response = client.get(format!("/api/games/{}", game_id)).dispatch();

    assert_eq!(response.status(), Status::Ok);

    let body: Value = serde_json::from_str(&response.into_string().unwrap()).unwrap();
    assert_eq!(body["id"], game_id);
    assert_eq!(body["name"], "Pokemon TCG");
    assert_eq!(body["image_url"], "https://example.com/pokemon.png");
    assert_eq!(body["set_count"], 0);
}

#[test]
fn test_get_game_not_found() {
    let client = create_test_client();

    let response = client.get("/api/games/99999").dispatch();

    assert_eq!(response.status(), Status::NotFound);

    let body: Value = serde_json::from_str(&response.into_string().unwrap()).unwrap();
    assert_eq!(body["error"], "Game not found");
}

#[test]
fn test_get_game_no_auth_required() {
    // GET endpoints should be publicly accessible
    let client = create_test_client();

    let create_response = create_game(&client, "Pokemon TCG", None);
    let game_id = create_response["id"].as_i64().unwrap();

    // No auth header - should still work
    let response = client.get(format!("/api/games/{}", game_id)).dispatch();

    assert_eq!(response.status(), Status::Ok);
}

// ============================================================================
// GET /api/games (List Games)
// ============================================================================

#[test]
fn test_list_games_empty() {
    let client = create_test_client();

    let response = client.get("/api/games").dispatch();

    assert_eq!(response.status(), Status::Ok);

    let body: Value = serde_json::from_str(&response.into_string().unwrap()).unwrap();
    assert!(body.as_array().unwrap().is_empty());
}

#[test]
fn test_list_games_multiple() {
    let client = create_test_client();

    // Create multiple games
    create_game(&client, "Pokemon TCG", None);
    create_game(&client, "Magic: The Gathering", None);
    create_game(&client, "Yu-Gi-Oh!", None);

    let response = client.get("/api/games").dispatch();

    assert_eq!(response.status(), Status::Ok);

    let body: Value = serde_json::from_str(&response.into_string().unwrap()).unwrap();
    let games = body.as_array().unwrap();

    assert_eq!(games.len(), 3);

    // Verify all games are present (order may vary)
    let names: Vec<&str> = games.iter().map(|g| g["name"].as_str().unwrap()).collect();
    assert!(names.contains(&"Pokemon TCG"));
    assert!(names.contains(&"Magic: The Gathering"));
    assert!(names.contains(&"Yu-Gi-Oh!"));
}

#[test]
fn test_list_games_no_auth_required() {
    let client = create_test_client();

    create_game(&client, "Pokemon TCG", None);

    // No auth header - should still work
    let response = client.get("/api/games").dispatch();

    assert_eq!(response.status(), Status::Ok);

    let body: Value = serde_json::from_str(&response.into_string().unwrap()).unwrap();
    assert_eq!(body.as_array().unwrap().len(), 1);
}

// ============================================================================
// Database Isolation Tests
// ============================================================================

#[test]
fn test_database_isolation() {
    // Each test client should have its own isolated database
    let client1 = create_test_client();
    let client2 = create_test_client();

    // Create game in client1
    create_game(&client1, "Pokemon TCG", None);

    // client2 should not see it
    let response = client2.get("/api/games").dispatch();
    let body: Value = serde_json::from_str(&response.into_string().unwrap()).unwrap();

    assert!(body.as_array().unwrap().is_empty());
}
