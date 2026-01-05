use std::collections::HashMap;

use rocket::State;
use rocket::http::Status;
use rocket::serde::json::Json;
use serde::{Deserialize, Serialize};

use crate::{AdminAuth, DbConn, ErrorResponse};

#[derive(Deserialize)]
pub struct CreateCardRequest {
    name: String,
    collector_number: String,
    image_url: Option<String>,
    #[serde(default)]
    attributes: HashMap<String, String>,
}

#[derive(Serialize)]
pub struct CardsResponse {
    ids: Vec<i64>,
}

#[derive(Serialize)]
pub struct CardItem {
    id: i64,
    name: String,
    collector_number: String,
    image_url: Option<String>,
    attributes: HashMap<String, String>,
}

#[post("/<game_id>/sets/<set_id>/cards", format = "json", data = "<req>")]
pub fn create(
    _auth: AdminAuth,
    db: &State<DbConn>,
    game_id: i64,
    set_id: i64,
    req: Json<Vec<CreateCardRequest>>,
) -> Result<Json<CardsResponse>, (Status, Json<ErrorResponse>)> {
    let _ = game_id; // Included in path for consistency, but not used in query
    let conn = db.0.lock().unwrap();

    let card_inputs: Vec<_> = req
        .iter()
        .map(|c| super::CreateCardInput {
            name: &c.name,
            collector_number: &c.collector_number,
            image_url: c.image_url.as_deref(),
            attributes: &c.attributes,
        })
        .collect();

    let ids = super::create_many(&conn, set_id, &card_inputs).map_err(|e| {
        let (status, error) = match e {
            super::CreateCardError::SetNotFound => (Status::NotFound, "Set not found"),
            super::CreateCardError::DatabaseError => {
                (Status::InternalServerError, "Failed to create cards")
            }
        };
        (status, Json(ErrorResponse::new(error)))
    })?;

    Ok(Json(CardsResponse { ids }))
}

#[get("/<game_id>/sets/<set_id>/cards/<card_id>")]
pub fn get(
    db: &State<DbConn>,
    game_id: i64,
    set_id: i64,
    card_id: i64,
) -> Result<Json<CardItem>, (Status, Json<ErrorResponse>)> {
    let _ = game_id;
    let conn = db.0.lock().unwrap();

    let card = super::get(&conn, set_id, card_id)
        .map_err(|_| {
            (
                Status::InternalServerError,
                Json(ErrorResponse::new("Failed to get card")),
            )
        })?
        .ok_or_else(|| (Status::NotFound, Json(ErrorResponse::new("Card not found"))))?;

    Ok(Json(CardItem {
        id: card.id,
        name: card.name,
        collector_number: card.collector_number,
        image_url: card.image_url,
        attributes: card.attributes,
    }))
}

#[get("/<game_id>/sets/<set_id>/cards")]
pub fn list(
    db: &State<DbConn>,
    game_id: i64,
    set_id: i64,
) -> Result<Json<Vec<CardItem>>, (Status, Json<ErrorResponse>)> {
    let _ = game_id;
    let conn = db.0.lock().unwrap();

    let cards = super::list(&conn, set_id).map_err(|_| {
        (
            Status::InternalServerError,
            Json(ErrorResponse::new("Failed to list cards")),
        )
    })?;

    let items = cards
        .into_iter()
        .map(|c| CardItem {
            id: c.id,
            name: c.name,
            collector_number: c.collector_number,
            image_url: c.image_url,
            attributes: c.attributes,
        })
        .collect();

    Ok(Json(items))
}

pub fn routes() -> Vec<rocket::Route> {
    routes![create, get, list]
}
