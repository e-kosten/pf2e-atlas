use atlas_cli_support::CliPathMode;
use clap::{ArgGroup, Args, Subcommand, ValueEnum};
use std::path::PathBuf;

#[derive(Debug, Args)]
pub(crate) struct SimilarityArgs {
    #[command(subcommand)]
    pub command: SimilarityCommand,
}
impl SimilarityArgs {
    pub fn json(&self) -> bool {
        match &self.command {
            SimilarityCommand::Evaluate(options) => options.json,
        }
    }
}
#[derive(Debug, Subcommand)]
pub(crate) enum SimilarityCommand {
    /// Run production captures, exact fixed-pool/oracle scoring or declared warm timings.
    Evaluate(EvaluateArgs),
}
#[derive(Debug, Clone, Copy, ValueEnum, PartialEq, Eq)]
pub(crate) enum Phase {
    Capture,
    FixedPool,
    Oracle,
    Performance,
    All,
}
#[derive(Debug, Args)]
#[command(group(ArgGroup::new("scenario_selection").required(true).args(["scenarios", "all_scenarios"])))]
pub(crate) struct EvaluateArgs {
    #[arg(long)]
    pub manifest: PathBuf,
    #[arg(long)]
    pub corpus: PathBuf,
    #[arg(
        long,
        help = "Input integrity receipt; default: freeze.json beside manifest"
    )]
    pub freeze: Option<PathBuf>,
    #[arg(long, value_enum, default_value_t = Phase::Capture)]
    pub phase: Phase,
    #[arg(long = "scenario", help = "Exact scenario ID; repeat for a subset")]
    pub scenarios: Vec<String>,
    #[arg(
        long,
        help = "Explicitly select all scenarios, including the frozen holdout"
    )]
    pub all_scenarios: bool,
    #[arg(long = "profile", help = "Exact manifest profile; repeat for a subset")]
    pub profiles: Vec<String>,
    #[arg(
        long = "variant",
        help = "Exact manifest variant ID; repeat for a subset"
    )]
    pub variants: Vec<String>,
    #[arg(long, value_parser = clap::value_parser!(u32).range(1..=1000), help = "Override capture or declared performance repetition count")]
    pub repetitions: Option<u32>,
    #[arg(
        long,
        help = "Caller-declared build commit, bound to the executable hash in the report"
    )]
    pub build_revision: String,
    #[arg(
        long,
        help = "Create-new per-scenario/profile reports here; JSON stdout contains the report index"
    )]
    pub output_dir: PathBuf,
    #[arg(long)]
    pub index: Option<PathBuf>,
    #[arg(long, value_enum, default_value_t = CliPathMode::Global)]
    pub path_mode: CliPathMode,
    #[arg(long)]
    pub json: bool,
}
