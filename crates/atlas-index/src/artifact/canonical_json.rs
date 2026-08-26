use atlas_domain::RecordKey;
use atlas_record::*;
use serde_json::{Map, Value};

pub(crate) trait CanonicalJson: Sized {
    fn to_canonical_json(&self) -> Value;
    fn from_canonical_json(value: Value, path: &str) -> Result<Self, String>;
}

pub(crate) fn encode<T: CanonicalJson>(value: &T) -> Result<String, String> {
    serde_json::to_string(&value.to_canonical_json()).map_err(|error| error.to_string())
}

pub(crate) fn decode<T: CanonicalJson>(value: &str, path: &str) -> Result<T, String> {
    let parsed = serde_json::from_str(value).map_err(|error| format!("{path}: {error}"))?;
    T::from_canonical_json(parsed, path)
}

fn object(value: Value, path: &str) -> Result<Map<String, Value>, String> {
    value
        .as_object()
        .cloned()
        .ok_or_else(|| format!("{path}: expected object"))
}

fn field<T: CanonicalJson>(
    object: &mut Map<String, Value>,
    name: &'static str,
    path: &str,
) -> Result<T, String> {
    let value = object
        .remove(name)
        .ok_or_else(|| format!("{path}.{name}: required field is missing"))?;
    T::from_canonical_json(value, &format!("{path}.{name}"))
}

fn finish(object: Map<String, Value>, path: &str) -> Result<(), String> {
    if object.is_empty() {
        Ok(())
    } else {
        Err(format!(
            "{path}: unknown fields: {}",
            object.keys().cloned().collect::<Vec<_>>().join(", ")
        ))
    }
}

fn object_value<const N: usize>(fields: [(&str, Value); N]) -> Value {
    Value::Object(
        fields
            .into_iter()
            .map(|(name, value)| (name.to_string(), value))
            .collect(),
    )
}

fn tagged(tag: &str, value: Option<Value>) -> Value {
    let mut object = Map::new();
    object.insert("kind".to_string(), Value::String(tag.to_string()));
    if let Some(value) = value {
        object.insert("value".to_string(), value);
    }
    Value::Object(object)
}

fn take_tag(value: Value, path: &str) -> Result<(String, Option<Value>), String> {
    let mut object = object(value, path)?;
    let kind = field::<String>(&mut object, "kind", path)?;
    let value = object.remove("value");
    finish(object, path)?;
    Ok((kind, value))
}

impl CanonicalJson for String {
    fn to_canonical_json(&self) -> Value {
        Value::String(self.clone())
    }

    fn from_canonical_json(value: Value, path: &str) -> Result<Self, String> {
        value
            .as_str()
            .map(str::to_string)
            .ok_or_else(|| format!("{path}: expected string"))
    }
}

impl CanonicalJson for bool {
    fn to_canonical_json(&self) -> Value {
        Value::Bool(*self)
    }

    fn from_canonical_json(value: Value, path: &str) -> Result<Self, String> {
        value
            .as_bool()
            .ok_or_else(|| format!("{path}: expected boolean"))
    }
}

impl CanonicalJson for i64 {
    fn to_canonical_json(&self) -> Value {
        Value::Number((*self).into())
    }

    fn from_canonical_json(value: Value, path: &str) -> Result<Self, String> {
        value
            .as_i64()
            .ok_or_else(|| format!("{path}: expected integer"))
    }
}

macro_rules! unsigned_json {
    ($ty:ty) => {
        impl CanonicalJson for $ty {
            fn to_canonical_json(&self) -> Value {
                Value::Number((*self).into())
            }

            fn from_canonical_json(value: Value, path: &str) -> Result<Self, String> {
                let value = value
                    .as_u64()
                    .ok_or_else(|| format!("{path}: expected unsigned integer"))?;
                value
                    .try_into()
                    .map_err(|_| format!("{path}: unsigned integer is out of range"))
            }
        }
    };
}
unsigned_json!(u32);
unsigned_json!(u8);

impl<T: CanonicalJson> CanonicalJson for Vec<T> {
    fn to_canonical_json(&self) -> Value {
        Value::Array(self.iter().map(CanonicalJson::to_canonical_json).collect())
    }

    fn from_canonical_json(value: Value, path: &str) -> Result<Self, String> {
        let values = value
            .as_array()
            .cloned()
            .ok_or_else(|| format!("{path}: expected array"))?;
        values
            .into_iter()
            .enumerate()
            .map(|(index, value)| T::from_canonical_json(value, &format!("{path}[{index}]")))
            .collect()
    }
}

impl<T: CanonicalJson> CanonicalJson for Option<T> {
    fn to_canonical_json(&self) -> Value {
        self.as_ref()
            .map(CanonicalJson::to_canonical_json)
            .unwrap_or(Value::Null)
    }

    fn from_canonical_json(value: Value, path: &str) -> Result<Self, String> {
        if value.is_null() {
            Ok(None)
        } else {
            T::from_canonical_json(value, path).map(Some)
        }
    }
}

macro_rules! serde_json_codec {
    ($($ty:ty),+ $(,)?) => {$ (
        impl CanonicalJson for $ty {
            fn to_canonical_json(&self) -> Value {
                serde_json::to_value(self).expect("storage-neutral serde value")
            }

            fn from_canonical_json(value: Value, path: &str) -> Result<Self, String> {
                serde_json::from_value(value).map_err(|error| format!("{path}: {error}"))
            }
        }
    )+ };
}
serde_json_codec!(RecordKey, RichDocument);

impl<T: CanonicalJson> CanonicalJson for FactValue<T> {
    fn to_canonical_json(&self) -> Value {
        match self {
            Self::Missing => tagged("missing", None),
            Self::Null => tagged("null", None),
            Self::Value(value) => tagged("value", Some(value.to_canonical_json())),
        }
    }

    fn from_canonical_json(value: Value, path: &str) -> Result<Self, String> {
        let (kind, value) = take_tag(value, path)?;
        match (kind.as_str(), value) {
            ("missing", None) => Ok(Self::Missing),
            ("null", None) => Ok(Self::Null),
            ("value", Some(value)) => T::from_canonical_json(value, path).map(Self::Value),
            _ => Err(format!("{path}: invalid fact value `{kind}`")),
        }
    }
}

impl<T: CanonicalJson> CanonicalJson for CreatureFact<T> {
    fn to_canonical_json(&self) -> Value {
        object_value([
            ("value", self.value.to_canonical_json()),
            ("provenance", self.provenance.to_canonical_json()),
        ])
    }

    fn from_canonical_json(value: Value, path: &str) -> Result<Self, String> {
        let mut value = object(value, path)?;
        let result = Self {
            value: field(&mut value, "value", path)?,
            provenance: field(&mut value, "provenance", path)?,
        };
        finish(value, path)?;
        Ok(result)
    }
}

macro_rules! struct_json {
    ($ty:path { $($field:ident),+ $(,)? }) => {
        impl CanonicalJson for $ty {
            fn to_canonical_json(&self) -> Value {
                let mut object = Map::new();
                $(object.insert(stringify!($field).to_string(), self.$field.to_canonical_json());)+
                Value::Object(object)
            }

            fn from_canonical_json(value: Value, path: &str) -> Result<Self, String> {
                let mut value = object(value, path)?;
                let result = Self { $($field: field(&mut value, stringify!($field), path)?,)+ };
                finish(value, path)?;
                Ok(result)
            }
        }
    };
}

macro_rules! string_newtype_json {
    ($ty:path, $getter:ident, $ctor:expr) => {
        impl CanonicalJson for $ty {
            fn to_canonical_json(&self) -> Value {
                Value::String(self.$getter().to_string())
            }

            fn from_canonical_json(value: Value, path: &str) -> Result<Self, String> {
                let value = String::from_canonical_json(value, path)?;
                ($ctor)(value).map_err(|_| format!("{path}: invalid canonical value"))
            }
        }
    };
}

macro_rules! unit_enum_json {
    ($ty:path { $($variant:ident => $value:literal),+ $(,)? }) => {
        impl CanonicalJson for $ty {
            fn to_canonical_json(&self) -> Value {
                Value::String(match self { $(Self::$variant => $value,)+ }.to_string())
            }

            fn from_canonical_json(value: Value, path: &str) -> Result<Self, String> {
                let value = String::from_canonical_json(value, path)?;
                match value.as_str() {
                    $($value => Ok(Self::$variant),)+
                    _ => Err(format!("{path}: invalid enum value `{value}`")),
                }
            }
        }
    };
}

string_newtype_json!(CreatureSourceId, as_str, CreatureSourceId::new);
string_newtype_json!(CreatureTrait, as_str, CreatureTrait::new);
string_newtype_json!(PublicationLicense, as_str, PublicationLicense::new);
string_newtype_json!(SenseType, as_str, SenseType::new);
string_newtype_json!(Language, as_str, Language::new);
string_newtype_json!(PredicateTerm, as_str, PredicateTerm::new);
string_newtype_json!(IwrType, as_str, IwrType::new);
string_newtype_json!(IwrQualifier, as_str, IwrQualifier::new);
string_newtype_json!(CreatureResourceKind, as_str, CreatureResourceKind::new);
string_newtype_json!(CreatureComponentId, as_str, CreatureComponentId::new);
string_newtype_json!(CreatureEntityId, as_str, CreatureEntityId::new);
string_newtype_json!(StableSourceLocator, as_str, StableSourceLocator::new);
string_newtype_json!(CreatureOccurrenceId, as_str, CreatureOccurrenceId::new);
string_newtype_json!(ContentKey, as_str, ContentKey::new);
string_newtype_json!(CreatureStatistic, as_str, CreatureStatistic::new);
string_newtype_json!(CreatureAllianceName, as_str, CreatureAllianceName::new);

impl CanonicalJson for CreatureNote {
    fn to_canonical_json(&self) -> Value {
        Value::String(self.as_str().to_string())
    }
    fn from_canonical_json(value: Value, path: &str) -> Result<Self, String> {
        String::from_canonical_json(value, path).map(CreatureNote::new)
    }
}

impl CanonicalJson for atlas_domain::Rarity {
    fn to_canonical_json(&self) -> Value {
        Value::String(self.as_str().to_string())
    }
    fn from_canonical_json(value: Value, path: &str) -> Result<Self, String> {
        let value = String::from_canonical_json(value, path)?;
        Self::from_canonical(&value).ok_or_else(|| format!("{path}: invalid rarity `{value}`"))
    }
}

unit_enum_json!(CreatureFamily { Npc => "npc" });
unit_enum_json!(CreatureSize { Tiny => "tiny", Small => "small", Medium => "medium", Large => "large", Huge => "huge", Gargantuan => "gargantuan" });
unit_enum_json!(CreatureSkillKind { Acrobatics => "acrobatics", Arcana => "arcana", Athletics => "athletics", Crafting => "crafting", Deception => "deception", Diplomacy => "diplomacy", Intimidation => "intimidation", Medicine => "medicine", Nature => "nature", Occultism => "occultism", Performance => "performance", Religion => "religion", Society => "society", Stealth => "stealth", Survival => "survival", Thievery => "thievery", Lore => "lore" });
unit_enum_json!(CreatureSaveKind { Fortitude => "fortitude", Reflex => "reflex", Will => "will" });
unit_enum_json!(CreatureIwrKind { Immunity => "immunity", Resistance => "resistance", Weakness => "weakness" });
unit_enum_json!(ResourceCurrentPolicy { SerializedValueIsProvenanceOnly => "serialized_value_is_provenance_only" });
unit_enum_json!(ShieldCurrentPolicy { SerializedHitPointsAreProvenanceOnly => "serialized_hit_points_are_provenance_only" });
unit_enum_json!(CreatureSourceField { Identity => "identity", Level => "level", Rarity => "rarity", Traits => "traits", Size => "size", Publication => "publication", Adjustment => "adjustment", SourceAlliance => "source_alliance", Perception => "perception", Initiative => "initiative", Languages => "languages", Skills => "skills", LegacyAbilities => "legacy_abilities", Defenses => "defenses", Movement => "movement", Resources => "resources", EmbeddedEntities => "embedded_entities" });
unit_enum_json!(CreatureDerivation { RecordClassificationProjection => "record_classification_projection", LegacyActorProjection => "legacy_actor_projection", DisplayLabel => "display_label" });
unit_enum_json!(CreatureUnsupportedSourceField { ResourceMaximumDrift => "resource_maximum_drift" });
unit_enum_json!(UnsupportedSourceShape { Missing => "missing", Null => "null", String => "string", Number => "number", Boolean => "boolean", Array => "array", Object => "object" });
unit_enum_json!(UnsupportedSourceReason { OpenVocabulary => "open_vocabulary", AmbiguousLegacyShape => "ambiguous_legacy_shape", InvalidPredicate => "invalid_predicate", NonCanonicalRuntimeValue => "non_canonical_runtime_value", SourceFieldDrift => "source_field_drift" });
unit_enum_json!(CreatureEntityFamily { Strike => "strike", Action => "action", SpellcastingEntry => "spellcasting_entry", Spell => "spell", Equipment => "equipment", Lore => "lore", Affliction => "affliction", Armor => "armor", Backpack => "backpack", Book => "book", Condition => "condition", Consumable => "consumable", Effect => "effect", Shield => "shield", Treasure => "treasure", Weapon => "weapon", Unsupported => "unsupported" });
unit_enum_json!(OccurrenceIdentityStability { StableNestedSourceId => "stable_nested_source_id", UnstableOwnerFamilyOrdinal => "unstable_owner_family_ordinal" });
unit_enum_json!(CreatureDeltaDisposition { ContextualValue => "contextual_value", ExplicitOverride => "explicit_override", ExplicitSuppression => "explicit_suppression", UnsupportedPreserved => "unsupported_preserved" });
unit_enum_json!(CreatureRollKind { Attack => "attack", DifficultyClass => "difficulty_class", Check => "check" });
unit_enum_json!(CreatureEntityRelationshipKind { GrantedBy => "granted_by", ItemGrant => "item_grant", LinkedWeapon => "linked_weapon", PreparedSpell => "prepared_spell" });
unit_enum_json!(CreatureRelationshipExecution { ProvenanceOnly => "provenance_only" });
unit_enum_json!(ActivityRollAbility { Strength => "strength", Dexterity => "dexterity", Constitution => "constitution", Intelligence => "intelligence", Wisdom => "wisdom", Charisma => "charisma" });
unit_enum_json!(ContentIdentityStability { StableSourceIdentity => "stable_source_identity", UnstableAuthoredOrdinal => "unstable_authored_ordinal" });
unit_enum_json!(ContentRole { PrimaryDescription => "primary_description", Summary => "summary", SupplementalRules => "supplemental_rules", EmbeddedCapability => "embedded_capability", JournalPage => "journal_page", TableResult => "table_result", GeneratedNarrative => "generated_narrative", Provenance => "provenance" });
unit_enum_json!(ContentDiagnosticKind { UnsupportedTag => "unsupported_tag", UnsupportedAttribute => "unsupported_attribute", UnknownFoundryMacro => "unknown_foundry_macro", UnresolvedLink => "unresolved_link", UnstableIdentity => "unstable_identity" });
unit_enum_json!(ContentExclusionReason { DeferredEntityFamily => "deferred_entity_family", MissingTypedOwner => "missing_typed_owner" });

struct_json!(CreatureIdentity {
    record_key,
    source_id,
    name,
    family
});
struct_json!(CreatureProvenance {
    source_path,
    source_contract_version,
    source_system_version,
    source_upstream_commit
});
struct_json!(CreaturePublication {
    title,
    remaster,
    license
});
struct_json!(CreatureInitiative { statistic });
struct_json!(CreatureLegacyAbilities {
    strength,
    dexterity,
    constitution,
    intelligence,
    wisdom,
    charisma
});
struct_json!(CreaturePerception {
    modifier,
    details,
    has_vision,
    senses
});
struct_json!(CreatureSense {
    id,
    authored_order,
    sense_type,
    acuity,
    range
});
struct_json!(CreatureLanguages { values, details });
struct_json!(CreatureSkill {
    id,
    authored_order,
    kind,
    label,
    modifier,
    note,
    variants,
    source_item_id
});
struct_json!(CreatureSkillVariant {
    id,
    authored_order,
    modifier,
    label,
    predicate
});
struct_json!(CreatureDefenses {
    armor_class,
    hit_points,
    hardness,
    shield,
    saves,
    all_saves_note,
    immunities,
    resistances,
    weaknesses
});
struct_json!(CreatureShield {
    armor_class_bonus,
    broken_threshold,
    hardness,
    maximum_hit_points,
    serialized_hit_points,
    current_policy
});
struct_json!(CreatureArmorClass { value, details });
struct_json!(CreatureHitPoints {
    value,
    maximum,
    temporary,
    temporary_maximum,
    details
});
struct_json!(CreatureSaves {
    fortitude,
    reflex,
    will
});
struct_json!(CreatureSave {
    id,
    kind,
    value,
    details
});
struct_json!(CreatureIwr {
    id,
    authored_order,
    kind,
    iwr_type,
    value,
    exceptions,
    double_vs,
    apply_once
});
struct_json!(CreatureSpeed {
    id,
    authored_order,
    mode,
    value,
    label,
    details
});
struct_json!(CreatureResource {
    id,
    authored_order,
    kind,
    label,
    maximum,
    serialized_value,
    source_drift,
    current_policy
});
struct_json!(UnsupportedSourceValue {
    shape,
    value,
    reason
});
struct_json!(CreatureUnsupportedSourceFact { field, value });
struct_json!(CreatureEntitySourceIdentity {
    nested_source_id,
    stable_source_locator,
    source_locators
});
struct_json!(CreatureSourceLocator {
    source_path,
    locator,
    precedence
});
struct_json!(CreatureOccurrenceContext {
    group,
    rank,
    location,
    slot,
    uses,
    contextual_label
});
struct_json!(CreatureOccurrenceDelta {
    field_path,
    source_state,
    canonical_value,
    local_value,
    disposition
});
struct_json!(CreatureStrikeCapability {
    traits,
    attack_effects,
    rolls,
    damage,
    action_cost,
    unsupported_notes
});
struct_json!(CreatureActionCapability {
    category,
    traits,
    action_cost,
    frequency,
    self_effect,
    self_effect_label,
    requirements,
    cost,
    rolls,
    damage,
    unsupported_notes
});
struct_json!(CreatureSpellcastingEntryCapability {
    preparation,
    tradition,
    attack,
    dc,
    slots,
    unsupported_notes
});
struct_json!(CreatureSpellSlot {
    rank,
    maximum,
    serialized_value,
    prepared
});
struct_json!(CreatureSpellCapability {
    traits,
    base_rank,
    signature,
    traditions,
    requirements,
    cost,
    counteraction,
    ritual,
    target,
    area,
    range,
    time,
    duration,
    defense,
    damage,
    action_cost,
    unsupported_notes
});
struct_json!(CreatureRitualContext {
    primary_check,
    secondary_casters,
    secondary_checks
});
struct_json!(CreatureSpellArea { area_type, value });
struct_json!(CreatureSpellDuration { value, sustained });
struct_json!(CreatureSpellDefense { save, basic });
struct_json!(CreatureEquipmentCapability {
    traits,
    level,
    usage,
    quantity,
    uses,
    unsupported_notes
});
struct_json!(CreatureLoreCapability {
    modifier,
    unsupported_notes
});
struct_json!(CreatureUnsupportedCapability {
    source_item_type,
    source_slug,
    traits,
    unsupported_notes
});
struct_json!(CreatureFrequency {
    maximum,
    period,
    serialized_value
});
struct_json!(CreatureActorSpellcastingContext {
    rituals_dc,
    unsupported_notes
});
struct_json!(CreatureEntityRelationship {
    source,
    kind,
    target,
    source_path,
    contextual_label,
    lifecycle,
    execution
});
struct_json!(CreatureUseLimit {
    maximum,
    serialized_value
});
struct_json!(CreatureRoll {
    id,
    label,
    kind,
    value,
    ability
});
struct_json!(CreatureDamage {
    id,
    formula,
    damage_type,
    category,
    kinds,
    apply_modifier
});
struct_json!(UnsupportedMechanicNote { source_path, value });
struct_json!(ContentId {
    parent_record_key,
    content_key
});
struct_json!(ContentProvenance {
    source_record_key,
    relative_source_path,
    field_or_pointer_family,
    nested_source_id,
    authored_ordinal_or_range,
    authored_label
});
struct_json!(ContentDiagnostic {
    kind,
    subject,
    detail
});
struct_json!(ContentExclusion {
    parent_record_key,
    content_key,
    relative_source_path,
    label,
    reason
});

macro_rules! unsupported_enum_json {
    ($ty:path { $($variant:ident => $label:literal),+ $(,)? }) => {
        impl CanonicalJson for $ty {
            fn to_canonical_json(&self) -> Value {
                match self {
                    $(Self::$variant => tagged($label, None),)+
                    Self::Unsupported(value) => tagged("unsupported", Some(value.to_canonical_json())),
                }
            }
            fn from_canonical_json(value: Value, path: &str) -> Result<Self, String> {
                let (kind, value) = take_tag(value, path)?;
                match (kind.as_str(), value) {
                    $(($label, None) => Ok(Self::$variant),)+
                    ("unsupported", Some(value)) => UnsupportedSourceValue::from_canonical_json(value, path).map(Self::Unsupported),
                    _ => Err(format!("{path}: invalid enum value `{kind}`")),
                }
            }
        }
    };
}

unsupported_enum_json!(SenseAcuity { Precise => "precise", Imprecise => "imprecise", Vague => "vague" });
unsupported_enum_json!(CreatureMovementMode { Land => "land", Burrow => "burrow", Climb => "climb", Fly => "fly", Swim => "swim" });
unsupported_enum_json!(CreatureSpellPreparation { Prepared => "prepared", Spontaneous => "spontaneous", Focus => "focus", Innate => "innate", Ritual => "ritual" });
unsupported_enum_json!(CreatureAdjustment { Elite => "elite", Weak => "weak" });

macro_rules! named_or_unsupported_json {
    ($ty:path, $named:ty) => {
        impl CanonicalJson for $ty {
            fn to_canonical_json(&self) -> Value {
                match self {
                    Self::Named(value) => tagged("named", Some(value.to_canonical_json())),
                    Self::Unsupported(value) => {
                        tagged("unsupported", Some(value.to_canonical_json()))
                    }
                }
            }

            fn from_canonical_json(value: Value, path: &str) -> Result<Self, String> {
                match take_tag(value, path)? {
                    (kind, Some(value)) if kind == "named" => {
                        <$named>::from_canonical_json(value, path).map(Self::Named)
                    }
                    (kind, Some(value)) if kind == "unsupported" => {
                        UnsupportedSourceValue::from_canonical_json(value, path)
                            .map(Self::Unsupported)
                    }
                    (kind, _) => Err(format!("{path}: invalid named value `{kind}`")),
                }
            }
        }
    };
}

named_or_unsupported_json!(CreatureInitiativeStatistic, CreatureStatistic);
named_or_unsupported_json!(CreatureSourceAlliance, CreatureAllianceName);

impl<T: CanonicalJson> CanonicalJson for CreatureSourceScalar<T> {
    fn to_canonical_json(&self) -> Value {
        match self {
            Self::Value(value) => tagged("value", Some(value.to_canonical_json())),
            Self::Unsupported(value) => tagged("unsupported", Some(value.to_canonical_json())),
        }
    }

    fn from_canonical_json(value: Value, path: &str) -> Result<Self, String> {
        match take_tag(value, path)? {
            (kind, Some(value)) if kind == "value" => {
                T::from_canonical_json(value, path).map(Self::Value)
            }
            (kind, Some(value)) if kind == "unsupported" => {
                UnsupportedSourceValue::from_canonical_json(value, path).map(Self::Unsupported)
            }
            (kind, _) => Err(format!("{path}: invalid source scalar `{kind}`")),
        }
    }
}

impl CanonicalJson for CreaturePreparedSpellSlot {
    fn to_canonical_json(&self) -> Value {
        match self {
            Self::Spell {
                id,
                name,
                expended,
                prepared,
                authored_order,
            } => tagged(
                "spell",
                Some(object_value([
                    ("id", id.to_canonical_json()),
                    ("name", name.to_canonical_json()),
                    ("expended", expended.to_canonical_json()),
                    ("prepared", prepared.to_canonical_json()),
                    ("authored_order", authored_order.to_canonical_json()),
                ])),
            ),
            Self::Unsupported(value) => tagged("unsupported", Some(value.to_canonical_json())),
        }
    }

    fn from_canonical_json(value: Value, path: &str) -> Result<Self, String> {
        match take_tag(value, path)? {
            (kind, Some(value)) if kind == "spell" => {
                let mut value = object(value, path)?;
                let result = Self::Spell {
                    id: field(&mut value, "id", path)?,
                    name: field(&mut value, "name", path)?,
                    expended: field(&mut value, "expended", path)?,
                    prepared: field(&mut value, "prepared", path)?,
                    authored_order: field(&mut value, "authored_order", path)?,
                };
                finish(value, path)?;
                Ok(result)
            }
            (kind, Some(value)) if kind == "unsupported" => {
                UnsupportedSourceValue::from_canonical_json(value, path).map(Self::Unsupported)
            }
            (kind, _) => Err(format!("{path}: invalid prepared spell slot `{kind}`")),
        }
    }
}

impl CanonicalJson for CreatureFactProvenance {
    fn to_canonical_json(&self) -> Value {
        match self {
            Self::Source(value) => tagged("source", Some(value.to_canonical_json())),
            Self::Derived(value) => tagged("derived", Some(value.to_canonical_json())),
        }
    }
    fn from_canonical_json(value: Value, path: &str) -> Result<Self, String> {
        match take_tag(value, path)? {
            (kind, Some(value)) if kind == "source" => {
                CreatureSourceField::from_canonical_json(value, path).map(Self::Source)
            }
            (kind, Some(value)) if kind == "derived" => {
                CreatureDerivation::from_canonical_json(value, path).map(Self::Derived)
            }
            (kind, _) => Err(format!("{path}: invalid fact provenance `{kind}`")),
        }
    }
}

impl CanonicalJson for CreaturePredicate {
    fn to_canonical_json(&self) -> Value {
        match self {
            Self::Term(value) => tagged("term", Some(value.to_canonical_json())),
            Self::Not(value) => tagged("not", Some(value.to_canonical_json())),
            Self::Any(value) => tagged("any", Some(value.to_canonical_json())),
            Self::AtLeast { term, minimum } => tagged(
                "at_least",
                Some(object_value([
                    ("term", term.to_canonical_json()),
                    ("minimum", minimum.to_canonical_json()),
                ])),
            ),
            Self::Unsupported(value) => tagged("unsupported", Some(value.to_canonical_json())),
        }
    }
    fn from_canonical_json(value: Value, path: &str) -> Result<Self, String> {
        match take_tag(value, path)? {
            (kind, Some(value)) if kind == "term" => {
                PredicateTerm::from_canonical_json(value, path).map(Self::Term)
            }
            (kind, Some(value)) if kind == "not" => {
                PredicateTerm::from_canonical_json(value, path).map(Self::Not)
            }
            (kind, Some(value)) if kind == "any" => {
                Vec::from_canonical_json(value, path).map(Self::Any)
            }
            (kind, Some(value)) if kind == "at_least" => {
                let mut value = object(value, path)?;
                let result = Self::AtLeast {
                    term: field(&mut value, "term", path)?,
                    minimum: field(&mut value, "minimum", path)?,
                };
                finish(value, path)?;
                Ok(result)
            }
            (kind, Some(value)) if kind == "unsupported" => {
                UnsupportedSourceValue::from_canonical_json(value, path).map(Self::Unsupported)
            }
            (kind, _) => Err(format!("{path}: invalid creature predicate `{kind}`")),
        }
    }
}

macro_rules! integer_or_unsupported_json {
    ($ty:path) => {
        impl CanonicalJson for $ty {
            fn to_canonical_json(&self) -> Value {
                match self {
                    Self::Integer(value) => tagged("integer", Some(value.to_canonical_json())),
                    Self::Unsupported(value) => {
                        tagged("unsupported", Some(value.to_canonical_json()))
                    }
                }
            }
            fn from_canonical_json(value: Value, path: &str) -> Result<Self, String> {
                match take_tag(value, path)? {
                    (kind, Some(value)) if kind == "integer" => {
                        i64::from_canonical_json(value, path).map(Self::Integer)
                    }
                    (kind, Some(value)) if kind == "unsupported" => {
                        UnsupportedSourceValue::from_canonical_json(value, path)
                            .map(Self::Unsupported)
                    }
                    (kind, _) => Err(format!("{path}: invalid integer value `{kind}`")),
                }
            }
        }
    };
}
integer_or_unsupported_json!(CreatureNumber);
integer_or_unsupported_json!(CreatureResourceAmount);

impl CanonicalJson for CreatureEntityTarget {
    fn to_canonical_json(&self) -> Value {
        match self {
            Self::CanonicalRecord(value) => {
                tagged("canonical_record", Some(value.to_canonical_json()))
            }
            Self::ActorOwned(value) => tagged("actor_owned", Some(value.to_canonical_json())),
        }
    }
    fn from_canonical_json(value: Value, path: &str) -> Result<Self, String> {
        match take_tag(value, path)? {
            (kind, Some(value)) if kind == "canonical_record" => {
                RecordKey::from_canonical_json(value, path).map(Self::CanonicalRecord)
            }
            (kind, Some(value)) if kind == "actor_owned" => {
                CreatureEntityId::from_canonical_json(value, path).map(Self::ActorOwned)
            }
            (kind, _) => Err(format!("{path}: invalid entity target `{kind}`")),
        }
    }
}

impl CanonicalJson for CreatureRelationshipTarget {
    fn to_canonical_json(&self) -> Value {
        match self {
            Self::Occurrence(value) => tagged("occurrence", Some(value.to_canonical_json())),
            Self::UnresolvedNestedSourceId(value) => tagged(
                "unresolved_nested_source_id",
                Some(value.to_canonical_json()),
            ),
        }
    }

    fn from_canonical_json(value: Value, path: &str) -> Result<Self, String> {
        match take_tag(value, path)? {
            (kind, Some(value)) if kind == "occurrence" => {
                CreatureOccurrenceId::from_canonical_json(value, path).map(Self::Occurrence)
            }
            (kind, Some(value)) if kind == "unresolved_nested_source_id" => {
                CreatureSourceId::from_canonical_json(value, path)
                    .map(Self::UnresolvedNestedSourceId)
            }
            (kind, _) => Err(format!("{path}: invalid relationship target `{kind}`")),
        }
    }
}

impl CanonicalJson for CreatureOccurrenceParent {
    fn to_canonical_json(&self) -> Value {
        match self {
            Self::Creature => tagged("creature", None),
            Self::SpellcastingEntry(value) => {
                tagged("spellcasting_entry", Some(value.to_canonical_json()))
            }
        }
    }
    fn from_canonical_json(value: Value, path: &str) -> Result<Self, String> {
        match take_tag(value, path)? {
            (kind, None) if kind == "creature" => Ok(Self::Creature),
            (kind, Some(value)) if kind == "spellcasting_entry" => {
                CreatureOccurrenceId::from_canonical_json(value, path).map(Self::SpellcastingEntry)
            }
            (kind, _) => Err(format!("{path}: invalid occurrence parent `{kind}`")),
        }
    }
}

impl CanonicalJson for CreatureDeltaValue {
    fn to_canonical_json(&self) -> Value {
        match self {
            Self::Text(value) => tagged("text", Some(value.to_canonical_json())),
            Self::Integer(value) => tagged("integer", Some(value.to_canonical_json())),
            Self::Boolean(value) => tagged("boolean", Some(value.to_canonical_json())),
            Self::Uses(value) => tagged("uses", Some(value.to_canonical_json())),
        }
    }
    fn from_canonical_json(value: Value, path: &str) -> Result<Self, String> {
        let (kind, value) = take_tag(value, path)?;
        let value = value.ok_or_else(|| format!("{path}: `{kind}` requires a value"))?;
        match kind.as_str() {
            "text" => String::from_canonical_json(value, path).map(Self::Text),
            "integer" => i64::from_canonical_json(value, path).map(Self::Integer),
            "boolean" => bool::from_canonical_json(value, path).map(Self::Boolean),
            "uses" => CreatureUseLimit::from_canonical_json(value, path).map(Self::Uses),
            _ => Err(format!("{path}: invalid delta value `{kind}`")),
        }
    }
}

impl CanonicalJson for CreatureCapability {
    fn to_canonical_json(&self) -> Value {
        match self {
            Self::Strike(value) => tagged("strike", Some(value.to_canonical_json())),
            Self::Action(value) => tagged("action", Some(value.to_canonical_json())),
            Self::SpellcastingEntry(value) => {
                tagged("spellcasting_entry", Some(value.to_canonical_json()))
            }
            Self::Spell(value) => tagged("spell", Some(value.to_canonical_json())),
            Self::Equipment(value) => tagged("equipment", Some(value.to_canonical_json())),
            Self::Lore(value) => tagged("lore", Some(value.to_canonical_json())),
            Self::Unsupported(value) => tagged("unsupported", Some(value.to_canonical_json())),
        }
    }
    fn from_canonical_json(value: Value, path: &str) -> Result<Self, String> {
        let (kind, value) = take_tag(value, path)?;
        let value = value.ok_or_else(|| format!("{path}: `{kind}` requires a value"))?;
        match kind.as_str() {
            "strike" => {
                CreatureStrikeCapability::from_canonical_json(value, path).map(Self::Strike)
            }
            "action" => {
                CreatureActionCapability::from_canonical_json(value, path).map(Self::Action)
            }
            "spellcasting_entry" => {
                CreatureSpellcastingEntryCapability::from_canonical_json(value, path)
                    .map(Self::SpellcastingEntry)
            }
            "spell" => CreatureSpellCapability::from_canonical_json(value, path).map(Self::Spell),
            "equipment" => {
                CreatureEquipmentCapability::from_canonical_json(value, path).map(Self::Equipment)
            }
            "lore" => CreatureLoreCapability::from_canonical_json(value, path).map(Self::Lore),
            "unsupported" => CreatureUnsupportedCapability::from_canonical_json(value, path)
                .map(Self::Unsupported),
            _ => Err(format!("{path}: invalid creature capability `{kind}`")),
        }
    }
}

impl CanonicalJson for CreatureSpellSave {
    fn to_canonical_json(&self) -> Value {
        match self {
            Self::Fortitude => tagged("fortitude", None),
            Self::Reflex => tagged("reflex", None),
            Self::Will => tagged("will", None),
            Self::Unsupported(value) => tagged("unsupported", Some(value.to_canonical_json())),
        }
    }
    fn from_canonical_json(value: Value, path: &str) -> Result<Self, String> {
        match take_tag(value, path)? {
            (kind, None) if kind == "fortitude" => Ok(Self::Fortitude),
            (kind, None) if kind == "reflex" => Ok(Self::Reflex),
            (kind, None) if kind == "will" => Ok(Self::Will),
            (kind, Some(value)) if kind == "unsupported" => {
                UnsupportedSourceValue::from_canonical_json(value, path).map(Self::Unsupported)
            }
            (kind, _) => Err(format!("{path}: invalid spell save `{kind}`")),
        }
    }
}

impl CanonicalJson for CreatureActionCost {
    fn to_canonical_json(&self) -> Value {
        match self {
            Self::Passive => tagged("passive", None),
            Self::Reaction => tagged("reaction", None),
            Self::FreeAction => tagged("free_action", None),
            Self::Actions(value) => tagged("actions", Some(value.to_canonical_json())),
            Self::Time(value) => tagged("time", Some(value.to_canonical_json())),
            Self::Unsupported(value) => tagged("unsupported", Some(value.to_canonical_json())),
        }
    }
    fn from_canonical_json(value: Value, path: &str) -> Result<Self, String> {
        match take_tag(value, path)? {
            (kind, None) if kind == "passive" => Ok(Self::Passive),
            (kind, None) if kind == "reaction" => Ok(Self::Reaction),
            (kind, None) if kind == "free_action" => Ok(Self::FreeAction),
            (kind, Some(value)) if kind == "actions" => {
                u8::from_canonical_json(value, path).map(Self::Actions)
            }
            (kind, Some(value)) if kind == "time" => {
                String::from_canonical_json(value, path).map(Self::Time)
            }
            (kind, Some(value)) if kind == "unsupported" => {
                UnsupportedSourceValue::from_canonical_json(value, path).map(Self::Unsupported)
            }
            (kind, _) => Err(format!("{path}: invalid action cost `{kind}`")),
        }
    }
}

impl CanonicalJson for CreatureDamageKind {
    fn to_canonical_json(&self) -> Value {
        match self {
            Self::Damage => tagged("damage", None),
            Self::Healing => tagged("healing", None),
            Self::Unsupported(value) => tagged("unsupported", Some(value.to_canonical_json())),
        }
    }
    fn from_canonical_json(value: Value, path: &str) -> Result<Self, String> {
        match take_tag(value, path)? {
            (kind, None) if kind == "damage" => Ok(Self::Damage),
            (kind, None) if kind == "healing" => Ok(Self::Healing),
            (kind, Some(value)) if kind == "unsupported" => {
                UnsupportedSourceValue::from_canonical_json(value, path).map(Self::Unsupported)
            }
            (kind, _) => Err(format!("{path}: invalid damage kind `{kind}`")),
        }
    }
}

impl CanonicalJson for ContentOwner {
    fn to_canonical_json(&self) -> Value {
        match self {
            Self::Record(value) => tagged("record", Some(value.to_canonical_json())),
            Self::CreatureEntity(value) => {
                tagged("creature_entity", Some(value.to_canonical_json()))
            }
            Self::CreatureOccurrence(value) => {
                tagged("creature_occurrence", Some(value.to_canonical_json()))
            }
        }
    }
    fn from_canonical_json(value: Value, path: &str) -> Result<Self, String> {
        let (kind, value) = take_tag(value, path)?;
        let value = value.ok_or_else(|| format!("{path}: `{kind}` requires a value"))?;
        match kind.as_str() {
            "record" => RecordKey::from_canonical_json(value, path).map(Self::Record),
            "creature_entity" => {
                CreatureEntityId::from_canonical_json(value, path).map(Self::CreatureEntity)
            }
            "creature_occurrence" => {
                CreatureOccurrenceId::from_canonical_json(value, path).map(Self::CreatureOccurrence)
            }
            _ => Err(format!("{path}: invalid content owner `{kind}`")),
        }
    }
}

impl CanonicalJson for ContentOrigin {
    fn to_canonical_json(&self) -> Value {
        match self {
            Self::RecordField {
                source_kind,
                relative_source_path,
            } => tagged(
                "record_field",
                Some(object_value([
                    (
                        "source_kind",
                        Value::String(source_kind.as_str().to_string()),
                    ),
                    (
                        "relative_source_path",
                        relative_source_path.to_canonical_json(),
                    ),
                ])),
            ),
            Self::EmbeddedEntityField {
                family,
                nested_source_id,
                relative_source_path,
            } => tagged(
                "embedded_entity_field",
                Some(object_value([
                    ("family", family.to_canonical_json()),
                    ("nested_source_id", nested_source_id.to_canonical_json()),
                    (
                        "relative_source_path",
                        relative_source_path.to_canonical_json(),
                    ),
                ])),
            ),
            Self::Generated { source_kind } => tagged(
                "generated",
                Some(Value::String(source_kind.as_str().to_string())),
            ),
        }
    }
    fn from_canonical_json(value: Value, path: &str) -> Result<Self, String> {
        match take_tag(value, path)? {
            (kind, Some(value)) if kind == "record_field" => {
                let mut value = object(value, path)?;
                let source_kind = parse_content_source_kind(
                    &field::<String>(&mut value, "source_kind", path)?,
                    path,
                )?;
                let relative_source_path = field(&mut value, "relative_source_path", path)?;
                finish(value, path)?;
                Ok(Self::RecordField {
                    source_kind,
                    relative_source_path,
                })
            }
            (kind, Some(value)) if kind == "embedded_entity_field" => {
                let mut value = object(value, path)?;
                let family = field(&mut value, "family", path)?;
                let nested_source_id = field(&mut value, "nested_source_id", path)?;
                let relative_source_path = field(&mut value, "relative_source_path", path)?;
                finish(value, path)?;
                Ok(Self::EmbeddedEntityField {
                    family,
                    nested_source_id,
                    relative_source_path,
                })
            }
            (kind, Some(value)) if kind == "generated" => Ok(Self::Generated {
                source_kind: parse_content_source_kind(
                    &String::from_canonical_json(value, path)?,
                    path,
                )?,
            }),
            (kind, _) => Err(format!("{path}: invalid content origin `{kind}`")),
        }
    }
}

impl CanonicalJson for DuplicateContentStatus {
    fn to_canonical_json(&self) -> Value {
        match self {
            Self::Unique => tagged("unique", None),
            Self::CopiedFromCanonicalTarget { target_record_key } => tagged(
                "copied_from_canonical_target",
                Some(target_record_key.to_canonical_json()),
            ),
        }
    }
    fn from_canonical_json(value: Value, path: &str) -> Result<Self, String> {
        match take_tag(value, path)? {
            (kind, None) if kind == "unique" => Ok(Self::Unique),
            (kind, Some(value)) if kind == "copied_from_canonical_target" => {
                RecordKey::from_canonical_json(value, path)
                    .map(|target_record_key| Self::CopiedFromCanonicalTarget { target_record_key })
            }
            (kind, _) => Err(format!("{path}: invalid duplicate content status `{kind}`")),
        }
    }
}

struct_json!(CreatureEntity {
    id,
    family,
    label,
    source_identity
});
struct_json!(CreatureEntityOccurrence {
    id,
    identity_stability,
    owner,
    target,
    family,
    authored_order,
    source_sort,
    source_folder,
    source_identity,
    parent,
    context,
    capability,
    deltas
});
struct_json!(CreatureEmbeddedEntities {
    entities,
    occurrences,
    relationships,
    actor_spellcasting
});

impl CanonicalJson for OwnedRichContentDocument {
    fn to_canonical_json(&self) -> Value {
        object_value([
            ("id", self.id.to_canonical_json()),
            (
                "identity_stability",
                self.identity_stability.to_canonical_json(),
            ),
            ("owner", self.owner.to_canonical_json()),
            ("role", self.role.to_canonical_json()),
            ("origin", self.origin.to_canonical_json()),
            (
                "visibility",
                Value::String(self.visibility.as_str().to_string()),
            ),
            ("provenance", self.provenance.to_canonical_json()),
            (
                "source_kind",
                Value::String(self.source_kind.as_str().to_string()),
            ),
            ("authored_order", self.authored_order.to_canonical_json()),
            ("label", self.label.to_canonical_json()),
            ("document", self.document.to_canonical_json()),
            (
                "duplicate_status",
                self.duplicate_status.to_canonical_json(),
            ),
            ("diagnostics", self.diagnostics.to_canonical_json()),
        ])
    }
    fn from_canonical_json(value: Value, path: &str) -> Result<Self, String> {
        let mut value = object(value, path)?;
        let id = field(&mut value, "id", path)?;
        let identity_stability = field(&mut value, "identity_stability", path)?;
        let owner = field(&mut value, "owner", path)?;
        let role = field(&mut value, "role", path)?;
        let origin = field(&mut value, "origin", path)?;
        let visibility =
            parse_content_visibility(&field::<String>(&mut value, "visibility", path)?, path)?;
        let provenance = field(&mut value, "provenance", path)?;
        let source_kind =
            parse_content_source_kind(&field::<String>(&mut value, "source_kind", path)?, path)?;
        let authored_order = field(&mut value, "authored_order", path)?;
        let label = field(&mut value, "label", path)?;
        let document = field(&mut value, "document", path)?;
        let duplicate_status = field(&mut value, "duplicate_status", path)?;
        let diagnostics = field(&mut value, "diagnostics", path)?;
        finish(value, path)?;
        Ok(Self::new(
            id,
            identity_stability,
            owner,
            role,
            origin,
            visibility,
            provenance,
            source_kind,
            authored_order,
            label,
            document,
            duplicate_status,
            diagnostics,
        ))
    }
}

struct_json!(OwnedRichContent {
    documents,
    exclusions
});

impl CanonicalJson for CreatureRecord {
    fn to_canonical_json(&self) -> Value {
        object_value([
            ("identity", self.identity.to_canonical_json()),
            ("level", self.level.to_canonical_json()),
            ("rarity", self.rarity.to_canonical_json()),
            ("traits", self.traits.to_canonical_json()),
            ("size", self.size.to_canonical_json()),
            ("publication", self.publication.to_canonical_json()),
            ("adjustment", self.adjustment.to_canonical_json()),
            ("source_alliance", self.source_alliance.to_canonical_json()),
            ("perception", self.perception.to_canonical_json()),
            ("initiative", self.initiative.to_canonical_json()),
            ("languages", self.languages.to_canonical_json()),
            ("skills", self.skills.to_canonical_json()),
            (
                "legacy_abilities",
                self.legacy_abilities.to_canonical_json(),
            ),
            ("defenses", self.defenses.to_canonical_json()),
            ("movement", self.movement.to_canonical_json()),
            ("resources", self.resources.to_canonical_json()),
            (
                "embedded_entities",
                self.embedded_entities.to_canonical_json(),
            ),
            ("content", self.content.to_canonical_json()),
            ("provenance", self.provenance.to_canonical_json()),
        ])
    }
    fn from_canonical_json(value: Value, path: &str) -> Result<Self, String> {
        let mut value = object(value, path)?;
        let result = Self {
            identity: field(&mut value, "identity", path)?,
            level: field(&mut value, "level", path)?,
            rarity: field(&mut value, "rarity", path)?,
            traits: field(&mut value, "traits", path)?,
            size: field(&mut value, "size", path)?,
            publication: field(&mut value, "publication", path)?,
            adjustment: field(&mut value, "adjustment", path)?,
            source_alliance: field(&mut value, "source_alliance", path)?,
            perception: field(&mut value, "perception", path)?,
            initiative: field(&mut value, "initiative", path)?,
            languages: field(&mut value, "languages", path)?,
            skills: field(&mut value, "skills", path)?,
            legacy_abilities: field(&mut value, "legacy_abilities", path)?,
            defenses: field(&mut value, "defenses", path)?,
            movement: field(&mut value, "movement", path)?,
            resources: field(&mut value, "resources", path)?,
            embedded_entities: field(&mut value, "embedded_entities", path)?,
            content: field(&mut value, "content", path)?,
            provenance: field(&mut value, "provenance", path)?,
        };
        finish(value, path)?;
        Ok(result)
    }
}

impl CanonicalJson for RecordBody {
    fn to_canonical_json(&self) -> Value {
        match self {
            Self::Creature(value) => tagged("creature", Some(value.to_canonical_json())),
        }
    }
    fn from_canonical_json(value: Value, path: &str) -> Result<Self, String> {
        match take_tag(value, path)? {
            (kind, Some(value)) if kind == "creature" => {
                CreatureRecord::from_canonical_json(value, path).map(Self::Creature)
            }
            (kind, _) => Err(format!("{path}: invalid record body `{kind}`")),
        }
    }
}

fn parse_content_source_kind(value: &str, path: &str) -> Result<ContentSourceKind, String> {
    ContentSourceKind::from_canonical(value)
        .ok_or_else(|| format!("{path}: invalid content source kind `{value}`"))
}

fn parse_content_visibility(value: &str, path: &str) -> Result<ContentVisibility, String> {
    ContentVisibility::from_canonical(value)
        .ok_or_else(|| format!("{path}: invalid content visibility `{value}`"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn canonical_json_rejects_unknown_fields() {
        let source = r#"{"kind":"value","value":1}"#;
        decode::<FactValue<i64>>(source, "fixture").expect("canonical value");
        assert!(
            decode::<FactValue<i64>>(r#"{"extra":1,"kind":"value","value":1}"#, "fixture").is_err()
        );
    }
}
