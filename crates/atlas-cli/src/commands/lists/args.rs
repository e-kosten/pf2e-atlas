use std::path::PathBuf;

use clap::{Args, Subcommand};

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
    #[command(about = "Remove a record from a saved list")]
    Remove(ListRemoveOptions),
    #[command(about = "Delete a saved list")]
    Delete(ListDeleteOptions),
}

#[derive(Debug, Args)]
#[command(after_help = "Example:\n  atlas lists create undead-research --name \"Undead Research\"")]
pub(crate) struct ListCreateOptions {
    #[arg(help = "Stable saved-list slug, using lowercase letters, digits, and '-'")]
    pub(crate) slug: String,
    #[arg(long, help = "Human-friendly saved-list name")]
    pub(crate) name: String,
    #[arg(long, help = "Optional saved-list description")]
    pub(crate) description: Option<String>,
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
    #[arg(help = "Saved-list slug")]
    pub(crate) slug: String,
    #[command(flatten)]
    pub(crate) paths: ListsPathOptions,
    #[arg(long, help = "Emit the standard JSON envelope")]
    pub(crate) json: bool,
}

#[derive(Debug, Args)]
#[command(after_help = "Example:\n  atlas lists add undead-research \"Skeleton Guard\"")]
pub(crate) struct ListAddOptions {
    #[arg(help = "Saved-list slug")]
    pub(crate) slug: String,
    #[arg(help = "Canonical record key, strict name, or verified alias")]
    pub(crate) record_ref: String,
    #[arg(long, help = "Optional note for this saved-list item")]
    pub(crate) note: Option<String>,
    #[command(flatten)]
    pub(crate) paths: ListsPathOptions,
    #[arg(long, help = "Emit the standard JSON envelope")]
    pub(crate) json: bool,
}

#[derive(Debug, Args)]
pub(crate) struct ListRemoveOptions {
    #[arg(help = "Saved-list slug")]
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
    #[arg(help = "Saved-list slug")]
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
