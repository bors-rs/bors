use super::{
    CheckRun, CheckSuite, Comment, Commit, Hook, Issue, Label, Oid, PullRequest, Pusher,
    Repository, Review, ReviewComment, User,
};
use serde::{de, Deserialize};
use std::str::FromStr;
use thiserror::Error;

#[derive(Clone, Debug)]
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
    Ping,
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
    Wildcard,
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
            "ping" => Ok(Ping),
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
            "*" => Ok(Wildcard),
            _ => Err(ParseEventTypeError),
        }
    }
}

impl<'de> Deserialize<'de> for EventType {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: de::Deserializer<'de>,
    {
        let s = <String>::deserialize(deserializer)?;
        Self::from_str(&s).map_err(de::Error::custom)
    }
}

#[derive(Clone, Debug)]
#[allow(clippy::large_enum_variant)]
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
    IssueComment(IssueCommentEvent),
    Issues(IssueEvent),
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
    Ping(PingEvent),
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
    Status(StatusEvent),
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
            EventType::IssueComment => Event::IssueComment(serde_json::from_slice(json)?),
            EventType::Issues => Event::Issues(serde_json::from_slice(json)?),
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
            EventType::Ping => Event::Ping(serde_json::from_slice(json)?),
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
            EventType::Status => Event::Status(serde_json::from_slice(json)?),
            EventType::Team => Event::Team,
            EventType::TeamAdd => Event::TeamAdd,
            EventType::Watch => Event::Watch,
            // TODO have an error type if we try to De a wildcard event payload since they don't
            // exist
            EventType::Wildcard => unimplemented!(),
        };
        Ok(event)
    }
}

#[derive(Clone, Debug, Deserialize)]
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

#[derive(Clone, Debug, Deserialize)]
pub struct PullRequestEvent {
    pub action: PullRequestEventAction,
    pub number: u64,
    pub pull_request: PullRequest,
    pub repository: Repository,
    pub sender: User,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PullRequestReviewEventAction {
    Submitted,
    Edited,
    Dismissed,
}

impl PullRequestReviewEventAction {
    pub fn is_submitted(&self) -> bool {
        if let PullRequestReviewEventAction::Submitted = self {
            true
        } else {
            false
        }
    }
}

#[derive(Clone, Debug, Deserialize)]
pub struct PullRequestReviewEvent {
    pub action: PullRequestReviewEventAction,
    pub review: Review,
    pub pull_request: PullRequest,
    pub repository: Repository,
    pub sender: User,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PullRequestReviewCommentEventAction {
    Created,
    Edited,
    Deleted,
}

impl PullRequestReviewCommentEventAction {
    pub fn is_created(&self) -> bool {
        if let PullRequestReviewCommentEventAction::Created = self {
            true
        } else {
            false
        }
    }
}

#[derive(Clone, Debug, Deserialize)]
pub struct PullRequestReviewCommentEvent {
    pub action: PullRequestReviewCommentEventAction,
    pub comment: ReviewComment,
    pub pull_request: PullRequest,
    pub repository: Repository,
    pub sender: User,
}

#[derive(Clone, Debug, Deserialize)]
pub struct PushEvent {
    #[serde(rename = "ref")]
    pub git_ref: String,
    pub before: Oid,
    pub after: Oid,
    pub pusher: Pusher,
    pub created: bool,
    pub deleted: bool,
    pub forced: bool,
    pub base_ref: Option<String>,
    pub compare: String,
    pub commits: Vec<Commit>,
    pub head_commit: Option<Commit>,
    pub repository: Repository,
    pub sender: User,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CheckRunEventAction {
    Created,
    Rerequested,
    Completed,
    RequestedAction,
}

#[derive(Clone, Debug, Deserialize)]
pub struct RequestedAction {
    pub identifier: String,
}

#[derive(Clone, Debug, Deserialize)]
pub struct CheckRunEvent {
    pub action: CheckRunEventAction,
    pub check_run: CheckRun,
    pub requested_action: Option<RequestedAction>,
    pub repository: Repository,
    pub sender: User,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CheckSuiteEventAction {
    Completed,
    Requested,
    Rerequested,
}

#[derive(Clone, Debug, Deserialize)]
pub struct CheckSuiteEvent {
    pub action: CheckSuiteEventAction,
    pub check_suite: CheckSuite,
    pub repository: Repository,
    pub sender: User,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum IssueCommentEventAction {
    Created,
    Edited,
    Deleted,
}

impl IssueCommentEventAction {
    pub fn is_created(&self) -> bool {
        if let IssueCommentEventAction::Created = self {
            true
        } else {
            false
        }
    }
}

#[derive(Clone, Debug, Deserialize)]
pub struct IssueCommentEvent {
    pub action: IssueCommentEventAction,
    // changes: // If action is Edited
    pub issue: Issue,
    pub comment: Comment,
    pub repository: Repository,
    pub sender: User,
}

#[derive(Clone, Debug, Deserialize)]
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

#[derive(Clone, Debug, Deserialize)]
pub struct IssueEvent {
    pub action: IssueEventAction,
    // changes: // If action is Edited
    pub issue: Issue,
    pub assignee: Option<User>,
    pub label: Option<Label>,
    pub repository: Repository,
    pub sender: User,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum StatusEventState {
    Pending,
    Success,
    Failure,
    Error,
}

#[derive(Clone, Debug, Deserialize)]
pub struct StatusEvent {
    pub sha: Oid,
    pub state: StatusEventState,
    pub description: Option<String>,
    pub target_url: Option<String>,
    // branches: ???,
    // commit: ???,
}

#[derive(Clone, Debug, Deserialize)]
pub struct PingEvent {
    pub zen: String,
    pub hook_id: u64,
    pub hook: Hook,
    pub repository: Repository,
    pub sender: User,
}

#[cfg(test)]
mod test {
    use super::{
        CheckRunEvent, CheckSuiteEvent, IssueCommentEvent, IssueEvent,
        PullRequestReviewCommentEvent, PullRequestReviewEvent, PushEvent, StatusEvent,
    };

    #[test]
    fn push_event() {
        const PUSH_JSON: &str = include_str!("../test-input/push-event.json");
        let _push: PushEvent = serde_json::from_str(PUSH_JSON).unwrap();
    }

    #[test]
    fn issue_comment_event() {
        const JSON: &str = include_str!("../test-input/issue-comment-event.json");
        let _: Vec<IssueCommentEvent> = serde_json::from_str(JSON).unwrap();
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

    #[test]
    fn check_run_event() {
        const JSON: &str = include_str!("../test-input/check-run-event.json");
        let _: CheckRunEvent = serde_json::from_str(JSON).unwrap();
    }

    #[test]
    fn check_suite_event() {
        const JSON: &str = include_str!("../test-input/check-suite-event.json");
        let _: CheckSuiteEvent = serde_json::from_str(JSON).unwrap();
    }

    #[test]
    fn pull_request_review() {
        const JSON: &str = include_str!("../test-input/pull-request-review-event.json");
        let _: PullRequestReviewEvent = serde_json::from_str(JSON).unwrap();
    }

    #[test]
    fn pull_request_review_comment() {
        const JSON: &str = include_str!("../test-input/pull-request-review-comment-event.json");
        let _: PullRequestReviewCommentEvent = serde_json::from_str(JSON).unwrap();
    }
}
