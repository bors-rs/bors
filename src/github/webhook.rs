use super::{Event, EventType};
use bytes::Bytes;
use log::debug;

#[derive(Debug)]
pub struct Webhook {
    pub event: Event,
    pub event_type: EventType,
    pub guid: String,
    pub signature: Option<String>,
    pub body: Bytes,
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
                debug!("no key to check");
                true
            }
        }
    }
}
