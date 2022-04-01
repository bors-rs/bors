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
pub const SIGNATURE_256_HEADER: &str = "X-Hub-Signature-256";

#[derive(Clone, Debug)]
pub struct Webhook {
    pub event_type: EventType,
    pub delivery_id: String,
    pub signature: Option<String>,
    pub signature_256: Option<String>,
    pub body: Vec<u8>,
}

impl Webhook {
    pub fn check_signature(&self, key: Option<&[u8]>) -> bool {
        // Skip validation if no key
        // FIXME: Refactor code so it's required
        if key.is_none() {
            warn!(
                "[webhook {}] No secret specified; signature ignored",
                self.delivery_id
            );
            return false;
        }

        if let Some(ref signature) = self.signature_256 {
            let hash = hex::encode(hmac_sha256::HMAC::mac(&self.body, key.unwrap()));
            let signature = &signature["sha256=".len()..];

            debug!(
                "[webhook {}] SHA-256 Found: {} Expected: {}",
                self.delivery_id, signature, hash
            );
            signature == hash
        } else {
            // There is no signature, reject it
            warn!("[webhook {}] No signature present", self.delivery_id);
            false
        }
    }

    pub fn to_event(&self) -> Result<Event, std::io::Error> {
        Event::from_json(self.event_type, &self.body)
    }
}
