mod installation;
mod smee_client;

#[cfg(test)]
mod test;

pub use self::{installation::Installation, smee_client::SmeeClient};

use crate::{config::GithubConfig, Error, Result};
use anyhow::anyhow;
use futures::future::{self, TryFutureExt};
use github::{
    EventType, Webhook, DELIVERY_ID_HEADER, EVENT_TYPE_HEADER, SIGNATURE_256_HEADER,
    SIGNATURE_HEADER,
};
use hyper::{
    body,
    header::{HeaderValue, CONTENT_LENGTH, CONTENT_TYPE},
    server::conn::AddrStream,
    service::{make_service_fn, service_fn},
    Body, Method, Request, Response, Server as HyperServer, StatusCode,
};
use log::{debug, error, info, trace, warn};
use lru::LruCache;
use std::{
    net::SocketAddr,
    sync::{
        atomic::{AtomicUsize, Ordering},
        Arc,
    },
};
use tokio::sync::{Mutex, RwLock};

const INDEX_HTML: &str = include_str!("../../html/index.html");
const REPO_HTML: &str = include_str!("../../html/repo.html");

#[derive(Clone, Debug)]
pub struct Server {
    config: GithubConfig,
    counter: Arc<AtomicUsize>,
    /// Installations which contain various services
    installations: Arc<RwLock<Vec<Installation>>>,
    lru_webhooks: Arc<Mutex<LruCache<String, ()>>>,
}

impl Server {
    pub fn new(config: GithubConfig) -> Self {
        Self {
            config,
            counter: Arc::new(AtomicUsize::new(0)),
            installations: Arc::new(RwLock::new(Vec::new())),
            lru_webhooks: Arc::new(Mutex::new(LruCache::new(10000))),
        }
    }

    pub async fn add_installation(&mut self, installation: Installation) {
        self.installations.write().await.push(installation);
    }

    pub async fn start(self, addr: SocketAddr) -> Result<()> {
        // The closure inside `make_service_fn` is run for each connection,
        // creating a 'service' to handle requests for that specific connection.
        let make_service = make_service_fn(|_socket: &AddrStream| {
            // While the state was moved into the make_service closure,
            // we need to clone it here because this closure is called
            // once for every connection.
            let server = self.clone();

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

        hyper_server.await?;

        Ok(())
    }

    async fn serve(mut self, request: Request<Body>) -> Result<Response<Body>> {
        self.counter.fetch_add(1, Ordering::AcqRel);
        self.route_http_request(request).await
    }

    async fn route_http_request(&mut self, request: Request<Body>) -> Result<Response<Body>> {
        match (request.method(), request.uri().path()) {
            (&Method::GET, "/") => {
                let count = self.counter.load(Ordering::Relaxed);

                let template = liquid::ParserBuilder::with_stdlib()
                    .build()
                    .unwrap()
                    .parse(INDEX_HTML)
                    .unwrap();

                let repos = self
                    .installations
                    .read()
                    .await
                    .iter()
                    .map(|i| i.config().repo().to_owned())
                    .collect::<Vec<_>>();
                let data = liquid::object!({
                    "request_count": count,
                    "repos": repos,
                });
                let output = template.render(&data).unwrap();

                let response = Response::new(Body::from(output));
                Ok(response)
            }
            (&Method::GET, "/github") => Ok(Response::builder()
                .status(StatusCode::METHOD_NOT_ALLOWED)
                .body(Body::empty())?),
            (&Method::POST, "/github") => self.route_github(request).await,
            (_, route) if route == "/repos" || route.starts_with("/repos/") => {
                self.route_repos(request).await
            }
            _ => Ok(Response::builder()
                .status(StatusCode::NOT_FOUND)
                .body(Body::empty())?),
        }
    }

    // XXX Really rough code for dumping internal state
    async fn route_repos(&mut self, request: Request<Body>) -> Result<Response<Body>> {
        let path = request.uri().path();

        if path == "/repos" || path == "/repos/" {
            let mut body = String::new();
            body.push_str("Repositories:\n\n");

            for installation in self.installations.read().await.iter() {
                body.push_str(&format!(
                    "{}/{}\n",
                    installation.owner(),
                    installation.name()
                ));
            }

            return Ok(Response::new(Body::from(body)));
        }

        for installation in self.installations.read().await.iter() {
            let route = format!(
                "/repos/{owner}/{repo}/",
                owner = installation.owner(),
                repo = installation.name()
            );

            if path == &route[..route.len() - 1] || path == route {
                let template = liquid::ParserBuilder::with_stdlib()
                    .build()
                    .unwrap()
                    .parse(REPO_HTML)
                    .unwrap();

                let body = template
                    .render(&installation.repo_liquid_object().await)
                    .unwrap();

                return Ok(Response::new(Body::from(body)));
            } else if path.starts_with(&route) && path.ends_with("/debug") {
                let body = format!(
                    "{}/{}\n\nConfig:\n{:#?}\n\nState:\n{:#?}",
                    installation.owner(),
                    installation.name(),
                    installation.config(),
                    installation.state().await,
                );

                return Ok(Response::new(Body::from(body)));
            } else if path.starts_with(&route) && path.ends_with("/sync") {
                installation.sync().await;
                return Ok(Response::new(Body::from("Syncing Pull Requests!")));
            }
        }

        Ok(Response::builder()
            .status(StatusCode::NOT_FOUND)
            .body(Body::empty())?)
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

        self.handle_webhook(webhook).await?;

        Ok(Response::builder()
            .status(StatusCode::OK)
            .header(CONTENT_TYPE, "text/plain")
            .header(CONTENT_LENGTH, 2)
            .body(Body::from("OK"))?)
    }

    /// Handles an incoming webhook request.  Drops duplicate requests
    pub(super) async fn handle_webhook(&mut self, webhook: Webhook) -> Result<()> {
        // If we've recently seen this webhook, let's drop it to prevent duplicates
        if let Some(()) = self
            .lru_webhooks
            .lock()
            .await
            .put(webhook.delivery_id.clone(), ())
        {
            debug!(
                "Duplicate webhook: {}.  Dropping it...",
                webhook.delivery_id
            );
            return Ok(());
        }

        // Process the current webhook
        trace!("Handling Webhook: {}", webhook.delivery_id);
        if !webhook.check_signature(self.config.webhook_secret().map(str::as_bytes)) {
            warn!(
                "Signature check FAILED! Skipping Event. [{:?},{}]",
                webhook.event_type, webhook.delivery_id
            );
            return Ok(());
        }

        // Convert the webhook to an event so that we can get out the installation information
        let event = match webhook.to_event() {
            Ok(webhook) => webhook,
            Err(_err) => {
                let pretty_json = serde_json::to_string_pretty(
                    &serde_json::from_slice::<serde_json::Value>(&webhook.body).unwrap(),
                )
                .unwrap();
                error!(
                    "Webhook could not be Deserialized\n\nEventType {:#?}\n\nError: {:#?}\n\nPayload: {:#?}",
                    webhook.event_type,
                    github::Event::from_json(webhook.event_type, pretty_json.as_bytes())
                        .unwrap_err(),
                    pretty_json,
                );
                return Ok(());
            }
        };

        // XXX Right now we only handle Webhook installations for Repositories
        let installations = self.installations.read().await;
        if let Some(installation) = event.repository().and_then(|repository| {
            installations
                .iter()
                .find(|i| i.owner() == repository.owner.login && i.name() == repository.name)
        }) {
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

    let signature_256 = match request
        .headers()
        .get(SIGNATURE_256_HEADER)
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
        signature_256,
        body,
    })
}
