use std::collections::HashMap;

use rocket::State;
use rocket::http::Status;
use rocket::serde::json::Json;
use serde::{Deserialize, Serialize};

use crate::{DbConn, ErrorResponse, SessionAuth};

#[derive(Deserialize)]
pub struct CreateDeckRequest {
    collection_id: i64,
    name: String,
}

#[derive(Serialize)]
pub struct DeckResponse {
    id: i64,
}

#[derive(Serialize)]
pub struct DeckListItem {
    id: i64,
    collection_id: i64,
    name: String,
    created_at: String,
    collection_name: String,
    game_name: String,
    game_image_url: Option<String>,
    card_count: i64,
}

#[derive(Serialize)]
pub struct DeckCardItem {
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
    req: Json<CreateDeckRequest>,
) -> Result<Json<DeckResponse>, (Status, Json<ErrorResponse>)> {
    let conn = db.0.lock().unwrap();

    let id = super::create(&conn, auth.0.id, req.collection_id, &req.name).map_err(|e| {
        let (status, error) = match e {
            super::CreateDeckError::CollectionNotFound => {
                (Status::NotFound, "Collection not found")
            }
            super::CreateDeckError::NotOwner => (Status::Forbidden, "Access denied"),
            super::CreateDeckError::DatabaseError => {
                (Status::InternalServerError, "Failed to create deck")
            }
        };
        (status, Json(ErrorResponse::new(error)))
    })?;

    Ok(Json(DeckResponse { id }))
}

#[get("/")]
pub fn list(
    auth: SessionAuth,
    db: &State<DbConn>,
) -> Result<Json<Vec<DeckListItem>>, (Status, Json<ErrorResponse>)> {
    let conn = db.0.lock().unwrap();

    let decks = super::list_by_user(&conn, auth.0.id).map_err(|_| {
        (
            Status::InternalServerError,
            Json(ErrorResponse::new("Failed to list decks")),
        )
    })?;

    let items = decks
        .into_iter()
        .map(|d| DeckListItem {
            id: d.id,
            collection_id: d.collection_id,
            name: d.name,
            created_at: d.created_at,
            collection_name: d.collection_name,
            game_name: d.game_name,
            game_image_url: d.game_image_url,
            card_count: d.card_count,
        })
        .collect();

    Ok(Json(items))
}

#[get("/<deck_id>")]
pub fn get(
    auth: SessionAuth,
    db: &State<DbConn>,
    deck_id: i64,
) -> Result<Json<DeckListItem>, (Status, Json<ErrorResponse>)> {
    let conn = db.0.lock().unwrap();

    let deck = super::get(&conn, auth.0.id, deck_id).map_err(|e| {
        let (status, error) = match e {
            super::GetDeckError::NotFound => (Status::NotFound, "Deck not found"),
            super::GetDeckError::NotOwner => (Status::Forbidden, "Access denied"),
            super::GetDeckError::DatabaseError => {
                (Status::InternalServerError, "Failed to get deck")
            }
        };
        (status, Json(ErrorResponse::new(error)))
    })?;

    Ok(Json(DeckListItem {
        id: deck.id,
        collection_id: deck.collection_id,
        name: deck.name,
        created_at: deck.created_at,
        collection_name: deck.collection_name,
        game_name: deck.game_name,
        game_image_url: deck.game_image_url,
        card_count: deck.card_count,
    }))
}

#[get("/<deck_id>/cards")]
pub fn list_cards(
    auth: SessionAuth,
    db: &State<DbConn>,
    deck_id: i64,
) -> Result<Json<Vec<DeckCardItem>>, (Status, Json<ErrorResponse>)> {
    let conn = db.0.lock().unwrap();

    let cards = super::list_deck_cards(&conn, deck_id, auth.0.id).map_err(|e| {
        let (status, error) = match e {
            super::GetDeckError::NotFound => (Status::NotFound, "Deck not found"),
            super::GetDeckError::NotOwner => (Status::Forbidden, "Access denied"),
            super::GetDeckError::DatabaseError => {
                (Status::InternalServerError, "Failed to list deck cards")
            }
        };
        (status, Json(ErrorResponse::new(error)))
    })?;

    let items = cards
        .into_iter()
        .map(|c| DeckCardItem {
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

#[patch("/<deck_id>/cards", format = "json", data = "<updates>")]
pub fn update_cards(
    auth: SessionAuth,
    db: &State<DbConn>,
    deck_id: i64,
    updates: Json<Vec<CardQuantityUpdateRequest>>,
) -> Result<Status, (Status, Json<ErrorResponse>)> {
    let conn = db.0.lock().unwrap();

    let card_updates: Vec<super::CardQuantityUpdate> = updates
        .into_inner()
        .into_iter()
        .map(|u| super::CardQuantityUpdate {
            card_id: u.card_id,
            quantity: u.quantity,
        })
        .collect();

    super::update_deck_cards(&conn, deck_id, auth.0.id, &card_updates).map_err(|e| {
        let (status, error) = match e {
            super::UpdateDeckCardsError::DeckNotFound => (Status::NotFound, "Deck not found"),
            super::UpdateDeckCardsError::NotOwner => (Status::Forbidden, "Access denied"),
            super::UpdateDeckCardsError::CardNotInCollection => {
                (Status::BadRequest, "Card not in collection")
            }
            super::UpdateDeckCardsError::InsufficientQuantity => {
                (Status::BadRequest, "Quantity exceeds collection")
            }
            super::UpdateDeckCardsError::DatabaseError => {
                (Status::InternalServerError, "Failed to update deck cards")
            }
        };
        (status, Json(ErrorResponse::new(error)))
    })?;

    Ok(Status::NoContent)
}

pub fn routes() -> Vec<rocket::Route> {
    routes![create, list, get, list_cards, update_cards]
}
