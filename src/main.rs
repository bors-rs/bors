use bors::{Config, Database, Error, Service};
use futures::future;
use hyper::{
    server::conn::AddrStream,
    service::{make_service_fn, service_fn},
    Server,
};
use log::{error, info};
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
    Serve(ServeConfig),
}

#[derive(StructOpt)]
struct ServeConfig {
    #[structopt(long, default_value = "3000")]
    port: u16,
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    let opts = Options::from_args();

    // set up logging, allowing info level logging by default
    env_logger::from_env(env_logger::Env::default().default_filter_or("info")).init();

    info!("bors starting");

    let config = Config::from_file(&opts.config)?;
    info!("using database {}", config.database);

    let _db = Database::open(&config.database)?;

    match opts.command {
        Command::Serve(config) => {
            let addr = ([127, 0, 0, 1], config.port).into();

            let service = Service::new();

            // The closure inside `make_service_fn` is run for each connection,
            // creating a 'service' to handle requests for that specific connection.
            let make_service = make_service_fn(|socket: &AddrStream| {
                info!("remote address: {:?}", socket.remote_addr());

                // While the state was moved into the make_service closure,
                // we need to clone it here because this closure is called
                // once for every connection.
                let service = service.clone();

                // This is the `Service` that will handle the connection.
                future::ok::<_, Error>(service_fn(move |request| {
                    let service = service.clone();
                    service.serve(request)
                }))
            });

            let server = Server::bind(&addr).serve(make_service);

            info!("Listening on http://{}", addr);

            server.await?;

            Ok(())
        }
    }
}
