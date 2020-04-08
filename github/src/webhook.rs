use super::{Event, EventType};
use log::{debug, warn};

/// The GitHub header key used to pass the event type
///
/// Github API docs: https://developer.github.com/webhooks/#delivery-headers
pub const EVENT_TYPE_HEADER: &str = "X-Github-Event";

/// The GitHub header key used to pass the unique ID for the webhook event
///
/// Github API docs: https://developer.github.com/webhooks/#delivery-headers
pub const DELIVERY_ID_HEADER: &str = "X-Github-Delivery";

/// The GitHub header key used to pass the HMAC hexdigest
///
/// Github API docs: https://developer.github.com/webhooks/#delivery-headers
pub const SIGNATURE_HEADER: &str = "X-Hub-Signature";

#[derive(Clone, Debug)]
pub struct Webhook {
    pub event_type: EventType,
    pub delivery_id: String,
    pub signature: Option<String>,
    pub body: Vec<u8>,
}

impl Webhook {
    pub fn check_signature(&self, key: Option<&[u8]>) -> bool {
        match (key, &self.signature) {
            (Some(key), Some(signature)) if signature.starts_with("sha1=") => {
                let hash = hex::encode(hmacsha1::hmac_sha1(key, &self.body));
                let signature = &signature["sha1=".len()..];

                debug!("hash: {}", hash);
                debug!("sig:  {}", signature);
                hash == signature
            }
            // We are expecting a signature and we either recieved it in a different format than
            // expected or no signature was sent.
            (Some(_), _) => false,
            // No key or signature to check
            (None, _) => {
                warn!("No secret specified; signature ignored");
                true
            }
        }
    }

    pub fn to_event(&self) -> Result<Event, serde_json::Error> {
        Event::from_json(self.event_type, &self.body)
    }
}
