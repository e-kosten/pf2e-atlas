//! Picker and mutation applicability share this policy.
use super::super::conditions::ModeledCondition;
use atlas_app_model::{EncounterConditionApplicabilityView, EncounterConditionEligibilityView};
use atlas_local_state::{EncounterParticipant, ParticipantKind};

#[derive(Debug, Clone)]
pub(in crate::encounters) struct ConditionImmunity {
    pub slug: String,
    pub coverage: ImmunityCoverage,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(in crate::encounters) enum ImmunityCoverage {
    Unconditional,
    Conditional,
    Unknown,
}

#[derive(Debug, Clone, Default)]
pub(in crate::encounters) struct ConditionImmunityFacts {
    pub entries: Vec<ConditionImmunity>,
    pub authored_complete: bool,
    pub uncertain_types: Vec<&'static str>,
}
impl From<Vec<ConditionImmunity>> for ConditionImmunityFacts {
    fn from(entries: Vec<ConditionImmunity>) -> Self {
        Self {
            entries,
            authored_complete: true,
            uncertain_types: vec![],
        }
    }
}

#[derive(Debug, Clone, Default)]
pub(in crate::encounters) struct ConditionApplicabilityFacts {
    pub immunities: ConditionImmunityFacts,
    pub broken_threshold: Option<serde_json::Number>,
    pub broken_threshold_known_absent: bool,
}

pub(in crate::encounters) fn condition_applicability(
    condition: ModeledCondition,
    participant: &EncounterParticipant,
    facts: &ConditionApplicabilityFacts,
) -> Option<EncounterConditionEligibilityView> {
    let kind = match participant.participant_kind {
        ParticipantKind::Hazard => EncounterConditionApplicabilityView::Hazard,
        ParticipantKind::Creature | ParticipantKind::Pc => {
            EncounterConditionApplicabilityView::Creature
        }
    };
    if !condition.applies_to.contains(&kind) {
        return None;
    }
    if condition.slug == "broken"
        && (facts.broken_threshold_known_absent
            || participant.conditions.iter().any(|condition| {
                super::super::conditions::condition_rule_for_key(condition.condition_key.as_deref())
                    == Some(super::super::conditions::ConditionRule::Petrified)
            }))
    {
        return Some(EncounterConditionEligibilityView::Immune {
            reason: "No Broken Threshold; this object cannot become broken.".into(),
        });
    }
    if condition.slug == "broken" && facts.broken_threshold.is_none() {
        return Some(EncounterConditionEligibilityView::NeedsContext {
            reason: "Broken Threshold is unavailable.".into(),
        });
    }
    if facts.immunities.entries.iter().any(|immunity| {
        immunity.slug == condition.slug && immunity.coverage == ImmunityCoverage::Unconditional
    }) {
        return Some(EncounterConditionEligibilityView::Immune {
            reason: format!("Immune to {}.", condition.name.to_ascii_lowercase()),
        });
    }
    if facts
        .immunities
        .entries
        .iter()
        .any(|immunity| immunity.slug == condition.slug)
    {
        return Some(EncounterConditionEligibilityView::NeedsContext {
            reason: "Confirm whether the condition immunity's exceptions apply.".into(),
        });
    }
    if !facts.immunities.authored_complete
        || facts.immunities.uncertain_types.contains(&condition.slug)
    {
        return Some(EncounterConditionEligibilityView::NeedsContext {
            reason: "Condition immunities are unavailable.".into(),
        });
    }
    if participant.participant_kind == ParticipantKind::Pc
        && matches!(
            condition.slug,
            "helpful" | "friendly" | "indifferent" | "unfriendly" | "hostile"
        )
    {
        return Some(EncounterConditionEligibilityView::NeedsContext {
            reason: "Only a supernatural effect can impose this attitude on a PC.".into(),
        });
    }
    Some(EncounterConditionEligibilityView::Available)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn unconditional_immunity_proves_ineligibility_despite_a_conditional_duplicate() {
        let condition = crate::encounters::conditions::modeled_condition_by_rule(
            crate::encounters::conditions::ConditionRule::Prone,
        )
        .unwrap();
        let facts = ConditionApplicabilityFacts {
            immunities: vec![
                ConditionImmunity {
                    slug: "prone".into(),
                    coverage: ImmunityCoverage::Conditional,
                },
                ConditionImmunity {
                    slug: "prone".into(),
                    coverage: ImmunityCoverage::Unconditional,
                },
            ]
            .into(),
            ..Default::default()
        };
        assert!(matches!(
            condition_applicability(
                condition,
                &super::super::numerical_tests::participant(),
                &facts
            ),
            Some(EncounterConditionEligibilityView::Immune { .. })
        ));
    }
}
