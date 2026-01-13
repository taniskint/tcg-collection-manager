use rocket::State;
use rocket::http::Status;
use rocket::serde::json::Json;
use serde::{Deserialize, Serialize};

use crate::{AdminAuth, DbConn, ErrorResponse};

#[derive(Deserialize)]
pub struct CreateGameRequest {
    name: String,
    image_url: Option<String>,
    card_back_image_url: Option<String>,
}

#[derive(Serialize)]
pub struct GameResponse {
    id: i64,
}

#[derive(Serialize)]
pub struct GameListItem {
    id: i64,
    name: String,
    image_url: Option<String>,
    card_back_image_url: Option<String>,
    set_count: i64,
}

#[post("/", format = "json", data = "<req>")]
pub fn create(
    _auth: AdminAuth,
    db: &State<DbConn>,
    req: Json<CreateGameRequest>,
) -> Result<Json<GameResponse>, (Status, Json<ErrorResponse>)> {
    let conn = db.0.lock().unwrap();

    let id = super::create(
        &conn,
        &req.name,
        req.image_url.as_deref(),
        req.card_back_image_url.as_deref(),
    )
    .map_err(|e| {
        let (status, error) = match e {
            super::CreateGameError::NameExists => (Status::Conflict, "Game already exists"),
            super::CreateGameError::DatabaseError => {
                (Status::InternalServerError, "Failed to create game")
            }
        };
        (status, Json(ErrorResponse::new(error)))
    })?;

    Ok(Json(GameResponse { id }))
}

#[get("/<game_id>")]
pub fn get(
    db: &State<DbConn>,
    game_id: i64,
) -> Result<Json<GameListItem>, (Status, Json<ErrorResponse>)> {
    let conn = db.0.lock().unwrap();

    let game = super::get(&conn, game_id)
        .map_err(|_| {
            (
                Status::InternalServerError,
                Json(ErrorResponse::new("Failed to get game")),
            )
        })?
        .ok_or_else(|| (Status::NotFound, Json(ErrorResponse::new("Game not found"))))?;

    Ok(Json(GameListItem {
        id: game.id,
        name: game.name,
        image_url: game.image_url,
        card_back_image_url: game.card_back_image_url,
        set_count: game.set_count,
    }))
}

#[get("/")]
pub fn list(db: &State<DbConn>) -> Result<Json<Vec<GameListItem>>, (Status, Json<ErrorResponse>)> {
    let conn = db.0.lock().unwrap();

    let games = super::list(&conn).map_err(|_| {
        (
            Status::InternalServerError,
            Json(ErrorResponse::new("Failed to list games")),
        )
    })?;

    let items = games
        .into_iter()
        .map(|g| GameListItem {
            id: g.id,
            name: g.name,
            image_url: g.image_url,
            card_back_image_url: g.card_back_image_url,
            set_count: g.set_count,
        })
        .collect();

    Ok(Json(items))
}

pub fn routes() -> Vec<rocket::Route> {
    routes![create, get, list]
}
