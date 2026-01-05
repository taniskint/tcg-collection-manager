mod model;
pub mod routes;

#[cfg(test)]
mod tests;

pub use model::{CreateGameError, create, get, init_table, list};
