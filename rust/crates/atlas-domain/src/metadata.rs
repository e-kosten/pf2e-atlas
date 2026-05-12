use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MetadataSetField {
    Traits,
    Families,
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
    SourceCategory,
    Size,
    Usage,
    WeaponGroup,
    ArmorGroup,
    ItemCategory,
    BaseItem,
    SaveType,
    AreaType,
    DurationUnit,
    Rarity,
    VariantFamilyKey,
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
    HasDescription,
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
        op: CollectionOperator,
        #[serde(skip_serializing_if = "Option::is_none")]
        value: Option<String>,
    },
    EnumString {
        field: MetadataEnumStringField,
        op: StringOperator,
        #[serde(skip_serializing_if = "Option::is_none")]
        value: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        values: Option<Vec<String>>,
    },
    Text {
        field: MetadataTextStringField,
        op: TextOperator,
        #[serde(skip_serializing_if = "Option::is_none")]
        value: Option<String>,
    },
    Number {
        field: MetadataNumberField,
        op: NumberOperator,
        #[serde(skip_serializing_if = "Option::is_none")]
        value: Option<f64>,
        #[serde(skip_serializing_if = "Option::is_none")]
        min: Option<f64>,
        #[serde(skip_serializing_if = "Option::is_none")]
        max: Option<f64>,
    },
    Boolean {
        field: MetadataBooleanField,
        op: BooleanOperator,
        #[serde(skip_serializing_if = "Option::is_none")]
        value: Option<bool>,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EqualityOperator {
    Eq,
    NotEq,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum OrderingOperator {
    Gt,
    Gte,
    Lt,
    Lte,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum NullOperator {
    IsNull,
    IsNotNull,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CollectionOperator {
    Includes,
    IsNull,
    IsNotNull,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum StringOperator {
    Eq,
    NotEq,
    In,
    NotIn,
    IsNull,
    IsNotNull,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TextOperator {
    Eq,
    NotEq,
    Contains,
    NotContains,
    IsNull,
    IsNotNull,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum NumberOperator {
    Eq,
    Gt,
    Gte,
    Lt,
    Lte,
    Between,
    IsNull,
    IsNotNull,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum BooleanOperator {
    Eq,
    IsNull,
    IsNotNull,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MetricOperator {
    Eq,
    NotEq,
    Gt,
    Gte,
    Lt,
    Lte,
}

pub type NumericMetricOperator = MetricOperator;
