use rocket::State;
use rocket::http::Status;
use rocket::serde::json::Json;
use serde::{Deserialize, Serialize};

use crate::{AdminAuth, DbConn, ErrorResponse};

#[derive(Deserialize)]
pub struct CreateSetRequest {
    name: String,
    image_url: Option<String>,
    publish_date: String,
}

#[derive(Serialize)]
pub struct SetResponse {
    id: i64,
}

#[derive(Serialize)]
pub struct SetListItem {
    id: i64,
    name: String,
    image_url: Option<String>,
    publish_date: String,
}

#[post("/<game_id>/sets", format = "json", data = "<req>")]
pub fn create(
    _auth: AdminAuth,
    db: &State<DbConn>,
    game_id: i64,
    req: Json<CreateSetRequest>,
) -> Result<Json<SetResponse>, (Status, Json<ErrorResponse>)> {
    let conn = db.0.lock().unwrap();

    let id = super::create(&conn, game_id, &req.name, req.image_url.as_deref(), &req.publish_date).map_err(|e| {
        let (status, error) = match e {
            super::CreateSetError::GameNotFound => (Status::NotFound, "Game not found"),
            super::CreateSetError::NameExists => {
                (Status::Conflict, "Set name already exists for this game")
            }
            super::CreateSetError::DatabaseError => {
                (Status::InternalServerError, "Failed to create set")
            }
        };
        (status, Json(ErrorResponse::new(error)))
    })?;

    Ok(Json(SetResponse { id }))
}

#[get("/<game_id>/sets/<set_id>")]
pub fn get(
    db: &State<DbConn>,
    game_id: i64,
    set_id: i64,
) -> Result<Json<SetListItem>, (Status, Json<ErrorResponse>)> {
    let conn = db.0.lock().unwrap();

    let set = super::get(&conn, game_id, set_id)
        .map_err(|_| {
            (
                Status::InternalServerError,
                Json(ErrorResponse::new("Failed to get set")),
            )
        })?
        .ok_or_else(|| (Status::NotFound, Json(ErrorResponse::new("Set not found"))))?;

    Ok(Json(SetListItem {
        id: set.id,
        name: set.name,
        image_url: set.image_url,
        publish_date: set.publish_date,
    }))
}

#[get("/<game_id>/sets")]
pub fn list(
    db: &State<DbConn>,
    game_id: i64,
) -> Result<Json<Vec<SetListItem>>, (Status, Json<ErrorResponse>)> {
    let conn = db.0.lock().unwrap();

    let sets = super::list(&conn, game_id).map_err(|_| {
        (
            Status::InternalServerError,
            Json(ErrorResponse::new("Failed to list sets")),
        )
    })?;

    let items = sets
        .into_iter()
        .map(|s| SetListItem {
            id: s.id,
            name: s.name,
            image_url: s.image_url,
            publish_date: s.publish_date,
        })
        .collect();

    Ok(Json(items))
}

pub fn routes() -> Vec<rocket::Route> {
    routes![create, get, list]
}
