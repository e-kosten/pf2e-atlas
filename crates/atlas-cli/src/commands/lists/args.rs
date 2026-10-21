use std::path::PathBuf;

use clap::{Args, Subcommand};

use atlas_cli_support::CliPathMode;

#[derive(Debug, Args)]
#[command(
    after_help = "Examples:\n  atlas lists create fey-options --name \"Fey Options\"\n  atlas lists add fey-options --record pathfinder-monster-core:lUBkzsSqMfQBczU1\n  atlas lists show fey-options\n\nResolve ambiguous names with `atlas record resolve <NAME>` and use a returned record key."
)]
pub(crate) struct ListsArgs {
    #[command(subcommand)]
    pub(crate) command: ListsCommand,
}

#[derive(Debug, Subcommand)]
pub(crate) enum ListsCommand {
    #[command(about = "Create a saved list")]
    Create(ListCreateOptions),
    #[command(name = "ls", alias = "list", about = "List saved lists")]
    Ls(ListLsOptions),
    #[command(about = "Show one saved list")]
    Show(ListShowOptions),
    #[command(about = "Add records to a saved list")]
    Add(ListAddOptions),
    #[command(about = "Export a saved list as a portable JSON document")]
    Export(ListExportOptions),
    #[command(about = "Import a saved list from a portable JSON document")]
    Import(ListImportOptions),
    #[command(about = "Edit saved-list metadata")]
    Edit(ListEditOptions),
    #[command(about = "Remove a record from a saved list")]
    Remove(ListRemoveOptions),
    #[command(about = "Set or clear a note on an existing list entry")]
    Note(ListNoteOptions),
    #[command(about = "Delete a saved list")]
    Delete(ListDeleteOptions),
}

#[derive(Debug, Args)]
#[command(
    after_help = "Example:\n  atlas lists create fey-options --name \"Fey Options\" --record pathfinder-monster-core:lUBkzsSqMfQBczU1 --record pathfinder-monster-core:Ehtm5k9iBYTvSUcZ"
)]
pub(crate) struct ListCreateOptions {
    #[arg(help = "Stable saved-list id, using lowercase letters, digits, and '-'")]
    pub(crate) slug: String,
    #[arg(long, help = "Human-friendly saved-list name")]
    pub(crate) name: String,
    #[arg(long, help = "Optional saved-list description")]
    pub(crate) description: Option<String>,
    #[arg(long = "tag", help = "Add a saved-list tag; repeat for multiple tags")]
    pub(crate) tags: Vec<String>,
    #[arg(
        long = "record",
        help = "Initial record key or strict name; repeat for multiple records"
    )]
    #[arg(add = clap_complete::ArgValueCompleter::new(crate::commands::completions::record_keys))]
    pub(crate) records: Vec<String>,
    #[command(flatten)]
    pub(crate) paths: ListsPathOptions,
    #[arg(long, help = "Emit the standard JSON envelope")]
    pub(crate) json: bool,
}

#[derive(Debug, Args)]
pub(crate) struct ListLsOptions {
    #[command(flatten)]
    pub(crate) paths: ListsPathOptions,
    #[arg(long, help = "Emit the standard JSON envelope")]
    pub(crate) json: bool,
}

#[derive(Debug, Args)]
pub(crate) struct ListShowOptions {
    #[arg(help = "Saved-list id", add = clap_complete::ArgValueCompleter::new(crate::commands::completions::list_slugs))]
    pub(crate) slug: String,
    #[arg(long, help = "Emit only saved-list record keys")]
    pub(crate) keys_only: bool,
    #[arg(long, help = "Omit record payloads from JSON item output")]
    pub(crate) no_records: bool,
    #[command(flatten)]
    pub(crate) paths: ListsPathOptions,
    #[arg(long, help = "Emit the standard JSON envelope")]
    pub(crate) json: bool,
}

#[derive(Debug, Args)]
#[command(after_help = "Example:\n  atlas lists add undead-research --record \"Skeleton Guard\"")]
pub(crate) struct ListAddOptions {
    #[arg(help = "Saved-list id", add = clap_complete::ArgValueCompleter::new(crate::commands::completions::list_slugs))]
    pub(crate) slug: String,
    #[arg(help = "Canonical record key, strict name, or verified alias", add = clap_complete::ArgValueCompleter::new(crate::commands::completions::record_keys))]
    pub(crate) record_refs: Vec<String>,
    #[arg(
        long = "record",
        help = "Record key or strict name; repeat for multiple records"
    )]
    #[arg(add = clap_complete::ArgValueCompleter::new(crate::commands::completions::record_keys))]
    pub(crate) records: Vec<String>,
    #[arg(long, help = "Read record refs from stdin, one per line")]
    pub(crate) stdin: bool,
    #[arg(long, help = "Apply this note to each added record")]
    pub(crate) note: Option<String>,
    #[command(flatten)]
    pub(crate) paths: ListsPathOptions,
    #[arg(long, help = "Emit the standard JSON envelope")]
    pub(crate) json: bool,
}

#[derive(Debug, Args)]
pub(crate) struct ListExportOptions {
    #[arg(help = "Saved-list id", add = clap_complete::ArgValueCompleter::new(crate::commands::completions::list_slugs))]
    pub(crate) slug: String,
    #[arg(long, help = "Write the export JSON document to this path")]
    pub(crate) output: Option<PathBuf>,
    #[command(flatten)]
    pub(crate) paths: ListsPathOptions,
}

#[derive(Debug, Args)]
pub(crate) struct ListImportOptions {
    #[arg(help = "Saved-list export JSON document")]
    pub(crate) input: PathBuf,
    #[arg(long, help = "Override the imported saved-list id")]
    pub(crate) id: Option<String>,
    #[arg(long, help = "Replace the target saved list if it already exists")]
    pub(crate) replace: bool,
    #[command(flatten)]
    pub(crate) paths: ListsPathOptions,
    #[arg(long, help = "Emit the standard JSON envelope")]
    pub(crate) json: bool,
}

#[derive(Debug, Args)]
pub(crate) struct ListEditOptions {
    #[arg(help = "Saved-list id or stable list key", add = clap_complete::ArgValueCompleter::new(crate::commands::completions::list_slugs))]
    pub(crate) list_ref: String,
    #[arg(long, help = "Set a new saved-list id")]
    pub(crate) id: Option<String>,
    #[arg(long, help = "Set a new saved-list name")]
    pub(crate) name: Option<String>,
    #[arg(long, help = "Set a new saved-list description")]
    pub(crate) description: Option<String>,
    #[arg(long, help = "Clear the saved-list description")]
    pub(crate) clear_description: bool,
    #[arg(
        long = "tag",
        help = "Replace saved-list tags; repeat for multiple tags"
    )]
    pub(crate) tags: Vec<String>,
    #[arg(long, help = "Remove all saved-list tags")]
    pub(crate) clear_tags: bool,
    #[command(flatten)]
    pub(crate) paths: ListsPathOptions,
    #[arg(long, help = "Emit the standard JSON envelope")]
    pub(crate) json: bool,
}

#[derive(Debug, Args)]
pub(crate) struct ListRemoveOptions {
    #[arg(help = "Saved-list id", add = clap_complete::ArgValueCompleter::new(crate::commands::completions::list_slugs))]
    pub(crate) slug: String,
    #[arg(help = "Canonical record key, strict name, or verified alias", add = clap_complete::ArgValueCompleter::new(crate::commands::completions::record_keys))]
    pub(crate) record_ref: String,
    #[command(flatten)]
    pub(crate) paths: ListsPathOptions,
    #[arg(long, help = "Emit the standard JSON envelope")]
    pub(crate) json: bool,
}

#[derive(Debug, Args)]
pub(crate) struct ListDeleteOptions {
    #[arg(help = "Saved-list id", add = clap_complete::ArgValueCompleter::new(crate::commands::completions::list_slugs))]
    pub(crate) slug: String,
    #[command(flatten)]
    pub(crate) paths: ListsPathOptions,
    #[arg(long, help = "Emit the standard JSON envelope")]
    pub(crate) json: bool,
}

#[derive(Debug, Args)]
pub(crate) struct ListsPathOptions {
    #[arg(
        long,
        help = "Override the SQLite workspace containing reference data and saved lists"
    )]
    pub(crate) index: Option<PathBuf>,
    #[arg(long, value_enum, default_value_t = CliPathMode::Global, help = "Use global runtime paths or checkout-local repo paths")]
    pub(crate) path_mode: CliPathMode,
}

#[derive(Debug, Args)]
#[command(
    after_help = "Examples:\n  atlas lists note fey-options pathfinder-monster-core:lUBkzsSqMfQBczU1 --text \"An ally for the garden encounter\"\n  atlas lists note fey-options pathfinder-monster-core:lUBkzsSqMfQBczU1 --clear"
)]
pub(crate) struct ListNoteOptions {
    #[arg(help = "Saved-list id", add = clap_complete::ArgValueCompleter::new(crate::commands::completions::list_slugs))]
    pub(crate) slug: String,
    #[arg(help = "Canonical record key of an existing list entry", add = clap_complete::ArgValueCompleter::new(crate::commands::completions::record_keys))]
    pub(crate) record_key: String,
    #[arg(
        long,
        required_unless_present = "clear",
        conflicts_with = "clear",
        help = "Set the entry note"
    )]
    pub(crate) text: Option<String>,
    #[arg(long, help = "Remove the entry note")]
    pub(crate) clear: bool,
    #[command(flatten)]
    pub(crate) paths: ListsPathOptions,
    #[arg(long, help = "Emit the standard JSON envelope")]
    pub(crate) json: bool,
}
