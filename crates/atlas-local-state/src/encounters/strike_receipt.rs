//! Version-one durable strike facts. Product labels and response layouts are derived on read.
use atlas_dice::RollOutcome;
use serde::{Deserialize, Serialize};

use crate::{LocalStateError, LocalStateResult};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct StoredStrikeReceipt {
    pub request_id: String,
    pub encounter_key: String,
    pub participant_key: String,
    pub roll: StoredStrikeRoll,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct StoredStrikeRoll {
    pub title: String,
    pub outcome: StoredStrikeOutcome,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum StoredStrikeOutcome {
    Attack {
        outcome: RollOutcome,
    },
    Damage {
        components: Vec<StoredStrikeDamage>,
        totals: Vec<StoredStrikeDamageTotal>,
        total: Option<i32>,
        persistent_total: Option<i32>,
        minimum_adjustment: i32,
        persistent_minimum_adjustment: i32,
    },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct StoredStrikeDamage {
    pub id: String,
    pub formula: String,
    pub damage_type: Option<String>,
    pub category: Option<String>,
    pub outcome: RollOutcome,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct StoredStrikeDamageTotal {
    pub damage_type: String,
    pub persistent: bool,
    pub raw_total: i32,
    pub total: i32,
}

/// Envelope v1 contains this action-specific payload, not an app response.
#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct StrikePayload<T> {
    action: StrikeAction,
    pub receipt: T,
}
#[derive(Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
enum StrikeAction {
    Strike,
}

pub(super) fn encode(receipt: &StoredStrikeReceipt) -> LocalStateResult<String> {
    super::receipt_codec::encode(&StrikePayload {
        action: StrikeAction::Strike,
        receipt,
    })
}

impl StoredStrikeReceipt {
    pub(super) fn validate(&self) -> LocalStateResult<()> {
        if self.request_id.len() > 128
            || self.encounter_key.len() > 256
            || self.participant_key.len() > 256
            || self.roll.title.len() > 4096
        {
            return Err(LocalStateError::InvalidRollRequest(
                "strike receipt exceeds supported bounds",
            ));
        }
        if let StoredStrikeOutcome::Damage {
            components, totals, ..
        } = &self.roll.outcome
            && (components.len() > 64
                || totals.len() > 64
                || components.iter().any(|p| {
                    p.id.len() > 256
                        || p.formula.len() > 4096
                        || p.damage_type.as_ref().is_some_and(|s| s.len() > 256)
                        || p.category.as_ref().is_some_and(|s| s.len() > 256)
                })
                || totals.iter().any(|t| t.damage_type.len() > 256))
        {
            return Err(LocalStateError::InvalidRollRequest(
                "strike receipt exceeds supported bounds",
            ));
        }
        if serde_json::to_vec(self)?.len() > 256 * 1024 {
            return Err(LocalStateError::InvalidRollRequest(
                "strike receipt exceeds supported bounds",
            ));
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn stored_attack_has_no_product_response_shape() -> LocalStateResult<()> {
        let receipt = StoredStrikeReceipt {
            request_id: "request".into(),
            encounter_key: "encounter".into(),
            participant_key: "participant".into(),
            roll: StoredStrikeRoll {
                title: "Claw".into(),
                outcome: StoredStrikeOutcome::Attack {
                    outcome: atlas_dice::prepare("17")?.roll()?,
                },
            },
        };
        let encoded = encode(&receipt)?;
        let value: serde_json::Value = serde_json::from_str(&encoded)?;
        assert_eq!(value["version"], 1);
        assert_eq!(value["payload"]["action"], "strike");
        assert_eq!(
            value["payload"]["receipt"]["roll"]["outcome"]["kind"],
            "attack"
        );
        assert!(value["payload"].get("attack").is_none());
        assert_eq!(
            super::super::receipt_codec::decode::<StrikePayload<StoredStrikeReceipt>>(&encoded)?
                .receipt,
            receipt
        );
        Ok(())
    }
    #[test]
    fn damage_facts_round_trip_without_display_labels() -> LocalStateResult<()> {
        let receipt = StoredStrikeReceipt {
            request_id: "request".into(),
            encounter_key: "encounter".into(),
            participant_key: "participant".into(),
            roll: StoredStrikeRoll {
                title: "Flaming claw".into(),
                outcome: StoredStrikeOutcome::Damage {
                    components: vec![StoredStrikeDamage {
                        id: "fire".into(),
                        formula: "-2".into(),
                        damage_type: Some("fire".into()),
                        category: Some("persistent".into()),
                        outcome: atlas_dice::prepare("-2")?.roll()?,
                    }],
                    totals: vec![StoredStrikeDamageTotal {
                        damage_type: "fire".into(),
                        persistent: true,
                        raw_total: -2,
                        total: 0,
                    }],
                    total: None,
                    persistent_total: Some(1),
                    minimum_adjustment: 0,
                    persistent_minimum_adjustment: 1,
                },
            },
        };
        receipt.validate()?;
        let encoded = encode(&receipt)?;
        assert_eq!(
            super::super::receipt_codec::decode::<StrikePayload<StoredStrikeReceipt>>(&encoded)?
                .receipt,
            receipt
        );
        let mut wrong_action: serde_json::Value = serde_json::from_str(&encoded)?;
        wrong_action["payload"]["action"] = "initiative".into();
        assert!(
            super::super::receipt_codec::decode::<StrikePayload<StoredStrikeReceipt>>(
                &wrong_action.to_string()
            )
            .is_err()
        );
        Ok(())
    }
}
