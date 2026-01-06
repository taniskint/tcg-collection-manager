mod model;
pub mod routes;

#[cfg(test)]
mod tests;

pub use model::{
    create, get, init_deck_cards_table, init_table, list_by_user, list_deck_cards,
    update_deck_cards, CardQuantityUpdate, CreateDeckError, GetDeckError, UpdateDeckCardsError,
};
