use crate::{command::Command, graphql::GithubClient, Config};
use futures::{channel::mpsc, lock::Mutex, sink::SinkExt, stream::StreamExt};
use github::{Event, EventType};
use hotpot_db::HotPot;
use log::info;

#[derive(Debug)]
#[allow(clippy::large_enum_variant)]
pub enum Request {
    Webhook { event: Event, delivery_id: String },
}

#[derive(Clone, Debug)]
pub struct EventProcessorSender {
    inner: mpsc::Sender<Request>,
}

impl EventProcessorSender {
    pub fn new(inner: mpsc::Sender<Request>) -> Self {
        Self { inner }
    }

    pub async fn webhook(
        &mut self,
        event: Event,
        delivery_id: String,
    ) -> Result<(), mpsc::SendError> {
        self.inner
            .send(Request::Webhook { event, delivery_id })
            .await
    }
}

#[async_trait::async_trait]
impl probot::Service for EventProcessorSender {
    fn name(&self) -> &'static str {
        "bors"
    }

    fn route(&self, _event_type: EventType) -> bool {
        true
    }

    async fn handle(&self, event: &Event, delivery_id: &str) {
        self.clone()
            .webhook(event.clone(), delivery_id.to_owned())
            .await
            .unwrap();
    }
}

#[derive(Debug)]
pub struct EventProcessor {
    config: Config,
    github: GithubClient,
    db: Mutex<HotPot>,
    requests_rx: mpsc::Receiver<Request>,
}

impl EventProcessor {
    pub fn new(config: Config) -> (EventProcessorSender, Self) {
        let (tx, rx) = mpsc::channel(1024);
        let github = GithubClient::new(&config.github_api_token);

        (
            EventProcessorSender::new(tx),
            Self {
                config,
                github,
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
            Webhook { event, delivery_id } => self.handle_webhook(event, delivery_id).await,
        }
    }

    async fn handle_webhook(&self, event: Event, delivery_id: String) {
        info!("Handling Webhook: {}", delivery_id);

        //TODO route on the request
        match &event {
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
