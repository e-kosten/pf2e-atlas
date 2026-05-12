use serde::{Deserialize, Serialize};

use crate::{PackName, RecordFamily, RecordKey};

pub type Level = i16;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Rarity {
    Common,
    Uncommon,
    Rare,
    Unique,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum ActionCost {
    Actions { count: u8 },
    Free,
    Reaction,
    Passive,
    Variable,
    Other { value: String },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PublicationFamily {
    Core,
    Rules,
    Adventure,
    Unknown,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TextStatus {
    Resolved,
    Missing,
    LocalizedPlaceholder,
    UnsupportedMarkup,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Publication {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    pub publication_family: PublicationFamily,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub remaster: Option<bool>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SourceProvenance {
    pub pack: PackName,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source_path: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RecordSummary {
    pub key: RecordKey,
    pub name: String,
    pub record_family: RecordFamily,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub level: Option<Level>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rarity: Option<Rarity>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub action_cost: Option<ActionCost>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub traits: Vec<String>,
    pub publication: Publication,
    pub source: SourceProvenance,
    pub text_status: TextStatus,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub summary_text: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn record_summary_round_trips_with_lightweight_record_facts() {
        let summary = RecordSummary {
            key: RecordKey::parse("rules:abc123").expect("record key should parse"),
            name: "Treat Wounds".to_string(),
            record_family: RecordFamily::Rule,
            level: Some(1),
            rarity: Some(Rarity::Common),
            action_cost: Some(ActionCost::Actions { count: 1 }),
            traits: vec!["exploration".to_string(), "healing".to_string()],
            publication: Publication {
                title: Some("Player Core".to_string()),
                publication_family: PublicationFamily::Core,
                remaster: Some(true),
            },
            source: SourceProvenance {
                pack: PackName::new("rules").expect("pack name should parse"),
                source_path: Some("packs/actions/treat-wounds.json".to_string()),
            },
            text_status: TextStatus::Resolved,
            summary_text: Some("Use Medicine to help a wounded creature recover.".to_string()),
        };

        let json = serde_json::to_string(&summary).expect("summary should serialize");
        assert!(json.contains("\"key\":\"rules:abc123\""));
        assert!(json.contains("\"record_family\":\"rule\""));
        assert!(json.contains("\"action_cost\":{\"kind\":\"actions\",\"count\":1}"));
        assert!(json.contains("\"publication_family\":\"core\""));
        assert!(json.contains("\"text_status\":\"resolved\""));

        let decoded: RecordSummary =
            serde_json::from_str(&json).expect("summary should deserialize");
        assert_eq!(decoded, summary);
    }

    #[test]
    fn action_cost_preserves_non_numeric_source_values() {
        let cost = ActionCost::Other {
            value: "1 to 3 actions".to_string(),
        };
        let json = serde_json::to_string(&cost).expect("action cost should serialize");
        assert_eq!(json, "{\"kind\":\"other\",\"value\":\"1 to 3 actions\"}");

        let decoded: ActionCost = serde_json::from_str(&json).expect("action cost should parse");
        assert_eq!(decoded, cost);
    }

    #[test]
    fn rarity_is_closed() {
        assert!(serde_json::from_str::<Rarity>("\"common\"").is_ok());
        assert!(serde_json::from_str::<Rarity>("\"mythic\"").is_err());
    }
}
