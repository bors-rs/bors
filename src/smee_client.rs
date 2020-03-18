use crate::{
    event_processor::EventProcessorSender,
    github::{Event, EventType, Webhook},
    Result,
};
use bytes::{Buf, Bytes, BytesMut};
use hyper::{body::HttpBody, Body, Client, Request};
use hyper_tls::HttpsConnector;
use log::debug;
use serde::Deserialize;
use serde_json::value::RawValue;
use std::{borrow::Cow, str};

pub struct SmeeClient {
    uri: String,
    event_processor_tx: EventProcessorSender,
}

impl SmeeClient {
    pub fn with_uri<U: Into<String>>(uri: U, event_processor_tx: EventProcessorSender) -> Self {
        SmeeClient {
            uri: uri.into(),
            event_processor_tx,
        }
    }

    //TODO take a closer look at the errors that happen in this call stack to determine which are
    // fatal and which should be handled and ignored
    pub async fn start(mut self) -> Result<()> {
        let connector = HttpsConnector::new();
        let client = Client::builder().build(connector);
        let request = Request::builder()
            .method("GET")
            .uri(&self.uri)
            .header("Accept", "text/event-stream")
            .body(Body::empty())?;
        let mut response = client.request(request).await?;
        debug!("response status = {}", response.status());

        let body = response.body_mut();
        let mut event_parser = SmeeEventParser::from_body(body);
        while let Some(event) = event_parser.next().await? {
            match event {
                SmeeEvent::Ready => debug!("ready!"),
                SmeeEvent::Ping => debug!("ping!"),
                SmeeEvent::Message(webhook) => {
                    debug!("message!");
                    // Send Event to EventProcessor
                    self.event_processor_tx.webhook(webhook).await?;
                }
            }
        }

        Ok(())
    }
}

#[allow(clippy::large_enum_variant)]
enum SmeeEvent {
    Ready,
    Ping,
    Message(Webhook),
}

#[derive(Debug, Deserialize)]
struct SmeeMessage<'a> {
    #[serde(rename = "x-github-event")]
    event_type: EventType,
    #[serde(borrow, rename = "body")]
    event: &'a RawValue,
    #[serde(rename = "x-github-delivery")]
    guid: String,
    #[serde(rename = "x-hub-signature")]
    signature: Option<String>,
}

struct ServerSentEvent<'a> {
    event: Option<&'a str>,
    data: Option<Cow<'a, str>>,
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

    async fn next(&mut self) -> Result<Option<SmeeEvent>> {
        while let Some(raw_event) = self.next_server_sent_event().await? {
            let server_sent_event = Self::parse_server_sent_event(&raw_event)?;

            if let Some(event) = Self::parse_smee_event(server_sent_event)? {
                return Ok(Some(event));
            }
        }

        Ok(None)
    }

    /// Contiune polling data off of the stream, splitting off and returning every time a complete
    /// Event is found
    async fn next_server_sent_event(&mut self) -> Result<Option<BytesMut>> {
        loop {
            if let Some(idx) = self.find_end_of_event() {
                return Ok(Some(self.buffer.split_to(idx)));
            }

            if let Some(data) = self.body.data().await {
                let data = data?;
                self.buffer.extend_from_slice(data.bytes());
            } else {
                return Ok(None);
            }
        }
    }

    fn parse_smee_event(server_sent_event: ServerSentEvent<'_>) -> Result<Option<SmeeEvent>> {
        let event = match (server_sent_event.event, server_sent_event.data) {
            (Some("ready"), Some(_)) => SmeeEvent::Ready,
            (Some("ping"), Some(_)) => SmeeEvent::Ping,
            (None, Some(data)) => {
                let smee_msg: SmeeMessage = serde_json::from_str(&data)?;
                let body = Bytes::copy_from_slice(smee_msg.event.get().as_bytes());
                let event = Event::from_json(&smee_msg.event_type, &body)?;
                let webhook = Webhook {
                    event_type: smee_msg.event_type,
                    event,
                    guid: smee_msg.guid,
                    signature: smee_msg.signature,
                    body,
                };
                SmeeEvent::Message(webhook)
            }
            _ => return Ok(None),
        };

        Ok(Some(event))
    }

    fn parse_server_sent_event(raw_event: &[u8]) -> Result<ServerSentEvent<'_>> {
        let mut event = None;
        let mut data: Option<Cow<'_, str>> = None;

        for line in str::from_utf8(raw_event)?.lines() {
            // skip comments
            if line.starts_with(':') {
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
                        event = Some(value);
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

        Ok(ServerSentEvent { event, data })
    }

    fn find_end_of_event(&self) -> Option<usize> {
        fn find_end_of_subsequence(buffer: &[u8], pattern: &[u8]) -> Option<usize> {
            buffer
                .windows(pattern.len())
                .position(|window| window == pattern)
                .map(|idx| idx + pattern.len())
        }

        find_end_of_subsequence(&self.buffer, b"\n\n")
            .or_else(|| find_end_of_subsequence(&self.buffer, b"\r\r"))
            .or_else(|| find_end_of_subsequence(&self.buffer, b"\r\n\r\n"))
    }
}
