use rocket::State;
use rocket::http::Status;
use rocket::serde::json::Json;
use serde::{Deserialize, Serialize};

use crate::{DbConn, ErrorResponse};

#[derive(Deserialize)]
pub struct CreateUserRequest {
    username: String,
    email: String,
    password: String,
}

#[derive(Serialize)]
pub struct UserResponse {
    id: i64,
}

#[post("/", format = "json", data = "<req>")]
pub fn create(
    db: &State<DbConn>,
    req: Json<CreateUserRequest>,
) -> Result<Json<UserResponse>, (Status, Json<ErrorResponse>)> {
    let conn = db.0.lock().unwrap();

    let id = super::create(&conn, &req.username, &req.email, &req.password).map_err(|e| {
        let (status, error) = match e {
            super::CreateUserError::UsernameExists => (Status::Conflict, "Username already exists"),
            super::CreateUserError::EmailExists => (Status::Conflict, "Email already exists"),
            super::CreateUserError::HashError => {
                (Status::InternalServerError, "Failed to hash password")
            }
            super::CreateUserError::DatabaseError => {
                (Status::InternalServerError, "Failed to create user")
            }
        };
        (status, Json(ErrorResponse::new(error)))
    })?;

    Ok(Json(UserResponse { id }))
}

pub fn routes() -> Vec<rocket::Route> {
    routes![create]
}
