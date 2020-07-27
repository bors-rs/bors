use crate::{
    event_processor::EventProcessor,
    server::{Installation, Server},
    Config, Result,
};
use structopt::StructOpt;

#[derive(StructOpt)]
pub struct ServeOptions {
    #[structopt(long, default_value = "3000")]
    port: u16,

    #[structopt(long)]
    /// smee.io URL
    smee: Option<String>,
}

//TODO Make sure to join and await on all of the JoinHandles of the tasks that get spawned
pub async fn run_serve(config: Config, options: &ServeOptions) -> Result<()> {
    let Config { repo, github, git } = config;
    let mut builder = Server::builder();

    for repo in repo {
        let (tx, event_processor) = EventProcessor::new(repo.clone(), &github, &git)?;
        tokio::spawn(event_processor.start());

        let installation = Installation::new(repo, tx);
        builder.add_installation(installation);
    }

    if let Some(smee_uri) = &options.smee {
        builder.smee(Some(smee_uri.clone()));
    }

    let addr = ([0, 0, 0, 0], options.port).into();
    builder.serve(addr).await?;
    Ok(())
}
