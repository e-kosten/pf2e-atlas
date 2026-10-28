use serde::{Deserialize, Serialize};
use ts_rs::TS;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
pub struct PreparedContentFieldView {
    pub locator: crate::ContentLocationView,
    pub role: String,
    pub source_fingerprint: Option<String>,
    pub body: PreparedFieldBodyView,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub section_rendering: Option<PreparedSectionRenderingView>,
}
/// Optional, transient rendering copy. Canonical section ordinals annotate the
/// original block topology; persisted prepared HTML and its hashes stay separate.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
pub struct PreparedSectionRenderingView {
    pub html: String,
    pub selection_version: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum PreparedFieldBodyView {
    Html {
        html: String,
        controls: Vec<ContentControlView>,
        inclusions: Vec<PreparedInclusionView>,
    },
    Plain {
        text: String,
    },
    Unavailable {
        state: atlas_domain::QueryFieldState,
    },
    Bounded {
        reason: String,
    },
}
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
pub struct ContentControlView {
    pub ordinal: usize,
    pub control: ContentControlKindView,
}
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum ContentControlKindView {
    Check {
        statistic: Option<String>,
        options: std::collections::BTreeMap<String, String>,
    },
    Damage {
        formula: String,
        options: std::collections::BTreeMap<String, String>,
        literal: Option<ContentRollPresentationView>,
        roll: Option<RollContentRequest>,
    },
    Command {
        command: String,
        arguments: String,
        options: std::collections::BTreeMap<String, String>,
        literal: Option<ContentRollPresentationView>,
        roll: Option<RollContentRequest>,
    },
    Template {
        shape: Option<String>,
        options: std::collections::BTreeMap<String, String>,
    },
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
pub struct ContentRollPresentationView {
    pub heading: String,
    pub components: Vec<ContentRollComponentView>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
pub struct RollUnitView {
    pub singular: String,
    pub plural: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
pub struct ContentRollComponentView {
    pub formula: String,
    pub label: String,
    pub expression: atlas_dice::RollPreviewExpression,
    pub unit: Option<RollUnitView>,
}

/// Selects an authored interaction, never a caller-supplied executable formula.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
pub struct RollContentRequest {
    pub content_interpretation_version: String,
    pub source: crate::ContentLocationView,
    pub source_fingerprint: String,
    pub ordinal: usize,
    pub authored_formula: String,
    pub authored_command: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub spell_rank: Option<u8>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub encounter: Option<ContentRollEncounterContext>,
}

/// Identity and revision only; numeric context is always reread on the server.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
pub struct ContentRollEncounterContext {
    pub encounter_ref: String,
    pub participant_key: String,
    pub participant_revision: i64,
    pub artifact_generation: String,
    pub condition_id: Option<i64>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
pub struct ContentRollResultView {
    pub components: Vec<ContentRollComponentResultView>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
pub struct ContentRollComponentResultView {
    pub label: String,
    pub outcome: atlas_dice::RollOutcome,
}

/// An attributed marker destination within one already-selected prepared body.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
pub struct PreparedReferenceView {
    pub ordinal: usize,
    pub status: String,
    pub target: Option<crate::RecordNavigationView>,
    pub url: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub effect_label: Option<crate::EffectReferenceLabelView>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub contextual_target: Option<crate::RecordNavigationView>,
    /// Derived action command targets its containing root action; readers retain
    /// the label without offering redundant navigation. Target evidence remains.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub self_action_reference: Option<bool>,
}

/// Flat binding namespaces for inserted prepared fields in the composed HTML.
/// Each occurrence keeps its exact source, including repeated sibling targets.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
pub struct PreparedInclusionView {
    pub namespace: usize,
    pub source: crate::ContentLocationView,
    pub controls: Vec<ContentControlView>,
    pub references: Vec<crate::RecordRelationshipView>,
}
