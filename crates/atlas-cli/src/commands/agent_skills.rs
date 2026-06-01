use std::io::IsTerminal;
use std::path::Path;
use std::process::ExitCode;

use serde::Serialize;

use args::{
    AgentArgs, AgentCommand, AgentSkillsCommand, AgentSkillsDoctorOptions,
    AgentSkillsInstallOptions,
};

use crate::agent_skills::install::{SkillInstallResult, install_skill};
use crate::agent_skills::package::{bundled_skill, bundled_skills};
use crate::agent_skills::plan::{InstallStatus, SkillInstallPlan, build_plan};
use crate::agent_skills::prompt::{confirm_install, prompt_scope, prompt_targets};
use crate::agent_skills::registry::{SkillScope, SkillTargetId, selected_scopes, selected_targets};
use crate::output::{write_json_data, write_json_error};

pub(crate) mod args;

impl AgentArgs {
    pub(crate) fn uses_json(&self) -> bool {
        match &self.command {
            AgentCommand::Skills(skills) => match &skills.command {
                AgentSkillsCommand::Install(options) => options.json,
                AgentSkillsCommand::Doctor(options) => options.json,
            },
        }
    }
}

#[derive(Debug, Serialize)]
struct AgentSkillsDoctorData {
    workspace_root: String,
    skills: Vec<crate::agent_skills::package::SkillPackageInfo>,
    targets: Vec<SkillInstallPlan>,
    shadowing: Vec<AgentSkillShadowing>,
}

#[derive(Debug, Serialize)]
struct AgentSkillShadowing {
    skill: String,
    target: &'static str,
    target_label: &'static str,
    workspace_skill_path: String,
    global_skill_path: String,
    message: String,
}

#[derive(Debug, Serialize)]
struct AgentSkillsInstallData {
    workspace_root: String,
    plans: Vec<SkillInstallPlan>,
    results: Vec<SkillInstallResult>,
}

pub(crate) fn run_agent(agent: AgentArgs) -> Result<ExitCode, String> {
    match agent.command {
        AgentCommand::Skills(skills) => match skills.command {
            AgentSkillsCommand::Install(options) => run_agent_skills_install(options),
            AgentSkillsCommand::Doctor(options) => run_agent_skills_doctor(options),
        },
    }
}

pub(crate) fn run_agent_skills_doctor(
    options: AgentSkillsDoctorOptions,
) -> Result<ExitCode, String> {
    let workspace = std::env::current_dir()
        .map_err(|error| format!("failed to determine current working directory: {error}"))?;
    let packages = match options.skill {
        Some(skill) => match resolve_package(&skill) {
            Ok(package) => vec![package],
            Err(error) if options.json => {
                write_json_error("unknown_skill", error)?;
                return Ok(ExitCode::from(2));
            }
            Err(error) => return Err(error),
        },
        None => bundled_skills().to_vec(),
    };
    let target = options.target.map(SkillTargetId::from);
    let scope = options.scope.map(SkillScope::from);
    let targets = selected_targets(target);
    let scopes = selected_scopes(scope);
    let mut plans = Vec::new();
    let mut skills = Vec::new();
    for package in &packages {
        skills.push(package.info()?);
        for target in &targets {
            for scope in &scopes {
                plans.push(build_plan(package, *target, *scope, workspace.clone())?);
            }
        }
    }
    let data = AgentSkillsDoctorData {
        workspace_root: workspace.display().to_string(),
        skills,
        shadowing: shadowing_diagnostics(&plans),
        targets: plans,
    };
    if options.json {
        write_json_data(data)?;
    } else {
        print_doctor(&data);
    }
    Ok(ExitCode::SUCCESS)
}

pub(crate) fn run_agent_skills_install(
    options: AgentSkillsInstallOptions,
) -> Result<ExitCode, String> {
    let workspace = std::env::current_dir()
        .map_err(|error| format!("failed to determine current working directory: {error}"))?;
    let package = match resolve_package(&options.skill) {
        Ok(package) => package,
        Err(error) if options.json => {
            write_json_error("unknown_skill", error)?;
            return Ok(ExitCode::from(2));
        }
        Err(error) => return Err(error),
    };
    let interactive = options.target.is_none()
        && options.scope.is_none()
        && !options.yes
        && std::io::stdin().is_terminal()
        && std::io::stdout().is_terminal();

    let (targets, scopes, confirmed_force) = if interactive {
        let selected_targets = prompt_targets(&selected_targets(None))?;
        if selected_targets.is_empty() {
            return cancelled(options.json);
        }
        let scope = prompt_scope()?;
        (selected_targets, vec![scope], true)
    } else {
        if options.target.is_none() || options.scope.is_none() || !options.yes {
            let message =
                "non-interactive install requires --target, --scope, and --yes".to_string();
            if options.json {
                write_json_error("invalid_input", message)?;
                return Ok(ExitCode::from(2));
            }
            return Err(message);
        }
        (
            selected_targets(options.target.map(SkillTargetId::from)),
            selected_scopes(options.scope.map(SkillScope::from)),
            options.force,
        )
    };

    let mut plans = Vec::new();
    for target in &targets {
        for scope in &scopes {
            plans.push(build_plan(&package, *target, *scope, workspace.clone())?);
        }
    }
    if interactive && !confirm_install(&plans)? {
        return cancelled(options.json);
    }

    let mut results = Vec::new();
    for plan in &plans {
        match install_skill(&package, plan, confirmed_force || options.force) {
            Ok(result) => results.push(result),
            Err(error) if options.json => {
                write_json_error("install_failed", error)?;
                return Ok(ExitCode::from(3));
            }
            Err(error) => return Err(error),
        }
    }

    let data = AgentSkillsInstallData {
        workspace_root: workspace.display().to_string(),
        plans,
        results,
    };
    if options.json {
        write_json_data(data)?;
    } else {
        print_install(&data);
    }
    Ok(ExitCode::SUCCESS)
}

fn resolve_package(
    skill: &str,
) -> Result<crate::agent_skills::package::BundledSkillPackage, String> {
    match bundled_skill(skill) {
        Some(package) => Ok(package),
        None => Err(format!("unknown bundled Atlas skill `{skill}`")),
    }
}

fn cancelled(json: bool) -> Result<ExitCode, String> {
    if json {
        write_json_error("cancelled", "skill install cancelled".to_string())?;
    } else {
        eprintln!("cancelled");
    }
    Ok(ExitCode::from(1))
}

fn print_doctor(data: &AgentSkillsDoctorData) {
    println!("workspace: {}", data.workspace_root);
    for plan in &data.targets {
        println!(
            "{} {} {}: {:?}",
            plan.target_label, plan.scope, plan.skill_path, plan.status
        );
    }
    for diagnostic in &data.shadowing {
        println!("{}", diagnostic.message);
    }
}

fn print_install(data: &AgentSkillsInstallData) {
    for result in &data.results {
        println!(
            "{} {} {}: {:?}",
            result.target, result.scope, result.skill_path, result.action
        );
    }
}

fn shadowing_diagnostics(plans: &[SkillInstallPlan]) -> Vec<AgentSkillShadowing> {
    let mut diagnostics = Vec::new();
    for workspace in plans
        .iter()
        .filter(|plan| plan.scope == SkillScope::Workspace && installed(plan))
    {
        if let Some(global) = plans.iter().find(|plan| {
            plan.scope == SkillScope::Global
                && plan.target == workspace.target
                && plan.skill.name == workspace.skill.name
                && installed(plan)
        }) {
            diagnostics.push(AgentSkillShadowing {
                skill: workspace.skill.name.clone(),
                target: workspace.target,
                target_label: workspace.target_label,
                workspace_skill_path: workspace.skill_path.clone(),
                global_skill_path: global.skill_path.clone(),
                message: format!(
                    "{} has workspace and global installs of {}; workspace install shadows global install",
                    workspace.target_label, workspace.skill.name
                ),
            });
        }
    }
    diagnostics
}

fn installed(plan: &SkillInstallPlan) -> bool {
    match plan.status {
        InstallStatus::Missing | InstallStatus::NotWritable => false,
        InstallStatus::Symlinked => Path::new(&plan.skill_path).exists(),
        InstallStatus::InstalledIdentical
        | InstallStatus::InstalledDifferent
        | InstallStatus::InstalledInvalid => true,
    }
}
