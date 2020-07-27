mod installation;
mod smee_client;

#[cfg(test)]
mod test;

pub use self::installation::Installation;
use self::smee_client::SmeeClient;

use crate::{Error, Result};
use anyhow::anyhow;
use futures::{
    future::{self, FutureExt, TryFutureExt},
    try_join,
};
use github::{EventType, Webhook, DELIVERY_ID_HEADER, EVENT_TYPE_HEADER, SIGNATURE_HEADER};
use hyper::{
    body,
    header::{HeaderValue, CONTENT_LENGTH, CONTENT_TYPE},
    server::conn::AddrStream,
    service::{make_service_fn, service_fn},
    Body, Method, Request, Response, Server as HyperServer, StatusCode,
};
use log::{error, info, trace, warn};
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
    installations: Vec<Installation>,
}

impl ServerBuilder {
    pub fn smee(&mut self, smee_url: Option<String>) -> &mut Self {
        self.smee = true;
        self.smee_url = smee_url;
        self
    }

    pub fn add_installation(&mut self, installation: Installation) -> &mut Self {
        self.installations.push(installation);
        self
    }

    pub async fn serve(self, addr: SocketAddr) -> Result<()> {
        // Construct the server
        let server = Server::new(self.installations);

        // The closure inside `make_service_fn` is run for each connection,
        // creating a 'service' to handle requests for that specific connection.
        let make_service = make_service_fn(|_socket: &AddrStream| {
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
            let smee_handle =
                tokio::spawn(smee_client.start()).map(|join_result| join_result.unwrap());
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
    /// Installations which contain various services
    installations: Arc<Vec<Installation>>,
}

impl Server {
    pub fn builder() -> ServerBuilder {
        ServerBuilder::default()
    }

    fn new(installations: Vec<Installation>) -> Self {
        Self {
            counter: Arc::new(AtomicUsize::new(0)),
            installations: Arc::new(installations),
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

    // fn have_route_to_service(&self, path: &str) -> bool {
    //     for installation in self.installations.iter() {
    //         let route = format!(
    //             "/{service}/{owner}/{repo}/",
    //             service = installation.services()[0].name(),
    //             owner = installation.owner(),
    //             repo = installation.name()
    //         );

    //         if path == &route[..route.len() - 1] || path.starts_with(&route) {
    //             return true;
    //         }
    //     }

    //     false
    // }

    // async fn route_to_service(&mut self, request: Request<Body>) -> Option<Result<Response<Body>>> {
    //     for installation in self.installations.iter() {
    //         let route = format!(
    //             "/{service}/{owner}/{repo}/",
    //             service = installation.services()[0].name(),
    //             owner = installation.owner(),
    //             repo = installation.name()
    //         );

    //         if path == &route[..route.len() - 1] || path.starts_with(&route) {
    //             return Some(installation.services()[0].http_route(request)
    //             return true;
    //         }
    //     }

    //     None
    // }

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

        self.handle_webhook(webhook).await?;

        Ok(Response::builder()
            .status(StatusCode::OK)
            .header(CONTENT_TYPE, "text/plain")
            .header(CONTENT_LENGTH, 2)
            .body(Body::from("OK"))?)
    }

    //TODO maybe insert into database here
    pub(super) async fn handle_webhook(&mut self, webhook: Webhook) -> Result<()> {
        trace!("Handling Webhook: {}", webhook.delivery_id);

        // Convert the webhook to an event so that we can get out the installation information
        let event = match webhook.to_event() {
            Ok(webhook) => webhook,
            Err(_err) => {
                let pretty_json = serde_json::to_string_pretty(
                    &serde_json::from_slice::<serde_json::Value>(&webhook.body).unwrap(),
                )
                .unwrap();
                let error = format!(
                    "Webhook could not be Deserialized\n\nEventType {:#?}\n\nError: {:#?}",
                    webhook.event_type,
                    github::Event::from_json(webhook.event_type, pretty_json.as_bytes())
                        .unwrap_err()
                );
                warn!("{}", error);
                let json_path = format!("{}.json", webhook.delivery_id);
                let error_path = format!("{}.err", webhook.delivery_id);
                std::fs::write(json_path, pretty_json.as_bytes()).unwrap();
                std::fs::write(error_path, error.as_bytes()).unwrap();
                return Ok(());
            }
        };

        // XXX Right now we only handle Webhook installations for Repositories
        if let Some(installation) = event.repository().and_then(|repository| {
            self.installations
                .iter()
                .find(|i| i.owner() == repository.owner.login && i.name() == repository.name)
        }) {
            if !webhook.check_signature(installation.secret().map(str::as_bytes)) {
                warn!("Signature check FAILED! Skipping Event.");
                return Ok(());
            }

            installation
                .handle_webhook(&event, &webhook.delivery_id)
                .await;
        }

        Ok(())
    }
}

async fn webhook_from_request(request: Request<Body>) -> Result<Webhook> {
    // Webhooks from github should only contain json payloads
    match request.headers().get(CONTENT_TYPE).map(HeaderValue::to_str) {
        Some(Ok("application/json")) => {}
        _ => return Err(anyhow!("unknown content type")),
    }

    let event_type = match request
        .headers()
        .get(EVENT_TYPE_HEADER)
        .and_then(|h| HeaderValue::to_str(h).ok())
        .and_then(|s| s.parse::<EventType>().ok())
    {
        Some(event) => event,
        _ => return Err(anyhow!("missing valid X-GitHub-Event header")),
    };

    let delivery_id = match request
        .headers()
        .get(DELIVERY_ID_HEADER)
        .and_then(|h| HeaderValue::to_str(h).ok())
    {
        Some(guid) => guid.to_owned(),
        _ => return Err(anyhow!("missing valid X-GitHub-Delivery header")),
    };

    let signature = match request
        .headers()
        .get(SIGNATURE_HEADER)
        .and_then(|h| HeaderValue::to_str(h).ok())
    {
        Some(signature) => Some(signature.to_owned()),
        _ => None,
    };

    let body = body::to_bytes(request.into_body()).await?.to_vec();

    Ok(Webhook {
        event_type,
        delivery_id,
        signature,
        body,
    })
}
