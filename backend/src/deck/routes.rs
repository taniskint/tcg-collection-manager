use std::collections::HashMap;

use rocket::State;
use rocket::http::Status;
use rocket::serde::json::Json;
use rusqlite::params;
use serde::{Deserialize, Serialize};

use crate::{Config, DbConn, ErrorResponse, SessionAuth};

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

#[derive(Deserialize)]
pub struct UpdateDeckRequest {
    name: String,
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

#[patch("/<deck_id>", format = "json", data = "<req>")]
pub fn update(
    auth: SessionAuth,
    db: &State<DbConn>,
    deck_id: i64,
    req: Json<UpdateDeckRequest>,
) -> Result<Status, (Status, Json<ErrorResponse>)> {
    let conn = db.0.lock().unwrap();

    super::update(&conn, deck_id, auth.0.id, &req.name).map_err(|e| {
        let (status, error) = match e {
            super::UpdateDeckError::NotFound => (Status::NotFound, "Deck not found"),
            super::UpdateDeckError::NotOwner => (Status::Forbidden, "Access denied"),
            super::UpdateDeckError::DatabaseError => {
                (Status::InternalServerError, "Failed to update deck")
            }
        };
        (status, Json(ErrorResponse::new(error)))
    })?;

    Ok(Status::NoContent)
}

#[delete("/<deck_id>")]
pub fn delete(
    auth: SessionAuth,
    db: &State<DbConn>,
    deck_id: i64,
) -> Result<Status, (Status, Json<ErrorResponse>)> {
    let conn = db.0.lock().unwrap();

    super::delete(&conn, deck_id, auth.0.id).map_err(|e| {
        let (status, error) = match e {
            super::DeleteDeckError::NotFound => (Status::NotFound, "Deck not found"),
            super::DeleteDeckError::NotOwner => (Status::Forbidden, "Access denied"),
            super::DeleteDeckError::DatabaseError => {
                (Status::InternalServerError, "Failed to delete deck")
            }
        };
        (status, Json(ErrorResponse::new(error)))
    })?;

    Ok(Status::NoContent)
}

#[derive(Serialize)]
pub struct AtlasSheet {
    url: String,
    card_count: i64,
}

#[derive(Serialize)]
pub struct TabletopSimulatorResponse {
    #[serde(skip_serializing_if = "Option::is_none")]
    card_back: Option<String>,
    sheets: Vec<AtlasSheet>,
}

#[get("/<deck_id>/tabletop-simulator")]
pub async fn generate_tabletop_simulator(
    db: &State<DbConn>,
    config: &State<Config>,
    deck_id: i64,
) -> Result<Json<TabletopSimulatorResponse>, (Status, Json<ErrorResponse>)> {
    // 1. Fetch deck cards AND game card_back_image_url
    let (deck_cards, card_back_image_url) = {
        let conn = db.0.lock().unwrap();

        // Fetch deck cards
        let deck_cards = super::list_deck_cards_public(&conn, deck_id).map_err(|e| {
            let (status, error) = match e {
                super::GetDeckError::NotFound => (Status::NotFound, "Deck not found"),
                super::GetDeckError::NotOwner => (Status::Forbidden, "Access denied"),
                super::GetDeckError::DatabaseError => {
                    (Status::InternalServerError, "Failed to get deck")
                }
            };
            (status, Json(ErrorResponse::new(error)))
        })?;

        // Fetch game card_back_image_url
        // Query: deck -> collection -> game
        let card_back: Option<String> = conn
            .query_row(
                "SELECT g.card_back_image_url
                 FROM decks d
                 JOIN collections c ON d.collection_id = c.id
                 JOIN games g ON c.game_id = g.id
                 WHERE d.id = ?1",
                params![deck_id],
                |row| row.get(0),
            )
            .ok();

        (deck_cards, card_back)
    };

    // Validate deck is not empty
    if deck_cards.is_empty() {
        return Err((
            Status::BadRequest,
            Json(ErrorResponse::new("Deck is empty")),
        ));
    }

    // Check S3 config
    let s3_config = config.s3.as_ref().ok_or((
        Status::ServiceUnavailable,
        Json(ErrorResponse::new("S3 not configured")),
    ))?;

    // 2. Calculate deck hash
    let deck_hash = crate::atlas::calculate_deck_hash(&deck_cards);

    // 3. Check cache
    let s3_client = crate::atlas::create_s3_client().await;
    let total_cards: i64 = deck_cards.iter().map(|c| c.quantity).sum();
    let atlas_count = ((total_cards as usize + 69) / 70).max(1);

    let mut sheets = Vec::new();
    let mut cached = true;

    for i in 0..atlas_count {
        let key = crate::atlas::get_s3_key(deck_id, &deck_hash, i);
        match crate::atlas::check_cache(&s3_client, &s3_config.bucket, &key).await {
            Ok(Some(url)) => {
                let cards_in_atlas = if i == atlas_count - 1 {
                    ((total_cards - 1) % 70) + 1
                } else {
                    70
                };
                sheets.push(AtlasSheet {
                    url,
                    card_count: cards_in_atlas,
                });
            }
            _ => {
                cached = false;
                break;
            }
        }
    }

    if cached {
        return Ok(Json(TabletopSimulatorResponse {
            card_back: card_back_image_url,
            sheets,
        }));
    }

    // 4. Generate atlases
    let atlases = crate::atlas::generate_atlases(&deck_cards)
        .await
        .map_err(|e| {
            eprintln!("Atlas generation error: {:?}", e);
            (
                Status::InternalServerError,
                Json(ErrorResponse::new("Failed to generate atlases")),
            )
        })?;

    // 5. Upload to S3
    sheets.clear();
    for (i, atlas) in atlases.iter().enumerate() {
        let key = crate::atlas::get_s3_key(deck_id, &deck_hash, i);

        // Encode to PNG
        let mut png_data = Vec::new();
        let mut cursor = std::io::Cursor::new(&mut png_data);
        atlas
            .image
            .write_to(&mut cursor, image::ImageFormat::Png)
            .map_err(|e| {
                eprintln!("PNG encoding error: {:?}", e);
                (
                    Status::InternalServerError,
                    Json(ErrorResponse::new("Failed to encode atlas")),
                )
            })?;

        let url = crate::atlas::upload_atlas(&s3_client, &s3_config.bucket, &key, png_data)
            .await
            .map_err(|e| {
                eprintln!("S3 upload error: {:?}", e);
                (
                    Status::InternalServerError,
                    Json(ErrorResponse::new("Failed to upload atlas")),
                )
            })?;

        sheets.push(AtlasSheet {
            url,
            card_count: atlas.card_count as i64,
        });
    }

    Ok(Json(TabletopSimulatorResponse {
        card_back: card_back_image_url,
        sheets,
    }))
}

pub fn routes() -> Vec<rocket::Route> {
    routes![
        create,
        list,
        get,
        list_cards,
        update_cards,
        update,
        delete,
        generate_tabletop_simulator
    ]
}
