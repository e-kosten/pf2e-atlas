use crate::cli::args::FilterOptions;
use atlas_cli_support::CliPathMode;
use atlas_domain::OwnedContentLocator;
use atlas_domain::SourcePassageAddress;
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
    #[command(about = "Resolve exact names and verified aliases to record keys")]
    Resolve(Box<RecordResolveOptions>),
    #[command(
        about = "Dump the complete checked Foundry DTO, including embedded documents",
        long_about = "Dump a product record's complete checked Foundry DTO and source path/hash from the artifact, including embedded documents and retained invalid/additional fields. The checked serialization includes presence/union tags; it is not original Foundry JSON. This command needs neither the source clone nor embedding models. For hash-verified original files, use atlas-dev index record --original."
    )]
    Source(RecordSourceOptions),
    /// Read a trait or vocabulary definition by exact key.
    Definition(super::definition::DefinitionOptions),
    /// Roll a published table by exact name, record key or reviewed navigation.
    RollTable(Box<RollTableOptions>),
    /// Show suggested variants with naming evidence.
    Variants(VariantsOptions),
}
#[derive(Debug, Args)]
#[command(
    after_help = "Examples:\n  atlas record get actionspf2e:1kGNdIIhuglAjIp9\n  atlas record get actionspf2e:1kGNdIIhuglAjIp9 --detail full\n  atlas record get actionspf2e:1kGNdIIhuglAjIp9 --json\n\nFor a name, use atlas record resolve \"Treat Wounds\" and open a returned key."
)]
pub(crate) struct RecordGetOptions {
    #[arg(long, value_parser = clap::value_parser!(u8).range(1..=10), help = "Preview a spell at this rank")]
    pub(crate) rank: Option<u8>,
    #[arg(required_unless_present="navigation",num_args=1..,help="Foundry keys in pack:id form")]
    pub(crate) keys: Vec<String>,
    #[arg(
        long,
        value_enum,
        default_value = "standard",
        help = "Summary identity, standard reference, or full reference detail"
    )]
    pub(crate) detail: TerminalDetail,
    #[arg(long, value_enum, conflicts_with_all=["field","passage"], help="Select a reference section; repeat to combine")]
    pub(crate) section: Vec<RecordSection>,
    #[arg(long, value_parser=parse_navigation, conflicts_with_all=["keys","owners","rank","field","passage","source_fingerprint"], help="Open an exact generation-bound navigation object returned by Atlas")]
    pub(crate) navigation: Option<Box<super::navigation::Locator>>,
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
    #[arg(long, help = "Override the SQLite workspace path")]
    pub(crate) index: Option<PathBuf>,
    #[arg(long, value_enum, default_value_t = CliPathMode::Global, help = "Use global runtime paths or checkout-local repo paths")]
    pub(crate) path_mode: CliPathMode,
    #[arg(
        long,
        help = "Emit the selected reference as JSON; use record source for the checked Foundry DTO"
    )]
    pub(crate) json: bool,
}

#[derive(Debug, Args)]
pub(crate) struct RecordSourceOptions {
    #[arg(help = "Foundry root key in pack:id form; the dump includes its embedded documents")]
    pub(crate) key: String,
    #[arg(long, help = "Override the SQLite workspace path")]
    pub(crate) index: Option<PathBuf>,
    #[arg(long, value_enum, default_value_t = CliPathMode::Global, help = "Use global runtime paths or checkout-local repo paths")]
    pub(crate) path_mode: CliPathMode,
    #[arg(
        long,
        help = "Wrap the checked snapshot and source provenance in the standard JSON envelope"
    )]
    pub(crate) json: bool,
}
#[derive(Debug, Args)]
#[command(
    after_help = "Example:\n  atlas record resolve \"Treat Wounds\" --pack-name actionspf2e --json\n\nAmbiguous names require choosing a returned key. Use atlas search for partial or fuzzy names."
)]
pub(crate) struct RecordResolveOptions {
    #[arg(required=true,num_args=1..,help="Strict names or verified aliases")]
    pub(crate) queries: Vec<String>,
    #[command(flatten)]
    pub(crate) filter_options: FilterOptions,
    #[arg(
        long,
        default_value_t = 5,
        help = "Maximum candidates for ambiguous names; a unique match is always returned"
    )]
    pub(crate) alternatives: usize,
    #[arg(long, help = "Override the SQLite workspace path")]
    pub(crate) index: Option<PathBuf>,
    #[arg(long, value_enum, default_value_t = CliPathMode::Global, help = "Use global runtime paths or checkout-local repo paths")]
    pub(crate) path_mode: CliPathMode,
    #[arg(long, help = "Emit the standard JSON envelope")]
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

#[derive(Debug, Args)]
#[command(
    after_help = "Examples:\n  atlas record variants \"Dread Ampoule (Moderate)\"\n  atlas record variants equipment-srd:IvFEJqp2MUew65nQ --json"
)]
pub(crate) struct VariantsOptions {
    #[arg(help = "Seed record key or strict resolvable record name")]
    pub(crate) record_ref: String,
    #[arg(long, help = "Override the SQLite artifact path")]
    pub(crate) index: Option<PathBuf>,
    #[arg(long, value_enum, default_value_t = CliPathMode::Global, help = "Use global runtime paths or checkout-local repo paths")]
    pub(crate) path_mode: CliPathMode,
    #[arg(long, help = "Emit the standard JSON envelope")]
    pub(crate) json: bool,
}

#[derive(Debug, Args)]
pub(crate) struct RollTableOptions {
    #[arg(
        required_unless_present = "navigation",
        conflicts_with = "navigation",
        help = "Canonical table key or exact name"
    )]
    pub(crate) record_ref: Option<String>,
    #[arg(long, value_parser=parse_navigation, help="Reviewed table navigation from a record or search result")]
    pub(crate) navigation: Option<Box<super::navigation::Locator>>,
    #[arg(long, help = "Override the SQLite workspace path")]
    pub(crate) index: Option<PathBuf>,
    #[arg(long, value_enum, default_value_t=CliPathMode::Global)]
    pub(crate) path_mode: CliPathMode,
    #[arg(long)]
    pub(crate) json: bool,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, ValueEnum, serde::Serialize)]
#[serde(rename_all = "snake_case")]
pub(crate) enum RecordSection {
    Rules,
    Defenses,
    Strikes,
    Abilities,
    Spellcasting,
}

impl RecordSection {
    pub(super) fn read_section(self) -> atlas_app_service::RecordReadSection {
        use atlas_app_service::RecordReadSection as Read;
        match self {
            Self::Rules => Read::Rules,
            Self::Defenses => Read::Defenses,
            Self::Strikes => Read::Strikes,
            Self::Abilities => Read::Abilities,
            Self::Spellcasting => Read::Spellcasting,
        }
    }
    pub(super) fn name(self) -> &'static str {
        match self {
            Self::Rules => "rules",
            Self::Defenses => "defenses",
            Self::Strikes => "strikes",
            Self::Abilities => "abilities",
            Self::Spellcasting => "spellcasting",
        }
    }
}

fn parse_navigation(text: &str) -> Result<Box<super::navigation::Locator>, String> {
    super::navigation::Locator::parse(text).map(Box::new)
}
