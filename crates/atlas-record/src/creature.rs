use atlas_domain::{Rarity, RecordKey};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RecordBody {
    Creature(CreatureRecord),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CreatureRecord {
    pub identity: CreatureIdentity,
    pub level: CreatureFact<i64>,
    pub rarity: CreatureFact<Rarity>,
    pub traits: CreatureFact<Vec<CreatureTrait>>,
    pub size: CreatureFact<CreatureSize>,
    pub publication: CreatureFact<CreaturePublication>,
    pub adjustment: CreatureFact<CreatureAdjustment>,
    pub source_alliance: CreatureFact<CreatureSourceAlliance>,
    pub perception: CreatureFact<CreaturePerception>,
    pub initiative: CreatureFact<CreatureInitiative>,
    pub languages: CreatureFact<CreatureLanguages>,
    pub skills: CreatureFact<Vec<CreatureSkill>>,
    pub legacy_abilities: CreatureFact<CreatureLegacyAbilities>,
    pub defenses: CreatureFact<CreatureDefenses>,
    pub movement: CreatureFact<Vec<CreatureSpeed>>,
    pub resources: CreatureFact<Vec<CreatureResource>>,
    pub embedded_entities: CreatureFact<crate::CreatureEmbeddedEntities>,
    pub content: crate::OwnedRichContent,
    pub provenance: CreatureProvenance,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CreatureIdentity {
    pub record_key: RecordKey,
    pub source_id: CreatureSourceId,
    pub name: String,
    pub family: CreatureFamily,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum CreatureFamily {
    Npc,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct CreatureSourceId(String);

impl CreatureSourceId {
    pub fn new(value: impl Into<String>) -> Result<Self, InvalidCanonicalId> {
        validated_id(value.into()).map(Self)
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CreatureProvenance {
    pub source_path: String,
    pub source_contract_version: String,
    pub source_system_version: String,
    pub source_upstream_commit: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CreatureFact<T> {
    pub value: FactValue<T>,
    pub provenance: CreatureFactProvenance,
}

impl<T> CreatureFact<T> {
    pub fn source(value: FactValue<T>, field: CreatureSourceField) -> Self {
        Self {
            value,
            provenance: CreatureFactProvenance::Source(field),
        }
    }

    pub fn derived(value: FactValue<T>, derivation: CreatureDerivation) -> Self {
        Self {
            value,
            provenance: CreatureFactProvenance::Derived(derivation),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FactValue<T> {
    Missing,
    Null,
    Value(T),
}

impl<T> FactValue<T> {
    pub fn as_value(&self) -> Option<&T> {
        match self {
            Self::Value(value) => Some(value),
            Self::Missing | Self::Null => None,
        }
    }

    pub fn map<U>(self, map: impl FnOnce(T) -> U) -> FactValue<U> {
        match self {
            Self::Missing => FactValue::Missing,
            Self::Null => FactValue::Null,
            Self::Value(value) => FactValue::Value(map(value)),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CreatureFactProvenance {
    Source(CreatureSourceField),
    Derived(CreatureDerivation),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum CreatureSourceField {
    Identity,
    Level,
    Rarity,
    Traits,
    Size,
    Publication,
    Adjustment,
    SourceAlliance,
    Perception,
    Initiative,
    Languages,
    Skills,
    LegacyAbilities,
    Defenses,
    Movement,
    Resources,
    EmbeddedEntities,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum CreatureDerivation {
    RecordClassificationProjection,
    LegacyActorProjection,
    DisplayLabel,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct CreatureTrait(String);

impl CreatureTrait {
    pub fn new(value: impl Into<String>) -> Result<Self, InvalidCanonicalSlug> {
        validated_slug(value.into()).map(Self)
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum CreatureSize {
    Tiny,
    Small,
    Medium,
    Large,
    Huge,
    Gargantuan,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CreatureAdjustment {
    Elite,
    Weak,
    Unsupported(UnsupportedSourceValue),
}

impl CreatureAdjustment {
    pub fn from_source(value: &str) -> Option<Self> {
        match value {
            "elite" => Some(Self::Elite),
            "weak" => Some(Self::Weak),
            _ => None,
        }
    }
}

impl CreatureSize {
    pub fn from_source(value: &str) -> Option<Self> {
        match value {
            "tiny" => Some(Self::Tiny),
            "sm" | "small" => Some(Self::Small),
            "med" | "medium" => Some(Self::Medium),
            "lg" | "large" => Some(Self::Large),
            "huge" => Some(Self::Huge),
            "grg" | "gargantuan" => Some(Self::Gargantuan),
            _ => None,
        }
    }

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

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CreaturePublication {
    pub title: FactValue<String>,
    pub remaster: FactValue<bool>,
    pub license: FactValue<PublicationLicense>,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct PublicationLicense(String);

impl PublicationLicense {
    pub fn new(value: impl Into<String>) -> Result<Self, InvalidCanonicalValue> {
        validated_text(value.into()).map(Self)
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CreaturePerception {
    pub modifier: FactValue<i64>,
    pub details: FactValue<CreatureNote>,
    pub has_vision: FactValue<bool>,
    pub senses: FactValue<Vec<CreatureSense>>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CreatureInitiative {
    pub statistic: FactValue<CreatureInitiativeStatistic>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CreatureInitiativeStatistic {
    Named(CreatureStatistic),
    Unsupported(UnsupportedSourceValue),
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct CreatureStatistic(String);

impl CreatureStatistic {
    pub fn new(value: impl Into<String>) -> Result<Self, InvalidCanonicalSlug> {
        validated_slug(value.into()).map(Self)
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CreatureSourceAlliance {
    Named(CreatureAllianceName),
    Unsupported(UnsupportedSourceValue),
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct CreatureAllianceName(String);

impl CreatureAllianceName {
    pub fn new(value: impl Into<String>) -> Result<Self, InvalidCanonicalValue> {
        validated_text(value.into()).map(Self)
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CreatureSense {
    pub id: CreatureComponentId,
    pub authored_order: u32,
    pub sense_type: SenseType,
    pub acuity: FactValue<SenseAcuity>,
    pub range: FactValue<i64>,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct SenseType(String);

impl SenseType {
    pub fn new(value: impl Into<String>) -> Result<Self, InvalidCanonicalSlug> {
        validated_slug(value.into()).map(Self)
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SenseAcuity {
    Precise,
    Imprecise,
    Vague,
    Unsupported(UnsupportedSourceValue),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CreatureLanguages {
    pub values: FactValue<Vec<Language>>,
    pub details: FactValue<CreatureNote>,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Language(String);

impl Language {
    pub fn new(value: impl Into<String>) -> Result<Self, InvalidCanonicalSlug> {
        validated_slug(value.into()).map(Self)
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CreatureSkill {
    pub id: CreatureComponentId,
    pub authored_order: u32,
    pub source_entries: Vec<CreatureSkillSourceEntry>,
    pub kind: CreatureSkillKind,
    pub label: String,
    pub modifier: FactValue<i64>,
    pub note: FactValue<CreatureNote>,
    pub variants: FactValue<Vec<CreatureSkillVariant>>,
    pub source_item_id: FactValue<CreatureSourceId>,
    pub unmodeled: FactValue<CreatureUnmodeledSkill>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CreatureSkillSourceEntry {
    pub authored_key: String,
    pub modifier: FactValue<i64>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CreatureUnmodeledSkill {
    pub authored_key: String,
    pub base: FactValue<i64>,
    pub reason: CreatureUnmodeledSkillReason,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum CreatureUnmodeledSkillReason {
    UnknownAuthoredKey,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CreatureLegacyAbilities {
    pub strength: FactValue<i64>,
    pub dexterity: FactValue<i64>,
    pub constitution: FactValue<i64>,
    pub intelligence: FactValue<i64>,
    pub wisdom: FactValue<i64>,
    pub charisma: FactValue<i64>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum CreatureSkillKind {
    Acrobatics,
    Arcana,
    Athletics,
    Crafting,
    Deception,
    Diplomacy,
    Intimidation,
    Medicine,
    Nature,
    Occultism,
    Performance,
    Religion,
    Society,
    Stealth,
    Survival,
    Thievery,
    Lore,
    Unmodeled,
}

impl CreatureSkillKind {
    pub const STANDARD: [Self; 16] = [
        Self::Acrobatics,
        Self::Arcana,
        Self::Athletics,
        Self::Crafting,
        Self::Deception,
        Self::Diplomacy,
        Self::Intimidation,
        Self::Medicine,
        Self::Nature,
        Self::Occultism,
        Self::Performance,
        Self::Religion,
        Self::Society,
        Self::Stealth,
        Self::Survival,
        Self::Thievery,
    ];

    pub fn from_source_slug(value: &str) -> Option<Self> {
        match value {
            "acrobatics" => Some(Self::Acrobatics),
            "arcana" => Some(Self::Arcana),
            "athletics" => Some(Self::Athletics),
            "crafting" => Some(Self::Crafting),
            "deception" => Some(Self::Deception),
            "diplomacy" => Some(Self::Diplomacy),
            "intimidation" => Some(Self::Intimidation),
            "medicine" => Some(Self::Medicine),
            "nature" => Some(Self::Nature),
            "occultism" => Some(Self::Occultism),
            "performance" => Some(Self::Performance),
            "religion" => Some(Self::Religion),
            "society" => Some(Self::Society),
            "stealth" => Some(Self::Stealth),
            "survival" => Some(Self::Survival),
            "thievery" => Some(Self::Thievery),
            _ => None,
        }
    }

    pub const fn source_slug(self) -> &'static str {
        match self {
            Self::Acrobatics => "acrobatics",
            Self::Arcana => "arcana",
            Self::Athletics => "athletics",
            Self::Crafting => "crafting",
            Self::Deception => "deception",
            Self::Diplomacy => "diplomacy",
            Self::Intimidation => "intimidation",
            Self::Medicine => "medicine",
            Self::Nature => "nature",
            Self::Occultism => "occultism",
            Self::Performance => "performance",
            Self::Religion => "religion",
            Self::Society => "society",
            Self::Stealth => "stealth",
            Self::Survival => "survival",
            Self::Thievery => "thievery",
            Self::Lore => "lore",
            Self::Unmodeled => "unmodeled",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CreatureSkillVariant {
    pub id: CreatureComponentId,
    pub authored_order: u32,
    pub modifier: FactValue<i64>,
    pub label: FactValue<String>,
    pub predicate: FactValue<Vec<CreaturePredicate>>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CreaturePredicate {
    Term(PredicateTerm),
    Not(PredicateTerm),
    Any(Vec<PredicateTerm>),
    AtLeast { term: PredicateTerm, minimum: i64 },
    Unsupported(UnsupportedSourceValue),
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct PredicateTerm(String);

impl PredicateTerm {
    pub fn new(value: impl Into<String>) -> Result<Self, InvalidCanonicalValue> {
        validated_text(value.into()).map(Self)
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct CreatureNote(String);

impl CreatureNote {
    pub fn new(value: impl Into<String>) -> Self {
        Self(value.into())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CreatureDefenses {
    pub armor_class: FactValue<CreatureArmorClass>,
    pub hit_points: FactValue<CreatureHitPoints>,
    pub hardness: FactValue<i64>,
    pub shield: FactValue<CreatureShield>,
    pub saves: FactValue<CreatureSaves>,
    pub all_saves_note: FactValue<CreatureNote>,
    pub immunities: FactValue<Vec<CreatureIwr>>,
    pub resistances: FactValue<Vec<CreatureIwr>>,
    pub weaknesses: FactValue<Vec<CreatureIwr>>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CreatureShield {
    pub armor_class_bonus: FactValue<i64>,
    pub broken_threshold: FactValue<i64>,
    pub hardness: FactValue<i64>,
    pub maximum_hit_points: FactValue<i64>,
    pub serialized_hit_points: FactValue<i64>,
    pub current_policy: ShieldCurrentPolicy,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum ShieldCurrentPolicy {
    SerializedHitPointsAreProvenanceOnly,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CreatureArmorClass {
    pub value: FactValue<i64>,
    pub details: FactValue<CreatureNote>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CreatureHitPoints {
    pub value: FactValue<CreatureNumber>,
    pub maximum: FactValue<i64>,
    pub temporary: FactValue<i64>,
    pub temporary_maximum: FactValue<i64>,
    pub details: FactValue<CreatureNote>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CreatureNumber {
    Integer(i64),
    Unsupported(UnsupportedSourceValue),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CreatureSaves {
    pub fortitude: FactValue<CreatureSave>,
    pub reflex: FactValue<CreatureSave>,
    pub will: FactValue<CreatureSave>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CreatureSave {
    pub id: CreatureComponentId,
    pub kind: CreatureSaveKind,
    pub value: FactValue<i64>,
    pub details: FactValue<CreatureNote>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum CreatureSaveKind {
    Fortitude,
    Reflex,
    Will,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CreatureIwr {
    pub id: CreatureComponentId,
    pub authored_order: u32,
    pub kind: CreatureIwrKind,
    pub iwr_type: IwrType,
    pub value: FactValue<i64>,
    pub exceptions: FactValue<Vec<IwrQualifier>>,
    pub double_vs: FactValue<Vec<IwrQualifier>>,
    pub apply_once: FactValue<bool>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum CreatureIwrKind {
    Immunity,
    Resistance,
    Weakness,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct IwrType(String);

impl IwrType {
    pub fn new(value: impl Into<String>) -> Result<Self, InvalidCanonicalSlug> {
        validated_slug(value.into()).map(Self)
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct IwrQualifier(String);

impl IwrQualifier {
    pub fn new(value: impl Into<String>) -> Result<Self, InvalidCanonicalSlug> {
        validated_slug(value.into()).map(Self)
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CreatureSpeed {
    pub id: CreatureComponentId,
    pub authored_order: u32,
    pub mode: CreatureMovementMode,
    pub value: FactValue<i64>,
    pub label: FactValue<String>,
    pub details: FactValue<CreatureNote>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CreatureMovementMode {
    Land,
    Burrow,
    Climb,
    Fly,
    Swim,
    Unsupported(UnsupportedSourceValue),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CreatureResource {
    pub id: CreatureComponentId,
    pub authored_order: u32,
    pub kind: CreatureResourceKind,
    pub label: String,
    pub maximum: FactValue<CreatureResourceAmount>,
    pub serialized_value: FactValue<CreatureResourceAmount>,
    pub source_drift: FactValue<Vec<CreatureUnsupportedSourceFact>>,
    pub current_policy: ResourceCurrentPolicy,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct CreatureResourceKind(String);

impl CreatureResourceKind {
    pub fn new(value: impl Into<String>) -> Result<Self, InvalidCanonicalSlug> {
        validated_slug(value.into()).map(Self)
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CreatureResourceAmount {
    Integer(i64),
    Unsupported(UnsupportedSourceValue),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum ResourceCurrentPolicy {
    SerializedValueIsProvenanceOnly,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct CreatureComponentId(String);

impl CreatureComponentId {
    pub fn new(value: impl Into<String>) -> Result<Self, InvalidCanonicalId> {
        validated_id(value.into()).map(Self)
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UnsupportedSourceValue {
    pub shape: UnsupportedSourceShape,
    pub value: String,
    pub reason: UnsupportedSourceReason,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CreatureUnsupportedSourceFact {
    pub field: CreatureUnsupportedSourceField,
    pub value: UnsupportedSourceValue,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum CreatureUnsupportedSourceField {
    ResourceMaximumDrift,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum UnsupportedSourceShape {
    Missing,
    Null,
    String,
    Number,
    Boolean,
    Array,
    Object,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum UnsupportedSourceReason {
    OpenVocabulary,
    AmbiguousLegacyShape,
    InvalidPredicate,
    NonCanonicalRuntimeValue,
    SourceFieldDrift,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InvalidCanonicalId;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InvalidCanonicalSlug;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InvalidCanonicalValue;

fn validated_id(value: String) -> Result<String, InvalidCanonicalId> {
    if value.is_empty() || value.chars().any(char::is_whitespace) {
        Err(InvalidCanonicalId)
    } else {
        Ok(value)
    }
}

fn validated_slug(value: String) -> Result<String, InvalidCanonicalSlug> {
    if value.is_empty()
        || value
            .chars()
            .any(|character| character.is_whitespace() || character.is_ascii_uppercase())
    {
        Err(InvalidCanonicalSlug)
    } else {
        Ok(value)
    }
}

fn validated_text(value: String) -> Result<String, InvalidCanonicalValue> {
    if value.trim().is_empty() {
        Err(InvalidCanonicalValue)
    } else {
        Ok(value)
    }
}

#[cfg(test)]
mod tests {
    use super::{CreatureAdjustment, CreatureSize, CreatureSkillKind, FactValue, Language};

    #[test]
    fn closed_creature_vocabularies_reject_unknown_values() {
        assert_eq!(CreatureSize::from_source("med"), Some(CreatureSize::Medium));
        assert_eq!(CreatureSize::from_source("colossal"), None);
        assert_eq!(
            CreatureAdjustment::from_source("elite"),
            Some(CreatureAdjustment::Elite)
        );
        assert_eq!(CreatureAdjustment::from_source("mythic"), None);
        assert_eq!(
            CreatureSkillKind::from_source_slug("occultism"),
            Some(CreatureSkillKind::Occultism)
        );
        assert_eq!(CreatureSkillKind::from_source_slug("shadow-key"), None);
    }

    #[test]
    fn open_source_vocabularies_use_validated_newtypes() {
        let language = Language::new("chthonian").expect("valid language slug");
        assert_eq!(language.as_str(), "chthonian");
        assert!(Language::new("Invalid Language").is_err());
    }

    #[test]
    fn fact_value_preserves_missing_null_and_value() {
        assert_eq!(FactValue::<i64>::Missing.as_value(), None);
        assert_eq!(FactValue::<i64>::Null.as_value(), None);
        assert_eq!(FactValue::Value(0).as_value(), Some(&0));
    }
}
