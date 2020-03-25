use crate::{command::Command, probot, Config};
use futures::{channel::mpsc, lock::Mutex, sink::SinkExt, stream::StreamExt};
use github::{Event, Webhook};
use hotpot_db::HotPot;
use log::info;

#[derive(Debug)]
#[allow(clippy::large_enum_variant)]
pub enum Request {
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

#[async_trait::async_trait]
impl probot::Service for EventProcessorSender {
    fn route(&self, _webhook: &Webhook) -> bool {
        true
    }

    async fn handle(&self, webhook: &Webhook) {
        self.clone().webhook(webhook.clone()).await.unwrap();
    }
}

#[derive(Debug)]
pub struct EventProcessor {
    config: Config,
    db: Mutex<HotPot>,
    requests_rx: mpsc::Receiver<Request>,
}

impl EventProcessor {
    pub fn new(config: Config) -> (EventProcessorSender, Self) {
        let (tx, rx) = mpsc::channel(1024);
        (
            EventProcessorSender::new(tx),
            Self {
                config,
                db: Mutex::new(HotPot::new()),
                requests_rx: rx,
            },
        )
    }

    pub async fn start(mut self) {
        while let Some(request) = self.requests_rx.next().await {
            self.handle_request(request).await
        }
    }

    async fn handle_request(&self, request: Request) {
        use Request::*;
        match request {
            Webhook(webhook) => self.handle_webhook(webhook).await,
        }
    }

    async fn handle_webhook(&self, webhook: Webhook) {
        info!("Handling Webhook: {}", webhook.guid);

        //TODO route on the request
        match &webhook.event {
            Event::PullRequest(_e) => {}
            Event::IssueComment(e) => {
                // Only process commands from newly created comments
                if e.action.is_created() && e.issue.is_pull_request() {
                    self.process_comment(e.comment.body()).await
                }
            }
            Event::PullRequestReview(e) => {
                if e.action.is_submitted() {
                    self.process_comment(e.review.body()).await
                }
            }
            Event::PullRequestReviewComment(e) => {
                if e.action.is_created() {
                    self.process_comment(e.comment.body()).await
                }
            }
            // Unsupported Event
            _ => {}
        }
    }

    async fn process_comment(&self, comment: Option<&str>) {
        info!("comment: {:#?}", comment);
        match comment.and_then(Command::from_comment) {
            Some(Ok(_)) => {
                info!("Valid Command");
            }
            Some(Err(_)) => {
                info!("Invalid Command");
            }
            None => {
                info!("No command in comment");
            }
        }
    }
}
