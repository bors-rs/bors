use super::{PullRequest, Repository, Review, ReviewComment, User};
use serde::Deserialize;
use thiserror::Error;

pub enum EventType {
    CheckRun,
    CheckSuite,
    CommitComment,
    ContentReference,
    Create,
    Delete,
    DeployKey,
    Deployment,
    DeploymentStatus,
    Download, // Depricated
    Follow,   // Depricated
    Fork,
    ForkApply, // Depricated
    GithubAppAuthorization,
    Gist, // Depricated
    Gollum,
    Installation,
    InstallationRepositories,
    IssueComment,
    Issues,
    Label,
    MarketplacePurchase,
    Member,
    Membership,
    Meta,
    Milestone,
    Organization,
    OrgBlock,
    Package,
    PageBuild,
    ProjectCard,
    ProjectColumn,
    Project,
    Public,
    PullRequest,
    PullRequestReview,
    PullRequestReviewComment,
    Push,
    Release,
    RepositoryDispatch,
    Repository,
    RepositoryImport,
    RepositoryVulnerabilityAlert,
    SecurityAdvisory,
    Star,
    Status,
    Team,
    TeamAdd,
    Watch,
}

#[derive(Error, Debug)]
#[error("invalid github webhook event")]
pub struct ParseEventTypeError;

impl std::str::FromStr for EventType {
    type Err = ParseEventTypeError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        use EventType::*;

        match s {
            "check_run" => Ok(CheckRun),
            "check_suite" => Ok(CheckSuite),
            "commit_comment" => Ok(CommitComment),
            "content_reference" => Ok(ContentReference),
            "create" => Ok(Create),
            "delete" => Ok(Delete),
            "deploy_key" => Ok(DeployKey),
            "deployment" => Ok(Deployment),
            "deployment_status" => Ok(DeploymentStatus),
            "download" => Ok(Download),
            "follow" => Ok(Follow),
            "fork" => Ok(Fork),
            "fork_apply" => Ok(ForkApply),
            "github_app_authorization" => Ok(GithubAppAuthorization),
            "gist" => Ok(Gist),
            "gollum" => Ok(Gollum),
            "installation" => Ok(Installation),
            "installation_repositories" => Ok(InstallationRepositories),
            "issue_comment" => Ok(IssueComment),
            "issues" => Ok(Issues),
            "label" => Ok(Label),
            "marketplace_purchase" => Ok(MarketplacePurchase),
            "member" => Ok(Member),
            "membership" => Ok(Membership),
            "meta" => Ok(Meta),
            "milestone" => Ok(Milestone),
            "organization" => Ok(Organization),
            "org_block" => Ok(OrgBlock),
            "package" => Ok(Package),
            "page_build" => Ok(PageBuild),
            "project_card" => Ok(ProjectCard),
            "project_column" => Ok(ProjectColumn),
            "project" => Ok(Project),
            "public" => Ok(Public),
            "pull_request" => Ok(PullRequest),
            "pull_request_review" => Ok(PullRequestReview),
            "pull_request_review_comment" => Ok(PullRequestReviewComment),
            "push" => Ok(Push),
            "release" => Ok(Release),
            "repository_dispatch" => Ok(RepositoryDispatch),
            "repository" => Ok(Repository),
            "repository_import" => Ok(RepositoryImport),
            "repository_vulnerability_alert" => Ok(RepositoryVulnerabilityAlert),
            "security_advisory" => Ok(SecurityAdvisory),
            "star" => Ok(Star),
            "status" => Ok(Status),
            "team" => Ok(Team),
            "team_add" => Ok(TeamAdd),
            "watch" => Ok(Watch),
            _ => Err(ParseEventTypeError),
        }
    }
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "snake_case")]
/// The action that was performed. Can be one of assigned, unassigned, review_requested,
/// review_request_removed, labeled, unlabeled, opened, edited, closed, ready_for_review, locked,
/// unlocked, or reopened. If the action is closed and the merged key is false, the pull request
/// was closed with unmerged commits. If the action is closed and the merged key is true, the pull
/// request was merged. While webhooks are also triggered when a pull request is synchronized,
/// Events API timelines don't include pull request events with the synchronize action.
pub enum PullRequestEventAction {
    Assigned,
    Unassigned,
    Labeled,
    Unlabeled,
    Opened,
    Edited,
    Closed,
    Reopened,
    Synchronize,
    ReadyForReview,
    Locked,
    Unlocked,
}

#[derive(Debug, Deserialize)]
pub struct PullRequestEvent {
    action: PullRequestEventAction,
    number: u64,
    pull_request: PullRequest,
    repository: Repository,
    sender: User,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PullRequestReviewEventAction {
    Submitted,
    Edited,
    Dismissed,
}

#[derive(Debug, Deserialize)]
pub struct PullRequestReviewEvent {
    action: PullRequestReviewEventAction,
    review: Review,
    pull_request: PullRequest,
    repository: Repository,
    sender: User,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PullRequestReviewCommentEventAction {
    Created,
    Edited,
    Deleted,
}

#[derive(Debug, Deserialize)]
pub struct PullRequestReviewCommentEvent {
    action: PullRequestReviewCommentEventAction,
    comment: ReviewComment,
    pull_request: PullRequest,
    repository: Repository,
    sender: User,
}
