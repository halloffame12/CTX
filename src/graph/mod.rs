//! The code graph: symbols, dependencies, and impact queries over SQLite.

pub mod database;
pub mod dependencies;
pub mod impact;
pub mod symbols;

pub use database::Database;
