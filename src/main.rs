use bors::{run_serve, Config, Database, Error, ServeOptions};
use log::info;
use std::path::PathBuf;
use structopt::StructOpt;

#[derive(StructOpt)]
struct Options {
    #[structopt(short, long, parse(from_os_str), default_value = "bors.toml")]
    /// config file to use
    config: PathBuf,

    #[structopt(subcommand)]
    command: Command,
}

#[derive(StructOpt)]
enum Command {
    #[structopt(name = "serve")]
    /// Run the server
    Serve(ServeOptions),
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    let opts = Options::from_args();

    // set up logging, allowing info level logging by default
    env_logger::from_env(env_logger::Env::default().default_filter_or("info")).init();

    info!("bors starting");

    let config = Config::from_file(&opts.config)?;
    info!("using database {}", config.database);

    let db = Database::open(&config.database)?;

    match &opts.command {
        Command::Serve(options) => run_serve(config, &db, options).await,
    }
}
