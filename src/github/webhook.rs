use super::{Event, EventType};
use bytes::Bytes;
use log::{debug, warn};

#[derive(Clone, Debug)]
pub struct Webhook {
    pub event: Event,
    pub guid: String,
}

#[derive(Clone, Debug)]
pub(crate) struct RawWebhook {
    pub event_type: EventType,
    pub guid: String,
    pub signature: Option<String>,
    pub body: Bytes,
}

impl RawWebhook {
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

    pub fn to_webhook(&self) -> Result<Webhook, serde_json::Error> {
        let event = Event::from_json(&self.event_type, &self.body)?;
        Ok(Webhook {
            event,
            guid: self.guid.clone(),
        })
    }
}
