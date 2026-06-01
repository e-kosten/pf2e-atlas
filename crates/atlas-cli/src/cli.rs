use std::process::ExitCode;

use clap::{CommandFactory, Parser, Subcommand};

use crate::cli::args::CliProgressMode;
use crate::commands::agent_skills::args::AgentArgs;
use crate::commands::completions::args::CompletionsArgs;
use crate::commands::filter_discovery::args::{FiltersArgs, FiltersCommand};
use crate::commands::graph::args::{GraphArgs, GraphCommand};
use crate::commands::index::args::{IndexArgs, IndexCommand};
use crate::commands::record::args::{RecordArgs, RecordCommand};
use crate::commands::search::args::SearchOptions;
use crate::commands::setup::args::SetupArgs;
use crate::commands::similar::args::SimilarOptions;
use crate::{commands, output, progress};

pub(crate) mod args;
pub(crate) mod parse;

#[derive(Debug, Parser)]
#[command(name = "atlas")]
#[command(version)]
#[command(about = "PF2e Atlas local search and index tooling")]
#[command(
    after_help = "Examples:\n  atlas setup\n  atlas setup --no-embeddings\n  atlas record get actionspf2e:1kGNdIIhuglAjIp9\n  atlas record resolve \"Treat Wounds\" --pack-name actionspf2e"
)]
pub(crate) struct Cli {
    #[arg(
        long,
        global = true,
        value_enum,
        default_value_t = CliProgressMode::Auto,
        help_heading = "Output",
        help = "Control progress rendering: auto shows human progress on terminals, never suppresses routine progress, always forces it"
    )]
    pub(crate) progress: CliProgressMode,
    #[command(subcommand)]
    pub(crate) command: Command,
}

#[derive(Debug, Subcommand)]
pub(crate) enum Command {
    #[command(about = "Install, repair, or check local Atlas runtime data")]
    Setup(SetupArgs),
    #[command(about = "Build, validate, inspect, and analyze Atlas indexes")]
    Index(IndexArgs),
    #[command(about = "Fetch and resolve Atlas records")]
    Record(RecordArgs),
    #[command(about = "Retrieve local record reference graph context")]
    Graph(GraphArgs),
    #[command(about = "Find records similar to a seed record")]
    Similar(Box<SimilarOptions>),
    #[command(about = "Run Atlas search commands")]
    Search(Box<SearchOptions>),
    #[command(about = "Discover filter fields and values")]
    Filters(FiltersArgs),
    #[command(about = "Install and inspect Atlas agent integrations")]
    Agent(AgentArgs),
    #[command(about = "Generate shell completion scripts")]
    Completions(CompletionsArgs),
}

pub(crate) fn main() -> ExitCode {
    let cli = match Cli::try_parse() {
        Ok(cli) => cli,
        Err(error) => {
            if std::env::args().any(|arg| arg == "--json") {
                let _ = output::write_json_error("invalid_input", error.to_string());
                return ExitCode::from(2);
            }
            let _ = error.print();
            return ExitCode::from(error.exit_code() as u8);
        }
    };
    progress::init_tracing(progress::ProgressOptions {
        mode: cli.progress.into(),
        json: cli.command.uses_json(),
        setup_timing: cli.command.uses_setup_timing(),
    });
    match run(cli) {
        Ok(code) => code,
        Err(error) => {
            eprintln!("{error}");
            ExitCode::from(2)
        }
    }
}

impl Command {
    fn uses_json(&self) -> bool {
        match self {
            Self::Setup(args) => args.paths.json,
            Self::Index(args) => match &args.command {
                IndexCommand::Analyze(options) => options.json,
                IndexCommand::Build(options) => options.json,
                IndexCommand::Check(options) => options.json,
                IndexCommand::Inspect(options) => options.json,
                IndexCommand::Validate(options) => options.json,
            },
            Self::Record(args) => match &args.command {
                RecordCommand::Get(options) => options.json,
                RecordCommand::Resolve(options) => options.json,
            },
            Self::Graph(args) => match &args.command {
                GraphCommand::Links(options) => options.json,
                GraphCommand::Uses(options) => options.json,
                GraphCommand::Variants(options) => options.json,
                GraphCommand::Remaster(options) => options.json,
            },
            Self::Similar(options) => options.json,
            Self::Search(options) => options.json,
            Self::Filters(filters) => match &filters.command {
                FiltersCommand::Fields(options) => options.json,
                FiltersCommand::Values(options) => options.json,
            },
            Self::Agent(args) => args.uses_json(),
            Self::Completions(_) => false,
        }
    }

    fn uses_setup_timing(&self) -> bool {
        matches!(self, Self::Setup(_))
    }
}

fn run(cli: Cli) -> Result<ExitCode, String> {
    match cli.command {
        Command::Setup(args) => commands::setup::run_setup(args),
        Command::Index(index) => match index.command {
            IndexCommand::Analyze(options) => commands::index::run_index_analyze(options),
            IndexCommand::Build(options) => commands::index::run_index_build(options),
            IndexCommand::Check(options) => commands::index::run_index_check(options),
            IndexCommand::Inspect(options) => commands::index::run_index_inspect(options),
            IndexCommand::Validate(options) => commands::index::run_index_validate(options),
        },
        Command::Record(record) => match record.command {
            RecordCommand::Get(options) => commands::record::run_record_get(options),
            RecordCommand::Resolve(options) => commands::record::run_record_resolve(*options),
        },
        Command::Graph(graph) => match graph.command {
            GraphCommand::Links(options) => commands::graph::run_graph_links(options),
            GraphCommand::Uses(options) => commands::graph::run_graph_uses(options),
            GraphCommand::Variants(options) => commands::graph::run_graph_variants(options),
            GraphCommand::Remaster(options) => commands::graph::run_graph_remaster(options),
        },
        Command::Similar(options) => commands::similar::run_similar(*options),
        Command::Search(options) => commands::search::run_search(*options),
        Command::Filters(filters) => match filters.command {
            FiltersCommand::Fields(options) => {
                commands::filter_discovery::run_filters_fields(*options)
            }
            FiltersCommand::Values(options) => {
                commands::filter_discovery::run_filters_values(*options)
            }
        },
        Command::Agent(agent) => commands::agent_skills::run_agent(agent),
        Command::Completions(args) => commands::completions::run_completions(args, Cli::command()),
    }
}
