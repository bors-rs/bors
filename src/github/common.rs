use serde::{de, Deserialize};
use std::fmt;

#[derive(Debug, Deserialize)]
pub struct NodeId(String);

#[derive(Debug, Deserialize)]
pub struct Oid(String);

#[derive(Debug)]
pub struct DateTime(String);

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
                Ok(DateTime(v.to_owned()))
            }

            fn visit_u64<E>(self, v: u64) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                // TODO actually convert this to a timestamp
                Ok(DateTime(format!("{}", v)))
            }
        }

        if deserializer.is_human_readable() {
            deserializer.deserialize_any(DateTimeVisitor)
        } else {
            deserializer.deserialize_u64(DateTimeVisitor)
        }
    }
}
