mod cli;
use mars_rover::start_server;
use clap::Parser;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    // TODO setup simple console output logger
    // For every connection, we must make a `Service` to handle all
    // incoming HTTP requests on said connection.

    let args = cli::Args::parse();
    let project_handler = args.get_project_manager().await;
    let addr = args.get_addr();

    simple_logger::SimpleLogger::new()
        .with_colors(true)
        .with_level(log::LevelFilter::Info)
        .init()?;

    start_server(addr, project_handler).await
}
