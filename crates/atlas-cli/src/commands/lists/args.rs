use std::path::PathBuf;

use clap::{Args, Subcommand, ValueEnum};

use crate::cli::args::CliPathMode;

#[derive(Debug, Args)]
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
    #[command(about = "Add a resolvable record to a saved list")]
    Add(ListAddOptions),
    #[command(about = "Export a saved list as a portable JSON document")]
    Export(ListExportOptions),
    #[command(about = "Import a saved list from a portable JSON document")]
    Import(ListImportOptions),
    #[command(about = "Edit saved-list metadata")]
    Edit(ListEditOptions),
    #[command(about = "Remove a record from a saved list")]
    Remove(ListRemoveOptions),
    #[command(about = "Delete a saved list")]
    Delete(ListDeleteOptions),
}

#[derive(Debug, Args)]
#[command(after_help = "Example:\n  atlas lists create undead-research --name \"Undead Research\"")]
pub(crate) struct ListCreateOptions {
    #[arg(help = "Stable saved-list id, using lowercase letters, digits, and '-'")]
    pub(crate) slug: String,
    #[arg(long, help = "Human-friendly saved-list name")]
    pub(crate) name: String,
    #[arg(long, help = "Optional saved-list description")]
    pub(crate) description: Option<String>,
    #[arg(long = "tag", help = "Add a saved-list tag; repeat for multiple tags")]
    pub(crate) tags: Vec<String>,
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
    #[arg(help = "Saved-list id")]
    pub(crate) slug: String,
    #[arg(
        long,
        help = "Emit compact item summaries instead of full record payloads"
    )]
    pub(crate) summary: bool,
    #[arg(long, help = "Emit only saved-list record keys")]
    pub(crate) keys_only: bool,
    #[arg(long, help = "Omit hydrated record payloads from item output")]
    pub(crate) no_records: bool,
    #[arg(
        long,
        value_enum,
        help = "Control record payload detail in JSON output"
    )]
    pub(crate) detail: Option<ListShowDetail>,
    #[command(flatten)]
    pub(crate) paths: ListsPathOptions,
    #[arg(long, help = "Emit the standard JSON envelope")]
    pub(crate) json: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
pub(crate) enum ListShowDetail {
    Preview,
    Standard,
    None,
}

#[derive(Debug, Args)]
#[command(after_help = "Example:\n  atlas lists add undead-research \"Skeleton Guard\"")]
pub(crate) struct ListAddOptions {
    #[arg(help = "Saved-list id")]
    pub(crate) slug: String,
    #[arg(help = "Canonical record key, strict name, or verified alias")]
    pub(crate) record_refs: Vec<String>,
    #[arg(long, help = "Read record refs from stdin, one per line")]
    pub(crate) stdin: bool,
    #[arg(long, help = "Optional note for this saved-list item")]
    pub(crate) note: Option<String>,
    #[command(flatten)]
    pub(crate) paths: ListsPathOptions,
    #[arg(long, help = "Emit the standard JSON envelope")]
    pub(crate) json: bool,
}

#[derive(Debug, Args)]
pub(crate) struct ListExportOptions {
    #[arg(help = "Saved-list id")]
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
    #[arg(help = "Saved-list id or stable list key")]
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
    #[arg(help = "Saved-list id")]
    pub(crate) slug: String,
    #[arg(help = "Canonical record key, strict name, or verified alias")]
    pub(crate) record_ref: String,
    #[command(flatten)]
    pub(crate) paths: ListsPathOptions,
    #[arg(long, help = "Emit the standard JSON envelope")]
    pub(crate) json: bool,
}

#[derive(Debug, Args)]
pub(crate) struct ListDeleteOptions {
    #[arg(help = "Saved-list id")]
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
        help = "Override the SQLite artifact path; local state resolves beside it"
    )]
    pub(crate) index: Option<PathBuf>,
    #[arg(long, value_enum, default_value_t = CliPathMode::Global, help = "Use global runtime paths or checkout-local repo paths")]
    pub(crate) path_mode: CliPathMode,
}
