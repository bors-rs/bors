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

        // Land the PR at the head of the queue
        if let Some(head) = self.head.take() {
            let pull = pulls.remove(&head).expect("PR should exist");
            let merge_oid = pull.merge_oid.unwrap();
            let head_repo = pull.head_repo.unwrap();

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
