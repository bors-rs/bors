use crate::{
    config::RepoConfig,
    git::GitRepository,
    graphql::GithubClient,
    project_board::ProjectBoard,
    state::{Priority, PullRequestState, Status, StatusType, TestSuiteResult},
    Result,
};
use github::Oid;
use log::info;
use std::collections::HashMap;

#[derive(Debug, PartialEq, PartialOrd, Eq, Ord)]
pub struct QueueEntry {
    status: StatusType,

    /// Indicates the priority of the PR
    priority: Priority,

    number: u64,
}

impl QueueEntry {
    pub fn new(number: u64, status: StatusType, priority: Priority) -> Self {
        Self {
            number,
            status,
            priority,
        }
    }
}

#[derive(Clone, Debug)]
pub struct MergeQueue {
    /// The current head of the queue, the PR that is currently being tested
    head: Option<u64>,
}

impl MergeQueue {
    pub fn new() -> Self {
        Self { head: None }
    }

    pub fn reset(&mut self) {
        self.head = None;
    }

    async fn land_pr(
        &mut self,
        config: &RepoConfig,
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
            if config.maintainer_mode() {
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

                    pull.update_status(Status::InReview, config, github, project_board)
                        .await?;

                    let comment =
                    ":exclamation: failed to update PR in-place; halting merge.\n\
                    Make sure that that [\"Allow edits from maintainers\"]\
                    (https://help.github.com/en/github/collaborating-with-issues-and-pull-requests/allowing-changes-to-a-pull-request-branch-created-from-a-fork) \
                    is enabled before attempting to reland this PR.";

                    github
                        .issues()
                        .create_comment(config.owner(), config.name(), pull.number, &comment)
                        .await?;

                    return Ok(());
                }

                // TODO we probably shouldn't spin waiting here. It might be better to wait till we
                // get a webhook back from Github that the PR was updated
                let r = format!("refs/pull/{}/head", pull.number);
                for i in 0..15 {
                    info!(
                        "Waiting for Github to update its ref '{}': attempt {}",
                        r, i
                    );

                    // Delay a few seconds to try and let Github properly update its references
                    tokio::time::delay_for(std::time::Duration::from_secs(1)).await;

                    let github = github
                        .pulls()
                        .get(config.owner(), config.name(), pull.number)
                        .await
                        .map(|p| p.into_inner().head.sha);
                    let git = repo.fetch_ref(&r);

                    match (git, github) {
                        (Ok(git), Ok(github)) => {
                            if merge_oid == &git && merge_oid == &github {
                                info!("Github's ref '{}' has been updated", r);
                                break;
                            }
                        }
                        (git, github) => {
                            info!("Github's ref's haven't updated yet.\nExpected: '{}'\nActual: git '{:?}' github '{:?}'", merge_oid, git, github);
                        }
                    }
                }
            }
        }

        // Finally 'merge' the PR by updating the 'base_ref' with `merge_oid`
        if let Err(e) = github
            .git()
            .update_ref(
                config.owner(),
                config.name(),
                &format!("heads/{}", pull.base_ref_name),
                &merge_oid,
                false,
            )
            .await
        {
            pull.update_status(Status::InReview, config, github, project_board)
                .await?;

            let comment = format!(
                "Error occured while trying to merge into {}:\n```\n{:#?}\n```",
                pull.base_ref_name, e
            );

            github
                .issues()
                .create_comment(config.owner(), config.name(), pull.number, &comment)
                .await?;

            return Ok(());
        }

        if let Some(board) = project_board {
            board.delete_card(github, &mut pull).await?;
        }

        // Actually remove the PR
        pulls.remove(&head);

        Ok(())
    }

    pub async fn process_queue(
        &mut self,
        config: &RepoConfig,
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
        config: &RepoConfig,
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
        let (merge_oid, test_suite_result) = match &pull.status {
            Status::Testing {
                merge_oid,
                tests_started_at,
                test_results,
            } => {
                let test_suite_result =
                    TestSuiteResult::new(*tests_started_at, test_results, config);
                (merge_oid, test_suite_result)
            }
            _ => {
                self.head = None;
                return Ok(());
            }
        };

        Self::update_github_based_on_test_suite_results(
            &pull,
            &test_suite_result,
            merge_oid,
            config,
            github,
        )
        .await?;

        match test_suite_result {
            TestSuiteResult::Failed { .. } | TestSuiteResult::TimedOut => {
                // Remove the PR from the Queue
                // XXX Maybe mark as "Failed"?
                pull.update_status(Status::InReview, config, github, project_board)
                    .await?;
                self.head.take();
            }

            TestSuiteResult::Passed => {
                self.land_pr(config, github, repo, project_board, pulls)
                    .await?;
            }

            TestSuiteResult::Pending => {}
        }

        Ok(())
    }

    async fn update_github_based_on_test_suite_results(
        pull: &PullRequestState,
        test_suite_result: &TestSuiteResult,
        merge_oid: &Oid,
        config: &RepoConfig,
        github: &GithubClient,
    ) -> Result<()> {
        match test_suite_result {
            TestSuiteResult::Failed { name, result } => {
                // Create github status/check
                github
                    .repos()
                    .create_status(
                        config.owner(),
                        config.name(),
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
                        config.owner(),
                        config.name(),
                        pull.number,
                        &format!(
                            ":broken_heart: Test Failed - [{}]({})",
                            name, result.details_url
                        ),
                    )
                    .await?;
            }
            TestSuiteResult::Passed => {
                // Create github status/check on the merge commit
                github
                    .repos()
                    .create_status(
                        config.owner(),
                        config.name(),
                        &merge_oid.to_string(),
                        &github::client::CreateStatusRequest {
                            state: github::StatusEventState::Success,
                            target_url: None,
                            description: None,
                            context: "bors",
                        },
                    )
                    .await?;
            }

            TestSuiteResult::TimedOut => {
                info!("PR #{} timed-out", pull.number);

                github
                    .repos()
                    .create_status(
                        config.owner(),
                        config.name(),
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
                        config.owner(),
                        config.name(),
                        pull.number,
                        ":boom: Tests timed-out",
                    )
                    .await?;
            }
            TestSuiteResult::Pending => {}
        }

        Ok(())
    }

    async fn process_next_head(
        &mut self,
        config: &RepoConfig,
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
        queue.sort_unstable_by_key(|p| {
            QueueEntry::new(p.number, p.status.status_type(), p.priority(config))
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
                pull.has_label(config.labels().squash()),
            )? {
                repo.push_branch("auto")?;
                info!("pushed 'auto' branch");

                pull.update_status(Status::testing(merge_oid), config, github, project_board)
                    .await?;
                self.head = Some(pull.number);

                // Create github status
                github
                    .repos()
                    .create_status(
                        config.owner(),
                        config.name(),
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
                pull.update_status(Status::InReview, config, github, project_board)
                    .await?;

                github
                    .repos()
                    .create_status(
                        config.owner(),
                        config.name(),
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
                        config.owner(),
                        config.name(),
                        pull.number,
                        ":lock: Merge Conflict",
                    )
                    .await?;
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn priority_sort() {
        let mut entries = vec![
            QueueEntry::new(1, StatusType::InReview, Priority::Normal),
            QueueEntry::new(10, StatusType::InReview, Priority::High),
        ];

        entries.sort();

        let expected = vec![
            QueueEntry::new(10, StatusType::InReview, Priority::High),
            QueueEntry::new(1, StatusType::InReview, Priority::Normal),
        ];
        assert_eq!(entries, expected);

        let mut entries = vec![
            QueueEntry::new(10, StatusType::InReview, Priority::Normal),
            QueueEntry::new(1, StatusType::InReview, Priority::Normal),
        ];

        entries.sort();

        let expected = vec![
            QueueEntry::new(1, StatusType::InReview, Priority::Normal),
            QueueEntry::new(10, StatusType::InReview, Priority::Normal),
        ];
        assert_eq!(entries, expected);

        let mut entries = vec![
            QueueEntry::new(1, StatusType::InReview, Priority::Low),
            QueueEntry::new(10, StatusType::InReview, Priority::Normal),
        ];

        entries.sort();

        let expected = vec![
            QueueEntry::new(10, StatusType::InReview, Priority::Normal),
            QueueEntry::new(1, StatusType::InReview, Priority::Low),
        ];
        assert_eq!(entries, expected);
    }
}
