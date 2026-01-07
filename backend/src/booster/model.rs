use std::collections::HashMap;

use rand::Rng;
use rusqlite::{Connection, Error as SqliteError, params};
use serde::{Deserialize, Serialize};

pub struct Booster {
    pub id: i64,
    pub set_id: i64,
    pub name: String,
    pub spec: String,
}

#[derive(Serialize)]
pub struct OpenedCard {
    pub id: i64,
    pub name: String,
    pub collector_number: String,
    pub image_url: Option<String>,
    pub attributes: HashMap<String, String>,
    pub quantity: i64,
}

#[derive(Debug)]
pub enum CreateBoosterError {
    SetNotFound,
    NameExists,
    DatabaseError,
}

#[derive(Debug)]
pub enum OpenPacksError {
    BoosterNotFound,
    CollectionNotFound,
    NotOwner,
    GameMismatch,
    InvalidSpec,
    NoMatchingCards,
    DatabaseError,
}

#[derive(Deserialize)]
struct SlotChoice {
    attributes: HashMap<String, Vec<String>>,
    chance: f64,
    #[serde(rename = "allowDupes", default)]
    allow_dupes: bool,
}

pub fn init_table(conn: &Connection) -> Result<(), SqliteError> {
    conn.execute(
        "CREATE TABLE IF NOT EXISTS boosters (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            set_id INTEGER NOT NULL,
            name TEXT NOT NULL,
            spec TEXT NOT NULL,
            FOREIGN KEY (set_id) REFERENCES sets(id),
            UNIQUE(set_id, name)
        )",
        [],
    )?;
    Ok(())
}

pub fn create(
    conn: &Connection,
    set_id: i64,
    name: &str,
    spec: &str,
) -> Result<i64, CreateBoosterError> {
    // Verify the set exists
    let set_exists: bool = conn
        .query_row(
            "SELECT EXISTS(SELECT 1 FROM sets WHERE id = ?1)",
            params![set_id],
            |row| row.get(0),
        )
        .unwrap_or(false);

    if !set_exists {
        return Err(CreateBoosterError::SetNotFound);
    }

    conn.execute(
        "INSERT INTO boosters (set_id, name, spec) VALUES (?1, ?2, ?3)",
        params![set_id, name, spec],
    )
    .map_err(|e| {
        if let SqliteError::SqliteFailure(err, _) = &e
            && err.extended_code == 2067
        {
            return CreateBoosterError::NameExists;
        }
        CreateBoosterError::DatabaseError
    })?;

    Ok(conn.last_insert_rowid())
}

pub fn list(conn: &Connection, set_id: i64) -> Result<Vec<Booster>, SqliteError> {
    let mut stmt = conn.prepare("SELECT id, set_id, name, spec FROM boosters WHERE set_id = ?1")?;
    let boosters = stmt
        .query_map(params![set_id], |row| {
            Ok(Booster {
                id: row.get(0)?,
                set_id: row.get(1)?,
                name: row.get(2)?,
                spec: row.get(3)?,
            })
        })?
        .collect::<Result<Vec<_>, _>>()?;

    Ok(boosters)
}

pub fn get(conn: &Connection, booster_id: i64) -> Result<Option<Booster>, SqliteError> {
    let mut stmt = conn.prepare("SELECT id, set_id, name, spec FROM boosters WHERE id = ?1")?;
    let mut rows = stmt.query(params![booster_id])?;

    match rows.next()? {
        Some(row) => Ok(Some(Booster {
            id: row.get(0)?,
            set_id: row.get(1)?,
            name: row.get(2)?,
            spec: row.get(3)?,
        })),
        None => Ok(None),
    }
}

struct CardInfo {
    id: i64,
    name: String,
    collector_number: String,
    image_url: Option<String>,
    attributes: HashMap<String, String>,
}

fn deserialize_attributes(json: Option<String>) -> HashMap<String, String> {
    json.and_then(|s| serde_json::from_str(&s).ok())
        .unwrap_or_default()
}

pub fn open_packs(
    conn: &Connection,
    booster_id: i64,
    collection_id: i64,
    user_id: i64,
    count: i64,
) -> Result<Vec<OpenedCard>, OpenPacksError> {
    // 1. Get booster and its set_id
    let booster = get(conn, booster_id)
        .map_err(|_| OpenPacksError::DatabaseError)?
        .ok_or(OpenPacksError::BoosterNotFound)?;

    // 2. Get game_id from the set
    let set_game_id: i64 = conn
        .query_row(
            "SELECT game_id FROM sets WHERE id = ?1",
            params![booster.set_id],
            |row| row.get(0),
        )
        .map_err(|_| OpenPacksError::DatabaseError)?;

    // 3. Verify collection belongs to user and matches the game
    let collection_check: Result<(i64, i64), _> = conn.query_row(
        "SELECT user_id, game_id FROM collections WHERE id = ?1",
        params![collection_id],
        |row| Ok((row.get(0)?, row.get(1)?)),
    );

    match collection_check {
        Ok((owner_id, game_id)) => {
            if owner_id != user_id {
                return Err(OpenPacksError::NotOwner);
            }
            if game_id != set_game_id {
                return Err(OpenPacksError::GameMismatch);
            }
        }
        Err(rusqlite::Error::QueryReturnedNoRows) => {
            return Err(OpenPacksError::CollectionNotFound);
        }
        Err(_) => {
            return Err(OpenPacksError::DatabaseError);
        }
    }

    // 4. Parse the spec JSON
    let slots: Vec<Vec<SlotChoice>> =
        serde_json::from_str(&booster.spec).map_err(|_| OpenPacksError::InvalidSpec)?;

    // 5. Get all cards from the set
    let mut stmt = conn
        .prepare(
            "SELECT id, name, collector_number, image_url, attributes
             FROM cards WHERE set_id = ?1",
        )
        .map_err(|_| OpenPacksError::DatabaseError)?;

    let cards: Vec<CardInfo> = stmt
        .query_map(params![booster.set_id], |row| {
            Ok(CardInfo {
                id: row.get(0)?,
                name: row.get(1)?,
                collector_number: row.get(2)?,
                image_url: row.get(3)?,
                attributes: deserialize_attributes(row.get(4)?),
            })
        })
        .map_err(|_| OpenPacksError::DatabaseError)?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|_| OpenPacksError::DatabaseError)?;

    // 6. Open packs and collect cards
    let mut rng = rand::rng();
    let mut opened_cards: HashMap<i64, (CardInfo, i64)> = HashMap::new();

    for _ in 0..count {
        let mut pack_card_ids: Vec<i64> = Vec::new();

        for slot in &slots {
            // Roll for which choice in this slot
            let roll: f64 = rng.random();
            let mut cumulative = 0.0;
            let mut chosen_choice: Option<&SlotChoice> = None;

            for choice in slot {
                cumulative += choice.chance;
                if roll < cumulative {
                    chosen_choice = Some(choice);
                    break;
                }
            }

            // Default to last choice if none selected (handles floating point errors)
            let choice = chosen_choice.unwrap_or_else(|| slot.last().unwrap());

            // Filter cards that match the attributes
            let matching_cards: Vec<&CardInfo> = cards
                .iter()
                .filter(|card| {
                    // Check if card matches all attribute requirements
                    for (attr_key, attr_values) in &choice.attributes {
                        if let Some(card_attr_value) = card.attributes.get(attr_key) {
                            // Card has this attribute, check if it matches any of the allowed values
                            if !attr_values.iter().any(|v| v == card_attr_value) {
                                return false;
                            }
                        } else {
                            // Card doesn't have this attribute
                            return false;
                        }
                    }
                    true
                })
                .filter(|card| {
                    // If allowDupes is false, skip cards already in this pack
                    if !choice.allow_dupes {
                        !pack_card_ids.contains(&card.id)
                    } else {
                        true
                    }
                })
                .collect();

            if matching_cards.is_empty() {
                return Err(OpenPacksError::NoMatchingCards);
            }

            // Pick a random card from matching cards
            let idx = rng.random_range(0..matching_cards.len());
            let selected_card = matching_cards[idx];

            pack_card_ids.push(selected_card.id);

            // Add to opened_cards
            opened_cards
                .entry(selected_card.id)
                .and_modify(|(_, qty)| *qty += 1)
                .or_insert_with(|| {
                    (
                        CardInfo {
                            id: selected_card.id,
                            name: selected_card.name.clone(),
                            collector_number: selected_card.collector_number.clone(),
                            image_url: selected_card.image_url.clone(),
                            attributes: selected_card.attributes.clone(),
                        },
                        1,
                    )
                });
        }
    }

    // 7. Add cards to collection_cards table (upsert)
    for (card_id, (_, quantity)) in &opened_cards {
        conn.execute(
            "INSERT INTO collection_cards (collection_id, card_id, quantity)
             VALUES (?1, ?2, ?3)
             ON CONFLICT(collection_id, card_id) DO UPDATE SET quantity = quantity + ?3",
            params![collection_id, card_id, quantity],
        )
        .map_err(|_| OpenPacksError::DatabaseError)?;
    }

    // 8. Return aggregated OpenedCard list
    let result: Vec<OpenedCard> = opened_cards
        .into_iter()
        .map(|(_, (card, quantity))| OpenedCard {
            id: card.id,
            name: card.name,
            collector_number: card.collector_number,
            image_url: card.image_url,
            attributes: card.attributes,
            quantity,
        })
        .collect();

    Ok(result)
}
