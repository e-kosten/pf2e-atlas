use atlas_domain::RecordKey;

use crate::{ActivityRollAbility, CreatureSourceId, FactValue, UnsupportedSourceValue};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CreatureEmbeddedEntities {
    pub entities: Vec<CreatureEntity>,
    pub occurrences: Vec<CreatureEntityOccurrence>,
    pub relationships: Vec<CreatureEntityRelationship>,
    pub actor_spellcasting: FactValue<CreatureActorSpellcastingContext>,
}

impl Default for CreatureEmbeddedEntities {
    fn default() -> Self {
        Self {
            entities: Vec::new(),
            occurrences: Vec::new(),
            relationships: Vec::new(),
            actor_spellcasting: FactValue::Missing,
        }
    }
}

impl CreatureEmbeddedEntities {
    pub fn occurrences_of(
        &self,
        family: CreatureEntityFamily,
    ) -> impl Iterator<Item = &CreatureEntityOccurrence> {
        self.occurrences
            .iter()
            .filter(move |occurrence| occurrence.family == family)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CreatureEntity {
    pub id: CreatureEntityId,
    pub family: CreatureEntityFamily,
    pub label: String,
    pub source_identity: CreatureEntitySourceIdentity,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct CreatureEntityId(String);

impl CreatureEntityId {
    pub fn new(value: impl Into<String>) -> Result<Self, InvalidCreatureEntityId> {
        let value = value.into();
        if value.is_empty() || value.chars().any(char::is_whitespace) {
            return Err(InvalidCreatureEntityId);
        }
        Ok(Self(value))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }

    pub fn actor_owned(
        owner: &RecordKey,
        family: CreatureEntityFamily,
        occurrence: &CreatureOccurrenceId,
    ) -> Self {
        Self(format!(
            "actor-owned:{owner}:{}:{}",
            family.as_str(),
            occurrence.as_str()
        ))
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InvalidCreatureEntityId;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum CreatureEntityFamily {
    Strike,
    Action,
    SpellcastingEntry,
    Spell,
    Equipment,
    Lore,
    Affliction,
    Armor,
    Backpack,
    Book,
    Condition,
    Consumable,
    Effect,
    Shield,
    Treasure,
    Weapon,
    Unsupported,
}

impl CreatureEntityFamily {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Strike => "strike",
            Self::Action => "action",
            Self::SpellcastingEntry => "spellcasting-entry",
            Self::Spell => "spell",
            Self::Equipment => "equipment",
            Self::Lore => "lore",
            Self::Affliction => "affliction",
            Self::Armor => "armor",
            Self::Backpack => "backpack",
            Self::Book => "book",
            Self::Condition => "condition",
            Self::Consumable => "consumable",
            Self::Effect => "effect",
            Self::Shield => "shield",
            Self::Treasure => "treasure",
            Self::Weapon => "weapon",
            Self::Unsupported => "unsupported",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CreatureEntitySourceIdentity {
    pub nested_source_id: FactValue<CreatureSourceId>,
    pub stable_source_locator: FactValue<StableSourceLocator>,
    pub source_locators: Vec<CreatureSourceLocator>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CreatureSourceLocator {
    pub source_path: String,
    pub locator: StableSourceLocator,
    pub precedence: u8,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct StableSourceLocator(String);

impl StableSourceLocator {
    pub fn new(value: impl Into<String>) -> Result<Self, InvalidStableSourceLocator> {
        let value = value.into();
        if value.trim().is_empty() {
            return Err(InvalidStableSourceLocator);
        }
        Ok(Self(value))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InvalidStableSourceLocator;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CreatureEntityOccurrence {
    pub id: CreatureOccurrenceId,
    pub identity_stability: OccurrenceIdentityStability,
    pub owner: RecordKey,
    pub target: CreatureEntityTarget,
    pub family: CreatureEntityFamily,
    pub authored_order: u32,
    pub source_sort: FactValue<i64>,
    pub source_folder: FactValue<String>,
    pub source_identity: CreatureEntitySourceIdentity,
    pub parent: CreatureOccurrenceParent,
    pub context: CreatureOccurrenceContext,
    pub capability: CreatureCapability,
    pub deltas: Vec<CreatureOccurrenceDelta>,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct CreatureOccurrenceId(String);

impl CreatureOccurrenceId {
    pub fn new(value: impl Into<String>) -> Result<Self, InvalidCreatureEntityId> {
        CreatureEntityId::new(value).map(|id| Self(id.0))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }

    pub fn stable_nested(
        owner: &RecordKey,
        family: CreatureEntityFamily,
        nested_source_id: &CreatureSourceId,
    ) -> Self {
        Self(format!(
            "occurrence:{owner}:{}:{}",
            family.as_str(),
            nested_source_id.as_str()
        ))
    }

    pub fn unstable_ordinal(
        owner: &RecordKey,
        family: CreatureEntityFamily,
        ordinal: usize,
    ) -> Self {
        Self(format!(
            "occurrence:{owner}:{}:ordinal-{ordinal}",
            family.as_str()
        ))
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum OccurrenceIdentityStability {
    StableNestedSourceId,
    UnstableOwnerFamilyOrdinal,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CreatureEntityTarget {
    CanonicalRecord(RecordKey),
    ActorOwned(CreatureEntityId),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CreatureOccurrenceParent {
    Creature,
    SpellcastingEntry(CreatureOccurrenceId),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CreatureOccurrenceContext {
    pub group: FactValue<String>,
    pub rank: FactValue<i64>,
    pub location: FactValue<String>,
    pub slot: FactValue<String>,
    pub uses: FactValue<CreatureUseLimit>,
    pub contextual_label: FactValue<String>,
}

impl Default for CreatureOccurrenceContext {
    fn default() -> Self {
        Self {
            group: FactValue::Missing,
            rank: FactValue::Missing,
            location: FactValue::Missing,
            slot: FactValue::Missing,
            uses: FactValue::Missing,
            contextual_label: FactValue::Missing,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CreatureOccurrenceDelta {
    pub field_path: String,
    pub source_state: FactValue<CreatureDeltaValue>,
    pub canonical_value: FactValue<CreatureDeltaValue>,
    pub local_value: FactValue<CreatureDeltaValue>,
    pub disposition: CreatureDeltaDisposition,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CreatureDeltaValue {
    Text(String),
    Integer(i64),
    Boolean(bool),
    Uses(CreatureUseLimit),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum CreatureDeltaDisposition {
    ContextualValue,
    ExplicitOverride,
    ExplicitSuppression,
    UnsupportedPreserved,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CreatureCapability {
    Strike(CreatureStrikeCapability),
    Action(CreatureActionCapability),
    SpellcastingEntry(CreatureSpellcastingEntryCapability),
    Spell(CreatureSpellCapability),
    Equipment(CreatureEquipmentCapability),
    Lore(CreatureLoreCapability),
    Unsupported(CreatureUnsupportedCapability),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CreatureUnsupportedCapability {
    pub source_item_type: String,
    pub source_slug: FactValue<String>,
    pub traits: FactValue<Vec<String>>,
    pub unsupported_notes: Vec<UnsupportedMechanicNote>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CreatureStrikeCapability {
    pub traits: FactValue<Vec<String>>,
    pub attack_effects: FactValue<Vec<String>>,
    pub rolls: Vec<CreatureRoll>,
    pub damage: FactValue<Vec<CreatureDamage>>,
    pub action_cost: CreatureActionCost,
    pub unsupported_notes: Vec<UnsupportedMechanicNote>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CreatureActionCapability {
    pub category: FactValue<String>,
    pub traits: FactValue<Vec<String>>,
    pub action_cost: CreatureActionCost,
    pub frequency: FactValue<CreatureFrequency>,
    pub self_effect: FactValue<String>,
    pub self_effect_label: FactValue<String>,
    pub requirements: FactValue<String>,
    pub cost: FactValue<String>,
    pub rolls: Vec<CreatureRoll>,
    pub damage: FactValue<Vec<CreatureDamage>>,
    pub unsupported_notes: Vec<UnsupportedMechanicNote>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CreatureSpellcastingEntryCapability {
    pub preparation: FactValue<CreatureSpellPreparation>,
    pub tradition: FactValue<String>,
    pub attack: FactValue<i64>,
    pub dc: FactValue<i64>,
    pub slots: FactValue<Vec<CreatureSpellSlot>>,
    pub unsupported_notes: Vec<UnsupportedMechanicNote>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CreatureSpellPreparation {
    Prepared,
    Spontaneous,
    Focus,
    Innate,
    Ritual,
    Unsupported(UnsupportedSourceValue),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CreatureSpellSlot {
    pub rank: i64,
    pub maximum: FactValue<CreatureSourceScalar<i64>>,
    pub serialized_value: FactValue<CreatureSourceScalar<i64>>,
    pub prepared: FactValue<Vec<CreaturePreparedSpellSlot>>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CreaturePreparedSpellSlot {
    Spell {
        id: FactValue<CreatureSourceId>,
        name: FactValue<String>,
        expended: FactValue<bool>,
        prepared: FactValue<bool>,
        authored_order: u32,
    },
    Unsupported(UnsupportedSourceValue),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CreatureSpellCapability {
    pub traits: FactValue<Vec<String>>,
    pub base_rank: FactValue<i64>,
    pub signature: FactValue<bool>,
    pub traditions: FactValue<Vec<String>>,
    pub requirements: FactValue<String>,
    pub cost: FactValue<String>,
    pub counteraction: FactValue<bool>,
    pub ritual: FactValue<CreatureRitualContext>,
    pub target: FactValue<String>,
    pub area: FactValue<CreatureSpellArea>,
    pub range: FactValue<String>,
    pub time: FactValue<String>,
    pub duration: FactValue<CreatureSpellDuration>,
    pub defense: FactValue<CreatureSpellDefense>,
    pub damage: FactValue<Vec<CreatureDamage>>,
    pub action_cost: CreatureActionCost,
    pub unsupported_notes: Vec<UnsupportedMechanicNote>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CreatureRitualContext {
    pub primary_check: FactValue<String>,
    pub secondary_casters: FactValue<CreatureSourceScalar<i64>>,
    pub secondary_checks: FactValue<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CreatureSpellArea {
    pub area_type: FactValue<String>,
    pub value: FactValue<i64>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CreatureSpellDuration {
    pub value: FactValue<String>,
    pub sustained: FactValue<bool>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CreatureSpellDefense {
    pub save: FactValue<CreatureSpellSave>,
    pub basic: FactValue<bool>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CreatureSpellSave {
    Fortitude,
    Reflex,
    Will,
    Unsupported(UnsupportedSourceValue),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CreatureEquipmentCapability {
    pub traits: FactValue<Vec<String>>,
    pub level: FactValue<i64>,
    pub usage: FactValue<String>,
    pub quantity: FactValue<i64>,
    pub uses: FactValue<CreatureUseLimit>,
    pub unsupported_notes: Vec<UnsupportedMechanicNote>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CreatureLoreCapability {
    pub modifier: FactValue<i64>,
    pub unsupported_notes: Vec<UnsupportedMechanicNote>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CreatureActionCost {
    Passive,
    Reaction,
    FreeAction,
    Actions(u8),
    Time(String),
    Unsupported(UnsupportedSourceValue),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CreatureFrequency {
    pub maximum: FactValue<i64>,
    pub period: FactValue<String>,
    pub serialized_value: FactValue<i64>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CreatureActorSpellcastingContext {
    pub rituals_dc: FactValue<CreatureSourceScalar<i64>>,
    pub unsupported_notes: Vec<UnsupportedMechanicNote>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CreatureEntityRelationship {
    pub source: CreatureOccurrenceId,
    pub kind: CreatureEntityRelationshipKind,
    pub target: CreatureRelationshipTarget,
    pub source_path: String,
    pub contextual_label: FactValue<String>,
    pub lifecycle: FactValue<String>,
    pub execution: CreatureRelationshipExecution,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum CreatureEntityRelationshipKind {
    GrantedBy,
    ItemGrant,
    LinkedWeapon,
    PreparedSpell,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CreatureRelationshipTarget {
    Occurrence(CreatureOccurrenceId),
    UnresolvedNestedSourceId(CreatureSourceId),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum CreatureRelationshipExecution {
    ProvenanceOnly,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CreatureUseLimit {
    pub maximum: FactValue<i64>,
    pub serialized_value: FactValue<i64>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CreatureRoll {
    pub id: String,
    pub label: String,
    pub kind: CreatureRollKind,
    pub value: FactValue<i64>,
    pub ability: FactValue<ActivityRollAbility>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum CreatureRollKind {
    Attack,
    DifficultyClass,
    Check,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CreatureDamage {
    pub id: String,
    pub formula: FactValue<String>,
    pub damage_type: FactValue<String>,
    pub category: FactValue<String>,
    pub kinds: FactValue<Vec<CreatureDamageKind>>,
    pub apply_modifier: FactValue<CreatureSourceScalar<bool>>,
}

/// A serialized scalar whose exact source shape is part of the creature fact.
///
/// This keeps supported values typed while retaining unexpected source values
/// without coercing them or dropping the owning embedded entity.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CreatureSourceScalar<T> {
    Value(T),
    Unsupported(UnsupportedSourceValue),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CreatureDamageKind {
    Damage,
    Healing,
    Unsupported(UnsupportedSourceValue),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UnsupportedMechanicNote {
    pub source_path: String,
    pub value: UnsupportedSourceValue,
}

#[cfg(test)]
mod tests {
    use atlas_domain::{PackName, RecordId, RecordKey};

    use super::{
        CreatureEmbeddedEntities, CreatureEntityFamily, CreatureEntityOccurrence,
        CreatureEntitySourceIdentity, CreatureEntityTarget, CreatureOccurrenceContext,
        CreatureOccurrenceId, CreatureOccurrenceParent, OccurrenceIdentityStability,
    };
    use crate::{CreatureActionCapability, CreatureActionCost, CreatureCapability, FactValue};

    #[test]
    fn repeated_targets_remain_distinct_and_authored_order_is_data() {
        let owner = key("bestiary", "actor");
        let target = key("spells", "fireball");
        let occurrence = |id: &str, order| CreatureEntityOccurrence {
            id: CreatureOccurrenceId::new(id).expect("occurrence id"),
            identity_stability: OccurrenceIdentityStability::StableNestedSourceId,
            owner: owner.clone(),
            target: CreatureEntityTarget::CanonicalRecord(target.clone()),
            family: CreatureEntityFamily::Spell,
            authored_order: order,
            source_sort: FactValue::Missing,
            source_folder: FactValue::Missing,
            source_identity: CreatureEntitySourceIdentity {
                nested_source_id: FactValue::Missing,
                stable_source_locator: FactValue::Missing,
                source_locators: Vec::new(),
            },
            parent: CreatureOccurrenceParent::Creature,
            context: CreatureOccurrenceContext::default(),
            capability: CreatureCapability::Action(CreatureActionCapability {
                category: FactValue::Missing,
                traits: FactValue::Missing,
                action_cost: CreatureActionCost::Passive,
                frequency: FactValue::Missing,
                self_effect: FactValue::Missing,
                self_effect_label: FactValue::Missing,
                requirements: FactValue::Missing,
                cost: FactValue::Missing,
                rolls: Vec::new(),
                damage: FactValue::Value(Vec::new()),
                unsupported_notes: Vec::new(),
            }),
            deltas: Vec::new(),
        };
        let mut embedded = CreatureEmbeddedEntities {
            entities: Vec::new(),
            occurrences: vec![occurrence("occ:one", 0), occurrence("occ:two", 1)],
            relationships: Vec::new(),
            actor_spellcasting: FactValue::Missing,
        };
        embedded.occurrences.reverse();
        embedded.occurrences[0].authored_order = 0;
        embedded.occurrences[1].authored_order = 1;

        let ids = embedded
            .occurrences_of(CreatureEntityFamily::Spell)
            .map(|occurrence| occurrence.id.as_str())
            .collect::<Vec<_>>();
        assert_eq!(ids, ["occ:two", "occ:one"]);
        assert!(embedded.occurrences.iter().all(|occurrence| {
            occurrence.target == CreatureEntityTarget::CanonicalRecord(target.clone())
        }));
    }

    fn key(pack: &str, id: &str) -> RecordKey {
        RecordKey::new(
            PackName::new(pack.to_string()).expect("pack"),
            RecordId::new(id.to_string()).expect("id"),
        )
    }
}
