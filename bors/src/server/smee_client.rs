use super::Server;
use crate::Result;
use bytes::{Buf, BytesMut};
use github::{EventType, Webhook};
use log::{debug, info, trace, warn};
use reqwest::{Client, Response};
use serde::Deserialize;
use serde_json::value::RawValue;
use std::{borrow::Cow, str};

pub struct SmeeClient {
    uri: String,
    server: Server,
}

impl SmeeClient {
    //TODO handle `http://smee.io/new` uri
    pub fn with_uri<U: Into<String>>(uri: U, server: Server) -> Self {
        SmeeClient {
            uri: uri.into(),
            server,
        }
    }

    //TODO take a closer look at the errors that happen in this call stack to determine which are
    // fatal and which should be handled and ignored
    pub async fn start(mut self) -> Result<()> {
        // If there are any errors with the stream, log and restart the client
        while let Err(e) = self.run().await {
            warn!("Smee Error: {:?}", e);
            tokio::time::sleep(std::time::Duration::from_secs(10)).await;
        }

        Ok(())
    }

    async fn run(&mut self) -> Result<()> {
        info!("Starting SmeeClient with {}", self.uri);

        let client = Client::new();
        let mut response = client
            .get(&self.uri)
            .header("Accept", "text/event-stream")
            .body("")
            .send()
            .await?;
        debug!("response status = {}", response.status());

        let mut event_parser = SmeeEventParser::from_body(&mut response);
        while let Some(event) = event_parser.next().await? {
            match event {
                SmeeEvent::Ready => trace!("ready!"),
                SmeeEvent::Ping => trace!("ping!"),
                SmeeEvent::Message(webhook) => {
                    trace!("message!");
                    // Have the server process the webhook
                    self.server.handle_webhook(webhook).await?;
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
    delivery_id: String,
    #[serde(rename = "x-hub-signature")]
    signature: Option<String>,
    #[serde(rename = "x-hub-signature-256")]
    signature_256: Option<String>,
}

struct ServerSentEvent<'a> {
    event: Option<&'a str>,
    data: Option<Cow<'a, str>>,
}

struct SmeeEventParser<'b> {
    body: &'b mut Response,
    buffer: BytesMut,
}

impl<'b> SmeeEventParser<'b> {
    fn from_body(body: &'b mut Response) -> Self {
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

            if let Some(data) = self.body.chunk().await? {
                //let data = data?;
                self.buffer.extend_from_slice(data.chunk());
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
                let webhook = Webhook {
                    event_type: smee_msg.event_type,
                    delivery_id: smee_msg.delivery_id,
                    signature: smee_msg.signature,
                    signature_256: smee_msg.signature_256,
                    body: smee_msg.event.get().as_bytes().to_owned(),
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
