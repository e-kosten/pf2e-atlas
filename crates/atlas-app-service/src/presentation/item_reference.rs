//! Authored feat/effect reading facts, without character or encounter execution.
use super::*;
use atlas_foundry_model::generated::{DurationDataExpiry, DurationDataUnit};

pub(super) fn effect_duration(item: ItemSourceView<'_>) -> FactView<EffectDurationView> {
    let ItemSourceView::EffectSource(source) = item else {
        return unavailable();
    };
    fact(
        SourceFieldView::from(&source.system)
            .and_then(|s| (&s.duration).into())
            .map(|duration| EffectDurationView {
                duration: fact(
                    SourceFieldView::from(&duration.unit).and_then(|unit| match unit {
                        DurationDataUnit::Encounter => {
                            SourceFieldView::Value("Until the encounter ends".into())
                        }
                        DurationDataUnit::Unlimited => SourceFieldView::Value("Unlimited".into()),
                        unit => SourceFieldView::from(&duration.value).and_then(|amount| {
                            if !amount.as_f64().is_some_and(|n| n >= 0.0) {
                                return SourceFieldView::ProjectionInvalid {
                                    source_path: "/system/duration/value",
                                    reason: "Timed duration must be nonnegative",
                                };
                            }
                            let singular = amount.as_f64() == Some(1.0);
                            let label = match unit {
                                DurationDataUnit::Rounds => "round",
                                DurationDataUnit::Minutes => "minute",
                                DurationDataUnit::Hours => "hour",
                                DurationDataUnit::Days => "day",
                                DurationDataUnit::Encounter => "encounter",
                                DurationDataUnit::Unlimited => "unlimited",
                            };
                            SourceFieldView::Value(format!(
                                "{amount} {label}{}",
                                if singular { "" } else { "s" }
                            ))
                        }),
                    }),
                ),
                sustained: fact(SourceFieldView::from(&duration.sustained).map(|v| *v)),
                expiry: fact(
                    // PF2e 6.12.4 EffectPF2e.prepareBaseData clears expiry for these units.
                    match SourceFieldView::from(&duration.unit) {
                        SourceFieldView::Value(
                            DurationDataUnit::Encounter | DurationDataUnit::Unlimited,
                        ) => SourceFieldView::NotApplicable,
                        _ => match SourceFieldView::from(&duration.expiry) {
                            SourceFieldView::Null => SourceFieldView::NotApplicable,
                            other => other,
                        },
                    }
                    .map(|expiry| {
                        match expiry {
                            DurationDataExpiry::TurnStart => "At the start of the turn",
                            DurationDataExpiry::TurnEnd => "At the end of the turn",
                            DurationDataExpiry::RoundEnd => "At the end of the round",
                        }
                        .into()
                    }),
                ),
            }),
    )
}
