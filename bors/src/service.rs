use crate::{event_processor::EventProcessor, Config, Result};
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
    let secret = config.secret.clone();
    let (tx, event_processor) = EventProcessor::new(config)?;
    tokio::spawn(event_processor.start());

    let mut builder = probot::Server::builder();
    if let Some(smee_uri) = &options.smee {
        builder = builder.smee(Some(smee_uri.clone()));
    }

    if let Some(secret) = secret {
        builder = builder.secret(secret);
    }

    let addr = ([127, 0, 0, 1], options.port).into();
    builder.add_service(Box::new(tx)).serve(addr).await?;
    Ok(())
}
