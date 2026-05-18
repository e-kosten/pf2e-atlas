use std::fs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use serde::Serialize;

use super::package::{ATLAS_SKILL_MANIFEST, BundledSkillPackage};
use super::plan::{InstallAction, SkillInstallPlan};

#[derive(Debug, Serialize)]
pub(crate) struct SkillInstallResult {
    pub(crate) target: &'static str,
    pub(crate) scope: &'static str,
    pub(crate) skill_path: String,
    pub(crate) action: InstallAction,
    pub(crate) changed: bool,
}

#[derive(Debug, Serialize)]
struct AtlasSkillManifest<'a> {
    schema_version: u8,
    package_name: &'a str,
    installer: &'static str,
    content_hash_algorithm: &'static str,
    content_hash: &'a str,
    installed_at_unix_seconds: u64,
}

pub(crate) fn install_skill(
    package: &BundledSkillPackage,
    plan: &SkillInstallPlan,
    force: bool,
) -> Result<SkillInstallResult, String> {
    if !plan.can_install() {
        if matches!(plan.action, InstallAction::Blocked) {
            return Err(format!(
                "skill `{}` cannot be installed at {}: {}",
                package.name,
                plan.skill_path,
                plan.message
                    .as_deref()
                    .unwrap_or("install destination is blocked")
            ));
        }
        return Ok(SkillInstallResult {
            target: plan.target,
            scope: plan.scope.as_str(),
            skill_path: plan.skill_path.clone(),
            action: plan.action,
            changed: false,
        });
    }
    if matches!(
        plan.action,
        InstallAction::Update | InstallAction::Reinstall
    ) && !force
    {
        return Err(format!(
            "skill `{}` already exists at {} and differs; rerun with --force to replace it",
            package.name, plan.skill_path
        ));
    }
    if plan.symlinked && !force {
        return Err(format!(
            "skill install path {} uses a symlink; rerun with --force after reviewing the resolved path",
            plan.skill_path
        ));
    }

    let root = PathBuf::from(&plan.root_path);
    let skill_path = PathBuf::from(&plan.skill_path);
    fs::create_dir_all(&root).map_err(|error| {
        format!(
            "failed to create skill target root {}: {error}",
            root.display()
        )
    })?;

    if skill_path.exists() && is_symlink(&skill_path) {
        let resolved = skill_path.canonicalize().map_err(|error| {
            format!(
                "failed to resolve symlink {}: {error}",
                skill_path.display()
            )
        })?;
        if !resolved.is_dir() {
            return Err(format!(
                "symlinked skill path {} does not resolve to a directory",
                skill_path.display()
            ));
        }
        stage_and_publish_to_resolved_path(package, &resolved)?;
    } else {
        let tmp = root.join(format!(".{}.install.tmp", package.name));
        if tmp.exists() {
            fs::remove_dir_all(&tmp).map_err(|error| {
                format!(
                    "failed to remove stale temp directory {}: {error}",
                    tmp.display()
                )
            })?;
        }
        fs::create_dir(&tmp).map_err(|error| {
            format!("failed to create temp directory {}: {error}", tmp.display())
        })?;
        write_package(package, &tmp)?;
        if skill_path.exists() {
            remove_existing_install_path(&skill_path)?;
        }
        fs::rename(&tmp, &skill_path).map_err(|error| {
            format!(
                "failed to publish skill directory {}: {error}",
                skill_path.display()
            )
        })?;
    }

    Ok(SkillInstallResult {
        target: plan.target,
        scope: plan.scope.as_str(),
        skill_path: plan.skill_path.clone(),
        action: plan.action,
        changed: true,
    })
}

fn stage_and_publish_to_resolved_path(
    package: &BundledSkillPackage,
    resolved: &Path,
) -> Result<(), String> {
    let parent = resolved.parent().ok_or_else(|| {
        format!(
            "resolved skill path {} does not have a parent directory",
            resolved.display()
        )
    })?;
    let tmp = parent.join(format!(".{}.install.tmp", package.name));
    if tmp.exists() {
        remove_existing_install_path(&tmp)?;
    }
    fs::create_dir(&tmp)
        .map_err(|error| format!("failed to create temp directory {}: {error}", tmp.display()))?;
    write_package(package, &tmp)?;
    fs::remove_dir_all(resolved).map_err(|error| {
        format!(
            "failed to remove resolved skill directory {}: {error}",
            resolved.display()
        )
    })?;
    fs::rename(&tmp, resolved).map_err(|error| {
        format!(
            "failed to publish skill directory {}: {error}",
            resolved.display()
        )
    })?;
    Ok(())
}

fn remove_existing_install_path(path: &Path) -> Result<(), String> {
    let metadata = fs::symlink_metadata(path).map_err(|error| {
        format!(
            "failed to inspect existing skill path {}: {error}",
            path.display()
        )
    })?;
    if metadata.is_dir() && !metadata.file_type().is_symlink() {
        fs::remove_dir_all(path).map_err(|error| {
            format!(
                "failed to remove existing skill directory {}: {error}",
                path.display()
            )
        })
    } else {
        fs::remove_file(path).map_err(|error| {
            format!(
                "failed to remove existing skill path {}: {error}",
                path.display()
            )
        })
    }
}

fn write_package(package: &BundledSkillPackage, destination: &Path) -> Result<(), String> {
    for file in package.files {
        let path = destination.join(file.relative_path);
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).map_err(|error| {
                format!("failed to create directory {}: {error}", parent.display())
            })?;
        }
        fs::write(&path, file.contents)
            .map_err(|error| format!("failed to write {}: {error}", path.display()))?;
    }
    let info = package.info()?;
    let manifest = AtlasSkillManifest {
        schema_version: 1,
        package_name: package.name,
        installer: "pf2e-atlas",
        content_hash_algorithm: "sha256",
        content_hash: &info.content_hash,
        installed_at_unix_seconds: SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_err(|error| format!("system clock is before UNIX epoch: {error}"))?
            .as_secs(),
    };
    let manifest_path = destination.join(ATLAS_SKILL_MANIFEST);
    let manifest_json = serde_json::to_string_pretty(&manifest)
        .map_err(|error| format!("failed to render Atlas skill manifest: {error}"))?;
    fs::write(&manifest_path, manifest_json)
        .map_err(|error| format!("failed to write {}: {error}", manifest_path.display()))?;
    Ok(())
}

fn is_symlink(path: &Path) -> bool {
    fs::symlink_metadata(path)
        .map(|metadata| metadata.file_type().is_symlink())
        .unwrap_or(false)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::agent_skills::package::bundled_skill;
    use crate::agent_skills::plan::{InstallStatus, SkillInstallPlan};
    use crate::agent_skills::registry::SkillScope;

    #[test]
    fn install_errors_for_blocked_plan() {
        let package = bundled_skill("pf2e-atlas-cli").expect("bundled skill");
        let skill = package.info().expect("skill info");
        let plan = SkillInstallPlan {
            skill,
            target: "codex",
            target_label: "Codex",
            scope: SkillScope::Workspace,
            root_path: "/tmp/.codex/skills".to_string(),
            skill_path: "/tmp/.codex/skills/pf2e-atlas-cli".to_string(),
            resolved_root_path: None,
            resolved_skill_path: None,
            status: InstallStatus::NotWritable,
            action: InstallAction::Blocked,
            writable: false,
            symlinked: false,
            installed_hash: None,
            message: Some("target root is not writable".to_string()),
        };

        let error = install_skill(&package, &plan, false).expect_err("blocked install fails");
        assert!(error.contains("cannot be installed"));
        assert!(error.contains("target root is not writable"));
    }
}
