use crate::{LocalStateError, LocalStateResult};
use atlas_domain::RecordKey;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;

pub const MAX_PC_ABILITIES: usize = 1024;
pub const MAX_PC_ABILITY_LABEL: usize = 255;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PcReference {
    pub abilities: Vec<PcAbility>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PcAbility {
    pub key: String,
    pub label: String,
    pub choice: Option<String>,
    pub kind: PcAbilityKind,
    pub automatic: Option<RecordKey>,
    pub ambiguous: bool,
    pub manual: Option<RecordKey>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PcAbilityKind {
    Feat,
    Heritage,
    Special,
    Spell,
    FocusSpell,
}

impl PcAbilityKind {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Feat => "feat",
            Self::Heritage => "heritage",
            Self::Special => "special",
            Self::Spell => "spell",
            Self::FocusSpell => "focus_spell",
        }
    }
}

impl PcAbility {
    pub fn record_key(&self) -> Option<&RecordKey> {
        self.manual.as_ref().or(self.automatic.as_ref())
    }
}

impl PcReference {
    pub fn validate(&self) -> LocalStateResult<()> {
        let mut keys = HashSet::new();
        let valid_label =
            |label: &str| !label.trim().is_empty() && label.chars().count() <= MAX_PC_ABILITY_LABEL;
        if self.abilities.len() > MAX_PC_ABILITIES
            || self.abilities.iter().any(|ability| {
                ability.key.len() != 64
                    || !ability.key.bytes().all(|byte| byte.is_ascii_hexdigit())
                    || !keys.insert(ability.key.as_str())
                    || !valid_label(&ability.label)
                    || ability
                        .choice
                        .as_deref()
                        .is_some_and(|label| !valid_label(label))
                    || (ability.automatic.is_some() && ability.ambiguous)
            })
        {
            return Err(LocalStateError::InvalidParty(
                "Invalid PC ability reference.",
            ));
        }
        Ok(())
    }

    pub fn record_keys(&self) -> impl Iterator<Item = &RecordKey> {
        self.abilities.iter().flat_map(|ability| {
            [ability.automatic.as_ref(), ability.manual.as_ref()]
                .into_iter()
                .flatten()
        })
    }
}

pub(crate) fn encode(reference: &PcReference) -> LocalStateResult<String> {
    serde_json::to_string(reference)
        .map_err(|error| LocalStateError::InvalidWorkspaceRecovery(error.to_string()))
}

pub(crate) fn decode(payload: &str) -> LocalStateResult<PcReference> {
    let reference: PcReference = serde_json::from_str(payload)
        .map_err(|error| LocalStateError::InvalidWorkspaceRecovery(error.to_string()))?;
    reference.validate()?;
    Ok(reference)
}
