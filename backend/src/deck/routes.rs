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

pub fn routes() -> Vec<rocket::Route> {
    routes![create, list, get]
}
