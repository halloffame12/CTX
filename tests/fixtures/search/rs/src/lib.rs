pub mod models;
pub mod utils;

pub use models::user::User;
pub use models::user::UserRepository;
pub use utils::format::format_date;

pub const APP_NAME: &str = "ctx";

pub fn version() -> &'static str {
    "0.1.0"
}