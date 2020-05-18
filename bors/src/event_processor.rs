use crate::{
    command::Command, config::RepoConfig, git::GitRepository, graphql::GithubClient,
    project_board::ProjectBoard, queue::MergeQueue, state::PullRequestState, Config, Result,
};
use futures::{channel::mpsc, lock::Mutex, sink::SinkExt, stream::StreamExt};
use github::{Event, EventType, NodeId};
use hotpot_db::HotPot;
use log::{info, warn};
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
    git_repository: GitRepository,
    merge_queue: MergeQueue,
    project_board: Option<ProjectBoard>,
    pulls: HashMap<u64, PullRequestState>,
    db: Mutex<HotPot>,
    requests_rx: mpsc::Receiver<Request>,
}

impl EventProcessor {
    pub fn new(config: Config) -> Result<(EventProcessorSender, Self)> {
        let (tx, rx) = mpsc::channel(1024);
        let github = GithubClient::new(&config.github_api_token);
        let git_repository = GitRepository::from_config(&config)?;

        Ok((
            EventProcessorSender::new(tx),
            Self {
                config,
                github,
                git_repository,
                merge_queue: MergeQueue::new(),
                project_board: None,
                pulls: HashMap::new(),
                db: Mutex::new(HotPot::new()),
                requests_rx: rx,
            },
        ))
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
        info!(
            "Handling Webhook: event = '{:?}', id = {}",
            event.event_type(),
            delivery_id
        );

        //TODO route on the request
        match &event {
            Event::PullRequest(e) => self.handle_pull_request_event(e),
            Event::CheckRun(e) => self.handle_check_run_event(e),
            Event::Status(e) => self.handle_status_event(e),
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
                    self.process_review(&e.sender.login, e.pull_request.number, &e.review)
                        .await;
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

        self.process_merge_queue().await
    }

    fn handle_pull_request_event(&mut self, event: &github::PullRequestEvent) {
        use github::PullRequestEventAction;

        info!(
            "Handling PullRequestEvent '{:?}' for PR #{}",
            event.action, event.pull_request.number
        );

        match event.action {
            PullRequestEventAction::Synchronize => {
                if let Some(pr) = self.pulls.get_mut(&event.pull_request.number) {
                    pr.update_head(event.pull_request.head.sha.clone())
                }
            }
            PullRequestEventAction::Opened | PullRequestEventAction::Reopened => {
                let state = PullRequestState::from_pull_request(&event.pull_request);

                info!("PR #{} Opened", state.number);

                if self.pulls.insert(state.number, state).is_some() {
                    warn!("Opened/Reopened event replaced an existing PullRequestState");
                }
            }
            PullRequestEventAction::Closed => {
                // From [Github's API docs](https://developer.github.com/v3/activity/events/types/#events-api-payload-31):
                //      If the action is closed and the merged key is false, the pull request was
                //      closed with unmerged commits. If the action is closed and the merged key
                //      is true, the pull request was merged.
                let merged = event.pull_request.merged.unwrap_or(false);

                if merged {
                    info!("pr #{} successfully Merged!", event.pull_request.number);
                }

                // XXX Do we need to call into the MergeQueue to notify it that a PR was merged or
                // closed?
                self.pulls.remove(&event.pull_request.number);
            }

            // Do nothing for actions we're not interested in
            _ => {}
        }
    }

    fn pull_from_merge_oid(&mut self, oid: &github::Oid) -> Option<&mut PullRequestState> {
        use crate::state::Status;

        self.pulls
            .iter_mut()
            .find(|(_n, pr)| match &pr.status {
                Status::Testing { merge_oid, .. } => merge_oid == oid,
                Status::InReview | Status::Queued => false,
            })
            .map(|(_n, pr)| pr)
    }

    fn handle_check_run_event(&mut self, event: &github::CheckRunEvent) {
        info!("Handling CheckRunEvent");

        // Skip the event if it hasn't completed
        let conclusion = match (
            event.action,
            event.check_run.status,
            event.check_run.conclusion,
        ) {
            (
                github::CheckRunEventAction::Completed,
                github::CheckStatus::Completed,
                Some(conclusion),
            ) => conclusion,
            _ => return,
        };

        if let Some(pr) = self.pull_from_merge_oid(&event.check_run.head_sha) {
            pr.add_build_result(
                &event.check_run.name,
                &event.check_run.details_url,
                conclusion,
            );
        }
    }

    // XXX This currently shoehorns github's statuses to fit into the new checks api. We should
    // probably introduce a few types to distinguish between the two
    fn handle_status_event(&mut self, event: &github::StatusEvent) {
        // Skip the event if it hasn't completed
        let conclusion = match event.state {
            github::StatusEventState::Pending => return,
            github::StatusEventState::Success => github::Conclusion::Success,
            github::StatusEventState::Failure => github::Conclusion::Failure,
            github::StatusEventState::Error => github::Conclusion::Failure,
        };

        if let Some(pr) = self.pull_from_merge_oid(&event.sha) {
            pr.add_build_result(
                &event.context,
                &event.target_url.as_deref().unwrap_or(""),
                conclusion,
            );
        }
    }

    async fn process_merge_queue(&mut self) {
        self.merge_queue
            .process_queue(
                &self.config,
                &self.github,
                &mut self.git_repository,
                &mut self.pulls,
            )
            .await
            .unwrap()
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
                if command.is_authorized(&ctx).await.unwrap() {
                    command.execute(&mut ctx).await.unwrap();
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

    async fn process_review(&mut self, sender: &str, pr_number: u64, review: &github::Review) {
        if let Some(command) = Command::from_review(review) {
            let mut ctx = self.command_context(sender, pr_number).unwrap();
            // Check if the user is authorized before executing the command
            if command.is_authorized(&ctx).await.unwrap() {
                command.execute(&mut ctx).await.unwrap();
            }
        }
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

        // Sync and reset project board
        let board = crate::project_board::ProjectBoard::synchronize_or_init(
            &self.github,
            &self.config,
            &mut self.pulls,
        )
        .await
        .unwrap();

        self.project_board = Some(board);

        info!("Done Synchronizing");
    }
}

pub struct CommandContext<'a> {
    pull_request: &'a mut PullRequestState,
    github: &'a GithubClient,
    config: &'a RepoConfig,
    sender: &'a str,
}

impl<'a> CommandContext<'a> {
    pub fn pr(&self) -> &PullRequestState {
        &self.pull_request
    }

    pub fn pr_mut(&mut self) -> &mut PullRequestState {
        &mut self.pull_request
    }

    pub fn github(&self) -> &GithubClient {
        &self.github
    }

    pub fn config(&self) -> &RepoConfig {
        &self.config
    }

    pub fn sender(&self) -> &str {
        &self.sender
    }

    pub fn sender_is_author(&self) -> bool {
        if let Some(author) = &self.pull_request.author {
            author == self.sender
        } else {
            false
        }
    }

    pub async fn create_pr_comment(&self, body: &str) -> Result<()> {
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
