use serde::{Deserialize, Serialize};
use ts_rs::TS;

use crate::EncounterRuntimeView;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
pub struct RecordSurfaceView {
    pub metadata: RecordSurfaceMetadataView,
    pub profile: RecordSurfaceProfileView,
    pub presentation: RecordSurfacePresentationView,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub encounter: Option<EncounterRuntimeView>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
#[ts(rename_all = "snake_case")]
pub enum RecordSurfaceProfileView {
    SearchCompact,
    RecordDetail,
    EncounterParticipant,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
pub struct RecordSurfaceMetadataView {
    #[serde(skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub record_key: Option<String>,
    pub title: String,
    pub kind: String,
    pub kind_label: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub level: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub rarity: Option<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub traits: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub source: Option<RecordSurfaceSourceView>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
pub struct RecordSurfaceSourceView {
    #[serde(skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub publication_title: Option<String>,
    pub pack_label: String,
    pub document_type: String,
    pub record_type: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub source_path: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub source_contract_version: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub source_system_version: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub source_upstream_commit: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[serde(tag = "presentation_type", rename_all = "snake_case")]
#[ts(tag = "presentation_type", rename_all = "snake_case")]
pub enum RecordSurfacePresentationView {
    Creature { body: Box<CreatureSurfaceView> },
    Unavailable { unavailable: SurfaceUnavailableView },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
pub struct SurfaceUnavailableView {
    pub reason: SurfaceUnavailableReasonView,
    pub requested_kind: String,
    pub message: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
#[ts(rename_all = "snake_case")]
pub enum SurfaceUnavailableReasonView {
    RecordFamilyNotMigrated,
    RecordUnavailable,
    ManualParticipant,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
pub struct CreatureSurfaceView {
    #[serde(skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub vitals: Option<CreatureSurfaceVitalsView>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub defenses: Option<CreatureSurfaceDefensesView>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub saves: Option<CreatureSurfaceSavesView>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub awareness: Option<CreatureSurfaceAwarenessView>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub abilities: Option<CreatureSurfaceAbilitiesView>,
    #[serde(skip_serializing_if = "optional_vec_is_empty")]
    #[ts(optional)]
    pub skills: Option<Vec<CreatureSurfaceSkillView>>,
    #[serde(skip_serializing_if = "optional_vec_is_empty")]
    #[ts(optional)]
    pub movement: Option<Vec<CreatureSurfaceMovementView>>,
    #[serde(skip_serializing_if = "optional_vec_is_empty")]
    #[ts(optional)]
    pub resources: Option<Vec<CreatureSurfaceResourceView>>,
    #[serde(skip_serializing_if = "optional_vec_is_empty")]
    #[ts(optional)]
    pub spellcasting: Option<Vec<CreatureSurfaceSpellcastingView>>,
    #[serde(skip_serializing_if = "optional_vec_is_empty")]
    #[ts(optional)]
    pub activities: Option<Vec<CreatureSurfaceActivityView>>,
    #[serde(skip_serializing_if = "optional_vec_is_empty")]
    #[ts(optional)]
    pub content: Option<Vec<CreatureSurfaceContentView>>,
    #[serde(skip_serializing_if = "optional_vec_is_empty")]
    #[ts(optional)]
    pub relationships: Option<Vec<CreatureSurfaceRelationshipView>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub provenance: Option<CreatureSurfaceProvenanceView>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
pub struct CreatureSurfaceVitalsView {
    #[serde(skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub hit_points: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub details: Option<String>,
    pub provenance: CreatureSurfaceFactProvenanceView,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
pub struct CreatureSurfaceDefensesView {
    #[serde(skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub armor_class: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub armor_class_details: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub hardness: Option<i64>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub immunities: Vec<CreatureSurfaceIwrView>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub resistances: Vec<CreatureSurfaceIwrView>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub weaknesses: Vec<CreatureSurfaceIwrView>,
    pub provenance: CreatureSurfaceFactProvenanceView,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
pub struct CreatureSurfaceIwrView {
    pub component_id: String,
    pub authored_order: u32,
    pub kind: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub amount: Option<i64>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub exceptions: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub double_vs: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
pub struct CreatureSurfaceSavesView {
    #[serde(skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub fortitude: Option<CreatureSurfaceSaveView>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub reflex: Option<CreatureSurfaceSaveView>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub will: Option<CreatureSurfaceSaveView>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub all_saves_note: Option<String>,
    pub provenance: CreatureSurfaceFactProvenanceView,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
pub struct CreatureSurfaceSaveView {
    pub component_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub modifier: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub details: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
pub struct CreatureSurfaceAwarenessView {
    #[serde(skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub perception: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub details: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub has_vision: Option<bool>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub senses: Vec<CreatureSurfaceSenseView>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub languages: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub language_details: Option<String>,
    pub provenance: CreatureSurfaceFactProvenanceView,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
pub struct CreatureSurfaceSenseView {
    pub component_id: String,
    pub authored_order: u32,
    pub kind: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub acuity: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub range_feet: Option<i64>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
pub struct CreatureSurfaceAbilitiesView {
    #[serde(skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub strength: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub dexterity: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub constitution: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub intelligence: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub wisdom: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub charisma: Option<i64>,
    pub provenance: CreatureSurfaceFactProvenanceView,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
pub struct CreatureSurfaceSkillView {
    pub component_id: String,
    pub authored_order: u32,
    pub kind: String,
    pub label: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub modifier: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub note: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
pub struct CreatureSurfaceMovementView {
    pub component_id: String,
    pub authored_order: u32,
    pub mode: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub label: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub speed_feet: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub details: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
pub struct CreatureSurfaceResourceView {
    pub component_id: String,
    pub authored_order: u32,
    pub kind: String,
    pub label: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub maximum: Option<i64>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
pub struct CreatureSurfaceActivityView {
    pub occurrence_id: String,
    pub authored_order: u32,
    pub activity_type: CreatureSurfaceActivityTypeView,
    pub label: String,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub traits: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub action_cost: Option<CreatureSurfaceActionCostView>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub rolls: Vec<CreatureSurfaceRollView>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub damage: Vec<CreatureSurfaceDamageView>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
#[ts(rename_all = "snake_case")]
pub enum CreatureSurfaceActivityTypeView {
    Strike,
    Action,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(tag = "cost_type", rename_all = "snake_case")]
#[ts(tag = "cost_type", rename_all = "snake_case")]
pub enum CreatureSurfaceActionCostView {
    Passive,
    Reaction,
    FreeAction,
    Actions { count: u8 },
    Time { value: String },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
pub struct CreatureSurfaceRollView {
    pub roll_id: String,
    pub label: String,
    pub kind: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub modifier: Option<i64>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
pub struct CreatureSurfaceDamageView {
    pub damage_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub formula: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub damage_type: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub category: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
pub struct CreatureSurfaceSpellcastingView {
    pub occurrence_id: String,
    pub authored_order: u32,
    pub label: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub preparation: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub tradition: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub attack_modifier: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub difficulty_class: Option<i64>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub spells: Vec<CreatureSurfaceSpellView>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
pub struct CreatureSurfaceSpellView {
    pub occurrence_id: String,
    pub authored_order: u32,
    pub label: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub target_record_key: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub rank: Option<i64>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub traits: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
pub struct CreatureSurfaceContentView {
    pub content_key: String,
    pub owner: CreatureSurfaceContentOwnerView,
    pub role: CreatureSurfaceContentRoleView,
    pub authored_order: u32,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub label: Option<String>,
    pub text: String,
    pub content_hash: String,
    pub visibility: String,
    pub provenance: CreatureSurfaceContentProvenanceView,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(tag = "owner_type", rename_all = "snake_case")]
#[ts(tag = "owner_type", rename_all = "snake_case")]
pub enum CreatureSurfaceContentOwnerView {
    Record { record_key: String },
    Entity { entity_id: String },
    Occurrence { occurrence_id: String },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
#[ts(rename_all = "snake_case")]
pub enum CreatureSurfaceContentRoleView {
    PrimaryDescription,
    Summary,
    SupplementalRules,
    EmbeddedCapability,
    JournalPage,
    TableResult,
    GeneratedNarrative,
    Provenance,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
pub struct CreatureSurfaceContentProvenanceView {
    pub source_record_key: String,
    pub relative_source_path: String,
    pub field_family: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub nested_source_id: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
pub struct CreatureSurfaceRelationshipView {
    pub source_occurrence_id: String,
    pub kind: CreatureSurfaceRelationshipKindView,
    pub target: CreatureSurfaceRelationshipTargetView,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub contextual_label: Option<String>,
    pub provenance_only: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
#[ts(rename_all = "snake_case")]
pub enum CreatureSurfaceRelationshipKindView {
    GrantedBy,
    ItemGrant,
    LinkedWeapon,
    PreparedSpell,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(tag = "target_type", rename_all = "snake_case")]
#[ts(tag = "target_type", rename_all = "snake_case")]
pub enum CreatureSurfaceRelationshipTargetView {
    Occurrence { occurrence_id: String },
    UnresolvedSource { source_id: String },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
pub struct CreatureSurfaceProvenanceView {
    pub source_path: String,
    pub source_contract_version: String,
    pub source_system_version: String,
    pub source_upstream_commit: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
pub struct CreatureSurfaceFactProvenanceView {
    pub owner: CreatureSurfaceFactOwnerView,
    pub field: CreatureSurfaceSourceFieldView,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
#[ts(rename_all = "snake_case")]
pub enum CreatureSurfaceFactOwnerView {
    CanonicalCreature,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
#[ts(rename_all = "snake_case")]
pub enum CreatureSurfaceSourceFieldView {
    Defenses,
    Perception,
    Languages,
    Skills,
    LegacyAbilities,
}

fn optional_vec_is_empty<T>(values: &Option<Vec<T>>) -> bool {
    values.as_ref().is_none_or(Vec::is_empty)
}
