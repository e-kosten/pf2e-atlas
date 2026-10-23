//! Runtime overlays on authored typed statistics; unknown inputs stay unknown.
pub(crate) use super::stat_block::*;
use atlas_local_state::ParticipantVariant;
use atlas_record::source_record::{
    SourceBackedRecord, SourceFieldView, SourceNpcAdjustment, SourceQueryView,
};
use serde_json::Number;

#[cfg(test)]
thread_local! {
    static BASELINES: std::cell::Cell<usize> = const { std::cell::Cell::new(0) };
}
#[cfg(test)]
pub(super) fn baseline_count(reset: bool) -> usize {
    BASELINES.with(|count| if reset { count.replace(0) } else { count.get() })
}

#[cfg(test)]
pub(super) fn record_baseline() {
    BASELINES.with(|count| count.set(count.get() + 1));
}

pub(super) fn authored_variant(record: &SourceBackedRecord) -> Option<ParticipantVariant> {
    SourceQueryView::new(record.source(), record.key().pack().as_str(), "")
        .actor()
        .authored_adjustment()
        .value()
        .map(|v| match v {
            SourceNpcAdjustment::Normal => ParticipantVariant::Normal,
            SourceNpcAdjustment::Elite => ParticipantVariant::Elite,
            SourceNpcAdjustment::Weak => ParticipantVariant::Weak,
        })
}
pub(super) fn initial_variant(
    record: &SourceBackedRecord,
) -> (
    Option<ParticipantVariant>,
    atlas_local_state::ParticipantVariantOrigin,
) {
    use atlas_local_state::ParticipantVariantOrigin;
    if let Some(variant) = authored_variant(record) {
        return (Some(variant), ParticipantVariantOrigin::InheritedKnown);
    }
    if SourceQueryView::new(record.source(), record.key().pack().as_str(), "")
        .actor()
        .npc_adjustment_is_unset()
    {
        (
            Some(ParticipantVariant::Normal),
            ParticipantVariantOrigin::DefaultUnadjusted,
        )
    } else {
        (None, ParticipantVariantOrigin::InheritedUnknown)
    }
}
pub(super) fn integer(value: &Number) -> Option<i64> {
    if let Some(i) = value.as_i64() {
        return Some(i);
    }
    if value.is_u64() {
        return None;
    }
    let n = value.as_f64()?;
    (n.is_finite() && n.fract() == 0.0 && n >= i64::MIN as f64 && n < 9223372036854775808.0)
        .then_some(n as i64)
}
pub(super) fn hp_delta(variant: ParticipantVariant, level: i64) -> i64 {
    match variant {
        ParticipantVariant::Normal => 0,
        ParticipantVariant::Elite => {
            if level >= 20 {
                30
            } else if level >= 5 {
                20
            } else if level >= 2 {
                15
            } else {
                10
            }
        }
        ParticipantVariant::Weak => {
            -if level >= 21 {
                30
            } else if level >= 6 {
                20
            } else if level >= 3 {
                15
            } else if level == 1 || level == 2 {
                10
            } else {
                0
            }
        }
    }
}
pub(super) fn adjusted_npc_level(level: i64, variant: ParticipantVariant) -> i64 {
    let base = level.clamp(-1, 100);
    match variant {
        ParticipantVariant::Normal => base,
        ParticipantVariant::Elite => base + if base < 1 { 2 } else { 1 },
        ParticipantVariant::Weak => base - if base == 1 { 2 } else { 1 },
    }
}
pub(super) fn effective_max(
    record: &SourceBackedRecord,
    variant: Option<ParticipantVariant>,
) -> Option<i64> {
    let actor = SourceQueryView::new(record.source(), record.key().pack().as_str(), "").actor();
    let hp = integer(actor.hp_maximum().value()?)?;
    if matches!(actor.authored_adjustment(), SourceFieldView::NotApplicable) {
        return Some(hp);
    }
    let variant = variant?;
    if variant == ParticipantVariant::Normal {
        return Some(hp);
    }
    let level = integer(actor.level().value()?)?;
    hp.checked_add(hp_delta(variant, level))
}
