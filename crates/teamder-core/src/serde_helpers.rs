//! Serde helpers for dealing with mixed-format date fields in MongoDB.
//!
//! Some writes use `bson::DateTime` (which serde-deserializes as a `{"$date": ...}`
//! map) while older / seed writes are RFC 3339 strings. These helpers accept either.

use chrono::{DateTime, TimeZone, Utc};
use serde::{
    de::{self, MapAccess, Visitor},
    Deserialize, Deserializer, Serialize, Serializer,
};
use std::fmt;

pub mod flexible_datetime {
    use super::*;

    pub fn serialize<S: Serializer>(dt: &DateTime<Utc>, s: S) -> Result<S::Ok, S::Error> {
        dt.serialize(s)
    }

    pub fn deserialize<'de, D: Deserializer<'de>>(d: D) -> Result<DateTime<Utc>, D::Error> {
        d.deserialize_any(FlexVisitor)
    }
}

pub mod flexible_datetime_opt {
    use super::*;

    pub fn serialize<S: Serializer>(
        dt: &Option<DateTime<Utc>>,
        s: S,
    ) -> Result<S::Ok, S::Error> {
        dt.serialize(s)
    }

    pub fn deserialize<'de, D: Deserializer<'de>>(
        d: D,
    ) -> Result<Option<DateTime<Utc>>, D::Error> {
        #[derive(Deserialize)]
        struct Wrap(#[serde(deserialize_with = "super::flexible_datetime::deserialize")] DateTime<Utc>);
        Ok(Option::<Wrap>::deserialize(d)?.map(|Wrap(dt)| dt))
    }
}

struct FlexVisitor;

impl<'de> Visitor<'de> for FlexVisitor {
    type Value = DateTime<Utc>;

    fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str("RFC 3339 datetime string, millisecond timestamp, or BSON Date map")
    }

    fn visit_str<E: de::Error>(self, v: &str) -> Result<DateTime<Utc>, E> {
        DateTime::parse_from_rfc3339(v)
            .map(|d| d.with_timezone(&Utc))
            .map_err(de::Error::custom)
    }

    fn visit_string<E: de::Error>(self, v: String) -> Result<DateTime<Utc>, E> {
        self.visit_str(&v)
    }

    fn visit_i64<E: de::Error>(self, v: i64) -> Result<DateTime<Utc>, E> {
        Utc.timestamp_millis_opt(v)
            .single()
            .ok_or_else(|| de::Error::custom("invalid millisecond timestamp"))
    }

    fn visit_u64<E: de::Error>(self, v: u64) -> Result<DateTime<Utc>, E> {
        self.visit_i64(v as i64)
    }

    fn visit_map<A: MapAccess<'de>>(self, mut m: A) -> Result<DateTime<Utc>, A::Error> {
        while let Some(key) = m.next_key::<String>()? {
            if key == "$date" {
                let v = m.next_value::<DateInner>()?;
                return v.to_chrono().map_err(de::Error::custom);
            }
            let _: de::IgnoredAny = m.next_value()?;
        }
        Err(de::Error::custom("missing $date in BSON Date map"))
    }
}

#[derive(Deserialize)]
#[serde(untagged)]
enum DateInner {
    Millis(i64),
    NumberLong {
        #[serde(rename = "$numberLong")]
        n: String,
    },
    Str(String),
}

impl DateInner {
    fn to_chrono(self) -> Result<DateTime<Utc>, String> {
        let ms = match self {
            DateInner::Millis(n) => n,
            DateInner::NumberLong { n } => n.parse::<i64>().map_err(|e| e.to_string())?,
            DateInner::Str(s) => {
                return DateTime::parse_from_rfc3339(&s)
                    .map(|d| d.with_timezone(&Utc))
                    .map_err(|e| e.to_string());
            }
        };
        Utc.timestamp_millis_opt(ms)
            .single()
            .ok_or_else(|| "invalid millisecond timestamp".into())
    }
}
