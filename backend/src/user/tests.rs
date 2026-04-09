use rocket::http::{ContentType, Status};
use rocket::local::blocking::Client;
use serde_json::{Value, json};

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

    // Extract session_id from cookie
    let cookies = response.cookies();
    let session_cookie = cookies
        .iter()
        .find(|c| c.name() == "session_id")
        .expect("session_id cookie not found");

    session_cookie.value().to_string()
}

fn update_user(
    client: &Client,
    session_id: &str,
    user_id: i64,
    username: Option<&str>,
    email: Option<&str>,
    password: Option<&str>,
    current_password: &str,
) -> Status {
    let mut body = json!({
        "current_password": current_password
    });

    if let Some(u) = username {
        body["username"] = json!(u);
    }
    if let Some(e) = email {
        body["email"] = json!(e);
    }
    if let Some(p) = password {
        body["password"] = json!(p);
    }

    let response = client
        .patch(format!("/api/users/{}", user_id))
        .header(ContentType::JSON)
        .cookie(rocket::http::Cookie::new("session_id", session_id))
        .body(body.to_string())
        .dispatch();

    response.status()
}

fn delete_user(client: &Client, session_id: &str, user_id: i64, password: &str) -> Status {
    let response = client
        .delete(format!("/api/users/{}", user_id))
        .header(ContentType::JSON)
        .cookie(rocket::http::Cookie::new("session_id", session_id))
        .body(
            json!({
                "password": password
            })
            .to_string(),
        )
        .dispatch();

    response.status()
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

#[test]
fn test_create_user_username_too_long() {
    let client = create_test_client();

    // Create a username with 51 characters (exceeds 50 limit)
    let long_username = "a".repeat(51);

    let response = client
        .post("/api/users")
        .header(ContentType::JSON)
        .body(
            json!({
                "username": long_username,
                "email": "test@example.com",
                "password": "password123"
            })
            .to_string(),
        )
        .dispatch();

    assert_eq!(response.status(), Status::BadRequest);

    let body: Value = serde_json::from_str(&response.into_string().unwrap()).unwrap();
    assert_eq!(body["error"], "Username must be 50 characters or less");
}

#[test]
fn test_create_user_password_too_long() {
    let client = create_test_client();

    // Create a password with 201 characters (exceeds 200 limit)
    let long_password = "a".repeat(201);

    let response = client
        .post("/api/users")
        .header(ContentType::JSON)
        .body(
            json!({
                "username": "testuser",
                "email": "test@example.com",
                "password": long_password
            })
            .to_string(),
        )
        .dispatch();

    assert_eq!(response.status(), Status::BadRequest);

    let body: Value = serde_json::from_str(&response.into_string().unwrap()).unwrap();
    assert_eq!(body["error"], "Password must be 200 characters or less");
}

#[test]
fn test_create_user_username_max_length() {
    let client = create_test_client();

    // Create a username with exactly 50 characters (at the limit)
    let max_username = "a".repeat(50);

    let response = client
        .post("/api/users")
        .header(ContentType::JSON)
        .body(
            json!({
                "username": max_username,
                "email": "test@example.com",
                "password": "password123"
            })
            .to_string(),
        )
        .dispatch();

    assert_eq!(response.status(), Status::Ok);
}

#[test]
fn test_create_user_password_max_length() {
    let client = create_test_client();

    // Create a password with exactly 200 characters (at the limit)
    let max_password = "a".repeat(200);

    let response = client
        .post("/api/users")
        .header(ContentType::JSON)
        .body(
            json!({
                "username": "testuser",
                "email": "test@example.com",
                "password": max_password
            })
            .to_string(),
        )
        .dispatch();

    assert_eq!(response.status(), Status::Ok);
}

// ============================================================================
// PATCH /api/users/:id (Update User)
// ============================================================================

#[test]
fn test_update_user_username_success() {
    let client = create_test_client();
    let user = create_user(&client, "alice", "alice@example.com", "password123");
    let user_id = user["id"].as_i64().unwrap();
    let session_id = login(&client, "alice", "password123");

    let status = update_user(
        &client,
        &session_id,
        user_id,
        Some("alice_updated"),
        None,
        None,
        "password123",
    );

    assert_eq!(status, Status::NoContent);

    // Verify login works with new username
    let new_session_id = login(&client, "alice_updated", "password123");
    assert!(!new_session_id.is_empty());
}

#[test]
fn test_update_user_email_success() {
    let client = create_test_client();
    let user = create_user(&client, "alice", "alice@example.com", "password123");
    let user_id = user["id"].as_i64().unwrap();
    let session_id = login(&client, "alice", "password123");

    let status = update_user(
        &client,
        &session_id,
        user_id,
        None,
        Some("newemail@example.com"),
        None,
        "password123",
    );

    assert_eq!(status, Status::NoContent);

    // Verify login works with new email
    let new_session_id = login(&client, "newemail@example.com", "password123");
    assert!(!new_session_id.is_empty());
}

#[test]
fn test_update_user_password_success() {
    let client = create_test_client();
    let user = create_user(&client, "alice", "alice@example.com", "password123");
    let user_id = user["id"].as_i64().unwrap();
    let session_id = login(&client, "alice", "password123");

    let status = update_user(
        &client,
        &session_id,
        user_id,
        None,
        None,
        Some("newpassword456"),
        "password123",
    );

    assert_eq!(status, Status::NoContent);

    // Verify old password no longer works
    let response = client
        .post("/api/sessions")
        .header(ContentType::JSON)
        .body(
            json!({
                "email_or_username": "alice",
                "password": "password123"
            })
            .to_string(),
        )
        .dispatch();

    assert_eq!(response.status(), Status::Unauthorized);

    // Verify new password works
    let new_session_id = login(&client, "alice", "newpassword456");
    assert!(!new_session_id.is_empty());
}

#[test]
fn test_update_user_multiple_fields_success() {
    let client = create_test_client();
    let user = create_user(&client, "alice", "alice@example.com", "password123");
    let user_id = user["id"].as_i64().unwrap();
    let session_id = login(&client, "alice", "password123");

    let status = update_user(
        &client,
        &session_id,
        user_id,
        Some("alice_new"),
        Some("alice_new@example.com"),
        Some("newpassword789"),
        "password123",
    );

    assert_eq!(status, Status::NoContent);

    // Verify login with new credentials
    let new_session_id = login(&client, "alice_new", "newpassword789");
    assert!(!new_session_id.is_empty());
}

#[test]
fn test_update_user_same_username() {
    let client = create_test_client();
    let user = create_user(&client, "alice", "alice@example.com", "password123");
    let user_id = user["id"].as_i64().unwrap();
    let session_id = login(&client, "alice", "password123");

    let status = update_user(
        &client,
        &session_id,
        user_id,
        Some("alice"),
        None,
        None,
        "password123",
    );

    assert_eq!(status, Status::NoContent);
}

#[test]
fn test_update_user_same_email() {
    let client = create_test_client();
    let user = create_user(&client, "alice", "alice@example.com", "password123");
    let user_id = user["id"].as_i64().unwrap();
    let session_id = login(&client, "alice", "password123");

    let status = update_user(
        &client,
        &session_id,
        user_id,
        None,
        Some("alice@example.com"),
        None,
        "password123",
    );

    assert_eq!(status, Status::NoContent);
}

#[test]
fn test_update_user_without_auth() {
    let client = create_test_client();
    let user = create_user(&client, "alice", "alice@example.com", "password123");
    let user_id = user["id"].as_i64().unwrap();

    let response = client
        .patch(format!("/api/users/{}", user_id))
        .header(ContentType::JSON)
        .body(
            json!({
                "username": "alice_new",
                "current_password": "password123"
            })
            .to_string(),
        )
        .dispatch();

    assert_eq!(response.status(), Status::Unauthorized);
}

#[test]
fn test_update_user_not_owner() {
    let client = create_test_client();
    let _user1 = create_user(&client, "alice", "alice@example.com", "password123");
    let user2 = create_user(&client, "bob", "bob@example.com", "password456");
    let user2_id = user2["id"].as_i64().unwrap();
    let session_id = login(&client, "alice", "password123");

    let status = update_user(
        &client,
        &session_id,
        user2_id,
        Some("bob_updated"),
        None,
        None,
        "password456",
    );

    assert_eq!(status, Status::Forbidden);
}

#[test]
fn test_update_user_not_found() {
    let client = create_test_client();
    let _user = create_user(&client, "alice", "alice@example.com", "password123");
    let session_id = login(&client, "alice", "password123");

    let status = update_user(
        &client,
        &session_id,
        999,
        Some("alice_new"),
        None,
        None,
        "password123",
    );

    assert_eq!(status, Status::Forbidden);
}

#[test]
fn test_update_user_wrong_current_password() {
    let client = create_test_client();
    let user = create_user(&client, "alice", "alice@example.com", "password123");
    let user_id = user["id"].as_i64().unwrap();
    let session_id = login(&client, "alice", "password123");

    let status = update_user(
        &client,
        &session_id,
        user_id,
        Some("alice_new"),
        None,
        None,
        "wrongpassword",
    );

    assert_eq!(status, Status::Unauthorized);
}

#[test]
fn test_update_user_duplicate_username() {
    let client = create_test_client();
    let _user1 = create_user(&client, "alice", "alice@example.com", "password123");
    let user2 = create_user(&client, "bob", "bob@example.com", "password456");
    let user2_id = user2["id"].as_i64().unwrap();
    let session_id = login(&client, "bob", "password456");

    let status = update_user(
        &client,
        &session_id,
        user2_id,
        Some("alice"),
        None,
        None,
        "password456",
    );

    assert_eq!(status, Status::Conflict);
}

#[test]
fn test_update_user_duplicate_email() {
    let client = create_test_client();
    let _user1 = create_user(&client, "alice", "alice@example.com", "password123");
    let user2 = create_user(&client, "bob", "bob@example.com", "password456");
    let user2_id = user2["id"].as_i64().unwrap();
    let session_id = login(&client, "bob", "password456");

    let status = update_user(
        &client,
        &session_id,
        user2_id,
        None,
        Some("alice@example.com"),
        None,
        "password456",
    );

    assert_eq!(status, Status::Conflict);
}

#[test]
fn test_update_user_username_too_long() {
    let client = create_test_client();
    let user = create_user(&client, "alice", "alice@example.com", "password123");
    let user_id = user["id"].as_i64().unwrap();
    let session_id = login(&client, "alice", "password123");

    let long_username = "a".repeat(51);

    let response = client
        .patch(format!("/api/users/{}", user_id))
        .header(ContentType::JSON)
        .cookie(rocket::http::Cookie::new("session_id", &session_id))
        .body(
            json!({
                "username": long_username,
                "current_password": "password123"
            })
            .to_string(),
        )
        .dispatch();

    assert_eq!(response.status(), Status::BadRequest);

    let body: Value = serde_json::from_str(&response.into_string().unwrap()).unwrap();
    assert_eq!(body["error"], "Username must be 50 characters or less");
}

#[test]
fn test_update_user_password_too_long() {
    let client = create_test_client();
    let user = create_user(&client, "alice", "alice@example.com", "password123");
    let user_id = user["id"].as_i64().unwrap();
    let session_id = login(&client, "alice", "password123");

    let long_password = "a".repeat(201);

    let response = client
        .patch(format!("/api/users/{}", user_id))
        .header(ContentType::JSON)
        .cookie(rocket::http::Cookie::new("session_id", &session_id))
        .body(
            json!({
                "password": long_password,
                "current_password": "password123"
            })
            .to_string(),
        )
        .dispatch();

    assert_eq!(response.status(), Status::BadRequest);

    let body: Value = serde_json::from_str(&response.into_string().unwrap()).unwrap();
    assert_eq!(body["error"], "Password must be 200 characters or less");
}

#[test]
fn test_update_user_no_fields() {
    let client = create_test_client();
    let user = create_user(&client, "alice", "alice@example.com", "password123");
    let user_id = user["id"].as_i64().unwrap();
    let session_id = login(&client, "alice", "password123");

    let response = client
        .patch(format!("/api/users/{}", user_id))
        .header(ContentType::JSON)
        .cookie(rocket::http::Cookie::new("session_id", &session_id))
        .body(
            json!({
                "current_password": "password123"
            })
            .to_string(),
        )
        .dispatch();

    assert_eq!(response.status(), Status::BadRequest);
}

// ============================================================================
// DELETE /api/users/:id (Delete User)
// ============================================================================

#[test]
fn test_delete_user_success() {
    let client = create_test_client();
    let user = create_user(&client, "alice", "alice@example.com", "password123");
    let user_id = user["id"].as_i64().unwrap();
    let session_id = login(&client, "alice", "password123");

    let status = delete_user(&client, &session_id, user_id, "password123");

    assert_eq!(status, Status::NoContent);

    // Verify user can't login after deletion
    let response = client
        .post("/api/sessions")
        .header(ContentType::JSON)
        .body(
            json!({
                "email_or_username": "alice",
                "password": "password123"
            })
            .to_string(),
        )
        .dispatch();

    assert_eq!(response.status(), Status::Unauthorized);
}

#[test]
fn test_delete_user_with_collections_and_decks() {
    let client = create_test_client();

    // Create game, set, cards (need admin auth for these)
    let game_response = client
        .post("/api/games")
        .header(ContentType::JSON)
        .header(rocket::http::Header::new(
            "Authorization",
            "Bearer test-admin-key-12345",
        ))
        .body(
            json!({
                "name": "Test Game",
                "image_url": "http://example.com/game.jpg"
            })
            .to_string(),
        )
        .dispatch();

    assert_eq!(game_response.status(), Status::Ok);
    let game: Value = serde_json::from_str(&game_response.into_string().unwrap()).unwrap();
    let game_id = game["id"].as_i64().unwrap();

    let set_response = client
        .post(format!("/api/games/{}/sets", game_id))
        .header(ContentType::JSON)
        .header(rocket::http::Header::new(
            "Authorization",
            "Bearer test-admin-key-12345",
        ))
        .body(
            json!({
                "name": "Test Set",
                "image_url": "http://example.com/set.jpg",
                "publish_date": "2024-01-01"
            })
            .to_string(),
        )
        .dispatch();

    assert_eq!(set_response.status(), Status::Ok);
    let set: Value = serde_json::from_str(&set_response.into_string().unwrap()).unwrap();
    let set_id = set["id"].as_i64().unwrap();

    let card_response = client
        .post(format!("/api/games/{}/sets/{}/cards", game_id, set_id))
        .header(ContentType::JSON)
        .header(rocket::http::Header::new(
            "Authorization",
            "Bearer test-admin-key-12345",
        ))
        .body(
            json!([{
                "name": "Test Card",
                "collector_number": "001",
                "image_url": "http://example.com/card.jpg",
                "attributes": {}
            }])
            .to_string(),
        )
        .dispatch();

    assert_eq!(card_response.status(), Status::Ok);
    let cards: Value = serde_json::from_str(&card_response.into_string().unwrap()).unwrap();
    let card_id = cards["ids"][0].as_i64().unwrap();

    // Create user and login
    let user = create_user(&client, "alice", "alice@example.com", "password123");
    let user_id = user["id"].as_i64().unwrap();
    let session_id = login(&client, "alice", "password123");

    // Create collection
    let collection_response = client
        .post("/api/collections")
        .header(ContentType::JSON)
        .cookie(rocket::http::Cookie::new("session_id", &session_id))
        .body(
            json!({
                "game_id": game_id,
                "name": "Test Collection"
            })
            .to_string(),
        )
        .dispatch();

    assert_eq!(collection_response.status(), Status::Ok);
    let collection: Value =
        serde_json::from_str(&collection_response.into_string().unwrap()).unwrap();
    let collection_id = collection["id"].as_i64().unwrap();

    // Add cards to collection
    client
        .patch(format!("/api/collections/{}/cards", collection_id))
        .header(ContentType::JSON)
        .cookie(rocket::http::Cookie::new("session_id", &session_id))
        .body(
            json!([{
                "card_id": card_id,
                "quantity": 5
            }])
            .to_string(),
        )
        .dispatch();

    // Create deck
    let deck_response = client
        .post("/api/decks")
        .header(ContentType::JSON)
        .cookie(rocket::http::Cookie::new("session_id", &session_id))
        .body(
            json!({
                "collection_id": collection_id,
                "name": "Test Deck"
            })
            .to_string(),
        )
        .dispatch();

    assert_eq!(deck_response.status(), Status::Ok);
    let deck: Value = serde_json::from_str(&deck_response.into_string().unwrap()).unwrap();
    let deck_id = deck["id"].as_i64().unwrap();

    // Add cards to deck
    client
        .patch(format!("/api/decks/{}/cards", deck_id))
        .header(ContentType::JSON)
        .cookie(rocket::http::Cookie::new("session_id", &session_id))
        .body(
            json!([{
                "card_id": card_id,
                "quantity": 3
            }])
            .to_string(),
        )
        .dispatch();

    // Delete user
    let status = delete_user(&client, &session_id, user_id, "password123");
    assert_eq!(status, Status::NoContent);

    // Verify user can't login
    let response = client
        .post("/api/sessions")
        .header(ContentType::JSON)
        .body(
            json!({
                "email_or_username": "alice",
                "password": "password123"
            })
            .to_string(),
        )
        .dispatch();

    assert_eq!(response.status(), Status::Unauthorized);
}

#[test]
fn test_delete_user_without_auth() {
    let client = create_test_client();
    let user = create_user(&client, "alice", "alice@example.com", "password123");
    let user_id = user["id"].as_i64().unwrap();

    let response = client
        .delete(format!("/api/users/{}", user_id))
        .header(ContentType::JSON)
        .body(
            json!({
                "password": "password123"
            })
            .to_string(),
        )
        .dispatch();

    assert_eq!(response.status(), Status::Unauthorized);
}

#[test]
fn test_delete_user_not_owner() {
    let client = create_test_client();
    let _user1 = create_user(&client, "alice", "alice@example.com", "password123");
    let user2 = create_user(&client, "bob", "bob@example.com", "password456");
    let user2_id = user2["id"].as_i64().unwrap();
    let session_id = login(&client, "alice", "password123");

    let status = delete_user(&client, &session_id, user2_id, "password456");

    assert_eq!(status, Status::Forbidden);
}

#[test]
fn test_delete_user_not_found() {
    let client = create_test_client();
    let _user = create_user(&client, "alice", "alice@example.com", "password123");
    let session_id = login(&client, "alice", "password123");

    let status = delete_user(&client, &session_id, 999, "password123");

    assert_eq!(status, Status::Forbidden);
}

#[test]
fn test_delete_user_wrong_password() {
    let client = create_test_client();
    let user = create_user(&client, "alice", "alice@example.com", "password123");
    let user_id = user["id"].as_i64().unwrap();
    let session_id = login(&client, "alice", "password123");

    let status = delete_user(&client, &session_id, user_id, "wrongpassword");

    assert_eq!(status, Status::Unauthorized);
}
