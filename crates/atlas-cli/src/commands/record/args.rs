use std::path::PathBuf;

use atlas_domain::DetailLevel;
use clap::{Args, Subcommand};

use crate::cli::args::{CliPathMode, FilterOptions};
use crate::cli::parse::{DETAIL_HELP, parse_detail_level};

#[derive(Debug, Args)]
pub(crate) struct RecordArgs {
    #[command(subcommand)]
    pub(crate) command: RecordCommand,
}

#[derive(Debug, Subcommand)]
pub(crate) enum RecordCommand {
    #[command(about = "Fetch one or more records by canonical record key")]
    Get(RecordGetOptions),
    #[command(about = "Resolve one or more strict record names or aliases")]
    Resolve(Box<RecordResolveOptions>),
}

#[derive(Debug, Args)]
#[command(
    after_help = "Examples:\n  atlas record get actionspf2e:1kGNdIIhuglAjIp9\n  atlas record get equipment-srd:s1vB3HdXjMigYAnY\n  atlas record get actionspf2e:1kGNdIIhuglAjIp9 --detail standard --json"
)]
pub(crate) struct RecordGetOptions {
    #[arg(required = true, num_args = 1.., help = "Canonical record keys in pack:id form; this command does not resolve names")]
    pub(crate) keys: Vec<String>,
    #[arg(long, value_parser = parse_detail_level, default_value = "standard", help = DETAIL_HELP)]
    pub(crate) detail: DetailLevel,
    #[arg(long, help = "Include raw source JSON in JSON output")]
    pub(crate) include_raw: bool,
    #[arg(long, help = "Override the SQLite artifact path")]
    pub(crate) index: Option<PathBuf>,
    #[arg(long, value_enum, default_value_t = CliPathMode::Global, help = "Use global runtime paths or checkout-local repo paths")]
    pub(crate) path_mode: CliPathMode,
    #[arg(long, help = "Emit the standard JSON envelope")]
    pub(crate) json: bool,
}

#[derive(Debug, Args)]
#[command(
    after_help = "Examples:\n  atlas record resolve \"Treat Wounds\" --pack-name actionspf2e\n  atlas record resolve \"Treat Wounds\" --alternatives 3 --json\n\nFilter discovery:\n  atlas filters fields\n  atlas filters values --field traits --kind rule"
)]
pub(crate) struct RecordResolveOptions {
    #[arg(required = true, num_args = 1.., help = "Strict record names or verified aliases to resolve")]
    pub(crate) queries: Vec<String>,
    #[arg(long, value_parser = parse_detail_level, default_value = "standard", help = DETAIL_HELP)]
    pub(crate) detail: DetailLevel,
    #[arg(
        long,
        help = "Canonical SearchFilterNode JSON used to narrow strict resolution"
    )]
    pub(crate) filter_json: Option<String>,
    #[command(flatten)]
    pub(crate) filter_options: FilterOptions,
    #[arg(
        long,
        default_value_t = 5,
        help = "Return up to this many alternatives when a strict query is ambiguous"
    )]
    pub(crate) alternatives: u8,
    #[arg(long, help = "Include raw source JSON in JSON output")]
    pub(crate) include_raw: bool,
    #[arg(long, help = "Override the SQLite artifact path")]
    pub(crate) index: Option<PathBuf>,
    #[arg(long, value_enum, default_value_t = CliPathMode::Global, help = "Use global runtime paths or checkout-local repo paths")]
    pub(crate) path_mode: CliPathMode,
    #[arg(long, help = "Emit the standard JSON envelope")]
    pub(crate) json: bool,
}
