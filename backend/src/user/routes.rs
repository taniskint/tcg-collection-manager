use rocket::State;
use rocket::http::Status;
use rocket::serde::json::Json;
use serde::{Deserialize, Serialize};

use crate::{DbConn, ErrorResponse, SessionAuth};

#[derive(Deserialize)]
pub struct CreateUserRequest {
    username: String,
    email: String,
    password: String,
}

#[derive(Deserialize)]
pub struct UpdateUserRequest {
    username: Option<String>,
    email: Option<String>,
    password: Option<String>,
    current_password: String,
}

#[derive(Deserialize)]
pub struct DeleteUserRequest {
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
    // Validate username length
    if req.username.len() > 50 {
        return Err((
            Status::BadRequest,
            Json(ErrorResponse::new("Username must be 50 characters or less")),
        ));
    }

    // Validate password length
    if req.password.len() > 200 {
        return Err((
            Status::BadRequest,
            Json(ErrorResponse::new("Password must be 200 characters or less")),
        ));
    }

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

#[patch("/<user_id>", format = "json", data = "<req>")]
pub fn update(
    auth: SessionAuth,
    db: &State<DbConn>,
    user_id: i64,
    req: Json<UpdateUserRequest>,
) -> Result<Status, (Status, Json<ErrorResponse>)> {
    // Verify ownership
    if auth.0.id != user_id {
        return Err((Status::Forbidden, Json(ErrorResponse::new("Access denied"))));
    }

    // Validate username length if provided
    if let Some(username) = &req.username {
        if username.len() > 50 {
            return Err((
                Status::BadRequest,
                Json(ErrorResponse::new("Username must be 50 characters or less")),
            ));
        }
    }

    // Validate password length if provided
    if let Some(password) = &req.password {
        if password.len() > 200 {
            return Err((
                Status::BadRequest,
                Json(ErrorResponse::new("Password must be 200 characters or less")),
            ));
        }
    }

    let conn = db.0.lock().unwrap();

    super::update(
        &conn,
        user_id,
        &req.current_password,
        req.username.as_deref(),
        req.email.as_deref(),
        req.password.as_deref(),
    )
    .map_err(|e| {
        let (status, error) = match e {
            super::UpdateUserError::NotFound => (Status::NotFound, "User not found"),
            super::UpdateUserError::InvalidPassword => (Status::Unauthorized, "Invalid password"),
            super::UpdateUserError::UsernameExists => (Status::Conflict, "Username already exists"),
            super::UpdateUserError::EmailExists => (Status::Conflict, "Email already exists"),
            super::UpdateUserError::NoFieldsProvided => (Status::BadRequest, "No fields to update"),
            super::UpdateUserError::HashError => {
                (Status::InternalServerError, "Failed to hash password")
            }
            super::UpdateUserError::VerifyError => {
                (Status::InternalServerError, "Failed to verify password")
            }
            super::UpdateUserError::DatabaseError => {
                (Status::InternalServerError, "Failed to update user")
            }
        };
        (status, Json(ErrorResponse::new(error)))
    })?;

    Ok(Status::NoContent)
}

#[delete("/<user_id>", format = "json", data = "<req>")]
pub fn delete(
    auth: SessionAuth,
    db: &State<DbConn>,
    user_id: i64,
    req: Json<DeleteUserRequest>,
) -> Result<Status, (Status, Json<ErrorResponse>)> {
    // Verify ownership
    if auth.0.id != user_id {
        return Err((Status::Forbidden, Json(ErrorResponse::new("Access denied"))));
    }

    let conn = db.0.lock().unwrap();

    super::delete(&conn, user_id, &req.password).map_err(|e| {
        let (status, error) = match e {
            super::DeleteUserError::NotFound => (Status::NotFound, "User not found"),
            super::DeleteUserError::InvalidPassword => (Status::Unauthorized, "Invalid password"),
            super::DeleteUserError::VerifyError => {
                (Status::InternalServerError, "Failed to verify password")
            }
            super::DeleteUserError::DatabaseError => {
                (Status::InternalServerError, "Failed to delete user")
            }
        };
        (status, Json(ErrorResponse::new(error)))
    })?;

    Ok(Status::NoContent)
}

pub fn routes() -> Vec<rocket::Route> {
    routes![create, update, delete]
}

