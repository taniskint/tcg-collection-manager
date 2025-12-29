use rocket::http::{ContentType, Header, Status};
use rocket::local::blocking::Client;
use serde_json::{json, Value};

use crate::test_helpers::{admin_auth_header, create_test_client};

// ============================================================================
// Helper Functions
// ============================================================================

fn create_game(client: &Client, name: &str) -> i64 {
    let response = client
        .post("/games")
        .header(ContentType::JSON)
        .header(Header::new("Authorization", admin_auth_header()))
        .body(json!({ "name": name }).to_string())
        .dispatch();

    assert_eq!(response.status(), Status::Ok);
    let body: Value = serde_json::from_str(&response.into_string().unwrap()).unwrap();
    body["id"].as_i64().unwrap()
}

fn create_set(client: &Client, game_id: i64, name: &str, image_url: Option<&str>) -> Value {
    let body = match image_url {
        Some(url) => json!({ "name": name, "image_url": url }),
        None => json!({ "name": name }),
    };

    let response = client
        .post(format!("/games/{}/sets", game_id))
        .header(ContentType::JSON)
        .header(Header::new("Authorization", admin_auth_header()))
        .body(body.to_string())
        .dispatch();

    assert_eq!(response.status(), Status::Ok);
    serde_json::from_str(&response.into_string().unwrap()).unwrap()
}

// ============================================================================
// POST /games/<game_id>/sets (Create Set)
// ============================================================================

#[test]
fn test_create_set_success() {
    let client = create_test_client();
    let game_id = create_game(&client, "Pokemon TCG");

    let response = client
        .post(format!("/games/{}/sets", game_id))
        .header(ContentType::JSON)
        .header(Header::new("Authorization", admin_auth_header()))
        .body(json!({ "name": "Shrouded Fable" }).to_string())
        .dispatch();

    assert_eq!(response.status(), Status::Ok);

    let body: Value = serde_json::from_str(&response.into_string().unwrap()).unwrap();
    assert!(body["id"].as_i64().is_some());
    assert!(body["id"].as_i64().unwrap() > 0);
}

#[test]
fn test_create_set_with_image_url() {
    let client = create_test_client();
    let game_id = create_game(&client, "Pokemon TCG");

    let response = client
        .post(format!("/games/{}/sets", game_id))
        .header(ContentType::JSON)
        .header(Header::new("Authorization", admin_auth_header()))
        .body(
            json!({
                "name": "Shrouded Fable",
                "image_url": "https://example.com/shrouded-fable.png"
            })
            .to_string(),
        )
        .dispatch();

    assert_eq!(response.status(), Status::Ok);

    let body: Value = serde_json::from_str(&response.into_string().unwrap()).unwrap();
    let set_id = body["id"].as_i64().unwrap();

    // Verify the set was created with the image URL
    let get_response = client
        .get(format!("/games/{}/sets/{}", game_id, set_id))
        .dispatch();
    let set: Value = serde_json::from_str(&get_response.into_string().unwrap()).unwrap();
    assert_eq!(set["image_url"], "https://example.com/shrouded-fable.png");
}

#[test]
fn test_create_set_game_not_found() {
    let client = create_test_client();

    let response = client
        .post("/games/99999/sets")
        .header(ContentType::JSON)
        .header(Header::new("Authorization", admin_auth_header()))
        .body(json!({ "name": "Shrouded Fable" }).to_string())
        .dispatch();

    assert_eq!(response.status(), Status::NotFound);

    let body: Value = serde_json::from_str(&response.into_string().unwrap()).unwrap();
    assert_eq!(body["error"], "Game not found");
}

#[test]
fn test_create_set_duplicate_name_same_game() {
    let client = create_test_client();
    let game_id = create_game(&client, "Pokemon TCG");

    // Create first set
    create_set(&client, game_id, "Shrouded Fable", None);

    // Try to create duplicate
    let response = client
        .post(format!("/games/{}/sets", game_id))
        .header(ContentType::JSON)
        .header(Header::new("Authorization", admin_auth_header()))
        .body(json!({ "name": "Shrouded Fable" }).to_string())
        .dispatch();

    assert_eq!(response.status(), Status::Conflict);

    let body: Value = serde_json::from_str(&response.into_string().unwrap()).unwrap();
    assert_eq!(body["error"], "Set name already exists for this game");
}

#[test]
fn test_create_set_same_name_different_games() {
    let client = create_test_client();
    let game1_id = create_game(&client, "Pokemon TCG");
    let game2_id = create_game(&client, "Magic: The Gathering");

    // Create set in game 1
    create_set(&client, game1_id, "Base Set", None);

    // Same name in game 2 should work
    let response = client
        .post(format!("/games/{}/sets", game2_id))
        .header(ContentType::JSON)
        .header(Header::new("Authorization", admin_auth_header()))
        .body(json!({ "name": "Base Set" }).to_string())
        .dispatch();

    assert_eq!(response.status(), Status::Ok);
}

#[test]
fn test_create_set_without_auth() {
    let client = create_test_client();
    let game_id = create_game(&client, "Pokemon TCG");

    let response = client
        .post(format!("/games/{}/sets", game_id))
        .header(ContentType::JSON)
        .body(json!({ "name": "Shrouded Fable" }).to_string())
        .dispatch();

    assert_eq!(response.status(), Status::Unauthorized);
}

// ============================================================================
// GET /games/<game_id>/sets/<set_id> (Get Single Set)
// ============================================================================

#[test]
fn test_get_set_success() {
    let client = create_test_client();
    let game_id = create_game(&client, "Pokemon TCG");
    let create_response = create_set(&client, game_id, "Shrouded Fable", Some("https://example.com/sf.png"));
    let set_id = create_response["id"].as_i64().unwrap();

    let response = client
        .get(format!("/games/{}/sets/{}", game_id, set_id))
        .dispatch();

    assert_eq!(response.status(), Status::Ok);

    let body: Value = serde_json::from_str(&response.into_string().unwrap()).unwrap();
    assert_eq!(body["id"], set_id);
    assert_eq!(body["name"], "Shrouded Fable");
    assert_eq!(body["image_url"], "https://example.com/sf.png");
}

#[test]
fn test_get_set_not_found() {
    let client = create_test_client();
    let game_id = create_game(&client, "Pokemon TCG");

    let response = client
        .get(format!("/games/{}/sets/99999", game_id))
        .dispatch();

    assert_eq!(response.status(), Status::NotFound);

    let body: Value = serde_json::from_str(&response.into_string().unwrap()).unwrap();
    assert_eq!(body["error"], "Set not found");
}

#[test]
fn test_get_set_no_auth_required() {
    let client = create_test_client();
    let game_id = create_game(&client, "Pokemon TCG");
    let create_response = create_set(&client, game_id, "Shrouded Fable", None);
    let set_id = create_response["id"].as_i64().unwrap();

    // No auth header - should still work
    let response = client
        .get(format!("/games/{}/sets/{}", game_id, set_id))
        .dispatch();

    assert_eq!(response.status(), Status::Ok);
}

// ============================================================================
// GET /games/<game_id>/sets (List Sets)
// ============================================================================

#[test]
fn test_list_sets_empty() {
    let client = create_test_client();
    let game_id = create_game(&client, "Pokemon TCG");

    let response = client.get(format!("/games/{}/sets", game_id)).dispatch();

    assert_eq!(response.status(), Status::Ok);

    let body: Value = serde_json::from_str(&response.into_string().unwrap()).unwrap();
    assert!(body.as_array().unwrap().is_empty());
}

#[test]
fn test_list_sets_multiple() {
    let client = create_test_client();
    let game_id = create_game(&client, "Pokemon TCG");

    create_set(&client, game_id, "Shrouded Fable", None);
    create_set(&client, game_id, "Surging Sparks", None);
    create_set(&client, game_id, "Prismatic Evolutions", None);

    let response = client.get(format!("/games/{}/sets", game_id)).dispatch();

    assert_eq!(response.status(), Status::Ok);

    let body: Value = serde_json::from_str(&response.into_string().unwrap()).unwrap();
    let sets = body.as_array().unwrap();

    assert_eq!(sets.len(), 3);

    let names: Vec<&str> = sets.iter().map(|s| s["name"].as_str().unwrap()).collect();
    assert!(names.contains(&"Shrouded Fable"));
    assert!(names.contains(&"Surging Sparks"));
    assert!(names.contains(&"Prismatic Evolutions"));
}

#[test]
fn test_list_sets_only_for_specified_game() {
    let client = create_test_client();
    let game1_id = create_game(&client, "Pokemon TCG");
    let game2_id = create_game(&client, "Magic: The Gathering");

    create_set(&client, game1_id, "Shrouded Fable", None);
    create_set(&client, game2_id, "Innistrad", None);

    // List sets for game 1
    let response = client.get(format!("/games/{}/sets", game1_id)).dispatch();
    let body: Value = serde_json::from_str(&response.into_string().unwrap()).unwrap();
    let sets = body.as_array().unwrap();

    assert_eq!(sets.len(), 1);
    assert_eq!(sets[0]["name"], "Shrouded Fable");
}

// ============================================================================
// Set Count in Game
// ============================================================================

#[test]
fn test_game_set_count_updates() {
    let client = create_test_client();
    let game_id = create_game(&client, "Pokemon TCG");

    // Initially 0 sets
    let response = client.get(format!("/games/{}", game_id)).dispatch();
    let game: Value = serde_json::from_str(&response.into_string().unwrap()).unwrap();
    assert_eq!(game["set_count"], 0);

    // Add sets
    create_set(&client, game_id, "Shrouded Fable", None);
    create_set(&client, game_id, "Surging Sparks", None);

    // Now 2 sets
    let response = client.get(format!("/games/{}", game_id)).dispatch();
    let game: Value = serde_json::from_str(&response.into_string().unwrap()).unwrap();
    assert_eq!(game["set_count"], 2);
}
