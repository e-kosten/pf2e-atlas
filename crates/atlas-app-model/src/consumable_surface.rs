use serde::{Deserialize, Serialize};
use ts_rs::TS;

use crate::CreatureSurfaceContentView;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
pub struct ConsumableSurfaceView {
    pub slug: ConsumableFactView<String>,
    pub level: ConsumableFactView<i32>,
    pub category: ConsumableFactView<String>,
    pub rarity: ConsumableFactView<String>,
    pub traits: ConsumableFactView<Vec<String>>,
    pub other_tags: ConsumableFactView<Vec<String>>,
    pub usage: ConsumableFactView<String>,
    pub base_item: ConsumableFactView<String>,
    pub bulk: ConsumableFactView<String>,
    pub size: ConsumableFactView<String>,
    pub stack_group: ConsumableFactView<String>,
    pub material: ConsumableFactView<ConsumableMaterialView>,
    pub price: ConsumableFactView<ConsumablePriceView>,
    pub maximum_uses: ConsumableFactView<i32>,
    pub auto_destroy: ConsumableFactView<bool>,
    pub maximum_hp: ConsumableFactView<i32>,
    pub hardness: ConsumableFactView<i32>,
    pub publication: ConsumableFactView<ConsumablePublicationView>,
    pub source_state: ConsumableSourceStateView,
    pub damage: ConsumableFactView<ConsumableDamageView>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub spell_child: Option<ConsumableSpellChildLinkView>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub content: Vec<CreatureSurfaceContentView>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(tag = "state", content = "value", rename_all = "snake_case")]
#[ts(tag = "state", content = "value", rename_all = "snake_case")]
pub enum ConsumableFactView<T> {
    Missing,
    Null,
    Known(T),
    Unsupported,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
pub struct ConsumablePriceView {
    pub denominations: ConsumableFactView<Vec<ConsumablePriceDenominationView>>,
    pub per: ConsumableFactView<i32>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
pub struct ConsumableMaterialView {
    pub grade: ConsumableFactView<String>,
    pub material_type: ConsumableFactView<String>,
    pub effects: ConsumableFactView<Vec<String>>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
pub struct ConsumablePublicationView {
    pub title: ConsumableFactView<String>,
    pub license: ConsumableFactView<String>,
    pub remaster: ConsumableFactView<bool>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
pub struct ConsumablePriceDenominationView {
    pub denomination: String,
    pub amount: i32,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
pub struct ConsumableDamageView {
    pub formula: ConsumableFactView<String>,
    pub category: ConsumableFactView<String>,
    pub damage_type: ConsumableFactView<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
pub struct ConsumableSourceStateView {
    pub quantity: ConsumableFactView<i32>,
    pub current_uses: ConsumableFactView<i32>,
    pub current_hp: ConsumableFactView<i32>,
    pub container_id: ConsumableFactView<String>,
    pub equipped: ConsumableFactView<ConsumableEquippedView>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
pub struct ConsumableEquippedView {
    pub carry_type: ConsumableFactView<String>,
    pub hands_held: ConsumableFactView<i32>,
    pub in_slot: ConsumableFactView<bool>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
pub struct ConsumableSpellChildLinkView {
    pub child_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub target_record_key: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
pub struct ConsumableOccurrenceView {
    pub occurrence_id: String,
    pub authored_order: u32,
    pub name: String,
    pub identity_stability: ConsumableOccurrenceIdentityStabilityView,
    pub target: ConsumableOccurrenceTargetView,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub definition: Option<Box<ConsumableSurfaceView>>,
    pub source_state: ConsumableSourceStateView,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub spell_child: Option<ConsumableSpellChildLinkView>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub content: Vec<CreatureSurfaceContentView>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
#[ts(rename_all = "snake_case")]
pub enum ConsumableOccurrenceIdentityStabilityView {
    StableSourceIdentity,
    UnstableOwnerOrdinal,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(tag = "state", rename_all = "snake_case")]
#[ts(tag = "state", rename_all = "snake_case")]
pub enum ConsumableOccurrenceTargetView {
    Resolved {
        record_key: String,
        mismatch_fields: Vec<String>,
    },
    ParentOwned {
        reason: ConsumableTargetReasonView,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
#[ts(rename_all = "snake_case")]
pub enum ConsumableTargetReasonView {
    NoLocator,
    MalformedOrDuplicateLocator,
    TargetMissing,
    WrongDocumentOrFamily,
}
