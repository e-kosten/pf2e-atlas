use std::process::ExitCode;

use atlas_cli_support::{CliProgressMode, ProgressOptions, init_tracing, write_json_error};
use clap::{Parser, Subcommand};

use crate::commands;
use crate::commands::index::args::{IndexArgs, IndexCommand};
use crate::commands::source::args::{SourceArgs, SourceCommand};

#[derive(Debug, Parser)]
#[command(
    name = "atlas-dev",
    version,
    about = "Atlas developer source and artifact diagnostics"
)]
#[command(
    after_help = "Examples:\n  atlas-dev source analyze --source vendor/pf2e --json\n  atlas-dev source audit-paths --record-type npc\n  atlas-dev index inspect --json"
)]
struct Cli {
    #[arg(long, global = true, value_enum, default_value_t = CliProgressMode::Auto,
        help_heading = "Output", help = "Control progress rendering: auto shows human progress on terminals, never suppresses routine progress, always forces it")]
    progress: CliProgressMode,
    #[command(subcommand)]
    command: Command,
}

#[derive(Debug, Subcommand)]
enum Command {
    #[command(about = "Analyze and audit Foundry source data")]
    Source(SourceArgs),
    #[command(about = "Inspect built Atlas artifacts")]
    Index(IndexArgs),
}

pub(crate) fn main() -> ExitCode {
    let cli = match Cli::try_parse() {
        Ok(cli) => cli,
        Err(error) => {
            if std::env::args().any(|arg| arg == "--json") {
                let _ = write_json_error("invalid_input", error.to_string());
                return ExitCode::from(2);
            }
            let _ = error.print();
            return ExitCode::from(error.exit_code() as u8);
        }
    };
    let json = match &cli.command {
        Command::Source(args) => match &args.command {
            SourceCommand::Load(options) => options.json,
            SourceCommand::Analyze(options) => options.json,
            SourceCommand::AuditPaths(options) => options.json,
        },
        Command::Index(args) => match &args.command {
            IndexCommand::Inspect(options) => options.json,
            IndexCommand::Record(options) => options.json,
        },
    };
    init_tracing(ProgressOptions {
        mode: cli.progress.into(),
        json,
        setup_timing: false,
    });
    let result = match cli.command {
        Command::Source(args) => match args.command {
            SourceCommand::Load(options) => commands::source::run_source_load(options),
            SourceCommand::Analyze(options) => commands::source::run_source_analyze(options),
            SourceCommand::AuditPaths(options) => commands::source::run_source_audit_paths(options),
        },
        Command::Index(args) => match args.command {
            IndexCommand::Inspect(options) => commands::index::run_index_inspect(options),
            IndexCommand::Record(options) => commands::index::run_index_record(options),
        },
    };
    match result {
        Ok(code) => code,
        Err(error) => {
            if json {
                let _ = write_json_error("command_failed", error);
            } else {
                eprintln!("{error}");
            }
            ExitCode::from(2)
        }
    }
}
