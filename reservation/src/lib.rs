use abi::{Error, Reservation, ReservationQuery};
use async_trait::async_trait;
use sqlx::PgPool;

mod manager;

pub type ReservationId = String;
pub type UserId = String;
pub type ResourceId = String;

#[derive(Debug)]
pub struct ReservationManager {
    pool: PgPool,
}

#[async_trait]
pub trait Rsvp {
    /// to make a reservation
    async fn reserve(&self, rsvp: Reservation) -> Result<Reservation, Error>;
    /// to change reservation status.
    /// if the current status is pending, to changed it to confirmed.
    async fn change_status(&self, id: ReservationId) -> Result<Reservation, Error>;
    // to update note
    async fn update_note(&self, id: ReservationId, note: String) -> Result<Reservation, Error>;
    // to delete one reservation
    async fn delete(&self, id: ReservationId) -> Result<Reservation, Error>;
    /// to get the reservation by id
    async fn get(&self, id: ReservationId) -> Result<Reservation, Error>;
    /// query reservations
    async fn query(
        &self,
        user_id: UserId,
        query: ReservationQuery,
    ) -> Result<Vec<Reservation>, Error>;
}
