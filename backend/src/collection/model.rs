use std::collections::HashMap;

use rusqlite::{Connection, Error as SqliteError, params};

pub struct CollectionWithGame {
    pub id: i64,
    pub user_id: i64,
    pub game_id: i64,
    pub name: String,
    pub created_at: String,
    pub game_name: String,
    pub game_image_url: Option<String>,
    pub card_count: i64,
}

pub struct CollectionCard {
    pub id: i64,
    pub name: String,
    pub collector_number: String,
    pub image_url: Option<String>,
    pub attributes: HashMap<String, String>,
    pub set_id: i64,
    pub set_name: String,
    pub quantity: i64,
}

pub struct CardQuantityUpdate {
    pub card_id: i64,
    pub quantity: i64,
}

#[derive(Debug)]
pub enum CreateCollectionError {
    GameNotFound,
    DatabaseError,
}

#[derive(Debug)]
pub enum GetCollectionError {
    NotFound,
    NotOwner,
    DatabaseError,
}

#[derive(Debug)]
pub enum UpdateCollectionCardsError {
    CollectionNotFound,
    NotOwner,
    CardNotFound,
    GameMismatch,
    DatabaseError,
}

pub fn create(
    conn: &Connection,
    user_id: i64,
    game_id: i64,
    name: &str,
) -> Result<i64, CreateCollectionError> {
    let game_exists: bool = conn
        .query_row(
            "SELECT 1 FROM games WHERE id = ?1",
            params![game_id],
            |_| Ok(true),
        )
        .unwrap_or(false);

    if !game_exists {
        return Err(CreateCollectionError::GameNotFound);
    }

    let created_at = chrono::Utc::now().to_rfc3339();

    conn.execute(
        "INSERT INTO collections (user_id, game_id, name, created_at) VALUES (?1, ?2, ?3, ?4)",
        params![user_id, game_id, name, created_at],
    )
    .map_err(|_| CreateCollectionError::DatabaseError)?;

    Ok(conn.last_insert_rowid())
}

pub fn list_by_user(
    conn: &Connection,
    user_id: i64,
) -> Result<Vec<CollectionWithGame>, SqliteError> {
    let mut stmt = conn.prepare(
        "SELECT c.id, c.user_id, c.game_id, c.name, c.created_at, g.name, g.image_url,
                COALESCE(SUM(cc.quantity), 0) as card_count
         FROM collections c
         JOIN games g ON c.game_id = g.id
         LEFT JOIN collection_cards cc ON c.id = cc.collection_id
         WHERE c.user_id = ?1
         GROUP BY c.id
         ORDER BY c.created_at DESC",
    )?;

    let collections = stmt
        .query_map(params![user_id], |row| {
            Ok(CollectionWithGame {
                id: row.get(0)?,
                user_id: row.get(1)?,
                game_id: row.get(2)?,
                name: row.get(3)?,
                created_at: row.get(4)?,
                game_name: row.get(5)?,
                game_image_url: row.get(6)?,
                card_count: row.get(7)?,
            })
        })?
        .collect::<Result<Vec<_>, _>>()?;

    Ok(collections)
}

pub fn get(
    conn: &Connection,
    id: i64,
    user_id: i64,
) -> Result<CollectionWithGame, GetCollectionError> {
    let result = conn.query_row(
        "SELECT c.id, c.user_id, c.game_id, c.name, c.created_at, g.name, g.image_url,
                COALESCE(SUM(cc.quantity), 0) as card_count
         FROM collections c
         JOIN games g ON c.game_id = g.id
         LEFT JOIN collection_cards cc ON c.id = cc.collection_id
         WHERE c.id = ?1
         GROUP BY c.id",
        params![id],
        |row| {
            Ok(CollectionWithGame {
                id: row.get(0)?,
                user_id: row.get(1)?,
                game_id: row.get(2)?,
                name: row.get(3)?,
                created_at: row.get(4)?,
                game_name: row.get(5)?,
                game_image_url: row.get(6)?,
                card_count: row.get(7)?,
            })
        },
    );

    match result {
        Ok(collection) => {
            if collection.user_id != user_id {
                Err(GetCollectionError::NotOwner)
            } else {
                Ok(collection)
            }
        }
        Err(rusqlite::Error::QueryReturnedNoRows) => Err(GetCollectionError::NotFound),
        Err(_) => Err(GetCollectionError::DatabaseError),
    }
}

pub fn init_table(conn: &Connection) -> Result<(), SqliteError> {
    conn.execute(
        "CREATE TABLE IF NOT EXISTS collections (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            user_id INTEGER NOT NULL,
            game_id INTEGER NOT NULL,
            name TEXT NOT NULL,
            created_at TEXT NOT NULL,
            FOREIGN KEY (user_id) REFERENCES users(id),
            FOREIGN KEY (game_id) REFERENCES games(id)
        )",
        [],
    )?;
    Ok(())
}

pub fn init_collection_cards_table(conn: &Connection) -> Result<(), SqliteError> {
    conn.execute(
        "CREATE TABLE IF NOT EXISTS collection_cards (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            collection_id INTEGER NOT NULL,
            card_id INTEGER NOT NULL,
            quantity INTEGER NOT NULL DEFAULT 1,
            FOREIGN KEY (collection_id) REFERENCES collections(id),
            FOREIGN KEY (card_id) REFERENCES cards(id),
            UNIQUE(collection_id, card_id)
        )",
        [],
    )?;
    Ok(())
}

fn deserialize_attributes(json: Option<String>) -> HashMap<String, String> {
    json.and_then(|s| serde_json::from_str(&s).ok())
        .unwrap_or_default()
}

pub fn list_collection_cards(
    conn: &Connection,
    collection_id: i64,
    user_id: i64,
) -> Result<Vec<CollectionCard>, GetCollectionError> {
    // First verify the collection exists and belongs to the user
    let collection_check: Result<(i64,), _> = conn.query_row(
        "SELECT user_id FROM collections WHERE id = ?1",
        params![collection_id],
        |row| Ok((row.get(0)?,)),
    );

    match collection_check {
        Ok((owner_id,)) => {
            if owner_id != user_id {
                return Err(GetCollectionError::NotOwner);
            }
        }
        Err(rusqlite::Error::QueryReturnedNoRows) => {
            return Err(GetCollectionError::NotFound);
        }
        Err(_) => {
            return Err(GetCollectionError::DatabaseError);
        }
    }

    // Get all cards in the collection with full details
    let mut stmt = conn
        .prepare(
            "SELECT c.id, c.name, c.collector_number, c.image_url, c.attributes,
                    s.id, s.name, cc.quantity
             FROM collection_cards cc
             JOIN cards c ON cc.card_id = c.id
             JOIN sets s ON c.set_id = s.id
             WHERE cc.collection_id = ?1
             ORDER BY s.name, c.collector_number",
        )
        .map_err(|_| GetCollectionError::DatabaseError)?;

    let cards = stmt
        .query_map(params![collection_id], |row| {
            Ok(CollectionCard {
                id: row.get(0)?,
                name: row.get(1)?,
                collector_number: row.get(2)?,
                image_url: row.get(3)?,
                attributes: deserialize_attributes(row.get(4)?),
                set_id: row.get(5)?,
                set_name: row.get(6)?,
                quantity: row.get(7)?,
            })
        })
        .map_err(|_| GetCollectionError::DatabaseError)?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|_| GetCollectionError::DatabaseError)?;

    Ok(cards)
}

pub fn update_collection_cards(
    conn: &Connection,
    collection_id: i64,
    user_id: i64,
    updates: &[CardQuantityUpdate],
) -> Result<(), UpdateCollectionCardsError> {
    // First verify the collection exists and get its game_id
    let collection_check: Result<(i64, i64), _> = conn.query_row(
        "SELECT user_id, game_id FROM collections WHERE id = ?1",
        params![collection_id],
        |row| Ok((row.get(0)?, row.get(1)?)),
    );

    let game_id = match collection_check {
        Ok((owner_id, gid)) => {
            if owner_id != user_id {
                return Err(UpdateCollectionCardsError::NotOwner);
            }
            gid
        }
        Err(rusqlite::Error::QueryReturnedNoRows) => {
            return Err(UpdateCollectionCardsError::CollectionNotFound);
        }
        Err(_) => {
            return Err(UpdateCollectionCardsError::DatabaseError);
        }
    };

    // Process each update
    for update in updates {
        // TODO: If we have performance issues, consider batching these checks or caching the game_id lookup
        // Verify card exists and belongs to the same game
        let card_check: Result<i64, _> = conn.query_row(
            "SELECT g.id FROM cards c
             JOIN sets s ON c.set_id = s.id
             JOIN games g ON s.game_id = g.id
             WHERE c.id = ?1",
            params![update.card_id],
            |row| row.get(0),
        );

        match card_check {
            Ok(card_game_id) => {
                if card_game_id != game_id {
                    return Err(UpdateCollectionCardsError::GameMismatch);
                }
            }
            Err(rusqlite::Error::QueryReturnedNoRows) => {
                return Err(UpdateCollectionCardsError::CardNotFound);
            }
            Err(_) => {
                return Err(UpdateCollectionCardsError::DatabaseError);
            }
        }

        if update.quantity <= 0 {
            // Remove the card from the collection
            conn.execute(
                "DELETE FROM collection_cards WHERE collection_id = ?1 AND card_id = ?2",
                params![collection_id, update.card_id],
            )
            .map_err(|_| UpdateCollectionCardsError::DatabaseError)?;
        } else {
            // Upsert: insert or update the quantity
            conn.execute(
                "INSERT INTO collection_cards (collection_id, card_id, quantity)
                 VALUES (?1, ?2, ?3)
                 ON CONFLICT(collection_id, card_id) DO UPDATE SET quantity = ?3",
                params![collection_id, update.card_id, update.quantity],
            )
            .map_err(|_| UpdateCollectionCardsError::DatabaseError)?;
        }
    }

    Ok(())
}
