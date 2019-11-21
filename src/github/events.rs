use super::{
    CheckRun, CheckSuite, Comment, Commit, Issue, Label, Oid, PullRequest, Pusher, Repository,
    Review, ReviewComment, User,
};
use serde::{de, Deserialize};
use std::str::FromStr;
use thiserror::Error;

#[derive(Debug)]
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
    RegistryPackage,
    Release,
    Repository,
    RepositoryDispatch,
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

impl FromStr for EventType {
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
            "registry_package" => Ok(RegistryPackage),
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

impl<'de> Deserialize<'de> for EventType {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: de::Deserializer<'de>,
    {
        let s = <&str>::deserialize(deserializer)?;
        Self::from_str(s).map_err(de::Error::custom)
    }
}

#[derive(Debug)]
pub enum Event {
    CheckRun(CheckRunEvent),
    CheckSuite(CheckSuiteEvent),
    CommitComment,
    ContentReference,
    Create,
    Delete,
    DeployKey,
    Deployment,
    DeploymentStatus,
    Download,
    Follow,
    Fork,
    ForkApply,
    GithubAppAuthorization,
    Gist,
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
    PullRequest(PullRequestEvent),
    PullRequestReview(PullRequestReviewEvent),
    PullRequestReviewComment(PullRequestReviewCommentEvent),
    Push,
    RegistryPackage,
    Release,
    Repository,
    RepositoryDispatch,
    RepositoryImport,
    RepositoryVulnerabilityAlert,
    SecurityAdvisory,
    Star,
    Status,
    Team,
    TeamAdd,
    Watch,
}

impl Event {
    // Maybe rename this or use TryFrom<(EventType, &[u8])>?
    pub fn from_json(event_type: &EventType, json: &[u8]) -> Result<Self, serde_json::Error> {
        let event = match event_type {
            EventType::CheckRun => Event::CheckRun(serde_json::from_slice(json)?),
            EventType::CheckSuite => Event::CheckSuite(serde_json::from_slice(json)?),
            EventType::CommitComment => Event::CommitComment,
            EventType::ContentReference => Event::ContentReference,
            EventType::Create => Event::Create,
            EventType::Delete => Event::Delete,
            EventType::DeployKey => Event::DeployKey,
            EventType::Deployment => Event::Deployment,
            EventType::DeploymentStatus => Event::DeploymentStatus,
            EventType::Download => Event::Download,
            EventType::Follow => Event::Follow,
            EventType::Fork => Event::Fork,
            EventType::ForkApply => Event::ForkApply,
            EventType::GithubAppAuthorization => Event::GithubAppAuthorization,
            EventType::Gist => Event::Gist,
            EventType::Gollum => Event::Gollum,
            EventType::Installation => Event::Installation,
            EventType::InstallationRepositories => Event::InstallationRepositories,
            EventType::IssueComment => Event::IssueComment,
            EventType::Issues => Event::Issues,
            EventType::Label => Event::Label,
            EventType::MarketplacePurchase => Event::MarketplacePurchase,
            EventType::Member => Event::Member,
            EventType::Membership => Event::Membership,
            EventType::Meta => Event::Meta,
            EventType::Milestone => Event::Milestone,
            EventType::Organization => Event::Organization,
            EventType::OrgBlock => Event::OrgBlock,
            EventType::Package => Event::Package,
            EventType::PageBuild => Event::PageBuild,
            EventType::ProjectCard => Event::ProjectCard,
            EventType::ProjectColumn => Event::ProjectColumn,
            EventType::Project => Event::Project,
            EventType::Public => Event::Public,
            EventType::PullRequest => Event::PullRequest(serde_json::from_slice(json)?),
            EventType::PullRequestReview => Event::PullRequestReview(serde_json::from_slice(json)?),
            EventType::PullRequestReviewComment => {
                Event::PullRequestReviewComment(serde_json::from_slice(json)?)
            }
            EventType::Push => Event::Push,
            EventType::RegistryPackage => Event::RegistryPackage,
            EventType::Release => Event::Release,
            EventType::Repository => Event::Repository,
            EventType::RepositoryDispatch => Event::RepositoryDispatch,
            EventType::RepositoryImport => Event::RepositoryImport,
            EventType::RepositoryVulnerabilityAlert => Event::RepositoryVulnerabilityAlert,
            EventType::SecurityAdvisory => Event::SecurityAdvisory,
            EventType::Star => Event::Star,
            EventType::Status => Event::Status,
            EventType::Team => Event::Team,
            EventType::TeamAdd => Event::TeamAdd,
            EventType::Watch => Event::Watch,
        };
        Ok(event)
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

#[derive(Debug, Deserialize)]
pub struct PushEvent {
    #[serde(rename = "ref")]
    git_ref: String,
    before: Oid,
    after: Oid,
    pusher: Pusher,
    created: bool,
    deleted: bool,
    forced: bool,
    base_ref: Option<String>,
    compare: String,
    commits: Vec<Commit>,
    head_commit: Option<Commit>,
    repository: Repository,
    sender: User,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CheckRunEventAction {
    Created,
    Rerequested,
    Completed,
    RequestedAction,
}

#[derive(Debug, Deserialize)]
pub struct RequestedAction {
    identifier: String,
}

#[derive(Debug, Deserialize)]
pub struct CheckRunEvent {
    action: CheckRunEventAction,
    check_run: CheckRun,
    requested_action: Option<RequestedAction>,
    repository: Repository,
    sender: User,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CheckSuiteEventAction {
    Completed,
    Requested,
    Rerequested,
}

#[derive(Debug, Deserialize)]
pub struct CheckSuiteEvent {
    action: CheckSuiteEventAction,
    check_run: CheckSuite,
    repository: Repository,
    sender: User,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum IssueCommentEventAction {
    Created,
    Edited,
    Deleted,
}

#[derive(Debug, Deserialize)]
pub struct IssueCommentEvent {
    action: IssueCommentEventAction,
    // changes: // If action is Edited
    issue: Issue,
    comment: Comment,
    repository: Repository,
    sender: User,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum IssueEventAction {
    Opened,
    Edited,
    Deleted,
    Pinned,
    Unpinned,
    Closed,
    Reopened,
    Assigned,
    Unassigned,
    Labeled,
    Unlabeled,
    Locked,
    Unlocked,
    Transferred,
    Milestoned,
    Demilestoned,
}

#[derive(Debug, Deserialize)]
pub struct IssueEvent {
    action: IssueEventAction,
    // changes: // If action is Edited
    issue: Issue,
    assignee: Option<User>,
    label: Option<Label>,
    repository: Repository,
    sender: User,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum StatusEventState {
    Pending,
    Success,
    Failure,
    Error,
}
#[derive(Debug, Deserialize)]
pub struct StatusEvent {
    sha: Oid,
    state: StatusEventState,
    description: Option<String>,
    target_url: Option<String>,
    // branches: ???,
    // commit: ???,
}

#[cfg(test)]
mod test {
    use super::{IssueCommentEvent, IssueEvent, PushEvent, StatusEvent};

    #[test]
    fn push_event() {
        const PUSH_JSON: &str = include_str!("../test-input/push-event.json");
        let _push: PushEvent = serde_json::from_str(PUSH_JSON).unwrap();
    }

    #[test]
    fn issue_comment_event() {
        const JSON: &str = include_str!("../test-input/issue-comment-event.json");
        let _: IssueCommentEvent = serde_json::from_str(JSON).unwrap();
    }

    #[test]
    fn issue_event() {
        const JSON: &str = include_str!("../test-input/issue-event.json");
        let _: IssueEvent = serde_json::from_str(JSON).unwrap();
    }

    #[test]
    fn status_event() {
        const JSON: &str = include_str!("../test-input/status-event.json");
        let _: StatusEvent = serde_json::from_str(JSON).unwrap();
    }
}
