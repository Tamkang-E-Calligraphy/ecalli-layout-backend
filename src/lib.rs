use fjall::Database;
use std::sync::OnceLock;

pub mod api;
pub mod feature;

pub static DB: OnceLock<Database> = OnceLock::new();
pub const KEY: &str = "default_gen";
