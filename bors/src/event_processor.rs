use crate::{
    command::Command, error::Result, graphql::GithubClient, state::PullRequestState, Config,
};
use futures::{channel::mpsc, lock::Mutex, sink::SinkExt, stream::StreamExt};
use github::{Event, EventType, NodeId};
use hotpot_db::HotPot;
use log::info;
use std::collections::HashMap;

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
    pulls: HashMap<u64, PullRequestState>,
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
                pulls: HashMap::new(),
                db: Mutex::new(HotPot::new()),
                requests_rx: rx,
            },
        )
    }

    pub async fn start(mut self) {
        self.synchronize().await;

        while let Some(request) = self.requests_rx.next().await {
            self.handle_request(request).await
        }
    }

    async fn handle_request(&mut self, request: Request) {
        use Request::*;
        match request {
            Webhook { event, delivery_id } => self.handle_webhook(event, delivery_id).await,
        }
    }

    async fn handle_webhook(&mut self, event: Event, delivery_id: String) {
        info!("Handling Webhook: {}", delivery_id);

        //TODO route on the request
        match &event {
            Event::PullRequest(_e) => {}
            Event::IssueComment(e) => {
                // Only process commands from newly created comments
                if e.action.is_created() && e.issue.is_pull_request() {
                    self.process_comment(
                        &e.sender.login,
                        e.issue.number,
                        e.comment.body(),
                        &e.comment.node_id,
                    )
                    .await
                }
            }
            Event::PullRequestReview(e) => {
                if e.action.is_submitted() {
                    self.process_comment(
                        &e.sender.login,
                        e.pull_request.number,
                        e.review.body(),
                        &e.review.node_id,
                    )
                    .await
                }
            }
            Event::PullRequestReviewComment(e) => {
                if e.action.is_created() {
                    self.process_comment(
                        &e.sender.login,
                        e.pull_request.number,
                        e.comment.body(),
                        &e.comment.node_id,
                    )
                    .await
                }
            }
            // Unsupported Event
            _ => {}
        }
    }

    async fn process_comment(
        &mut self,
        user: &str,
        pr_number: u64,
        comment: Option<&str>,
        node_id: &NodeId,
    ) {
        info!("comment: {:#?}", comment);

        match comment.and_then(Command::from_comment) {
            Some(Ok(command)) => {
                info!("Valid Command");

                self.github
                    .add_reaction(node_id, github::ReactionType::Rocket)
                    .await
                    // TODO handle or ignore error
                    .unwrap();

                // Check if the user is authorized before executing the command
                if self.is_authorized(user, pr_number, &command).await.unwrap() {
                    self.execute_command(user, pr_number, command).await;
                }
            }
            Some(Err(_)) => {
                info!("Invalid Command");
                self.github
                    .issues()
                    .create_comment(
                        self.config.repo().owner(),
                        self.config.repo().name(),
                        pr_number,
                        &format!(":exclamation: Invalid command\n\n{}", Command::help()),
                    )
                    .await
                    // TODO handle or ignore error
                    .unwrap();
            }
            None => {
                info!("No command in comment");
            }
        }
    }

    // TODO handle `delegate`
    async fn is_authorized(&self, user: &str, pr_number: u64, command: &Command) -> Result<bool> {
        let mut is_authorized = false;
        let mut reason = None;

        // Check to see if the user is a collaborator
        if self
            .github
            .repos()
            .is_collaborator(self.config.repo().owner(), self.config.repo().name(), user)
            .await?
            .into_inner()
        {
            is_authorized = true;
        } else {
            reason = Some("Not Collaborator");
        }

        // Check to see if the user issuing the command is attempting to approve their own PR
        if let Some(author) = self
            .pulls
            .get(&pr_number)
            .and_then(|state| state.author.as_deref())
        {
            use crate::command::CommandType;
            if !self.config.repo().allow_self_review()
                && is_authorized
                && user == author
                && matches!(command.command_type, CommandType::Approve(_))
            {
                is_authorized = false;
                reason = Some("Can't approve your own PR");
            }
        }

        // Post a comment to Github if there was a reason why the user wasn't authorized
        if !is_authorized {
            if let Some(reason) = reason {
                self.github
                    .issues()
                    .create_comment(
                        self.config.repo().owner(),
                        self.config.repo().name(),
                        pr_number,
                        &format!("@{}: :key: Insufficient privileges: {}", user, reason),
                    )
                    .await?;
            }
        }

        Ok(is_authorized)
    }

    async fn execute_command(&mut self, user: &str, pr_number: u64, command: Command) {
        use crate::command::CommandType;

        match command.command_type {
            CommandType::Approve(a) => {
                if let Some(priority) = a.priority() {
                    self.set_priority(pr_number, priority);
                }

                self.approve_pr(user, pr_number).await.unwrap();
            }
            CommandType::Unapprove => unimplemented!(),
            CommandType::Land(l) => {
                if let Some(priority) = l.priority() {
                    self.set_priority(pr_number, priority);
                }

                unimplemented!();
            }
            CommandType::Retry(r) => {
                if let Some(priority) = r.priority() {
                    self.set_priority(pr_number, priority);
                }
                unimplemented!();
            }
            CommandType::Cancel => unimplemented!(),
            CommandType::Priority(p) => self.set_priority(pr_number, p.priority()),
        }
    }

    fn set_priority(&mut self, pr_number: u64, priority: u32) {
        if let Some(pr) = self.pulls.get_mut(&pr_number) {
            pr.priority = priority;
        }
    }

    async fn approve_pr(&mut self, reviewer: &str, pr_number: u64) -> Result<()> {
        if let Some(pr) = self.pulls.get_mut(&pr_number) {
            // Skip adding approval if author matches the "reviewer"
            if !self.config.repo().allow_self_review() {
                match &pr.author {
                    Some(a) if a == reviewer => return Ok(()),
                    _ => {}
                }
            }

            let msg = if pr.approved_by.insert(reviewer.to_owned()) {
                // The approval was added
                format!(
                    ":pushpin: Commit {:7} has been approved by `{}`",
                    pr.head_ref_oid, reviewer
                )
            } else {
                // 'reviewer' had already approved the PR
                format!(
                    "@{} :bulb: You have already approved this PR, no need to approve it again",
                    reviewer
                )
            };

            self.github
                .issues()
                .create_comment(
                    self.config.repo().owner(),
                    self.config.repo().name(),
                    pr_number,
                    &msg,
                )
                .await?;
        }

        Ok(())
    }

    async fn synchronize(&mut self) {
        info!("Synchronizing");

        // TODO: Handle error
        let pulls = self
            .github
            .open_pulls(self.config.repo().owner(), self.config.repo().name())
            .await
            .unwrap();
        info!("{} Open PullRequests", pulls.len());

        // TODO: Scrape the comments/Reviews of each PR to pull out reviewer/approval data

        self.pulls.clear();
        self.pulls
            .extend(pulls.into_iter().map(|pr| (pr.number, pr)));

        info!("Done Synchronizing");
    }
}
