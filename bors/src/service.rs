use crate::{event_processor::EventProcessor, Config, Result};
use probot::{Installation, Server};
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
    let mut installation = Installation::new(config.repo().owner(), config.repo().name());
    if let Some(secret) = &config.secret {
        installation.with_secret(secret);
    }

    let (tx, event_processor) = EventProcessor::new(config)?;
    tokio::spawn(event_processor.start());
    installation.with_service(Box::new(tx));

    let mut builder = Server::builder();
    if let Some(smee_uri) = &options.smee {
        builder = builder.smee(Some(smee_uri.clone()));
    }

    let addr = ([127, 0, 0, 1], options.port).into();
    builder.add_installation(installation).serve(addr).await?;
    Ok(())
}
