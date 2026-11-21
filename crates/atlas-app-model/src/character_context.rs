//! Saved character context for agent and terminal workflows.
use crate::{CharacterSummaryView, PcAbilityView, SheetGroupView};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

/// Selectable portions of one saved character. Source text is opt-in.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CharacterSection {
    Identity,
    Statistics,
    Proficiencies,
    CharacterOptions,
    Abilities,
    Spellcasting,
    Equipment,
    Money,
    Crafting,
    Companions,
    Source,
}

impl CharacterSection {
    pub const INTERPRETED: [Self; 10] = [
        Self::Identity,
        Self::Statistics,
        Self::Proficiencies,
        Self::CharacterOptions,
        Self::Abilities,
        Self::Spellcasting,
        Self::Equipment,
        Self::Money,
        Self::Crafting,
        Self::Companions,
    ];
}

/// A decoder-verified scalar, ordered list or named object, never raw source JSON.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum ImportedFactValue {
    Text(String),
    Integer(i64),
    Boolean(bool),
    List(Vec<ImportedFactView>),
    Object(BTreeMap<String, ImportedFactView>),
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "state", content = "value", rename_all = "snake_case")]
pub enum ImportedFactView {
    Missing,
    Null,
    Unrecognized,
    Value(ImportedFactValue),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CharacterSourceState {
    NoImport,
    ReferenceOnly,
    Available,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AbilityLinkState {
    Automatic,
    Manual,
    Unlinked,
    Ambiguous,
    Unavailable,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CharacterAbilityView {
    #[serde(flatten)]
    pub ability: PcAbilityView,
    pub link_state: AbilityLinkState,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct CharacterAbilityFilter {
    pub needs_review: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub kind: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub query: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CharacterAbilitiesView {
    pub total: usize,
    pub shown: usize,
    pub filter: CharacterAbilityFilter,
    pub entries: Vec<CharacterAbilityView>,
}

/// Selected source fields and shared human presentation have distinct provenance.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CharacterSectionView {
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub fields: BTreeMap<String, ImportedFactView>,
    #[serde(flatten)]
    pub abilities: Option<CharacterAbilitiesView>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source_json: Option<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub record_links: Vec<crate::RecordTitleLinkView>,
    /// Existing formatted/derived sheet rows, supplied by the shared presenter.
    #[serde(skip)]
    pub presentation: Vec<SheetGroupView>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImportedCharacterIdentity {
    pub name: String,
    pub level: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CharacterContextView {
    pub character: CharacterSummaryView,
    pub source_state: CharacterSourceState,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub imported_identity: Option<ImportedCharacterIdentity>,
    pub selected_sections: Vec<CharacterSection>,
    pub sections: BTreeMap<CharacterSection, CharacterSectionView>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PartyContextView {
    pub party_key: String,
    pub name: String,
    pub revision: i64,
    pub selected_sections: Vec<CharacterSection>,
    pub members: Vec<CharacterContextView>,
}

#[derive(Debug, Clone, Default)]
pub struct CharacterContextRequest {
    pub sections: Vec<CharacterSection>,
    pub abilities: CharacterAbilityFilter,
}

#[derive(Debug, Clone, Serialize)]
pub struct CharacterImportPreviewView {
    pub source_sha256: String,
    pub artifact_generation: String,
    pub name: String,
    pub level: i64,
    pub changes: Option<crate::ImportChangesView>,
    pub selected_sections: Vec<CharacterSection>,
    pub proposed: BTreeMap<CharacterSection, CharacterSectionView>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub current: Option<CharacterContextView>,
}
