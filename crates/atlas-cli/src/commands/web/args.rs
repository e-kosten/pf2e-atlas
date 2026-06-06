use std::path::PathBuf;

use clap::Args;

use crate::cli::args::CliPathMode;

pub(crate) const DEFAULT_WEB_PORT: u16 = 4727;

#[derive(Debug, Args)]
#[command(
    after_help = "Examples:\n  atlas web\n  atlas web --open\n  atlas web --port 4728 --path-mode repo"
)]
pub(crate) struct WebArgs {
    #[arg(
        long,
        global = true,
        value_enum,
        default_value_t = CliPathMode::Global,
        help_heading = "Path Overrides",
        help = "Use global install paths by default, or force checkout-local developer paths with repo"
    )]
    pub(crate) path_mode: CliPathMode,
    #[arg(
        long,
        global = true,
        help_heading = "Path Overrides",
        help = "Override the PF2E source checkout path"
    )]
    pub(crate) source: Option<PathBuf>,
    #[arg(
        long,
        global = true,
        help_heading = "Path Overrides",
        help = "Override the embedding model cache root"
    )]
    pub(crate) embedding_cache_path: Option<PathBuf>,
    #[arg(
        long,
        global = true,
        help_heading = "Path Overrides",
        help = "Override the SQLite artifact path"
    )]
    pub(crate) index: Option<PathBuf>,
    #[arg(
        long,
        help = "Bind this exact localhost port instead of auto-selecting from the default"
    )]
    pub(crate) port: Option<u16>,
    #[arg(long, help = "Open the local web URL in the default browser")]
    pub(crate) open: bool,
}
