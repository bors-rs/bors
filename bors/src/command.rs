//! Defines commands which can be asked to be performed

use crate::{
    config::RepoConfig,
    event_processor::{ActivePullRequestContext, CommandContext},
    project_board::ProjectBoard,
    state::{Priority, Status},
    Result,
};
use github::client::NewPullRequest;
use log::info;
use thiserror::Error;

#[derive(Error, Debug)]
#[error("invalid command")]
pub struct ParseCommandError;

#[derive(Debug)]
pub struct Command {
    cmd: String,
    command_type: CommandType,
}

#[derive(Debug)]
enum CommandType {
    Land(Land),
    Cancel,
    Canary,
    CherryPick(CherryPick),
    Help,
    Priority(PriorityCommand),
}

impl CommandType {
    fn name(&self) -> &'static str {
        match &self {
            CommandType::Land(_) => "Land",
            CommandType::Cancel => "Cancel",
            CommandType::Canary => "Canary",
            CommandType::CherryPick(_) => "CherryPick",
            CommandType::Help => "Help",
            CommandType::Priority(_) => "Priority",
        }
    }
}

impl Command {
    pub fn from_comment(c: &str) -> Option<Result<Self, ParseCommandError>> {
        c.lines()
            .find(|line| line.starts_with('/'))
            .map(Self::from_line)
    }

    #[allow(dead_code)]
    pub fn from_comment_with_username(
        c: &str,
        my_username: &str,
    ) -> Option<Result<Self, ParseCommandError>> {
        c.lines()
            .find(|line| Self::line_starts_with_username(line, my_username))
            .map(|line| Self::from_line_with_username(line, my_username))
    }

    fn from_line_with_username(s: &str, my_username: &str) -> Result<Self, ParseCommandError> {
        if !Self::line_starts_with_username(s, my_username) {
            return Err(ParseCommandError);
        }

        let command_type = Self::from_iter(s.split_whitespace().skip(1))?;

        Ok(Command {
            cmd: s.to_owned(),
            command_type,
        })
    }

    fn line_starts_with_username(line: &str, my_username: &str) -> bool {
        if let Some(name) = line.split_whitespace().next() {
            if name.starts_with('@') {
                return &name[1..] == my_username;
            }
        }

        false
    }

    fn from_line(s: &str) -> Result<Self, ParseCommandError> {
        if !s.starts_with('/') {
            return Err(ParseCommandError);
        }

        let command_type = Self::from_iter(s[1..].split_whitespace())?;

        Ok(Command {
            cmd: s.to_owned(),
            command_type,
        })
    }

    fn from_iter<'a, I>(iter: I) -> Result<CommandType, ParseCommandError>
    where
        I: IntoIterator<Item = &'a str>,
    {
        let mut iter = iter.into_iter();

        let command_name = if let Some(name) = iter.next() {
            name
        } else {
            return Err(ParseCommandError);
        };

        // Arguments take the form of `<key>=<value>`
        let args = iter.map(|arg| {
            if let Some(idx) = arg.find('=') {
                (&arg[..idx], Some(&arg[idx + 1..]))
            } else {
                (arg, None)
            }
        });

        let command_type = match command_name {
            "land" | "merge" => CommandType::Land(Land::with_args(args)?),
            "cancel" | "stop" => CommandType::Cancel,
            "canary" | "try" => CommandType::Canary,
            "cherry" | "cherry-pick" => CommandType::CherryPick(CherryPick::with_args(args)?),
            "help" | "h" => CommandType::Help,
            "priority" => CommandType::Priority(PriorityCommand::with_args(args)?),

            _ => return Err(ParseCommandError),
        };

        Ok(command_type)
    }

    /// Display help information for Commands, formatted for use in Github comments
    pub fn help<'a>(
        config: &'a RepoConfig,
        project_board: Option<&'a ProjectBoard>,
    ) -> impl std::fmt::Display + 'a {
        Help::new(config, project_board)
    }

    pub async fn is_authorized(&self, ctx: &CommandContext<'_>) -> Result<bool> {
        let mut is_authorized = false;
        let mut reason = None;

        // Check to see if the user is a collaborator
        if ctx
            .github()
            .repos()
            .is_collaborator(ctx.config().owner(), ctx.config().name(), ctx.sender())
            .await?
            .into_inner()
        {
            is_authorized = true;
        } else {
            reason = Some("Not Collaborator");
        }

        // Post a comment to Github if there was a reason why the user wasn't authorized
        if !is_authorized {
            if let Some(reason) = reason {
                ctx.create_pr_comment(&format!(
                    "@{}: :key: Insufficient privileges: {}",
                    ctx.sender(),
                    reason
                ))
                .await?;
            }
        }

        Ok(is_authorized)
    }

    pub async fn execute(&self, ctx: &mut CommandContext<'_>) -> Result<()> {
        info!("Executing command '{}'", self.command_type.name());

        match &self.command_type {
            CommandType::Land(l) => Self::execute_land(ctx, l.priority(), l.squash).await?,
            CommandType::Cancel => Self::cancel_land(ctx).await?,
            CommandType::Canary => Self::canary_land(ctx).await?,
            CommandType::CherryPick(c) => Self::cherry_pick(ctx, c.target()).await?,
            CommandType::Help => {
                ctx.create_pr_comment(&Help::new(ctx.config(), ctx.project_board()).to_string())
                    .await?
            }
            CommandType::Priority(p) => Self::execute_priority(ctx, p.priority()).await?,
        }

        Ok(())
    }

    async fn execute_land(
        ctx: &mut CommandContext<'_>,
        priority: Option<Priority>,
        squash: Option<bool>,
    ) -> Result<()> {
        let mut ctx = if let Some(ctx) = ctx.active_pull_request_context().await {
            ctx
        } else {
            return Ok(());
        };

        if let Some(priority) = priority {
            Self::set_priority(&mut ctx, priority).await?;
        }
        if let Some(squash) = squash {
            Self::set_squash(&mut ctx, squash).await?;
        }

        Self::mark_pr_ready_to_land(&mut ctx).await
    }

    async fn execute_priority(ctx: &mut CommandContext<'_>, priority: Priority) -> Result<()> {
        let mut ctx = if let Some(ctx) = ctx.active_pull_request_context().await {
            ctx
        } else {
            return Ok(());
        };

        Self::set_priority(&mut ctx, priority).await
    }

    async fn set_priority(
        ctx: &mut ActivePullRequestContext<'_>,
        priority: Priority,
    ) -> Result<()> {
        info!("#{}: set priority to {:?}", ctx.pr().number, priority);

        let high_priority_label = ctx.config().labels().high_priority().to_owned();
        let low_priority_label = ctx.config().labels().low_priority().to_owned();
        match priority {
            Priority::High => {
                ctx.set_label(&high_priority_label).await?;
                ctx.remove_label(&low_priority_label).await?;
            }
            Priority::Normal => {
                ctx.remove_label(&high_priority_label).await?;
                ctx.remove_label(&low_priority_label).await?;
            }
            Priority::Low => {
                ctx.set_label(&low_priority_label).await?;
                ctx.remove_label(&high_priority_label).await?;
            }
        }

        Ok(())
    }

    async fn set_squash(ctx: &mut ActivePullRequestContext<'_>, squash: bool) -> Result<()> {
        info!("#{}: set squash to {}", ctx.pr().number, squash);

        let label = ctx.config().labels().squash().to_owned();

        if squash {
            ctx.set_label(&label).await?;
        } else {
            ctx.remove_label(&label).await?;
        }

        Ok(())
    }

    async fn mark_pr_ready_to_land(ctx: &mut ActivePullRequestContext<'_>) -> Result<()> {
        info!("attempting to mark pr #{} ReadyToLand", ctx.pr().number);

        // Skip marking for land on draft PRs
        if ctx.pr().is_draft() {
            ctx.create_pr_comment(
                ":clipboard: Looks like this PR is still in progress, unable to queue for landing",
            )
            .await?;
            return Ok(());
        }

        match ctx.pr().status {
            Status::InReview => {
                // double check the approval on the PR
                if ctx.config().require_review() && !ctx.pr().approved {
                    let approved = ctx
                        .github()
                        .get_review_decision(
                            ctx.config().repo().owner(),
                            ctx.config().repo().name(),
                            ctx.pr().number,
                        )
                        .await?;

                    ctx.pr_mut().approved = approved;
                }

                if ctx.pr().approved || !ctx.config().require_review() {
                    ctx.update_pr_status(Status::queued()).await?;
                    info!("pr #{} queued for landing", ctx.pr().number);
                } else {
                    info!(
                        "pr #{} is missing approvals, unable to queue for landing",
                        ctx.pr().number
                    );

                    let msg = format!(
                        "@{} :exclamation: This PR is still missing approvals, unable to queue for landing",
                        ctx.sender(),
                    );
                    ctx.create_pr_comment(&msg).await?;
                }
            }
            Status::Queued(_) | Status::Testing { .. } => {
                info!("pr #{} already queued for landing", ctx.pr().number);

                let msg = format!(
                    "@{} :bulb: This PR is already queued for landing",
                    ctx.sender(),
                );

                ctx.create_pr_comment(&msg).await?;
            }
            Status::Canary { .. } => {
                let msg = format!(
                    "@{} :bulb: This PR is currently being canaried, cancel the canary before queuing for landing",
                    ctx.sender(),
                );

                ctx.create_pr_comment(&msg).await?;
            }
        }

        Ok(())
    }

    async fn cancel_land(ctx: &mut CommandContext<'_>) -> Result<()> {
        let mut ctx = if let Some(ctx) = ctx.active_pull_request_context().await {
            ctx
        } else {
            return Ok(());
        };

        info!("Canceling land of pr #{}", ctx.pr().number);

        ctx.update_pr_status(Status::InReview).await
    }

    async fn canary_land(ctx: &mut CommandContext<'_>) -> Result<()> {
        let mut ctx = if let Some(ctx) = ctx.active_pull_request_context().await {
            ctx
        } else {
            return Ok(());
        };

        info!("Canarying land of pr #{}", ctx.pr().number);

        match ctx.pr().status {
            Status::InReview => {
                let canary_running = if let Some(board) = ctx.project_board() {
                    board.list_canary_cards(&ctx).await.map_or(false, |cards| cards.len() > 0)
                } else {
                    false
                };

                // If a canary is running, don't start this one up.
                if canary_running {
                    ctx.create_pr_comment("There is already another PR running a canary")
                        .await?;
                } else {
                    ctx.pr_mut().canary_requested = true;
                }
            }
            Status::Queued(_) | Status::Testing { .. } => {
                let msg = format!(
                    "@{} :bulb: This PR is currently queued for landing, cancel first if you want to canary the landing",
                    ctx.sender(),
                );

                ctx.create_pr_comment(&msg).await?;
            }
            Status::Canary { .. } => {
                ctx.create_pr_comment("This PR is already being canaried")
                    .await?;
            }
        }

        Ok(())
    }

    async fn cherry_pick(ctx: &mut CommandContext<'_>, target: &str) -> Result<()> {
        // Check if target is a valid branch
        if ctx.git_repository().fetch_ref(target).is_err() {
            info!("invalid cherry-pick target: '{}'", target);
            let msg = format!(
                "@{} :exclamation: '{}' is an invalid branch target for cherry-picking",
                ctx.sender(),
                target,
            );
            ctx.create_pr_comment(&msg).await?;
            return Ok(());
        }

        // Get Commit range from PR
        let pull = ctx
            .github()
            .pulls()
            .get(ctx.config().owner(), ctx.config().name(), ctx.number())
            .await?
            .into_inner();
        let base_oid = pull.base.sha;
        let head_oid = pull.head.sha;

        let branch = format!("pick/{}/{}", ctx.number(), target);

        if ctx
            .git_repository()
            .fetch_and_cherry_pick(target, &branch, &base_oid, &head_oid)?
            .is_none()
        {
            let msg = format!(
                "@{} :exclamation: cherry-pick failed, possibly due to conflicts. \
                You can perform the cherry-pick yourself by running the following commands:\n\
                ```\n\
                git fetch {url} {target} {head_oid}\n\
                git checkout {target}\n\
                git cherry-pick {base_oid}..{head_oid}\n\
                ```\n\
                ",
                ctx.sender(),
                url = ctx.config().repo().to_github_https_url(),
                target = target,
                head_oid = head_oid,
                base_oid = base_oid,
            );
            ctx.create_pr_comment(&msg).await?;

            return Ok(());
        }

        // Push branch and open pull request
        ctx.git_repository().push_branch(&branch)?;
        info!("pushed '{}' branch", branch);

        let title = format!(
            "Cherry-pick PR #{} into {}: {}",
            ctx.number(),
            target,
            pull.title
        );
        let quoted_body = if let Some(body) = pull.body {
            let mut quoted_body = String::new();
            for line in body.lines() {
                quoted_body.push_str("> ");
                quoted_body.push_str(line);
                quoted_body.push('\n');
            }
            quoted_body
        } else {
            "".to_owned()
        };
        let body = format!(
            "This cherry-pick was triggerd by a request on #{}\n\
            Please review the diff to ensure there are not any unexpected changes.\n\
            \n\
            {}
            \n\
            cc @{}",
            ctx.number(),
            quoted_body,
            ctx.sender()
        );

        let request = NewPullRequest {
            title,
            body: Some(body),
            head: branch.to_owned(),
            base: target.to_owned(),
            maintainer_can_modify: Some(true),
            draft: Some(false),
        };

        let new_pull = ctx
            .github()
            .pulls()
            .create(ctx.config().owner(), ctx.config().name(), request)
            .await?
            .into_inner();

        let msg = format!(
            "@{} :cherries: Opened PR #{} to cherry-pick these changes into {}",
            ctx.sender(),
            new_pull.number,
            target
        );
        ctx.create_pr_comment(&msg).await?;

        Ok(())
    }
}

struct Help<'a> {
    config: &'a RepoConfig,
    project_board: Option<&'a ProjectBoard>,
}

impl<'a> Help<'a> {
    fn new(config: &'a RepoConfig, project_board: Option<&'a ProjectBoard>) -> Self {
        Self {
            config,
            project_board,
        }
    }
}

#[allow(clippy::write_literal)]
impl std::fmt::Display for Help<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        const NON_BREAKING_SPACE: &str = "&nbsp;";

        writeln!(f, "<details>")?;
        write!(f, "<summary>")?;
        write!(f, "Bors help and documentation")?;
        writeln!(f, "</summary>")?;
        writeln!(f, "<br />")?;
        writeln!(f)?;

        let queue_link = self.project_board.map(|p| p.board().html_url.as_str());
        let queue = if let Some(link) = queue_link {
            format!("[Merge Queue]({})", link)
        } else {
            "Merge Queue".to_owned()
        };
        writeln!(
            f,
            "Bors is a Github App used to manage merging of PRs in order to ensure that CI is always \
            green. It does so by maintaining a {queue}. Once a PR reaches the head of the Merge \
            Queue it is rebased on top of the latest version of the PR's `base-branch` (generally \
            `master`) and then triggers CI. If CI comes back green the PR is then merged into the \
            `base-branch`. Regardless of the outcome, the next PR is the queue is then processed.",
            queue = queue,
        )?;
        writeln!(f)?;

        //
        // General
        //
        writeln!(f, "### General")?;

        if let Some(link) = queue_link {
            writeln!(
                f,
                "- This project's Merge Queue can be found [here]({}).",
                link
            )?;
        }

        if self.config.require_review() {
            writeln!(
                f,
                "- This project requires PRs to be [Reviewed]\
                (https://help.github.com/en/github/collaborating-with-issues-and-pull-requests/about-pull-request-reviews) \
                and Approved before they can be queued for merging.",
            )?;
        }

        if self.config.maintainer_mode() {
            writeln!(
                f,
                "- Before PR's can be merged they must be configured with \
                [\"Allow edits from maintainers\"]\
                (https://help.github.com/en/github/collaborating-with-issues-and-pull-requests/allowing-changes-to-a-pull-request-branch-created-from-a-fork) \
                enabled. This is needed for Bors to be able to update PR's in-place \
                so that Github can properly recognize and mark them as \"merged\" \
                once they've been merged into the upstream branch.",
            )?;
        }

        //
        // Commands
        //
        writeln!(f, "### Commands")?;
        writeln!(
            f,
            "Bors actions can be triggered by posting a comment which includes a line of the form `/<action>`."
        )?;
        writeln!(f, "| Command | Action | Description |")?;
        writeln!(f, "| --- | --- | --- |")?;
        writeln!(
            f,
            "| __Land__ | `land`, `merge` | attempt to land or merge a PR |"
        )?;
        writeln!(
            f,
            "| __Canary__ | `canary`, `try` | canary a PR by performing all checks without merging |"
        )?;
        writeln!(
            f,
            "| __Cancel__ | `cancel`, `stop` | stop an in-progress land |"
        )?;
        writeln!(
            f,
            "| __Cherry Pick__ | `cherry-pick <target>` | cherry-pick a PR into `<target>` branch |"
        )?;
        writeln!(
            f,
            "| __Priority__ | `priority` | set the priority level for a PR (`high`, `normal`, `low`) |"
        )?;
        writeln!(f, "| __Help__ | `help`, `h` | show this help message |")?;
        writeln!(f)?;

        //
        // Options
        //
        writeln!(f, "### Options")?;
        writeln!(
            f,
            "Options for Pull Requests are configured through the application of labels.",
        )?;

        writeln!(
            f,
            "| {align}Option{align} | Description |",
            align = NON_BREAKING_SPACE.repeat(10)
        )?;
        writeln!(f, "| --- | --- |")?;
        writeln!(
            f,
            "| ![label: {name}](https://img.shields.io/static/v1?label=&message={name}&color=lightgrey) | {desc} |",
            name = self.config.labels().high_priority(),
            desc = "Indicates that the PR is high-priority. \
            When queued the PR will be placed at the head of the merge queue.",
        )?;
        writeln!(
            f,
            "| ![label: {name}](https://img.shields.io/static/v1?label=&message={name}&color=lightgrey) | {desc} |",
            name = self.config.labels().low_priority(),
            desc = "Indicates that the PR is low-priority. \
            When queued the PR will be placed at the back of the merge queue.",
        )?;
        writeln!(
            f,
            "| ![label: {name}](https://img.shields.io/static/v1?label=&message={name}&color=lightgrey) | {desc} |",
            name = self.config.labels().squash(),
            desc = "Before merging the PR will be squashed down to a single commit, \
            only retaining the commit message of the first commit in the PR.",
        )?;

        writeln!(f)?;
        writeln!(f, "</details>")
    }
}

#[derive(Debug)]
struct Land {
    priority: Option<PriorityCommand>,
    squash: Option<bool>,
}

impl Land {
    fn with_args<'a, I>(iter: I) -> Result<Self, ParseCommandError>
    where
        I: IntoIterator<Item = (&'a str, Option<&'a str>)>,
    {
        let mut priority = None;
        let mut squash = None;

        for (key, value) in iter {
            match key {
                "p" | "priority" => {
                    priority = Some(PriorityCommand::from_arg(value)?);
                }
                "squash+" => {
                    squash = Some(true);
                }
                "squash-" => {
                    squash = Some(false);
                }

                // First key we hit that we don't understand we should just bail
                _ => break,
            }
        }

        Ok(Self { priority, squash })
    }

    fn priority(&self) -> Option<Priority> {
        self.priority.as_ref().map(PriorityCommand::priority)
    }
}

#[derive(Debug)]
struct PriorityCommand {
    priority: Priority,
}

impl PriorityCommand {
    fn with_args<'a, I>(iter: I) -> Result<Self, ParseCommandError>
    where
        I: IntoIterator<Item = (&'a str, Option<&'a str>)>,
    {
        let mut iter = iter.into_iter();

        let arg = iter
            .next()
            .and_then(|(k, v)| if v.is_some() { None } else { Some(k) });

        Self::from_arg(arg)
    }

    fn from_arg(value: Option<&str>) -> Result<Self, ParseCommandError> {
        if let Some(v) = value {
            //TODO better error message (expected integer)
            let priority = v.parse().map_err(|_| ParseCommandError)?;
            Ok(Self { priority })
        } else {
            // No value specified
            //TODO better error message
            Err(ParseCommandError)
        }
    }

    fn priority(&self) -> Priority {
        self.priority
    }
}

#[derive(Debug)]
struct CherryPick {
    target: String,
}

impl CherryPick {
    fn with_args<'a, I>(iter: I) -> Result<Self, ParseCommandError>
    where
        I: IntoIterator<Item = (&'a str, Option<&'a str>)>,
    {
        let mut iter = iter.into_iter();

        let arg = iter
            .next()
            .and_then(|(k, v)| if v.is_some() { None } else { Some(k) });

        Self::from_arg(arg)
    }

    fn from_arg(value: Option<&str>) -> Result<Self, ParseCommandError> {
        if let Some(v) = value {
            Ok(Self {
                target: v.to_owned(),
            })
        } else {
            // No value specified
            //TODO better error message
            Err(ParseCommandError)
        }
    }

    fn target(&self) -> &str {
        &self.target
    }
}
