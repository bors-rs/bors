use crate::{
    event_processor,
    github::{Event, EventType, Webhook},
    smee_client::SmeeClient,
    Config, Database, Error, Result,
};
use futures::{channel::mpsc, future};
use hyper::{
    body,
    header::{HeaderValue, CONTENT_LENGTH, CONTENT_TYPE},
    server::conn::AddrStream,
    service::{make_service_fn, service_fn},
    Body, Method, Request, Response, Server, StatusCode,
};
use log::{error, info};
use std::sync::{
    atomic::{AtomicUsize, Ordering},
    Arc,
};
use structopt::StructOpt;

#[derive(Clone, Debug)]
pub struct Service {
    counter: Arc<AtomicUsize>,
    event_processor_tx: mpsc::Sender<event_processor::Request>,
}

impl Service {
    pub fn new(event_processor_tx: mpsc::Sender<event_processor::Request>) -> Self {
        Self {
            counter: Arc::new(AtomicUsize::new(0)),
            event_processor_tx,
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

        let webhook = match webhook_from_request(request).await {
            Ok(webhook) => webhook,
            Err(e) => {
                error!("parsing payload: {:#?}", e);
                return Ok(Response::builder()
                    .status(StatusCode::BAD_REQUEST)
                    .body(Body::empty())?);
            }
        };

        info!("{:#?}", webhook.event_type);
        //TODO route on the request
        match webhook.event {
            Event::PullRequest(_event) => {}
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

async fn webhook_from_request(
    request: Request<Body>,
    //secret: Option<String>
) -> Result<Webhook> {
    // Webhooks from github should only contain json payloads
    match request.headers().get(CONTENT_TYPE).map(HeaderValue::to_str) {
        Some(Ok("application/json")) => {}
        _ => return Err("unknown content type".into()),
    }

    let event_type = match request
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

    let guid = match request
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

    let signature = match request
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

    let body = body::to_bytes(request.into_body()).await?;

    let event = Event::from_json(&event_type, &body)?;

    // TODO check Signature
    Ok(Webhook {
        event,
        event_type,
        guid,
        signature,
        body: Some(body),
    })
}

#[derive(StructOpt)]
pub struct ServeOptions {
    #[structopt(long, default_value = "3000")]
    port: u16,

    #[structopt(long)]
    // TODO: this should be mututally exclusive with ip/port options
    /// smee.io URL
    smee: Option<String>,
}

pub async fn run_serve(_config: &Config, _db: &Database, options: &ServeOptions) -> Result<()> {
    match options.smee {
        None => {
            let addr = ([127, 0, 0, 1], options.port).into();

            let (tx, event_processor) = event_processor::EventProcessor::new();
            tokio::spawn(event_processor.start());

            let service = Service::new(tx);

            // The closure inside `make_service_fn` is run for each connection,
            // creating a 'service' to handle requests for that specific connection.
            let make_service = make_service_fn(|socket: &AddrStream| {
                info!("remote address: {:?}", socket.remote_addr());

                // While the state was moved into the make_service closure,
                // we need to clone it here because this closure is called
                // once for every connection.
                let service = service.clone();

                // This is the `Service` that will handle the connection.
                future::ok::<_, Error>(service_fn(move |request| {
                    let service = service.clone();
                    service.serve(request)
                }))
            });

            let server = Server::bind(&addr).serve(make_service);

            info!("Listening on http://{}", addr);

            server.await?;

            Ok(())
        }
        Some(ref smee_uri) => {
            let client = SmeeClient::with_uri(smee_uri);
            //tokio::spawn(client.start());
            client.start().await;

            Ok(())
        }
    }
}

#[cfg(test)]
mod test {
    use super::Service;
    use futures::channel::mpsc;
    use hyper::{Body, Method, Request, StatusCode, Uri, Version};

    #[tokio::test]
    async fn pull_request_event() {
        static PAYLOAD: &str = include_str!("test-input/pull-request-event-payload");
        let request = request_from_raw_http(PAYLOAD);
        let (tx, _rx) = mpsc::channel(0);

        let service = Service::new(tx);

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

        let mut request_builder = Request::builder().method(method).uri(uri).version(version);
        for (key, value) in headers {
            request_builder = request_builder.header(key, value);
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
