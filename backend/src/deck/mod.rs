mod model;
pub mod routes;

#[cfg(test)]
mod tests;

pub use model::{
    create, delete, get, init_deck_cards_table, init_table, list_by_user, list_deck_cards, update,
    update_deck_cards, CardQuantityUpdate, CreateDeckError, DeleteDeckError, GetDeckError,
    UpdateDeckCardsError, UpdateDeckError,
};
