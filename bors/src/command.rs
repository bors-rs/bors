//! Defines commands which can be asked to be performed

use crate::{config::RepoConfig, event_processor::CommandContext, Result};
use log::info;
use thiserror::Error;

#[derive(Error, Debug)]
#[error("invalid command")]
pub struct ParseCommnadError;

#[derive(Debug)]
pub struct Command {
    cmd: String,
    command_type: CommandType,
}

#[derive(Debug)]
enum CommandType {
    Land(Land),
    Cancel,
    Help,
    Priority(Priority),
}

impl CommandType {
    fn name(&self) -> &'static str {
        match &self {
            CommandType::Land(_) => "Land",
            CommandType::Cancel => "Cancel",
            CommandType::Help => "Help",
            CommandType::Priority(_) => "Priority",
        }
    }
}

impl Command {
    pub fn from_comment(c: &str) -> Option<Result<Self, ParseCommnadError>> {
        c.lines()
            .find(|line| line.starts_with('/'))
            .map(Self::from_line)
    }

    #[allow(dead_code)]
    pub fn from_comment_with_username(
        c: &str,
        my_username: &str,
    ) -> Option<Result<Self, ParseCommnadError>> {
        c.lines()
            .find(|line| Self::line_starts_with_username(line, my_username))
            .map(|line| Self::from_line_with_username(line, my_username))
    }

    fn from_line_with_username(s: &str, my_username: &str) -> Result<Self, ParseCommnadError> {
        if !Self::line_starts_with_username(s, my_username) {
            return Err(ParseCommnadError);
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

    fn from_line(s: &str) -> Result<Self, ParseCommnadError> {
        if !s.starts_with('/') {
            return Err(ParseCommnadError);
        }

        let command_type = Self::from_iter(s[1..].split_whitespace())?;

        Ok(Command {
            cmd: s.to_owned(),
            command_type,
        })
    }

    fn from_iter<'a, I>(iter: I) -> Result<CommandType, ParseCommnadError>
    where
        I: IntoIterator<Item = &'a str>,
    {
        let mut iter = iter.into_iter();

        let command_name = if let Some(name) = iter.next() {
            name
        } else {
            return Err(ParseCommnadError);
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
            "help" | "h" => CommandType::Help,
            "priority" => CommandType::Priority(Priority::with_args(args)?),

            _ => return Err(ParseCommnadError),
        };

        Ok(command_type)
    }

    /// Display help information for Commands, formatted for use in Github comments
    pub fn help<'a>(config: &'a RepoConfig) -> impl std::fmt::Display + 'a {
        Help::new(config)
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

    pub async fn execute(&self, mut ctx: &mut CommandContext<'_>) -> Result<()> {
        info!("Executing command '{}'", self.command_type.name());

        match &self.command_type {
            CommandType::Land(l) => {
                if let Some(priority) = l.priority() {
                    Self::set_priority(&mut ctx, priority).await?;
                }
                if let Some(squash) = l.squash {
                    Self::set_squash(&mut ctx, squash).await?;
                }

                Self::mark_pr_ready_to_land(&mut ctx).await?;
            }
            CommandType::Cancel => Self::cancel_land(ctx).await?,
            CommandType::Help => {
                ctx.create_pr_comment(&Help::new(ctx.config()).to_string())
                    .await?
            }
            CommandType::Priority(p) => Self::set_priority(&mut ctx, p.priority()).await?,
        }

        Ok(())
    }

    async fn set_priority(ctx: &mut CommandContext<'_>, priority: u32) -> Result<()> {
        info!("#{}: set priority to {}", ctx.pr().number, priority);

        let label = ctx.config().labels().high_priority().to_owned();
        if priority > 0 {
            ctx.set_label(&label).await?;
        } else {
            ctx.remove_label(&label).await?;
        }

        Ok(())
    }

    async fn set_squash(ctx: &mut CommandContext<'_>, squash: bool) -> Result<()> {
        info!("#{}: set squash to {}", ctx.pr().number, squash);

        let label = ctx.config().labels().squash().to_owned();

        if squash {
            ctx.set_label(&label).await?;
        } else {
            ctx.remove_label(&label).await?;
        }

        Ok(())
    }

    async fn mark_pr_ready_to_land(ctx: &mut CommandContext<'_>) -> Result<()> {
        use crate::state::Status;

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
                if ctx.pr().approved || !ctx.config().require_review() {
                    ctx.update_pr_status(Status::Queued).await?;
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
            Status::Queued | Status::Testing { .. } => {
                info!("pr #{} already queued for landing", ctx.pr().number);

                let msg = format!(
                    "@{} :bulb: This PR is already queued for landing",
                    ctx.sender(),
                );

                ctx.create_pr_comment(&msg).await?;
            }
        }

        Ok(())
    }

    async fn cancel_land(ctx: &mut CommandContext<'_>) -> Result<()> {
        use crate::state::Status;

        info!("Canceling land of pr #{}", ctx.pr().number);

        ctx.update_pr_status(Status::InReview).await
    }
}

struct Help<'a> {
    config: &'a RepoConfig,
    // project_board: Option<&'a ProjectBoard>,
}

impl<'a> Help<'a> {
    fn new(config: &'a RepoConfig) -> Self {
        Self { config }
    }
}

impl std::fmt::Display for Help<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        const NON_BREAKING_SPACE: &str = "&nbsp;";

        writeln!(f, "<details>")?;
        write!(f, "<summary>")?;
        write!(f, "Bors commands and options")?;
        writeln!(f, "</summary>")?;
        writeln!(f, "<br />")?;
        writeln!(f)?;

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
            "| __Cancel__ | `cancel`, `stop` | stop an in-progress land |"
        )?;
        writeln!(f, "| __Help__ | `help`, `h` | show this help message |")?;
        writeln!(
            f,
            "| __Priority__ | `priority` | set the priority level for a PR |"
        )?;
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
    priority: Option<Priority>,
    squash: Option<bool>,
}

impl Land {
    fn with_args<'a, I>(iter: I) -> Result<Self, ParseCommnadError>
    where
        I: IntoIterator<Item = (&'a str, Option<&'a str>)>,
    {
        let mut priority = None;
        let mut squash = None;

        for (key, value) in iter {
            match key {
                "p" | "priority" => {
                    priority = Some(Priority::from_arg(value)?);
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

    fn priority(&self) -> Option<u32> {
        self.priority.as_ref().map(Priority::priority)
    }
}

// XXX Don't have priority be an integer, it should be a boolean
#[derive(Debug)]
struct Priority {
    priority: u32,
}

impl Priority {
    fn with_args<'a, I>(iter: I) -> Result<Self, ParseCommnadError>
    where
        I: IntoIterator<Item = (&'a str, Option<&'a str>)>,
    {
        let mut iter = iter.into_iter();

        let arg = iter
            .next()
            .and_then(|(k, v)| if v.is_some() { None } else { Some(k) });

        Self::from_arg(arg)
    }

    fn from_arg(value: Option<&str>) -> Result<Self, ParseCommnadError> {
        if let Some(v) = value {
            //TODO better error message (expected integer)
            let priority = v.parse().map_err(|_| ParseCommnadError)?;
            Ok(Self { priority })
        } else {
            // No value specified
            //TODO better error message
            return Err(ParseCommnadError);
        }
    }

    fn priority(&self) -> u32 {
        self.priority
    }
}
