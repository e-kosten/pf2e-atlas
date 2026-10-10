use crate::cli::args::FilterOptions;
use atlas_cli_support::CliPathMode;
use atlas_domain::SourcePassageAddress;
use atlas_record::source_content::OwnedContentLocator;
use clap::{Args, Subcommand, ValueEnum};
use std::path::PathBuf;
#[derive(Debug, Args)]
pub(crate) struct RecordArgs {
    #[command(subcommand)]
    pub(crate) command: RecordCommand,
}
#[derive(Debug, Subcommand)]
pub(crate) enum RecordCommand {
    Get(Box<RecordGetOptions>),
    Resolve(Box<RecordResolveOptions>),
}
#[derive(Debug, Args)]
pub(crate) struct RecordGetOptions {
    #[arg(required=true,num_args=1..,help="Foundry keys in pack:id form")]
    pub(crate) keys: Vec<String>,
    #[arg(
        long,
        value_enum,
        default_value = "standard",
        help = "Terminal detail: summary or standard; JSON returns the full selected detail DTO"
    )]
    pub(crate) detail: TerminalDetail,
    #[arg(long,help="Checked owner chain JSON from a search witness",value_parser=parse_owners)]
    pub(crate) owners: Option<OwnerChainArgument>,
    #[arg(long, help = "Selected field path, relative to the requested owner")]
    pub(crate) field: Option<String>,
    #[arg(long,help="Exact passage address JSON from a search witness",value_parser=parse_passage,requires="field")]
    pub(crate) passage: Option<SourcePassageAddress>,
    #[arg(
        long,
        help = "Artifact source fingerprint accompanying snapshot-local owners"
    )]
    pub(crate) source_fingerprint: Option<String>,
    #[arg(long)]
    pub(crate) index: Option<PathBuf>,
    #[arg(long,value_enum,default_value_t=CliPathMode::Global)]
    pub(crate) path_mode: CliPathMode,
    #[arg(long)]
    pub(crate) json: bool,
}
#[derive(Debug, Args)]
pub(crate) struct RecordResolveOptions {
    #[arg(required=true,num_args=1..,help="Strict names or verified aliases")]
    pub(crate) queries: Vec<String>,
    #[command(flatten)]
    pub(crate) filter_options: FilterOptions,
    #[arg(long, default_value_t = 5)]
    pub(crate) alternatives: usize,
    #[arg(long)]
    pub(crate) index: Option<PathBuf>,
    #[arg(long,value_enum,default_value_t=CliPathMode::Global)]
    pub(crate) path_mode: CliPathMode,
    #[arg(long)]
    pub(crate) json: bool,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
pub(crate) enum TerminalDetail {
    Summary,
    Standard,
}
#[derive(Debug, Clone)]
pub(crate) struct OwnerChainArgument(pub(crate) Vec<OwnedContentLocator>);

fn parse_owners(value: &str) -> Result<OwnerChainArgument, String> {
    serde_json::from_str(value)
        .map(OwnerChainArgument)
        .map_err(|e| e.to_string())
}
fn parse_passage(value: &str) -> Result<SourcePassageAddress, String> {
    serde_json::from_str(value).map_err(|e| e.to_string())
}
