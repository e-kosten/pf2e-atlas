//! Directed relationship knowledge and explicit GM updates.
use serde::{Deserialize, Serialize};
use ts_rs::TS;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
pub enum RelationshipPropertyView {
    Detection,
    Awareness,
    Attitude,
    Control,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
pub enum DetectionStateView {
    Observed,
    Hidden,
    Undetected,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
pub enum AttitudeStateView {
    Helpful,
    Friendly,
    Indifferent,
    Unfriendly,
    Hostile,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(tag = "kind", content = "value", rename_all = "snake_case")]
pub enum RelationshipValueView {
    Detection(DetectionStateView),
    Awareness(bool),
    Attitude(AttitudeStateView),
    NoControl,
    Control {
        controller_key: String,
        instructions: Option<String>,
    },
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum RelationshipProvenanceView {
    Table,
    Sources {
        condition_ids: Vec<i64>,
    },
    Source {
        condition_id: Option<i64>,
        record_key: Option<String>,
        application_id: Option<String>,
    },
    SourceDefault,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
pub struct RelationshipCandidateView {
    pub value: RelationshipValueView,
    pub provenance: RelationshipProvenanceView,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum RelationshipKnowledgeView {
    Known {
        value: RelationshipValueView,
        provenance: RelationshipProvenanceView,
    },
    Unknown {
        required_facts: Vec<String>,
    },
    Conflicting {
        candidates: Vec<RelationshipCandidateView>,
    },
    Invalidated {
        prior_source: RelationshipProvenanceView,
    },
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS, Default)]
#[serde(rename_all = "snake_case")]
pub enum RelationshipLayerView {
    #[default]
    Baseline,
    Adjudication,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
pub struct EncounterRelationshipView {
    pub subject_key: String,
    pub other_key: String,
    pub property: RelationshipPropertyView,
    pub knowledge: RelationshipKnowledgeView,
    pub baseline: Option<RelationshipKnowledgeView>,
    pub adjudication: Option<RelationshipKnowledgeView>,
    pub source_condition_ids: Vec<i64>,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
pub struct EncounterRelationshipsView {
    pub encounter_key: String,
    pub expected_revision: i64,
    pub relationships: Vec<EncounterRelationshipView>,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(deny_unknown_fields)]
pub struct SetEncounterRelationshipRequest {
    #[serde(default)]
    pub layer: RelationshipLayerView,
    pub request_id: String,
    pub expected_encounter_revision: i64,
    pub subject_key: String,
    pub other_key: String,
    pub property: RelationshipPropertyView,
    /// None removes the baseline; active condition causes remain intact.
    pub knowledge: Option<RelationshipKnowledgeView>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
pub enum DetectionCheckOutcomeView {
    CriticalSuccess,
    Success,
    Failure,
    CriticalFailure,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum DetectionEventView {
    BecameInvisible {
        observed_before: bool,
        can_observe: bool,
    },
    BecameAware,
    Seek {
        outcome: DetectionCheckOutcomeView,
        prior: DetectionStateView,
        can_observe: bool,
    },
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(deny_unknown_fields)]
pub struct ApplyEncounterDetectionEventRequest {
    pub request_id: String,
    pub expected_encounter_revision: i64,
    pub subject_key: String,
    pub observer_key: String,
    pub event: DetectionEventView,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS, Default)]
pub struct EncounterRequestContextView {
    pub reasonable: Option<bool>,
    pub against_goals: Option<bool>,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
pub enum EncounterRequestOutcomeView {
    CheckRequired,
    Prohibited,
    NeedsContext,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
pub struct EncounterRequestRequirementsView {
    pub outcome: EncounterRequestOutcomeView,
    pub required_facts: Vec<String>,
    pub reason: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(deny_unknown_fields)]
pub struct WorsenEncounterAttitudeRequest {
    pub request_id: String,
    pub expected_encounter_revision: i64,
    pub subject_key: String,
    pub other_key: String,
    pub previous: AttitudeStateView,
    pub attitude: AttitudeStateView,
}
