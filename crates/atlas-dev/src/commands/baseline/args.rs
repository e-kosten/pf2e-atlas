use clap::{Args, Subcommand};
use std::path::PathBuf;

#[derive(Debug, Args)]
pub(crate) struct BaselineArgs {
    #[arg(long, global = true, default_value = ".")]
    pub checkout: PathBuf,
    #[arg(long, global = true)]
    pub store: Option<PathBuf>,
    #[arg(long, global = true)]
    pub json: bool,
    #[command(subcommand)]
    pub command: BaselineCommand,
}
#[derive(Debug, Subcommand)]
pub(crate) enum BaselineCommand {
    /// Capture and validate an existing executable and workspace.
    Publish(PublishArgs),
    /// Seed only a missing private workspace. Never migrate or reset.
    Init(Selection),
    /// Report provenance and candidate compatibility without changing data.
    Status(Candidate),
    /// Start a backend using private data and shared source/model assets.
    Web(Candidate),
}
#[derive(Debug, Args)]
pub(crate) struct Selection {
    #[arg(long)]
    pub baseline: Option<String>,
}
#[derive(Debug, Args)]
pub(crate) struct Candidate {
    #[command(flatten)]
    pub selection: Selection,
    #[arg(long)]
    pub binary: Option<PathBuf>,
    #[arg(long, default_value_t = 4727)]
    pub port: u16,
}
#[derive(Debug, Args)]
pub(crate) struct PublishArgs {
    #[arg(long)]
    pub binary: PathBuf,
    #[arg(long)]
    pub index: PathBuf,
    #[arg(long)]
    pub build_commit: String,
    #[arg(long, value_parser = ["dev", "debug-full", "release", "dist"])]
    pub build_profile: String,
    #[arg(long)]
    pub toolchain: String,
    #[arg(long)]
    pub source: Option<PathBuf>,
    #[arg(long)]
    pub embedding_cache_path: Option<PathBuf>,
}
