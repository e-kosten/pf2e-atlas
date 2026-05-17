use atlas_domain::{MetricDomain, MetricValueType};

use super::model::{
    MetricDefinition, MetricGroup, MetricKeyPattern, MetricKeySegment, MetricLabelTemplate,
    MetricVariableVocabulary, pattern_definition, static_definition,
};

pub const ARMOR_CLASS: MetricDefinition = static_definition(
    MetricDomain::Actor,
    "ac.value",
    MetricValueType::Number,
    "ac",
    MetricLabelTemplate::FoundryI18n {
        key: "PF2E.ArmorClassLabel",
        fallback: "Armor Class",
    },
    Some(MetricLabelTemplate::FoundryI18n {
        key: "PF2E.ArmorClassShortLabel",
        fallback: "AC",
    }),
    MetricGroup::Defense,
);
pub const HARDNESS: MetricDefinition = static_definition(
    MetricDomain::Actor,
    "hardness.value",
    MetricValueType::Number,
    "hardness",
    MetricLabelTemplate::Static("Hardness"),
    None,
    MetricGroup::Defense,
);
pub const HP_VALUE: MetricDefinition = static_definition(
    MetricDomain::Actor,
    "hp.value",
    MetricValueType::Number,
    "hp",
    MetricLabelTemplate::Static("Hit Points"),
    Some(MetricLabelTemplate::Static("HP")),
    MetricGroup::Health,
);
pub const HP_MAX: MetricDefinition = static_definition(
    MetricDomain::Actor,
    "hp.max",
    MetricValueType::Number,
    "hp",
    MetricLabelTemplate::Static("Maximum Hit Points"),
    Some(MetricLabelTemplate::Static("Max HP")),
    MetricGroup::Health,
);
pub const HP_BROKEN_THRESHOLD: MetricDefinition = static_definition(
    MetricDomain::Actor,
    "hp.bt",
    MetricValueType::Number,
    "hp",
    MetricLabelTemplate::Static("Broken Threshold"),
    Some(MetricLabelTemplate::Static("BT")),
    MetricGroup::Health,
);
pub const PERCEPTION_MOD: MetricDefinition = static_definition(
    MetricDomain::Actor,
    "perception.mod",
    MetricValueType::Number,
    "perception",
    MetricLabelTemplate::Static("Perception modifier"),
    Some(MetricLabelTemplate::Static("Perception")),
    MetricGroup::Perception,
);
pub const STEALTH_MOD: MetricDefinition = static_definition(
    MetricDomain::Actor,
    "stealth.mod",
    MetricValueType::Number,
    "stealth",
    MetricLabelTemplate::Static("Stealth modifier"),
    Some(MetricLabelTemplate::Static("Stealth")),
    MetricGroup::Stealth,
);
pub const STEALTH_DC: MetricDefinition = static_definition(
    MetricDomain::Actor,
    "stealth.dc",
    MetricValueType::Number,
    "stealth",
    MetricLabelTemplate::Static("Stealth DC"),
    None,
    MetricGroup::Stealth,
);

pub mod ability {
    use super::*;

    const ABILITY_MOD_SEGMENTS: &[MetricKeySegment] = &[
        MetricKeySegment::Literal("ability"),
        MetricKeySegment::Variable {
            name: "ability",
            vocabulary: MetricVariableVocabulary::Ability,
        },
        MetricKeySegment::Literal("mod"),
    ];

    pub const MOD: MetricDefinition = pattern_definition(
        MetricDomain::Actor,
        MetricKeyPattern {
            segments: ABILITY_MOD_SEGMENTS,
        },
        MetricValueType::Number,
        "ability",
        MetricLabelTemplate::Template("{ability} modifier"),
        Some(MetricLabelTemplate::Template("{ability}")),
        MetricGroup::Abilities,
    );

    pub fn mod_key(ability: &str) -> String {
        format!("ability.{ability}.mod")
    }
}

pub mod save {
    use super::*;

    const SAVE_MOD_SEGMENTS: &[MetricKeySegment] = &[
        MetricKeySegment::Literal("save"),
        MetricKeySegment::Variable {
            name: "save",
            vocabulary: MetricVariableVocabulary::Save,
        },
        MetricKeySegment::Literal("mod"),
    ];

    pub const MOD: MetricDefinition = pattern_definition(
        MetricDomain::Actor,
        MetricKeyPattern {
            segments: SAVE_MOD_SEGMENTS,
        },
        MetricValueType::Number,
        "save",
        MetricLabelTemplate::Template("{save} modifier"),
        None,
        MetricGroup::Saves,
    );
    pub const BEST: MetricDefinition = static_definition(
        MetricDomain::Actor,
        "save.best",
        MetricValueType::Text,
        "save",
        MetricLabelTemplate::Static("Best save"),
        None,
        MetricGroup::Saves,
    );
    pub const WORST: MetricDefinition = static_definition(
        MetricDomain::Actor,
        "save.worst",
        MetricValueType::Text,
        "save",
        MetricLabelTemplate::Static("Worst save"),
        None,
        MetricGroup::Saves,
    );

    pub fn mod_key(save: &str) -> String {
        format!("save.{save}.mod")
    }
}

pub mod skill {
    use super::*;

    const SKILL_MOD_SEGMENTS: &[MetricKeySegment] = &[
        MetricKeySegment::Literal("skill"),
        MetricKeySegment::Variable {
            name: "skill",
            vocabulary: MetricVariableVocabulary::Skill,
        },
        MetricKeySegment::Literal("mod"),
    ];
    const SKILL_RANK_SEGMENTS: &[MetricKeySegment] = &[
        MetricKeySegment::Literal("skill"),
        MetricKeySegment::Variable {
            name: "skill",
            vocabulary: MetricVariableVocabulary::Skill,
        },
        MetricKeySegment::Literal("rank"),
    ];
    const SKILL_PROFICIENT_SEGMENTS: &[MetricKeySegment] = &[
        MetricKeySegment::Literal("skill"),
        MetricKeySegment::Variable {
            name: "skill",
            vocabulary: MetricVariableVocabulary::Skill,
        },
        MetricKeySegment::Literal("proficient"),
    ];

    pub const MOD: MetricDefinition = pattern_definition(
        MetricDomain::Actor,
        MetricKeyPattern {
            segments: SKILL_MOD_SEGMENTS,
        },
        MetricValueType::Number,
        "skill",
        MetricLabelTemplate::Template("{skill} modifier"),
        Some(MetricLabelTemplate::Template("{skill}")),
        MetricGroup::Skills,
    );
    pub const RANK: MetricDefinition = pattern_definition(
        MetricDomain::Actor,
        MetricKeyPattern {
            segments: SKILL_RANK_SEGMENTS,
        },
        MetricValueType::Number,
        "skill",
        MetricLabelTemplate::Template("{skill} rank"),
        None,
        MetricGroup::Skills,
    );
    pub const PROFICIENT: MetricDefinition = pattern_definition(
        MetricDomain::Actor,
        MetricKeyPattern {
            segments: SKILL_PROFICIENT_SEGMENTS,
        },
        MetricValueType::Boolean,
        "skill",
        MetricLabelTemplate::Template("{skill} proficiency"),
        None,
        MetricGroup::Skills,
    );

    pub fn mod_key(skill: &str) -> String {
        format!("skill.{skill}.mod")
    }

    pub fn rank_key(skill: &str) -> String {
        format!("skill.{skill}.rank")
    }

    pub fn proficient_key(skill: &str) -> String {
        format!("skill.{skill}.proficient")
    }
}

pub mod speed {
    use super::*;

    const SPEED_VALUE_SEGMENTS: &[MetricKeySegment] = &[
        MetricKeySegment::Literal("speed"),
        MetricKeySegment::Variable {
            name: "movement",
            vocabulary: MetricVariableVocabulary::MovementType,
        },
        MetricKeySegment::Literal("value"),
    ];

    pub const VALUE: MetricDefinition = pattern_definition(
        MetricDomain::Actor,
        MetricKeyPattern {
            segments: SPEED_VALUE_SEGMENTS,
        },
        MetricValueType::Number,
        "speed",
        MetricLabelTemplate::Template("{movement} Speed"),
        None,
        MetricGroup::Movement,
    );

    pub fn value_key(movement: &str) -> String {
        format!("speed.{movement}.value")
    }
}

pub mod sense {
    use super::*;

    const SENSE_RANGE_SEGMENTS: &[MetricKeySegment] = &[
        MetricKeySegment::Literal("sense"),
        MetricKeySegment::Variable {
            name: "sense",
            vocabulary: MetricVariableVocabulary::SenseType,
        },
        MetricKeySegment::Literal("range"),
    ];

    pub const RANGE: MetricDefinition = pattern_definition(
        MetricDomain::Actor,
        MetricKeyPattern {
            segments: SENSE_RANGE_SEGMENTS,
        },
        MetricValueType::Number,
        "sense",
        MetricLabelTemplate::Template("{sense} range"),
        None,
        MetricGroup::Senses,
    );

    pub fn range_key(sense: &str) -> String {
        format!("sense.{sense}.range")
    }
}

pub mod disable {
    use super::*;

    const DISABLE_SKILL_DC_MIN_SEGMENTS: &[MetricKeySegment] = &[
        MetricKeySegment::Literal("disable"),
        MetricKeySegment::Variable {
            name: "skill",
            vocabulary: MetricVariableVocabulary::Skill,
        },
        MetricKeySegment::Literal("dc"),
        MetricKeySegment::Literal("min"),
    ];
    const DISABLE_SKILL_DC_MAX_SEGMENTS: &[MetricKeySegment] = &[
        MetricKeySegment::Literal("disable"),
        MetricKeySegment::Variable {
            name: "skill",
            vocabulary: MetricVariableVocabulary::Skill,
        },
        MetricKeySegment::Literal("dc"),
        MetricKeySegment::Literal("max"),
    ];
    const DISABLE_SKILL_RANK_MIN_SEGMENTS: &[MetricKeySegment] = &[
        MetricKeySegment::Literal("disable"),
        MetricKeySegment::Variable {
            name: "skill",
            vocabulary: MetricVariableVocabulary::Skill,
        },
        MetricKeySegment::Literal("rank"),
        MetricKeySegment::Literal("min"),
    ];

    pub const DC_MIN: MetricDefinition = static_definition(
        MetricDomain::Actor,
        "disable.dc.min",
        MetricValueType::Number,
        "disable",
        MetricLabelTemplate::Static("Minimum disable DC"),
        Some(MetricLabelTemplate::Static("Disable DC")),
        MetricGroup::Disable,
    );
    pub const DC_MAX: MetricDefinition = static_definition(
        MetricDomain::Actor,
        "disable.dc.max",
        MetricValueType::Number,
        "disable",
        MetricLabelTemplate::Static("Maximum disable DC"),
        Some(MetricLabelTemplate::Static("Disable DC max")),
        MetricGroup::Disable,
    );
    pub const SKILL_DC_MIN: MetricDefinition = pattern_definition(
        MetricDomain::Actor,
        MetricKeyPattern {
            segments: DISABLE_SKILL_DC_MIN_SEGMENTS,
        },
        MetricValueType::Number,
        "disable",
        MetricLabelTemplate::Template("Minimum {skill} disable DC"),
        Some(MetricLabelTemplate::Template("{skill} disable DC")),
        MetricGroup::Disable,
    );
    pub const SKILL_DC_MAX: MetricDefinition = pattern_definition(
        MetricDomain::Actor,
        MetricKeyPattern {
            segments: DISABLE_SKILL_DC_MAX_SEGMENTS,
        },
        MetricValueType::Number,
        "disable",
        MetricLabelTemplate::Template("Maximum {skill} disable DC"),
        Some(MetricLabelTemplate::Template("{skill} disable DC max")),
        MetricGroup::Disable,
    );
    pub const SKILL_RANK_MIN: MetricDefinition = pattern_definition(
        MetricDomain::Actor,
        MetricKeyPattern {
            segments: DISABLE_SKILL_RANK_MIN_SEGMENTS,
        },
        MetricValueType::Number,
        "disable",
        MetricLabelTemplate::Template("Minimum {skill} disable rank"),
        None,
        MetricGroup::Disable,
    );

    pub fn skill_dc_min_key(skill: &str) -> String {
        format!("disable.{skill}.dc.min")
    }

    pub fn skill_dc_max_key(skill: &str) -> String {
        format!("disable.{skill}.dc.max")
    }

    pub fn skill_rank_min_key(skill: &str) -> String {
        format!("disable.{skill}.rank.min")
    }
}
