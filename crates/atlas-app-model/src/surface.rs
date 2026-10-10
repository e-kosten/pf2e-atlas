use serde::{Deserialize, Serialize};
use ts_rs::TS;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
#[ts(rename_all = "snake_case")]
pub enum RecordSurfaceProfileView {
    SearchCompact,
    RecordDetail,
    EncounterParticipant,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
pub struct RecordSurfaceView {
    pub record_key: String,
    pub title: String,
    pub kind: String,
    pub profile: RecordSurfaceProfileView,
    pub header: RecordSurfaceHeaderView,
    pub sections: Vec<RecordSurfaceSectionView>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
pub struct RecordSurfaceHeaderView {
    #[serde(skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub level_label: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub kind_label: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub rarity: Option<String>,
    pub traits: Vec<SurfaceBadgeView>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub publication: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub pack: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
pub struct SurfaceBadgeView {
    pub kind: String,
    pub label: String,
    pub value: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
#[ts(rename_all = "snake_case")]
pub enum RecordSurfaceSectionKindView {
    Identity,
    Description,
    Vitals,
    Runtime,
    Defenses,
    Saves,
    Abilities,
    Skills,
    Movement,
    Activities,
    Conditions,
    Notes,
    RichContent,
    References,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
pub struct RecordSurfaceSectionView {
    pub kind: RecordSurfaceSectionKindView,
    pub title: String,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub values: Vec<SurfaceValueView>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub groups: Vec<SurfaceValueGroupView>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub activities: Vec<SurfaceActivityView>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub notes: Vec<SurfaceNoteView>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub content: Option<PreparedContentFieldView>,
    pub collapsed_by_default: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
pub struct SurfaceValueGroupView {
    pub key: String,
    pub label: String,
    pub values: Vec<SurfaceValueView>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
pub struct SurfaceValueView {
    pub key: String,
    pub label: String,
    pub value: SurfaceScalarView,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub base_value: Option<SurfaceScalarView>,
    pub adjusted: bool,
    pub display: SurfaceValueDisplayView,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub adjustments: Vec<SurfaceAdjustmentView>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub suppressed_adjustments: Vec<SurfaceAdjustmentView>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub notes: Vec<SurfaceNoteView>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[serde(tag = "kind", content = "value", rename_all = "snake_case")]
#[ts(rename_all = "snake_case")]
pub enum SurfaceScalarView {
    Number(#[ts(type = "number")] serde_json::Number),
    Text(String),
    Formula(String),
    DistanceFeet(#[ts(type = "number")] serde_json::Number),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
#[ts(rename_all = "snake_case")]
pub enum SurfaceValueDisplayView {
    StaticNumber,
    SignedModifier,
    Distance,
    Formula,
    Text,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
pub struct SurfaceAdjustmentView {
    pub label: String,
    pub source: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub delta: Option<SurfaceScalarView>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub reason: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
pub struct SurfaceNoteView {
    pub label: String,
    pub text: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub source: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
pub struct SurfaceActivityView {
    pub navigation: crate::RecordNavigationView,
    pub key: String,
    pub label: String,
    pub kind: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub usage: Option<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub values: Vec<SurfaceValueView>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub groups: Vec<SurfaceValueGroupView>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub notes: Vec<SurfaceNoteView>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
pub struct PreparedContentFieldView {
    pub locator: atlas_record::source_content::SourceContentLocator,
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
    },
    Plain {
        text: String,
    },
    Unavailable {
        state: atlas_domain::QueryFieldState,
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
