//! Authored feat/effect reading facts, without character or encounter execution.
use super::*;
use atlas_foundry_model::generated::{
    SourceFromSchemaDurationDataSchemaExpiry, SourceFromSchemaDurationDataSchemaUnit,
};

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
                        SourceFromSchemaDurationDataSchemaUnit::Encounter => {
                            SourceFieldView::Value("Until the encounter ends".into())
                        }
                        SourceFromSchemaDurationDataSchemaUnit::Unlimited => {
                            SourceFieldView::Value("Unlimited".into())
                        }
                        unit => SourceFieldView::from(&duration.value).and_then(|amount| {
                            if !amount.as_f64().is_some_and(|n| n >= 0.0) {
                                return SourceFieldView::ProjectionInvalid {
                                    source_path: "/system/duration/value",
                                    reason: "Timed duration must be nonnegative",
                                };
                            }
                            let singular = amount.as_f64() == Some(1.0);
                            let label = match unit {
                                SourceFromSchemaDurationDataSchemaUnit::Rounds => "round",
                                SourceFromSchemaDurationDataSchemaUnit::Minutes => "minute",
                                SourceFromSchemaDurationDataSchemaUnit::Hours => "hour",
                                SourceFromSchemaDurationDataSchemaUnit::Days => "day",
                                SourceFromSchemaDurationDataSchemaUnit::Encounter => "encounter",
                                SourceFromSchemaDurationDataSchemaUnit::Unlimited => "unlimited",
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
                    // PF2e 8.6.1 AbstractEffectPF2e.totalDuration makes these units unlimited.
                    match SourceFieldView::from(&duration.unit) {
                        SourceFieldView::Value(
                            SourceFromSchemaDurationDataSchemaUnit::Encounter
                            | SourceFromSchemaDurationDataSchemaUnit::Unlimited,
                        ) => SourceFieldView::NotApplicable,
                        _ => match SourceFieldView::from(&duration.expiry) {
                            SourceFieldView::Null => SourceFieldView::NotApplicable,
                            other => other,
                        },
                    }
                    .map(|expiry| {
                        match expiry {
                            SourceFromSchemaDurationDataSchemaExpiry::TurnStart => {
                                "At the start of the turn"
                            }
                            SourceFromSchemaDurationDataSchemaExpiry::TurnEnd => {
                                "At the end of the turn"
                            }
                            SourceFromSchemaDurationDataSchemaExpiry::RoundEnd => {
                                "At the end of the round"
                            }
                        }
                        .into()
                    }),
                ),
            }),
    )
}

pub(super) fn consumable(item: ItemSourceView<'_>) -> Option<ConsumablePresentationView> {
    let ItemSourceView::ConsumableSource(source) = item else {
        return None;
    };
    let system = SourceFieldView::from(&source.system);
    let uses = system.and_then(|s| (&s.uses).into());
    Some(ConsumablePresentationView {
        maximum_hp: number(
            system
                .and_then(|s| (&s.hp).into())
                .and_then(|hp| (&hp.max).into()),
        ),
        hardness: number(system.and_then(|s| (&s.hardness).into())),
        auto_destroy: fact(uses.and_then(|u| (&u.auto_destroy).into()).map(|v| *v)),
        spell: unavailable(),
        spell_presented_inline: false,
        category: identifier(item.consumable_category()),
        damage: fact(item.consumable_damage().map(|damage| ConsumableDamageView {
            formula: text((&damage.formula).into()),
            kind: identifier(item.consumable_damage_kind()),
            damage_type: identifier(item.consumable_damage_type()),
        })),
        maximum_uses: number(uses.and_then(|u| (&u.max).into())),
        source_uses: number(uses.and_then(|u| (&u.value).into())),
    })
}
