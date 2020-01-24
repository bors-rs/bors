use crate::github::Webhook;
use crate::Config;
use futures::{channel::mpsc, sink::SinkExt, stream::StreamExt};
use log::{info, warn};

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

    pub async fn webhook(&mut self, webhook: Webhook) -> Result<(), mpsc::SendError> {
        self.inner.send(Request::Webhook(webhook)).await
    }
}

#[derive(Debug)]
pub struct EventProcessor {
    config: Config,
    requests_rx: mpsc::Receiver<Request>,
}

impl EventProcessor {
    pub fn new(config: Config) -> (EventProcessorSender, Self) {
        let (tx, rx) = mpsc::channel(1024);
        (
            EventProcessorSender::new(tx),
            Self {
                config,
                requests_rx: rx,
            },
        )
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

        if webhook.check_signature(self.config.secret()) {
            //TODO Handle event
            info!("Signature check PASSED!");
        } else {
            warn!("Signature check FAILED! Skipping Event.");
        }
    }
}
