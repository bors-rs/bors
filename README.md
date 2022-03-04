# A merge bot for GitHub pull requests

Bors is a Github App used to manage merging of PRs in order to ensure that CI is always green.
It does so by maintaining a Merge Queue.
Once a PR reaches the head of the Merge Queue it is rebased on top of the latest version of the PR's base-branch (generally master) and then triggers CI.
If CI comes back green the PR is then merged into the base-branch.
Regardless of the outcome, the next PR is the queue is then processed.


## Building

```
➜  bors git:(master) ✗ cargo build          
   Compiling bors v0.0.0 (/home/user/github.com/bors-rs/bors/bors)
    Finished dev [unoptimized + debuginfo] target(s) in 14.37s
```

## Testing

`cargo test`

### Setup + Configuration

Requires:
- GitHub personal access token
- ssh key added to github user account
- setup a webhook pointing to the bors server using the `/github` endpoint

1. Create a Github [application](https://docs.github.com/en/developers/apps/building-github-apps/creating-a-github-app) e.g. `bors-app`
   1. Put `bors-<your_name>` as the application name
   2. Put this repo's address as the homepage URL e.g. https://github.com/bors-rs/bors
   3. Activate webhook, and put the URL to your server/github e.g. https://my-bors.domain/github
   4. Repository Permissions
      * Actions - Read and Write
      * Checks - Read and Write
      * Commit statuses - Read and Write
      * Contents - Read and Write
      * Deployments - Read-only
      * Issues - Read and Write
      * Metadata - Read-only
      * Projects - Read and Write
      * Pull Requests - Read and Write
   5. Subscribe To Events
      * Meta
      * Check Run
      * Commit comment
      * Deployment Review
      * Issue Comment
      * Issues
      * Label
      * Project
      * Project Card
      * Project Column
      * Pull Request
      * Pull Request Review
      * Pull Request Review Comment
      * Pull Request Review Thread
      * Push
      * Repository
      * Status
      * Workflow Dispatch
      * Workflow Job
      * Workflow Run
2. Create a Github [machine user](https://docs.github.com/en/developers/overview/managing-deploy-keys#machine-users) e.g. bors
3. Create a GitHub personal access token in the `bot` account. Add it to the bors config under `github/api-token`.
4. Create an SSH key in the `bot` account.  Provide this key locally on the bot's machine. Add it to the bors config under `git/ssh-key-file`.
5. Add a webhook for the repo that points to the bors server using the `/github` endpoint.  Configure it to use `application-json`, and provide the secret under `github/webhook-secret`
6. Add any CI and appropriate SSH Keys.  CircleCI requires an SSH key for a machine user for multiple repos (e.g. the bot above).  Then, it can be added as dependent steps in the config.
7. Startup a server with appropriate commands that's configured to receive messages.  You can open the server's main page for status, and repo specific status by clicking on the repos.

### Running

```
➜  bors git:(jnaulty/fix-typo) ✗ cargo run -- -c bors.toml serve
   Compiling bors v0.0.0 (/home/jnaulty/github.com/bors-rs/bors/bors)
    Finished dev [unoptimized + debuginfo] target(s) in 13.74s
     Running `target/debug/bors -c bors.toml serve`
[2020-08-28T09:19:55Z INFO  bors] bors starting
[2020-08-28T09:19:55Z INFO  bors::server] Listening on http://0.0.0.0:3000
[2020-08-28T09:19:55Z INFO  bors::git] using existing on-disk repo at /home/jnaulty/github.com/bors-rs/bors/repos/jnaulty/spoon-fork-bors
[2020-08-28T09:19:55Z INFO  bors::event_processor] Synchronizing
[2020-08-28T09:19:56Z INFO  bors::event_processor] 1 Open PullRequests
[2020-08-28T09:19:58Z INFO  bors::event_processor] Done Synchronizing
```

### How does it work?

#### On commands
* Bors listens for pull request interactions (e.g. `/land`) in comments.
* Bors receives webhook messages and it scans for those interactions.
* Bors then will determine whether the PR has the appropriate approvals.
* After that, bors will move the commits to either the `auto`(for `/land`) or `canary`(for `/canary`) branches to run testing separately.
* Bors waits on webhook responses telling it that CI passed for the configured checks.
* It will then merge them into the `main` branch if it's a `/land` command or provide a summary for a `/canary` command.

#### State
* Bors uses a Github project to keep track of state.  It moves the PRs between stages to determine whether it is queued, testing, or in review.

##  Pull Request Interactions


### Commands
Bors actions can be triggered by posting a comment which includes a line of the form `/<action>`.
| Command | Action | Description |
| --- | --- | --- |
| __Land__ | `land`, `merge` | attempt to land or merge a PR |
| __Canary__ | `canary`, `try` | canary a PR by performing all checks without merging |
| __Cancel__ | `cancel`, `stop` | stop an in-progress land |
| __Cherry Pick__ | `cherry-pick <target>` | cherry-pick a PR into `<target>` branch |
| __Priority__ | `priority` | set the priority level for a PR (`high`, `normal`, `low`) |
| __Help__ | `help`, `h` | show this help message |

### Options
Options for Pull Requests are configured through the application of labels.
| &nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;Option&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp; | Description |
| --- | --- |
| ![label: bors-high-priority](https://img.shields.io/static/v1?label=&message=bors-high-priority&color=lightgrey) | Indicates that the PR is high-priority. When queued the PR will be placed at the head of the merge queue. |
| ![label: bors-low-priority](https://img.shields.io/static/v1?label=&message=bors-low-priority&color=lightgrey) | Indicates that the PR is low-priority. When queued the PR will be placed at the back of the merge queue. |
| ![label: bors-squash](https://img.shields.io/static/v1?label=&message=bors-squash&color=lightgrey) | Before merging the PR will be squashed down to a single commit, only retaining the commit message of the first commit in the PR. |
