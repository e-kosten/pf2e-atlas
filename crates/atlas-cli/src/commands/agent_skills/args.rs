use clap::{Args, Subcommand, ValueEnum};

use crate::agent_skills::registry::{SkillScope, SkillTargetId};

#[derive(Debug, Args)]
pub(crate) struct AgentArgs {
    #[command(subcommand)]
    pub(crate) command: AgentCommand,
}

#[derive(Debug, Subcommand)]
pub(crate) enum AgentCommand {
    #[command(about = "Install bundled Atlas skills into agent skill roots")]
    Skills(AgentSkillsArgs),
}

#[derive(Debug, Args)]
#[command(
    after_help = "Examples:\n  atlas agent skills install\n  atlas agent skills install --target codex --scope global --yes\n  atlas agent skills doctor --json"
)]
pub(crate) struct AgentSkillsArgs {
    #[command(subcommand)]
    pub(crate) command: AgentSkillsCommand,
}

#[derive(Debug, Subcommand)]
pub(crate) enum AgentSkillsCommand {
    #[command(about = "Install a bundled Atlas skill")]
    Install(AgentSkillsInstallOptions),
    #[command(about = "Inspect bundled and installed Atlas skills")]
    Doctor(AgentSkillsDoctorOptions),
}

#[derive(Debug, Clone, Args)]
#[command(
    after_help = "Examples:\n  atlas agent skills install\n  atlas agent skills install --target codex --scope global --yes\n  atlas agent skills install --target agents --scope workspace --force --yes --json"
)]
pub(crate) struct AgentSkillsInstallOptions {
    #[arg(
        long,
        default_value = "pf2e-atlas-cli",
        help = "Bundled Atlas skill to install"
    )]
    pub(crate) skill: String,
    #[arg(long, value_enum, help = "Agent skill target to install into")]
    pub(crate) target: Option<CliAgentSkillTarget>,
    #[arg(
        long,
        value_enum,
        help = "Install into workspace-local or global skill roots"
    )]
    pub(crate) scope: Option<CliAgentSkillScope>,
    #[arg(long, help = "Confirm non-interactive installs")]
    pub(crate) yes: bool,
    #[arg(long, help = "Replace a differing existing install")]
    pub(crate) force: bool,
    #[arg(long, help = "Emit the standard JSON envelope")]
    pub(crate) json: bool,
}

#[derive(Debug, Clone, Args)]
#[command(
    after_help = "Examples:\n  atlas agent skills doctor\n  atlas agent skills doctor --target codex --scope global\n  atlas agent skills doctor --json"
)]
pub(crate) struct AgentSkillsDoctorOptions {
    #[arg(long, help = "Limit inspection to one bundled Atlas skill")]
    pub(crate) skill: Option<String>,
    #[arg(long, value_enum, help = "Limit inspection to one agent skill target")]
    pub(crate) target: Option<CliAgentSkillTarget>,
    #[arg(
        long,
        value_enum,
        help = "Limit inspection to workspace-local or global skill roots"
    )]
    pub(crate) scope: Option<CliAgentSkillScope>,
    #[arg(long, help = "Emit the standard JSON envelope")]
    pub(crate) json: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
pub(crate) enum CliAgentSkillScope {
    Workspace,
    Global,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
pub(crate) enum CliAgentSkillTarget {
    Agents,
    Claude,
    Codex,
    Copilot,
    Gemini,
    Kiro,
}

impl From<CliAgentSkillScope> for SkillScope {
    fn from(scope: CliAgentSkillScope) -> Self {
        match scope {
            CliAgentSkillScope::Workspace => Self::Workspace,
            CliAgentSkillScope::Global => Self::Global,
        }
    }
}

impl From<CliAgentSkillTarget> for SkillTargetId {
    fn from(target: CliAgentSkillTarget) -> Self {
        match target {
            CliAgentSkillTarget::Agents => Self::Agents,
            CliAgentSkillTarget::Claude => Self::Claude,
            CliAgentSkillTarget::Codex => Self::Codex,
            CliAgentSkillTarget::Copilot => Self::Copilot,
            CliAgentSkillTarget::Gemini => Self::Gemini,
            CliAgentSkillTarget::Kiro => Self::Kiro,
        }
    }
}
