use std::collections::HashMap;

use rusqlite::{Connection, Error as SqliteError, params};

pub struct DeckWithCollection {
    pub id: i64,
    pub collection_id: i64,
    pub name: String,
    pub created_at: String,
    pub collection_name: String,
    pub game_name: String,
    pub game_image_url: Option<String>,
    pub card_count: i64,
}

pub struct DeckCard {
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
pub enum CreateDeckError {
    CollectionNotFound,
    NotOwner,
    DatabaseError,
}

#[derive(Debug)]
pub enum GetDeckError {
    NotFound,
    NotOwner,
    DatabaseError,
}

#[derive(Debug)]
pub enum UpdateDeckCardsError {
    DeckNotFound,
    NotOwner,
    CardNotInCollection,
    InsufficientQuantity,
    DatabaseError,
}

#[derive(Debug)]
pub enum UpdateDeckError {
    NotFound,
    NotOwner,
    DatabaseError,
}

#[derive(Debug)]
pub enum DeleteDeckError {
    NotFound,
    NotOwner,
    DatabaseError,
}

pub fn init_table(conn: &Connection) -> Result<(), SqliteError> {
    conn.execute(
        "CREATE TABLE IF NOT EXISTS decks (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            collection_id INTEGER NOT NULL,
            name TEXT NOT NULL,
            created_at TEXT NOT NULL,
            FOREIGN KEY (collection_id) REFERENCES collections(id)
        )",
        [],
    )?;
    Ok(())
}

pub fn create(
    conn: &Connection,
    user_id: i64,
    collection_id: i64,
    name: &str,
) -> Result<i64, CreateDeckError> {
    // Check if collection exists and get its owner
    let collection_check: Result<i64, _> = conn.query_row(
        "SELECT user_id FROM collections WHERE id = ?1",
        params![collection_id],
        |row| row.get(0),
    );

    match collection_check {
        Ok(owner_id) => {
            if owner_id != user_id {
                return Err(CreateDeckError::NotOwner);
            }
        }
        Err(rusqlite::Error::QueryReturnedNoRows) => {
            return Err(CreateDeckError::CollectionNotFound);
        }
        Err(_) => {
            return Err(CreateDeckError::DatabaseError);
        }
    }

    let created_at = chrono::Utc::now().to_rfc3339();

    conn.execute(
        "INSERT INTO decks (collection_id, name, created_at) VALUES (?1, ?2, ?3)",
        params![collection_id, name, created_at],
    )
    .map_err(|_| CreateDeckError::DatabaseError)?;

    Ok(conn.last_insert_rowid())
}

pub fn list_by_user(
    conn: &Connection,
    user_id: i64,
) -> Result<Vec<DeckWithCollection>, SqliteError> {
    let mut stmt = conn.prepare(
        "SELECT d.id, d.collection_id, d.name, d.created_at, c.name, g.name, g.image_url,
                COALESCE(SUM(dc.quantity), 0) as card_count
         FROM decks d
         JOIN collections c ON d.collection_id = c.id
         JOIN games g ON c.game_id = g.id
         LEFT JOIN deck_cards dc ON d.id = dc.deck_id
         WHERE c.user_id = ?1
         GROUP BY d.id
         ORDER BY d.created_at DESC",
    )?;

    let decks = stmt
        .query_map(params![user_id], |row| {
            Ok(DeckWithCollection {
                id: row.get(0)?,
                collection_id: row.get(1)?,
                name: row.get(2)?,
                created_at: row.get(3)?,
                collection_name: row.get(4)?,
                game_name: row.get(5)?,
                game_image_url: row.get(6)?,
                card_count: row.get(7)?,
            })
        })?
        .collect::<Result<Vec<_>, _>>()?;

    Ok(decks)
}

pub fn get(
    conn: &Connection,
    user_id: i64,
    deck_id: i64,
) -> Result<DeckWithCollection, GetDeckError> {
    let result = conn.query_row(
        "SELECT d.id, d.collection_id, d.name, d.created_at, c.name, g.name, g.image_url, c.user_id,
                COALESCE(SUM(dc.quantity), 0) as card_count
         FROM decks d
         JOIN collections c ON d.collection_id = c.id
         JOIN games g ON c.game_id = g.id
         LEFT JOIN deck_cards dc ON d.id = dc.deck_id
         WHERE d.id = ?1
         GROUP BY d.id",
        params![deck_id],
        |row| {
            Ok((
                DeckWithCollection {
                    id: row.get(0)?,
                    collection_id: row.get(1)?,
                    name: row.get(2)?,
                    created_at: row.get(3)?,
                    collection_name: row.get(4)?,
                    game_name: row.get(5)?,
                    game_image_url: row.get(6)?,
                    card_count: row.get(8)?,
                },
                row.get::<_, i64>(7)?, // owner_id
            ))
        },
    );

    match result {
        Ok((deck, owner_id)) => {
            if owner_id != user_id {
                Err(GetDeckError::NotOwner)
            } else {
                Ok(deck)
            }
        }
        Err(rusqlite::Error::QueryReturnedNoRows) => Err(GetDeckError::NotFound),
        Err(_) => Err(GetDeckError::DatabaseError),
    }
}

pub fn update(
    conn: &Connection,
    deck_id: i64,
    user_id: i64,
    name: &str,
) -> Result<(), UpdateDeckError> {
    // First verify the deck exists and belongs to the user (via collection)
    let owner_check: Result<i64, _> = conn.query_row(
        "SELECT c.user_id FROM decks d
         JOIN collections c ON d.collection_id = c.id
         WHERE d.id = ?1",
        params![deck_id],
        |row| row.get(0),
    );

    match owner_check {
        Ok(owner_id) => {
            if owner_id != user_id {
                return Err(UpdateDeckError::NotOwner);
            }
        }
        Err(rusqlite::Error::QueryReturnedNoRows) => {
            return Err(UpdateDeckError::NotFound);
        }
        Err(_) => {
            return Err(UpdateDeckError::DatabaseError);
        }
    }

    conn.execute(
        "UPDATE decks SET name = ?1 WHERE id = ?2",
        params![name, deck_id],
    )
    .map_err(|_| UpdateDeckError::DatabaseError)?;

    Ok(())
}

pub fn delete(conn: &Connection, deck_id: i64, user_id: i64) -> Result<(), DeleteDeckError> {
    // First verify the deck exists and belongs to the user (via collection)
    let owner_check: Result<i64, _> = conn.query_row(
        "SELECT c.user_id FROM decks d
         JOIN collections c ON d.collection_id = c.id
         WHERE d.id = ?1",
        params![deck_id],
        |row| row.get(0),
    );

    match owner_check {
        Ok(owner_id) => {
            if owner_id != user_id {
                return Err(DeleteDeckError::NotOwner);
            }
        }
        Err(rusqlite::Error::QueryReturnedNoRows) => {
            return Err(DeleteDeckError::NotFound);
        }
        Err(_) => {
            return Err(DeleteDeckError::DatabaseError);
        }
    }

    // Delete associated deck_cards first
    conn.execute("DELETE FROM deck_cards WHERE deck_id = ?1", params![deck_id])
        .map_err(|_| DeleteDeckError::DatabaseError)?;

    // Delete the deck
    conn.execute("DELETE FROM decks WHERE id = ?1", params![deck_id])
        .map_err(|_| DeleteDeckError::DatabaseError)?;

    Ok(())
}

pub fn init_deck_cards_table(conn: &Connection) -> Result<(), SqliteError> {
    conn.execute(
        "CREATE TABLE IF NOT EXISTS deck_cards (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            deck_id INTEGER NOT NULL,
            card_id INTEGER NOT NULL,
            quantity INTEGER NOT NULL DEFAULT 1,
            FOREIGN KEY (deck_id) REFERENCES decks(id),
            FOREIGN KEY (card_id) REFERENCES cards(id),
            UNIQUE(deck_id, card_id)
        )",
        [],
    )?;
    Ok(())
}

fn deserialize_attributes(json: Option<String>) -> HashMap<String, String> {
    json.and_then(|s| serde_json::from_str(&s).ok())
        .unwrap_or_default()
}

pub fn list_deck_cards(
    conn: &Connection,
    deck_id: i64,
    user_id: i64,
) -> Result<Vec<DeckCard>, GetDeckError> {
    // First verify the deck exists and belongs to the user (via collection)
    let deck_check: Result<i64, _> = conn.query_row(
        "SELECT c.user_id FROM decks d
         JOIN collections c ON d.collection_id = c.id
         WHERE d.id = ?1",
        params![deck_id],
        |row| row.get(0),
    );

    match deck_check {
        Ok(owner_id) => {
            if owner_id != user_id {
                return Err(GetDeckError::NotOwner);
            }
        }
        Err(rusqlite::Error::QueryReturnedNoRows) => {
            return Err(GetDeckError::NotFound);
        }
        Err(_) => {
            return Err(GetDeckError::DatabaseError);
        }
    }

    // Get all cards in the deck with full details
    let mut stmt = conn
        .prepare(
            "SELECT c.id, c.name, c.collector_number, c.image_url, c.attributes,
                    s.id, s.name, dc.quantity
             FROM deck_cards dc
             JOIN cards c ON dc.card_id = c.id
             JOIN sets s ON c.set_id = s.id
             WHERE dc.deck_id = ?1
             ORDER BY s.name, c.collector_number",
        )
        .map_err(|_| GetDeckError::DatabaseError)?;

    let cards = stmt
        .query_map(params![deck_id], |row| {
            Ok(DeckCard {
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
        .map_err(|_| GetDeckError::DatabaseError)?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|_| GetDeckError::DatabaseError)?;

    Ok(cards)
}

pub fn update_deck_cards(
    conn: &Connection,
    deck_id: i64,
    user_id: i64,
    updates: &[CardQuantityUpdate],
) -> Result<(), UpdateDeckCardsError> {
    // First verify the deck exists and get its collection_id
    let deck_check: Result<(i64, i64), _> = conn.query_row(
        "SELECT c.user_id, d.collection_id FROM decks d
         JOIN collections c ON d.collection_id = c.id
         WHERE d.id = ?1",
        params![deck_id],
        |row| Ok((row.get(0)?, row.get(1)?)),
    );

    let collection_id = match deck_check {
        Ok((owner_id, cid)) => {
            if owner_id != user_id {
                return Err(UpdateDeckCardsError::NotOwner);
            }
            cid
        }
        Err(rusqlite::Error::QueryReturnedNoRows) => {
            return Err(UpdateDeckCardsError::DeckNotFound);
        }
        Err(_) => {
            return Err(UpdateDeckCardsError::DatabaseError);
        }
    };

    // Process each update
    for update in updates {
        // Verify card exists in the collection and get its quantity
        let collection_quantity: Result<i64, _> = conn.query_row(
            "SELECT quantity FROM collection_cards WHERE collection_id = ?1 AND card_id = ?2",
            params![collection_id, update.card_id],
            |row| row.get(0),
        );

        let collection_qty = match collection_quantity {
            Ok(qty) => qty,
            Err(rusqlite::Error::QueryReturnedNoRows) => {
                return Err(UpdateDeckCardsError::CardNotInCollection);
            }
            Err(_) => {
                return Err(UpdateDeckCardsError::DatabaseError);
            }
        };

        // Check that requested quantity doesn't exceed collection quantity
        if update.quantity > collection_qty {
            return Err(UpdateDeckCardsError::InsufficientQuantity);
        }

        if update.quantity <= 0 {
            // Remove the card from the deck
            conn.execute(
                "DELETE FROM deck_cards WHERE deck_id = ?1 AND card_id = ?2",
                params![deck_id, update.card_id],
            )
            .map_err(|_| UpdateDeckCardsError::DatabaseError)?;
        } else {
            // Upsert: insert or update the quantity
            conn.execute(
                "INSERT INTO deck_cards (deck_id, card_id, quantity)
                 VALUES (?1, ?2, ?3)
                 ON CONFLICT(deck_id, card_id) DO UPDATE SET quantity = ?3",
                params![deck_id, update.card_id, update.quantity],
            )
            .map_err(|_| UpdateDeckCardsError::DatabaseError)?;
        }
    }

    Ok(())
}
