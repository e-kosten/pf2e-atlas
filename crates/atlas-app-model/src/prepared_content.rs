use serde::{Deserialize, Serialize};

#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PreparedContentFieldView {
    pub locator: crate::ContentLocationView,
    pub role: String,
    #[cfg_attr(feature = "openapi", schema(required))]
    pub source_fingerprint: Option<String>,
    pub body: PreparedFieldBodyView,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[cfg_attr(feature = "openapi", schema(nullable = false))]
    pub section_rendering: Option<PreparedSectionRenderingView>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[cfg_attr(feature = "openapi", schema(nullable = false))]
    pub routine_header: Option<RoutineHeaderView>,
}
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RoutineHeaderView {
    pub actions: String,
    pub traits: Vec<crate::RecordBadgeView>,
}

#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CheckDetailsView {
    pub traits: Vec<crate::RecordBadgeView>,
}

/// Optional, transient rendering copy. Canonical section ordinals annotate the
/// original block topology; persisted prepared HTML and its hashes stay separate.
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PreparedSectionRenderingView {
    pub html: String,
    pub selection_version: String,
}

#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
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
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ContentControlView {
    pub ordinal: usize,
    pub control: ContentControlKindView,
}
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum ContentControlKindView {
    Check {
        #[cfg_attr(feature = "openapi", schema(required))]
        statistic: Option<String>,
        options: std::collections::BTreeMap<String, String>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        #[cfg_attr(feature = "openapi", schema(nullable = false))]
        details: Option<CheckDetailsView>,
    },
    Damage {
        formula: String,
        options: std::collections::BTreeMap<String, String>,
        #[cfg_attr(feature = "openapi", schema(required))]
        literal: Option<ContentRollPresentationView>,
        #[cfg_attr(feature = "openapi", schema(required))]
        roll: Option<ContentRollOffer>,
    },
    Command {
        command: String,
        arguments: String,
        options: std::collections::BTreeMap<String, String>,
        #[cfg_attr(feature = "openapi", schema(required))]
        literal: Option<ContentRollPresentationView>,
        #[cfg_attr(feature = "openapi", schema(required))]
        roll: Option<ContentRollOffer>,
    },
    Template {
        #[cfg_attr(feature = "openapi", schema(required))]
        shape: Option<String>,
        options: std::collections::BTreeMap<String, String>,
    },
}

#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ContentRollPresentationView {
    pub heading: String,
    pub components: Vec<ContentRollComponentView>,
}

#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RollUnitView {
    pub singular: String,
    pub plural: String,
}

#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ContentRollComponentView {
    pub formula: String,
    pub label: String,
    pub expression: atlas_dice::RollPreviewExpression,
    #[cfg_attr(feature = "openapi", schema(required))]
    pub unit: Option<RollUnitView>,
}

#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RollContentRequest {
    pub offer: ContentRollOffer,
}

/// Selects an authored interaction, never a caller-supplied executable formula.
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ContentRollOffer {
    pub content_interpretation_version: String,
    pub source: crate::ContentLocationView,
    pub source_fingerprint: String,
    pub ordinal: usize,
    pub authored_formula: String,
    #[cfg_attr(feature = "openapi", schema(required))]
    pub authored_command: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[cfg_attr(feature = "openapi", schema(nullable = false))]
    pub spell_rank: Option<u8>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[cfg_attr(feature = "openapi", schema(nullable = false))]
    pub encounter: Option<ContentRollEncounterContext>,
}

/// Identity and revision only; numeric context is always reread on the server.
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ContentRollEncounterContext {
    pub encounter_ref: String,
    pub participant_key: String,
    pub participant_revision: i64,
    pub artifact_generation: String,
    #[cfg_attr(feature = "openapi", schema(required))]
    pub condition_id: Option<i64>,
}

#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ContentRollResultView {
    pub components: Vec<ContentRollComponentResultView>,
}

#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ContentRollComponentResultView {
    pub label: String,
    pub outcome: atlas_dice::RollOutcome,
}

/// An attributed marker destination within one already-selected prepared body.
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PreparedReferenceView {
    pub ordinal: usize,
    pub status: String,
    #[cfg_attr(feature = "openapi", schema(required))]
    pub target: Option<crate::RecordNavigationView>,
    #[cfg_attr(feature = "openapi", schema(required))]
    pub url: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[cfg_attr(feature = "openapi", schema(nullable = false))]
    pub effect_label: Option<crate::EffectReferenceLabelView>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[cfg_attr(feature = "openapi", schema(nullable = false))]
    pub contextual_target: Option<crate::RecordNavigationView>,
    /// Derived action command targets its containing root action; readers retain
    /// the label without offering redundant navigation. Target evidence remains.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[cfg_attr(feature = "openapi", schema(nullable = false))]
    pub self_action_reference: Option<bool>,
}

/// Flat binding namespaces for inserted prepared fields in the composed HTML.
/// Each occurrence keeps its exact source, including repeated sibling targets.
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PreparedInclusionView {
    pub namespace: usize,
    pub source: crate::ContentLocationView,
    pub controls: Vec<ContentControlView>,
    pub references: Vec<crate::RecordRelationshipView>,
}
