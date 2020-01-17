use super::{Event, EventType};
use serde::de;
use serde_json::Value;
use std::str::FromStr;

// TODO need to store the hash somewhere so it can be checked
#[derive(Debug)]
pub struct WebhookPayload {
    request_id: String,
    event: Event,
}

impl WebhookPayload {
    pub fn from_json(value: &[u8]) -> Result<Self, serde_json::Error> {
        let payload: Value = serde_json::from_slice(value)?;
        match payload {
            Value::Object(map) => {
                let event_type = EventType::from_str(
                    map.get("x-github-event")
                        .ok_or_else(|| de::Error::custom("x-github-event missing"))?
                        .as_str()
                        .ok_or_else(|| de::Error::custom("x-github-event not a string"))?,
                )
                .map_err(|_| de::Error::custom("x-github-event invalid"))?;

                let request_id = map
                    .get("x-request-id")
                    .ok_or_else(|| de::Error::custom("x-request-id missing"))?
                    .as_str()
                    .ok_or_else(|| de::Error::custom("x-request-id not a string"))?
                    .to_string();

                let body = map
                    .get("body")
                    .ok_or_else(|| de::Error::custom("webhook body missing"))?;
                let event = Event::from_value(&event_type, body.clone())?;

                Ok(WebhookPayload { request_id, event })
            }
            _ => Err(de::Error::custom("webhook payload not an object")),
        }
    }
}
