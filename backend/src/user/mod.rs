mod model;
pub mod routes;

#[cfg(test)]
mod tests;

pub use model::{CreateUserError, create, init_table};
