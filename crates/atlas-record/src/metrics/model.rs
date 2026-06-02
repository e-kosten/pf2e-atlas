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
pub enum MetricKeyDefinition {
    Static(&'static str),
    Pattern(MetricKeyPattern),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MetricDefinition {
    pub domain: MetricDomain,
    pub key: MetricKeyDefinition,
    pub value_type: MetricValueType,
    pub namespace: &'static str,
    pub label: MetricLabelTemplate,
    pub short_label: Option<MetricLabelTemplate>,
    pub group: MetricGroup,
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
        match self.key {
            MetricKeyDefinition::Static(key) => Some(key),
            MetricKeyDefinition::Pattern(_) => None,
        }
    }

    pub const fn domain(self) -> MetricDomain {
        self.domain
    }

    pub const fn value_type(self) -> MetricValueType {
        self.value_type
    }

    pub const fn namespace(self) -> &'static str {
        self.namespace
    }

    pub const fn group(self) -> MetricGroup {
        self.group
    }

    pub(crate) const fn label_template(self) -> MetricLabelTemplate {
        self.label
    }

    pub(crate) const fn short_label_template(self) -> Option<MetricLabelTemplate> {
        self.short_label
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
    MetricDefinition {
        domain,
        key: MetricKeyDefinition::Static(key),
        value_type,
        namespace,
        label,
        short_label,
        group,
    }
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
    MetricDefinition {
        domain,
        key: MetricKeyDefinition::Pattern(pattern),
        value_type,
        namespace,
        label,
        short_label,
        group,
    }
}
