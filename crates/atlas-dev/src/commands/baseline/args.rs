use clap::{Args, Subcommand};
use std::path::PathBuf;

#[derive(Debug, Args)]
#[command(
    after_help = "Examples:\n  atlas-dev baseline init\n  atlas-dev baseline status --binary target/debug/atlas --json\n  atlas-dev baseline web --port 4728\n\nInit creates only a missing private workspace. Status and web use the frozen baseline executable unless --binary selects a candidate."
)]
pub(crate) struct BaselineArgs {
    #[arg(
        long,
        global = true,
        default_value = ".",
        help = "Git checkout to inspect or initialize; relative paths resolve from its root"
    )]
    pub checkout: PathBuf,
    #[arg(
        long,
        global = true,
        help = "Override the baseline store (default: .atlas/dev-baselines in the primary checkout)"
    )]
    pub store: Option<PathBuf>,
    #[arg(
        long,
        global = true,
        help = "Emit the baseline report as JSON; web streams server output"
    )]
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
    #[arg(
        long,
        help = "Baseline version ID; default: current published version (existing workspace receipts take precedence)"
    )]
    pub baseline: Option<String>,
}
#[derive(Debug, Args)]
pub(crate) struct Candidate {
    #[command(flatten)]
    pub selection: Selection,
    #[arg(
        long,
        help = "Candidate Atlas executable; default: frozen baseline executable"
    )]
    pub binary: Option<PathBuf>,
    #[arg(
        long,
        default_value_t = 4727,
        help = "Local web port, also used in the status report command"
    )]
    pub port: u16,
}
#[derive(Debug, Args)]
pub(crate) struct PublishArgs {
    #[arg(
        long,
        help = "Existing Atlas executable to capture; no compilation is performed"
    )]
    pub binary: PathBuf,
    #[arg(long, help = "Workspace database to snapshot and validate")]
    pub index: PathBuf,
    #[arg(long, help = "Git commit identifying the supplied executable build")]
    pub build_commit: String,
    #[arg(long, value_parser = ["dev", "debug-full", "release", "dist"], help = "Cargo profile used for the supplied executable")]
    pub build_profile: String,
    #[arg(long, help = "Rust toolchain used for the supplied executable")]
    pub toolchain: String,
    #[arg(
        long,
        help = "Shared PF2e source checkout outside disposable worktrees"
    )]
    pub source: Option<PathBuf>,
    #[arg(
        long,
        help = "Shared embedding model cache outside disposable worktrees"
    )]
    pub embedding_cache_path: Option<PathBuf>,
}
