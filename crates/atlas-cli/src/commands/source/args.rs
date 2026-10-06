use std::path::PathBuf;

use clap::{Args, Subcommand};

use crate::cli::args::CliPathMode;

#[derive(Debug, Args)]
pub(crate) struct SourceArgs {
    #[command(subcommand)]
    pub(crate) command: SourceCommand,
}

#[derive(Debug, Subcommand)]
pub(crate) enum SourceCommand {
    #[command(about = "Discover Foundry input shapes and compare source-schema snapshots")]
    Schema(SourceSchemaOptions),
    #[command(
        about = "Inspect values, frequencies, and source references for a Foundry input path"
    )]
    Values(SourceValuesOptions),
    #[command(about = "Analyze Foundry source ingest without writing SQLite")]
    Analyze(AnalyzeSourceOptions),
}

#[derive(Debug, Args)]
#[command(
    after_help = "Examples:\n  atlas source schema --record-type npc --min-records 10\n  atlas source schema --json > schema.json\n  atlas source schema --strict --baseline schema.json --json"
)]
pub(crate) struct SourceSchemaOptions {
    #[command(flatten)]
    pub(crate) selection: SourceSelectionOptions,
    #[arg(
        long,
        default_value_t = 1,
        help = "Only include paths present on at least this many records"
    )]
    pub(crate) min_records: usize,
    #[arg(
        long,
        default_value_t = 0,
        help = "Maximum paths to emit; 0 means all paths (required for complete snapshots)"
    )]
    pub(crate) limit: usize,
    #[arg(long, help = "Fail with exit 3 on schema changes; requires --baseline")]
    pub(crate) strict: bool,
    #[arg(
        long,
        help = "Compare a complete schema snapshot and expose added/removed paths, JSON type changes, and duplicate-member changes"
    )]
    pub(crate) baseline: Option<PathBuf>,
    #[arg(long, help = "Emit the standard JSON envelope")]
    pub(crate) json: bool,
}

#[derive(Debug, Args)]
#[command(
    after_help = "Examples:\n  atlas source analyze\n  atlas source analyze --source vendor/pf2e --manifest vendor/pf2e/static/system.json --json"
)]
pub(crate) struct AnalyzeSourceOptions {
    #[arg(long, help = "Override the PF2E source checkout path")]
    pub(crate) source: Option<PathBuf>,
    #[arg(long, value_enum, default_value_t = CliPathMode::Global, help = "Use global runtime paths or checkout-local repo paths")]
    pub(crate) path_mode: CliPathMode,
    #[arg(
        long,
        help = "Read Foundry pack declarations from this source manifest"
    )]
    pub(crate) manifest: Option<PathBuf>,
    #[arg(long, help = "Emit the standard JSON envelope")]
    pub(crate) json: bool,
}

#[derive(Debug, Args)]
pub(crate) struct SourceSelectionOptions {
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
}

#[derive(Debug, Args)]
#[command(
    after_help = "Examples:\n  atlas source values --record-type spell --path '$.system.traits.value[]'\n  atlas source values --source vendor/pf2e --record-type npc --path '$.system.attributes.hp.max' --json"
)]
pub(crate) struct SourceValuesOptions {
    #[command(flatten)]
    pub(crate) selection: SourceSelectionOptions,
    #[arg(
        long,
        help = "Exact normalized path from source schema; arrays use [] and known keyed maps use *"
    )]
    pub(crate) path: String,
    #[arg(
        long,
        default_value_t = 3,
        help = "Maximum concrete source references per distinct value; 0 disables samples"
    )]
    pub(crate) sample_limit: usize,
    #[arg(
        long,
        default_value_t = 0,
        help = "Maximum distinct values to display per document family; 0 means all. Counts always cover the full scan"
    )]
    pub(crate) limit: usize,
    #[arg(
        long,
        help = "Emit the standard JSON envelope with complete untruncated values"
    )]
    pub(crate) json: bool,
}
