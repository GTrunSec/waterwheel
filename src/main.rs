use anyhow::Result;
use waterwheel::{config, logging, server::Server, worker::Worker};

#[tokio::main]
async fn main() -> Result<()> {
    dotenv::dotenv().ok();

    let config = config::load()?;
    logging::setup(&config)?;

    let app = clap::App::new("waterwheel")
        .author("Steve Lee <sphen.lee@gmail.com>")
        .version(waterwheel::GIT_VERSION)
        .setting(clap::AppSettings::SubcommandRequiredElseHelp)
        .subcommand(
            clap::App::new("scheduler")
                .alias("server")
                .about("launch the scheduler process")
                .after_help(
                    "There should only be one scheduler active at a time.
                         The scheduler has an API server embedded.",
                ),
        )
        .subcommand(
            clap::App::new("api")
                .about("launch the API server process")
                .after_help("The API server may be launched many times for load balancing and HA"),
        )
        .subcommand(clap::App::new("worker").about("launch the worker process"));

    let args = app.get_matches();

    match args.subcommand() {
        ("scheduler", Some(_args)) => {
            let server = Server::new(config).await?;
            server.run_scheduler().await?;
        }
        ("api", Some(_args)) => {
            let server = Server::new(config).await?;
            server.run_api().await?;
        }
        ("worker", Some(_args)) => {
            let worker = Worker::new(config).await?;
            worker.run_worker().await?;
        }
        _ => unreachable!("clap should have already checked the subcommands"),
    }
}
