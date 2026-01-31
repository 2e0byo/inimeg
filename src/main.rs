use anyhow::Result;
use clap::Parser;
use cli::Cli;

mod cli;
mod handler;
#[allow(dead_code)]
mod request;
#[allow(dead_code)]
mod response;
mod server;
mod status;

fn main() -> Result<()> {
    let cli = cli::Cli::parse();
    match cli {
        Cli::Serve(config) => {
            let mut server = server::Server::try_from(&config)?;
            if let Some(path) = config.static_dir {
                let handler = handler::StaticHandler::new(path.canonicalize()?)?;
                server.add_handler(Box::new(handler));
            }
            server.run()?;
        }
    }
    Ok(())
}
