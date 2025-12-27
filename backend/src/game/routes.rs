use rocket::http::Status;
use rocket::serde::json::Json;
use rocket::State;
use serde::{Deserialize, Serialize};

use crate::{AdminAuth, DbConn, ErrorResponse};

#[derive(Deserialize)]
pub struct CreateGameRequest {
    name: String,
    image_url: Option<String>,
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
}

#[post("/", format = "json", data = "<req>")]
pub fn create(
    _auth: AdminAuth,
    db: &State<DbConn>,
    req: Json<CreateGameRequest>,
) -> Result<Json<GameResponse>, (Status, Json<ErrorResponse>)> {
    let conn = db.0.lock().unwrap();

    let id = super::create(&conn, &req.name, req.image_url.as_deref()).map_err(|e| {
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
        })
        .collect();

    Ok(Json(items))
}

pub fn routes() -> Vec<rocket::Route> {
    routes![create, list]
}
