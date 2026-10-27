//! Neutral vocabulary for the versioned, record-owned Core content policy.
use crate::SimilarityPartSelector;
use serde::{Deserialize, Serialize};
use ts_rs::TS;

/// Membership changes independently of lexical terms and native embedding inputs.
pub const CORE_SECTIONS_RECIPE: &str = "core-sections/v1";

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
pub enum SearchContentScope {
    #[default]
    All,
    Core,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
pub enum SimilaritySourceSections {
    #[default]
    All,
    Core,
    Custom,
}

/// A stable policy family for discovery and remembered source presets. The record
/// owner classifies checked source; adapters must not infer this from labels.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
pub enum CoreSectionFamily {
    Npc,
    Hazard,
    Spell,
    Rule,
    Equipment,
    CharacterOption,
    Deity,
    CampaignFeature,
    Vehicle,
    Army,
    Character,
    Familiar,
    /// Trait inventories retain their existing definition/label representation.
    Trait,
    Unmodelled,
}

impl CoreSectionFamily {
    pub const fn is_modelled(self) -> bool {
        !matches!(self, Self::Unmodelled)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
pub enum CoreFallbackReason {
    UnmodelledKind,
    NoCoreProse,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
pub enum CoreSectionRepresentation {
    IdentityAndProse,
    IdentityOnly,
    TraitDefinition,
    TraitLabelOnly,
}

/// Resolved selectors acknowledge one Core recipe within the checked inventory.
/// Required vector readiness remains independent of prose availability.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(deny_unknown_fields)]
pub struct CoreSectionSelection {
    pub family: CoreSectionFamily,
    pub selectors: Vec<SimilarityPartSelector>,
    pub representation: CoreSectionRepresentation,
    pub fallback_reason: Option<CoreFallbackReason>,
    pub recipe: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn omitted_content_preferences_preserve_all_defaults() {
        assert_eq!(SearchContentScope::default(), SearchContentScope::All);
        assert_eq!(
            SimilaritySourceSections::default(),
            SimilaritySourceSections::All
        );
    }

    #[test]
    fn policy_vocabulary_has_stable_wire_names() {
        for (value, expected) in [
            (CoreSectionFamily::Npc, "\"npc\""),
            (CoreSectionFamily::Character, "\"character\""),
            (CoreSectionFamily::CharacterOption, "\"character_option\""),
            (CoreSectionFamily::Familiar, "\"familiar\""),
            (CoreSectionFamily::Trait, "\"trait\""),
        ] {
            assert_eq!(serde_json::to_string(&value).unwrap(), expected);
        }
        assert_eq!(
            serde_json::to_string(&CoreFallbackReason::NoCoreProse).unwrap(),
            "\"no_core_prose\""
        );
        assert!(serde_json::from_str::<SearchContentScope>("\"description\"").is_err());
    }
}
