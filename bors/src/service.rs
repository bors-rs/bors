use crate::{
    config::{GitConfig, GithubConfig, RepoConfig},
    event_processor::EventProcessor,
    server::{Installation, Server, SmeeClient},
    Config, Result,
};
use futures::future::try_join_all;
use structopt::StructOpt;

#[derive(StructOpt)]
pub struct ServeOptions {
    #[structopt(long, default_value = "3000")]
    port: u16,

    #[structopt(long)]
    /// smee.io URL
    smee: Option<String>,
}

pub async fn run_serve(config: Config, options: &ServeOptions) -> Result<()> {
    let mut tasks = Vec::new();
    let server = Server::new();

    // Start up the server and optionally a smee client
    let addr = ([0, 0, 0, 0], options.port).into();
    tasks.push(tokio::spawn(server.clone().start(addr)));

    if let Some(smee_uri) = &options.smee {
        let smee_client = SmeeClient::with_uri(smee_uri.clone(), server.clone());
        let smee_handle = tokio::spawn(smee_client.start());
        tasks.push(smee_handle);
    }

    // Start up all of the configured repos
    let Config { repo, github, git } = config;
    for repo in repo {
        let github = github.clone();
        let git = git.clone();
        let server = server.clone();
        tasks.push(tokio::spawn(start_event_processor(
            server, repo, github, git,
        )));
    }

    // Join all of the spawned tasks
    try_join_all(tasks).await?;
    Ok(())
}

async fn start_event_processor(
    mut server: Server,
    repo: RepoConfig,
    github: GithubConfig,
    git: GitConfig,
) -> Result<()> {
    let repo_clone = repo.clone();
    let (tx, event_processor) =
        tokio::task::spawn_blocking(move || EventProcessor::new(repo_clone, &github, &git))
            .await??;
    tokio::spawn(event_processor.start());

    let installation = Installation::new(repo, tx);
    server.add_installation(installation).await;

    Ok(())
}
