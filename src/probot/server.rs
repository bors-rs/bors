use crate::{
    github::{Event, EventType, Webhook},
    probot::{service::Service, smee_client::SmeeClient},
    Error, Result,
};
use futures::{
    future::{self, FutureExt, TryFutureExt},
    try_join,
};
use hyper::{
    body,
    header::{HeaderValue, CONTENT_LENGTH, CONTENT_TYPE},
    server::conn::AddrStream,
    service::{make_service_fn, service_fn},
    Body, Method, Request, Response, Server as HyperServer, StatusCode,
};
use log::{error, info};
use std::{
    net::SocketAddr,
    sync::{
        atomic::{AtomicUsize, Ordering},
        Arc,
    },
};

#[derive(Default, Debug)]
pub struct ServerBuilder {
    smee: bool,
    /// smee.io URL
    smee_url: Option<String>,
    services: Vec<Box<dyn Service>>,
}

impl ServerBuilder {
    pub fn smee(mut self, smee_url: Option<String>) -> Self {
        self.smee = true;
        self.smee_url = smee_url;
        self
    }

    pub fn add_service(mut self, service: Box<dyn Service>) -> Self {
        self.services.push(service);
        self
    }

    pub async fn serve(self, addr: SocketAddr) -> Result<()> {
        // Construct the server
        let server = Server::new(self.services);

        // The closure inside `make_service_fn` is run for each connection,
        // creating a 'service' to handle requests for that specific connection.
        let make_service = make_service_fn(|socket: &AddrStream| {
            info!("remote address: {:?}", socket.remote_addr());

            // While the state was moved into the make_service closure,
            // we need to clone it here because this closure is called
            // once for every connection.
            let server = server.clone();

            // This is the `Service` that will handle the connection.
            future::ok::<_, Error>(service_fn(move |request| {
                let server = server.clone();
                server.serve(request)
            }))
        });

        info!("Listening on http://{}", addr);
        let hyper_server = HyperServer::bind(&addr)
            .serve(make_service)
            .map_err(Error::from);

        // spawn the smee client
        if let Some(smee_uri) = self.smee_url {
            let smee_client = SmeeClient::with_uri(smee_uri, server.clone());
            //let smee_handle = tokio::spawn(smee_client.start()).map_err(Error::from);
            let smee_handle = tokio::spawn(smee_client.start()).map(|join_result| {
                let res = join_result?;
                res
            });
            try_join!(hyper_server, smee_handle)?;
        } else {
            hyper_server.await?;
        }

        Ok(())
    }
}

#[derive(Clone, Debug)]
pub struct Server {
    counter: Arc<AtomicUsize>,
    services: Arc<Vec<Box<dyn Service>>>,
}

impl Server {
    pub fn builder() -> ServerBuilder {
        ServerBuilder::default()
    }

    fn new(services: Vec<Box<dyn Service>>) -> Self {
        Self {
            counter: Arc::new(AtomicUsize::new(0)),
            services: Arc::new(services),
        }
    }

    async fn serve(mut self, request: Request<Body>) -> Result<Response<Body>> {
        self.counter.fetch_add(1, Ordering::AcqRel);
        self.route_http_request(request).await
    }

    async fn route_http_request(&mut self, request: Request<Body>) -> Result<Response<Body>> {
        match (request.method(), request.uri().path()) {
            (&Method::GET, "/") => {
                let count = self.counter.load(Ordering::Relaxed);
                let response = Response::new(Body::from(format!("Request #{}\n", count)));
                Ok(response)
            }
            (&Method::GET, "/github") => Ok(Response::builder()
                .status(StatusCode::METHOD_NOT_ALLOWED)
                .body(Body::empty())?),
            (&Method::POST, "/github") => self.route_github(request).await,
            _ => Ok(Response::builder()
                .status(StatusCode::NOT_FOUND)
                .body(Body::empty())?),
        }
    }

    async fn route_github(&mut self, request: Request<Body>) -> Result<Response<Body>> {
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
        self.handle_webhook(webhook).await?;

        Ok(Response::builder()
            .status(StatusCode::OK)
            .header(CONTENT_TYPE, "text/plain")
            .header(CONTENT_LENGTH, 2)
            .body(Body::from("OK"))?)
    }

    pub(super) async fn handle_webhook(&mut self, webhook: Webhook) -> Result<()> {
        //TODO maybe check signatures and store webhook in database here
        for service in self.services.iter() {
            if service.route(&webhook) {
                service.handle(&webhook).await;
            }
        }
        Ok(())
    }
}

async fn webhook_from_request(request: Request<Body>) -> Result<Webhook> {
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
        Some(signature) => Some(signature.to_owned()),
        _ => None,
    };

    let body = body::to_bytes(request.into_body()).await?;
    let event = Event::from_json(&event_type, &body)?;

    Ok(Webhook {
        event,
        event_type,
        guid,
        signature,
        body,
    })
}

#[cfg(test)]
mod test {
    use super::Server;
    use hyper::{Body, Method, Request, StatusCode, Uri, Version};

    #[tokio::test]
    async fn pull_request_event() {
        static PAYLOAD: &str = include_str!("../test-input/pull-request-event-payload");
        let request = request_from_raw_http(PAYLOAD);

        let mut service = Server::new(vec![]);

        let resp = service.route_github(request).await.unwrap();
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
