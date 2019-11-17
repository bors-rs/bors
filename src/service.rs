use crate::Error;
use futures::stream::TryStreamExt;
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

    pub async fn serve(self, request: Request<Body>) -> Result<Response<Body>, Error> {
        self.counter.fetch_add(1, Ordering::AcqRel);
        self.handle_request(request).await
    }

    async fn handle_request(&self, request: Request<Body>) -> Result<Response<Body>, Error> {
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

    async fn handle_webhook(&self, request: Request<Body>) -> Result<Response<Body>, Error> {
        assert_eq!(request.method(), &Method::POST);
        assert_eq!(request.uri().path(), "/github");

        let _b = request.into_body().try_concat().await?;
        unimplemented!();
    }
}
