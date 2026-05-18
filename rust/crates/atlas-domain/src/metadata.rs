use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MetadataSetField {
    Traits,
    TaxonomyFamilies,
    DerivedTags,
    Traditions,
    SpellKinds,
    DamageTypes,
    Languages,
    SpeedTypes,
    Senses,
    Immunities,
    Resistances,
    Weaknesses,
    DisableSkills,
    VariantAxes,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MetadataEnumStringField {
    PackName,
    PackLabel,
    PublicationFamily,
    Size,
    Usage,
    SystemGroup,
    FoundryRecordType,
    BaseItem,
    Hands,
    SaveType,
    AreaType,
    DurationUnit,
    Rarity,
    VariantGroupKey,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MetadataTextStringField {
    PublicationTitle,
    RangeText,
    DurationText,
    TargetText,
    DisableText,
    VariantBaseName,
    VariantLabel,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MetadataNumberField {
    Level,
    PriceCp,
    BulkValue,
    ActionCost,
    Hands,
    RangeValue,
    AreaValue,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MetadataBooleanField {
    PublicationRemaster,
    Sustained,
    BasicSave,
    IsComplex,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "field_type", rename_all = "snake_case")]
pub enum MetadataPredicate {
    Set {
        field: MetadataSetField,
        #[serde(rename = "match")]
        r#match: MetadataSetMatch,
    },
    EnumString {
        field: MetadataEnumStringField,
        #[serde(rename = "match")]
        r#match: MetadataStringMatch,
    },
    Text {
        field: MetadataTextStringField,
        #[serde(rename = "match")]
        r#match: MetadataTextMatch,
    },
    Number {
        field: MetadataNumberField,
        #[serde(rename = "match")]
        r#match: MetadataNumberMatch,
    },
    Boolean {
        field: MetadataBooleanField,
        #[serde(rename = "match")]
        r#match: MetadataBooleanMatch,
    },
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum MetadataSetMatch {
    Includes { value: String },
    IsNull,
    IsNotNull,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum MetadataStringMatch {
    Eq { value: String },
    NotEq { value: String },
    IsNull,
    IsNotNull,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum MetadataTextMatch {
    Eq { value: String },
    NotEq { value: String },
    Contains { value: String },
    NotContains { value: String },
    IsNull,
    IsNotNull,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum MetadataNumberMatch {
    Eq { value: f64 },
    Gt { value: f64 },
    Gte { value: f64 },
    Lt { value: f64 },
    Lte { value: f64 },
    Between { min: f64, max: f64 },
    IsNull,
    IsNotNull,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum MetadataBooleanMatch {
    Eq { value: bool },
    IsNull,
    IsNotNull,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum NumericMetricOperator {
    Eq,
    NotEq,
    Gt,
    Gte,
    Lt,
    Lte,
}
