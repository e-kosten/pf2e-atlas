//! Explicit sensory check context; unknown ranges and source statistics stay unknown.
use super::super::{conditions::ConditionRule, stat_block::StatBlockView};
use atlas_local_state::EncounterParticipant;

pub(in crate::encounters) struct DazzledConcealment {
    pub applies: Option<bool>,
    pub source_ids: Vec<super::causes::ConditionSourceKey>,
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
    let causes = super::causes::condition_causes(
        observer,
        Some(other),
        baseline
            .map(|baseline| &baseline.source_conditions)
            .unwrap_or_else(|| super::source_conditions::SourceConditions::empty()),
    );
    let source_ids = causes
        .iter()
        .filter(|cause| cause.rule == ConditionRule::Dazzled)
        .map(|cause| cause.source.key())
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
