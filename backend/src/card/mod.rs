mod model;
pub mod routes;

pub use model::{create_many, get, init_table, list, CreateCardError, CreateCardInput};
