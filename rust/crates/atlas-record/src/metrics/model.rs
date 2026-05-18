use atlas_domain::{MetricDomain, MetricValueType};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MetricGroup {
    Abilities,
    Defense,
    Health,
    Perception,
    Saves,
    Skills,
    Movement,
    Senses,
    Stealth,
    Items,
    Disable,
}

impl MetricGroup {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Abilities => "abilities",
            Self::Defense => "defense",
            Self::Health => "health",
            Self::Perception => "perception",
            Self::Saves => "saves",
            Self::Skills => "skills",
            Self::Movement => "movement",
            Self::Senses => "senses",
            Self::Stealth => "stealth",
            Self::Items => "items",
            Self::Disable => "disable",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MetricLabelTemplate {
    Static(&'static str),
    FoundryI18n {
        key: &'static str,
        fallback: &'static str,
    },
    Template(&'static str),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MetricVariableVocabulary {
    Ability,
    Skill,
    Save,
    MovementType,
    SenseType,
    FreeSlug,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MetricKeySegment {
    Literal(&'static str),
    Variable {
        name: &'static str,
        vocabulary: MetricVariableVocabulary,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MetricKeyPattern {
    pub segments: &'static [MetricKeySegment],
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct StaticMetricDefinition {
    pub domain: MetricDomain,
    pub key: &'static str,
    pub value_type: MetricValueType,
    pub namespace: &'static str,
    pub label: MetricLabelTemplate,
    pub short_label: Option<MetricLabelTemplate>,
    pub group: MetricGroup,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PatternMetricDefinition {
    pub domain: MetricDomain,
    pub pattern: MetricKeyPattern,
    pub value_type: MetricValueType,
    pub namespace: &'static str,
    pub label: MetricLabelTemplate,
    pub short_label: Option<MetricLabelTemplate>,
    pub group: MetricGroup,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MetricDefinition {
    Static(StaticMetricDefinition),
    Pattern(PatternMetricDefinition),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MetricCapture {
    pub name: &'static str,
    pub raw: String,
    pub label: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MetricDefinitionMatch {
    pub definition: &'static MetricDefinition,
    pub captures: Vec<MetricCapture>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MetricDisplayLabel {
    pub label: String,
    pub short_label: Option<String>,
    pub known: bool,
}

impl MetricDefinition {
    pub const fn exact_key(self) -> Option<&'static str> {
        match self {
            Self::Static(definition) => Some(definition.key),
            Self::Pattern(_) => None,
        }
    }

    pub const fn domain(self) -> MetricDomain {
        match self {
            Self::Static(definition) => definition.domain,
            Self::Pattern(definition) => definition.domain,
        }
    }

    pub const fn value_type(self) -> MetricValueType {
        match self {
            Self::Static(definition) => definition.value_type,
            Self::Pattern(definition) => definition.value_type,
        }
    }

    pub const fn namespace(self) -> &'static str {
        match self {
            Self::Static(definition) => definition.namespace,
            Self::Pattern(definition) => definition.namespace,
        }
    }

    pub const fn group(self) -> MetricGroup {
        match self {
            Self::Static(definition) => definition.group,
            Self::Pattern(definition) => definition.group,
        }
    }

    pub(crate) const fn label_template(self) -> MetricLabelTemplate {
        match self {
            Self::Static(definition) => definition.label,
            Self::Pattern(definition) => definition.label,
        }
    }

    pub(crate) const fn short_label_template(self) -> Option<MetricLabelTemplate> {
        match self {
            Self::Static(definition) => definition.short_label,
            Self::Pattern(definition) => definition.short_label,
        }
    }
}

pub(crate) const fn static_definition(
    domain: MetricDomain,
    key: &'static str,
    value_type: MetricValueType,
    namespace: &'static str,
    label: MetricLabelTemplate,
    short_label: Option<MetricLabelTemplate>,
    group: MetricGroup,
) -> MetricDefinition {
    MetricDefinition::Static(StaticMetricDefinition {
        domain,
        key,
        value_type,
        namespace,
        label,
        short_label,
        group,
    })
}

pub(crate) const fn pattern_definition(
    domain: MetricDomain,
    pattern: MetricKeyPattern,
    value_type: MetricValueType,
    namespace: &'static str,
    label: MetricLabelTemplate,
    short_label: Option<MetricLabelTemplate>,
    group: MetricGroup,
) -> MetricDefinition {
    MetricDefinition::Pattern(PatternMetricDefinition {
        domain,
        pattern,
        value_type,
        namespace,
        label,
        short_label,
        group,
    })
}
