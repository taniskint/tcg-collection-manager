use rusqlite::{params, Connection, Error as SqliteError};

pub struct Collection {
    pub id: i64,
    pub user_id: i64,
    pub game_id: i64,
    pub name: String,
    pub created_at: String,
}

pub struct CollectionWithGame {
    pub id: i64,
    pub user_id: i64,
    pub game_id: i64,
    pub name: String,
    pub created_at: String,
    pub game_name: String,
    pub game_image_url: Option<String>,
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

pub fn list_by_user(conn: &Connection, user_id: i64) -> Result<Vec<CollectionWithGame>, SqliteError> {
    let mut stmt = conn.prepare(
        "SELECT c.id, c.user_id, c.game_id, c.name, c.created_at, g.name, g.image_url
         FROM collections c
         JOIN games g ON c.game_id = g.id
         WHERE c.user_id = ?1
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
            })
        })?
        .collect::<Result<Vec<_>, _>>()?;

    Ok(collections)
}

pub fn get(conn: &Connection, id: i64, user_id: i64) -> Result<CollectionWithGame, GetCollectionError> {
    let result = conn.query_row(
        "SELECT c.id, c.user_id, c.game_id, c.name, c.created_at, g.name, g.image_url
         FROM collections c
         JOIN games g ON c.game_id = g.id
         WHERE c.id = ?1",
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
