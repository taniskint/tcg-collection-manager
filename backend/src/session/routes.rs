use rocket::http::{Cookie, CookieJar, Status};
use rocket::serde::json::Json;
use rocket::State;
use serde::Deserialize;

use crate::{DbConn, ErrorResponse};

#[derive(Deserialize)]
pub struct LoginRequest {
    email_or_username: String,
    password: String,
}

#[post("/", format = "json", data = "<req>")]
pub fn create(
    db: &State<DbConn>,
    cookies: &CookieJar<'_>,
    req: Json<LoginRequest>,
) -> Result<Status, (Status, Json<ErrorResponse>)> {
    let conn = db.0.lock().unwrap();

    let session_id =
        super::create(&conn, &req.email_or_username, &req.password).map_err(|e| {
            let (status, error) = match e {
                super::CreateSessionError::InvalidCredentials => {
                    (Status::Unauthorized, "Invalid credentials")
                }
                super::CreateSessionError::VerifyError => {
                    (Status::InternalServerError, "Failed to verify password")
                }
                super::CreateSessionError::DatabaseError => {
                    (Status::InternalServerError, "Failed to create session")
                }
            };
            (status, Json(ErrorResponse::new(error)))
        })?;

    cookies.add(Cookie::new("session_id", session_id));

    Ok(Status::Ok)
}

#[delete("/<session_id>")]
pub fn delete(db: &State<DbConn>, cookies: &CookieJar<'_>, session_id: &str) -> Status {
    let conn = db.0.lock().unwrap();

    cookies.remove(Cookie::from("session_id"));

    if super::delete(&conn, session_id) {
        Status::Ok
    } else {
        Status::NotFound
    }
}

pub fn routes() -> Vec<rocket::Route> {
    routes![create, delete]
}
