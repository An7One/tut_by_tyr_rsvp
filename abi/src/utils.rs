use chrono::{DateTime, NaiveDateTime, TimeZone, Utc};
use prost_types::Timestamp;

pub fn convert_to_utc_time(ts: &Timestamp) -> DateTime<Utc> {
    let naive = NaiveDateTime::from_timestamp_opt(ts.seconds, ts.nanos as _).unwrap();
    Utc.from_utc_datetime(&naive)
}

pub fn convert_to_timestamp(dt: DateTime<Utc>) -> Timestamp {
    Timestamp {
        seconds: dt.timestamp(),
        nanos: dt.timestamp_subsec_nanos() as _,
    }
}
