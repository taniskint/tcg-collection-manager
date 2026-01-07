use std::collections::HashMap;

use rocket::State;
use rocket::http::Status;
use rocket::serde::json::Json;
use serde::{Deserialize, Serialize};

use crate::{AdminAuth, DbConn, ErrorResponse, SessionAuth};

#[derive(Deserialize)]
pub struct CreateBoosterRequest {
    name: String,
    spec: serde_json::Value,
}

#[derive(Serialize)]
pub struct BoosterResponse {
    id: i64,
}

#[derive(Serialize)]
pub struct BoosterListItem {
    id: i64,
    name: String,
}

#[derive(Deserialize)]
pub struct OpenPacksRequest {
    collection_id: i64,
    count: i64,
}

#[derive(Serialize)]
pub struct OpenedCardItem {
    id: i64,
    name: String,
    collector_number: String,
    image_url: Option<String>,
    attributes: HashMap<String, String>,
    quantity: i64,
}

#[derive(Serialize)]
pub struct OpenPacksResponse {
    cards: Vec<OpenedCardItem>,
}

#[post("/<_game_id>/sets/<set_id>/boosters", format = "json", data = "<req>")]
pub fn create(
    _auth: AdminAuth,
    db: &State<DbConn>,
    _game_id: i64,
    set_id: i64,
    req: Json<CreateBoosterRequest>,
) -> Result<Json<BoosterResponse>, (Status, Json<ErrorResponse>)> {
    let conn = db.0.lock().unwrap();

    let spec_json = serde_json::to_string(&req.spec).map_err(|_| {
        (
            Status::BadRequest,
            Json(ErrorResponse::new("Invalid spec format")),
        )
    })?;

    let id = super::create(&conn, set_id, &req.name, &spec_json).map_err(|e| {
        let (status, error) = match e {
            super::CreateBoosterError::SetNotFound => (Status::NotFound, "Set not found"),
            super::CreateBoosterError::NameExists => {
                (Status::Conflict, "Booster name already exists for this set")
            }
            super::CreateBoosterError::DatabaseError => {
                (Status::InternalServerError, "Failed to create booster")
            }
        };
        (status, Json(ErrorResponse::new(error)))
    })?;

    Ok(Json(BoosterResponse { id }))
}

#[get("/<_game_id>/sets/<set_id>/boosters")]
pub fn list(
    db: &State<DbConn>,
    _game_id: i64,
    set_id: i64,
) -> Result<Json<Vec<BoosterListItem>>, (Status, Json<ErrorResponse>)> {
    let conn = db.0.lock().unwrap();

    let boosters = super::list(&conn, set_id).map_err(|_| {
        (
            Status::InternalServerError,
            Json(ErrorResponse::new("Failed to list boosters")),
        )
    })?;

    let items = boosters
        .into_iter()
        .map(|b| BoosterListItem {
            id: b.id,
            name: b.name,
        })
        .collect();

    Ok(Json(items))
}

#[post("/<booster_id>/open", format = "json", data = "<req>")]
pub fn open(
    auth: SessionAuth,
    db: &State<DbConn>,
    booster_id: i64,
    req: Json<OpenPacksRequest>,
) -> Result<Json<OpenPacksResponse>, (Status, Json<ErrorResponse>)> {
    let conn = db.0.lock().unwrap();

    if req.count <= 0 {
        return Err((
            Status::BadRequest,
            Json(ErrorResponse::new("Count must be positive")),
        ));
    }

    let cards =
        super::open_packs(&conn, booster_id, req.collection_id, auth.0.id, req.count).map_err(
            |e| {
                let (status, error) = match e {
                    super::OpenPacksError::BoosterNotFound => {
                        (Status::NotFound, "Booster not found")
                    }
                    super::OpenPacksError::CollectionNotFound => {
                        (Status::NotFound, "Collection not found")
                    }
                    super::OpenPacksError::NotOwner => (Status::Forbidden, "Access denied"),
                    super::OpenPacksError::GameMismatch => (
                        Status::BadRequest,
                        "Collection does not match booster's game",
                    ),
                    super::OpenPacksError::InvalidSpec => {
                        (Status::InternalServerError, "Invalid booster spec")
                    }
                    super::OpenPacksError::NoMatchingCards => {
                        (Status::InternalServerError, "No matching cards for booster slot")
                    }
                    super::OpenPacksError::DatabaseError => {
                        (Status::InternalServerError, "Failed to open packs")
                    }
                };
                (status, Json(ErrorResponse::new(error)))
            },
        )?;

    let items = cards
        .into_iter()
        .map(|c| OpenedCardItem {
            id: c.id,
            name: c.name,
            collector_number: c.collector_number,
            image_url: c.image_url,
            attributes: c.attributes,
            quantity: c.quantity,
        })
        .collect();

    Ok(Json(OpenPacksResponse { cards: items }))
}

pub fn routes() -> Vec<rocket::Route> {
    routes![create, list]
}

pub fn open_routes() -> Vec<rocket::Route> {
    routes![open]
}
