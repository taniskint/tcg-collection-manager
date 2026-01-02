mod model;
pub mod routes;

#[cfg(test)]
mod tests;

pub use model::{create, get, init_table, list_by_user, CreateCollectionError, GetCollectionError};
