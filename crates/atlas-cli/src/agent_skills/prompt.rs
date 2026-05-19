use inquire::list_option::ListOption;
use inquire::validator::Validation;
use inquire::{Confirm, MultiSelect, Select};

use super::plan::SkillInstallPlan;
use super::registry::{SkillScope, SkillTarget};

pub(crate) fn prompt_targets(targets: &[SkillTarget]) -> Result<Vec<SkillTarget>, String> {
    MultiSelect::new("Install PF2e Atlas skill for:", targets.to_vec())
        .with_help_message("space toggles, enter confirms")
        .with_formatter(&|selected| {
            selected
                .iter()
                .map(|target| target.value.display_name)
                .collect::<Vec<_>>()
                .join(", ")
        })
        .with_validator(|selected: &[ListOption<&SkillTarget>]| {
            if selected.is_empty() {
                Ok(Validation::Invalid(
                    "select at least one install target".into(),
                ))
            } else {
                Ok(Validation::Valid)
            }
        })
        .prompt()
        .map_err(|error| format!("skill install cancelled: {error}"))
}

pub(crate) fn prompt_scope() -> Result<SkillScope, String> {
    Select::new(
        "Install scope:",
        vec![SkillScope::Workspace, SkillScope::Global],
    )
    .prompt()
    .map_err(|error| format!("skill install cancelled: {error}"))
}

pub(crate) fn confirm_install(plans: &[SkillInstallPlan]) -> Result<bool, String> {
    eprintln!("Atlas skill install plan:");
    for plan in plans {
        eprintln!(
            "- {} {}: {} (status: {:?}, action: {:?})",
            plan.target_label, plan.scope, plan.skill_path, plan.status, plan.action
        );
        if let Some(resolved) = &plan.resolved_skill_path
            && resolved != &plan.skill_path
        {
            eprintln!("  resolves to: {resolved}");
        }
    }
    Confirm::new("Apply this install plan?")
        .with_default(false)
        .prompt()
        .map_err(|error| format!("skill install cancelled: {error}"))
}
