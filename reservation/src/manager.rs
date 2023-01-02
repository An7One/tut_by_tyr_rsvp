use abi::{Error, Reservation, ReservationQuery, ReservationStatus};
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use sqlx::{postgres::types::PgRange, query, types::Uuid, PgPool, Row};

use crate::{ReservationId, ReservationManager, Rsvp, UserId};

#[async_trait]
impl Rsvp for ReservationManager {
    async fn reserve(&self, mut rsvp: Reservation) -> Result<Reservation, Error> {
        if rsvp.start.is_none() || rsvp.end.is_none() {
            return Err(Error::InvalidTime);
        }
        rsvp.validate()?;
        let status = ReservationStatus::from_i32(rsvp.status).unwrap_or(ReservationStatus::Pending);
        let timespan: PgRange<DateTime<Utc>> = rsvp.get_timespan().into();
        // to run the query
        let id: Uuid = query("INSERT INTO rsvp.reservations (user_id, resource_id, timespan, note, status) VALUES ($1, $2, $3, $4, $5::rsvp.reservation_status) RETURNING id")
        .bind(rsvp.user_id.to_owned())
        .bind(rsvp.resource_id.to_owned())
        .bind(timespan)
        .bind(rsvp.note.to_owned())
        .bind(status.to_string())
        .fetch_one(&self.pool)
        .await?
        .get(0);
        rsvp.id = id.to_string();
        Ok(rsvp)
    }
    async fn change_status(&self, _id: ReservationId) -> Result<Reservation, Error> {
        todo!()
    }
    async fn update_note(&self, _id: ReservationId, _note: String) -> Result<Reservation, Error> {
        todo!()
    }
    async fn delete(&self, _id: ReservationId) -> Result<(), Error> {
        todo!()
    }
    async fn get(&self, _id: ReservationId) -> Result<Reservation, Error> {
        todo!()
    }
    async fn query(
        &self,
        _user_id: UserId,
        _query: ReservationQuery,
    ) -> Result<Vec<Reservation>, Error> {
        todo!()
    }
}

impl ReservationManager {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use abi::ReservationConflictInfo;
    use chrono::FixedOffset;
    #[sqlx_database_tester::test(pool(variable = "migrated_pool", migrations = "../migrations"))]
    async fn reserve_should_work_for_valid_time_window() {
        let manager = ReservationManager::new(migrated_pool.clone());
        let start: DateTime<FixedOffset> = "2022-12-25T15:00:00-0700".parse().unwrap();
        let end: DateTime<FixedOffset> = "2022-12-28T12:00:00-0700".parse().unwrap();
        let rsvp_note =
            "I will arrive at 3PM. Please help upgrade to the executive room if possible"
                .to_owned();
        let rsvp: Reservation = Reservation::new_pending(
            "Leon".to_owned(),
            "ocean-view-room-777".to_owned(),
            start,
            end,
            rsvp_note,
        );
        let actual = manager.reserve(rsvp).await.unwrap();
        assert!(actual.id != "");
    }
    #[sqlx_database_tester::test(pool(variable = "migrated_pool", migrations = "../migrations"))]
    async fn reserve_should_reject_conflicting_reservations() {
        let manager = ReservationManager::new(migrated_pool.clone());
        let rsvp1 = Reservation::new_pending(
            "leon",
            "ocean-view-room-777",
            "2022-12-25T15:00:00-0700".parse().unwrap(),
            "2022-12-28T12:00:00-0700".parse().unwrap(),
            "hello",
        );
        let rsvp2 = Reservation::new_pending(
            "alice",
            "ocean-view-room-777",
            "2022-12-26T15:00:00-0700".parse().unwrap(),
            "2022-12-30T12:00:00-0700".parse().unwrap(),
            "hello",
        );
        let _reservation1 = manager.reserve(rsvp1).await.unwrap();
        let err = manager.reserve(rsvp2).await.unwrap_err();
        println!("{:?}", err);
        if let Error::ConflictingReservation(ReservationConflictInfo::Parsed(info)) = err{
            assert_eq!(info.new.rid, "ocean-view-room-777");
            assert_eq!(info.old.start.to_rfc3339(), "2022-12-25T22:00:00+00:00");
            assert_eq!(info.old.end.to_rfc3339(), "2022-12-28T19:00:00+00:00");
        }else{
            panic!("expect conflict reservation error");
        }
    }
}
