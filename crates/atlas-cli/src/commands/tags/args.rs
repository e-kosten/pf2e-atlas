use std::path::PathBuf;

use clap::{Args, Subcommand};

#[derive(Debug, Args)]
pub(crate) struct TagsArgs {
    #[command(subcommand)]
    pub(crate) command: TagsCommand,
}

#[derive(Debug, Subcommand)]
pub(crate) enum TagsCommand {
    #[command(about = "Validate authored tag catalog, assignments, and ontology suggestions")]
    Validate(TagsValidateOptions),
}

#[derive(Debug, Args)]
#[command(
    after_help = "Examples:\n  atlas tags validate\n  atlas tags validate --path data/tags --json"
)]
pub(crate) struct TagsValidateOptions {
    #[arg(
        long,
        default_value = "data/tags",
        help = "Tag corpus root containing catalog/, assignments/, and ontology-suggestions/"
    )]
    pub(crate) path: PathBuf,
    #[arg(long, help = "Emit the standard JSON envelope")]
    pub(crate) json: bool,
}
