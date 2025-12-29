use rocket::http::{ContentType, Status};
use rocket::local::blocking::Client;
use serde_json::{json, Value};

use crate::test_helpers::create_test_client;

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

fn login<'a>(client: &'a Client, email_or_username: &str, password: &str) -> rocket::local::blocking::LocalResponse<'a> {
    client
        .post("/api/sessions")
        .header(ContentType::JSON)
        .body(
            json!({
                "email_or_username": email_or_username,
                "password": password
            })
            .to_string(),
        )
        .dispatch()
}

// ============================================================================
// POST /api/sessions (Login)
// ============================================================================

#[test]
fn test_login_success_with_username() {
    let client = create_test_client();
    create_user(&client, "testuser", "test@example.com", "password123");

    let response = login(&client, "testuser", "password123");

    assert_eq!(response.status(), Status::Ok);

    // Should set a session cookie
    let cookies = response.cookies();
    let session_cookie = cookies.get("session_id");
    assert!(session_cookie.is_some());
    assert!(!session_cookie.unwrap().value().is_empty());
}

#[test]
fn test_login_success_with_email() {
    let client = create_test_client();
    create_user(&client, "testuser", "test@example.com", "password123");

    let response = login(&client, "test@example.com", "password123");

    assert_eq!(response.status(), Status::Ok);

    let cookies = response.cookies();
    assert!(cookies.get("session_id").is_some());
}

#[test]
fn test_login_invalid_username() {
    let client = create_test_client();
    create_user(&client, "testuser", "test@example.com", "password123");

    let response = login(&client, "wronguser", "password123");

    assert_eq!(response.status(), Status::Unauthorized);

    let body: Value = serde_json::from_str(&response.into_string().unwrap()).unwrap();
    assert_eq!(body["error"], "Invalid credentials");
}

#[test]
fn test_login_invalid_password() {
    let client = create_test_client();
    create_user(&client, "testuser", "test@example.com", "password123");

    let response = login(&client, "testuser", "wrongpassword");

    assert_eq!(response.status(), Status::Unauthorized);

    let body: Value = serde_json::from_str(&response.into_string().unwrap()).unwrap();
    assert_eq!(body["error"], "Invalid credentials");
}

#[test]
fn test_login_no_user_exists() {
    let client = create_test_client();

    let response = login(&client, "nonexistent", "password123");

    assert_eq!(response.status(), Status::Unauthorized);

    let body: Value = serde_json::from_str(&response.into_string().unwrap()).unwrap();
    assert_eq!(body["error"], "Invalid credentials");
}

#[test]
fn test_login_multiple_sessions() {
    let client = create_test_client();
    create_user(&client, "testuser", "test@example.com", "password123");

    // Create first session
    let response1 = login(&client, "testuser", "password123");
    let session1 = response1
        .cookies()
        .get("session_id")
        .unwrap()
        .value()
        .to_string();

    // Create second session (simulating login from another device)
    let response2 = login(&client, "testuser", "password123");
    let session2 = response2
        .cookies()
        .get("session_id")
        .unwrap()
        .value()
        .to_string();

    // Should be different session IDs
    assert_ne!(session1, session2);
}

// ============================================================================
// DELETE /api/sessions/<session_id> (Logout)
// ============================================================================

#[test]
fn test_logout_success() {
    let client = create_test_client();
    create_user(&client, "testuser", "test@example.com", "password123");

    // Login first
    let login_response = login(&client, "testuser", "password123");
    let session_id = login_response
        .cookies()
        .get("session_id")
        .unwrap()
        .value()
        .to_string();

    // Logout
    let response = client.delete(format!("/api/sessions/{}", session_id)).dispatch();

    assert_eq!(response.status(), Status::Ok);
}

#[test]
fn test_logout_invalid_session() {
    let client = create_test_client();

    let response = client
        .delete("/api/sessions/nonexistent-session-id")
        .dispatch();

    assert_eq!(response.status(), Status::NotFound);
}

#[test]
fn test_logout_twice() {
    let client = create_test_client();
    create_user(&client, "testuser", "test@example.com", "password123");

    // Login
    let login_response = login(&client, "testuser", "password123");
    let session_id = login_response
        .cookies()
        .get("session_id")
        .unwrap()
        .value()
        .to_string();

    // First logout - success
    let response1 = client.delete(format!("/api/sessions/{}", session_id)).dispatch();
    assert_eq!(response1.status(), Status::Ok);

    // Second logout - session already deleted
    let response2 = client.delete(format!("/api/sessions/{}", session_id)).dispatch();
    assert_eq!(response2.status(), Status::NotFound);
}

#[test]
fn test_logout_one_session_keeps_others() {
    let client = create_test_client();
    create_user(&client, "testuser", "test@example.com", "password123");

    // Create two sessions
    let response1 = login(&client, "testuser", "password123");
    let session1 = response1
        .cookies()
        .get("session_id")
        .unwrap()
        .value()
        .to_string();

    let response2 = login(&client, "testuser", "password123");
    let session2 = response2
        .cookies()
        .get("session_id")
        .unwrap()
        .value()
        .to_string();

    // Logout session 1
    let logout1 = client.delete(format!("/api/sessions/{}", session1)).dispatch();
    assert_eq!(logout1.status(), Status::Ok);

    // Session 2 should still be valid (can be logged out)
    let logout2 = client.delete(format!("/api/sessions/{}", session2)).dispatch();
    assert_eq!(logout2.status(), Status::Ok);
}
