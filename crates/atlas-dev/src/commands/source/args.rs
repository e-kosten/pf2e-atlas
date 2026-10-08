use std::path::PathBuf;

use atlas_cli_support::CliPathMode;
use clap::{Args, Subcommand};

#[derive(Debug, Args)]
pub(crate) struct SourceArgs {
    #[command(subcommand)]
    pub(crate) command: SourceCommand,
}

#[derive(Debug, Subcommand)]
pub(crate) enum SourceCommand {
    #[command(about = "Load generated source DTOs before Atlas normalization or storage")]
    Load(LoadOptions),
    #[command(about = "Analyze Foundry source ingest without writing SQLite")]
    Analyze(AnalyzeOptions),
    #[command(about = "Audit raw Foundry JSON paths and known ingest coverage")]
    AuditPaths(AuditPathsOptions),
}

#[derive(Debug, Args)]
#[command(
    after_help = "Examples:\n  atlas-dev source load --source vendor/pf2e\n  atlas-dev source load --source vendor/pf2e --json"
)]
pub(crate) struct LoadOptions {
    #[arg(long, help = "Override the PF2E source checkout path")]
    pub(crate) source: Option<PathBuf>,
    #[arg(long, value_enum, default_value_t = CliPathMode::Global, help = "Use global runtime paths or checkout-local repo paths")]
    pub(crate) path_mode: CliPathMode,
    #[arg(long, help = "Override the Foundry manifest path")]
    pub(crate) manifest: Option<PathBuf>,
    #[arg(
        long,
        help = "Emit the loading summary and every diagnostic in the standard JSON envelope"
    )]
    pub(crate) json: bool,
}

#[derive(Debug, Args)]
#[command(
    after_help = "Examples:\n  atlas-dev source audit-paths --record-type npc --min-records 10\n  atlas-dev source audit-paths --pack-name pathfinder-bestiary --json"
)]
pub(crate) struct AuditPathsOptions {
    #[arg(long, help = "Override the PF2E source checkout path")]
    pub(crate) source: Option<PathBuf>,
    #[arg(long, value_enum, default_value_t = CliPathMode::Global, help = "Use global runtime paths or checkout-local repo paths")]
    pub(crate) path_mode: CliPathMode,
    #[arg(long, help = "Override the Foundry manifest path")]
    pub(crate) manifest: Option<PathBuf>,
    #[arg(long, help = "Only scan one manifest pack name")]
    pub(crate) pack_name: Option<String>,
    #[arg(long, help = "Only scan packs with this Foundry document type")]
    pub(crate) document_type: Option<String>,
    #[arg(long, help = "Only scan records with this Foundry record type")]
    pub(crate) record_type: Option<String>,
    #[arg(
        long,
        default_value_t = 1,
        help = "Only include paths present on at least this many records"
    )]
    pub(crate) min_records: usize,
    #[arg(long, default_value_t = 50, help = "Maximum paths to print or emit")]
    pub(crate) limit: usize,
    #[arg(long, help = "Emit the standard JSON envelope")]
    pub(crate) json: bool,
}

#[derive(Debug, Args)]
#[command(
    after_help = "Examples:\n  atlas-dev source analyze\n  atlas-dev source analyze --source vendor/pf2e --manifest vendor/pf2e/static/system.json --json"
)]
pub(crate) struct AnalyzeOptions {
    #[arg(long, help = "Override the PF2E source checkout path")]
    pub(crate) source: Option<PathBuf>,
    #[arg(long, value_enum, default_value_t = CliPathMode::Global, help = "Use global runtime paths or checkout-local repo paths")]
    pub(crate) path_mode: CliPathMode,
    #[arg(long, help = "Override the Foundry manifest path")]
    pub(crate) manifest: Option<PathBuf>,
    #[arg(long, help = "Emit the standard JSON envelope")]
    pub(crate) json: bool,
}
