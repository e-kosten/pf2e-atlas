use serde::{Deserialize, Serialize};
use ts_rs::TS;

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
