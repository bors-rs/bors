use crate::{graphql::GithubClient, state::PullRequestState, Config, Result};
use github::{Project, ProjectColumn};
use std::collections::HashMap;

const PROJECT_BOARD_NAME: &str = "bors";
const REVIEW_COLUMN_NAME: &str = "In Review";
const QUEUED_COLUMN_NAME: &str = "Queued";
const TESTING_COLUMN_NAME: &str = "Testing";

#[derive(Debug)]
pub struct ProjectBoard {
    board: Project,
    review_column: ProjectColumn,
    queued_column: ProjectColumn,
    testing_column: ProjectColumn,
}

impl ProjectBoard {
    pub async fn move_to_review(
        &self,
        github: &GithubClient,
        pull: &PullRequestState,
    ) -> Result<()> {
        if let Some(card_id) = pull.project_card_id {
            Self::move_card_to_column(github, card_id, self.review_column.id).await?;
        }

        Ok(())
    }

    pub async fn move_to_queued(
        &self,
        github: &GithubClient,
        pull: &PullRequestState,
    ) -> Result<()> {
        if let Some(card_id) = pull.project_card_id {
            Self::move_card_to_column(github, card_id, self.queued_column.id).await?;
        }

        Ok(())
    }

    pub async fn move_to_testing(
        &self,
        github: &GithubClient,
        pull: &PullRequestState,
    ) -> Result<()> {
        if let Some(card_id) = pull.project_card_id {
            Self::move_card_to_column(github, card_id, self.testing_column.id).await?;
        }

        Ok(())
    }

    pub async fn create_card(
        &self,
        github: &GithubClient,
        pull: &mut PullRequestState,
    ) -> Result<()> {
        assert!(pull.project_card_id.is_none());

        let request = github::client::CreateProjectCardRequest {
            note: None,
            content_id: Some(pull.id),
            content_type: Some("PullRequest".into()),
        };
        let card = github
            .projects()
            .create_card(self.review_column.id, &request)
            .await?
            .into_inner();

        pull.project_card_id = Some(card.id);

        Ok(())
    }

    pub async fn delete_card(
        &self,
        github: &GithubClient,
        pull: &mut PullRequestState,
    ) -> Result<()> {
        if let Some(card_id) = pull.project_card_id.take() {
            github.projects().delete_card(card_id).await?;
        }

        Ok(())
    }

    pub async fn synchronize_or_init(
        github: &GithubClient,
        config: &Config,
        open_pulls: &mut HashMap<u64, PullRequestState>,
    ) -> Result<Self> {
        let board = Self::create_or_get_project_board(github, config).await?;

        let (review_column, queued_column, testing_column) =
            Self::create_or_get_columns(github, board.id).await?;

        Self::init_project_cards(
            github,
            open_pulls,
            review_column.id,
            queued_column.id,
            testing_column.id,
        )
        .await?;

        Ok(Self {
            board,
            review_column,
            queued_column,
            testing_column,
        })
    }

    async fn create_or_get_project_board(
        github: &GithubClient,
        config: &Config,
    ) -> Result<github::Project> {
        let mut project_board = None;
        for project in github
            .projects()
            .list_for_repo(config.repo().owner(), config.repo().name(), None)
            .await?
            .into_inner()
        {
            if project.name == PROJECT_BOARD_NAME {
                project_board = Some(project);
                break;
            }
        }

        let project_board = if let Some(board) = project_board {
            board
        } else {
            github
                .projects()
                .create_for_repo(config.repo().owner(), config.repo().name(), "bors", None)
                .await?
                .into_inner()
        };

        Ok(project_board)
    }

    async fn create_or_get_columns(
        github: &GithubClient,
        project_id: u64,
    ) -> Result<(
        github::ProjectColumn,
        github::ProjectColumn,
        github::ProjectColumn,
    )> {
        let mut review_column = None;
        let mut queued_column = None;
        let mut testing_column = None;

        for column in github
            .projects()
            .list_columns(project_id, None)
            .await?
            .into_inner()
        {
            match column.name.as_ref() {
                "In Review" => review_column = Some(column),
                "Queued" => queued_column = Some(column),
                "Testing" => testing_column = Some(column),
                // Delete columns which don't match
                _ => {
                    github.projects().delete_column(column.id).await?;
                }
            }
        }

        let review_column =
            Self::unwrap_or_create_column(review_column, REVIEW_COLUMN_NAME, project_id, github)
                .await?;
        let queued_column =
            Self::unwrap_or_create_column(queued_column, QUEUED_COLUMN_NAME, project_id, github)
                .await?;
        let testing_column =
            Self::unwrap_or_create_column(testing_column, TESTING_COLUMN_NAME, project_id, github)
                .await?;

        Ok((review_column, queued_column, testing_column))
    }

    async fn unwrap_or_create_column(
        column: Option<github::ProjectColumn>,
        column_name: &str,
        project_id: u64,
        github: &GithubClient,
    ) -> Result<github::ProjectColumn> {
        let column = if let Some(column) = column {
            column
        } else {
            github
                .projects()
                .create_column(project_id, column_name)
                .await?
                .into_inner()
        };

        Ok(column)
    }

    async fn init_project_cards(
        github: &GithubClient,
        open_pulls: &mut HashMap<u64, PullRequestState>,
        review_column_id: u64,
        queued_column_id: u64,
        testing_column_id: u64,
    ) -> Result<()> {
        Self::assign_or_delete_cards_in_column(github, open_pulls, review_column_id, None).await?;
        Self::assign_or_delete_cards_in_column(
            github,
            open_pulls,
            queued_column_id,
            Some(review_column_id),
        )
        .await?;
        Self::assign_or_delete_cards_in_column(
            github,
            open_pulls,
            testing_column_id,
            Some(review_column_id),
        )
        .await?;

        // Create cards for remaining PRs
        for (_n, pull) in open_pulls.iter_mut() {
            if pull.project_card_id.is_none() {
                let request = github::client::CreateProjectCardRequest {
                    note: None,
                    content_id: Some(pull.id),
                    content_type: Some("PullRequest".into()),
                };
                let card = github
                    .projects()
                    .create_card(review_column_id, &request)
                    .await?
                    .into_inner();

                pull.project_card_id = Some(card.id);
            }
        }

        Ok(())
    }

    async fn assign_or_delete_cards_in_column(
        github: &GithubClient,
        open_pulls: &mut HashMap<u64, PullRequestState>,
        column_id: u64,
        dst_column: Option<u64>,
    ) -> Result<()> {
        for card in github
            .projects()
            .list_cards(column_id, None)
            .await?
            .into_inner()
        {
            match card.issue_number().and_then(|n| open_pulls.get_mut(&n)) {
                Some(pull) => {
                    pull.project_card_id = Some(card.id);
                    if let Some(dst_column) = dst_column {
                        Self::move_card_to_column(github, card.id, dst_column).await?;
                    }
                }
                None => {
                    github.projects().delete_card(card.id).await.unwrap();
                }
            }
        }
        Ok(())
    }

    async fn move_card_to_column(
        github: &GithubClient,
        card_id: u64,
        column_id: u64,
    ) -> Result<()> {
        let request = github::client::MoveProjectCardRequest {
            position: "bottom".into(),
            column_id: Some(column_id),
        };
        github.projects().move_card(card_id, &request).await?;
        Ok(())
    }
}
