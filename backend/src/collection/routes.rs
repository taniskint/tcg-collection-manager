use std::collections::HashMap;

use rocket::State;
use rocket::http::Status;
use rocket::serde::json::Json;
use serde::{Deserialize, Serialize};

use crate::{DbConn, ErrorResponse, SessionAuth};

#[derive(Deserialize)]
pub struct CreateCollectionRequest {
    game_id: i64,
    name: String,
}

#[derive(Serialize)]
pub struct CollectionResponse {
    id: i64,
}

#[derive(Serialize)]
pub struct CollectionListItem {
    id: i64,
    game_id: i64,
    name: String,
    created_at: String,
    game_name: String,
    game_image_url: Option<String>,
}

#[derive(Serialize)]
pub struct CollectionDetail {
    id: i64,
    game_id: i64,
    name: String,
    created_at: String,
    game_name: String,
    game_image_url: Option<String>,
    card_count: i64,
}

#[derive(Serialize)]
pub struct CollectionCardItem {
    id: i64,
    name: String,
    collector_number: String,
    image_url: Option<String>,
    attributes: HashMap<String, String>,
    set_id: i64,
    set_name: String,
    quantity: i64,
}

#[derive(Deserialize)]
pub struct CardQuantityUpdateRequest {
    card_id: i64,
    quantity: i64,
}

#[post("/", format = "json", data = "<req>")]
pub fn create(
    auth: SessionAuth,
    db: &State<DbConn>,
    req: Json<CreateCollectionRequest>,
) -> Result<Json<CollectionResponse>, (Status, Json<ErrorResponse>)> {
    let conn = db.0.lock().unwrap();

    let id = super::create(&conn, auth.0.id, req.game_id, &req.name).map_err(|e| {
        let (status, error) = match e {
            super::CreateCollectionError::GameNotFound => (Status::NotFound, "Game not found"),
            super::CreateCollectionError::DatabaseError => {
                (Status::InternalServerError, "Failed to create collection")
            }
        };
        (status, Json(ErrorResponse::new(error)))
    })?;

    Ok(Json(CollectionResponse { id }))
}

#[get("/")]
pub fn list(
    auth: SessionAuth,
    db: &State<DbConn>,
) -> Result<Json<Vec<CollectionListItem>>, (Status, Json<ErrorResponse>)> {
    let conn = db.0.lock().unwrap();

    let collections = super::list_by_user(&conn, auth.0.id).map_err(|_| {
        (
            Status::InternalServerError,
            Json(ErrorResponse::new("Failed to list collections")),
        )
    })?;

    let items = collections
        .into_iter()
        .map(|c| CollectionListItem {
            id: c.id,
            game_id: c.game_id,
            name: c.name,
            created_at: c.created_at,
            game_name: c.game_name,
            game_image_url: c.game_image_url,
        })
        .collect();

    Ok(Json(items))
}

#[get("/<collection_id>")]
pub fn get(
    auth: SessionAuth,
    db: &State<DbConn>,
    collection_id: i64,
) -> Result<Json<CollectionDetail>, (Status, Json<ErrorResponse>)> {
    let conn = db.0.lock().unwrap();

    let collection = super::get(&conn, collection_id, auth.0.id).map_err(|e| {
        let (status, error) = match e {
            super::GetCollectionError::NotFound => (Status::NotFound, "Collection not found"),
            super::GetCollectionError::NotOwner => (Status::Forbidden, "Access denied"),
            super::GetCollectionError::DatabaseError => {
                (Status::InternalServerError, "Failed to get collection")
            }
        };
        (status, Json(ErrorResponse::new(error)))
    })?;

    Ok(Json(CollectionDetail {
        id: collection.id,
        game_id: collection.game_id,
        name: collection.name,
        created_at: collection.created_at,
        game_name: collection.game_name,
        game_image_url: collection.game_image_url,
        card_count: collection.card_count,
    }))
}

#[get("/<collection_id>/cards")]
pub fn list_cards(
    auth: SessionAuth,
    db: &State<DbConn>,
    collection_id: i64,
) -> Result<Json<Vec<CollectionCardItem>>, (Status, Json<ErrorResponse>)> {
    let conn = db.0.lock().unwrap();

    let cards = super::list_collection_cards(&conn, collection_id, auth.0.id).map_err(|e| {
        let (status, error) = match e {
            super::GetCollectionError::NotFound => (Status::NotFound, "Collection not found"),
            super::GetCollectionError::NotOwner => (Status::Forbidden, "Access denied"),
            super::GetCollectionError::DatabaseError => {
                (Status::InternalServerError, "Failed to list cards")
            }
        };
        (status, Json(ErrorResponse::new(error)))
    })?;

    let items = cards
        .into_iter()
        .map(|c| CollectionCardItem {
            id: c.id,
            name: c.name,
            collector_number: c.collector_number,
            image_url: c.image_url,
            attributes: c.attributes,
            set_id: c.set_id,
            set_name: c.set_name,
            quantity: c.quantity,
        })
        .collect();

    Ok(Json(items))
}

#[patch("/<collection_id>/cards", format = "json", data = "<req>")]
pub fn update_cards(
    auth: SessionAuth,
    db: &State<DbConn>,
    collection_id: i64,
    req: Json<Vec<CardQuantityUpdateRequest>>,
) -> Result<Status, (Status, Json<ErrorResponse>)> {
    let conn = db.0.lock().unwrap();

    let updates: Vec<super::CardQuantityUpdate> = req
        .into_inner()
        .into_iter()
        .map(|r| super::CardQuantityUpdate {
            card_id: r.card_id,
            quantity: r.quantity,
        })
        .collect();

    super::update_collection_cards(&conn, collection_id, auth.0.id, &updates).map_err(|e| {
        let (status, error) = match e {
            super::UpdateCollectionCardsError::CollectionNotFound => {
                (Status::NotFound, "Collection not found")
            }
            super::UpdateCollectionCardsError::NotOwner => (Status::Forbidden, "Access denied"),
            super::UpdateCollectionCardsError::CardNotFound => (Status::NotFound, "Card not found"),
            super::UpdateCollectionCardsError::GameMismatch => (
                Status::BadRequest,
                "Card does not belong to collection's game",
            ),
            super::UpdateCollectionCardsError::DatabaseError => {
                (Status::InternalServerError, "Failed to update cards")
            }
        };
        (status, Json(ErrorResponse::new(error)))
    })?;

    Ok(Status::NoContent)
}

pub fn routes() -> Vec<rocket::Route> {
    routes![create, list, get, list_cards, update_cards]
}
