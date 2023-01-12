use abi::{Error, Reservation, ReservationQuery, ReservationStatus, Validator};
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use sqlx::{postgres::types::PgRange, types::Uuid, PgPool, Row};

use crate::{ReservationId, ReservationManager, Rsvp};

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
        let id: Uuid = sqlx::query("INSERT INTO rsvp.reservations (user_id, resource_id, timespan, note, status) VALUES ($1, $2, $3, $4, $5::rsvp.reservation_status) RETURNING id")
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
    async fn change_status(&self, id: ReservationId) -> Result<Reservation, Error> {
        // if the current status is pending,
        // to change it to confirmed,
        // otherwise to do nothing
        let id: Uuid = Uuid::parse_str(&id).map_err(|_| Error::InvalidReservationId(id.clone()))?;
        let rsvp: Reservation = sqlx::query_as("UPDATE rsvp.reservations SET status = 'confirmed' WHERE id = $1 AND status = 'pending' RETURNING *")
            .bind(id)
            .fetch_one(&self.pool)
            .await?;
        // println!("{:?}", rsvp);
        Ok(rsvp)
    }
    /// to update the note of the reservation
    async fn update_note(&self, id: ReservationId, note: String) -> Result<Reservation, Error> {
        id.validate()?;
        let rsvp: Reservation =
            sqlx::query_as("UPDATE rsvp.reservations SET note = $1 WHERE id = $2 RETURNING *")
                .bind(note)
                .bind(id)
                .fetch_one(&self.pool)
                .await?;
        Ok(rsvp)
    }
    /// to delete the reservation by its id
    async fn delete(&self, id: ReservationId) -> Result<Reservation, Error> {
        id.validate()?;
        let rsvp: Reservation =
            sqlx::query_as("DELETE FROM rsvp.reservations WHERE id = $1 RETURNING *")
                .bind(id)
                .fetch_one(&self.pool)
                .await?;
        Ok(rsvp)
    }
    // to get one reservation by its id
    async fn get(&self, id: ReservationId) -> Result<Reservation, Error> {
        id.validate()?;
        let rsvp: Reservation = sqlx::query_as("SELECT FROM rsvp.reservations WHERE id = $id")
            .bind(id)
            .fetch_one(&self.pool)
            .await?;
        Ok(rsvp)
    }
    // to query reservation(s) by `query`
    async fn query(&self, query: ReservationQuery) -> Result<Vec<Reservation>, Error> {
        let user_id = str_to_option(&query.user_id);
        let resource_id = str_to_option(&query.resource_id);
        let range: PgRange<DateTime<Utc>> = query.get_timespan();
        let status =
            ReservationStatus::from_i32(query.status).unwrap_or(ReservationStatus::Pending);
        let rsvp = sqlx::query_as(
            "SELECT * FROM rsvp.query($1, $2, $3, $4::rsvp.reservation_status, $5, $6, $7)",
        )
        .bind(user_id)
        .bind(resource_id)
        .bind(range)
        .bind(status.to_string())
        .bind(query.page)
        .bind(query.desc)
        .bind(query.page_size)
        .fetch_all(&self.pool)
        .await?;
        Ok(rsvp)
    }
}

fn str_to_option(s: &str) -> Option<&str> {
    return if s.is_empty() { None } else { Some(s) };
}

impl ReservationManager {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use abi::Error::*;
    use abi::{ReservationConflict, ReservationConflictInfo, ReservationWindow};
    const DUMMY_USER_ID_LEON: &str = "dummy_user_id_leon";
    const DUMMY_USER_ID_ALICE: &str = "dummy_user_id_alice";
    const DUMMY_ROOM_NAME: &str = "ocean-view-room-777";
    #[sqlx_database_tester::test(pool(variable = "migrated_pool", migrations = "../migrations"))]
    async fn reserve_should_work_for_valid_time_window() {
        let (rsvp, _manager) = make_reservation_for_leon(migrated_pool.clone()).await;
        assert!(!rsvp.id.is_empty());
    }
    #[sqlx_database_tester::test(pool(variable = "migrated_pool", migrations = "../migrations"))]
    async fn reserve_should_reject_conflicting_reservations() {
        let (_rsvp1, manager) = make_reservation_for_leon(migrated_pool.clone()).await;
        let rsvp2 = Reservation::new_pending(
            DUMMY_USER_ID_ALICE,
            DUMMY_ROOM_NAME,
            "2023-12-26T15:00:00-0700".parse().unwrap(),
            "2023-12-30T12:00:00-0700".parse().unwrap(),
            "hello",
        );
        let err = manager.reserve(rsvp2).await.unwrap_err();
        let info = ReservationConflictInfo::Parsed(ReservationConflict {
            new: ReservationWindow {
                rid: DUMMY_ROOM_NAME.to_owned(),
                start: "2023-12-26T15:00:00-0700".parse().unwrap(),
                end: "2023-12-30T12:00:00-0700".parse().unwrap(),
            },
            old: ReservationWindow {
                rid: DUMMY_ROOM_NAME.to_owned(),
                start: "2023-12-25T15:00:00-0700".parse().unwrap(),
                end: "2023-12-28T12:00:00-0700".parse().unwrap(),
            },
        });
        assert_eq!(err, ConflictingReservation(info));
    }
    #[sqlx_database_tester::test(pool(variable = "migrated_pool", migrations = "../migrations"))]
    async fn reserve_change_status_should_work() {
        let (rsvp_pending, manager) = make_reservation_for_leon(migrated_pool.clone()).await;
        let rsvp_after_change = manager.change_status(rsvp_pending.id).await.unwrap();
        assert_eq!(
            rsvp_after_change.status,
            ReservationStatus::Confirmed as i32
        );
    }
    #[sqlx_database_tester::test(pool(variable = "migrated_pool", migrations = "../migrations"))]
    async fn reserve_change_staus_not_pending_should_do_nothing() {
        let (rsvp_pending, manager) = make_reservation_for_leon(migrated_pool.clone()).await;
        let rsvp = manager.change_status(rsvp_pending.id).await.unwrap();
        let err = manager.change_status(rsvp.id).await.unwrap_err();
        assert_eq!(err, Error::NotFound);
    }
    #[sqlx_database_tester::test(pool(variable = "migrated_pool", migrations = "../migrations"))]
    async fn query_reservations_should_work() {
        let (rsvp, manager) = make_reservation_for_leon(migrated_pool.clone()).await;
        let query = ReservationQuery::new(
            DUMMY_USER_ID_LEON,
            DUMMY_ROOM_NAME,
            "2023-12-24T15:00:00-0700".parse().unwrap(),
            "2023-12-30T12:00:00-0700".parse().unwrap(),
            ReservationStatus::Pending,
            1,
            10,
            false,
        );
        let rsvps = manager.query(query).await.unwrap();
        assert_eq!(rsvps.len(), 1);
        assert_eq!(rsvps[0], rsvp);
    }
    async fn make_reservation_for_leon(pool: PgPool) -> (Reservation, ReservationManager) {
        make_reservation(
            pool,
            DUMMY_USER_ID_LEON,
            DUMMY_ROOM_NAME,
            "2023-12-25T15:00:00-0700",
            "2023-12-28T12:00:00-0700",
            "I will arrive at 3PM. Please help to upgrade to the executive room if possible. Thanks."
        ).await
    }
    async fn make_reservation(
        pool: PgPool,
        uid: &str,
        rid: &str,
        start: &str,
        end: &str,
        note: &str,
    ) -> (Reservation, ReservationManager) {
        let manager = ReservationManager::new(pool.clone());
        let rsvp =
            Reservation::new_pending(uid, rid, start.parse().unwrap(), end.parse().unwrap(), note);
        let res = manager.reserve(rsvp).await;
        println!("{:?}", res);
        (res.unwrap(), manager)
    }
}
