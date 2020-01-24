use crate::github::Webhook;
use futures::{channel::mpsc, sink::SinkExt, stream::StreamExt};
use log::info;

#[derive(Debug)]
#[allow(clippy::large_enum_variant)]
pub enum Request {
    Empty,
    Webhook(Webhook),
}

#[derive(Clone, Debug)]
pub struct EventProcessorSender {
    inner: mpsc::Sender<Request>,
}

impl EventProcessorSender {
    pub fn new(inner: mpsc::Sender<Request>) -> Self {
        Self { inner }
    }

    //TODO don't unwrap this but rather return an Error
    pub async fn webhook(&mut self, webhook: Webhook) {
        self.inner.send(Request::Webhook(webhook)).await.unwrap()
    }
}

#[derive(Debug)]
pub struct EventProcessor {
    requests_rx: mpsc::Receiver<Request>,
}

impl EventProcessor {
    pub fn new() -> (EventProcessorSender, Self) {
        let (tx, rx) = mpsc::channel(1024);
        (EventProcessorSender::new(tx), Self { requests_rx: rx })
    }

    pub async fn start(mut self) {
        while let Some(request) = self.requests_rx.next().await {
            self.handle_request(request);
        }
    }

    fn handle_request(&self, request: Request) {
        use Request::*;
        match request {
            Webhook(webhook) => self.handle_webhook(webhook),
            Empty => {}
        }
    }

    fn handle_webhook(&self, webhook: Webhook) {
        info!("Handling Webhook: {}", webhook.guid);
    }
}
