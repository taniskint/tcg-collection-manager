mod model;
pub mod routes;

#[cfg(test)]
mod tests;

pub use model::{CreateDeckError, GetDeckError, create, get, init_table, list_by_user};
