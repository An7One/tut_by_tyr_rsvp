mod error;
mod pb;
mod types;
mod utils;

pub use error::*;
pub use pb::*;
pub use types::*;
pub use utils::*;

/// database equivalent of the enum `reservation_status`
#[derive(Clone, Copy, Debug, PartialEq, Eq, sqlx::Type)]
#[sqlx(type_name = "reservation_status", rename_all = "lowercase")]
pub enum RsvpStatus {
    Unknown,
    Pending,
    Confirmed,
    Blocked,
}
