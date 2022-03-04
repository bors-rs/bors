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

1. Create a GitHub personal access token in the `bot` account. Add it to the bors config under `github/api-token`.
2. Create a SSH key in the `bot` account.  Provide this key locally on the bot's machine. Add it to the bors config under `git/ssh-key-file`.
3. Add a webhook for the repo that points to the bors server using the `/github` endpoint.  Configure it to use `application-json`, and provide the secret under `github/webhook-secret`


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
