use chrono::{DateTime, Utc};
use regex::Regex;
use std::{collections::HashMap, convert::Infallible, str::FromStr};

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ReservationConflictInfo {
    Parsed(ReservationConflict),
    Unparsed(String),
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ReservationConflict {
    pub new: ReservationWindow,
    pub old: ReservationWindow,
}

pub struct ParsedInfo {
    new: HashMap<String, String>,
    old: HashMap<String, String>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ReservationWindow {
    pub rid: String,
    pub start: DateTime<Utc>,
    pub end: DateTime<Utc>,
}

impl FromStr for ReservationConflictInfo {
    type Err = Infallible;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.parse() {
            Ok(conflict) => Ok(ReservationConflictInfo::Parsed(conflict)),
            Err(_) => Ok(ReservationConflictInfo::Unparsed(s.to_owned())),
        }
    }
}

impl TryFrom<ParsedInfo> for ReservationConflict {
    type Error = ();

    fn try_from(value: ParsedInfo) -> Result<Self, Self::Error> {
        Ok(Self {
            new: value.new.try_into()?,
            old: value.old.try_into()?,
        })
    }
}

impl TryFrom<HashMap<String, String>> for ReservationWindow {
    type Error = ();

    fn try_from(value: HashMap<String, String>) -> Result<Self, Self::Error> {
        let timespan_str = value.get("timespan").ok_or(())?.replace('"', "");
        let mut split = timespan_str.splitn(2, ',');
        let start = parse_datetime(split.next().ok_or(())?)?;
        let end = parse_datetime(split.next().ok_or(())?)?;
        Ok(Self {
            rid: value.get("resource_id").ok_or(())?.to_string(),
            start,
            end,
        })
    }
}

impl FromStr for ReservationConflict {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        ParsedInfo::from_str(s)?.try_into()
    }
}

impl FromStr for ParsedInfo {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let re = Regex::new(r#"\((?P<k1>[a-zA-Z0-9_-]+)\s*,\s*(?P<k2>[a-zA-Z0-9_-]+)\)=\((?P<v1>[a-zA-Z0-9_-]+)\s*,\s*\[(?P<v2>[^\)\]]+)"#).unwrap();
        let mut maps: Vec<Option<HashMap<String, String>>> = {
            let mut maps: Vec<Option<HashMap<String, String>>> = Vec::new();
            for cap in re.captures_iter(s) {
                let mut map: HashMap<String, String> = HashMap::new();
                map.insert(cap["k1"].to_owned(), cap["v1"].to_owned());
                map.insert(cap["k2"].to_owned(), cap["v2"].to_owned());
                maps.push(Some(map));
            }
            maps
        };
        if maps.len() != 2 {
            return Err(());
        }
        Ok(ParsedInfo {
            new: maps[0].take().unwrap(),
            old: maps[1].take().unwrap(),
        })
    }
}

fn parse_datetime(s: &str) -> Result<DateTime<Utc>, ()> {
    Ok(DateTime::parse_from_str(s, "%Y-%m-%d %H:%M:%S%#z")
        .map_err(|_| ())?
        .with_timezone(&Utc))
}

#[cfg(test)]
mod test {
    use super::*;
    use core::panic;
    const ERR_MSG: &str = "Key (resource_id, timespan)=(ocean-view-room-777, [\"2022-12-26 22:00:00+00\",\"2022-12-30 19:00:00+00\")) conflicts with existing key (resource_id, timespan)=(ocean-view-room-777, [\"2022-12-25 22:00:00+00\",\"2022-12-28 19:00:00+00\")).";
    #[test]
    fn parse_datetime_should_work() {
        let dt = parse_datetime("2022-12-26 22:00:00+00").unwrap();
        assert_eq!(dt.to_rfc3339(), "2022-12-26T22:00:00+00:00");
    }
    #[test]
    fn parsed_info_should_word() {
        let info: ParsedInfo = ERR_MSG.parse().unwrap();
        assert_eq!(info.new["resource_id"], "ocean-view-room-777");
        assert_eq!(
            info.new["timespan"],
            "\"2022-12-26 22:00:00+00\",\"2022-12-30 19:00:00+00\""
        );
        assert_eq!(info.old["resource_id"], "ocean-view-room-777");
        assert_eq!(
            info.old["timespan"],
            "\"2022-12-25 22:00:00+00\",\"2022-12-28 19:00:00+00\""
        );
    }
    #[test]
    fn hash_map_to_reservation_window_should_work() {
        let map = {
            let mut map = HashMap::new();
            map.insert("resource_id".to_string(), "ocean-view-room-713".to_string());
            map.insert(
                "timespan".to_string(),
                "\"2022-12-26 22:00:00+00\",\"2022-12-30 19:00:00+00\"".to_string(),
            );
            map
        };
        let window: ReservationWindow = map.try_into().unwrap();
        assert_eq!(window.rid, "ocean-view-room-713");
        assert_eq!(window.start.to_rfc3339(), "2022-12-26T22:00:00+00:00");
        assert_eq!(window.end.to_rfc3339(), "2022-12-30T19:00:00+00:00");
    }
    #[test]
    fn conflict_error_message_should_parse() {
        let info = ERR_MSG.parse().unwrap();
        match info {
            ReservationConflictInfo::Parsed(conflict) => {
                assert_eq!(conflict.new.rid, "ocean-view-room-777");
                assert_eq!(conflict.new.start.to_rfc3339(), "2022-12-26T22:00:00+00:00");
                assert_eq!(conflict.new.end.to_rfc3339(), "2022-12-30T19:00:00+00:00");
                assert_eq!(conflict.old.rid, "ocean-view-room-777");
                assert_eq!(conflict.old.start.to_rfc3339(), "2022-12-25T22:00:00+00:00");
                assert_eq!(conflict.old.end.to_rfc3339(), "2022-12-28T19:00:00+00:00");
            }
            ReservationConflictInfo::Unparsed(_) => panic!("should be parsed"),
        }
    }
}
