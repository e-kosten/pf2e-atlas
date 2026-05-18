use std::fs;
use std::path::{Path, PathBuf};

use serde::Serialize;

use super::package::{
    ATLAS_SKILL_MANIFEST, BundledSkillPackage, SkillPackageInfo, installed_content_hash,
};
use super::registry::{SkillScope, SkillTarget};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub(crate) enum InstallStatus {
    Missing,
    InstalledIdentical,
    InstalledDifferent,
    InstalledInvalid,
    NotWritable,
    Symlinked,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub(crate) enum InstallAction {
    Install,
    Update,
    Reinstall,
    Unchanged,
    Blocked,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct SkillInstallPlan {
    pub(crate) skill: SkillPackageInfo,
    pub(crate) target: &'static str,
    pub(crate) target_label: &'static str,
    pub(crate) scope: SkillScope,
    pub(crate) root_path: String,
    pub(crate) skill_path: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) resolved_root_path: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) resolved_skill_path: Option<String>,
    pub(crate) status: InstallStatus,
    pub(crate) action: InstallAction,
    pub(crate) writable: bool,
    pub(crate) symlinked: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) installed_hash: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) message: Option<String>,
}

impl SkillInstallPlan {
    pub(crate) fn can_install(&self) -> bool {
        matches!(
            self.action,
            InstallAction::Install | InstallAction::Update | InstallAction::Reinstall
        )
    }
}

pub(crate) fn build_plan(
    package: &BundledSkillPackage,
    target: SkillTarget,
    scope: SkillScope,
    workspace_dir: PathBuf,
) -> Result<SkillInstallPlan, String> {
    let base = target.base_for_scope(scope, workspace_dir.clone())?;
    let root = target.root_for_scope(scope, workspace_dir)?;
    let skill_path = root.join(package.name);
    let skill = package.info()?;
    let symlinked =
        path_uses_symlink_under(&base, &root) || path_uses_symlink_under(&base, &skill_path);
    let resolved_root = symlinked.then(|| resolve_path(&root)).flatten();
    let resolved_skill = symlinked.then(|| resolve_path(&skill_path)).flatten();

    let (status, installed_hash, message) = status_for_path(&skill_path, &skill.content_hash);
    let writable = install_path_writable(&root, &skill_path);
    let status = if !writable {
        InstallStatus::NotWritable
    } else if symlinked {
        InstallStatus::Symlinked
    } else {
        status
    };
    let action = action_for_status(if matches!(status, InstallStatus::Symlinked) {
        status_for_path(&skill_path, &skill.content_hash).0
    } else {
        status
    });

    Ok(SkillInstallPlan {
        skill,
        target: target.id.as_str(),
        target_label: target.display_name,
        scope,
        root_path: root.display().to_string(),
        skill_path: skill_path.display().to_string(),
        resolved_root_path: resolved_root.map(|path| path.display().to_string()),
        resolved_skill_path: resolved_skill.map(|path| path.display().to_string()),
        status,
        action,
        writable,
        symlinked,
        installed_hash,
        message,
    })
}

fn status_for_path(
    skill_path: &Path,
    expected_hash: &str,
) -> (InstallStatus, Option<String>, Option<String>) {
    if !skill_path.exists() {
        return (InstallStatus::Missing, None, None);
    }
    if !skill_path.is_dir() {
        return (
            InstallStatus::InstalledInvalid,
            None,
            Some("installed skill path exists but is not a directory".to_string()),
        );
    }
    let manifest_path = skill_path.join(ATLAS_SKILL_MANIFEST);
    let manifest = fs::read_to_string(&manifest_path);
    let Ok(manifest) = manifest else {
        return (
            InstallStatus::InstalledInvalid,
            None,
            Some("installed skill is missing Atlas manifest".to_string()),
        );
    };
    let manifest_hash = manifest_hash(&manifest);
    let installed_hash = match installed_content_hash(skill_path) {
        Ok(hash) => hash,
        Err(error) => {
            return (
                InstallStatus::InstalledInvalid,
                None,
                Some(format!("installed skill cannot be hashed: {error}")),
            );
        }
    };
    match manifest_hash {
        Some(hash) if hash == installed_hash && installed_hash == expected_hash => (
            InstallStatus::InstalledIdentical,
            Some(installed_hash),
            None,
        ),
        Some(hash) if hash != installed_hash => (
            InstallStatus::InstalledDifferent,
            Some(installed_hash),
            Some("installed skill content differs from its Atlas manifest".to_string()),
        ),
        Some(_) => (
            InstallStatus::InstalledDifferent,
            Some(installed_hash),
            None,
        ),
        None => (
            InstallStatus::InstalledInvalid,
            None,
            Some("installed Atlas manifest does not contain a content hash".to_string()),
        ),
    }
}

fn manifest_hash(manifest: &str) -> Option<String> {
    let value = serde_json::from_str::<serde_json::Value>(manifest).ok()?;
    value.get("content_hash")?.as_str().map(ToString::to_string)
}

fn install_path_writable(root: &Path, skill_path: &Path) -> bool {
    nearest_existing_ancestor(root).is_some_and(path_writable)
        && (!skill_path.exists() || path_writable(skill_path))
}

fn nearest_existing_ancestor(path: &Path) -> Option<&Path> {
    path.ancestors().find(|ancestor| ancestor.exists())
}

fn path_writable(path: &Path) -> bool {
    fs::metadata(path)
        .map(|metadata| !metadata.permissions().readonly())
        .unwrap_or(false)
}

fn action_for_status(status: InstallStatus) -> InstallAction {
    match status {
        InstallStatus::Missing => InstallAction::Install,
        InstallStatus::InstalledIdentical => InstallAction::Unchanged,
        InstallStatus::InstalledDifferent => InstallAction::Update,
        InstallStatus::InstalledInvalid => InstallAction::Reinstall,
        InstallStatus::NotWritable => InstallAction::Blocked,
        InstallStatus::Symlinked => InstallAction::Blocked,
    }
}

fn is_symlink(path: &Path) -> bool {
    fs::symlink_metadata(path)
        .map(|metadata| metadata.file_type().is_symlink())
        .unwrap_or(false)
}

fn path_uses_symlink_under(base: &Path, path: &Path) -> bool {
    path.ancestors()
        .take_while(|ancestor| *ancestor != base)
        .any(is_symlink)
}

fn resolve_path(path: &Path) -> Option<PathBuf> {
    if path.exists() {
        return path.canonicalize().ok();
    }
    let mut suffix = PathBuf::new();
    for ancestor in path.ancestors() {
        if ancestor.exists() {
            let mut resolved = ancestor.canonicalize().ok()?;
            if !suffix.as_os_str().is_empty() {
                resolved.push(suffix);
            }
            return Some(resolved);
        }
        if let Some(name) = ancestor.file_name() {
            suffix = PathBuf::from(name).join(suffix);
        }
    }
    None
}
