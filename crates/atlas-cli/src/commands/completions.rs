use std::process::ExitCode;

use clap::Command;
use clap_complete::generate;

use args::CompletionsArgs;

pub(crate) mod args;

pub(crate) fn run_completions(
    args: CompletionsArgs,
    mut command: Command,
) -> Result<ExitCode, String> {
    generate(args.shell, &mut command, "atlas", &mut std::io::stdout());
    Ok(ExitCode::SUCCESS)
}
