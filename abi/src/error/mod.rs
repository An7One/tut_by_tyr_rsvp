pub mod conflict;
pub use conflict::{ReservationConflict, ReservationConflictInfo, ReservationWindow};

use sqlx::postgres::PgDatabaseError;
use thiserror;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    // #[error("data store")]
    // Disconnect(#[from] io::Error),
    // #[error("the data for key `{0}` is not available")]
    // Redaction(String),
    // #[error("invalid header (expected {expected:?}, found {found:?}")]
    // InvalidHeader { expected: String, found: String },
    #[error("Database error")]
    DbError(sqlx::Error),
    #[error("Invalid resource id: {0}")]
    InvalidResourceId(String),
    #[error("Invalid start or end time for the reservation")]
    InvalidTime,
    #[error("Conflicting Reservation")]
    ConflictingReservation(ReservationConflictInfo),
    #[error("Invalid user id: {0}")]
    InvalidUserId(String),
    #[error("unknown data store error")]
    Unknown,
}

impl From<sqlx::Error> for Error {
    fn from(e: sqlx::Error) -> Self {
        match e {
            sqlx::Error::Database(e) => {
                let err: &PgDatabaseError = e.downcast_ref();
                match (err.code(), err.schema(), err.table()) {
                    ("23P01", Some("rsvp"), Some("reservations")) => {
                        Error::ConflictingReservation(err.detail().unwrap().parse().unwrap())
                    }
                    _ => Error::DbError(sqlx::Error::Database(e)),
                }
            }
            _ => Error::DbError(e),
        }
    }
}
