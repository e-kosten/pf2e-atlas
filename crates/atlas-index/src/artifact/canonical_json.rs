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

impl<T: CanonicalJson> CanonicalJson for Box<T> {
    fn to_canonical_json(&self) -> Value {
        self.as_ref().to_canonical_json()
    }

    fn from_canonical_json(value: Value, path: &str) -> Result<Self, String> {
        T::from_canonical_json(value, path).map(Box::new)
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

impl<T: CanonicalJson> CanonicalJson for SpellSourceValue<T> {
    fn to_canonical_json(&self) -> Value {
        match self {
            Self::Known(value) => tagged("known", Some(value.to_canonical_json())),
            Self::Unsupported(value) => tagged("unsupported", Some(value.to_canonical_json())),
        }
    }

    fn from_canonical_json(value: Value, path: &str) -> Result<Self, String> {
        match take_tag(value, path)? {
            (kind, Some(value)) if kind == "known" => {
                T::from_canonical_json(value, path).map(Self::Known)
            }
            (kind, Some(value)) if kind == "unsupported" => {
                UnsupportedSourceValue::from_canonical_json(value, path).map(Self::Unsupported)
            }
            (kind, _) => Err(format!("{path}: invalid spell source value `{kind}`")),
        }
    }
}

impl<T: CanonicalJson> CanonicalJson for SpellOrderedMember<T> {
    fn to_canonical_json(&self) -> Value {
        object_value([
            ("key", self.key.to_canonical_json()),
            ("authored_order", self.authored_order.to_canonical_json()),
            ("value", self.value.to_canonical_json()),
        ])
    }

    fn from_canonical_json(value: Value, path: &str) -> Result<Self, String> {
        let mut value = object(value, path)?;
        let result = Self {
            key: field(&mut value, "key", path)?,
            authored_order: field(&mut value, "authored_order", path)?,
            value: field(&mut value, "value", path)?,
        };
        finish(value, path)?;
        Ok(result)
    }
}

impl<T: CanonicalJson> CanonicalJson for SpellKeyedPatch<T> {
    fn to_canonical_json(&self) -> Value {
        object_value([("members", self.members.to_canonical_json())])
    }

    fn from_canonical_json(value: Value, path: &str) -> Result<Self, String> {
        let mut value = object(value, path)?;
        let result = Self {
            members: field(&mut value, "members", path)?,
        };
        finish(value, path)?;
        Ok(result)
    }
}

impl<T: CanonicalJson> CanonicalJson for SpellKeyedPatchMember<T> {
    fn to_canonical_json(&self) -> Value {
        object_value([
            ("key", self.key.to_canonical_json()),
            ("authored_order", self.authored_order.to_canonical_json()),
            ("operation", self.operation.to_canonical_json()),
        ])
    }

    fn from_canonical_json(value: Value, path: &str) -> Result<Self, String> {
        let mut value = object(value, path)?;
        let result = Self {
            key: field(&mut value, "key", path)?,
            authored_order: field(&mut value, "authored_order", path)?,
            operation: field(&mut value, "operation", path)?,
        };
        finish(value, path)?;
        Ok(result)
    }
}

impl<T: CanonicalJson> CanonicalJson for SpellKeyedPatchOperation<T> {
    fn to_canonical_json(&self) -> Value {
        match self {
            Self::Merge(value) => tagged("merge", Some(value.to_canonical_json())),
            Self::Delete => tagged("delete", None),
            Self::Unsupported(value) => tagged("unsupported", Some(value.to_canonical_json())),
        }
    }

    fn from_canonical_json(value: Value, path: &str) -> Result<Self, String> {
        match take_tag(value, path)? {
            (kind, Some(value)) if kind == "merge" => {
                T::from_canonical_json(value, path).map(Self::Merge)
            }
            (kind, None) if kind == "delete" => Ok(Self::Delete),
            (kind, Some(value)) if kind == "unsupported" => {
                UnsupportedSourceValue::from_canonical_json(value, path).map(Self::Unsupported)
            }
            (kind, _) => Err(format!(
                "{path}: invalid spell keyed patch operation `{kind}`"
            )),
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
string_newtype_json!(HazardSourceId, as_str, HazardSourceId::new);
string_newtype_json!(HazardComponentId, as_str, HazardComponentId::new);
string_newtype_json!(HazardEntityId, as_str, HazardEntityId::new);
string_newtype_json!(HazardOccurrenceId, as_str, HazardOccurrenceId::new);
string_newtype_json!(HazardRelationshipId, as_str, HazardRelationshipId::new);
string_newtype_json!(HazardTrait, as_str, HazardTrait::new);
string_newtype_json!(ContentKey, as_str, ContentKey::new);
string_newtype_json!(CreatureStatistic, as_str, CreatureStatistic::new);
string_newtype_json!(CreatureAllianceName, as_str, CreatureAllianceName::new);
string_newtype_json!(SpellSourceId, as_str, SpellSourceId::new);
string_newtype_json!(SpellChildId, as_str, SpellChildId::new);
string_newtype_json!(SpellTrait, as_str, SpellTrait::new);
string_newtype_json!(SpellTradition, as_str, SpellTradition::new);
string_newtype_json!(SpellAreaType, as_str, SpellAreaType::new);
string_newtype_json!(SpellStatistic, as_str, SpellStatistic::new);
string_newtype_json!(SpellOverlayId, as_str, SpellOverlayId::new);
string_newtype_json!(SpellFormId, as_str, SpellFormId::new);

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
unit_enum_json!(CreatureSkillKind { Acrobatics => "acrobatics", Arcana => "arcana", Athletics => "athletics", Crafting => "crafting", Deception => "deception", Diplomacy => "diplomacy", Intimidation => "intimidation", Medicine => "medicine", Nature => "nature", Occultism => "occultism", Performance => "performance", Religion => "religion", Society => "society", Stealth => "stealth", Survival => "survival", Thievery => "thievery", Lore => "lore", Unmodeled => "unmodeled" });
unit_enum_json!(CreatureUnmodeledSkillReason { UnknownAuthoredKey => "unknown_authored_key" });
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
unit_enum_json!(HazardEntityFamily { Action => "action", Strike => "strike", Condition => "condition", Effect => "effect", UnsupportedChild => "unsupported_child" });
unit_enum_json!(HazardExpectedShape { Any => "any", Boolean => "boolean", Integer => "integer", String => "string", StringOrBoolean => "string_or_boolean", StringOrArray => "string_or_array", Array => "array", Object => "object", ClosedVocabulary => "closed_vocabulary", RichDocument => "rich_document" });
unit_enum_json!(HazardSourceShape { Missing => "missing", Null => "null", Boolean => "boolean", Number => "number", String => "string", Array => "array", Object => "object" });
unit_enum_json!(HazardDiagnosticCode { UnexpectedShape => "unexpected_shape", UnsupportedValue => "unsupported_value", LegacyField => "legacy_field", UnsupportedRuleElement => "unsupported_rule_element", InvalidCanonicalValue => "invalid_canonical_value", UnstableIdentity => "unstable_identity" });
unit_enum_json!(HazardSize { Tiny => "tiny", Small => "small", Medium => "medium", Large => "large", Huge => "huge", Gargantuan => "gargantuan" });
unit_enum_json!(HazardComplexity { Simple => "simple", Complex => "complex" });
unit_enum_json!(HazardSaveKind { Fortitude => "fortitude", Reflex => "reflex", Will => "will" });
unit_enum_json!(HazardSourceAttackMode { Melee => "melee", Ranged => "ranged" });
unit_enum_json!(HazardOccurrenceIdentityStability { StableSourceIdentity => "stable_source_identity", UnstableAuthoredOrdinal => "unstable_authored_ordinal" });
unit_enum_json!(HazardActionType { Action => "action", Reaction => "reaction", Free => "free", Passive => "passive" });
unit_enum_json!(HazardActionCount { One => "one", Two => "two", Three => "three" });
unit_enum_json!(HazardActionCategory { Interaction => "interaction", Defensive => "defensive", Offensive => "offensive", Familiar => "familiar" });
unit_enum_json!(HazardFrequencyInterval { Turn => "turn", Round => "round", OneMinute => "PT1M", TenMinutes => "PT10M", OneHour => "PT1H", TwentyFourHours => "PT24H", Day => "day", Week => "P1W", Month => "P1M", Year => "P1Y" });
unit_enum_json!(HazardDamageCategory { Persistent => "persistent", Precision => "precision", Splash => "splash" });
unit_enum_json!(HazardRuleMode { Add => "add", Remove => "remove", Override => "override" });
unit_enum_json!(HazardRelationshipKind { Contains => "contains" });
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
unit_enum_json!(SpellOverlayType { Override => "override" });
unit_enum_json!(SpellUnsupportedSourceField { SystemMember => "system_member", ProvenanceMember => "provenance_member", ClassificationMember => "classification_member", CastingMember => "casting_member", TargetingMember => "targeting_member", DefenseMember => "defense_member", DurationMember => "duration_member", LocationMember => "location_member", LegacyTraitSelection => "legacy_trait_selection", AreaMember => "area_member", DamageMember => "damage_member", HeighteningMember => "heightening_member", OverlayMember => "overlay_member", RitualMember => "ritual_member" });
unit_enum_json!(SpellHeighteningType { Interval => "interval" });
unit_enum_json!(SpellFormField { Classification => "classification", Casting => "casting", Targeting => "targeting", Defense => "defense", Damage => "damage", Duration => "duration", Heightening => "heightening", Rules => "rules" });
unit_enum_json!(SpellFormPatchSource { Base => "base", Overlay => "overlay", FixedHeightening => "fixed_heightening", IntervalHeightening => "interval_heightening" });

impl<T: CanonicalJson> CanonicalJson for HazardSourceValue<T> {
    fn to_canonical_json(&self) -> Value {
        match self {
            Self::Typed(value) => tagged("typed", Some(value.to_canonical_json())),
            Self::Unsupported(value) => tagged("unsupported", Some(value.to_canonical_json())),
        }
    }

    fn from_canonical_json(value: Value, path: &str) -> Result<Self, String> {
        match take_tag(value, path)? {
            (kind, Some(value)) if kind == "typed" => {
                T::from_canonical_json(value, path).map(Self::Typed)
            }
            (kind, Some(value)) if kind == "unsupported" => {
                HazardUnsupportedValue::from_canonical_json(value, path).map(Self::Unsupported)
            }
            (kind, _) => Err(format!("{path}: invalid hazard source support `{kind}`")),
        }
    }
}

impl<T: CanonicalJson> CanonicalJson for HazardFact<T> {
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

impl CanonicalJson for HazardUnsupportedOwner {
    fn to_canonical_json(&self) -> Value {
        match self {
            Self::Record(value) => tagged("record", Some(value.to_canonical_json())),
            Self::Entity(value) => tagged("entity", Some(value.to_canonical_json())),
            Self::Occurrence(value) => tagged("occurrence", Some(value.to_canonical_json())),
        }
    }

    fn from_canonical_json(value: Value, path: &str) -> Result<Self, String> {
        match take_tag(value, path)? {
            (kind, Some(value)) if kind == "record" => {
                RecordKey::from_canonical_json(value, path).map(Self::Record)
            }
            (kind, Some(value)) if kind == "entity" => {
                HazardEntityId::from_canonical_json(value, path).map(Self::Entity)
            }
            (kind, Some(value)) if kind == "occurrence" => {
                HazardOccurrenceId::from_canonical_json(value, path).map(Self::Occurrence)
            }
            (kind, _) => Err(format!("{path}: invalid unsupported owner `{kind}`")),
        }
    }
}

impl CanonicalJson for HazardEmitsSound {
    fn to_canonical_json(&self) -> Value {
        match self {
            Self::Boolean(value) => tagged("boolean", Some(value.to_canonical_json())),
            Self::Named(value) => tagged("named", Some(value.to_canonical_json())),
        }
    }

    fn from_canonical_json(value: Value, path: &str) -> Result<Self, String> {
        match take_tag(value, path)? {
            (kind, Some(value)) if kind == "boolean" => {
                bool::from_canonical_json(value, path).map(Self::Boolean)
            }
            (kind, Some(value)) if kind == "named" => {
                String::from_canonical_json(value, path).map(Self::Named)
            }
            (kind, _) => Err(format!("{path}: invalid emits-sound value `{kind}`")),
        }
    }
}

impl CanonicalJson for HazardEntitySourceIdentity {
    fn to_canonical_json(&self) -> Value {
        match self {
            Self::Stable { source_id } => tagged(
                "stable",
                Some(object_value([("source_id", source_id.to_canonical_json())])),
            ),
            Self::Fallback {
                locator,
                diagnostic,
            } => tagged(
                "fallback",
                Some(object_value([
                    ("locator", locator.to_canonical_json()),
                    ("diagnostic", diagnostic.to_canonical_json()),
                ])),
            ),
        }
    }

    fn from_canonical_json(value: Value, path: &str) -> Result<Self, String> {
        match take_tag(value, path)? {
            (kind, Some(value)) if kind == "stable" => {
                let mut value = object(value, path)?;
                let source_id = field(&mut value, "source_id", path)?;
                finish(value, path)?;
                Ok(Self::Stable { source_id })
            }
            (kind, Some(value)) if kind == "fallback" => {
                let mut value = object(value, path)?;
                let locator = field(&mut value, "locator", path)?;
                let diagnostic = field(&mut value, "diagnostic", path)?;
                finish(value, path)?;
                Ok(Self::Fallback {
                    locator,
                    diagnostic,
                })
            }
            (kind, _) => Err(format!("{path}: invalid hazard source identity `{kind}`")),
        }
    }
}

impl CanonicalJson for HazardCapability {
    fn to_canonical_json(&self) -> Value {
        match self {
            Self::Action(value) => tagged("action", Some(value.to_canonical_json())),
            Self::Strike(value) => tagged("strike", Some(value.to_canonical_json())),
            Self::Condition(value) => tagged("condition", Some(value.to_canonical_json())),
            Self::Effect(value) => tagged("effect", Some(value.to_canonical_json())),
            Self::UnsupportedChild(value) => {
                tagged("unsupported_child", Some(value.to_canonical_json()))
            }
        }
    }

    fn from_canonical_json(value: Value, path: &str) -> Result<Self, String> {
        match take_tag(value, path)? {
            (kind, Some(value)) if kind == "action" => {
                Box::<HazardActionCapability>::from_canonical_json(value, path).map(Self::Action)
            }
            (kind, Some(value)) if kind == "strike" => {
                Box::<HazardStrikeCapability>::from_canonical_json(value, path).map(Self::Strike)
            }
            (kind, Some(value)) if kind == "condition" => {
                Box::<HazardConditionCapability>::from_canonical_json(value, path)
                    .map(Self::Condition)
            }
            (kind, Some(value)) if kind == "effect" => {
                Box::<HazardEffectCapability>::from_canonical_json(value, path).map(Self::Effect)
            }
            (kind, Some(value)) if kind == "unsupported_child" => {
                Box::<HazardUnsupportedChildCapability>::from_canonical_json(value, path)
                    .map(Self::UnsupportedChild)
            }
            (kind, _) => Err(format!("{path}: invalid hazard capability `{kind}`")),
        }
    }
}

impl CanonicalJson for HazardRuleType {
    fn to_canonical_json(&self) -> Value {
        match self {
            Self::Single(value) => tagged("single", Some(value.to_canonical_json())),
            Self::Multiple(value) => tagged("multiple", Some(value.to_canonical_json())),
        }
    }

    fn from_canonical_json(value: Value, path: &str) -> Result<Self, String> {
        match take_tag(value, path)? {
            (kind, Some(value)) if kind == "single" => {
                String::from_canonical_json(value, path).map(Self::Single)
            }
            (kind, Some(value)) if kind == "multiple" => {
                Vec::<String>::from_canonical_json(value, path).map(Self::Multiple)
            }
            (kind, _) => Err(format!("{path}: invalid hazard rule type `{kind}`")),
        }
    }
}

impl CanonicalJson for HazardRuleElement {
    fn to_canonical_json(&self) -> Value {
        match self {
            Self::Immunity(value) => tagged("immunity", Some(value.to_canonical_json())),
            Self::ActiveEffectLike(value) => {
                tagged("active_effect_like", Some(value.to_canonical_json()))
            }
            Self::Aura(value) => tagged("aura", Some(value.to_canonical_json())),
            Self::DamageDice(value) => tagged("damage_dice", Some(value.to_canonical_json())),
            Self::FlatModifier(value) => tagged("flat_modifier", Some(value.to_canonical_json())),
            Self::Note(value) => tagged("note", Some(value.to_canonical_json())),
            Self::Unsupported(value) => tagged("unsupported", Some(value.to_canonical_json())),
        }
    }

    fn from_canonical_json(value: Value, path: &str) -> Result<Self, String> {
        match take_tag(value, path)? {
            (kind, Some(value)) if kind == "immunity" => {
                HazardImmunityRule::from_canonical_json(value, path).map(Self::Immunity)
            }
            (kind, Some(value)) if kind == "active_effect_like" => {
                HazardActiveEffectLikeRule::from_canonical_json(value, path)
                    .map(Self::ActiveEffectLike)
            }
            (kind, Some(value)) if kind == "aura" => {
                HazardAuraRule::from_canonical_json(value, path).map(Self::Aura)
            }
            (kind, Some(value)) if kind == "damage_dice" => {
                HazardDamageDiceRule::from_canonical_json(value, path).map(Self::DamageDice)
            }
            (kind, Some(value)) if kind == "flat_modifier" => {
                HazardFlatModifierRule::from_canonical_json(value, path).map(Self::FlatModifier)
            }
            (kind, Some(value)) if kind == "note" => {
                HazardNoteRule::from_canonical_json(value, path).map(Self::Note)
            }
            (kind, Some(value)) if kind == "unsupported" => {
                HazardUnsupportedRule::from_canonical_json(value, path).map(Self::Unsupported)
            }
            (kind, _) => Err(format!("{path}: invalid hazard rule element `{kind}`")),
        }
    }
}

impl CanonicalJson for HazardUnsupportedField {
    fn to_canonical_json(&self) -> Value {
        match self {
            Self::HazardUnexpected(value) => {
                tagged("hazard_unexpected", Some(value.to_canonical_json()))
            }
            Self::ActionUnexpected(value) => {
                tagged("action_unexpected", Some(value.to_canonical_json()))
            }
            Self::StrikeUnexpected(value) => {
                tagged("strike_unexpected", Some(value.to_canonical_json()))
            }
            Self::ConditionUnexpected(value) => {
                tagged("condition_unexpected", Some(value.to_canonical_json()))
            }
            Self::EffectUnexpected(value) => {
                tagged("effect_unexpected", Some(value.to_canonical_json()))
            }
            Self::UnsupportedChildField(value) => {
                tagged("unsupported_child_field", Some(value.to_canonical_json()))
            }
        }
    }

    fn from_canonical_json(value: Value, path: &str) -> Result<Self, String> {
        match take_tag(value, path)? {
            (kind, Some(value)) if kind == "hazard_unexpected" => {
                String::from_canonical_json(value, path).map(Self::HazardUnexpected)
            }
            (kind, Some(value)) if kind == "action_unexpected" => {
                String::from_canonical_json(value, path).map(Self::ActionUnexpected)
            }
            (kind, Some(value)) if kind == "strike_unexpected" => {
                String::from_canonical_json(value, path).map(Self::StrikeUnexpected)
            }
            (kind, Some(value)) if kind == "condition_unexpected" => {
                String::from_canonical_json(value, path).map(Self::ConditionUnexpected)
            }
            (kind, Some(value)) if kind == "effect_unexpected" => {
                String::from_canonical_json(value, path).map(Self::EffectUnexpected)
            }
            (kind, Some(value)) if kind == "unsupported_child_field" => {
                String::from_canonical_json(value, path).map(Self::UnsupportedChildField)
            }
            (kind, _) => Err(format!("{path}: invalid unsupported hazard field `{kind}`")),
        }
    }
}

impl CanonicalJson for HazardRelationshipTarget {
    fn to_canonical_json(&self) -> Value {
        match self {
            Self::Entity(value) => tagged("entity", Some(value.to_canonical_json())),
            Self::Occurrence(value) => tagged("occurrence", Some(value.to_canonical_json())),
        }
    }

    fn from_canonical_json(value: Value, path: &str) -> Result<Self, String> {
        match take_tag(value, path)? {
            (kind, Some(value)) if kind == "entity" => {
                HazardEntityId::from_canonical_json(value, path).map(Self::Entity)
            }
            (kind, Some(value)) if kind == "occurrence" => {
                HazardOccurrenceId::from_canonical_json(value, path).map(Self::Occurrence)
            }
            (kind, _) => Err(format!(
                "{path}: invalid hazard relationship target `{kind}`"
            )),
        }
    }
}

struct_json!(HazardIdentity {
    record_key,
    source_id,
    name
});
struct_json!(HazardProvenance {
    source_path,
    source_contract_version,
    source_system_version,
    source_upstream_commit,
    source_folder,
    image,
    source_creature_type,
    source_status_effects,
    actor_effects,
    token
});
struct_json!(HazardTokenSourceMetadata { name });
struct_json!(HazardProvenanceValue {
    authored_order,
    exact_json,
    source_shape
});
struct_json!(HazardFactProvenance {
    relative_source_path
});
struct_json!(HazardUnsupportedValue {
    exact_json,
    expected_shape,
    actual_shape,
    relative_source_path,
    owner,
    diagnostic_code
});
struct_json!(HazardPublication {
    title,
    remaster,
    license
});
struct_json!(HazardDetection {
    stealth_modifier,
    details
});
struct_json!(HazardDefenses {
    armor_class,
    hardness,
    hit_points,
    saves,
    immunities,
    weaknesses,
    resistances,
    source_metadata
});
struct_json!(HazardDefenseSourceMetadata { has_health });
struct_json!(HazardHitPoints {
    current,
    maximum,
    temporary,
    details,
    source_metadata
});
struct_json!(HazardHitPointSourceMetadata { temporary_maximum });
struct_json!(HazardSaves {
    fortitude,
    reflex,
    will,
    source_metadata
});
struct_json!(HazardSaveSourceMetadata {
    fortitude_detail,
    reflex_detail,
    will_detail
});
struct_json!(HazardIwr {
    id,
    authored_order,
    iwr_type,
    value,
    exceptions,
    double_vs
});
struct_json!(HazardLifecycle {
    description,
    disable,
    routine,
    reset
});
struct_json!(HazardEmbeddedEntities {
    entities,
    occurrences
});
struct_json!(HazardEntity {
    id,
    family,
    label,
    image,
    source_identity,
    capability
});
struct_json!(HazardEntityOccurrence {
    id,
    owner_record_key,
    entity_id,
    family,
    authored_order,
    source_sort,
    source_folder,
    source_ordinal,
    contextual_label,
    identity_stability
});
struct_json!(HazardItemCommon {
    description,
    publication,
    rules,
    slug,
    traits,
    rarity,
    lineage
});
struct_json!(HazardItemLineage { compendium_source });
struct_json!(HazardImmunityRule {
    authored_order,
    mode,
    immunity_types
});
struct_json!(HazardActiveEffectLikeRule {
    authored_order,
    mode,
    path,
    value
});
struct_json!(HazardAuraRule {
    authored_order,
    radius,
    slug,
    traits
});
struct_json!(HazardDamageDiceRule {
    authored_order,
    critical,
    dice_number,
    die_size,
    damage_type,
    selector
});
struct_json!(HazardFlatModifierRule {
    authored_order,
    critical,
    damage_type,
    selector,
    value
});
struct_json!(HazardNoteRule {
    authored_order,
    outcomes,
    selector,
    text,
    title,
    visibility
});
struct_json!(HazardUnsupportedRule {
    authored_order,
    source
});
struct_json!(HazardActionCapability {
    common,
    action_type,
    actions,
    category,
    death_note,
    frequency,
    self_effect,
    unsupported_fields
});
struct_json!(HazardFrequency {
    value,
    maximum,
    per
});
struct_json!(HazardSelfEffect { target_uuid, label });
struct_json!(HazardStrikeCapability {
    common,
    bonus,
    attack_effects,
    damage_rolls,
    source_metadata,
    unsupported_fields
});
struct_json!(HazardStrikeSourceMetadata {
    attack,
    weapon_type,
    attack_effects_custom
});
struct_json!(HazardStrikeDamage {
    source_key,
    authored_order,
    damage,
    damage_type,
    category
});
struct_json!(HazardConditionCapability {
    common,
    unsupported_fields
});
struct_json!(HazardEffectCapability {
    common,
    unsupported_fields
});
struct_json!(HazardUnsupportedChildCapability {
    child_type,
    common,
    unsupported_fields
});
struct_json!(HazardUnsupportedFact { field, value });
struct_json!(HazardRelationship {
    id,
    authored_order,
    source_occurrence_id,
    kind,
    target
});
struct_json!(HazardRecord {
    identity,
    level,
    rarity,
    traits,
    size,
    publication,
    complexity,
    detection,
    defenses,
    lifecycle,
    emits_sound,
    embedded_entities,
    content,
    relationships,
    unsupported_fields,
    provenance
});

struct_json!(CreatureIdentity {
    record_key,
    source_id,
    name,
    family
});
struct_json!(SpellIdentity {
    record_key,
    source_id,
    name
});
struct_json!(ConsumableSpellLocation {
    value,
    heightened_rank
});
struct_json!(ConsumableSpellSourceContext {
    slug,
    publication_title,
    publication_remaster,
    rarity
});
struct_json!(SpellSourceContext {
    image,
    publication_license,
    consumable_child
});
struct_json!(ConsumableSpellChild {
    parent_record_key,
    child_id,
    name,
    authored_order,
    location,
    standalone_locator,
    standalone_target,
    definition
});
struct_json!(SpellClassification {
    rank,
    traits,
    traditions
});
struct_json!(SpellCasting {
    time,
    cost,
    requirements,
    counteraction
});
struct_json!(SpellTargeting {
    target,
    range,
    area
});
struct_json!(SpellAreaValue {
    value,
    area_type,
    legacy_area_type,
    details,
    unsupported_notes
});
struct_json!(SpellLegacyAreaType {
    source_path,
    authored_key,
    authored_order,
    value
});
struct_json!(SpellDefenseValue { passive, save });
struct_json!(SpellSave { statistic, basic });
struct_json!(SpellDamage {
    apply_mod,
    category,
    formula,
    kinds,
    materials,
    damage_type
});
struct_json!(SpellDuration { value, sustained });
struct_json!(SpellIntervalHeightening {
    interval,
    area,
    damage
});
struct_json!(SpellFixedHeighteningLayer {
    key,
    authored_order,
    rank,
    patch
});
struct_json!(SpellOverlay {
    key,
    authored_order,
    overlay_id,
    source_id,
    sort,
    name,
    overlay_type,
    patch
});
struct_json!(SpellPatch {
    classification,
    casting,
    targeting,
    defense,
    damage,
    duration,
    heightening,
    rules,
    unsupported
});
struct_json!(SpellClassificationPatch {
    rank,
    traits,
    traditions
});
struct_json!(SpellCastingPatch {
    time,
    cost,
    requirements,
    counteraction
});
struct_json!(SpellTargetingPatch {
    target,
    range,
    area
});
struct_json!(SpellAreaPatch {
    value,
    area_type,
    legacy_area_type,
    details,
    unsupported_notes
});
struct_json!(SpellDefensePatch { passive, save });
struct_json!(SpellSavePatch { statistic, basic });
struct_json!(SpellDamagePatch {
    apply_mod,
    category,
    formula,
    kinds,
    materials,
    damage_type
});
struct_json!(SpellDurationPatch { value, sustained });
struct_json!(SpellHeighteningPatch {
    kind,
    interval,
    area,
    interval_damage
});
struct_json!(SpellTextPatch { value });
struct_json!(SpellRitual {
    primary_check,
    secondary_casters,
    secondary_checks
});
struct_json!(SpellRuleElement {
    authored_order,
    source_path,
    authored_key,
    authored_object_json,
    rule
});
struct_json!(SpellDamageDiceRule {
    selector,
    predicate,
    dice_number,
    die_size,
    damage_type,
    hide_if_disabled
});
struct_json!(SpellEphemeralEffectRule {
    predicate,
    selectors,
    uuid
});
struct_json!(SpellDamageAlterationRule {
    mode,
    predicate,
    property,
    selectors,
    slug,
    value
});
struct_json!(SpellRollOptionRule {
    domain,
    label,
    option,
    placement,
    predicate,
    suboptions,
    toggleable
});
struct_json!(SpellItemAlterationRule {
    item_id,
    mode,
    predicate,
    property,
    value
});
struct_json!(SpellRuleSuboption { label, value });
struct_json!(SpellUnsupportedRule {
    authored_key,
    source_path,
    value
});
struct_json!(SpellUnsupportedRulePredicate {
    source_path,
    authored_key,
    authored_order,
    value
});
struct_json!(SpellUnsupportedSourceFact {
    field,
    source_path,
    authored_key,
    authored_order,
    value
});
struct_json!(SpellUnsupportedPatchField {
    field,
    source_path,
    authored_key,
    authored_order,
    value
});
struct_json!(SpellProvenance {
    source_path,
    source_contract_version,
    source_system_version,
    source_upstream_commit,
    standalone_location
});
struct_json!(SpellDefinition {
    source_context,
    classification,
    casting,
    targeting,
    defense,
    damage,
    duration,
    heightening,
    overlays,
    ritual,
    rules,
    content,
    unsupported_notes,
    provenance
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
    source_entries,
    kind,
    label,
    modifier,
    note,
    variants,
    source_item_id,
    unmodeled
});
struct_json!(CreatureSkillSourceEntry {
    authored_key,
    modifier
});
struct_json!(CreatureUnmodeledSkill {
    authored_key,
    base,
    reason
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
            Self::HazardEntity(value) => tagged("hazard_entity", Some(value.to_canonical_json())),
            Self::HazardOccurrence(value) => {
                tagged("hazard_occurrence", Some(value.to_canonical_json()))
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
            "hazard_entity" => {
                HazardEntityId::from_canonical_json(value, path).map(Self::HazardEntity)
            }
            "hazard_occurrence" => {
                HazardOccurrenceId::from_canonical_json(value, path).map(Self::HazardOccurrence)
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
            Self::HazardEmbeddedEntityField {
                family,
                nested_source_id,
                relative_source_path,
            } => tagged(
                "hazard_embedded_entity_field",
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
            (kind, Some(value)) if kind == "hazard_embedded_entity_field" => {
                let mut value = object(value, path)?;
                let family = field(&mut value, "family", path)?;
                let nested_source_id = field(&mut value, "nested_source_id", path)?;
                let relative_source_path = field(&mut value, "relative_source_path", path)?;
                finish(value, path)?;
                Ok(Self::HazardEmbeddedEntityField {
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

impl CanonicalJson for SpellRangeValue {
    fn to_canonical_json(&self) -> Value {
        object_value([("authored_text", self.authored_text.to_canonical_json())])
    }

    fn from_canonical_json(value: Value, path: &str) -> Result<Self, String> {
        let mut value = object(value, path)?;
        let authored_text: String = field(&mut value, "authored_text", path)?;
        finish(value, path)?;
        Ok(Self::from_authored_text(authored_text))
    }
}

impl CanonicalJson for SpellStandaloneTarget {
    fn to_canonical_json(&self) -> Value {
        match self {
            Self::Resolved(value) => tagged("resolved", Some(value.to_canonical_json())),
            Self::Unresolved(value) => tagged("unresolved", Some(value.to_canonical_json())),
        }
    }

    fn from_canonical_json(value: Value, path: &str) -> Result<Self, String> {
        match take_tag(value, path)? {
            (kind, Some(value)) if kind == "resolved" => {
                RecordKey::from_canonical_json(value, path).map(Self::Resolved)
            }
            (kind, Some(value)) if kind == "unresolved" => {
                UnsupportedSourceValue::from_canonical_json(value, path).map(Self::Unresolved)
            }
            (kind, _) => Err(format!("{path}: invalid standalone spell target `{kind}`")),
        }
    }
}

impl CanonicalJson for SpellRulePredicate {
    fn to_canonical_json(&self) -> Value {
        match self {
            Self::Term(value) => tagged("term", Some(value.to_canonical_json())),
            Self::Or(value) => tagged("or", Some(value.to_canonical_json())),
            Self::Unsupported(value) => tagged("unsupported", Some(value.to_canonical_json())),
        }
    }

    fn from_canonical_json(value: Value, path: &str) -> Result<Self, String> {
        match take_tag(value, path)? {
            (kind, Some(value)) if kind == "term" => {
                String::from_canonical_json(value, path).map(Self::Term)
            }
            (kind, Some(value)) if kind == "or" => {
                Vec::<String>::from_canonical_json(value, path).map(Self::Or)
            }
            (kind, Some(value)) if kind == "unsupported" => {
                SpellUnsupportedRulePredicate::from_canonical_json(value, path)
                    .map(Self::Unsupported)
            }
            (kind, _) => Err(format!("{path}: invalid spell rule predicate `{kind}`")),
        }
    }
}

impl CanonicalJson for SpellRule {
    fn to_canonical_json(&self) -> Value {
        match self {
            Self::DamageDice(value) => tagged("damage_dice", Some(value.to_canonical_json())),
            Self::EphemeralEffect(value) => {
                tagged("ephemeral_effect", Some(value.to_canonical_json()))
            }
            Self::DamageAlteration(value) => {
                tagged("damage_alteration", Some(value.to_canonical_json()))
            }
            Self::RollOption(value) => tagged("roll_option", Some(value.to_canonical_json())),
            Self::ItemAlteration(value) => {
                tagged("item_alteration", Some(value.to_canonical_json()))
            }
            Self::Unsupported(value) => tagged("unsupported", Some(value.to_canonical_json())),
        }
    }

    fn from_canonical_json(value: Value, path: &str) -> Result<Self, String> {
        match take_tag(value, path)? {
            (kind, Some(value)) if kind == "damage_dice" => {
                SpellDamageDiceRule::from_canonical_json(value, path).map(Self::DamageDice)
            }
            (kind, Some(value)) if kind == "ephemeral_effect" => {
                SpellEphemeralEffectRule::from_canonical_json(value, path)
                    .map(Self::EphemeralEffect)
            }
            (kind, Some(value)) if kind == "damage_alteration" => {
                SpellDamageAlterationRule::from_canonical_json(value, path)
                    .map(Self::DamageAlteration)
            }
            (kind, Some(value)) if kind == "roll_option" => {
                SpellRollOptionRule::from_canonical_json(value, path).map(Self::RollOption)
            }
            (kind, Some(value)) if kind == "item_alteration" => {
                SpellItemAlterationRule::from_canonical_json(value, path).map(Self::ItemAlteration)
            }
            (kind, Some(value)) if kind == "unsupported" => {
                SpellUnsupportedRule::from_canonical_json(value, path).map(Self::Unsupported)
            }
            (kind, _) => Err(format!("{path}: invalid spell rule `{kind}`")),
        }
    }
}

impl CanonicalJson for SpellHeightening {
    fn to_canonical_json(&self) -> Value {
        match self {
            Self::Interval(value) => tagged("interval", Some(value.to_canonical_json())),
            Self::Fixed(value) => tagged("fixed", Some(value.to_canonical_json())),
        }
    }

    fn from_canonical_json(value: Value, path: &str) -> Result<Self, String> {
        match take_tag(value, path)? {
            (kind, Some(value)) if kind == "interval" => {
                SpellIntervalHeightening::from_canonical_json(value, path).map(Self::Interval)
            }
            (kind, Some(value)) if kind == "fixed" => {
                Vec::<SpellFixedHeighteningLayer>::from_canonical_json(value, path).map(Self::Fixed)
            }
            (kind, _) => Err(format!("{path}: invalid spell heightening `{kind}`")),
        }
    }
}

impl CanonicalJson for SpellRecord {
    fn to_canonical_json(&self) -> Value {
        object_value([
            ("identity", self.identity.to_canonical_json()),
            ("definition", self.definition.to_canonical_json()),
        ])
    }

    fn from_canonical_json(value: Value, path: &str) -> Result<Self, String> {
        let mut value = object(value, path)?;
        let result = Self {
            identity: field(&mut value, "identity", path)?,
            definition: field(&mut value, "definition", path)?,
        };
        finish(value, path)?;
        Ok(result)
    }
}

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
            Self::Hazard(value) => tagged("hazard", Some(value.to_canonical_json())),
            Self::Spell(value) => tagged("spell", Some(value.to_canonical_json())),
        }
    }
    fn from_canonical_json(value: Value, path: &str) -> Result<Self, String> {
        match take_tag(value, path)? {
            (kind, Some(value)) if kind == "creature" => {
                CreatureRecord::from_canonical_json(value, path).map(Self::Creature)
            }
            (kind, Some(value)) if kind == "hazard" => {
                HazardRecord::from_canonical_json(value, path).map(Self::Hazard)
            }
            (kind, Some(value)) if kind == "spell" => {
                SpellRecord::from_canonical_json(value, path).map(Self::Spell)
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

    fn known<T>(value: T) -> SpellFact<T> {
        FactValue::Value(SpellSourceValue::Known(value))
    }

    fn spell_fixture() -> RecordBody {
        let record_key = RecordKey::parse("spells-srd:rfZpqmj0AIIdkVIs").expect("record key");
        let overlay_key = "7qUa78M9vP8T3Z6S";
        RecordBody::Spell(SpellRecord {
            identity: SpellIdentity {
                record_key: record_key.clone(),
                source_id: SpellSourceId::new("rfZpqmj0AIIdkVIs").expect("source id"),
                name: "Heal".to_string(),
            },
            definition: SpellDefinition {
                source_context: SpellSourceContext {
                    image: known("icons/magic/life/cross-worn-green.webp".to_string()),
                    publication_license: known(PublicationLicense::new("ORC").expect("license")),
                    consumable_child: FactValue::Missing,
                },
                classification: known(SpellClassification {
                    rank: known(1),
                    traits: known(vec![SpellTrait::new("healing").expect("trait")]),
                    traditions: known(vec![SpellTradition::new("divine").expect("tradition")]),
                }),
                casting: known(SpellCasting {
                    time: known("1 to 3".to_string()),
                    cost: FactValue::Value(SpellSourceValue::Known(String::new())),
                    requirements: FactValue::Missing,
                    counteraction: known(false),
                }),
                targeting: known(SpellTargeting {
                    target: known("1 willing living creature".to_string()),
                    range: known(SpellRangeValue::from_authored_text("varies")),
                    area: FactValue::Null,
                }),
                defense: known(SpellDefenseValue {
                    passive: FactValue::Missing,
                    save: known(SpellSave {
                        statistic: known(SpellStatistic::new("fortitude").expect("statistic")),
                        basic: known(true),
                    }),
                }),
                damage: known(vec![SpellOrderedMember {
                    key: "0".to_string(),
                    authored_order: 0,
                    value: SpellDamage {
                        formula: known("1d8".to_string()),
                        category: FactValue::Null,
                        ..SpellDamage::default()
                    },
                }]),
                duration: known(SpellDuration {
                    value: known(String::new()),
                    sustained: known(false),
                }),
                heightening: known(SpellHeightening::Interval(SpellIntervalHeightening {
                    interval: known(1),
                    area: FactValue::Null,
                    damage: known(vec![SpellOrderedMember {
                        key: "0".to_string(),
                        authored_order: 0,
                        value: "1d8".to_string(),
                    }]),
                })),
                overlays: known(vec![SpellOverlay {
                    key: overlay_key.to_string(),
                    authored_order: 1,
                    overlay_id: SpellOverlayId::new(overlay_key).expect("overlay id"),
                    source_id: known(SpellOverlayId::new(overlay_key).expect("source id")),
                    sort: known(1),
                    name: FactValue::Missing,
                    overlay_type: known(SpellOverlayType::Override),
                    patch: SpellPatch {
                        defense: FactValue::Null,
                        targeting: known(SpellTargetingPatch {
                            range: known(SpellRangeValue::from_authored_text("touch")),
                            ..SpellTargetingPatch::default()
                        }),
                        ..SpellPatch::default()
                    },
                }]),
                ritual: FactValue::Null,
                rules: known(vec![SpellRuleElement {
                    authored_order: 0,
                    source_path: "system.rules.0".to_string(),
                    authored_key: "DamageDice".to_string(),
                    authored_object_json: r#"{"damageType":"vitality","diceNumber":"@spell.rank","dieSize":"d4","key":"DamageDice","predicate":["target:negative-healing"],"selector":"{item|_id}-damage"}"#.to_string(),
                    rule: SpellRule::DamageDice(SpellDamageDiceRule {
                        selector: known("{item|_id}-damage".to_string()),
                        predicate: known(vec![SpellRulePredicate::Term(
                            "target:negative-healing".to_string(),
                        )]),
                        dice_number: known("@spell.rank".to_string()),
                        die_size: known("d4".to_string()),
                        damage_type: known("vitality".to_string()),
                        hide_if_disabled: FactValue::Missing,
                    }),
                }]),
                content: OwnedRichContent::default(),
                unsupported_notes: Vec::new(),
                provenance: SpellProvenance {
                    source_path: "packs/spells/1st-rank/heal.json".to_string(),
                    source_contract_version: "pf2e-serialized-source/v1".to_string(),
                    source_system_version: "6.12.4".to_string(),
                    source_upstream_commit: "4cbdaa37d6c33e9519561bae2c59a23e0288cbce".to_string(),
                    standalone_location: FactValue::Missing,
                },
            },
        })
    }

    #[test]
    fn canonical_json_rejects_unknown_fields() {
        let source = r#"{"kind":"value","value":1}"#;
        decode::<FactValue<i64>>(source, "fixture").expect("canonical value");
        assert!(
            decode::<FactValue<i64>>(r#"{"extra":1,"kind":"value","value":1}"#, "fixture").is_err()
        );
    }

    #[test]
    fn hazard_rule_codec_preserves_typed_and_unsupported_members() {
        let owner =
            HazardUnsupportedOwner::Entity(HazardEntityId::new("rule-action").expect("entity id"));
        let rules = vec![
            HazardRuleElement::Aura(HazardAuraRule {
                authored_order: 0,
                radius: hazard_value(5, "/items/0/system/rules/0/radius"),
                slug: hazard_value("emit-cold".to_string(), "/items/0/system/rules/0/slug"),
                traits: hazard_value(
                    vec![HazardTrait::new("cold").expect("trait")],
                    "/items/0/system/rules/0/traits",
                ),
            }),
            HazardRuleElement::Unsupported(HazardUnsupportedRule {
                authored_order: 1,
                source: HazardUnsupportedValue {
                    exact_json: r#"{"key":"FutureRule","value":7}"#.to_string(),
                    expected_shape: HazardExpectedShape::Object,
                    actual_shape: HazardSourceShape::Object,
                    relative_source_path: "/items/0/system/rules/1".to_string(),
                    owner,
                    diagnostic_code: HazardDiagnosticCode::UnsupportedRuleElement,
                },
            }),
        ];
        let encoded = encode(&rules).expect("rules encode");
        let decoded = decode::<Vec<HazardRuleElement>>(&encoded, "rules").expect("rules decode");
        assert_eq!(decoded, rules);
    }

    #[test]
    fn hazard_source_attack_mode_codec_is_closed_and_round_trips() {
        for mode in [
            HazardSourceAttackMode::Melee,
            HazardSourceAttackMode::Ranged,
        ] {
            let encoded = encode(&mode).expect("attack mode encode");
            assert_eq!(
                decode::<HazardSourceAttackMode>(&encoded, "attack mode")
                    .expect("attack mode decode"),
                mode
            );
        }
        assert!(decode::<HazardSourceAttackMode>("\"future\"", "attack mode").is_err());
    }

    #[test]
    fn hazard_canonical_json_validates_ids_slugs_and_recomputes_content_derivations() {
        let body = RecordBody::Hazard(codec_hazard_fixture());
        let encoded = encode(&body).expect("hazard body encodes");
        assert!(!encoded.contains("codecVersion"));
        assert!(!encoded.contains("codec_version"));
        assert!(!encoded.contains("content_hash"));
        assert!(!encoded.contains("reference_occurrences"));

        let decoded = decode::<RecordBody>(&encoded, "body").expect("hazard body decodes");
        assert_eq!(decoded, body);

        let mut invalid_id: Value = serde_json::from_str(&encoded).expect("canonical json");
        *invalid_id
            .pointer_mut("/value/identity/source_id")
            .expect("source id") = Value::String("invalid id".to_string());
        assert!(decode::<RecordBody>(&invalid_id.to_string(), "body").is_err());

        let mut invalid_trait: Value = serde_json::from_str(&encoded).expect("canonical json");
        *invalid_trait
            .pointer_mut("/value/traits/value/value/value/0")
            .expect("trait") = Value::String("invalid trait".to_string());
        assert!(decode::<RecordBody>(&invalid_trait.to_string(), "body").is_err());

        let mut injected_derived: Value = serde_json::from_str(&encoded).expect("canonical json");
        let content = injected_derived
            .pointer_mut("/value/content/documents/0")
            .and_then(Value::as_object_mut)
            .expect("content document");
        content.insert(
            "content_hash".to_string(),
            Value::String("stale".to_string()),
        );
        content.insert(
            "reference_occurrences".to_string(),
            Value::Array(Vec::new()),
        );
        assert!(decode::<RecordBody>(&injected_derived.to_string(), "body").is_err());

        let mut changed_document: Value = serde_json::from_str(&encoded).expect("canonical json");
        *changed_document
            .pointer_mut("/value/content/documents/0/document")
            .expect("content document") =
            serde_json::to_value(RichDocument::new(vec![RichNode::Text {
                text: "Changed".to_string(),
            }]))
            .expect("rich document");
        let decoded = decode::<RecordBody>(&changed_document.to_string(), "body")
            .expect("changed document decodes");
        let RecordBody::Hazard(decoded) = decoded else {
            panic!("hazard body")
        };
        let document = &decoded.content.documents[0];
        assert_eq!(
            document.content_hash,
            ContentHash::for_document(&document.document)
        );
        assert!(document.reference_occurrences.is_empty());
    }

    fn codec_hazard_fixture() -> HazardRecord {
        let record_key = RecordKey::new(
            atlas_domain::PackName::new("hazards").expect("pack"),
            atlas_domain::RecordId::new("BHq5wpQU8hQEke8D").expect("record id"),
        );
        let document = RichDocument::new(vec![RichNode::FoundryLink {
            link: FoundryLink {
                target: RichLinkTarget::Unresolved {
                    target: "Compendium.pf2e.actionspf2e.Item.Grab an Edge".to_string(),
                    fallback_label: "Grab an Edge".to_string(),
                },
                label: None,
                source: FoundryLinkSource {
                    macro_kind: FoundryLinkMacroKind::Uuid,
                    authored_target: "Compendium.pf2e.actionspf2e.Item.Grab an Edge".to_string(),
                    relation: None,
                },
                behavior: FoundryLinkBehavior::Reference,
            },
        }]);
        let content = OwnedRichContent {
            documents: vec![OwnedRichContentDocument::new(
                ContentId::new(
                    record_key.clone(),
                    ContentKey::new("pitfall-description").expect("content key"),
                ),
                ContentIdentityStability::StableSourceIdentity,
                ContentOwner::Record(record_key.clone()),
                ContentRole::EmbeddedCapability,
                ContentOrigin::RecordField {
                    source_kind: ContentSourceKind::Description,
                    relative_source_path: "/system/details/description".to_string(),
                },
                ContentVisibility::Public,
                ContentProvenance {
                    source_record_key: record_key.clone(),
                    relative_source_path: "/system/details/description".to_string(),
                    field_or_pointer_family: "hazard.description".to_string(),
                    nested_source_id: None,
                    authored_ordinal_or_range: None,
                    authored_label: None,
                },
                ContentSourceKind::Description,
                0,
                None,
                document,
                DuplicateContentStatus::Unique,
                Vec::new(),
            )],
            exclusions: Vec::new(),
        };

        HazardRecord {
            identity: HazardIdentity {
                record_key,
                source_id: HazardSourceId::new("BHq5wpQU8hQEke8D").expect("source id"),
                name: "Hidden Pit".to_string(),
            },
            level: missing_hazard_fact("/system/details/level/value"),
            rarity: missing_hazard_fact("/system/traits/rarity"),
            traits: HazardFact::source(
                FactValue::Value(HazardSourceValue::Typed(vec![
                    HazardTrait::new("mechanical").expect("trait"),
                ])),
                "/system/traits/value",
            ),
            size: missing_hazard_fact("/system/traits/size/value"),
            publication: missing_hazard_fact("/system/details/publication"),
            complexity: missing_hazard_fact("/system/details/isComplex"),
            detection: missing_hazard_fact("/system/attributes/stealth"),
            defenses: missing_hazard_fact("/system/attributes"),
            lifecycle: missing_hazard_fact("/system/details"),
            emits_sound: missing_hazard_fact("/system/attributes/emitsSound"),
            embedded_entities: missing_hazard_fact("/items"),
            content,
            relationships: Vec::new(),
            unsupported_fields: Vec::new(),
            provenance: HazardProvenance {
                source_path: "packs/hazards/hidden-pit.json".to_string(),
                source_contract_version: "pf2e-serialized-source/v1".to_string(),
                source_system_version: "6.12.4".to_string(),
                source_upstream_commit: "4cbdaa37d6c33e9519561bae2c59a23e0288cbce".to_string(),
                source_folder: missing_hazard_fact("/folder"),
                image: missing_hazard_fact("/img"),
                source_creature_type: missing_hazard_fact("/system/creatureType"),
                source_status_effects: missing_hazard_fact("/system/statusEffects"),
                actor_effects: missing_hazard_fact("/effects"),
                token: missing_hazard_fact("/prototypeToken"),
            },
        }
    }

    fn missing_hazard_fact<T>(path: &str) -> HazardFact<T> {
        HazardFact::source(FactValue::Missing, path)
    }

    fn hazard_value<T>(value: T, path: &str) -> HazardFact<T> {
        HazardFact::source(
            FactValue::Value(HazardSourceValue::Typed(value)),
            path.to_string(),
        )
    }

    #[test]
    fn spell_body_codec_preserves_presence_key_order_and_opaque_form_identity() {
        let fixture = spell_fixture();
        let encoded = encode(&fixture).expect("encode spell body");
        let decoded = decode::<RecordBody>(&encoded, "spell_fixture").expect("decode spell body");
        assert_eq!(decoded, fixture);

        let spell = decoded.as_spell().expect("spell body");
        assert_eq!(
            spell.definition.provenance.standalone_location,
            FactValue::Missing
        );
        assert!(encoded.contains(r#""standalone_location":{"kind":"missing"}"#));
        assert_eq!(
            spell
                .definition
                .casting
                .as_value()
                .and_then(SpellSourceValue::as_known)
                .map(|casting| &casting.requirements),
            Some(&FactValue::Missing)
        );
        let overlays = spell
            .definition
            .overlays
            .as_value()
            .and_then(SpellSourceValue::as_known)
            .expect("overlays");
        assert_eq!(overlays[0].key, "7qUa78M9vP8T3Z6S");
        assert_eq!(overlays[0].authored_order, 1);
        let form_id = overlays[0].form_id(&spell.identity.record_key);
        assert!(form_id.as_str().starts_with("spell-form:"));
        assert!(!form_id.as_str().contains("spells-srd"));
        assert!(!form_id.as_str().contains(&overlays[0].key));
        assert_eq!(overlays[0].patch.defense, FactValue::Null);
        let rules = spell
            .definition
            .rules
            .as_value()
            .and_then(SpellSourceValue::as_known)
            .expect("rules");
        assert_eq!(rules[0].authored_order, 0);
        assert_eq!(rules[0].source_path, "system.rules.0");
        assert_eq!(rules[0].authored_key, "DamageDice");
        assert!(rules[0].authored_object_json.contains("@spell.rank"));
    }

    #[test]
    fn spell_rule_codec_preserves_source_backed_variants_order_and_multiplicity() {
        let phase_raw = r#"{"key":"EphemeralEffect","predicate":["item:slug:phase-bolt"],"selectors":["spell-attack-roll"],"uuid":"Compendium.pf2e.spell-effects.Item.Spell Effect: Phase Bolt"}"#;
        let phase_rules = vec![SpellRuleElement {
            authored_order: 0,
            source_path: "system.rules.0".to_string(),
            authored_key: "EphemeralEffect".to_string(),
            authored_object_json: phase_raw.to_string(),
            rule: SpellRule::EphemeralEffect(SpellEphemeralEffectRule {
                predicate: known(vec![SpellRulePredicate::Term(
                    "item:slug:phase-bolt".to_string(),
                )]),
                selectors: known(vec!["spell-attack-roll".to_string()]),
                uuid: known(
                    "Compendium.pf2e.spell-effects.Item.Spell Effect: Phase Bolt".to_string(),
                ),
            }),
        }];
        let phase_encoded = encode(&phase_rules).expect("encode Phase Bolt rules");
        let phase_decoded = decode::<Vec<SpellRuleElement>>(&phase_encoded, "phase_bolt_rules")
            .expect("decode Phase Bolt rules");
        assert_eq!(phase_decoded, phase_rules);
        assert_eq!(phase_decoded[0].authored_object_json, phase_raw);
        let SpellRule::EphemeralEffect(phase) = &phase_decoded[0].rule else {
            panic!("Phase Bolt ephemeral effect rule")
        };
        assert_eq!(
            phase
                .uuid
                .as_value()
                .and_then(SpellSourceValue::as_known)
                .map(String::as_str),
            Some("Compendium.pf2e.spell-effects.Item.Spell Effect: Phase Bolt")
        );

        let qi_raw = [
            r#"{"domain":"all","key":"RollOption","label":"PF2E.SpecificRule.Monk.QiSpells.HeavensThunder.RollOptionLabel","option":"heavens-thunder","placement":"spellcasting","predicate":["self:effect:heavens-thunder"],"suboptions":[{"label":"PF2E.TraitElectricity","value":"electricity"},{"label":"PF2E.TraitSonic","value":"sonic"}],"toggleable":true}"#,
            r#"{"key":"DamageAlteration","mode":"override","predicate":["heavens-thunder"],"property":"damage-type","selectors":["{item|id}-damage"],"value":"{item|flags.pf2e.rulesSelections.heavensThunder}"}"#,
            r#"{"itemId":"{item|id}","key":"ItemAlteration","mode":"remove","predicate":["heavens-thunder"],"property":"traits","value":"force"}"#,
            r#"{"itemId":"{item|id}","key":"ItemAlteration","mode":"add","predicate":["heavens-thunder"],"property":"traits","value":"{item|flags.pf2e.rulesSelections.heavensThunder}"}"#,
        ];
        let qi_rules = vec![
            SpellRuleElement {
                authored_order: 0,
                source_path: "system.rules.0".to_string(),
                authored_key: "RollOption".to_string(),
                authored_object_json: qi_raw[0].to_string(),
                rule: SpellRule::RollOption(SpellRollOptionRule {
                    domain: known("all".to_string()),
                    label: known(
                        "PF2E.SpecificRule.Monk.QiSpells.HeavensThunder.RollOptionLabel"
                            .to_string(),
                    ),
                    option: known("heavens-thunder".to_string()),
                    placement: known("spellcasting".to_string()),
                    predicate: known(vec![SpellRulePredicate::Term(
                        "self:effect:heavens-thunder".to_string(),
                    )]),
                    suboptions: known(vec![
                        SpellRuleSuboption {
                            label: known("PF2E.TraitElectricity".to_string()),
                            value: known("electricity".to_string()),
                        },
                        SpellRuleSuboption {
                            label: known("PF2E.TraitSonic".to_string()),
                            value: known("sonic".to_string()),
                        },
                    ]),
                    toggleable: known(true),
                }),
            },
            SpellRuleElement {
                authored_order: 1,
                source_path: "system.rules.1".to_string(),
                authored_key: "DamageAlteration".to_string(),
                authored_object_json: qi_raw[1].to_string(),
                rule: SpellRule::DamageAlteration(SpellDamageAlterationRule {
                    mode: known("override".to_string()),
                    predicate: known(vec![SpellRulePredicate::Term(
                        "heavens-thunder".to_string(),
                    )]),
                    property: known("damage-type".to_string()),
                    selectors: known(vec!["{item|id}-damage".to_string()]),
                    slug: FactValue::Missing,
                    value: known("{item|flags.pf2e.rulesSelections.heavensThunder}".to_string()),
                }),
            },
            SpellRuleElement {
                authored_order: 2,
                source_path: "system.rules.2".to_string(),
                authored_key: "ItemAlteration".to_string(),
                authored_object_json: qi_raw[2].to_string(),
                rule: SpellRule::ItemAlteration(SpellItemAlterationRule {
                    item_id: known("{item|id}".to_string()),
                    mode: known("remove".to_string()),
                    predicate: known(vec![SpellRulePredicate::Term(
                        "heavens-thunder".to_string(),
                    )]),
                    property: known("traits".to_string()),
                    value: known("force".to_string()),
                }),
            },
            SpellRuleElement {
                authored_order: 3,
                source_path: "system.rules.3".to_string(),
                authored_key: "ItemAlteration".to_string(),
                authored_object_json: qi_raw[3].to_string(),
                rule: SpellRule::ItemAlteration(SpellItemAlterationRule {
                    item_id: known("{item|id}".to_string()),
                    mode: known("add".to_string()),
                    predicate: known(vec![SpellRulePredicate::Term(
                        "heavens-thunder".to_string(),
                    )]),
                    property: known("traits".to_string()),
                    value: known("{item|flags.pf2e.rulesSelections.heavensThunder}".to_string()),
                }),
            },
        ];
        let qi_encoded = encode(&qi_rules).expect("encode Qi Blast rules");
        let qi_decoded = decode::<Vec<SpellRuleElement>>(&qi_encoded, "qi_blast_rules")
            .expect("decode Qi Blast rules");
        assert_eq!(qi_decoded, qi_rules);
        assert_eq!(
            qi_decoded
                .iter()
                .map(|element| element.authored_order)
                .collect::<Vec<_>>(),
            vec![0, 1, 2, 3]
        );
        assert_eq!(
            qi_decoded
                .iter()
                .map(|element| element.authored_object_json.as_str())
                .collect::<Vec<_>>(),
            qi_raw
        );
        let SpellRule::RollOption(roll_option) = &qi_decoded[0].rule else {
            panic!("Qi Blast roll option")
        };
        assert_eq!(
            roll_option
                .suboptions
                .as_value()
                .and_then(SpellSourceValue::as_known)
                .expect("typed suboptions")
                .iter()
                .filter_map(|suboption| {
                    suboption
                        .value
                        .as_value()
                        .and_then(SpellSourceValue::as_known)
                        .map(String::as_str)
                })
                .collect::<Vec<_>>(),
            vec!["electricity", "sonic"]
        );
        let SpellRule::DamageAlteration(alteration) = &qi_decoded[1].rule else {
            panic!("Qi Blast damage alteration")
        };
        assert_eq!(
            alteration
                .selectors
                .as_value()
                .and_then(SpellSourceValue::as_known),
            Some(&vec!["{item|id}-damage".to_string()])
        );
        assert!(matches!(
            (&qi_decoded[2].rule, &qi_decoded[3].rule),
            (
                SpellRule::ItemAlteration(SpellItemAlterationRule { mode: FactValue::Value(SpellSourceValue::Known(remove)), .. }),
                SpellRule::ItemAlteration(SpellItemAlterationRule { mode: FactValue::Value(SpellSourceValue::Known(add)), .. })
            ) if remove == "remove" && add == "add"
        ));
    }

    #[test]
    fn spell_body_codec_derives_query_only_range_and_rejects_unknown_members() {
        let encoded = encode(&spell_fixture()).expect("encode spell body");
        assert!(
            !encoded.contains(r#""numeric""#),
            "the numeric range derivation is query-only"
        );
        assert_eq!(
            decode::<RecordBody>(&encoded, "spell_fixture").expect("decode spell body"),
            spell_fixture(),
            "decoding recomputes the bounded range derivation from authored text"
        );

        let with_persisted_derivative = encoded.replacen(
            r#""authored_text":"touch""#,
            r#""authored_text":"touch","numeric":{"feet":30}"#,
            1,
        );
        assert_ne!(
            with_persisted_derivative, encoded,
            "fixture must contain an authored touch range"
        );
        assert!(decode::<RecordBody>(&with_persisted_derivative, "spell_fixture").is_err());

        let with_unknown = encoded.replacen(
            r#""source_id":"rfZpqmj0AIIdkVIs""#,
            r#""source_id":"rfZpqmj0AIIdkVIs","unknown":true"#,
            1,
        );
        assert_ne!(with_unknown, encoded, "fixture must contain spell identity");
        assert!(decode::<RecordBody>(&with_unknown, "spell_fixture").is_err());
    }

    #[test]
    fn spell_codec_preserves_source_backed_overlay_heightening_and_legacy_trait_evidence() {
        let patch = SpellHeighteningPatch {
            kind: known(SpellHeighteningType::Interval),
            interval: known(1),
            area: known(0),
            interval_damage: known(SpellKeyedPatch {
                members: vec![SpellKeyedPatchMember {
                    key: "0".to_string(),
                    authored_order: 0,
                    operation: SpellKeyedPatchOperation::Merge(SpellTextPatch {
                        value: known("2d6".to_string()),
                    }),
                }],
            }),
        };
        let encoded = encode(&patch).expect("encode Qi Blast overlay heightening");
        assert_eq!(
            decode::<SpellHeighteningPatch>(&encoded, "qi_blast_overlay")
                .expect("decode Qi Blast overlay heightening"),
            patch
        );

        let selected = SpellUnsupportedSourceFact {
            field: SpellUnsupportedSourceField::LegacyTraitSelection,
            source_path: "system.traits.selected.necromancy".to_string(),
            authored_key: "necromancy".to_string(),
            authored_order: Some(1),
            value: UnsupportedSourceValue {
                shape: UnsupportedSourceShape::String,
                value: r#""Necromancy""#.to_string(),
                reason: UnsupportedSourceReason::SourceFieldDrift,
            },
        };
        let encoded = encode(&selected).expect("encode Admonishing Ray legacy trait label");
        assert_eq!(
            decode::<SpellUnsupportedSourceFact>(&encoded, "admonishing_ray_selected")
                .expect("decode legacy trait label"),
            selected
        );
    }

    #[test]
    fn spell_body_codec_covers_fixed_patches_ritual_and_typed_unsupported_values() {
        let mut fixture = spell_fixture();
        let spell = match &mut fixture {
            RecordBody::Spell(spell) => spell,
            RecordBody::Creature(_) | RecordBody::Hazard(_) => panic!("spell fixture"),
        };
        let area_drift = UnsupportedSourceValue {
            shape: UnsupportedSourceShape::String,
            value: "burst".to_string(),
            reason: UnsupportedSourceReason::SourceFieldDrift,
        };
        spell.definition.heightening =
            known(SpellHeightening::Fixed(vec![SpellFixedHeighteningLayer {
                key: "5".to_string(),
                authored_order: 0,
                rank: SpellSourceValue::Known(5),
                patch: SpellPatch {
                    targeting: known(SpellTargetingPatch {
                        area: known(SpellAreaPatch {
                            value: known(30),
                            area_type: known(SpellAreaType::new("emanation").expect("area type")),
                            legacy_area_type: FactValue::Missing,
                            details: FactValue::Null,
                            unsupported_notes: vec![SpellUnsupportedSourceFact {
                                field: SpellUnsupportedSourceField::AreaMember,
                                source_path: "system.heightening.levels.5.area.drift".to_string(),
                                authored_key: "drift".to_string(),
                                authored_order: Some(0),
                                value: area_drift.clone(),
                            }],
                        }),
                        ..SpellTargetingPatch::default()
                    }),
                    damage: known(SpellKeyedPatch {
                        members: vec![SpellKeyedPatchMember {
                            key: "0".to_string(),
                            authored_order: 0,
                            operation: SpellKeyedPatchOperation::Delete,
                        }],
                    }),
                    unsupported: vec![SpellUnsupportedPatchField {
                        field: SpellFormField::Targeting,
                        source_path: "system.heightening.levels.5.area.drift".to_string(),
                        authored_key: "drift".to_string(),
                        authored_order: Some(0),
                        value: area_drift,
                    }],
                    ..SpellPatch::default()
                },
            }]));
        spell.definition.ritual = known(SpellRitual {
            primary_check: known("Arcana (master)".to_string()),
            secondary_casters: known(2),
            secondary_checks: known("Crafting or Occultism".to_string()),
        });

        let encoded = encode(&fixture).expect("encode variant fixture");
        assert_eq!(
            decode::<RecordBody>(&encoded, "spell_fixture").expect("decode variant fixture"),
            fixture
        );
    }

    #[test]
    fn consumable_spell_child_codec_preserves_arboreal_wand_identity_and_locator() {
        let definition = match spell_fixture() {
            RecordBody::Spell(spell) => spell.definition,
            RecordBody::Creature(_) | RecordBody::Hazard(_) => panic!("spell fixture"),
        };
        let child = ConsumableSpellChild {
            parent_record_key: RecordKey::parse("equipment-srd:eOtQtVRLeGH39dNx").expect("parent"),
            child_id: SpellChildId::new("7w37duycMs4YOBeu").expect("child id"),
            name: known("Heal".to_string()),
            authored_order: 0,
            location: known(ConsumableSpellLocation {
                value: FactValue::Null,
                heightened_rank: known(4),
            }),
            standalone_locator: known(
                StableSourceLocator::new("Compendium.pf2e.spells-srd.Item.rfZpqmj0AIIdkVIs")
                    .expect("source locator"),
            ),
            standalone_target: FactValue::Value(SpellStandaloneTarget::Resolved(
                RecordKey::parse("spells-srd:rfZpqmj0AIIdkVIs").expect("Heal key"),
            )),
            definition: SpellDefinition {
                source_context: SpellSourceContext {
                    image: known("icons/magic/life/cross-worn-green.webp".to_string()),
                    publication_license: known(PublicationLicense::new("ORC").expect("license")),
                    consumable_child: FactValue::Value(ConsumableSpellSourceContext {
                        slug: known("heal".to_string()),
                        publication_title: known("Pathfinder Player Core".to_string()),
                        publication_remaster: known(true),
                        rarity: known(atlas_domain::Rarity::Common),
                    }),
                },
                ..definition
            },
        };

        let encoded = encode(&child).expect("encode consumable child");
        let decoded = decode::<ConsumableSpellChild>(&encoded, "arboreal_wand_child")
            .expect("decode consumable child");
        assert_eq!(decoded, child);
        assert!(encoded.contains("7w37duycMs4YOBeu"));
        assert!(encoded.contains("rfZpqmj0AIIdkVIs"));
        assert!(encoded.contains("heightened_rank"));
        assert!(encoded.contains("Pathfinder Player Core"));
    }

    #[test]
    fn spell_codec_preserves_area_interval_presence_and_exact_unsupported_identity() {
        let mut fixture = spell_fixture();
        let spell = match &mut fixture {
            RecordBody::Spell(spell) => spell,
            RecordBody::Creature(_) | RecordBody::Hazard(_) => panic!("spell fixture"),
        };
        spell.definition.targeting = known(SpellTargeting {
            area: known(SpellAreaValue {
                value: known(10),
                area_type: known(SpellAreaType::new("burst").expect("area type")),
                legacy_area_type: known(SpellLegacyAreaType {
                    source_path: "system.area.areaType".to_string(),
                    authored_key: "areaType".to_string(),
                    authored_order: 2,
                    value: SpellAreaType::new("burst").expect("legacy area type"),
                }),
                details: FactValue::Missing,
                unsupported_notes: vec![
                    SpellUnsupportedSourceFact {
                        field: SpellUnsupportedSourceField::AreaMember,
                        source_path: "system.area.futureA".to_string(),
                        authored_key: "futureA".to_string(),
                        authored_order: Some(2),
                        value: UnsupportedSourceValue {
                            shape: UnsupportedSourceShape::Null,
                            value: "null".to_string(),
                            reason: UnsupportedSourceReason::SourceFieldDrift,
                        },
                    },
                    SpellUnsupportedSourceFact {
                        field: SpellUnsupportedSourceField::AreaMember,
                        source_path: "system.area.futureB".to_string(),
                        authored_key: "futureB".to_string(),
                        authored_order: Some(3),
                        value: UnsupportedSourceValue {
                            shape: UnsupportedSourceShape::String,
                            value: "kept exactly".to_string(),
                            reason: UnsupportedSourceReason::SourceFieldDrift,
                        },
                    },
                ],
            }),
            ..SpellTargeting::default()
        });
        spell.definition.heightening =
            known(SpellHeightening::Interval(SpellIntervalHeightening {
                interval: known(1),
                area: FactValue::Null,
                damage: FactValue::Missing,
            }));

        let encoded = encode(&fixture).expect("encode presence fixture");
        let decoded = decode::<RecordBody>(&encoded, "presence_fixture").expect("decode fixture");
        assert_eq!(decoded, fixture);
        assert!(encoded.contains(r#""details":{"kind":"missing"}"#));
        assert!(encoded.contains(r#""area":{"kind":"null"}"#));
        assert!(encoded.contains("system.area.futureA"));
        assert!(encoded.contains("system.area.futureB"));
    }
}
