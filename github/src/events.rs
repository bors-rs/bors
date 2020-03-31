use super::{
    CheckRun, CheckSuite, Comment, Commit, Hook, Issue, Key, Label, Oid, PullRequest, Pusher,
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
    // DEPRICATED: Download
    // DEPRICATED: Follow
    Fork,
    // DEPRICATED: ForkApply
    GithubAppAuthorization,
    // DEPRICATED: Gist
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
            "fork" => Ok(Fork),
            "github_app_authorization" => Ok(GithubAppAuthorization),
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
    CommitComment(CommitCommentEvent),
    ContentReference(ContentReferenceEvent),
    Create(CreateEvent),
    Delete(DeleteEvent),
    DeployKey(DeployKeyEvent),
    Deployment(DeploymentEvent),
    DeploymentStatus(DeploymentStatusEvent),
    Fork(ForkEvent),
    GithubAppAuthorization(GithubAppAuthorizationEvent),
    Gollum(GollumEvent),
    Installation(InstallationEvent),
    InstallationRepositories(InstallationRepositoriesEvent),
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
            EventType::CommitComment => Event::CommitComment(serde_json::from_slice(json)?),
            EventType::ContentReference => Event::ContentReference(serde_json::from_slice(json)?),
            EventType::Create => Event::Create(serde_json::from_slice(json)?),
            EventType::Delete => Event::Delete(serde_json::from_slice(json)?),
            EventType::DeployKey => Event::DeployKey(serde_json::from_slice(json)?),
            EventType::Deployment => Event::Deployment(serde_json::from_slice(json)?),
            EventType::DeploymentStatus => Event::DeploymentStatus(serde_json::from_slice(json)?),
            EventType::Fork => Event::Fork(serde_json::from_slice(json)?),
            EventType::GithubAppAuthorization => {
                Event::GithubAppAuthorization(serde_json::from_slice(json)?)
            }
            EventType::Gollum => Event::Gollum(serde_json::from_slice(json)?),
            EventType::Installation => Event::Installation(serde_json::from_slice(json)?),
            EventType::InstallationRepositories => {
                Event::InstallationRepositories(serde_json::from_slice(json)?)
            }
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

    pub fn event_type(&self) -> EventType {
        match &self {
            Event::CheckRun(_) => EventType::CheckRun,
            Event::CheckSuite(_) => EventType::CheckSuite,
            Event::CommitComment(_) => EventType::CommitComment,
            Event::ContentReference(_) => EventType::ContentReference,
            Event::Create(_) => EventType::Create,
            Event::Delete(_) => EventType::Delete,
            Event::DeployKey(_) => EventType::DeployKey,
            Event::Deployment(_) => EventType::Deployment,
            Event::DeploymentStatus(_) => EventType::DeploymentStatus,
            Event::Fork(_) => EventType::Fork,
            Event::GithubAppAuthorization(_) => EventType::GithubAppAuthorization,
            Event::Gollum(_) => EventType::Gollum,
            Event::Installation(_) => EventType::Installation,
            Event::InstallationRepositories(_) => EventType::InstallationRepositories,
            Event::IssueComment(_) => EventType::IssueComment,
            Event::Issues(_) => EventType::Issues,
            Event::Label => EventType::Label,
            Event::MarketplacePurchase => EventType::MarketplacePurchase,
            Event::Member => EventType::Member,
            Event::Membership => EventType::Membership,
            Event::Meta => EventType::Meta,
            Event::Milestone => EventType::Milestone,
            Event::Organization => EventType::Organization,
            Event::OrgBlock => EventType::OrgBlock,
            Event::Package => EventType::Package,
            Event::PageBuild => EventType::PageBuild,
            Event::Ping(_) => EventType::Ping,
            Event::ProjectCard => EventType::ProjectCard,
            Event::ProjectColumn => EventType::ProjectColumn,
            Event::Project => EventType::Project,
            Event::Public => EventType::Public,
            Event::PullRequest(_) => EventType::PullRequest,
            Event::PullRequestReview(_) => EventType::PullRequestReview,
            Event::PullRequestReviewComment(_) => EventType::PullRequestReviewComment,
            Event::Push => EventType::Push,
            Event::RegistryPackage => EventType::RegistryPackage,
            Event::Release => EventType::Release,
            Event::Repository => EventType::Repository,
            Event::RepositoryDispatch => EventType::RepositoryDispatch,
            Event::RepositoryImport => EventType::RepositoryImport,
            Event::RepositoryVulnerabilityAlert => EventType::RepositoryVulnerabilityAlert,
            Event::SecurityAdvisory => EventType::SecurityAdvisory,
            Event::Star => EventType::Star,
            Event::Status(_) => EventType::Status,
            Event::Team => EventType::Team,
            Event::TeamAdd => EventType::TeamAdd,
            Event::Watch => EventType::Watch,
        }
    }
}

/// The Action performed by a `CheckRunEvent`
#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CheckRunEventAction {
    Created,
    Rerequested,
    Completed,
    RequestedAction,
}

/// `RequestedAction` is included in a `CheckRunEvent` when a user has invoked an action,
/// i.e. when the `CheckRunEventAction` type is `RequestedAction`.
#[derive(Clone, Debug, Deserialize)]
pub struct RequestedAction {
    pub identifier: String,
}

/// GitHub API docs: https://developer.github.com/v3/activity/events/types/#checkrunevent
#[derive(Clone, Debug, Deserialize)]
pub struct CheckRunEvent {
    pub action: CheckRunEventAction,
    pub check_run: CheckRun,
    pub requested_action: Option<RequestedAction>,

    // Populated by Webhook events
    pub repository: Repository,
    pub sender: User,
}

/// The Action performed by a `CheckSuiteEvent`
#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CheckSuiteEventAction {
    Completed,
    Requested,
    Rerequested,
}

/// GitHub API docs: https://developer.github.com/v3/activity/events/types/#checksuiteevent
#[derive(Clone, Debug, Deserialize)]
pub struct CheckSuiteEvent {
    pub action: CheckSuiteEventAction,
    pub check_suite: CheckSuite,

    // Populated by Webhook events
    pub repository: Repository,
    pub sender: User,
}

/// GitHub API docs: https://developer.github.com/v3/activity/events/types/#commitcommentevent
#[derive(Clone, Debug, Deserialize)]
pub struct CommitCommentEvent {
    pub comment: Comment,

    // Populated by Webhook events
    pub repository: Repository,
    pub sender: User,
}

/// GitHub API docs: https://developer.github.com/v3/activity/events/types/#contentreferenceevent
#[derive(Clone, Debug, Deserialize)]
pub struct ContentReferenceEvent {
    pub action: String,
    // pub content_reference: ContentReference, //TODO add type

    // Populated by Webhook events
    pub repository: Repository,
    pub sender: User,
}

/// GitHub API docs: https://developer.github.com/v3/activity/events/types/#createevent
#[derive(Clone, Debug, Deserialize)]
pub struct CreateEvent {
    /// The object that was created. Possible values are: "repository", "branch", "tag"
    pub ref_type: String,
    #[serde(rename = "ref")]
    pub git_ref: String,
    pub master_branch: String,
    pub description: Option<String>,
    pub pusher_type: Option<String>,

    // Populated by Webhook events
    pub repository: Repository,
    pub sender: User,
}

/// GitHub API docs: https://developer.github.com/v3/activity/events/types/#deleteevent
#[derive(Clone, Debug, Deserialize)]
pub struct DeleteEvent {
    /// The object that was created. Possible values are: "branch", "tag"
    pub ref_type: String,
    #[serde(rename = "ref")]
    pub git_ref: String,
    pub pusher_type: Option<String>,

    // Populated by Webhook events
    pub repository: Repository,
    pub sender: User,
}

/// GitHub API docs: https://developer.github.com/v3/activity/events/types/#deploykeyevent
#[derive(Clone, Debug, Deserialize)]
pub struct DeployKeyEvent {
    /// The action performed. Possible values are: "created" or "deleted"
    pub action: String,
    pub key: Key,

    // Populated by Webhook events
    pub repository: Repository,
    pub sender: User,
}

/// GitHub API docs: https://developer.github.com/v3/activity/events/types/#deploymentevent
#[derive(Clone, Debug, Deserialize)]
pub struct DeploymentEvent {
    /// The action performed. Possible values are: "created"
    pub action: String,
    //pub deployment: Deployment, //TODO add type
    pub repository: Repository,

    // Populated by Webhook events
    pub sender: User,
}

/// GitHub API docs: https://developer.github.com/v3/activity/events/types/#deploymentstatusevent
#[derive(Clone, Debug, Deserialize)]
pub struct DeploymentStatusEvent {
    /// The action performed. Possible values are: "created"
    pub action: String,
    //pub deployment_status: DeploymentStatus, //TODO add type
    //pub deployment: Deployment, //TODO add type
    pub repository: Repository,

    // Populated by Webhook events
    pub sender: User,
}

/// GitHub API docs: https://developer.github.com/v3/activity/events/types/#forkevent
#[derive(Clone, Debug, Deserialize)]
pub struct ForkEvent {
    // The newly created fork
    pub forkee: Repository,

    // Populated by Webhook events
    pub repository: Repository,
    pub sender: User,
}

/// An event triggered when a user's authorization for a GitHub Application is revoked
///
/// GitHub API docs: https://developer.github.com/v3/activity/events/types/#githubappauthorizationevent
#[derive(Clone, Debug, Deserialize)]
pub struct GithubAppAuthorizationEvent {
    /// The action performed. Possible values are: "revoked"
    pub action: String,

    // Populated by Webhook events
    pub sender: User,
}

// Page represents a single Wiki page.
#[derive(Clone, Debug, Deserialize)]
pub struct Page {
    page_name: String,
    title: String,
    summary: Option<String>,
    action: String,
    sha: Oid,
    html_url: String,
}

/// GitHub API docs: https://developer.github.com/v3/activity/events/types/#gollumevent
#[derive(Clone, Debug, Deserialize)]
pub struct GollumEvent {
    pub pages: Vec<Page>,

    // Populated by Webhook events
    pub repository: Repository,
    pub sender: User,
}

/// GitHub API docs: https://developer.github.com/v3/activity/events/types/#installationevent
#[derive(Clone, Debug, Deserialize)]
pub struct InstallationEvent {
    /// The action performed. Possible values are: "created", "deleted", "new_permissions_accepted"
    pub action: String,
    //pub installation: Installation, //TODO add type
    pub sender: User,
}

/// GitHub API docs: https://developer.github.com/v3/activity/events/types/#installationrepositoriesevent
#[derive(Clone, Debug, Deserialize)]
pub struct InstallationRepositoriesEvent {
    /// The action performed. Possible values are: "added", "removed"
    pub action: String,
    //pub installation: Installation, //TODO add type
    /// The choice of repositories the installation is on. Can be either "selected" or "all"
    pub repository_selection: String,
    pub repositories_added: Vec<Repository>,
    pub repositories_removed: Vec<Repository>,

    pub sender: User,
}

#[derive(Clone, Debug, Deserialize)]
pub struct OldContents {
    pub from: String,
}

/// The representation of an edit made on an issue, pull request, or comment
#[derive(Clone, Debug, Deserialize)]
pub struct EditChange {
    pub title: Option<OldContents>,
    pub body: Option<OldContents>,
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

/// GitHub API docs: https://developer.github.com/v3/activity/events/types/#issuecommentevent
#[derive(Clone, Debug, Deserialize)]
pub struct IssueCommentEvent {
    pub action: IssueCommentEventAction,
    pub changes: Option<EditChange>, // If action is Edited
    pub issue: Issue,
    pub comment: Comment,

    // Populated by Webhook events
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

/// GitHub API docs: https://developer.github.com/v3/activity/events/types/#issuesevent
#[derive(Clone, Debug, Deserialize)]
pub struct IssueEvent {
    pub action: IssueEventAction,
    pub changes: Option<EditChange>, // If action is Edited
    pub issue: Issue,
    pub assignee: Option<User>,
    pub label: Option<Label>,

    // Populated by Webhook events
    pub repository: Repository,
    pub sender: User,
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
    ReviewRequest,
    ReviewRequestRemoved,
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
