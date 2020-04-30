use crate::{
    git::GitRepository,
    graphql::GithubClient,
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
        pulls: &mut HashMap<u64, PullRequestState>,
    ) -> Result<()> {
        let head = self
            .head
            .take()
            .expect("land_pr should only be called when there is a PR to land");

        let pull = pulls.remove(&head).expect("PR should exist");
        let merge_oid = pull.merge_oid.as_ref().unwrap();
        let head_repo = pull.head_repo.as_ref().unwrap();

        assert!(pull.maintainer_can_modify);

        // Before 'merging' the PR into the base ref we first update the PR with the rebased
        // commits that are to be imminently merged using the `maintainer_can_modify` feature.
        // This is done so that when the commits are finally pushed to the base ref that Github
        // will properly mark the PR as being 'merged'.
        repo.push_to_remote(
            &head_repo,
            &pull.head_ref_name,
            &pull.head_ref_oid,
            &merge_oid,
        )?;

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
        Ok(())
    }

    pub async fn process_queue(
        &mut self,
        config: &Config,
        github: &GithubClient,
        repo: &mut GitRepository,
        pulls: &mut HashMap<u64, PullRequestState>,
    ) -> Result<()> {
        // Ensure that only ever 1 PR is in "Testing" at a time
        assert!(
            pulls
                .iter()
                .filter(|(_n, p)| matches!(p.status, Status::Testing))
                .count()
                <= 1
        );

        // Process the PR at the head of the queue
        if let Some(head) = self.head {
            let pull = pulls.get_mut(&head).expect("PR should exist");

            // Check if there were any test failures from configured checks
            if let Some((name, result)) = config
                .repo()
                .checks()
                .filter_map(|name| {
                    pull.test_results
                        .get(name)
                        .map(|result| (name, result.clone()))
                })
                .find(|(_name, result)| !result.passed)
            {
                // Remove the PR from the Queue
                pull.status = Status::InReview;
                self.head.take();

                // XXX Create github status/check

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
            } else {
                // Check to see if all tests have completed and passed
                if config
                    .repo()
                    .checks()
                    .map(|name| pull.test_results.get(name))
                    .all(|maybe_result| {
                        if let Some(result) = maybe_result {
                            result.passed
                        } else {
                            false
                        }
                    })
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
                                pull.approved_by,
                                pull.merge_oid.as_ref().unwrap(),
                                pull.base_ref_name
                            ),
                        )
                        .await?;

                    // XXX Create github status/check
                    self.land_pr(config, github, repo, pulls).await?;
                }
            }
        }

        if self.head.is_none() {
            // Get the next ready PR to begin testing
            let next = pulls
                .iter_mut()
                .map(|(_n, p)| p)
                .filter(|p| matches!(p.status, Status::ReadyToLand))
                .min_by_key(|p| QueueEntry {
                    number: p.number,
                    priority: p.priority,
                });

            if let Some(pr) = next {
                info!("Creating merge for pr #{}", pr.number);

                // Attempt to rebase the PR onto 'base_ref' and push to the 'auto' branch for
                // testing
                if let Some(merge_oid) =
                    repo.fetch_and_rebase(&pr.base_ref_name, &pr.head_ref_oid, "auto")?
                {
                    repo.push_branch("auto")?;
                    info!("pushed 'auto' branch");

                    pr.merge_oid = Some(merge_oid);
                    pr.status = Status::Testing;
                    self.head = Some(pr.number);
                } else {
                    unimplemented!("Merge conflict");
                }
            }
        }

        Ok(())
    }
}
