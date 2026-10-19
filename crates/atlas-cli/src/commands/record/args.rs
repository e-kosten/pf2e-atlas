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
    #[command(about = "Read an Atlas reference; use source to inspect the complete Foundry DTO")]
    Get(Box<RecordGetOptions>),
    Resolve(Box<RecordResolveOptions>),
    #[command(
        about = "Dump the complete checked Foundry DTO, including embedded documents",
        long_about = "Dump a product record's complete checked Foundry DTO and source path/hash from the artifact, including embedded documents and retained invalid/additional fields. The checked serialization includes presence/union tags; it is not original Foundry JSON. This command needs neither the source clone nor embedding models. For hash-verified original files, use atlas-dev index record --original."
    )]
    Source(RecordSourceOptions),
}
#[derive(Debug, Args)]
pub(crate) struct RecordGetOptions {
    #[arg(long, value_parser = clap::value_parser!(u8).range(1..=10), help = "Preview a spell at this rank")]
    pub(crate) rank: Option<u8>,
    #[arg(required=true,num_args=1..,help="Foundry keys in pack:id form")]
    pub(crate) keys: Vec<String>,
    #[arg(
        long,
        value_enum,
        default_value = "standard",
        conflicts_with = "json",
        help = "Human detail: summary identity, standard reference, or full expanded embedded entries; conflicts with --json"
    )]
    pub(crate) detail: TerminalDetail,
    #[arg(long="embedded-path",help="Checked embedded path JSON from a search witness",value_parser=parse_owners)]
    pub(crate) owners: Option<OwnerChainArgument>,
    #[arg(long, help = "Selected field path, relative to the selected document")]
    pub(crate) field: Option<String>,
    #[arg(long,help="Exact passage address JSON from a search witness",value_parser=parse_passage,requires="field")]
    pub(crate) passage: Option<SourcePassageAddress>,
    #[arg(
        long,
        help = "Artifact source fingerprint accompanying snapshot-local embedded documents"
    )]
    pub(crate) source_fingerprint: Option<String>,
    #[arg(long)]
    pub(crate) index: Option<PathBuf>,
    #[arg(long,value_enum,default_value_t=CliPathMode::Global)]
    pub(crate) path_mode: CliPathMode,
    #[arg(
        long,
        help = "Emit the complete selected Atlas reference response; use record source for the Foundry DTO"
    )]
    pub(crate) json: bool,
}

#[derive(Debug, Args)]
pub(crate) struct RecordSourceOptions {
    #[arg(help = "Foundry root key in pack:id form; the dump includes its embedded documents")]
    pub(crate) key: String,
    #[arg(long)]
    pub(crate) index: Option<PathBuf>,
    #[arg(long, value_enum, default_value_t = CliPathMode::Global)]
    pub(crate) path_mode: CliPathMode,
    #[arg(
        long,
        help = "Wrap the checked snapshot and source provenance in the standard JSON envelope"
    )]
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
    Full,
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
