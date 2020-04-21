//! Defines commands which can be asked to be performed
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
