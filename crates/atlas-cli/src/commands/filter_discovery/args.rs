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
    #[arg(long)]
    pub(crate) index: Option<PathBuf>,
    #[arg(long,value_enum,default_value_t=CliPathMode::Global)]
    pub(crate) path_mode: CliPathMode,
    #[arg(long)]
    pub(crate) json: bool,
}
#[derive(Debug, Args)]
#[command(
    after_help = "Values retain all supplied filters; they do not replace a condition on the requested field.\nClosed fields include allowed choices with zero matches; open fields enumerate observed values.\nUse filters fields --field ID for the declared choices and CEL examples.\n\nExamples:\n  atlas filters values --field traits --kind spell\n  atlas filters values --field languages --kind creature --text common\n  atlas filters values --field 'movement.speeds[].type' --kind creature\n  atlas filters values --field defenses.hp.maximum --kind creature --json"
)]
pub(crate) struct FiltersValuesOptions {
    #[arg(long)]
    pub(crate) field: String,
    #[command(flatten)]
    pub(crate) filter_options: FilterOptions,
    #[arg(
        long,
        help = "Find categorical values by text; does not search record content"
    )]
    pub(crate) text: Option<String>,
    #[arg(long, default_value_t = 0)]
    pub(crate) offset: usize,
    #[arg(long, default_value_t = 50)]
    pub(crate) limit: usize,
    #[arg(long)]
    pub(crate) index: Option<PathBuf>,
    #[arg(long,value_enum,default_value_t=CliPathMode::Global)]
    pub(crate) path_mode: CliPathMode,
    #[arg(long)]
    pub(crate) json: bool,
}
