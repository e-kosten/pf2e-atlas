use atlas_domain::{Rarity, RecordKey};
use serde::{Deserialize, Serialize};

use crate::{FactValue, OwnedRichContent, PublicationLicense, RichDocument};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HazardRecord {
    pub identity: HazardIdentity,
    pub level: HazardFact<i64>,
    pub rarity: HazardFact<Rarity>,
    pub traits: HazardFact<Vec<HazardTrait>>,
    pub size: HazardFact<HazardSize>,
    pub publication: HazardFact<HazardPublication>,
    pub complexity: HazardFact<HazardComplexity>,
    pub detection: HazardFact<HazardDetection>,
    pub defenses: HazardFact<HazardDefenses>,
    pub lifecycle: HazardFact<HazardLifecycle>,
    pub emits_sound: HazardFact<HazardEmitsSound>,
    pub embedded_entities: HazardFact<HazardEmbeddedEntities>,
    pub content: OwnedRichContent,
    pub relationships: Vec<HazardRelationship>,
    pub unsupported_fields: Vec<HazardUnsupportedFact>,
    pub provenance: HazardProvenance,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct HazardIdentity {
    pub record_key: RecordKey,
    pub source_id: HazardSourceId,
    pub name: String,
}

macro_rules! validated_id {
    ($name:ident, $error:ident) => {
        #[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize)]
        #[serde(transparent)]
        pub struct $name(String);

        impl $name {
            pub fn new(value: impl Into<String>) -> Result<Self, $error> {
                let value = value.into();
                if value.trim().is_empty() || value.chars().any(char::is_whitespace) {
                    return Err($error);
                }
                Ok(Self(value))
            }

            pub fn as_str(&self) -> &str {
                &self.0
            }
        }

        #[derive(Debug, Clone, PartialEq, Eq)]
        pub struct $error;

        impl<'de> Deserialize<'de> for $name {
            fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                let value = String::deserialize(deserializer)?;
                Self::new(value).map_err(|_| serde::de::Error::custom("invalid hazard identifier"))
            }
        }
    };
}

validated_id!(HazardSourceId, InvalidHazardSourceId);
validated_id!(HazardEntityId, InvalidHazardEntityId);
validated_id!(HazardOccurrenceId, InvalidHazardOccurrenceId);
validated_id!(HazardRelationshipId, InvalidHazardRelationshipId);

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct HazardProvenance {
    pub source_path: String,
    pub source_contract_version: String,
    pub source_system_version: String,
    pub source_upstream_commit: String,
    pub source_folder: HazardFact<String>,
    pub image: HazardFact<String>,
    pub source_creature_type: HazardFact<String>,
    pub source_status_effects: HazardFact<Vec<String>>,
    pub actor_effects: HazardFact<Vec<HazardProvenanceValue>>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct HazardProvenanceValue {
    pub authored_order: u32,
    pub exact_json: String,
    pub source_shape: HazardSourceShape,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct HazardFact<T> {
    pub value: FactValue<HazardSourceValue<T>>,
    pub provenance: HazardFactProvenance,
}

impl<T> HazardFact<T> {
    pub fn source(
        value: FactValue<HazardSourceValue<T>>,
        relative_source_path: impl Into<String>,
    ) -> Self {
        Self {
            value,
            provenance: HazardFactProvenance {
                relative_source_path: relative_source_path.into(),
            },
        }
    }

    pub fn typed(&self) -> Option<&T> {
        self.value.as_value().and_then(HazardSourceValue::typed)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct HazardFactProvenance {
    pub relative_source_path: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "support", content = "value", rename_all = "snake_case")]
pub enum HazardSourceValue<T> {
    Typed(T),
    Unsupported(HazardUnsupportedValue),
}

impl<T> HazardSourceValue<T> {
    pub fn typed(&self) -> Option<&T> {
        match self {
            Self::Typed(value) => Some(value),
            Self::Unsupported(_) => None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct HazardUnsupportedValue {
    pub exact_json: String,
    pub expected_shape: HazardExpectedShape,
    pub actual_shape: HazardSourceShape,
    pub relative_source_path: String,
    pub owner: HazardUnsupportedOwner,
    pub diagnostic_code: HazardDiagnosticCode,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum HazardExpectedShape {
    Any,
    Boolean,
    Integer,
    String,
    StringOrBoolean,
    StringOrArray,
    Array,
    Object,
    ClosedVocabulary,
    RichDocument,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum HazardSourceShape {
    Missing,
    Null,
    Boolean,
    Number,
    String,
    Array,
    Object,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", content = "value", rename_all = "snake_case")]
pub enum HazardUnsupportedOwner {
    Record(RecordKey),
    Entity(HazardEntityId),
    Occurrence(HazardOccurrenceId),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum HazardDiagnosticCode {
    UnexpectedShape,
    UnsupportedValue,
    LegacyField,
    UnsupportedRuleElement,
    InvalidCanonicalValue,
    UnstableIdentity,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(transparent)]
pub struct HazardTrait {
    slug: String,
}

impl HazardTrait {
    pub fn new(value: impl Into<String>) -> Result<Self, InvalidHazardSlug> {
        let slug = value.into();
        if slug.trim().is_empty()
            || slug.chars().any(char::is_whitespace)
            || !slug.chars().all(|character| {
                character.is_ascii_alphanumeric() || matches!(character, '-' | '_')
            })
        {
            return Err(InvalidHazardSlug);
        }
        Ok(Self { slug })
    }

    pub fn as_str(&self) -> &str {
        &self.slug
    }
}

impl<'de> Deserialize<'de> for HazardTrait {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let value = String::deserialize(deserializer)?;
        Self::new(value).map_err(|_| serde::de::Error::custom("invalid hazard trait slug"))
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InvalidHazardSlug;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum HazardSize {
    Tiny,
    Small,
    Medium,
    Large,
    Huge,
    Gargantuan,
}

impl HazardSize {
    pub const fn as_source(self) -> &'static str {
        match self {
            Self::Tiny => "tiny",
            Self::Small => "sm",
            Self::Medium => "med",
            Self::Large => "lg",
            Self::Huge => "huge",
            Self::Gargantuan => "grg",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct HazardPublication {
    pub title: HazardFact<String>,
    pub remaster: HazardFact<bool>,
    pub license: HazardFact<PublicationLicense>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum HazardComplexity {
    Simple,
    Complex,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct HazardDetection {
    pub stealth_modifier: HazardFact<i64>,
    pub details: HazardFact<RichDocument>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct HazardDefenses {
    pub armor_class: HazardFact<i64>,
    pub hardness: HazardFact<i64>,
    pub hit_points: HazardFact<HazardHitPoints>,
    pub saves: HazardFact<HazardSaves>,
    pub immunities: HazardFact<Vec<HazardIwr>>,
    pub weaknesses: HazardFact<Vec<HazardIwr>>,
    pub resistances: HazardFact<Vec<HazardIwr>>,
    pub unsupported_fields: Vec<HazardUnsupportedFact>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct HazardHitPoints {
    pub current: HazardFact<i64>,
    pub maximum: HazardFact<i64>,
    pub temporary: HazardFact<i64>,
    pub details: HazardFact<RichDocument>,
    pub unsupported_fields: Vec<HazardUnsupportedFact>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct HazardSaves {
    pub fortitude: HazardFact<i64>,
    pub reflex: HazardFact<i64>,
    pub will: HazardFact<i64>,
    pub unsupported_fields: Vec<HazardUnsupportedFact>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum HazardSaveKind {
    Fortitude,
    Reflex,
    Will,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct HazardIwr {
    pub id: HazardComponentId,
    pub authored_order: u32,
    pub iwr_type: HazardFact<String>,
    pub value: HazardFact<i64>,
    pub exceptions: HazardFact<Vec<String>>,
    pub double_vs: HazardFact<Vec<String>>,
}

validated_id!(HazardComponentId, InvalidHazardComponentId);

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct HazardLifecycle {
    pub description: HazardFact<RichDocument>,
    pub disable: HazardFact<RichDocument>,
    pub routine: HazardFact<RichDocument>,
    pub reset: HazardFact<RichDocument>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", content = "value", rename_all = "snake_case")]
pub enum HazardEmitsSound {
    Boolean(bool),
    Named(String),
}

#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct HazardEmbeddedEntities {
    pub entities: Vec<HazardEntity>,
    pub occurrences: Vec<HazardEntityOccurrence>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct HazardEntity {
    pub id: HazardEntityId,
    pub family: HazardEntityFamily,
    pub label: String,
    pub image: HazardFact<String>,
    pub source_identity: HazardEntitySourceIdentity,
    pub capability: HazardCapability,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum HazardEntityFamily {
    Action,
    Strike,
    Condition,
    Effect,
    UnsupportedChild,
}

impl HazardEntityFamily {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Action => "action",
            Self::Strike => "strike",
            Self::Condition => "condition",
            Self::Effect => "effect",
            Self::UnsupportedChild => "unsupported_child",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum HazardEntitySourceIdentity {
    Stable {
        source_id: HazardSourceId,
    },
    Fallback {
        locator: String,
        diagnostic: HazardUnsupportedValue,
    },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct HazardEntityOccurrence {
    pub id: HazardOccurrenceId,
    pub owner_record_key: RecordKey,
    pub entity_id: HazardEntityId,
    pub family: HazardEntityFamily,
    pub authored_order: u32,
    pub source_sort: HazardFact<i64>,
    pub source_folder: HazardFact<String>,
    pub source_ordinal: u32,
    pub contextual_label: HazardFact<String>,
    pub identity_stability: HazardOccurrenceIdentityStability,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum HazardOccurrenceIdentityStability {
    StableSourceIdentity,
    UnstableAuthoredOrdinal,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", content = "value", rename_all = "snake_case")]
pub enum HazardCapability {
    Action(Box<HazardActionCapability>),
    Strike(Box<HazardStrikeCapability>),
    Condition(Box<HazardConditionCapability>),
    Effect(Box<HazardEffectCapability>),
    UnsupportedChild(Box<HazardUnsupportedChildCapability>),
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct HazardItemCommon {
    pub description: HazardFact<RichDocument>,
    pub publication: HazardFact<HazardPublication>,
    pub rules: HazardFact<Vec<HazardRuleElement>>,
    pub slug: HazardFact<String>,
    pub traits: HazardFact<Vec<HazardTrait>>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", content = "value", rename_all = "snake_case")]
pub enum HazardRuleElement {
    Immunity(HazardImmunityRule),
    ActiveEffectLike(HazardActiveEffectLikeRule),
    Aura(HazardAuraRule),
    DamageDice(HazardDamageDiceRule),
    FlatModifier(HazardFlatModifierRule),
    Note(HazardNoteRule),
    Unsupported(HazardUnsupportedRule),
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct HazardImmunityRule {
    pub authored_order: u32,
    pub mode: HazardFact<HazardRuleMode>,
    pub immunity_types: HazardFact<HazardRuleType>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum HazardRuleMode {
    Add,
    Remove,
    Override,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", content = "value", rename_all = "snake_case")]
pub enum HazardRuleType {
    Single(String),
    Multiple(Vec<String>),
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct HazardActiveEffectLikeRule {
    pub authored_order: u32,
    pub mode: HazardFact<HazardRuleMode>,
    pub path: HazardFact<String>,
    pub value: HazardFact<bool>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct HazardAuraRule {
    pub authored_order: u32,
    pub radius: HazardFact<i64>,
    pub slug: HazardFact<String>,
    pub traits: HazardFact<Vec<HazardTrait>>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct HazardDamageDiceRule {
    pub authored_order: u32,
    pub critical: HazardFact<bool>,
    pub dice_number: HazardFact<i64>,
    pub die_size: HazardFact<String>,
    pub damage_type: HazardFact<String>,
    pub selector: HazardFact<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct HazardFlatModifierRule {
    pub authored_order: u32,
    pub critical: HazardFact<bool>,
    pub damage_type: HazardFact<String>,
    pub selector: HazardFact<String>,
    pub value: HazardFact<i64>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct HazardNoteRule {
    pub authored_order: u32,
    pub outcomes: HazardFact<Vec<String>>,
    pub selector: HazardFact<String>,
    pub text: HazardFact<RichDocument>,
    pub title: HazardFact<String>,
    pub visibility: HazardFact<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct HazardUnsupportedRule {
    pub authored_order: u32,
    pub source: HazardUnsupportedValue,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct HazardActionCapability {
    pub common: HazardItemCommon,
    pub action_type: HazardFact<HazardActionType>,
    pub actions: HazardFact<HazardActionCount>,
    pub category: HazardFact<HazardActionCategory>,
    pub death_note: HazardFact<bool>,
    pub frequency: HazardFact<HazardFrequency>,
    pub self_effect: HazardFact<HazardSelfEffect>,
    pub unsupported_fields: Vec<HazardUnsupportedFact>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum HazardActionType {
    Action,
    Reaction,
    Free,
    Passive,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum HazardActionCount {
    One,
    Two,
    Three,
}

impl HazardActionCount {
    pub const fn value(self) -> u8 {
        match self {
            Self::One => 1,
            Self::Two => 2,
            Self::Three => 3,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum HazardActionCategory {
    Interaction,
    Defensive,
    Offensive,
    Familiar,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct HazardFrequency {
    pub value: HazardFact<i64>,
    pub maximum: HazardFact<i64>,
    pub per: HazardFact<HazardFrequencyInterval>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum HazardFrequencyInterval {
    #[serde(rename = "turn")]
    Turn,
    #[serde(rename = "round")]
    Round,
    #[serde(rename = "PT1M")]
    OneMinute,
    #[serde(rename = "PT10M")]
    TenMinutes,
    #[serde(rename = "PT1H")]
    OneHour,
    #[serde(rename = "PT24H")]
    TwentyFourHours,
    #[serde(rename = "day")]
    Day,
    #[serde(rename = "P1W")]
    Week,
    #[serde(rename = "P1M")]
    Month,
    #[serde(rename = "P1Y")]
    Year,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct HazardSelfEffect {
    pub target_uuid: HazardFact<String>,
    pub label: HazardFact<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct HazardStrikeCapability {
    pub common: HazardItemCommon,
    pub bonus: HazardFact<i64>,
    pub attack_effects: HazardFact<Vec<String>>,
    pub damage_rolls: HazardFact<Vec<HazardStrikeDamage>>,
    pub unsupported_fields: Vec<HazardUnsupportedFact>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct HazardStrikeDamage {
    pub source_key: String,
    pub authored_order: u32,
    pub damage: HazardFact<String>,
    pub damage_type: HazardFact<String>,
    pub category: HazardFact<HazardDamageCategory>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum HazardDamageCategory {
    Persistent,
    Precision,
    Splash,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct HazardConditionCapability {
    pub common: HazardItemCommon,
    pub unsupported_fields: Vec<HazardUnsupportedFact>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct HazardEffectCapability {
    pub common: HazardItemCommon,
    pub unsupported_fields: Vec<HazardUnsupportedFact>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct HazardUnsupportedChildCapability {
    pub child_type: String,
    pub common: HazardItemCommon,
    pub unsupported_fields: Vec<HazardUnsupportedFact>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct HazardUnsupportedFact {
    pub field: HazardUnsupportedField,
    pub value: HazardUnsupportedValue,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", content = "value", rename_all = "snake_case")]
pub enum HazardUnsupportedField {
    HazardUnexpected(String),
    DefensesHasHealth,
    HitPointsTempMax,
    SaveDetail(HazardSaveKind),
    ActionUnexpected(String),
    StrikeUnexpected(String),
    StrikeAttack,
    StrikeWeaponType,
    StrikeAttackEffectsCustom,
    StrikeTraitRarity,
    ConditionUnexpected(String),
    EffectUnexpected(String),
    UnsupportedChildField(String),
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct HazardRelationship {
    pub id: HazardRelationshipId,
    pub authored_order: u32,
    pub source_occurrence_id: Option<HazardOccurrenceId>,
    pub kind: HazardRelationshipKind,
    pub target: HazardRelationshipTarget,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum HazardRelationshipKind {
    Contains,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", content = "value", rename_all = "snake_case")]
pub enum HazardRelationshipTarget {
    Entity(HazardEntityId),
    Occurrence(HazardOccurrenceId),
}
