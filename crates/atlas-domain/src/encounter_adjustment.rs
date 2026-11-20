//! Stable selectors for encounter-owned GM modifiers.

#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum AdjustmentTarget {
    ArmorClass,
    Fortitude,
    Reflex,
    Will,
    Perception,
    Skill { key: String },
    Speed { mode: String },
    StrikeAttacks,
    SpellAttacks,
    SpellDcs,
}

impl AdjustmentTarget {
    pub fn is_supported(&self) -> bool {
        match self {
            Self::Skill { key } => matches!(
                key.as_str(),
                "acrobatics"
                    | "arcana"
                    | "athletics"
                    | "crafting"
                    | "deception"
                    | "diplomacy"
                    | "intimidation"
                    | "medicine"
                    | "nature"
                    | "occultism"
                    | "performance"
                    | "religion"
                    | "society"
                    | "stealth"
                    | "survival"
                    | "thievery"
            ),
            Self::Speed { mode } => {
                matches!(mode.as_str(), "land" | "burrow" | "climb" | "fly" | "swim")
            }
            _ => true,
        }
    }
}

#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AdjustmentModifierType {
    Stacks,
    Item,
    Status,
    Circumstance,
}
