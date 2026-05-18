use std::fmt;
use std::path::PathBuf;

use serde::Serialize;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub(crate) enum SkillScope {
    Workspace,
    Global,
}

impl SkillScope {
    pub(crate) const ALL: [Self; 2] = [Self::Workspace, Self::Global];

    pub(crate) const fn as_str(self) -> &'static str {
        match self {
            Self::Workspace => "workspace",
            Self::Global => "global",
        }
    }
}

impl fmt::Display for SkillScope {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.as_str())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub(crate) enum SkillTargetId {
    Agents,
    Claude,
    Codex,
    Copilot,
    Gemini,
    Kiro,
}

impl SkillTargetId {
    pub(crate) const fn as_str(self) -> &'static str {
        match self {
            Self::Agents => "agents",
            Self::Claude => "claude",
            Self::Codex => "codex",
            Self::Copilot => "copilot",
            Self::Gemini => "gemini",
            Self::Kiro => "kiro",
        }
    }
}

impl fmt::Display for SkillTargetId {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.as_str())
    }
}

#[derive(Debug, Clone, Copy)]
pub(crate) struct SkillTarget {
    pub(crate) id: SkillTargetId,
    pub(crate) display_name: &'static str,
    workspace_root: &'static str,
    global_root: &'static str,
}

impl SkillTarget {
    pub(crate) fn root_for_scope(
        self,
        scope: SkillScope,
        workspace_dir: PathBuf,
    ) -> Result<PathBuf, String> {
        let base = self.base_for_scope(scope, workspace_dir)?;
        match scope {
            SkillScope::Workspace => Ok(base.join(self.workspace_root)),
            SkillScope::Global => Ok(base.join(self.global_root)),
        }
    }

    pub(crate) fn base_for_scope(
        self,
        scope: SkillScope,
        workspace_dir: PathBuf,
    ) -> Result<PathBuf, String> {
        match scope {
            SkillScope::Workspace => Ok(workspace_dir),
            SkillScope::Global => home_dir().ok_or_else(|| {
                "could not determine home directory for global skill install".to_string()
            }),
        }
    }
}

impl fmt::Display for SkillTarget {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.display_name)
    }
}

pub(crate) const SKILL_TARGETS: &[SkillTarget] = &[
    SkillTarget {
        id: SkillTargetId::Agents,
        display_name: "Agent Skills",
        workspace_root: ".agents/skills",
        global_root: ".agents/skills",
    },
    SkillTarget {
        id: SkillTargetId::Claude,
        display_name: "Claude Code",
        workspace_root: ".claude/skills",
        global_root: ".claude/skills",
    },
    SkillTarget {
        id: SkillTargetId::Codex,
        display_name: "Codex",
        workspace_root: ".codex/skills",
        global_root: ".codex/skills",
    },
    SkillTarget {
        id: SkillTargetId::Copilot,
        display_name: "GitHub Copilot",
        workspace_root: ".github/skills",
        global_root: ".copilot/skills",
    },
    SkillTarget {
        id: SkillTargetId::Gemini,
        display_name: "Gemini CLI",
        workspace_root: ".gemini/skills",
        global_root: ".gemini/skills",
    },
    SkillTarget {
        id: SkillTargetId::Kiro,
        display_name: "Kiro",
        workspace_root: ".kiro/skills",
        global_root: ".kiro/skills",
    },
];

pub(crate) fn selected_targets(target: Option<SkillTargetId>) -> Vec<SkillTarget> {
    SKILL_TARGETS
        .iter()
        .copied()
        .filter(|candidate| target.is_none_or(|target| candidate.id == target))
        .collect()
}

pub(crate) fn selected_scopes(scope: Option<SkillScope>) -> Vec<SkillScope> {
    match scope {
        Some(scope) => vec![scope],
        None => SkillScope::ALL.to_vec(),
    }
}

fn home_dir() -> Option<PathBuf> {
    std::env::var_os("HOME")
        .or_else(|| std::env::var_os("USERPROFILE"))
        .map(PathBuf::from)
}
