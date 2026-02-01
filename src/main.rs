use anyhow::{Context, Result};
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
    pretty_env_logger::init();
    match cli {
        Cli::Serve(config) => {
            let mut server = server::Server::try_from(&config)?;
            if let Some(paths) = config.static_dirs {
                for path in paths {
                    let handler = handler::StaticHandler::new(
                        path.canonicalize()?,
                        path.file_stem().context("File stem")?.to_string_lossy(),
                    )?;
                    server.add_handler(Box::new(handler));
                }
            }
            if let Some(path) = config.root_dir {
                let handler = handler::StaticHandler::new(path.canonicalize()?, "/")?;
                server.add_handler(Box::new(handler));
            }
            server.run()?;
        }
    }
    Ok(())
}
