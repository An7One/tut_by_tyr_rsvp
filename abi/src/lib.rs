mod error;
mod pb;
mod types;
mod utils;

pub use error::*;
pub use pb::*;
pub use types::*;
pub use utils::*;

pub type ReservationId = String;
pub type UserId = String;
pub type ResourceId = String;

/// to validate the data structure,
/// to raise errors if it is invalid.
pub trait Validator {
    fn validate(&self) -> Result<(), Error>;
}

/// database equivalent of the enum `reservation_status`
#[derive(Clone, Copy, Debug, PartialEq, Eq, sqlx::Type)]
#[sqlx(type_name = "reservation_status", rename_all = "lowercase")]
pub enum RsvpStatus {
    Unknown,
    Pending,
    Confirmed,
    Blocked,
}

impl Validator for ReservationId {
    fn validate(&self) -> Result<(), Error> {
        if self.is_empty() {
            Err(Error::InvalidReservationId(self.to_owned()))
        } else {
            Ok(())
        }
    }
}

impl Validator for Reservation {
    fn validate(&self) -> Result<(), Error> {
        if self.user_id.is_empty() {
            return Err(Error::InvalidUserId(self.user_id.to_owned()));
        }
        if self.resource_id.is_empty() {
            return Err(Error::InvalidResourceId(self.resource_id.to_owned()));
        }
        if self.start.is_none() || self.end.is_none() {
            return Err(Error::InvalidTime);
        }
        Ok(())
    }
}
