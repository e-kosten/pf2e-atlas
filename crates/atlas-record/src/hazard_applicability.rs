//! Computed presentation policy; never part of the persisted hazard record or search projection.
use crate::HazardDefenses;

pub const HAZARD_APPLICABILITY_RULE_ID: &str = "pf2e-hazard-structural-applicability";
pub const HAZARD_APPLICABILITY_RULE_VERSION: u32 = 1;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HazardApplicabilityState {
    Applicable,
    Inapplicable,
    Unknown,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HazardDefenseApplicability {
    pub health: HazardApplicabilityState,
    pub structure: HazardApplicabilityState,
    pub rule_id: &'static str,
    pub rule_version: u32,
}

/// Pinned PF2e health uses positive HP maximum; structural defenses use HP maximum OR AC.
/// Missing, null, unsupported and invalid HP maximum never become an authored zero.
/// AC has no source minimum: any supported nonzero value proves structural applicability.
pub fn project_hazard_defense_applicability(
    defenses: &HazardDefenses,
) -> HazardDefenseApplicability {
    project_values(
        defenses
            .hit_points
            .typed()
            .and_then(|hp| hp.maximum.typed())
            .copied(),
        defenses.armor_class.typed().copied(),
    )
}

fn project_values(maximum: Option<i64>, armor_class: Option<i64>) -> HazardDefenseApplicability {
    use HazardApplicabilityState::{Applicable, Inapplicable, Unknown};
    let health = match maximum {
        Some(0) => Inapplicable,
        Some(value) if value > 0 => Applicable,
        _ => Unknown,
    };
    let structure = match (health, armor_class) {
        (Applicable, _) => Applicable,
        (_, Some(value)) if value != 0 => Applicable,
        (Inapplicable, Some(0)) => Inapplicable,
        _ => Unknown,
    };
    HazardDefenseApplicability {
        health,
        structure,
        rule_id: HAZARD_APPLICABILITY_RULE_ID,
        rule_version: HAZARD_APPLICABILITY_RULE_VERSION,
    }
}

#[cfg(test)]
mod tests {
    use super::{HazardApplicabilityState::*, project_values};

    #[test]
    fn only_two_known_zeros_prove_structural_inapplicability() {
        for (hp, ac, health, structure) in [
            (Some(0), Some(0), Inapplicable, Inapplicable),
            (Some(12), Some(0), Applicable, Applicable),
            (Some(0), Some(22), Inapplicable, Applicable),
            (Some(0), Some(-1), Inapplicable, Applicable),
            (None, Some(22), Unknown, Applicable),
            (Some(12), None, Applicable, Applicable),
            (None, Some(0), Unknown, Unknown),
            (Some(0), None, Inapplicable, Unknown),
            (None, None, Unknown, Unknown),
            (Some(-1), Some(0), Unknown, Unknown),
            (Some(i64::MAX), Some(0), Applicable, Applicable),
        ] {
            let result = project_values(hp, ac);
            assert_eq!(
                (result.health, result.structure),
                (health, structure),
                "{hp:?}, {ac:?}"
            );
        }
    }
}
