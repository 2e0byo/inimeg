use std::path::PathBuf;

/// Serve content.
#[derive(clap::Args)]
pub struct Serve {
    /// The certificate this server will use to authenticate itself to clients.
    #[arg(long)]
    pub certificate: PathBuf,
    #[arg(long)]
    pub private_key: PathBuf,
    #[arg(long, short, default_value_t = 1965)]
    pub port: usize,
}

/// Inimeg, a Gemini server built from the ground up.
#[derive(clap::Parser)]
pub enum Cli {
    Serve(Serve),
}
