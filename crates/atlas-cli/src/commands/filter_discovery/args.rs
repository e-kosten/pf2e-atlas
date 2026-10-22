use crate::cli::args::FilterOptions;
use atlas_cli_support::CliPathMode;
use clap::{Args, Subcommand};
use std::path::PathBuf;
#[derive(Debug, Args)]
pub(crate) struct FiltersArgs {
    #[command(subcommand)]
    pub(crate) command: FiltersCommand,
}
#[derive(Debug, Subcommand)]
pub(crate) enum FiltersCommand {
    #[command(about = "Discover supported CEL paths, operators, applicability and units")]
    Fields(Box<FiltersFieldsOptions>),
    #[command(about = "Discover contextual values, record counts and source availability")]
    Values(Box<FiltersValuesOptions>),
}
#[derive(Debug, Args)]
#[command(
    after_help = "Examples:\n  atlas filters fields\n  atlas filters fields --group movement\n  atlas filters fields --field defenses.hp.maximum --json\n  atlas filters fields --text speed\n\nDefault output groups fields for browsing. Use --all for the flat query catalog. Link conditions use links_to(KEY[, SCOPE]) or linked_from(KEY[, SCOPE]); SCOPE is whole_entry (default) or main_entry."
)]
pub(crate) struct FiltersFieldsOptions {
    #[arg(long, conflicts_with_all = ["field", "text"], help = "Browse a product group; run filters fields to list groups")]
    pub(crate) group: Option<String>,
    #[arg(long, conflicts_with_all = ["field", "group"], help = "List all field descriptors")]
    pub(crate) all: bool,
    #[arg(
        long,
        help = "Inspect a field; human collection output also lists its entry fields"
    )]
    pub(crate) field: Option<String>,
    #[arg(
        long,
        help = "Find fields by label, ID, scope or applicable source family"
    )]
    pub(crate) text: Option<String>,
    #[arg(long, help = "Override the SQLite workspace path")]
    pub(crate) index: Option<PathBuf>,
    #[arg(long, value_enum, default_value_t = CliPathMode::Global, help = "Use global runtime paths or checkout-local repo paths")]
    pub(crate) path_mode: CliPathMode,
    #[arg(long, help = "Emit the standard JSON envelope")]
    pub(crate) json: bool,
}
#[derive(Debug, Args)]
#[command(
    after_help = "Values retain all supplied filters unless --clause-id requests replacement suggestions.\nClosed fields include allowed choices with zero matches; open fields enumerate observed values.\nNumeric and collection fields return statistics, without paged values.\nUse filters fields --field ID for the declared choices and CEL examples.\n\nExamples:\n  atlas filters values --field traits --kind spell\n  atlas filters values --field languages --kind creature --text common\n  atlas filters values --field 'movement.speeds[].type' --kind creature\n  atlas filters values --field defenses.hp.maximum --kind creature --json"
)]
pub(crate) struct FiltersValuesOptions {
    #[arg(
        long,
        help = "Record-content query constraining contextual discovery; distinct from --text"
    )]
    pub(crate) query: Option<String>,
    #[arg(long, value_enum, default_value_t = crate::commands::search::args::CliRetrievalMode::Hybrid, help = "Text retrieval for --query; unsupported count contexts return an explicit error")]
    pub(crate) retrieval: crate::commands::search::args::CliRetrievalMode,
    #[arg(long, value_parser=crate::cli::args::parse_search_profile, help="Stored search profile for query context")]
    pub(crate) search_profile: Option<atlas_domain::SearchProfileId>,
    #[arg(long = "list", help = "Restrict context to this saved list")]
    pub(crate) list_ref: Option<String>,
    #[arg(
        long,
        help = "Replace this clause for contextual suggestions; omit to retain all conditions"
    )]
    pub(crate) clause_id: Option<String>,
    #[arg(
        long,
        help = "Require the artifact generation returned by the preceding search"
    )]
    pub(crate) artifact_generation: Option<String>,
    #[arg(
        long,
        help = "Catalog field ID; discover IDs with atlas filters fields"
    )]
    pub(crate) field: String,
    #[command(flatten)]
    pub(crate) filter_options: FilterOptions,
    #[arg(
        long,
        help = "Find categorical values by text; does not search record content"
    )]
    pub(crate) text: Option<String>,
    #[arg(
        long,
        default_value_t = 0,
        help = "Values to skip; applies to categorical and Boolean fields"
    )]
    pub(crate) offset: usize,
    #[arg(
        long,
        default_value_t = 50,
        help = "Maximum categorical or Boolean values to return"
    )]
    pub(crate) limit: usize,
    #[arg(long, help = "Override the SQLite workspace path")]
    pub(crate) index: Option<PathBuf>,
    #[arg(long, value_enum, default_value_t = CliPathMode::Global, help = "Use global runtime paths or checkout-local repo paths")]
    pub(crate) path_mode: CliPathMode,
    #[arg(long, help = "Emit the standard JSON envelope")]
    pub(crate) json: bool,
}
