use crate::{
    command::Command, config::RepoConfig, error::Result, graphql::GithubClient,
    state::PullRequestState, Config,
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

    fn command_context<'a>(
        &'a mut self,
        sender: &'a str,
        pr_number: u64,
    ) -> Option<CommandContext<'a>> {
        if let Some(pr) = self.pulls.get_mut(&pr_number) {
            Some(CommandContext {
                pull_request: pr,
                github: &self.github,
                config: self.config.repo(),
                sender,
            })
        } else {
            None
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

                let mut ctx = self.command_context(user, pr_number).unwrap();
                // Check if the user is authorized before executing the command
                if Self::is_authorized(&ctx, &command).await.unwrap() {
                    Self::execute_command(&mut ctx, command).await;
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
    async fn is_authorized(ctx: &CommandContext<'_>, command: &Command) -> Result<bool> {
        let mut is_authorized = false;
        let mut reason = None;

        // Check to see if the user is a collaborator
        if ctx
            .github()
            .repos()
            .is_collaborator(ctx.config().owner(), ctx.config().name(), ctx.sender())
            .await?
            .into_inner()
        {
            is_authorized = true;
        } else {
            reason = Some("Not Collaborator");
        }

        // Check to see if the user issuing the command is attempting to approve their own PR
        use crate::command::CommandType;
        if is_authorized
            && !ctx.config().allow_self_review()
            && ctx.sender_is_author()
            && matches!(command.command_type, CommandType::Approve(_))
        {
            is_authorized = false;
            reason = Some("Can't approve your own PR");
        }

        // Post a comment to Github if there was a reason why the user wasn't authorized
        if !is_authorized {
            if let Some(reason) = reason {
                ctx.create_pr_comment(&format!(
                    "@{}: :key: Insufficient privileges: {}",
                    ctx.sender(),
                    reason
                ))
                .await?;
            }
        }

        Ok(is_authorized)
    }

    async fn execute_command(mut ctx: &mut CommandContext<'_>, command: Command) {
        use crate::command::CommandType;

        match command.command_type {
            CommandType::Approve(a) => {
                if let Some(priority) = a.priority() {
                    Self::set_priority(&mut ctx, priority);
                }

                Self::approve_pr(&mut ctx).await.unwrap();
            }
            CommandType::Unapprove => unimplemented!(),
            CommandType::Land(l) => {
                if let Some(priority) = l.priority() {
                    Self::set_priority(&mut ctx, priority);
                }

                unimplemented!();
            }
            CommandType::Retry(r) => {
                if let Some(priority) = r.priority() {
                    Self::set_priority(&mut ctx, priority);
                }
                unimplemented!();
            }
            CommandType::Cancel => unimplemented!(),
            CommandType::Priority(p) => Self::set_priority(&mut ctx, p.priority()),
        }
    }

    fn set_priority(ctx: &mut CommandContext, priority: u32) {
        ctx.pr_mut().priority = priority;
    }

    async fn approve_pr(ctx: &mut CommandContext<'_>) -> Result<()> {
        // Skip adding approval if author matches the "reviewer"
        if !ctx.config().allow_self_review() && ctx.sender_is_author() {
            return Ok(());
        }

        let reviewer = ctx.sender().to_owned();
        let msg = if ctx.pr_mut().approved_by.insert(reviewer) {
            // The approval was added
            format!(
                ":pushpin: Commit {:7} has been approved by `{}`",
                ctx.pr().head_ref_oid,
                ctx.sender()
            )
        } else {
            // 'reviewer' had already approved the PR
            format!(
                "@{} :bulb: You have already approved this PR, no need to approve it again",
                ctx.sender()
            )
        };

        ctx.create_pr_comment(&msg).await?;

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

struct CommandContext<'a> {
    pull_request: &'a mut PullRequestState,
    github: &'a GithubClient,
    config: &'a RepoConfig,
    sender: &'a str,
}

impl<'a> CommandContext<'a> {
    fn pr(&self) -> &PullRequestState {
        &self.pull_request
    }

    fn pr_mut(&mut self) -> &mut PullRequestState {
        &mut self.pull_request
    }

    fn github(&self) -> &GithubClient {
        &self.github
    }

    fn config(&self) -> &RepoConfig {
        &self.config
    }

    fn sender(&self) -> &str {
        &self.sender
    }

    fn sender_is_author(&self) -> bool {
        if let Some(author) = &self.pull_request.author {
            author == self.sender
        } else {
            false
        }
    }

    async fn create_pr_comment(&self, body: &str) -> Result<()> {
        self.github()
            .issues()
            .create_comment(
                self.config().owner(),
                self.config().name(),
                self.pr().number,
                body,
            )
            .await?;
        Ok(())
    }
}
