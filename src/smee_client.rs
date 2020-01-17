use crate::github::WebhookPayload;
use bytes::{buf::BufExt, Buf, BytesMut};
use hyper::{body::HttpBody, Body, Client, Request};
use hyper_tls::HttpsConnector;
use log::info;
use std::str;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum SmeeError<B: HttpBody + std::fmt::Debug>
where
    B::Error: std::fmt::Debug,
{
    #[error("http error")]
    Http(#[from] hyper::http::Error),
    #[error("hyper error")]
    Hyper(#[from] hyper::Error),
    #[error("body error")]
    Body(B::Error),
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
        let mut event_parser = EventParser::from_body(body);
        while let Some(event) = event_parser.next().await.unwrap() {
            match event {
                Event::Ready => info!("ready!"),
                Event::Ping => info!("ping!"),
                Event::Message(ref _payload) => {
                    info!("message!");
                }
            }
        }
    }
}

enum Event {
    Ready,
    Ping,
    Message(WebhookPayload),
}

struct EventParser<'b, B: HttpBody + Unpin> {
    body: &'b mut B,
    buffer: BytesMut,
    event: Option<String>,
    data: Option<String>,
}

impl<'b, B: HttpBody + Unpin + std::fmt::Debug> EventParser<'b, B>
where
    B::Error: std::fmt::Debug,
{
    fn from_body(body: &'b mut B) -> Self {
        EventParser {
            body,
            buffer: BytesMut::with_capacity(0),
            event: None,
            data: None,
        }
    }

    async fn next(&mut self) -> Result<Option<Event>, SmeeError<B>> {
        while let Some(data) = self.body.data().await {
            let data = data.map_err(SmeeError::Body)?;
            info!("bytes = {:?}", data.bytes());
            self.buffer.extend_from_slice(data.bytes());
            if !self.contains_end_of_event() {
                info!("message isn't ended, reading more");
                continue;
            }

            info!("got whole message");
            let string = str::from_utf8(self.buffer.bytes())?.to_string();
            // clear the buffer
            self.buffer.clear();

            for line in string.lines() {
                info!("line = {}", line);

                // dispatch event if blank line
                if line.is_empty() {
                    // dispatch the event, step 1
                    if self.data.is_none() {
                        self.event = None;
                        self.data = None;
                        continue;
                    }

                    // dispatch the event, steps 3 & 4
                    info!("event = {:?}\ndata = {:?}", self.event, self.data);
                    let event = match self.event.as_ref().map(String::as_str) {
                        Some("ready") => Some(Event::Ready),
                        Some("ping") => Some(Event::Ping),
                        None => {
                            let payload =
                                WebhookPayload::from_json(self.data.as_ref().unwrap().as_bytes())?;
                            Some(Event::Message(payload))
                        }
                        _ => None,
                    };

                    self.event = None;
                    self.data = None;

                    if event.is_some() {
                        return Ok(event);
                    }
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
                            self.event = Some(value.to_string());
                        }
                    }

                    "data" => {
                        if let Some(value) = value {
                            if let Some(ref mut data) = self.data {
                                data.push_str("\n");
                                data.push_str(value);
                            } else {
                                self.data = Some(value.to_string());
                            }
                        }
                    }

                    // ignore other fields (id, retry, etc)
                    _ => {}
                }
            }
        }

        Ok(None)
    }

    fn contains_end_of_event(&self) -> bool {
        let length = self.buffer.len();
        info!("contains: length = {}", length);
        info!("eob = {:?}", &self.buffer[length - 4..]);
        (length >= 2 && &self.buffer[length - 2..] == b"\n\n")
            || (length >= 2 && &self.buffer[length - 2..] == b"\r\r")
            || (length >= 4 && &self.buffer[length - 4..] == b"\r\n\r\n")
    }
}
