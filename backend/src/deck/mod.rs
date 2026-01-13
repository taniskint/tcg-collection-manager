mod model;
pub mod routes;

#[cfg(test)]
mod tests;

pub use model::{
    create, delete, get, init_deck_cards_table, init_table, list_by_user, list_deck_cards,
    list_deck_cards_public, update, update_deck_cards, CardQuantityUpdate, CreateDeckError,
    DeckCard, DeleteDeckError, GetDeckError, UpdateDeckCardsError, UpdateDeckError,
};
