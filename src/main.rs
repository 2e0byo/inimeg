use anyhow::Result;
use clap::Parser;
use cli::Cli;

mod cli;
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
            let server = server::Server::try_from(config)?;
            server.run()?;
        }
    }
    Ok(())
}
