use serde::{Deserialize, Serialize};

use crate::{RecordKey, RecordSummary, SourceCategory};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ReferenceDirection {
    Outgoing,
    Backlink,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ReferenceRelationship {
    References,
    ReferencedBy,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ReferenceSource {
    pub pack_name: String,
    pub record_type: String,
    pub document_type: String,
    pub category: SourceCategory,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ReferenceEdge {
    pub from: RecordKey,
    pub to: RecordKey,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub display_text: Option<String>,
    pub reference_text: String,
    pub direction: ReferenceDirection,
    pub relationship: ReferenceRelationship,
    pub source: ReferenceSource,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RuleGraphRequest {
    pub keys: Vec<RecordKey>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub core_only: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub include_outgoing: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub include_backlinks: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_outgoing_per_primary: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_backlinks_per_primary: Option<u32>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RuleGraphResult {
    pub records: Vec<RecordSummary>,
    pub edges: Vec<ReferenceEdge>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RuleGraphCollectionResult {
    pub outgoing: RuleGraphResult,
    pub backlinks: RuleGraphResult,
    pub edges: Vec<ReferenceEdge>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RuleContextRequest {
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub rules: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub question: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub core_only: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub include_backlinks: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_outgoing_per_primary: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_backlinks_per_primary: Option<u32>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RuleContextResult {
    pub primary: Vec<RecordSummary>,
    pub outgoing: RuleGraphResult,
    pub backlinks: RuleGraphResult,
    pub edges: Vec<ReferenceEdge>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RemasterLinkSource {
    RemasterJournal,
    Migration,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RemasterLink {
    pub remaster: RecordKey,
    pub legacy: RecordKey,
    pub source: RemasterLinkSource,
    pub source_ref: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reference_edge_round_trips_with_rust_canonical_names() {
        let edge = ReferenceEdge {
            from: RecordKey::parse("actions:ReactiveStrike").expect("record key should parse"),
            to: RecordKey::parse("rules:Attack").expect("record key should parse"),
            display_text: Some("Attack".to_string()),
            reference_text: "@UUID[Compendium.pf2e.rules.Item.Attack]{Attack}".to_string(),
            direction: ReferenceDirection::Outgoing,
            relationship: ReferenceRelationship::References,
            source: ReferenceSource {
                pack_name: "actions".to_string(),
                record_type: "action".to_string(),
                document_type: "Item".to_string(),
                category: SourceCategory::Core,
            },
        };

        let json = serde_json::to_string(&edge).expect("edge should serialize");
        assert!(json.contains("\"from\":\"actions:ReactiveStrike\""));
        assert!(json.contains("\"direction\":\"outgoing\""));
        assert!(json.contains("\"relationship\":\"references\""));
        assert!(json.contains("\"pack_name\":\"actions\""));

        let decoded: ReferenceEdge = serde_json::from_str(&json).expect("edge should parse");
        assert_eq!(decoded, edge);
    }

    #[test]
    fn rule_graph_request_round_trips_direction_controls() {
        let request = RuleGraphRequest {
            keys: vec![RecordKey::parse("rules:Grab").expect("record key should parse")],
            core_only: Some(true),
            include_outgoing: Some(true),
            include_backlinks: Some(false),
            max_outgoing_per_primary: Some(4),
            max_backlinks_per_primary: Some(2),
        };

        let json = serde_json::to_string(&request).expect("request should serialize");
        assert!(json.contains("\"include_outgoing\":true"));
        assert!(json.contains("\"max_backlinks_per_primary\":2"));

        let decoded: RuleGraphRequest = serde_json::from_str(&json).expect("request should parse");
        assert_eq!(decoded, request);
    }

    #[test]
    fn remaster_link_is_separate_from_reference_edges() {
        let link = RemasterLink {
            remaster: RecordKey::parse("conditionitems:Off-Guard")
                .expect("record key should parse"),
            legacy: RecordKey::parse("conditionitems:Flat-Footed")
                .expect("record key should parse"),
            source: RemasterLinkSource::Migration,
            source_ref: "src/module/migration/migrations".to_string(),
        };

        let json = serde_json::to_string(&link).expect("link should serialize");
        assert!(json.contains("\"remaster\":\"conditionitems:Off-Guard\""));
        assert!(json.contains("\"legacy\":\"conditionitems:Flat-Footed\""));
        assert!(json.contains("\"source\":\"migration\""));
        assert!(!json.contains("relationship"));

        let decoded: RemasterLink = serde_json::from_str(&json).expect("link should parse");
        assert_eq!(decoded, link);
    }
}
