use crate::github::EventType;
use crate::Result;
use bytes::Bytes;
use futures::stream::TryStreamExt;
use hyper::header::HeaderValue;
use hyper::header::{CONTENT_LENGTH, CONTENT_TYPE};
use hyper::{Body, Method, Request, Response, StatusCode};
use std::sync::{
    atomic::{AtomicUsize, Ordering},
    Arc,
};

#[derive(Clone, Debug)]
pub struct Service {
    counter: Arc<AtomicUsize>,
}

impl Service {
    pub fn new() -> Self {
        Self {
            counter: Arc::new(AtomicUsize::new(0)),
        }
    }

    pub async fn serve(self, request: Request<Body>) -> Result<Response<Body>> {
        self.counter.fetch_add(1, Ordering::AcqRel);
        self.handle_request(request).await
    }

    async fn handle_request(&self, request: Request<Body>) -> Result<Response<Body>> {
        match (request.method(), request.uri().path()) {
            (&Method::GET, "/") => {
                let count = self.counter.load(Ordering::Relaxed);
                let response = Response::new(Body::from(format!("Request #{}\n", count)));
                Ok(response)
            }
            (&Method::GET, "/github") => Ok(Response::builder()
                .status(StatusCode::METHOD_NOT_ALLOWED)
                .body(Body::empty())?),
            (&Method::POST, "/github") => self.handle_webhook(request).await,
            _ => Ok(Response::builder()
                .status(StatusCode::NOT_FOUND)
                .body(Body::empty())?),
        }
    }

    async fn handle_webhook(&self, request: Request<Body>) -> Result<Response<Body>> {
        assert_eq!(request.method(), &Method::POST);
        assert_eq!(request.uri().path(), "/github");

        let webhook = match GithubWebhook::from_request(request).await {
            Ok(webhook) => webhook,
            Err(_) => {
                return Ok(Response::builder()
                    .status(StatusCode::BAD_REQUEST)
                    .body(Body::empty())?)
            }
        };

        //TODO route on the request
        match webhook.event {
            //TODO don't unwrap
            EventType::PullRequest => {
                let _pr_event: crate::github::PullRequestEvent =
                    serde_json::from_slice(&webhook.body).unwrap();
            }
            // Unsupported Event
            _ => {}
        }

        Ok(Response::builder()
            .status(StatusCode::OK)
            .header(CONTENT_TYPE, "text/plain")
            .header(CONTENT_LENGTH, 2)
            .body(Body::from("OK"))?)
    }
}

struct GithubWebhook {
    event: EventType,
    _guid: String,
    _signature: Option<String>,
    body: Bytes,
}

impl GithubWebhook {
    async fn from_request(request: Request<Body>, /*, secret: Option<String>*/) -> Result<Self> {
        // Webhooks from github should only contain json payloads
        match request.headers().get(CONTENT_TYPE).map(HeaderValue::to_str) {
            Some(Ok("application/json")) => {}
            _ => return Err("unknown content type".into()),
        }

        let event = match request
            .headers()
            .get("X-GitHub-Event")
            .map(HeaderValue::to_str)
            .transpose()
            .ok()
            .and_then(std::convert::identity)
            .map(str::parse::<EventType>)
            .transpose()
            .ok()
            .and_then(std::convert::identity)
        {
            Some(event) => event,
            _ => return Err("missing valid X-GitHub-Event header".into()),
        };

        let _guid = match request
            .headers()
            .get("X-GitHub-Delivery")
            .map(HeaderValue::to_str)
            .transpose()
            .ok()
            .and_then(std::convert::identity)
        {
            Some(guid) => guid.to_owned(),
            _ => return Err("missing valid X-GitHub-Delivery header".into()),
        };

        let _signature = match request
            .headers()
            .get("X-Hub-Signature")
            .map(HeaderValue::to_str)
            .transpose()
            .ok()
            .and_then(std::convert::identity)
        {
            Some(guid) if guid.starts_with("sha1=") => Some(guid["sha1=".len()..].to_owned()),
            _ => {
                None
                // TODO return an error if we're expecting a sig
                // return Err("missing valid X-Hub-Signature header".into());
            }
        };

        let body = request.into_body().try_concat().await?.into_bytes();

        // TODO check Signature
        Ok(Self {
            event,
            _guid,
            _signature,
            body,
        })
    }
}

#[cfg(test)]
mod test {
    use super::Service;
    use hyper::{Body, Method, Request, StatusCode, Uri, Version};

    #[tokio::test]
    async fn pull_request_event() {
        static PAYLOAD: &str = include_str!("test-input/pull-request-event-payload");
        let request = request_from_raw_http(PAYLOAD);

        let service = Service::new();

        let resp = service.handle_webhook(request).await.unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
        println!("{:?}", resp);
    }

    // Super quick and dirty parsing of raw http into a `Request<Body>` type.
    // This assumes that the content is JSON
    fn request_from_raw_http(raw: &'static str) -> Request<Body> {
        let (headers, payload) = raw.split_at(raw.find("{\n").unwrap());

        let mut headers = headers.lines();
        let (method, uri, version) = parse_method_line(headers.next().unwrap());
        let headers = headers.map(|line| {
            let (key, value) = line.split_at(line.find(":").unwrap());
            // remove the ':' from the value
            (key.trim(), value[1..].trim())
        });

        let mut request_builder = Request::builder();
        request_builder.method(method).uri(uri).version(version);
        for (key, value) in headers {
            request_builder.header(key, value);
        }

        request_builder.body(Body::from(payload)).unwrap()
    }

    fn parse_method_line(line: &str) -> (Method, Uri, Version) {
        let mut iter = line.split_whitespace();

        let method = iter.next().unwrap().parse().unwrap();

        let uri = iter.next().unwrap().parse().unwrap();

        let version = match iter.next().unwrap() {
            "HTTP/1.1" => Version::HTTP_11,
            _ => panic!("unknown version"),
        };

        (method, uri, version)
    }
}
