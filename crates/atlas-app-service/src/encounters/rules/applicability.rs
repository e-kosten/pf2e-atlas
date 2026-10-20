//! Picker and mutation applicability share this policy.
use super::super::conditions::ModeledCondition;
use atlas_app_model::{EncounterConditionApplicabilityView, EncounterConditionEligibilityView};
use atlas_local_state::{EncounterParticipant, ParticipantKind};

#[derive(Debug, Clone)]
pub(in crate::encounters) struct ConditionImmunity {
    pub slug: String,
    pub exceptions: Option<Vec<String>>,
}
#[derive(Debug, Clone, Default)]
pub(in crate::encounters) struct ConditionApplicabilityFacts {
    pub immunities: Option<Vec<ConditionImmunity>>,
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
    let Some(immunities) = &facts.immunities else {
        return Some(EncounterConditionEligibilityView::NeedsContext {
            reason: "Condition immunities are unavailable.".into(),
        });
    };
    if let Some(immunity) = immunities
        .iter()
        .find(|immunity| immunity.slug == condition.slug)
    {
        if immunity
            .exceptions
            .as_ref()
            .is_none_or(|exceptions| !exceptions.is_empty())
        {
            return Some(EncounterConditionEligibilityView::NeedsContext {
                reason: "Confirm whether the condition immunity's exceptions apply.".into(),
            });
        }
        return Some(EncounterConditionEligibilityView::Immune {
            reason: format!("Immune to {}.", condition.name.to_ascii_lowercase()),
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
