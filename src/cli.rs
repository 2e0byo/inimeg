use std::path::PathBuf;

/// Serve content.
#[derive(clap::Args)]
pub struct Serve {
    /// The certificate this server will use to authenticate itself to clients.
    #[arg(long)]
    pub certificate: PathBuf,
    /// The private key for this certificate.
    #[arg(long)]
    pub private_key: PathBuf,
    /// The port the server should listen on.
    #[arg(long, short, default_value_t = 1965)]
    pub port: usize,
    #[arg(long)]
    pub static_dir: Option<PathBuf>,
}

/// Inimeg, a Gemini server built from the ground up.
#[derive(clap::Parser)]
pub enum Cli {
    Serve(Serve),
}
