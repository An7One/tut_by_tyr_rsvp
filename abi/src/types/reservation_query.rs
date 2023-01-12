use chrono::{DateTime, Utc};
use sqlx::postgres::types::PgRange;

use crate::{
    convert_to_timestamp, get_timespan, validate_range, Error, ReservationQuery, ReservationStatus,
    Validator,
};

impl ReservationQuery {
    pub fn new(
        uid: impl Into<String>,
        rid: impl Into<String>,
        start: DateTime<Utc>,
        end: DateTime<Utc>,
        status: ReservationStatus,
        page: i32,
        page_size: i32,
        desc: bool,
    ) -> Self {
        Self {
            user_id: uid.into(),
            resource_id: rid.into(),
            start: Some(convert_to_timestamp(start)),
            end: Some(convert_to_timestamp(end)),
            status: status as i32,
            page,
            page_size,
            desc,
        }
    }
    pub fn get_status(&self) -> ReservationStatus {
        return ReservationStatus::from_i32(self.status).unwrap();
    }
    pub fn get_timespan(&self) -> PgRange<DateTime<Utc>> {
        return get_timespan(self.start.as_ref(), self.end.as_ref());
    }
}

impl Validator for ReservationQuery {
    fn validate(&self) -> Result<(), crate::Error> {
        if self.user_id.is_empty() {
            return Err(Error::InvalidUserId(self.user_id.to_owned()));
        }
        if self.resource_id.is_empty() {
            return Err(Error::InvalidResourceId(self.resource_id.to_owned()));
        }
        validate_range(self.start.as_ref(), self.end.as_ref())
    }
}
