use crate::{CreatureComponentId, CreatureOccurrenceId, CreatureSaveKind, CreatureSkillKind};

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum MechanicTarget {
    ArmorClass,
    MaxHp,
    Perception,
    Save {
        save: SaveKind,
    },
    /// Transitional sparse-record target. Canonical creature projections use
    /// `CreatureSkill`, whose component identity cannot be inferred from a metric key.
    Skill {
        slug: String,
    },
    CreatureSkill {
        skill_id: CreatureComponentId,
        kind: CreatureSkillKind,
    },
    AbilityModifier {
        ability: AbilityKind,
    },
    Movement {
        speed_id: CreatureComponentId,
    },
    ResourceMaximum {
        resource_id: CreatureComponentId,
    },
    ActivityActionCost {
        occurrence_id: CreatureOccurrenceId,
    },
    ActivityFrequency {
        occurrence_id: CreatureOccurrenceId,
    },
    ActivityUses {
        occurrence_id: CreatureOccurrenceId,
    },
    ActivityRoll {
        occurrence_id: CreatureOccurrenceId,
        roll_id: String,
    },
    ActivityDamage {
        occurrence_id: CreatureOccurrenceId,
        damage_id: String,
    },
    SpellcastingAttack {
        entry_occurrence_id: CreatureOccurrenceId,
    },
    SpellcastingDc {
        entry_occurrence_id: CreatureOccurrenceId,
    },
    SpellSlotMaximum {
        entry_occurrence_id: CreatureOccurrenceId,
        rank: i64,
    },
    ActorRitualDc,
}

impl MechanicTarget {
    pub fn id(&self) -> String {
        match self {
            Self::ArmorClass => "ac".to_string(),
            Self::MaxHp => "hp.max".to_string(),
            Self::Perception => "perception".to_string(),
            Self::Save { save } => format!("save.{}", save.as_str()),
            Self::Skill { slug } => format!("skill.{slug}"),
            Self::CreatureSkill { skill_id, .. } => {
                format!("skill.component.{}", skill_id.as_str())
            }
            Self::AbilityModifier { ability } => format!("ability.{}", ability.as_str()),
            Self::Movement { speed_id } => format!("movement.{}", speed_id.as_str()),
            Self::ResourceMaximum { resource_id } => {
                format!("resource.{}.max", resource_id.as_str())
            }
            Self::ActivityActionCost { occurrence_id } => {
                format!("activity.{}.action_cost", occurrence_id.as_str())
            }
            Self::ActivityFrequency { occurrence_id } => {
                format!("activity.{}.frequency", occurrence_id.as_str())
            }
            Self::ActivityUses { occurrence_id } => {
                format!("activity.{}.uses", occurrence_id.as_str())
            }
            Self::ActivityRoll {
                occurrence_id,
                roll_id,
            } => format!("activity.{}.roll.{roll_id}", occurrence_id.as_str()),
            Self::ActivityDamage {
                occurrence_id,
                damage_id,
            } => format!("activity.{}.damage.{damage_id}", occurrence_id.as_str()),
            Self::SpellcastingAttack {
                entry_occurrence_id,
            } => format!("spellcasting.{}.attack", entry_occurrence_id.as_str()),
            Self::SpellcastingDc {
                entry_occurrence_id,
            } => format!("spellcasting.{}.dc", entry_occurrence_id.as_str()),
            Self::SpellSlotMaximum {
                entry_occurrence_id,
                rank,
            } => format!(
                "spellcasting.{}.slot.{rank}.max",
                entry_occurrence_id.as_str()
            ),
            Self::ActorRitualDc => "spellcasting.ritual.dc".to_string(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum SaveKind {
    Fortitude,
    Reflex,
    Will,
}

impl SaveKind {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Fortitude => "fort",
            Self::Reflex => "ref",
            Self::Will => "will",
        }
    }

    pub(crate) const fn label(self) -> &'static str {
        match self {
            Self::Fortitude => "Fortitude",
            Self::Reflex => "Reflex",
            Self::Will => "Will",
        }
    }
}

impl From<CreatureSaveKind> for SaveKind {
    fn from(value: CreatureSaveKind) -> Self {
        match value {
            CreatureSaveKind::Fortitude => Self::Fortitude,
            CreatureSaveKind::Reflex => Self::Reflex,
            CreatureSaveKind::Will => Self::Will,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum AbilityKind {
    Strength,
    Dexterity,
    Constitution,
    Intelligence,
    Wisdom,
    Charisma,
}

impl AbilityKind {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Strength => "str",
            Self::Dexterity => "dex",
            Self::Constitution => "con",
            Self::Intelligence => "int",
            Self::Wisdom => "wis",
            Self::Charisma => "cha",
        }
    }

    pub(crate) const fn label(self) -> &'static str {
        match self {
            Self::Strength => "Strength",
            Self::Dexterity => "Dexterity",
            Self::Constitution => "Constitution",
            Self::Intelligence => "Intelligence",
            Self::Wisdom => "Wisdom",
            Self::Charisma => "Charisma",
        }
    }
}
