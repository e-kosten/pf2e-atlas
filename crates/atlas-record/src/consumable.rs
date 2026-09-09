use atlas_domain::{Rarity, RecordKey};

use crate::{
    FactValue, OwnedRichContent, PublicationLicense, SpellChildId, StableSourceLocator,
    UnsupportedSourceValue,
};

pub type ConsumableFact<T> = FactValue<ConsumableSourceValue<T>>;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ConsumableSourceValue<T> {
    Known(T),
    Unsupported(UnsupportedSourceValue),
}

impl<T> ConsumableSourceValue<T> {
    pub fn known(&self) -> Option<&T> {
        match self {
            Self::Known(value) => Some(value),
            Self::Unsupported(_) => None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConsumableRecord {
    pub identity: ConsumableIdentity,
    pub definition: ConsumableDefinition,
    pub source_state: ConsumableSourceState,
    pub content: OwnedRichContent,
    pub unsupported_content: Vec<UnsupportedSourceValue>,
    pub provenance: ConsumableProvenance,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConsumableIdentity {
    pub record_key: RecordKey,
    pub source_id: ConsumableSourceId,
    pub name: String,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ConsumableSourceId(String);

impl ConsumableSourceId {
    pub fn new(value: impl Into<String>) -> Result<Self, InvalidConsumableId> {
        validated_id(value.into()).map(Self)
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConsumableDefinition {
    pub slug: ConsumableFact<String>,
    pub level: ConsumableFact<i64>,
    pub category: ConsumableFact<String>,
    pub rarity: ConsumableFact<Rarity>,
    pub traits: ConsumableFact<Vec<String>>,
    pub other_tags: ConsumableFact<Vec<String>>,
    pub base_item: ConsumableFact<String>,
    pub bulk: ConsumableFact<ConsumableExactDecimal>,
    pub size: ConsumableFact<String>,
    pub stack_group: ConsumableFact<String>,
    pub material: ConsumableFact<ConsumableMaterial>,
    pub price: ConsumableFact<ConsumablePrice>,
    pub usage: ConsumableFact<String>,
    pub maximum_uses: ConsumableFact<i64>,
    pub auto_destroy: ConsumableFact<bool>,
    pub maximum_hp: ConsumableFact<i64>,
    pub hardness: ConsumableFact<i64>,
    pub damage: ConsumableFact<ConsumableDamage>,
    pub publication: ConsumableFact<ConsumablePublication>,
    pub rules: ConsumableFact<Vec<UnsupportedSourceValue>>,
    pub spell_child_id: FactValue<SpellChildId>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConsumableSourceState {
    pub quantity: ConsumableFact<i64>,
    pub current_uses: ConsumableFact<i64>,
    pub current_hp: ConsumableFact<i64>,
    pub container_id: ConsumableFact<String>,
    pub equipped: ConsumableFact<ConsumableEquippedState>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConsumableEquippedState {
    pub carry_type: ConsumableFact<String>,
    pub hands_held: ConsumableFact<i64>,
    pub in_slot: ConsumableFact<bool>,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ConsumableExactDecimal(String);

impl ConsumableExactDecimal {
    pub fn new(value: impl Into<String>) -> Result<Self, InvalidConsumableValue> {
        let value = value.into();
        if value.starts_with(['-', '+']) {
            return Err(InvalidConsumableValue);
        }
        let (whole, fraction) = value
            .split_once('.')
            .map_or((value.as_str(), None), |(whole, fraction)| {
                (whole, Some(fraction))
            });
        let whole = whole.parse::<i64>().map_err(|_| InvalidConsumableValue)?;
        let fraction = match fraction {
            None => 0_i64,
            Some(fraction) if fraction.len() == 1 => fraction
                .parse::<i64>()
                .map_err(|_| InvalidConsumableValue)?,
            Some(_) => return Err(InvalidConsumableValue),
        };
        if whole
            .checked_mul(10)
            .and_then(|value| value.checked_add(fraction))
            .is_none()
        {
            return Err(InvalidConsumableValue);
        }
        Ok(Self(value))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }

    pub fn as_f64(&self) -> Option<f64> {
        self.0.parse::<f64>().ok().filter(|value| value.is_finite())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConsumableMaterial {
    pub grade: ConsumableFact<String>,
    pub material_type: ConsumableFact<String>,
    pub effects: ConsumableFact<Vec<String>>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConsumablePrice {
    pub denominations: ConsumableFact<Vec<ConsumablePriceDenomination>>,
    pub per: ConsumableFact<i64>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConsumablePriceDenomination {
    pub denomination: String,
    pub amount: i64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConsumableDamage {
    pub formula: ConsumableFact<String>,
    pub category: ConsumableFact<String>,
    pub damage_type: ConsumableFact<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConsumablePublication {
    pub title: ConsumableFact<String>,
    pub license: ConsumableFact<PublicationLicense>,
    pub remaster: ConsumableFact<bool>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConsumableProvenance {
    pub image: ConsumableFact<String>,
    pub folder: ConsumableFact<String>,
    pub source_sort: ConsumableFact<i64>,
    pub target_locator: ConsumableLocatorState,
    pub source_path: String,
    pub source_contract_version: String,
    pub source_system_version: String,
    pub source_upstream_commit: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ConsumableOccurrenceSet {
    pub entities: Vec<ConsumableEntity>,
    pub occurrences: Vec<ConsumableOccurrence>,
}

/// Invalid attachment membership is corruption, never an absent occurrence.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ConsumableOccurrenceSetError {
    DuplicateEntity,
    MissingEntity,
    OrphanEntity,
    WrongOwner,
    DuplicateOccurrence,
    InvalidOrder,
}

impl std::fmt::Display for ConsumableOccurrenceSetError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "invalid consumable occurrence graph: {self:?}")
    }
}

impl std::error::Error for ConsumableOccurrenceSetError {}

impl ConsumableOccurrenceSet {
    pub fn validated_entities(
        &self,
    ) -> Result<
        std::collections::BTreeMap<&ConsumableEntityId, &ConsumableEntity>,
        ConsumableOccurrenceSetError,
    > {
        use ConsumableOccurrenceSetError as Error;
        let mut entities = std::collections::BTreeMap::new();
        let owner = self.entities.first().map(|entity| &entity.owner_record_key);
        for entity in &self.entities {
            if Some(&entity.owner_record_key) != owner {
                return Err(Error::WrongOwner);
            }
            if entities.insert(&entity.id, entity).is_some() {
                return Err(Error::DuplicateEntity);
            }
        }
        let mut used = std::collections::BTreeSet::new();
        let mut ids = std::collections::BTreeSet::new();
        let mut previous_order = None;
        for occurrence in &self.occurrences {
            let entity = entities
                .get(&occurrence.entity_id)
                .ok_or(Error::MissingEntity)?;
            if entity.owner_record_key != occurrence.owner_record_key {
                return Err(Error::WrongOwner);
            }
            if !ids.insert(&occurrence.id) {
                return Err(Error::DuplicateOccurrence);
            }
            if previous_order.is_some_and(|order| order >= occurrence.authored_order) {
                return Err(Error::InvalidOrder);
            }
            previous_order = Some(occurrence.authored_order);
            used.insert(&occurrence.entity_id);
        }
        if used.len() != entities.len() {
            return Err(Error::OrphanEntity);
        }
        Ok(entities)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConsumableEntity {
    pub id: ConsumableEntityId,
    pub owner_record_key: RecordKey,
    pub target: ConsumableEntityTarget,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ConsumableEntityId(String);

impl ConsumableEntityId {
    pub fn new(value: impl Into<String>) -> Result<Self, InvalidConsumableId> {
        validated_id(value.into()).map(Self)
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ConsumableEntityTarget {
    Resolved {
        record_key: RecordKey,
        immutable_mismatches: Vec<ConsumableMismatch>,
    },
    ParentOwned {
        definition: Box<ConsumableDefinition>,
        resolution: ConsumableTargetResolution,
        content_identity: FactValue<ConsumableContentIdentity>,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConsumableContentIdentity {
    pub content_key: String,
    pub content_hash: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ConsumableTargetResolution {
    NoLocator,
    MalformedOrDuplicateLocator(UnsupportedSourceValue),
    TargetMissing(StableSourceLocator),
    WrongDocumentOrFamily(StableSourceLocator),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConsumableMismatch {
    pub field_path: String,
    pub local_value: ConsumableMismatchValue,
    pub target_value: ConsumableMismatchValue,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ConsumableMismatchValue {
    String(ConsumableFact<String>),
    Integer(ConsumableFact<i64>),
    Boolean(ConsumableFact<bool>),
    Rarity(ConsumableFact<Rarity>),
    Strings(ConsumableFact<Vec<String>>),
    ExactDecimal(ConsumableFact<ConsumableExactDecimal>),
    Material(ConsumableFact<ConsumableMaterial>),
    Price(ConsumableFact<ConsumablePrice>),
    Damage(ConsumableFact<ConsumableDamage>),
    Publication(ConsumableFact<ConsumablePublication>),
    Rules(ConsumableFact<Vec<UnsupportedSourceValue>>),
    SpellChildId(FactValue<SpellChildId>),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConsumableOccurrence {
    pub id: ConsumableOccurrenceId,
    pub source_id: ConsumableFact<ConsumableSourceId>,
    pub identity_stability: ConsumableOccurrenceIdentityStability,
    pub owner_record_key: RecordKey,
    pub entity_id: ConsumableEntityId,
    pub authored_order: u32,
    pub source_path: String,
    pub source_sort: ConsumableFact<i64>,
    pub source_image: ConsumableFact<String>,
    pub source_folder: ConsumableFact<String>,
    pub contextual_name: String,
    pub locator: ConsumableLocatorState,
    pub state: ConsumableSourceState,
    pub spell_reuse: ConsumableSpellReuse,
    pub authored_content: OwnedRichContent,
    pub unsupported_content: Vec<UnsupportedSourceValue>,
}

impl ConsumableOccurrence {
    /// The occurrence owns its authored prose and any retained local Spell child.
    /// Reused target children are not copied into this traversal.
    pub fn owned_content(&self) -> impl Iterator<Item = &crate::OwnedRichContent> {
        let child = match &self.spell_reuse {
            ConsumableSpellReuse::Mismatch {
                local_evidence: ConsumableLocalSpellEvidence::Child(child),
                ..
            } => Some(&child.definition.content),
            _ => None,
        };
        std::iter::once(&self.authored_content).chain(child)
    }

    pub fn owned_content_mut(&mut self) -> impl Iterator<Item = &mut crate::OwnedRichContent> {
        let child = match &mut self.spell_reuse {
            ConsumableSpellReuse::Mismatch {
                local_evidence: ConsumableLocalSpellEvidence::Child(child),
                ..
            } => Some(&mut child.definition.content),
            _ => None,
        };
        std::iter::once(&mut self.authored_content).chain(child)
    }

    /// Approved occurrence prose stays searchable without restoring generic content ownership.
    pub fn searchable_content_documents(
        &self,
    ) -> impl Iterator<Item = &crate::OwnedRichContentDocument> {
        self.owned_content()
            .flat_map(|content| &content.documents)
            .filter(|document| {
                // A target-equal copy is still this parent's authored capability prose.
                document.visibility == crate::ContentVisibility::Public
            })
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ConsumableOccurrenceId(String);

impl ConsumableOccurrenceId {
    pub fn new(value: impl Into<String>) -> Result<Self, InvalidConsumableId> {
        validated_id(value.into()).map(Self)
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum ConsumableOccurrenceIdentityStability {
    StableSourceIdentity,
    UnstableOwnerOrdinal,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ConsumableLocatorState {
    Missing,
    Null,
    Known(StableSourceLocator),
    Unsupported(UnsupportedSourceValue),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ConsumableSpellReuse {
    NotPresent,
    Reused {
        target_child_id: SpellChildId,
    },
    Mismatch {
        reason: ConsumableSpellMismatchReason,
        local_evidence: ConsumableLocalSpellEvidence,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ConsumableLocalSpellEvidence {
    Missing,
    Null,
    Malformed(UnsupportedSourceValue),
    Child(Box<crate::ConsumableSpellChild>),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConsumableSpellMismatchReason {
    UnresolvedParent,
    TargetWithoutChild,
    LocalChildMissing,
    LocalChildMalformed,
    ChildIdentity,
    SourceContext,
    Definition,
    ContentOrReferences,
    OverlayOrFormOrder,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InvalidConsumableId;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InvalidConsumableValue;

fn validated_id(value: String) -> Result<String, InvalidConsumableId> {
    if value.is_empty() || value.chars().any(char::is_whitespace) {
        Err(InvalidConsumableId)
    } else {
        Ok(value)
    }
}

#[cfg(test)]
mod tests {
    use super::{ConsumableExactDecimal, ConsumableSourceId};

    #[test]
    fn identities_and_exact_bulk_reject_ambiguous_values() {
        assert!(ConsumableSourceId::new("wand-id").is_ok());
        assert!(ConsumableSourceId::new("wand id").is_err());
        assert_eq!(
            ConsumableExactDecimal::new("0.1")
                .expect("exact decimal")
                .as_str(),
            "0.1"
        );
        assert!(ConsumableExactDecimal::new("-1").is_err());
        assert!(ConsumableExactDecimal::new("-0.1").is_err());
        assert!(ConsumableExactDecimal::new("+1").is_err());
        assert!(ConsumableExactDecimal::new("1.2.3").is_err());
        assert!(ConsumableExactDecimal::new("1.25").is_err());
        assert!(ConsumableExactDecimal::new(format!("{}0", i64::MAX)).is_err());
    }
}
