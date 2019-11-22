use futures::{channel::mpsc, stream::StreamExt};
use log::info;

#[derive(Debug)]
pub enum Request {
    Empty,
}

#[derive(Debug)]
pub struct EventProcessor {
    requests_rx: mpsc::Receiver<Request>,
}

impl EventProcessor {
    pub fn new() -> (mpsc::Sender<Request>, Self) {
        let (tx, rx) = mpsc::channel(1024);
        (tx, Self { requests_rx: rx })
    }

    pub async fn start(mut self) {
        while let Some(_request) = self.requests_rx.next().await {
            info!("new request");
        }
    }
}
