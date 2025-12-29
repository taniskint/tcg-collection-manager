mod model;
pub mod routes;

#[cfg(test)]
mod tests;

pub use model::{create_many, get, init_table, list, CreateCardError, CreateCardInput};
