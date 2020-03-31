use super::{
    CheckRun, CheckSuite, Comment, Commit, DateTime, Hook, Issue, Key, Label, Milestone, Oid,
    PullRequest, Pusher, Repository, Review, ReviewComment, Team, User,
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
    Release,
    Repository,
    RepositoryDispatch,
    RepositoryImport,
    RepositoryVulnerabilityAlert,
    SecurityAdvisory,
    Sponsorship,
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
            "release" => Ok(Release),
            "repository_dispatch" => Ok(RepositoryDispatch),
            "repository" => Ok(Repository),
            "repository_import" => Ok(RepositoryImport),
            "repository_vulnerability_alert" => Ok(RepositoryVulnerabilityAlert),
            "security_advisory" => Ok(SecurityAdvisory),
            "sponsorship" => Ok(Sponsorship),
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
    Label(LabelEvent),
    MarketplacePurchase(MarketplacePurchaseEvent),
    Member(MemberEvent),
    Membership(MembershipEvent),
    Meta(MetaEvent),
    Milestone(MilestoneEvent),
    Organization(OrganizationEvent),
    OrgBlock(OrgBlockEvent),
    Package(PackageEvent),
    PageBuild(PageBuildEvent),
    Ping(PingEvent),
    ProjectCard(ProjectCardEvent),
    ProjectColumn(ProjectColumnEvent),
    Project(ProjectEvent),
    Public(PublicEvent),
    PullRequest(PullRequestEvent),
    PullRequestReview(PullRequestReviewEvent),
    PullRequestReviewComment(PullRequestReviewCommentEvent),
    Push(PushEvent),
    Release(ReleaseEvent),
    Repository(RepositoryEvent),
    RepositoryDispatch(RepositoryDispatchEvent),
    RepositoryImport(RepositoryImportEvent),
    RepositoryVulnerabilityAlert(RepositoryVulnerabilityAlertEvent),
    SecurityAdvisory(SecurityAdvisoryEvent),
    Sponsorship(SponsorshipEvent),
    Star(StarEvent),
    Status(StatusEvent),
    Team(TeamEvent),
    TeamAdd(TeamAddEvent),
    Watch(WatchEvent),
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
            EventType::Label => Event::Label(serde_json::from_slice(json)?),
            EventType::MarketplacePurchase => {
                Event::MarketplacePurchase(serde_json::from_slice(json)?)
            }
            EventType::Member => Event::Member(serde_json::from_slice(json)?),
            EventType::Membership => Event::Membership(serde_json::from_slice(json)?),
            EventType::Meta => Event::Meta(serde_json::from_slice(json)?),
            EventType::Milestone => Event::Milestone(serde_json::from_slice(json)?),
            EventType::Organization => Event::Organization(serde_json::from_slice(json)?),
            EventType::OrgBlock => Event::OrgBlock(serde_json::from_slice(json)?),
            EventType::Package => Event::Package(serde_json::from_slice(json)?),
            EventType::PageBuild => Event::PageBuild(serde_json::from_slice(json)?),
            EventType::Ping => Event::Ping(serde_json::from_slice(json)?),
            EventType::ProjectCard => Event::ProjectCard(serde_json::from_slice(json)?),
            EventType::ProjectColumn => Event::ProjectColumn(serde_json::from_slice(json)?),
            EventType::Project => Event::Project(serde_json::from_slice(json)?),
            EventType::Public => Event::Public(serde_json::from_slice(json)?),
            EventType::PullRequest => Event::PullRequest(serde_json::from_slice(json)?),
            EventType::PullRequestReview => Event::PullRequestReview(serde_json::from_slice(json)?),
            EventType::PullRequestReviewComment => {
                Event::PullRequestReviewComment(serde_json::from_slice(json)?)
            }
            EventType::Push => Event::Push(serde_json::from_slice(json)?),
            EventType::Release => Event::Release(serde_json::from_slice(json)?),
            EventType::Repository => Event::Repository(serde_json::from_slice(json)?),
            EventType::RepositoryDispatch => {
                Event::RepositoryDispatch(serde_json::from_slice(json)?)
            }
            EventType::RepositoryImport => Event::RepositoryImport(serde_json::from_slice(json)?),
            EventType::RepositoryVulnerabilityAlert => {
                Event::RepositoryVulnerabilityAlert(serde_json::from_slice(json)?)
            }
            EventType::SecurityAdvisory => Event::SecurityAdvisory(serde_json::from_slice(json)?),
            EventType::Sponsorship => Event::Sponsorship(serde_json::from_slice(json)?),
            EventType::Star => Event::Star(serde_json::from_slice(json)?),
            EventType::Status => Event::Status(serde_json::from_slice(json)?),
            EventType::Team => Event::Team(serde_json::from_slice(json)?),
            EventType::TeamAdd => Event::TeamAdd(serde_json::from_slice(json)?),
            EventType::Watch => Event::Watch(serde_json::from_slice(json)?),
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
            Event::Label(_) => EventType::Label,
            Event::MarketplacePurchase(_) => EventType::MarketplacePurchase,
            Event::Member(_) => EventType::Member,
            Event::Membership(_) => EventType::Membership,
            Event::Meta(_) => EventType::Meta,
            Event::Milestone(_) => EventType::Milestone,
            Event::Organization(_) => EventType::Organization,
            Event::OrgBlock(_) => EventType::OrgBlock,
            Event::Package(_) => EventType::Package,
            Event::PageBuild(_) => EventType::PageBuild,
            Event::Ping(_) => EventType::Ping,
            Event::ProjectCard(_) => EventType::ProjectCard,
            Event::ProjectColumn(_) => EventType::ProjectColumn,
            Event::Project(_) => EventType::Project,
            Event::Public(_) => EventType::Public,
            Event::PullRequest(_) => EventType::PullRequest,
            Event::PullRequestReview(_) => EventType::PullRequestReview,
            Event::PullRequestReviewComment(_) => EventType::PullRequestReviewComment,
            Event::Push(_) => EventType::Push,
            Event::Release(_) => EventType::Release,
            Event::Repository(_) => EventType::Repository,
            Event::RepositoryDispatch(_) => EventType::RepositoryDispatch,
            Event::RepositoryImport(_) => EventType::RepositoryImport,
            Event::RepositoryVulnerabilityAlert(_) => EventType::RepositoryVulnerabilityAlert,
            Event::SecurityAdvisory(_) => EventType::SecurityAdvisory,
            Event::Sponsorship(_) => EventType::Sponsorship,
            Event::Star(_) => EventType::Star,
            Event::Status(_) => EventType::Status,
            Event::Team(_) => EventType::Team,
            Event::TeamAdd(_) => EventType::TeamAdd,
            Event::Watch(_) => EventType::Watch,
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

/// The representation of an edit made on a label
#[derive(Clone, Debug, Deserialize)]
pub struct LabelChange {
    pub name: Option<OldContents>,
    pub color: Option<OldContents>,
}

/// GitHub API docs: https://developer.github.com/v3/activity/events/types/#labelevent
#[derive(Clone, Debug, Deserialize)]
pub struct LabelEvent {
    /// The action that was performed. Can be created, edited, or deleted
    pub action: String,
    pub label: Label,
    pub changes: Option<LabelChange>, // If action is Edited

    // Populated by Webhook events
    pub repository: Repository,
    pub sender: User,
}

/// GitHub API docs: https://developer.github.com/v3/activity/events/types/#marketplacepurchaseevent
#[derive(Clone, Debug, Deserialize)]
pub struct MarketplacePurchaseEvent {
    /// The action performed. Can be purchased, cancelled, pending_change, pending_change_cancelled, or changed
    pub action: String,
    //pub marketplace_purchase: MarketplacePurchase, //TODO add type
    pub effective_date: DateTime,

    // Populated by Webhook events
    pub sender: User,
}

/// Triggered when a user is added as a collaborator to a repository.
/// The Webhook event name is "member".
///
/// GitHub API docs: https://developer.github.com/v3/activity/events/types/#memberevent
#[derive(Clone, Debug, Deserialize)]
pub struct MemberEvent {
    /// The action that was performed. Can be "added", "removed", or "edited"
    pub action: String,
    pub member: User,
    // changes TODO

    // Populated by Webhook events
    pub repository: Repository,
    pub sender: User,
}

/// Triggered when a user is added or removed from a team.
/// The Webhook event name is "membership".
///
/// GitHub API docs: https://developer.github.com/v3/activity/events/types/#membershipevent
#[derive(Clone, Debug, Deserialize)]
pub struct MembershipEvent {
    /// The action that was performed. Can be "added", "removed"
    pub action: String,
    pub scope: String,
    pub member: User,
    pub team: Team,

    // Populated by Webhook events
    pub repository: Repository,
    pub sender: User,
}

/// Triggered when the webhook that this event is configured on is deleted.
/// This event will only listen for changes to the particular hook the event is installed on.
/// Therefore, it must be selected for each hook that you'd like to receive meta events for.
///
/// GitHub API docs: https://developer.github.com/v3/activity/events/types/#metaevent
#[derive(Clone, Debug, Deserialize)]
pub struct MetaEvent {
    /// The action that was performed. Can be "deleted"
    pub action: String,
    pub hook_id: u64,
    pub hook: Hook,

    // Populated by Webhook events
    pub repository: Repository,
    pub sender: User,
}

/// MilestoneEvent is triggered when a milestone is created, closed, opened, edited, or deleted.
///
/// GitHub API docs: https://developer.github.com/v3/activity/events/types/#milestoneevent
#[derive(Clone, Debug, Deserialize)]
pub struct MilestoneEvent {
    /// Action is the action that was performed. Possible values are:
    /// "created", "closed", "opened", "edited", "deleted"
    pub action: String,
    pub milestone: Milestone,
    //pub changes: //TODO add type

    // Populated by Webhook events
    pub repository: Repository,
    pub sender: User,
}

/// OrganizationEvent is triggered when an organization is deleted and renamed, and when a user is added,
/// removed, or invited to an organization.
/// Events of this type are not visible in timelines. These events are only used to trigger organization hooks.
///
/// GitHub API docs: https://developer.github.com/v3/activity/events/types/#organizationevent
#[derive(Clone, Debug, Deserialize)]
pub struct OrganizationEvent {
    // Action is the action that was performed.
    // Possible values are: "deleted", "renamed", "member_added", "member_removed", or "member_invited".
    pub action: String,
    //pub invitation: //TODO add type
    //pub membership: //TODO add type
    //pub organization: //TODO add type
    pub sender: User,
}

/// OrgBlockEvent is triggered when an organization blocks or unblocks a user.
/// The Webhook event name is "org_block".
///
/// GitHub API docs: https://developer.github.com/v3/activity/events/types/#orgblockevent
#[derive(Clone, Debug, Deserialize)]
pub struct OrgBlockEvent {
    // Action is the action that was performed.
    // Possible values are: "blocked" or "unblocked"
    pub action: String,
    pub blocked_user: User,
    //pub organization: Organization //TODO add type
    pub sender: User,
}

/// Triggered when a package version is published or updated in GitHub Packages.
///
/// GitHub API docs: https://developer.github.com/v3/activity/events/types/#packageevent
#[derive(Clone, Debug, Deserialize)]
pub struct PackageEvent {
    // Action is the action that was performed.
    // Possible values are: "published" or "updated"
    pub action: String,
    //pub package: Package, //TODO add type
    pub repository: Repository,
    //pub organization: Organization //TODO add type
    pub sender: User,
}

/// Represents an attempted build of a GitHub Pages site, whether successful or not
///
/// GitHub API docs: https://developer.github.com/v3/activity/events/types/#pagebuildevent
#[derive(Clone, Debug, Deserialize)]
pub struct PageBuildEvent {
    //pub build: PagesBuild, //TODO add type
    pub id: u64,

    // Populated by Webhook events
    pub repository: Repository,
    pub sender: User,
}

/// PingEvent is triggered when a Webhook is added to GitHub.
///
/// GitHub API docs: https://developer.github.com/webhooks/#ping-event
#[derive(Clone, Debug, Deserialize)]
pub struct PingEvent {
    pub zen: String,
    pub hook_id: u64,
    pub hook: Hook,
    pub repository: Repository,
    pub sender: User,
}

/// Triggered when a project card is created, edited, moved, converted to an issue, or deleted
///
/// GitHub API docs: https://developer.github.com/v3/activity/events/types/#projectcardevent
#[derive(Clone, Debug, Deserialize)]
pub struct ProjectCardEvent {
    /// The action performed on the project card. Can be created, edited, moved, converted, or deleted
    pub action: String,
    //pub changes: ProjectChange //TODO add type
    //pub project_card: ProjectCard //TODO add type
    /// The id of the card that this card now follows if the action was "moved".
    /// Will be None if it is the first card in a column.
    pub after_id: Option<u64>,

    // Populated by Webhook events
    pub repository: Repository,
    pub sender: User,
}

/// Triggered when a project column is created, updated, moved, or deleted
///
/// GitHub API docs: https://developer.github.com/v3/activity/events/types/#projectcolumnevent
#[derive(Clone, Debug, Deserialize)]
pub struct ProjectColumnEvent {
    /// The action that was performed on the project column. Can be one of created, edited, moved or deleted
    pub action: String,
    //pub changes: ProjectCardChange //TODO add type
    //pub project_column: ProjectColumn //TODO add type
    /// The id of the column that this column now follows if the action was "moved".
    /// Will be None if it is the first column in a project.
    pub after_id: Option<u64>,

    // Populated by Webhook events
    pub repository: Repository,
    pub sender: User,
}

/// Triggered when a project is created, updated, closed, reopened, or deleted
///
/// GitHub API docs: https://developer.github.com/v3/activity/events/types/#projectevent
#[derive(Clone, Debug, Deserialize)]
pub struct ProjectEvent {
    /// The action that was performed on the project. Can be one of created, edited, closed, reopened, or deleted
    pub action: String,
    //pub changes: ProjectChanges, //TODO add type
    //pub project: Project, //TODO add type

    // Populated by Webhook events
    pub repository: Repository,
    pub sender: User,
}

/// Triggered when a private repository is made public.
/// "Without a doubt: the best GitHub event."
///
/// GitHub API docs: https://developer.github.com/v3/activity/events/types/#publicevent
#[derive(Clone, Debug, Deserialize)]
pub struct PublicEvent {
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

/// Triggered when a pull request is assigned, unassigned, labeled, unlabeled, opened, edited,
/// closed, reopened, synchronize, ready_for_review, locked, unlocked or when a pull request review
/// is requested or removed.
///
/// GitHub API docs: https://developer.github.com/v3/activity/events/types/#pullrequestevent
#[derive(Clone, Debug, Deserialize)]
pub struct PullRequestEvent {
    pub action: PullRequestEventAction,
    pub number: u64,
    pub pull_request: PullRequest,
    //pub changes: // TODO add type
    /// RequestedReviewer is populated in "review_requested", "review_request_removed" event
    /// deliveries.  A request affecting multiple reviewers at once is split into multiple such
    /// event deliveries, each with a single, different RequestedReviewer.
    pub requested_reviwer: Option<User>,
    /// In the event that a team is requested instead of a user, "requested_team" gets sent in
    /// place of "requested_user" with the same delivery behavior.
    pub requested_team: Option<Team>,

    // Populated by Webhook events
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

/// Triggered when a pull request review is submitted into a non-pending state, the body is edited,
/// or the review is dismissed
///
/// GitHub API docs: https://developer.github.com/v3/activity/events/types/#pullrequestreviewevent
#[derive(Clone, Debug, Deserialize)]
pub struct PullRequestReviewEvent {
    pub action: PullRequestReviewEventAction,
    pub review: Review,
    pub pull_request: PullRequest,
    //pub changes: // TODO add type

    // Populated by Webhook events
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

/// Triggered when a comment on a pull request's unified diff is created, edited, or deleted (in
/// the Files Changed tab)
///
/// GitHub API docs: https://developer.github.com/v3/activity/events/types/#pullrequestreviewcommentevent
#[derive(Clone, Debug, Deserialize)]
pub struct PullRequestReviewCommentEvent {
    pub action: PullRequestReviewCommentEventAction,
    pub comment: ReviewComment,
    pub pull_request: PullRequest,
    //pub changes: // TODO add type

    // Populated by Webhook events
    pub repository: Repository,
    pub sender: User,
}

/// Triggered on a push to a repository branch. Branch pushes and repository tag pushes also
/// trigger webhook push events.
///
/// GitHub API docs: https://developer.github.com/v3/activity/events/types/#pushevent
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

    // Populated by Webhook events
    pub repository: Repository,
    pub sender: User,
}

/// Triggered when a release is published, unpublished, created, edited, deleted, or prereleased
///
/// GitHub API docs: https://developer.github.com/v3/activity/events/types/#releaseevent
#[derive(Clone, Debug, Deserialize)]
pub struct ReleaseEvent {
    /// The action that was performed. Currently, can be published, unpublished, created, edited,
    /// deleted, or prereleased
    pub action: String,
    //pub changes: // TODO add type
    //pub release: Release, // TODO add type

    // Populated by Webhook events
    pub repository: Repository,
    pub sender: User,
}

/// Triggered when a GitHub App sends a POST request to the "Create a repository dispatch event"
/// endpoint.
///
/// GitHub API docs: https://developer.github.com/v3/activity/events/types/#repositorydispatchevent
#[derive(Clone, Debug, Deserialize)]
pub struct RepositoryDispatchEvent {
    pub action: String,
    pub branch: String,
    //pub client_payload: // TODO add type

    // Populated by Webhook events
    pub repository: Repository,
    pub sender: User,
    //pub organization: Organization, //TODO add type
    //pub installation: Installation, //TODO add type
}

/// Triggered when a repository is created, archived, unarchived, renamed, edited, transferred,
/// made public, or made private. Organization hooks are also triggered when a repository is
/// deleted
///
/// GitHub API docs: https://developer.github.com/v3/activity/events/types/#repositoryevent
#[derive(Clone, Debug, Deserialize)]
pub struct RepositoryEvent {
    /// The action that was performed. This can be one of created, deleted (organization hooks
    /// only), archived, unarchived, edited, renamed, transferred, publicized, or privatized
    pub action: String,

    // Populated by Webhook events
    pub repository: Repository,
    pub sender: User,
    //pub organization: Organization, //TODO add type
    //pub installation: Installation, //TODO add type
}

/// Triggered when a successful, cancelled, or failed repository import finishes for a GitHub
/// organization or a personal repository. To receive this event for a personal repository, you
/// must create an empty repository prior to the import.
///
/// GitHub API docs: https://developer.github.com/v3/activity/events/types/#repositoryimportevent
#[derive(Clone, Debug, Deserialize)]
pub struct RepositoryImportEvent {
    /// The final state of the import. This can be one of success, cancelled, or failure
    pub status: String,

    // Populated by Webhook events
    pub repository: Repository,
    pub sender: User,
    //pub organization: Organization, //TODO add type
    //pub installation: Installation, //TODO add type
}

#[derive(Clone, Debug, Deserialize)]
pub struct Alert {
    pub id: u64,
    pub affected_range: String,
    pub affected_package_name: String,
    pub external_reference: String,
    pub external_identifier: String,
    pub fixed_in: String,
}

/// Triggered when a security alert is created, dismissed, or resolved.
///
/// GitHub API docs: https://developer.github.com/v3/activity/events/types/#repositoryvulnerabilityalertevent
#[derive(Clone, Debug, Deserialize)]
pub struct RepositoryVulnerabilityAlertEvent {
    /// The action that was performed. This can be one of create, dismiss, or resolve.
    pub action: String,
    pub alert: Alert,

    // Populated by Webhook events
    pub repository: Repository,
    pub sender: User,
    //pub organization: Organization, //TODO add type
    //pub installation: Installation, //TODO add type
}

/// Triggered when a new security advisory is published, updated, or withdrawn. A security advisory
/// provides information about security-related vulnerabilities in software on GitHub.
///
/// GitHub API docs: https://developer.github.com/v3/activity/events/types/#securityadvisoryevent
#[derive(Clone, Debug, Deserialize)]
pub struct SecurityAdvisoryEvent {
    /// The action that was performed. The action can be one of published, updated, or performed
    /// for all new events
    pub action: String,
    //pub security_advisory: SecurityAdvisory, //TODO add type
}

/// Triggered anytime a sponsorship listing is created, cancelled, edited, or has a tier_changed,
/// pending_cancellation, or pending_tier_change
///
/// GitHub API docs: https://developer.github.com/v3/activity/events/types/#sponsorshipevent
#[derive(Clone, Debug, Deserialize)]
pub struct SponsorshipEvent {
    /// The action that was performed. This can be one of created, cancelled, edited, tier_changed,
    /// pending_cancellation, or pending_tier_change
    pub action: String,
    //pub sponsorship: Sponsorship, //TODO add type
    pub sender: User,
}

/// Triggered when a star is added or removed from a repository.
///
/// GitHub API docs: https://developer.github.com/v3/activity/events/types/#starevent
#[derive(Clone, Debug, Deserialize)]
pub struct StarEvent {
    /// The action performed. Can be created or deleted.
    pub action: String,
    pub starred_at: DateTime,

    // Populated by Webhook events
    pub repository: Repository,
    pub sender: User,
    //pub organization: Organization, //TODO add type
    //pub installation: Installation, //TODO add type
}

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum StatusEventState {
    Pending,
    Success,
    Failure,
    Error,
}

/// Triggered when the status of a Git commit changes
///
/// GitHub: API docs: https://developer.github.com/v3/activity/events/types/#statusevent
#[derive(Clone, Debug, Deserialize)]
pub struct StatusEvent {
    pub sha: Oid,
    pub state: StatusEventState,
    pub description: Option<String>,
    pub target_url: Option<String>,
    // branches: ???,
    // commit: ???,
    pub repository: Repository,
    pub sender: User,
}

/// Triggered when an organization's team is created, deleted, edited, added_to_repository, or
/// removed_from_repository
///
/// GitHub API docs: https://developer.github.com/v3/activity/events/types/#teamevent
#[derive(Clone, Debug, Deserialize)]
pub struct TeamEvent {
    /// The action that was performed. Can be one of created, deleted, edited, added_to_repository,
    /// or removed_from_repository.
    pub action: String,
    pub team: Team,
    //pub changes: //TODO add type
    pub repository: Option<Repository>,

    // Populated by Webhook events
    pub sender: User,
    //pub organization: Organization, //TODO add type
    //pub installation: Installation, //TODO add type
}

/// Triggered when a repository is added to a team
///
/// GitHub API docs: https://developer.github.com/v3/activity/events/types/#teamaddevent
#[derive(Clone, Debug, Deserialize)]
pub struct TeamAddEvent {
    pub team: Team,
    pub repository: Repository,

    // Populated by Webhook events
    pub sender: User,
    //pub organization: Organization, //TODO add type
    //pub installation: Installation, //TODO add type
}

// Triggered when a user is created or deleted. Only global webhooks can subscribe to this event.
// This event is not available in the Events API.
//
// GitHub API docs: https://developer.github.com/enterprise/v3/activity/events/types/#userevent-enterprise
//#[derive(Clone, Debug, Deserialize)]
//pub struct UserEvent {
//    /// The action that was performed. "created" or "deleted"
//    pub action: String,
//    pub user: User,
//
//    //pub enterprise: Enterprise, // TODO add type
//    pub sender: User,
//}

/// Triggered when someone stars a repository. This event is not related to watching a repository.
///
/// GitHub API docs: https://developer.github.com/v3/activity/events/types/#watchevent
#[derive(Clone, Debug, Deserialize)]
pub struct WatchEvent {
    /// The action that was performed. Currently, can only be started
    pub action: String,

    // Populated by Webhook events
    pub repository: Repository,
    pub sender: User,
    //pub organization: Organization, //TODO add type
    //pub installation: Installation, //TODO add type
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
