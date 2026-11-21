use super::super::saved_workspace::{SelectionOptions, WorkspaceOptions};
use atlas_domain::RecordKey;
use clap::{ArgGroup, Args, Subcommand};
use std::path::PathBuf;

#[derive(Debug, Args)]
#[command(
    after_help = "Examples:\n  atlas characters list --json\n  atlas characters show \"Mira\" --section equipment --section money --json\n  atlas characters show CHARACTER_KEY --section abilities --needs-review --json\n\nReads accept a stable key or exact unique name. Mutations require keys and reviewed revisions."
)]
pub(crate) struct CharactersArgs {
    #[command(flatten)]
    pub workspace: WorkspaceOptions,
    #[command(subcommand)]
    pub command: CharacterCommand,
}

#[derive(Debug, Subcommand)]
pub(crate) enum CharacterCommand {
    /// List saved characters, including PCs outside parties.
    #[command(alias = "ls")]
    List {
        #[arg(long)]
        party: Option<String>,
    },
    /// Read character details; all interpreted sections by default.
    Show {
        #[arg(add = clap_complete::ArgValueCompleter::new(crate::commands::completions::character_keys))]
        character: String,
        #[command(flatten)]
        selection: SelectionOptions,
        #[arg(
            long,
            help = "Write exact original export to a new file; requires only --section source"
        )]
        output: Option<PathBuf>,
    },
    /// Create a reusable manual character; retry with the same request id and inputs.
    Create(CreateOptions),
    /// Edit canonical name or level, preserving the imported build.
    Edit(EditOptions),
    /// Delete a character, removing memberships and detaching encounter rules.
    Delete {
        character: String,
        #[arg(long)]
        expected_revision: i64,
    },
    /// Review a proposed Pathbuilder import without saving; '-' reads stdin.
    Preview {
        input: PathBuf,
        #[arg(long, help = "Review a replacement build for this saved character key")]
        character: Option<String>,
        #[command(flatten)]
        selection: SelectionOptions,
    },
    /// Save a new character from the exact reviewed Pathbuilder export.
    Import(ImportOptions),
    /// Replace a saved character's name, level and import with a reviewed export.
    Refresh {
        character: String,
        input: PathBuf,
        #[arg(long)]
        expected_revision: i64,
        #[command(flatten)]
        review: ImportReviewOptions,
    },
    /// Correct an imported ability's rule link, or restore its available automatic match.
    Link(LinkOptions),
}

#[derive(Debug, Args)]
pub(crate) struct CreateOptions {
    pub name: String,
    #[arg(long, value_parser = clap::value_parser!(i64).range(1..=20))]
    pub level: Option<i64>,
    #[arg(
        long,
        help = "Unique action id; reuse it only when retrying identical inputs"
    )]
    pub request_id: String,
    #[arg(long, requires = "expected_party_revision")]
    pub party: Option<String>,
    #[arg(long, requires = "party")]
    pub expected_party_revision: Option<i64>,
}

#[derive(Debug, Args)]
#[command(group(ArgGroup::new("changes").required(true).multiple(true).args(["name", "level", "clear_level"])))]
pub(crate) struct EditOptions {
    pub character: String,
    #[arg(long)]
    pub expected_revision: i64,
    #[arg(long)]
    pub name: Option<String>,
    #[arg(long, conflicts_with = "clear_level", value_parser = clap::value_parser!(i64).range(1..=20))]
    pub level: Option<i64>,
    #[arg(long)]
    pub clear_level: bool,
}

#[derive(Debug, Clone)]
pub(crate) struct RuleLink(pub String, pub RecordKey);
impl std::str::FromStr for RuleLink {
    type Err = String;
    fn from_str(text: &str) -> Result<Self, Self::Err> {
        let (ability, record) = text
            .split_once('=')
            .ok_or("Use ABILITY_KEY=PACK:ID for a reviewed rule link.")?;
        if ability.is_empty() {
            return Err("An ability key is required.".into());
        }
        Ok(Self(
            ability.into(),
            RecordKey::parse(record).map_err(|e| e.to_string())?,
        ))
    }
}

#[derive(Debug, Args)]
pub(crate) struct ImportReviewOptions {
    #[arg(long, help = "Exact source_sha256 returned by characters preview")]
    pub expected_source_sha256: String,
    #[arg(long, help = "Artifact generation returned by characters preview")]
    pub expected_generation: String,
    #[arg(
        long = "rule-link",
        help = "Reviewed ABILITY_KEY=PACK:ID correction; repeat for distinct abilities"
    )]
    pub links: Vec<RuleLink>,
}

#[derive(Debug, Args)]
pub(crate) struct ImportOptions {
    pub input: PathBuf,
    #[arg(long)]
    pub request_id: String,
    #[arg(long, requires = "expected_party_revision")]
    pub party: Option<String>,
    #[arg(long, requires = "party")]
    pub expected_party_revision: Option<i64>,
    #[command(flatten)]
    pub review: ImportReviewOptions,
}

#[derive(Debug, Args)]
#[command(group(ArgGroup::new("selection").required(true).args(["record", "automatic"])))]
pub(crate) struct LinkOptions {
    pub character: String,
    #[arg(long)]
    pub ability: String,
    #[arg(long)]
    pub expected_revision: i64,
    #[arg(long, requires = "expected_generation", add = clap_complete::ArgValueCompleter::new(crate::commands::completions::record_keys))]
    pub record: Option<RecordKey>,
    #[arg(long, requires = "record")]
    pub expected_generation: Option<String>,
    #[arg(long)]
    pub automatic: bool,
}
