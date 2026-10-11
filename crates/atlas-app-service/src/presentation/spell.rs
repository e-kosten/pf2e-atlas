use super::*;
use atlas_foundry_model::generated::*;

pub(super) fn authored_spell(source: &SpellSource) -> SpellPresentationView {
    let s = SourceFieldView::from(&source.system);
    SpellPresentationView {
        rank: number(
            s.and_then(|s| (&s.level).into())
                .and_then(|v| (&v.value).into()),
        ),
        traditions: identifiers(
            s.and_then(|s| (&s.traits).into())
                .and_then(|t| (&t.traditions).into()),
        ),
        cast: fact(
            s.and_then(|s| (&s.time).into())
                .and_then(|v| (&v.value).into())
                .map(|v| cast_label(v)),
        ),
        requirements: text(s.and_then(|s| (&s.requirements).into())),
        cost: text(
            s.and_then(|s| (&s.cost).into())
                .and_then(|v| (&v.value).into()),
        ),
        range: text(
            s.and_then(|s| (&s.range).into())
                .and_then(|v| (&v.value).into()),
        ),
        target: text(
            s.and_then(|s| (&s.target).into())
                .and_then(|v| (&v.value).into()),
        ),
        area: fact(s.and_then(|s| (&s.area).into()).map(area)),
        defense: fact(s.and_then(|s| (&s.defense).into()).map(defense)),
        duration: text(
            s.and_then(|s| (&s.duration).into())
                .and_then(|v| (&v.value).into()),
        ),
        sustained: fact(
            s.and_then(|s| (&s.duration).into())
                .and_then(|v| (&v.sustained).into())
                .map(|v| *v),
        ),
        damage: damage(s.and_then(|s| (&s.damage).into())),
        heightening: fact(s.and_then(|s| (&s.heightening).into()).map(heightening)),
        forms: fact(s.and_then(|s| (&s.overlays).into()).map(|o| {
            o.entries
                .iter()
                .map(|(id, v)| SpellFormView {
                    id: id.clone(),
                    name: text((&v.name).into()),
                    sort: number((&v.sort).into()),
                    changes: fact(SourceFieldView::from(&v.system).map(changes)),
                })
                .collect()
        })),
        casting_entry: text(
            s.and_then(|s| (&s.location).into())
                .and_then(|v| (&v.value).into()),
        ),
        authored_cast_rank: number(
            s.and_then(|s| (&s.location).into())
                .and_then(|v| (&v.heightened_level).into()),
        ),
    }
}
fn area(a: &SpellArea) -> SpellAreaView {
    SpellAreaView {
        shape: identifier((&a.r#type).into()),
        size: fact(SourceFieldView::from(&a.value).map(|v| match v {
            StringOrNumber::String(s) => format!("{s} feet"),
            StringOrNumber::Number(n) => format!("{n} feet"),
        })),
        details: text((&a.details).into()),
    }
}
fn defense(d: &SpellDefenseSource) -> SpellDefenseView {
    SpellDefenseView {
        statistic: identifier(SourceFieldView::from(&d.save).and_then(|v| (&v.statistic).into())),
        basic: fact(
            SourceFieldView::from(&d.save)
                .and_then(|v| (&v.basic).into())
                .map(|v| *v),
        ),
        passive: identifier(SourceFieldView::from(&d.passive).and_then(|v| (&v.statistic).into())),
    }
}
pub(super) fn damage(
    v: SourceFieldView<'_, &SpellSystemSourceDamage>,
) -> FactView<Vec<DamageComponentView>> {
    fact(v.map(|d| {
        d.entries
            .iter()
            .map(|(id, c)| DamageComponentView {
                id: id.clone(),
                formula: text((&c.formula).into()),
                damage_type: identifier((&c.r#type).into()),
                kinds: identifiers((&c.kinds).into()),
                category: identifier((&c.category).into()),
                materials: identifiers((&c.materials).into()),
                apply_modifier: fact(SourceFieldView::from(&c.apply_mod).map(|v| *v)),
            })
            .collect()
    }))
}
fn heightening(h: &SpellSystemSourceHeightening) -> SpellHeighteningView {
    match h {
        SpellSystemSourceHeightening::SpellHeighteningInterval(v) => {
            SpellHeighteningView::Interval {
                interval: number((&v.interval).into()),
                area: Box::new(number((&v.area).into())),
                damage: fact(SourceFieldView::from(&v.damage).map(|d| {
                    d.entries
                        .iter()
                        .map(|(id, f)| HeighteningDamageView {
                            id: id.clone(),
                            formula: FactView {
                                state: QueryFieldState::Value,
                                value: Some(f.clone()),
                            },
                        })
                        .collect()
                })),
            }
        }
        SpellSystemSourceHeightening::SpellHeighteningFixed(v) => SpellHeighteningView::Fixed {
            levels: fact(SourceFieldView::from(&v.levels).map(|l| {
                [
                    (1, &l._1),
                    (2, &l._2),
                    (3, &l._3),
                    (4, &l._4),
                    (5, &l._5),
                    (6, &l._6),
                    (7, &l._7),
                    (8, &l._8),
                    (9, &l._9),
                    (10, &l._10),
                ]
                .into_iter()
                .filter(|(_, v)| !matches!(v, atlas_foundry_model::SourcePresence::Missing))
                .map(|(rank, v)| SpellFixedLevelView {
                    rank,
                    changes: fact(SourceFieldView::from(v).map(changes)),
                })
                .collect()
            })),
        },
    }
}
macro_rules! changed_facts {
    ($s:ident,$heightening:expr) => {
        SpellChangeView {
            cast: fact(
                SourceFieldView::from(&$s.time)
                    .and_then(|v| (&v.value).into())
                    .map(|v| cast_label(v)),
            ),
            range: text(SourceFieldView::from(&$s.range).and_then(|v| (&v.value).into())),
            target: text(SourceFieldView::from(&$s.target).and_then(|v| (&v.value).into())),
            area: fact(SourceFieldView::from(&$s.area).map(area)),
            defense: fact(SourceFieldView::from(&$s.defense).map(defense)),
            duration: text(SourceFieldView::from(&$s.duration).and_then(|v| (&v.value).into())),
            sustained: fact(
                SourceFieldView::from(&$s.duration)
                    .and_then(|v| (&v.sustained).into())
                    .map(|v| *v),
            ),
            traits: fact(
                SourceFieldView::from(&$s.traits)
                    .and_then(|v| (&v.value).into())
                    .map(Clone::clone),
            ),
            traditions: identifiers(
                SourceFieldView::from(&$s.traits).and_then(|v| (&v.traditions).into()),
            ),
            damage: damage((&$s.damage).into()),
            heightening: $heightening,
        }
    };
}

fn changes(s: &PatchSpellSystemSource) -> SpellChangeView {
    changed_facts!(
        s,
        fact(SourceFieldView::from(&s.heightening).map(heightening_changes))
    )
}
fn nested_changes(s: &PatchSpellSystemSourceHeighteningLevels1) -> SpellChangeView {
    changed_facts!(
        s,
        fact(SourceFieldView::from(&s.heightening).map(|h| heightening_changes(h)))
    )
}
fn heightening_changes(h: &PatchSpellSystemSourceHeightening) -> SpellHeighteningChangeView {
    SpellHeighteningChangeView {
        kind: identifier((&h.r#type).into()),
        interval: number((&h.interval).into()),
        area: number((&h.area).into()),
        damage: fact(SourceFieldView::from(&h.damage).map(|d| {
            d.entries
                .iter()
                .map(|(id, f)| HeighteningDamageView {
                    id: id.clone(),
                    formula: FactView {
                        state: QueryFieldState::Value,
                        value: Some(f.clone()),
                    },
                })
                .collect()
        })),
        levels: fact(SourceFieldView::from(&h.levels).map(|l| {
            [
                (1, &l._1),
                (2, &l._2),
                (3, &l._3),
                (4, &l._4),
                (5, &l._5),
                (6, &l._6),
                (7, &l._7),
                (8, &l._8),
                (9, &l._9),
                (10, &l._10),
            ]
            .into_iter()
            .filter(|(_, v)| !matches!(v, atlas_foundry_model::SourcePresence::Missing))
            .map(|(rank, v)| SpellFixedLevelView {
                rank,
                changes: fact(SourceFieldView::from(v).map(|p| nested_changes(p))),
            })
            .collect()
        })),
    }
}
