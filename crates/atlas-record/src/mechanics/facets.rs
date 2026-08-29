use crate::{CreatureRollKind, CreatureSkillKind};

use super::{AbilityKind, SaveKind};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MechanicFacets {
    pub family: MechanicSourceFamily,
    pub surface: MechanicSurface,
    pub statistic: Option<MechanicStatistic>,
    pub ability: Option<AbilityKind>,
}

impl MechanicFacets {
    pub const fn armor_class() -> Self {
        Self {
            family: MechanicSourceFamily::Defense,
            surface: MechanicSurface::ArmorClass,
            statistic: None,
            ability: Some(AbilityKind::Dexterity),
        }
    }

    pub const fn hit_points() -> Self {
        Self {
            family: MechanicSourceFamily::Defense,
            surface: MechanicSurface::HitPoints,
            statistic: None,
            ability: None,
        }
    }

    pub const fn perception() -> Self {
        Self {
            family: MechanicSourceFamily::Awareness,
            surface: MechanicSurface::Check,
            statistic: Some(MechanicStatistic::Perception),
            ability: Some(AbilityKind::Wisdom),
        }
    }

    pub const fn saving_throw(save: SaveKind) -> Self {
        let ability = match save {
            SaveKind::Fortitude => AbilityKind::Constitution,
            SaveKind::Reflex => AbilityKind::Dexterity,
            SaveKind::Will => AbilityKind::Wisdom,
        };
        Self {
            family: MechanicSourceFamily::Defense,
            surface: MechanicSurface::SavingThrow,
            statistic: Some(MechanicStatistic::Save(save)),
            ability: Some(ability),
        }
    }

    pub fn sparse_skill(slug: &str) -> Self {
        Self {
            family: MechanicSourceFamily::Skill,
            surface: MechanicSurface::Check,
            statistic: None,
            ability: sparse_skill_ability(slug),
        }
    }

    pub const fn creature_skill(kind: CreatureSkillKind) -> Self {
        Self {
            family: MechanicSourceFamily::Skill,
            surface: MechanicSurface::Check,
            statistic: Some(MechanicStatistic::Skill(kind)),
            ability: creature_skill_ability(kind),
        }
    }

    pub const fn ability_modifier(ability: AbilityKind) -> Self {
        Self {
            family: MechanicSourceFamily::Ability,
            surface: MechanicSurface::RawModifier,
            statistic: Some(MechanicStatistic::Ability(ability)),
            ability: Some(ability),
        }
    }

    pub const fn movement() -> Self {
        Self {
            family: MechanicSourceFamily::Movement,
            surface: MechanicSurface::Movement,
            statistic: None,
            ability: None,
        }
    }

    pub const fn resource() -> Self {
        Self {
            family: MechanicSourceFamily::Resource,
            surface: MechanicSurface::Resource,
            statistic: None,
            ability: None,
        }
    }

    pub const fn action_economy(family: MechanicSourceFamily) -> Self {
        Self {
            family,
            surface: MechanicSurface::ActionEconomy,
            statistic: None,
            ability: None,
        }
    }

    pub const fn frequency(family: MechanicSourceFamily) -> Self {
        Self {
            family,
            surface: MechanicSurface::Frequency,
            statistic: None,
            ability: None,
        }
    }

    pub const fn uses(family: MechanicSourceFamily) -> Self {
        Self {
            family,
            surface: MechanicSurface::Uses,
            statistic: None,
            ability: None,
        }
    }

    pub const fn roll(family: MechanicSourceFamily, kind: CreatureRollKind) -> Self {
        let surface = match kind {
            CreatureRollKind::Attack => MechanicSurface::AttackRoll,
            CreatureRollKind::DifficultyClass => MechanicSurface::Dc,
            CreatureRollKind::Check => MechanicSurface::Check,
        };
        Self {
            family,
            surface,
            statistic: None,
            ability: None,
        }
    }

    pub const fn damage(family: MechanicSourceFamily) -> Self {
        Self {
            family,
            surface: MechanicSurface::Damage,
            statistic: None,
            ability: None,
        }
    }

    pub const fn spellcasting_attack() -> Self {
        Self {
            family: MechanicSourceFamily::Spellcasting,
            surface: MechanicSurface::AttackRoll,
            statistic: None,
            ability: None,
        }
    }

    pub const fn spellcasting_dc() -> Self {
        Self {
            family: MechanicSourceFamily::Spellcasting,
            surface: MechanicSurface::Dc,
            statistic: None,
            ability: None,
        }
    }

    pub const fn spell_slot() -> Self {
        Self {
            family: MechanicSourceFamily::Spellcasting,
            surface: MechanicSurface::SpellSlot,
            statistic: None,
            ability: None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum MechanicSourceFamily {
    Defense,
    Awareness,
    Skill,
    Ability,
    Movement,
    Resource,
    Strike,
    Spellcasting,
    Spell,
    Action,
    Unsupported,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum MechanicSurface {
    RawModifier,
    Check,
    Dc,
    ArmorClass,
    SavingThrow,
    AttackRoll,
    Damage,
    HitPoints,
    Movement,
    Resource,
    ActionEconomy,
    Frequency,
    Uses,
    SpellSlot,
}

impl MechanicSurface {
    pub const fn is_check_or_dc(self) -> bool {
        matches!(
            self,
            Self::Check | Self::Dc | Self::ArmorClass | Self::SavingThrow | Self::AttackRoll
        )
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum MechanicStatistic {
    Ability(AbilityKind),
    Perception,
    Save(SaveKind),
    Skill(CreatureSkillKind),
}

const fn creature_skill_ability(kind: CreatureSkillKind) -> Option<AbilityKind> {
    match kind {
        CreatureSkillKind::Acrobatics
        | CreatureSkillKind::Stealth
        | CreatureSkillKind::Thievery => Some(AbilityKind::Dexterity),
        CreatureSkillKind::Athletics => Some(AbilityKind::Strength),
        CreatureSkillKind::Arcana
        | CreatureSkillKind::Crafting
        | CreatureSkillKind::Occultism
        | CreatureSkillKind::Society => Some(AbilityKind::Intelligence),
        CreatureSkillKind::Medicine
        | CreatureSkillKind::Nature
        | CreatureSkillKind::Religion
        | CreatureSkillKind::Survival => Some(AbilityKind::Wisdom),
        CreatureSkillKind::Deception
        | CreatureSkillKind::Diplomacy
        | CreatureSkillKind::Intimidation
        | CreatureSkillKind::Performance => Some(AbilityKind::Charisma),
        CreatureSkillKind::Lore => None,
    }
}

fn sparse_skill_ability(slug: &str) -> Option<AbilityKind> {
    match slug {
        "acr" | "acrobatics" | "ste" | "stealth" | "thi" | "thievery" => {
            Some(AbilityKind::Dexterity)
        }
        "ath" | "athletics" => Some(AbilityKind::Strength),
        "arc" | "arcana" | "cra" | "crafting" | "occ" | "occultism" | "soc" | "society" => {
            Some(AbilityKind::Intelligence)
        }
        "med" | "medicine" | "nat" | "nature" | "rel" | "religion" | "sur" | "survival" => {
            Some(AbilityKind::Wisdom)
        }
        "dec" | "deception" | "dip" | "diplomacy" | "itm" | "intimidation" | "prf"
        | "performance" => Some(AbilityKind::Charisma),
        _ => None,
    }
}
