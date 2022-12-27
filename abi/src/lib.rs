mod pb;

use chrono::{DateTime, NaiveDateTime, TimeZone, Utc};
pub use pb::*;
use prost_types::Timestamp;

pub fn convert_to_utc_time(ts: Timestamp) -> DateTime<Utc> {
    let naive = NaiveDateTime::from_timestamp_opt(ts.seconds, ts.nanos as _).unwrap();
    Utc.from_utc_datetime(&naive)
}
