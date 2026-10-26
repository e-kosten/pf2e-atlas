use serde::{Deserialize, Serialize};
use ts_rs::TS;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
pub struct PreparedContentFieldView {
    pub locator: crate::ContentLocationView,
    pub role: String,
    pub source_fingerprint: Option<String>,
    pub body: PreparedFieldBodyView,
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
        literal: Option<Vec<ContentDamageComponentView>>,
        roll: Option<RollContentDamageRequest>,
    },
    Command {
        command: String,
        arguments: String,
        options: std::collections::BTreeMap<String, String>,
    },
    Template {
        shape: Option<String>,
        options: std::collections::BTreeMap<String, String>,
    },
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
pub struct ContentDamageComponentView {
    pub formula: String,
    pub label: String,
    pub expression: atlas_dice::RollPreviewExpression,
}

/// Selects an authored interaction, never a caller-supplied executable formula.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
pub struct RollContentDamageRequest {
    pub content_interpretation_version: String,
    pub source: crate::ContentLocationView,
    pub source_fingerprint: String,
    pub ordinal: usize,
    pub authored_formula: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
pub struct ContentDamageRollView {
    pub components: Vec<ContentDamageRollComponentView>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
pub struct ContentDamageRollComponentView {
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
