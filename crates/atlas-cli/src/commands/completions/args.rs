use clap::Args;
use clap_complete::Shell;

#[derive(Debug, Args)]
#[command(after_help = "Examples:\n  atlas completions zsh\n  atlas completions bash")]
pub(crate) struct CompletionsArgs {
    #[arg(value_enum, help = "Shell to generate completions for")]
    pub(crate) shell: Shell,
}
