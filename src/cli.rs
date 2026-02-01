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
    /// Static dirs to serve.
    ///
    /// The name of every dir will be used to filter incoming requests. For
    /// instance a dir at `foo/bar/tinylog` will handle incoming requests for
    /// `tinylog/` Any request with a query will be ignored by the static
    /// handler, permitting the addition of e.g. a search handler on top.
    #[arg(long)]
    pub static_dirs: Option<Vec<PathBuf>>,

    /// Static content to serve at the root. If you do this, the server is
    /// basically just a static file server, as all requests not including a
    /// query will be routed to this static dir.  (This may well be desirable.)
    #[arg(long)]
    pub root_dir: Option<PathBuf>,
}

/// Inimeg, a Gemini server built from the ground up.
#[derive(clap::Parser)]
pub enum Cli {
    Serve(Serve),
}
