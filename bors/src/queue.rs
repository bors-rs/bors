use crate::{
    git::GitRepository,
    graphql::GithubClient,
    project_board::ProjectBoard,
    state::{PullRequestState, Status},
    Config, Result,
};
use log::info;
use std::{
    cmp::{Ordering, Reverse},
    collections::HashMap,
};

#[derive(Debug, PartialEq, Eq)]
struct QueueEntry {
    number: u64,
    priority: u32,
}

impl PartialOrd for QueueEntry {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        match Reverse(self.priority).cmp(&Reverse(other.priority)) {
            Ordering::Equal => Some(self.number.cmp(&other.number)),
            ord => Some(ord),
        }
    }
}

impl Ord for QueueEntry {
    fn cmp(&self, other: &Self) -> Ordering {
        match Reverse(self.priority).cmp(&Reverse(other.priority)) {
            Ordering::Equal => self.number.cmp(&other.number),
            ord => ord,
        }
    }
}

#[derive(Debug)]
pub struct MergeQueue {
    /// The current head of the queue, the PR that is currently being tested
    head: Option<u64>,
}

impl MergeQueue {
    pub fn new() -> Self {
        Self { head: None }
    }

    async fn land_pr(
        &mut self,
        config: &Config,
        github: &GithubClient,
        repo: &mut GitRepository,
        project_board: Option<&ProjectBoard>,
        pulls: &mut HashMap<u64, PullRequestState>,
    ) -> Result<()> {
        let head = self
            .head
            .take()
            .expect("land_pr should only be called when there is a PR to land");

        let mut pull = pulls.get_mut(&head).expect("PR should exist");
        let merge_oid = match &pull.status {
            Status::Testing { merge_oid, .. } => merge_oid,
            // XXX Fix this
            _ => unreachable!(),
        };

        // Attempt to update the PR in-place
        if let Some(head_repo) = pull.head_repo.as_ref() {
            // Before 'merging' the PR into the base ref we first update the PR with the rebased
            // commits that are to be imminently merged using the `maintainer_can_modify` feature.
            // This is done so that when the commits are finally pushed to the base ref that Github
            // will properly mark the PR as being 'merged'.
            if repo
                .push_to_remote(
                    &head_repo,
                    &pull.head_ref_name,
                    &pull.head_ref_oid,
                    &merge_oid,
                )
                .is_err()
            {
                info!(
                    "unable to update pr #{} in-place. maintainer_can_modify: {}",
                    pull.number, pull.maintainer_can_modify
                );

                // XXX Add a config for this
                //if force_update_in_place {
                if false {
                    pull.update_status(Status::InReview, config.repo(), github, project_board)
                        .await?;

                    let comment = format!(
                        ":exclamation: failed to update PR in-place; halting merge.\n\
                        Make sure that that [\"Allow edits from maintainers\"]\
                        (https://help.github.com/en/github/collaborating-with-issues-and-pull-requests/allowing-changes-to-a-pull-request-branch-created-from-a-fork) \
                        is enabled before attempting to reland this PR.");

                    github
                        .issues()
                        .create_comment(
                            config.repo().owner(),
                            config.repo().name(),
                            pull.number,
                            &comment,
                        )
                        .await?;

                    return Ok(());
                }
            };
        }

        // Finally 'merge' the PR by updating the 'base_ref' with `merge_oid`
        github
            .git()
            .update_ref(
                config.repo().owner(),
                config.repo().name(),
                &format!("heads/{}", pull.base_ref_name),
                &merge_oid,
                false,
            )
            .await?;

        if let Some(board) = project_board {
            board.delete_card(github, &mut pull).await?;
        }
        pull.remove_labels(config.repo(), github).await?;

        // Actually remove the PR
        pulls.remove(&head);

        Ok(())
    }

    pub async fn process_queue(
        &mut self,
        config: &Config,
        github: &GithubClient,
        repo: &mut GitRepository,
        project_board: Option<&ProjectBoard>,
        pulls: &mut HashMap<u64, PullRequestState>,
    ) -> Result<()> {
        // Ensure that only ever 1 PR is in "Testing" at a time
        assert!(pulls.iter().filter(|(_n, p)| p.status.is_testing()).count() <= 1);

        // Process the PR at the head of the queue
        self.process_head(config, github, repo, project_board, pulls)
            .await?;

        if self.head.is_none() {
            self.process_next_head(config, github, repo, project_board, pulls)
                .await?;
        }

        Ok(())
    }

    async fn process_head(
        &mut self,
        config: &Config,
        github: &GithubClient,
        repo: &mut GitRepository,
        project_board: Option<&ProjectBoard>,
        pulls: &mut HashMap<u64, PullRequestState>,
    ) -> Result<()> {
        // Early return if there isn't anything at the head of the Queue currently being tested
        let head = if let Some(head) = self.head {
            head
        } else {
            return Ok(());
        };

        // Early return if the PR that was currently being tested was closed for some reason
        let pull = match pulls.get_mut(&head) {
            Some(pull) => pull,
            None => {
                self.head = None;
                return Ok(());
            }
        };

        // Early return if the PR that was currently being tested had its state changed from
        // `Status::Testing`, e.g. if the land was canceled.
        let (merge_oid, tests_started_at, test_results) = match &pull.status {
            Status::Testing {
                merge_oid,
                tests_started_at,
                test_results,
            } => (merge_oid, tests_started_at, test_results),
            _ => {
                self.head = None;
                return Ok(());
            }
        };

        // Check if there were any test failures from configured checks
        if let Some((name, result)) = config
            .repo()
            .checks()
            .filter_map(|name| test_results.get(name).map(|result| (name, result.clone())))
            .find(|(_name, result)| !result.passed)
        {
            // Remove the PR from the Queue
            // XXX Maybe mark as "Failed"?
            pull.update_status(Status::InReview, config.repo(), github, project_board)
                .await?;
            self.head.take();

            // Create github status/check
            github
                .repos()
                .create_status(
                    config.repo().owner(),
                    config.repo().name(),
                    &pull.head_ref_oid.to_string(),
                    &github::client::CreateStatusRequest {
                        state: github::StatusEventState::Failure,
                        target_url: Some(&result.details_url),
                        description: None,
                        context: "bors",
                    },
                )
                .await?;

            // Report the Error
            github
                .issues()
                .create_comment(
                    config.repo().owner(),
                    config.repo().name(),
                    pull.number,
                    &format!(
                        ":broken_heart: Test Failed - [{}]({})",
                        name, result.details_url
                    ),
                )
                .await?;

        // Check if all tests have completed and passed
        } else if config
            .repo()
            .checks()
            .map(|name| test_results.get(name))
            .all(|result| result.map(|r| r.passed).unwrap_or(false))
        {
            // Report the Success
            github
                .issues()
                .create_comment(
                    config.repo().owner(),
                    config.repo().name(),
                    pull.number,
                    &format!(
                        ":sunny: Tests Passed\nApproved by: {:?}\nPushing {} to {}...",
                        pull.approved_by, merge_oid, pull.base_ref_name
                    ),
                )
                .await?;

            // Create github status/check on the merge commit
            github
                .repos()
                .create_status(
                    config.repo().owner(),
                    config.repo().name(),
                    &merge_oid.to_string(),
                    &github::client::CreateStatusRequest {
                        state: github::StatusEventState::Success,
                        target_url: None,
                        description: None,
                        context: "bors",
                    },
                )
                .await?;

            self.land_pr(config, github, repo, project_board, pulls)
                .await?;

        // Check if the test has timed-out
        } else if tests_started_at.elapsed() >= config.repo().timeout() {
            info!("PR #{} timed-out", pull.number);

            // Remove the PR from the Queue
            // XXX Maybe mark as "Failed"?
            pull.update_status(Status::InReview, config.repo(), github, project_board)
                .await?;
            self.head = None;

            github
                .repos()
                .create_status(
                    config.repo().owner(),
                    config.repo().name(),
                    &pull.head_ref_oid.to_string(),
                    &github::client::CreateStatusRequest {
                        state: github::StatusEventState::Failure,
                        target_url: None,
                        description: Some("Timed-out"),
                        context: "bors",
                    },
                )
                .await?;

            // Report the Error
            github
                .issues()
                .create_comment(
                    config.repo().owner(),
                    config.repo().name(),
                    pull.number,
                    &format!(":boom: Tests timed-out"),
                )
                .await?;
        }

        Ok(())
    }

    async fn process_next_head(
        &mut self,
        config: &Config,
        github: &GithubClient,
        repo: &mut GitRepository,
        project_board: Option<&ProjectBoard>,
        pulls: &mut HashMap<u64, PullRequestState>,
    ) -> Result<()> {
        assert!(self.head.is_none());

        let mut queue: Vec<_> = pulls
            .iter_mut()
            .map(|(_n, p)| p)
            .filter(|p| p.status.is_queued())
            .collect();
        queue.sort_unstable_by_key(|p| QueueEntry {
            number: p.number,
            priority: p.priority,
        });
        let mut queue = queue.into_iter();

        while let (None, Some(pull)) = (self.head, queue.next()) {
            info!("Creating merge for pr #{}", pull.number);

            // Attempt to rebase the PR onto 'base_ref' and push to the 'auto' branch for
            // testing
            if let Some(merge_oid) = repo.fetch_and_rebase(
                &pull.base_ref_name,
                &pull.head_ref_oid,
                "auto",
                pull.number,
                pull.squash,
            )? {
                repo.push_branch("auto")?;
                info!("pushed 'auto' branch");

                pull.update_status(
                    Status::testing(merge_oid),
                    config.repo(),
                    github,
                    project_board,
                )
                .await?;
                self.head = Some(pull.number);

                // Create github status
                github
                    .repos()
                    .create_status(
                        config.repo().owner(),
                        config.repo().name(),
                        &pull.head_ref_oid.to_string(),
                        &github::client::CreateStatusRequest {
                            state: github::StatusEventState::Pending,
                            target_url: None,
                            description: None,
                            context: "bors",
                        },
                    )
                    .await?;
            } else {
                pull.update_status(Status::InReview, config.repo(), github, project_board)
                    .await?;

                github
                    .repos()
                    .create_status(
                        config.repo().owner(),
                        config.repo().name(),
                        &pull.head_ref_oid.to_string(),
                        &github::client::CreateStatusRequest {
                            state: github::StatusEventState::Error,
                            target_url: None,
                            description: Some("Merge Conflict"),
                            context: "bors",
                        },
                    )
                    .await?;

                github
                    .issues()
                    .create_comment(
                        config.repo().owner(),
                        config.repo().name(),
                        pull.number,
                        ":lock: Merge Conflict",
                    )
                    .await?;
            }
        }

        Ok(())
    }
}
