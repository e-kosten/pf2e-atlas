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
    #[command(about = "Discover closed choices, open value samples and availability counts")]
    Values(Box<FiltersValuesOptions>),
}
#[derive(Debug, Args)]
pub(crate) struct FiltersFieldsOptions {
    #[arg(long)]
    pub(crate) index: Option<PathBuf>,
    #[arg(long,value_enum,default_value_t=CliPathMode::Global)]
    pub(crate) path_mode: CliPathMode,
    #[arg(long)]
    pub(crate) json: bool,
}
#[derive(Debug, Args)]
#[command(
    after_help = "Examples:\n  atlas filters values --field traits --kind spell\n  atlas filters values --field actor.hp.maximum --kind creature --json"
)]
pub(crate) struct FiltersValuesOptions {
    #[arg(long)]
    pub(crate) field: String,
    #[command(flatten)]
    pub(crate) filter_options: FilterOptions,
    #[arg(long, help = "Filter sampled values by text")]
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
