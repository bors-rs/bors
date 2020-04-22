//! Defines commands which can be asked to be performed

use crate::{event_processor::CommandContext, Result};
use thiserror::Error;

#[derive(Error, Debug)]
#[error("invalid command")]
pub struct ParseCommnadError;

#[derive(Debug)]
pub struct Command {
    cmd: String,
    pub command_type: CommandType,
}

#[derive(Debug)]
pub enum CommandType {
    Approve(Approve),
    Unapprove,
    Land(Land),
    Retry(Retry),
    Cancel,
    Priority(Priority),
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
            "priority" => CommandType::Priority(Priority::with_args(args)?),

            _ => return Err(ParseCommnadError),
        };

        Ok(command_type)
    }

    /// Display help information for Commands, formatted for use in Github comments
    pub fn help() -> impl std::fmt::Display {
        Help
    }

    // TODO handle `delegate`
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

    pub async fn execute(&self, mut ctx: &mut CommandContext<'_>) {
        match &self.command_type {
            CommandType::Approve(a) => {
                if let Some(priority) = a.priority() {
                    Self::set_priority(&mut ctx, priority);
                }

                Self::approve_pr(&mut ctx).await.unwrap();
            }
            CommandType::Unapprove => unimplemented!(),
            CommandType::Land(l) => {
                if let Some(priority) = l.priority() {
                    Self::set_priority(&mut ctx, priority);
                }

                unimplemented!();
            }
            CommandType::Retry(r) => {
                if let Some(priority) = r.priority() {
                    Self::set_priority(&mut ctx, priority);
                }
                unimplemented!();
            }
            CommandType::Cancel => unimplemented!(),
            CommandType::Priority(p) => Self::set_priority(&mut ctx, p.priority()),
        }
    }

    fn set_priority(ctx: &mut CommandContext, priority: u32) {
        ctx.pr_mut().priority = priority;
    }

    async fn approve_pr(ctx: &mut CommandContext<'_>) -> Result<()> {
        // Skip adding approval if author matches the "reviewer"
        if !ctx.config().allow_self_review() && ctx.sender_is_author() {
            return Ok(());
        }

        let reviewer = ctx.sender().to_owned();
        let msg = if ctx.pr_mut().approved_by.insert(reviewer) {
            // The approval was added
            format!(
                ":pushpin: Commit {:7} has been approved by `{}`",
                ctx.pr().head_ref_oid,
                ctx.sender()
            )
        } else {
            // 'reviewer' had already approved the PR
            format!(
                "@{} :bulb: You have already approved this PR, no need to approve it again",
                ctx.sender()
            )
        };

        ctx.create_pr_comment(&msg).await?;

        Ok(())
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
        writeln!(
            f,
            "- __Priority__ `priority`: set the priority level for a PR"
        )?;

        writeln!(f)?;
        writeln!(f, "</details>")
    }
}

#[derive(Debug)]
pub struct Approve {
    priority: Option<Priority>,
}

impl Approve {
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

    pub fn priority(&self) -> Option<u32> {
        self.priority.as_ref().map(Priority::priority)
    }
}

#[derive(Debug)]
pub struct Land {
    priority: Option<Priority>,
}

impl Land {
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

    pub fn priority(&self) -> Option<u32> {
        self.priority.as_ref().map(Priority::priority)
    }
}

#[derive(Debug)]
pub struct Retry {
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

    pub fn priority(&self) -> Option<u32> {
        self.priority.as_ref().map(Priority::priority)
    }
}

#[derive(Debug)]
pub struct Priority {
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

    pub fn priority(&self) -> u32 {
        self.priority
    }
}
