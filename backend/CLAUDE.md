# TCG Collection Manager Backend

Rust backend for managing trading card game collections using Rocket framework.

## Commands

```bash
# Build
cargo build

# Run server (requires config.toml)
cargo run

# Run all tests
cargo test

# Run tests for a specific module
cargo test game::tests
cargo test set::tests
cargo test card::tests
cargo test user::tests
cargo test session::tests
cargo test collection::tests
cargo test deck::tests
cargo test booster::tests

# Lint
cargo clippy

# Format
cargo fmt
```

## Architecture

### Tech Stack
- **Framework**: Rocket 0.5.1 with JSON support
- **Database**: SQLite via rusqlite (bundled)
- **Auth**: bcrypt for passwords, UUID v4 for sessions, Bearer token for admin
- **Randomness**: rand crate for booster pack opening

### Module Structure

Each entity (game, set, card, user, session, collection, deck, booster) lives in `src/<entity>/` with:
- `mod.rs` - Re-exports and module glue
- `model.rs` - Database operations, takes `&Connection` directly
- `routes.rs` - HTTP handlers using `State<DbConn>`, converts model errors to HTTP status
- `tests.rs` - Integration tests using in-memory SQLite

### Key Types

- `DbConn(Mutex<Connection>)` - Thread-safe database connection wrapper
- `AdminAuth` - Request guard for admin-only endpoints (checks Bearer token)
- `SessionAuth` - Request guard for user endpoints (checks session_id cookie)
- `Config` - Loaded from `config.toml`, contains `admin_api_key`

### Route Mounting

Routes are mounted in `build_rocket()` in `src/main.rs`:
- `/api/users` - User registration
- `/api/sessions` - Login/logout/validate
- `/api/games` - Games CRUD + sets + cards + boosters (nested paths)
- `/api/collections` - User collections (requires session)
- `/api/decks` - User decks (requires session)
- `/api/boosters` - Open booster packs (requires session)
- `/` - Static files served from `../frontend` (using Rocket's FileServer)

### Testing Pattern

Tests use `create_test_client()` from `src/test_helpers.rs` which:
1. Creates in-memory SQLite database
2. Initializes all tables
3. Uses test config with known admin key
4. Returns a blocking Rocket test client

## Configuration

Create `config.toml` (see `config.example.toml`):
```toml
admin_api_key = "your-secret-key"
frontend_path = "../frontend"

[s3]
bucket = "your-bucket-name"
region = "us-west-2"
```

**Notes:**
- `admin_api_key` is required for admin endpoints
- `frontend_path` is required and specifies the location of static files
- `s3` configuration is optional and only needed for TTS deck atlas generation

## API Documentation

See `openapi.yaml` for complete API specification.
