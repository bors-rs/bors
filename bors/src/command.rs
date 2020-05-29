//! Defines commands which can be asked to be performed

use crate::{event_processor::CommandContext, Result};
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
    Approve(Approve),
    Unapprove,
    Land(Land),
    Retry(Retry),
    Cancel,
    Help,
    Priority(Priority),
}

impl CommandType {
    fn name(&self) -> &'static str {
        match &self {
            CommandType::Approve(_) => "Approve",
            CommandType::Unapprove => "Unapprove",
            CommandType::Land(_) => "Land",
            CommandType::Retry(_) => "Retry",
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
            "approve" | "lgtm" | "r+" => CommandType::Approve(Approve::with_args(args)?),
            "unapprove" | "r-" => CommandType::Unapprove,
            "land" | "merge" => CommandType::Land(Land::with_args(args)?),
            "retry" => CommandType::Retry(Retry::with_args(args)?),
            "cancel" | "stop" => CommandType::Cancel,
            "help" | "h" => CommandType::Help,
            "priority" => CommandType::Priority(Priority::with_args(args)?),

            _ => return Err(ParseCommnadError),
        };

        Ok(command_type)
    }

    pub fn from_review(review: &github::Review) -> Option<Command> {
        let command_type = match review.state {
            github::ReviewState::Approved => CommandType::Approve(Approve::new()),
            github::ReviewState::ChangesRequested => CommandType::Unapprove,
            github::ReviewState::Commented => return None,
            github::ReviewState::Dismissed => return None,
        };

        Some(Command {
            cmd: "FROM REVIEW".to_owned(),
            command_type,
        })
    }

    /// Display help information for Commands, formatted for use in Github comments
    pub fn help() -> impl std::fmt::Display {
        Help
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

        // Check to see if the user issuing the command is attempting to approve their own PR
        if is_authorized
            && !ctx.config().allow_self_review()
            && ctx.sender_is_author()
            && matches!(self.command_type, CommandType::Approve(_))
        {
            is_authorized = false;
            reason = Some("Can't approve your own PR");
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
            CommandType::Approve(a) => {
                if let Some(priority) = a.priority() {
                    Self::set_priority(&mut ctx, priority).await?;
                }
                if let Some(squash) = a.squash {
                    Self::set_squash(&mut ctx, squash).await?;
                }

                Self::approve_pr(&mut ctx, false).await?;
            }
            CommandType::Unapprove => Self::unapprove_pr(ctx).await?,
            CommandType::Land(l) => {
                if let Some(priority) = l.priority() {
                    Self::set_priority(&mut ctx, priority).await?;
                }
                if let Some(squash) = l.squash {
                    Self::set_squash(&mut ctx, squash).await?;
                }

                Self::approve_pr(&mut ctx, true).await?;
                Self::mark_pr_ready_to_land(&mut ctx).await?;
            }
            CommandType::Retry(r) => {
                if let Some(priority) = r.priority() {
                    Self::set_priority(&mut ctx, priority).await?;
                }
                unimplemented!();
            }
            CommandType::Cancel => Self::cancel_land(ctx).await?,
            CommandType::Help => ctx.create_pr_comment(&Help.to_string()).await?,
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

    async fn approve_pr(ctx: &mut CommandContext<'_>, is_land_command: bool) -> Result<()> {
        // Skip adding approval if author matches the "reviewer"
        if !ctx.config().allow_self_review() && ctx.sender_is_author() {
            return Ok(());
        }

        // Skip adding approval on draft PRs
        if ctx.pr().is_draft() {
            ctx.create_pr_comment(
                ":clipboard: Looks like this PR is still in progress, ignoring approval",
            )
            .await?;
            return Ok(());
        }

        info!("#{}: approved by '{}'", ctx.pr().number, ctx.sender());

        let reviewer = ctx.sender().to_owned();
        if ctx.pr_mut().approved_by.insert(reviewer) {
            // The approval was added
            let msg = format!(
                ":pushpin: Commit {:7} has been approved by `{}`",
                ctx.pr().head_ref_oid,
                ctx.sender()
            );
            ctx.create_pr_comment(&msg).await?;
        } else if !is_land_command {
            // 'reviewer' had already approved the PR
            let msg = format!(
                "@{} :bulb: You have already approved this PR, no need to approve it again",
                ctx.sender()
            );
            ctx.create_pr_comment(&msg).await?;
        }

        Ok(())
    }

    async fn unapprove_pr(ctx: &mut CommandContext<'_>) -> Result<()> {
        info!(
            "#{}: '{}' revoked their approval",
            ctx.pr().number,
            ctx.sender()
        );

        let reviewer = ctx.sender().to_owned();
        if ctx.pr_mut().approved_by.remove(&reviewer) {
            ctx.create_pr_comment(&format!(
                ":anguished: `{}`'s approval has been revoked",
                ctx.sender()
            ))
            .await?;
        }

        Ok(())
    }

    async fn mark_pr_ready_to_land(ctx: &mut CommandContext<'_>) -> Result<()> {
        use crate::state::Status;

        info!("attempting to mark pr #{} ReadyToLand", ctx.pr().number);

        match ctx.pr().status {
            Status::InReview => {
                if ctx.pr().approved || ctx.config().allow_self_review() {
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

struct Help;

impl std::fmt::Display for Help {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "<details>")?;
        write!(f, "<summary>")?;
        write!(f, "Bors commands and options")?;
        writeln!(f, "</summary>")?;
        writeln!(f, "<br />")?;
        writeln!(f)?;

        writeln!(
            f,
            "Bors actions can be triggered by posting a comment of the form `/<action>`:"
        )?;
        writeln!(
            f,
            "- __Approve__ `approve`, `lgtm`, `r+`: add your approval to a PR"
        )?;
        writeln!(
            f,
            "- __Unapprove__ `unapprove`, `r-`: remove your approval to a PR"
        )?;
        writeln!(
            f,
            "- __Land__ `land`, `merge`: attempt to land or merge a PR"
        )?;
        writeln!(
            f,
            "- __Retry__ `retry`: attempt to retry the last action (usually a land/merge)"
        )?;
        writeln!(f, "- __Cancel__ `cancel`, `stop`: stop an in-progress land")?;
        writeln!(f, "- __Help__ `help`, `h`: show this help message")?;
        writeln!(
            f,
            "- __Priority__ `priority`: set the priority level for a PR"
        )?;

        writeln!(f)?;
        writeln!(f, "</details>")
    }
}

#[derive(Debug)]
struct Approve {
    priority: Option<Priority>,
    squash: Option<bool>,
}

impl Approve {
    fn new() -> Self {
        Self {
            priority: None,
            squash: None,
        }
    }

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

#[derive(Debug)]
struct Retry {
    priority: Option<Priority>,
}

impl Retry {
    fn with_args<'a, I>(iter: I) -> Result<Self, ParseCommnadError>
    where
        I: IntoIterator<Item = (&'a str, Option<&'a str>)>,
    {
        let mut priority = None;

        for (key, value) in iter {
            match key {
                "p" | "priority" => {
                    priority = Some(Priority::from_arg(value)?);
                }

                // First key we hit that we don't understand we should just bail
                _ => break,
            }
        }

        Ok(Self { priority })
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
