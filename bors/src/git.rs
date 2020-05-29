use crate::{config::GitConfig, state::Repo, Config, Result};
use anyhow::{anyhow, Context};
use github::Oid;
use log::info;
use std::{
    path::{Path, PathBuf},
    process::Command,
};

const REPOS_DIR: &str = "repos";

#[derive(Debug)]
pub struct GitRepository {
    directory: PathBuf,
    github_repo: Repo,
    git_config: GitConfig,
}

impl GitRepository {
    pub fn from_config(config: &Config) -> Result<Self> {
        let github_repo = config.repo().repo().clone();
        let git_config = config.git.clone();
        let mut directory = std::env::current_dir()?;
        directory.push(REPOS_DIR);
        directory.push(github_repo.owner());
        directory.push(github_repo.name());

        if !Git::new().current_dir(&directory).is_git_repo()? {
            info!(
                "cloning '{}' to '{}'",
                github_repo.to_github_ssh_url(),
                directory.display()
            );
            Git::new()
                .with_ssh(&git_config.ssh_key_file)
                .clone(&directory, &github_repo)?;
        } else {
            info!("using existing on-disk repo at {}", directory.display());
        }

        if !Git::new()
            .current_dir(&directory)
            .remote_matches_github_repo(&github_repo)?
        {
            return Err(anyhow!(
                "on-disk repo's 'origin' remote doesn't match config"
            ));
        }

        Ok(Self {
            directory,
            github_repo,
            git_config,
        })
    }

    pub fn push_branch(&mut self, branch: &str) -> Result<()> {
        self.git().push_branch(branch, true)
    }

    pub fn push_to_remote(
        &mut self,
        repo: &Repo,
        branch: &str,
        old_oid: &Oid,
        new_oid: &Oid,
    ) -> Result<()> {
        self.git().push_to_remote(repo, branch, old_oid, new_oid)
    }

    pub fn fetch_and_rebase(
        &mut self,
        base_ref: &str,
        head_oid: &Oid,
        branch: &str,
        pr_number: u64,
        fixup_all: bool,
    ) -> Result<Option<Oid>> {
        // Fetch base ref and head_oid
        self.fetch(base_ref, head_oid)?;
        let base_oid = self.git().ref_to_oid(&format!("origin/{}", base_ref))?;
        self.rebase(&base_oid, head_oid, branch, pr_number, fixup_all)
    }

    fn fetch(&mut self, base_ref: &str, oid: &Oid) -> Result<()> {
        self.git().fetch(&[base_ref, &oid.to_string()])
    }

    // None represents a Merge conflict
    fn rebase(
        &mut self,
        base_oid: &Oid,
        head_oid: &Oid,
        branch: &str,
        pr_number: u64,
        fixup_all: bool,
    ) -> Result<Option<Oid>> {
        // First create the branch to work on for the rebase
        self.git().create_branch(branch, head_oid)?;

        if fixup_all {
            // Get the first commit in the PR
            let oid = self.git().get_first_commit(base_oid, head_oid)?;

            // This shouldn't result in a merge conflict since we're just rewording each commit
            // message in the series
            self.git().rebase(
                &oid,
                false,
                Some(format!("git commit --amend --fixup={}", oid)),
            )?;
        }

        // Attempt to perform the rebase
        if let Err(e) = self.git().rebase(base_oid, true, None) {
            info!("Rebase failed: {}", e);

            // the rebase failed, probably due to a merge conflict so we need to reset the state of
            // the tree and abort the rebase
            self.git().rebase_abort()?;
            Ok(None)
        } else {
            let head_oid = self.git().head_oid()?;

            // If the head_oid and base_oid's match after the rebase then it means that the rebased
            // commits resulted in no-ops
            if head_oid == *base_oid {
                Ok(None)
            } else {
                // Amend the tip commit to annotate that it closes the PR
                let editor = format!(
                    "git interpret-trailers --trailer \"Closes: #{}\" --in-place",
                    pr_number
                );
                self.git().amend(&editor)?;
                let head_oid = self.git().head_oid()?;

                Ok(Some(head_oid))
            }
        }
    }

    fn git(&self) -> Git {
        Git::new()
            .current_dir(&self.directory)
            .with_user(&self.git_config.user)
            .with_email(&self.git_config.email)
            .with_ssh(&self.git_config.ssh_key_file)
    }
}

struct Git {
    inner: Command,
}

impl Git {
    pub fn new() -> Self {
        let mut inner = Command::new("git");

        inner
            // Stop git looking at directories above the `REPOS_DIR`
            .env(
                "GIT_CEILING_DIRECTORIES",
                std::env::current_dir().unwrap().join(REPOS_DIR),
            )
            // Don't try and open an editor for things like `rebase -i`
            .env("GIT_EDITOR", "cat");

        Self { inner }
    }

    // Use `-C <path>` instead of `Command::current_dir` so that attempting to run a command in a
    // didirectory that doesn't exist doesn't fail to run the command but instead will succeed with
    // a non-zero exit code
    pub fn current_dir(mut self, path: &Path) -> Self {
        // self.inner.current_dir(path);
        self.inner.arg("-C").arg(path);
        self
    }

    pub fn with_ssh(mut self, ssh_key_file: &Path) -> Self {
        let path = if ssh_key_file.is_absolute() {
            ssh_key_file.to_path_buf()
        } else {
            std::env::current_dir().unwrap().join(ssh_key_file)
        };
        self.inner.env(
            "GIT_SSH_COMMAND",
            format!("ssh -i {} -S none -o 'IdentitiesOnly true'", path.display()),
        );
        self
    }

    pub fn with_user(mut self, user: &str) -> Self {
        self.inner.env("GIT_AUTHOR_NAME", user);
        self.inner.env("GIT_COMMITTER_NAME", user);
        self
    }

    pub fn with_email(mut self, email: &str) -> Self {
        self.inner.env("GIT_AUTHOR_EMAIL", email);
        self.inner.env("GIT_COMMITTER_EMAIL", email);
        self
    }

    pub fn with_editor(mut self, editor: &str) -> Self {
        self.inner.env("GIT_EDITOR", editor);
        self
    }

    fn run(mut self) -> Result<String> {
        let output = self.inner.output()?;

        if !output.status.success() {
            return Err(anyhow!(
                "failed to run git command:\n{}",
                String::from_utf8_lossy(&output.stderr)
            ));
        }

        Ok(String::from_utf8(output.stdout)?)
    }

    pub fn is_git_repo(mut self) -> Result<bool> {
        let output = self
            .inner
            .args(&["rev-parse", "--git-dir"])
            .output()
            .context("checking if a directory is a git repo")?;

        Ok(output.status.success())
    }

    pub fn remote_matches_github_repo(mut self, github_repo: &Repo) -> Result<bool> {
        self.inner.args(&["remote", "get-url", "origin"]);
        let output = self.run()?;

        Ok(output.trim() == github_repo.to_github_ssh_url())
    }

    pub fn clone(mut self, path: &Path, github_repo: &Repo) -> Result<()> {
        self.inner
            .arg("clone")
            .arg(github_repo.to_github_ssh_url())
            .arg(path);
        self.run()
            .with_context(|| format!("cloning {}", github_repo.to_github_ssh_url()))?;
        Ok(())
    }

    pub fn fetch<I, S>(mut self, refspec: I) -> Result<()>
    where
        I: IntoIterator<Item = S>,
        S: AsRef<std::ffi::OsStr>,
    {
        self.inner.arg("fetch").arg("origin").args(refspec);
        self.run()?;
        Ok(())
    }

    pub fn create_branch(mut self, branch_name: &str, oid: &Oid) -> Result<()> {
        self.inner
            .args(&["checkout", "-B", branch_name])
            .arg(oid.to_string());
        self.run()?;
        Ok(())
    }

    pub fn amend(mut self, editor: &str) -> Result<()> {
        self.inner.args(&["commit", "--amend"]);
        self.with_editor(editor).run()?;
        Ok(())
    }

    pub fn rebase_abort(mut self) -> Result<()> {
        self.inner.args(&["rebase", "--abort"]);
        self.run()?;
        Ok(())
    }

    pub fn rebase(mut self, base_oid: &Oid, autosquash: bool, exec: Option<String>) -> Result<()> {
        self.inner.args(&["rebase", "-i", "--force-rebase"]);
        self.inner.arg(base_oid.to_string());

        if autosquash {
            self.inner.arg("--autosquash");
        }

        if let Some(exec) = exec {
            self.inner.arg("--exec");
            self.inner.arg(exec);
        }

        self.run()?;
        Ok(())
    }

    pub fn get_first_commit(mut self, base_oid: &Oid, head_oid: &Oid) -> Result<Oid> {
        self.inner
            .arg("rev-list")
            .arg(&format!("{}..{}", base_oid, head_oid));
        let output = self.run()?;
        let first = output
            .lines()
            .last()
            .ok_or_else(|| anyhow!("unable to query first commit in series"))?;
        Ok(Oid::from_str(first.trim()))
    }

    pub fn head_oid(self) -> Result<Oid> {
        self.ref_to_oid("HEAD")
    }

    pub fn ref_to_oid(mut self, r: &str) -> Result<Oid> {
        self.inner.args(&["rev-parse", r]);
        let output = self.run()?;
        Ok(Oid::from_str(output.trim()))
    }

    pub fn push_branch(mut self, branch: &str, force: bool) -> Result<()> {
        self.inner.args(&["push", "origin"]);
        if force {
            self.inner.arg("--force-with-lease");
        }
        self.inner.arg(branch);
        self.run()?;
        Ok(())
    }

    pub fn push_to_remote(
        mut self,
        repo: &Repo,
        branch: &str,
        old_oid: &Oid,
        new_oid: &Oid,
    ) -> Result<()> {
        self.inner
            .arg("push")
            .arg(&format!("--force-with-lease={}:{}", branch, old_oid))
            .arg(repo.to_github_ssh_url())
            .arg(format!("{}:{}", new_oid, branch));
        self.run()?;
        Ok(())
    }
}
