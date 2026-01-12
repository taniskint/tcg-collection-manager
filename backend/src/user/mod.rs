mod model;
pub mod routes;

#[cfg(test)]
mod tests;

pub use model::{CreateUserError, UpdateUserError, DeleteUserError, create, update, delete, init_table};
