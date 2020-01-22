use super::{Event, EventType};
use bytes::Bytes;

#[derive(Debug)]
pub struct Webhook {
    pub event: Event,
    pub event_type: EventType,
    pub guid: String,
    pub signature: Option<String>,
    pub body: Option<Bytes>,
}
