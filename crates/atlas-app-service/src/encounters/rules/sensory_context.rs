//! Explicit sensory check context; unknown ranges and source statistics stay unknown.
use super::super::{
    conditions::ConditionRule,
    stat_block::{StatBlockView, StatisticTarget},
};
use atlas_local_state::EncounterParticipant;
use serde_json::Number;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(in crate::encounters) enum PerceptionBasis {
    General,
    VisualOnly,
    AuditoryOnly,
    MixedSound,
    Initiative,
}
#[derive(Debug, Clone, Copy)]
pub(in crate::encounters) struct SensoryContext {
    pub basis: PerceptionBasis,
    pub distance_feet: Option<f64>,
    pub requires_movement: Option<bool>,
}
pub(in crate::encounters) struct SensoryRequirements {
    pub perception: Option<Number>,
    pub critical_failure: bool,
    pub visual_effects_immune: bool,
    pub auditory_effects_immune: bool,
    pub all_terrain_difficult: bool,
    pub required_facts: Vec<String>,
}
pub(in crate::encounters) fn evaluate_senses(
    participant: &EncounterParticipant,
    baseline: Option<(&StatBlockView, bool)>,
    context: SensoryContext,
    other: Option<&str>,
) -> SensoryRequirements {
    let causes = super::causes::condition_causes(participant, other);
    let blinded = causes
        .iter()
        .any(|cause| cause.rule == ConditionRule::Blinded);
    let deafened = causes
        .iter()
        .any(|cause| cause.rule == ConditionRule::Deafened);
    let paralyzed = causes
        .iter()
        .any(|cause| cause.rule == ConditionRule::Paralyzed);
    let petrified = causes
        .iter()
        .any(|cause| cause.rule == ConditionRule::Petrified);
    let critical_failure = (blinded && context.basis == PerceptionBasis::VisualOnly)
        || (deafened && context.basis == PerceptionBasis::AuditoryOnly);
    let mut result = SensoryRequirements {
        perception: None,
        critical_failure,
        visual_effects_immune: blinded,
        auditory_effects_immune: deafened,
        all_terrain_difficult: blinded,
        required_facts: vec![],
    };
    if critical_failure || petrified {
        return result;
    }
    if paralyzed {
        match context.requires_movement {
            Some(false) => {}
            Some(true) => {
                result
                    .required_facts
                    .push("This sense requires movement that Paralyzed prevents.".into());
                return result;
            }
            None => {
                result
                    .required_facts
                    .push("Confirm whether this sensory check requires moving your body.".into());
                return result;
            }
        }
    }
    let Some((baseline, adjustment_applicable)) = baseline else {
        result
            .required_facts
            .push("Perception statistics are unavailable.".into());
        return result;
    };
    let mut contextual_baseline = baseline.clone();
    if let Some(distance) = context.distance_feet {
        for sense in &mut contextual_baseline.sense_facts {
            if let Some(range) = sense.range_feet {
                if distance > range {
                    sense.available = Some(false);
                } else {
                    sense.range_feet = None;
                }
            }
        }
    }
    let mut projected = if let Some(other) = other {
        super::project_participant_for_opponent(
            participant,
            &contextual_baseline,
            adjustment_applicable,
            &[],
            other,
        )
    } else {
        super::project_participant(
            participant,
            &contextual_baseline,
            adjustment_applicable,
            &[],
        )
    };
    if matches!(
        context.basis,
        PerceptionBasis::MixedSound | PerceptionBasis::Initiative
    ) {
        projected = super::project_initiative(projected, participant);
    }
    result.perception = projected
        .effective_stat(&StatisticTarget::Perception)
        .cloned();
    if result.perception.is_none() {
        result.required_facts.push(if blinded {
            "Confirm whether an available precise nonvisual sense reaches the subject.".into()
        } else {
            "Perception statistics are unavailable.".into()
        });
    }
    result
}

pub(in crate::encounters) struct DazzledConcealment {
    pub applies: Option<bool>,
    pub source_ids: Vec<i64>,
}
/// Range can rule an alternative out; being within range alone does not establish
/// echolocation or another special sense's remaining target requirements.
pub(in crate::encounters) fn dazzled_concealment(
    observer: &EncounterParticipant,
    other: &str,
    baseline: Option<&StatBlockView>,
    distance: Option<f64>,
    vision_only: Option<bool>,
) -> DazzledConcealment {
    let causes = super::causes::condition_causes(observer, Some(other));
    let source_ids = causes
        .iter()
        .filter(|cause| cause.rule == ConditionRule::Dazzled)
        .map(|cause| cause.source.condition_id)
        .collect::<Vec<_>>();
    if source_ids.is_empty() {
        return DazzledConcealment {
            applies: Some(false),
            source_ids,
        };
    }
    let deafened = causes
        .iter()
        .any(|cause| cause.rule == ConditionRule::Deafened);
    let established = baseline.and_then(|baseline| {
        if !baseline.senses_known {
            return None;
        }
        let vision = baseline.sense_facts.iter().filter(|sense| {
            sense.visual && sense.precision == Some(super::senses::Precision::Precise)
        }).collect::<Vec<_>>();
        if !vision.is_empty() && vision.iter().all(|sense| sense.available == Some(false)) {
            return Some(false);
        }
        if !vision.iter().any(|sense| sense.available == Some(true)) {
            return None;
        }
        let alternative = baseline.sense_facts.iter().any(|sense| {
            !(sense.visual
                || (deafened && sense.auditory)
                || sense.available == Some(false)
                || matches!(sense.precision, Some(super::senses::Precision::Imprecise | super::senses::Precision::Vague))
                || matches!((sense.range_feet, distance), (Some(range), Some(distance)) if distance > range))
        });
        (!alternative).then_some(true)
    });
    DazzledConcealment {
        applies: established.or(vision_only),
        source_ids,
    }
}
