use hyper::{Body, Request, Response};
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

    pub async fn serve(self, _request: Request<Body>) -> Result<Response<Body>, hyper::Error> {
        let count = self.counter.fetch_add(1, Ordering::AcqRel);
        let response = Response::new(Body::from(format!("Request #{}\n", count)));
        Ok(response)
    }
}
