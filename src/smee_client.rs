use crate::github::{Event, EventType, Webhook};
use bytes::{Buf, BytesMut};
use hyper::{body::HttpBody, Body, Client, Request};
use hyper_tls::HttpsConnector;
use log::{debug, info};
use serde::Deserialize;
use serde_json::value::RawValue;
use std::borrow::Cow;
use std::str;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum SmeeError {
    #[error("http error")]
    Http(#[from] hyper::http::Error),
    #[error("hyper error")]
    Hyper(#[from] hyper::Error),
    #[error("utf8 error")]
    Utf8(#[from] str::Utf8Error),
    #[error("json error")]
    Json(#[from] serde_json::Error),
}

pub struct SmeeClient {
    uri: String,
}

impl SmeeClient {
    pub fn with_uri<U: Into<String>>(uri: U) -> Self {
        SmeeClient { uri: uri.into() }
    }

    pub async fn start(self) {
        let connector = HttpsConnector::new();
        let client = Client::builder().build(connector);
        let request = Request::builder()
            .method("GET")
            .uri(&self.uri)
            .header("Accept", "text/event-stream")
            .body(Body::empty())
            .unwrap();
        let mut response = client.request(request).await.unwrap();
        info!("response status = {}", response.status());

        let body = response.body_mut();
        let mut event_parser = SmeeEventParser::from_body(body);
        while let Some(event) = event_parser.next().await.unwrap() {
            match event {
                SmeeEvent::Ready => info!("ready!"),
                SmeeEvent::Ping => info!("ping!"),
                SmeeEvent::Message(payload) => {
                    info!("message!: {:?}", payload);
                }
            }
        }
    }
}

enum SmeeEvent {
    Ready,
    Ping,
    Message(Webhook),
}

#[derive(Debug, Deserialize)]
pub struct SmeeMessage<'a> {
    #[serde(rename = "x-github-event")]
    event_type: EventType,
    #[serde(borrow, rename = "body")]
    event: &'a RawValue,
    #[serde(rename = "x-github-delivery")]
    guid: String,
    #[serde(rename = "x-hub-signature")]
    signature: Option<String>,
}

struct SmeeEventParser<'b> {
    body: &'b mut Body,
    buffer: BytesMut,
}

impl<'b> SmeeEventParser<'b> {
    fn from_body(body: &'b mut Body) -> Self {
        Self {
            body,
            buffer: BytesMut::new(),
        }
    }

    async fn next(&mut self) -> Result<Option<SmeeEvent>, SmeeError> {
        while let Some(data) = self.body.data().await {
            let data = data?;
            debug!("bytes = {:?}", data.bytes());
            self.buffer.extend_from_slice(data.bytes());
            if !self.contains_end_of_event() {
                debug!("message isn't ended, reading more");
                continue;
            }

            debug!("got whole message");
            if let Some(event) = Self::parse_smee_event(&self.buffer.split())? {
                return Ok(Some(event));
            }
        }

        Ok(None)
    }

    fn parse_smee_event(raw_event: &[u8]) -> Result<Option<SmeeEvent>, SmeeError> {
        let mut smee_event_type = None;
        let mut data: Option<Cow<'_, str>> = None;

        for line in str::from_utf8(raw_event)?.lines() {
            debug!("line = {}", line);

            // dispatch event if blank line
            if line.is_empty() {
                let event = match (smee_event_type.take(), data.take()) {
                    (Some("ready"), Some(_)) => Some(SmeeEvent::Ready),
                    (Some("ping"), Some(_)) => Some(SmeeEvent::Ping),
                    (None, Some(data)) => {
                        let smee_msg: SmeeMessage = serde_json::from_str(&data)?;
                        let event = Event::from_json(
                            &smee_msg.event_type,
                            smee_msg.event.get().as_bytes(),
                        )?;
                        let webhook = Webhook {
                            event_type: smee_msg.event_type,
                            event,
                            guid: smee_msg.guid,
                            signature: smee_msg.signature,
                            body: None,
                        };
                        Some(SmeeEvent::Message(webhook))
                    }
                    _ => continue,
                };

                return Ok(event);
            }

            // skip comments
            if line.starts_with(":") {
                continue;
            }

            let field;
            let mut value = None;

            if let Some(i) = line.find(':') {
                let (f, v) = line.split_at(i);
                field = f;
                if v.starts_with(": ") {
                    value = v.get(2..);
                } else {
                    value = v.get(1..);
                }
            } else {
                field = line;
            }

            match field {
                "event" => {
                    if let Some(value) = value {
                        smee_event_type = Some(value);
                    }
                }

                "data" => {
                    if let Some(value) = value {
                        if let Some(data) = data.as_mut() {
                            data.to_mut().push_str("\n");
                            data.to_mut().push_str(value);
                        } else {
                            data = Some(Cow::Borrowed(value));
                        }
                    }
                }

                // ignore other fields (id, retry, etc)
                _ => {}
            }
        }

        Ok(None)
    }

    fn contains_end_of_event(&self) -> bool {
        let length = self.buffer.len();
        debug!("contains: length = {}", length);
        debug!("eob = {:?}", &self.buffer[length - 4..]);
        (length >= 2 && &self.buffer[length - 2..] == b"\n\n")
            || (length >= 2 && &self.buffer[length - 2..] == b"\r\r")
            || (length >= 4 && &self.buffer[length - 4..] == b"\r\n\r\n")
    }
}
