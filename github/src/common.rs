use serde::{de, ser, Deserialize, Serialize};
use std::fmt;

#[derive(Clone, Debug, Deserialize)]
pub struct NodeId(String);

impl NodeId {
    pub fn id(&self) -> &str {
        &self.0
    }
}

#[derive(Clone, Debug, Deserialize)]
pub struct Oid(String);

#[derive(Clone, Debug)]
pub struct DateTime(chrono::DateTime<chrono::Utc>);

impl Serialize for DateTime {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: ser::Serializer,
    {
        if serializer.is_human_readable() {
            serializer.serialize_str(&self.0.to_rfc3339())
        } else {
            serializer.serialize_i64(self.0.timestamp())
        }
    }
}

// DateTime's from Github can either be in unix epoch time or a string format
impl<'de> Deserialize<'de> for DateTime {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: de::Deserializer<'de>,
    {
        struct DateTimeVisitor;
        impl<'de> de::Visitor<'de> for DateTimeVisitor {
            type Value = DateTime;

            fn expecting(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                write!(f, "date time string or seconds since unix epoch")
            }

            fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                Ok(DateTime(
                    v.parse().map_err(|e| E::custom(format!("{}", e)))?,
                ))
            }

            fn visit_i64<E>(self, v: i64) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                use chrono::{offset::LocalResult, TimeZone};

                match chrono::Utc.timestamp_opt(v, 0) {
                    LocalResult::Single(datetime) => Ok(DateTime(datetime)),
                    _ => Err(E::custom(format!("'{}' is not a legal timestamp", v))),
                }
            }

            fn visit_u64<E>(self, v: u64) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                self.visit_i64(v as i64)
            }
        }

        if deserializer.is_human_readable() {
            deserializer.deserialize_any(DateTimeVisitor)
        } else {
            deserializer.deserialize_u64(DateTimeVisitor)
        }
    }
}
