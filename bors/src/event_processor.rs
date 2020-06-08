use crate::{
    command::Command,
    config::{GitConfig, GithubConfig, RepoConfig},
    git::GitRepository,
    graphql::GithubClient,
    project_board::ProjectBoard,
    queue::MergeQueue,
    state::{PullRequestState, Status},
    Result,
};
use futures::{channel::mpsc, sink::SinkExt, stream::StreamExt};
use github::{Event, EventType, NodeId, PullRequestReviewEvent};
use log::{error, info, warn};
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
    config: RepoConfig,
    github: GithubClient,
    git_repository: GitRepository,
    merge_queue: MergeQueue,
    project_board: Option<ProjectBoard>,
    pulls: HashMap<u64, PullRequestState>,
    requests_rx: mpsc::Receiver<Request>,
}

impl EventProcessor {
    pub fn new(
        config: RepoConfig,
        github_config: &GithubConfig,
        git_config: &GitConfig,
    ) -> Result<(EventProcessorSender, Self)> {
        let (tx, rx) = mpsc::channel(1024);
        let github = GithubClient::new(&github_config.github_api_token);
        let git_repository = GitRepository::from_config(git_config, config.repo())?;

        Ok((
            EventProcessorSender::new(tx),
            Self {
                config,
                github,
                git_repository,
                merge_queue: MergeQueue::new(),
                project_board: None,
                pulls: HashMap::new(),
                requests_rx: rx,
            },
        ))
    }

    pub async fn start(mut self) {
        self.synchronize()
            .await
            .expect("unable to synchronize initial state");

        while let Some(request) = self.requests_rx.next().await {
            if let Err(e) = self.handle_request(request).await {
                error!("Error while handling request: {:?}", e);
            }
        }
    }

    async fn handle_request(&mut self, request: Request) -> Result<()> {
        use Request::*;
        match request {
            Webhook { event, delivery_id } => self.handle_webhook(event, delivery_id).await?,
        }

        Ok(())
    }

    async fn handle_webhook(&mut self, event: Event, delivery_id: String) -> Result<()> {
        info!(
            "Handling Webhook: event = '{:?}', id = {}",
            event.event_type(),
            delivery_id
        );

        // Verify that the event is from our configured repository
        if !event
            .repository()
            .map(|r| r.owner.login == self.config.owner() && r.name == self.config.name())
            .unwrap_or(false)
        {
            warn!("Recieved webhook intended for another repository");
            return Ok(());
        }

        match &event {
            Event::PullRequest(e) => self.handle_pull_request_event(e).await?,
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
                    .await?
                }
            }
            Event::PullRequestReview(e) => self.handle_pull_request_review_event(e).await?,
            Event::PullRequestReviewComment(e) => {
                if e.action.is_created() {
                    self.process_comment(
                        &e.sender.login,
                        e.pull_request.number,
                        e.comment.body(),
                        &e.comment.node_id,
                    )
                    .await?
                }
            }
            // Unsupported Event
            _ => {}
        }

        self.process_merge_queue().await?;

        Ok(())
    }

    async fn handle_pull_request_event(&mut self, event: &github::PullRequestEvent) -> Result<()> {
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
                let mut state = PullRequestState::from_pull_request(&event.pull_request);

                info!("PR #{} Opened", state.number);

                // Bors needs write access to the base repo to be able to function, so if the PR
                // originates from the base repo we don't need to worry about the
                // maintainer_can_modify bit (which actually can't be set oddly enough)
                let pr_is_from_base_repo = state
                    .head_repo
                    .as_ref()
                    .map(|repo| self.config.repo() == repo)
                    .unwrap_or(false);

                if self.config.maintainer_mode()
                    && !state.maintainer_can_modify
                    && !pr_is_from_base_repo
                {
                    self.github
                        .issues()
                        .create_comment(
                            self.config.repo().owner(),
                            self.config.repo().name(),
                            state.number,
                            &format!(":exclamation: before this PR can be merged please make sure that you enable \
                            [\"Allow edits from maintainers\"]\
                            (https://help.github.com/en/github/collaborating-with-issues-and-pull-requests/allowing-changes-to-a-pull-request-branch-created-from-a-fork).\n\n\
                            This is needed for tooling to be able to update this PR in-place so that Github can \
                            properly recognize and mark it as merged once its merged into the upstream branch"),
                        )
                        .await?;
                }

                if let Some(board) = &self.project_board {
                    board.create_card(&self.github, &mut state).await?;
                }

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
                if let Some(mut pull) = self.pulls.remove(&event.pull_request.number) {
                    if let Some(board) = &self.project_board {
                        board.delete_card(&self.github, &mut pull).await?;
                    }
                }
            }
            PullRequestEventAction::Labeled => {
                if let Some(label) = &event.label {
                    if let Some(pull) = self.pulls.get_mut(&event.pull_request.number) {
                        pull.labels.insert(label.name.clone());
                    }
                }
            }
            PullRequestEventAction::Unlabeled => {
                if let Some(label) = &event.label {
                    if let Some(pull) = self.pulls.get_mut(&event.pull_request.number) {
                        pull.labels.remove(&label.name);
                    }
                }
            }
            PullRequestEventAction::ConvertedToDraft => {
                if let Some(pull) = self.pulls.get_mut(&event.pull_request.number) {
                    pull.is_draft = true;
                }
            }
            PullRequestEventAction::ReadyForReview => {
                if let Some(pull) = self.pulls.get_mut(&event.pull_request.number) {
                    pull.is_draft = false;
                }
            }
            PullRequestEventAction::Edited => {
                // TODO maybe factor this out and run it on every PullRequestEvent type
                // Update PR state from Webhook
                if let Some(pull) = self.pulls.get_mut(&event.pull_request.number) {
                    if event.pull_request.title != pull.title {
                        pull.title = event.pull_request.title.clone();
                    }
                    let body = event.pull_request.body.as_deref().unwrap_or("");
                    if body != pull.body {
                        pull.body = body.to_owned();
                    }
                    if !matches!(pull.status, Status::Testing { .. }) {
                        if event.pull_request.base.git_ref != pull.base_ref_name {
                            pull.base_ref_name = event.pull_request.base.git_ref.clone();
                        }
                    }
                    if event.pull_request.base.sha != pull.base_ref_oid {
                        pull.base_ref_oid = event.pull_request.base.sha.clone();
                    }
                    if let Some(maintainer_can_modify) = event.pull_request.maintainer_can_modify {
                        pull.maintainer_can_modify = maintainer_can_modify;
                    }
                }
            }

            // Do nothing for actions we're not interested in
            _ => {}
        }

        Ok(())
    }

    fn pull_from_merge_oid(&mut self, oid: &github::Oid) -> Option<&mut PullRequestState> {
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

    async fn process_merge_queue(&mut self) -> Result<()> {
        self.merge_queue
            .process_queue(
                &self.config,
                &self.github,
                &mut self.git_repository,
                self.project_board.as_ref(),
                &mut self.pulls,
            )
            .await
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
                config: &self.config,
                project_board: self.project_board.as_ref(),
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
    ) -> Result<()> {
        info!("comment: {:#?}", comment);

        match comment.and_then(Command::from_comment) {
            Some(Ok(command)) => {
                info!("Valid Command");

                self.github
                    .add_reaction(node_id, github::ReactionType::Rocket)
                    .await?;

                let mut ctx = self.command_context(user, pr_number).unwrap();
                // Check if the user is authorized before executing the command
                if command.is_authorized(&ctx).await? {
                    command.execute(&mut ctx).await?;
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
                        &format!(
                            ":exclamation: Invalid command\n\n{}",
                            Command::help(&self.config, self.project_board.as_ref())
                        ),
                    )
                    .await?;
            }
            None => {
                info!("No command in comment");
            }
        }

        Ok(())
    }

    async fn handle_pull_request_review_event(&mut self, e: &PullRequestReviewEvent) -> Result<()> {
        let pr_number = e.pull_request.number;
        if let Some(pr) = self.pulls.get_mut(&pr_number) {
            pr.approved = self
                .github
                .get_review_decision(
                    self.config.repo().owner(),
                    self.config.repo().name(),
                    pr_number,
                )
                .await?;
        }

        if e.action.is_submitted() {
            self.process_comment(
                &e.sender.login,
                e.pull_request.number,
                e.review.body(),
                &e.review.node_id,
            )
            .await?
        }

        Ok(())
    }

    async fn synchronize(&mut self) -> Result<()> {
        info!("Synchronizing");

        let pulls = self
            .github
            .open_pulls(self.config.repo().owner(), self.config.repo().name())
            .await?;
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
        .await?;

        // Ensure all labels exist
        let owner = self.config.owner();
        let name = self.config.name();
        for label in self.config.labels().all() {
            if self
                .github
                .issues()
                .get_label(owner, name, label)
                .await
                .is_err()
            {
                self.github
                    .issues()
                    .create_label(owner, name, label, "D0D8D8", None)
                    .await?;
            }
        }

        self.project_board = Some(board);

        info!("Done Synchronizing");
        Ok(())
    }
}

pub struct CommandContext<'a> {
    pull_request: &'a mut PullRequestState,
    github: &'a GithubClient,
    config: &'a RepoConfig,
    project_board: Option<&'a ProjectBoard>,
    sender: &'a str,
}

impl<'a> CommandContext<'a> {
    pub fn pr(&self) -> &PullRequestState {
        &self.pull_request
    }

    #[allow(unused)]
    pub fn pr_mut(&mut self) -> &mut PullRequestState {
        &mut self.pull_request
    }

    pub fn github(&self) -> &GithubClient {
        &self.github
    }

    pub fn config(&self) -> &RepoConfig {
        &self.config
    }

    pub fn project_board(&self) -> Option<&'a ProjectBoard> {
        self.project_board
    }

    pub fn sender(&self) -> &str {
        &self.sender
    }

    #[allow(unused)]
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

    pub async fn update_pr_status(&mut self, status: Status) -> Result<()> {
        self.pull_request
            .update_status(status, self.config, self.github, self.project_board)
            .await?;

        Ok(())
    }

    pub async fn set_label(&mut self, label: &str) -> Result<()> {
        self.pull_request
            .add_label(self.config, self.github, label)
            .await
    }

    pub async fn remove_label(&mut self, label: &str) -> Result<()> {
        self.pull_request
            .remove_label(self.config, self.github, label)
            .await
    }
}
