//! Read-time heightening over the selected owned source, never a catalog replacement.
use atlas_foundry_model::{SourcePresence, generated::*};

fn replace<T: Clone>(target: &mut SourcePresence<T>, patch: &SourcePresence<T>) {
    if !matches!(patch, SourcePresence::Missing) {
        *target = patch.clone();
    }
}

fn object<T: Clone>(
    target: &mut SourcePresence<T>,
    patch: &SourcePresence<T>,
    merge: impl FnOnce(&mut T, &T),
) {
    match (target, patch) {
        (_, SourcePresence::Missing) => {}
        (SourcePresence::Value(target), SourcePresence::Value(patch)) => merge(target, patch),
        (target, patch) => *target = patch.clone(),
    }
}

macro_rules! members {
    ($target:expr, $patch:expr, $($field:ident),+ $(,)?) => {{
        $(replace(&mut $target.$field, &$patch.$field);)+
    }};
}

// Both generated patch types have these same named authored fields. Share the
// field policy directly rather than converting either patch through JSON/a shim.
macro_rules! patch_fields {
    ($system:ident, $patch:ident) => {{
        let system = &mut *$system;
        let patch = $patch;
        members!(system, patch, requirements, counteraction);
        for (target, patch) in [
            (&mut system.range, &patch.range),
            (&mut system.target, &patch.target),
            (&mut system.time, &patch.time),
            (&mut system.cost, &patch.cost),
        ] {
            object(target, patch, |a, b| members!(a, b, value));
        }
        object(&mut system.duration, &patch.duration, |a, b| {
            members!(a, b, value, sustained);
        });
        object(&mut system.area, &patch.area, |a, b| {
            members!(a, b, value, r#type, details);
        });
        object(&mut system.defense, &patch.defense, |a, b| {
            object(&mut a.save, &b.save, |a, b| {
                members!(a, b, statistic, basic)
            });
            object(&mut a.passive, &b.passive, |a, b| members!(a, b, statistic));
        });
        object(&mut system.traits, &patch.traits, |a, b| {
            members!(a, b, value, traditions, rarity, other_tags);
        });
        object(&mut system.damage, &patch.damage, |a, b| {
            for (id, component) in &b.entries {
                if let Some((_, target)) = a.entries.iter_mut().find(|(key, _)| key == id) {
                    members!(
                        target, component, formula, r#type, kinds, category, materials, apply_mod
                    );
                } else {
                    a.entries.push((id.clone(), component.clone()));
                }
            }
        });
    }};
}

fn unchanged_description(
    system: &SpellSystemSource,
    patch: &SourcePresence<ItemDescriptionSource>,
) -> bool {
    let mut description = system.description.clone();
    object(&mut description, patch, |a, b| members!(a, b, value, gm));
    description == system.description
}

fn empty_heightening_patch(patch: &SourcePresence<PatchSpellSystemSourceHeightening>) -> bool {
    match patch {
        SourcePresence::Missing => true,
        SourcePresence::Value(value) => {
            matches!(value.area, SourcePresence::Missing)
                && matches!(value.damage, SourcePresence::Missing)
                && matches!(value.interval, SourcePresence::Missing)
                && matches!(value.levels, SourcePresence::Missing)
                && matches!(value.r#type, SourcePresence::Missing)
                && value.additional_fields.fields().is_empty()
        }
        _ => false,
    }
}

fn patch_system(
    system: &mut SpellSystemSource,
    patch: &PatchSpellSystemSource,
) -> Result<(), String> {
    if !unchanged_description(system, &patch.description) {
        return Err("Heightened description changes are not supported.".into());
    }
    if !empty_heightening_patch(&patch.heightening) {
        return Err("Changes to heightening rules are not supported.".into());
    }
    patch_fields!(system, patch);
    Ok(())
}

/// A read-only casting-form source, before rank heightening. No actor rules run.
pub(super) fn casting_form(
    source: &SpellSource,
    overlay: &SpellOverlayOverride,
) -> Result<SpellSource, String> {
    if overlay.overlay_type.as_value() != Some(&PatchSpellOverlayOverrideOverlayType::Override) {
        return Err("This casting form is unavailable.".into());
    }
    let mut result = source.clone();
    let SourcePresence::Value(system) = &mut result.system else {
        return Err("Spell mechanics are unavailable.".into());
    };
    let patch = overlay
        .system
        .as_value()
        .ok_or("This form's changes are unavailable.")?;
    if !unchanged_description(system, &patch.description) {
        return Err("This form changes the spell description.".into());
    }
    if !matches!(patch.level, SourcePresence::Missing)
        || !matches!(patch.location, SourcePresence::Missing)
        || !patch.additional_fields.fields().is_empty()
        || patch
            .rules
            .as_value()
            .is_some_and(|rules| !rules.is_empty())
        || matches!(patch.rules, SourcePresence::Invalid(_))
    {
        return Err("This form has additional casting changes.".into());
    }
    if let SourcePresence::Value(heightening) = &patch.heightening {
        if !matches!(heightening.levels, SourcePresence::Missing)
            || !matches!(
                heightening.r#type,
                SourcePresence::Missing
                    | SourcePresence::Value(PatchSpellSystemSourceHeighteningType::Interval)
            )
            || !heightening.additional_fields.fields().is_empty()
        {
            return Err("This form changes fixed-rank rules.".into());
        }
        match &mut system.heightening {
            SourcePresence::Value(SpellSystemSourceHeightening::SpellHeighteningInterval(
                interval,
            )) => {
                members!(interval, heightening, area, interval);
                object(&mut interval.damage, &heightening.damage, |a, b| {
                    for (id, formula) in &b.entries {
                        if let Some((_, target)) = a.entries.iter_mut().find(|(key, _)| key == id) {
                            *target = formula.clone();
                        } else {
                            a.entries.push((id.clone(), formula.clone()));
                        }
                    }
                });
            }
            _ => return Err("This form's rank scaling is unavailable.".into()),
        }
    } else if !matches!(patch.heightening, SourcePresence::Missing) {
        return Err("This form's rank scaling is unavailable.".into());
    }
    patch_fields!(system, patch);
    // Prevent recursive form presentation while retaining the original item.
    system.overlays = SourcePresence::Missing;
    Ok(result)
}

/// Produce a transient checked source for projection; source snapshot is immutable.
pub(crate) fn heighten(source: &SpellSource, rank: u8) -> Result<SpellSource, String> {
    let mut result = source.clone();
    let SourcePresence::Value(system) = &mut result.system else {
        return Err("Spell mechanics are unavailable.".into());
    };
    let base = system
        .level
        .as_value()
        .and_then(|l| l.value.as_value())
        .and_then(|n| n.as_u64())
        .filter(|n| (1..=10).contains(n))
        .ok_or("The spell's original rank is unavailable.")? as u8;
    if !(base..=10).contains(&rank) {
        return Err(format!("Choose a rank from {base} to 10."));
    }
    match system.heightening.clone() {
        SourcePresence::Missing => {}
        SourcePresence::Value(SpellSystemSourceHeightening::SpellHeighteningFixed(fixed)) => {
            let levels = fixed
                .levels
                .as_value()
                .ok_or("Heightening changes are unavailable.")?;
            let layers = [
                &levels._1,
                &levels._2,
                &levels._3,
                &levels._4,
                &levels._5,
                &levels._6,
                &levels._7,
                &levels._8,
                &levels._9,
                &levels._10,
            ];
            for layer in layers.into_iter().take(usize::from(rank)) {
                match layer {
                    SourcePresence::Missing => {}
                    SourcePresence::Value(patch) => patch_system(system, patch)?,
                    _ => return Err("A heightening change is unavailable.".into()),
                }
            }
        }
        SourcePresence::Value(SpellSystemSourceHeightening::SpellHeighteningInterval(interval)) => {
            let step = interval
                .interval
                .as_value()
                .and_then(|n| n.as_u64())
                .filter(|n| *n > 0)
                .ok_or("The heightening interval is unavailable.")?;
            let steps = u64::from(rank - base) / step;
            if steps > 0 {
                if let SourcePresence::Value(increments) = &interval.damage {
                    let damage = match &mut system.damage {
                        SourcePresence::Value(damage) => damage,
                        _ => return Err("Spell damage is unavailable.".into()),
                    };
                    for (id, component) in &mut damage.entries {
                        if let Some((_, increment)) =
                            increments.entries.iter().find(|(key, _)| key == id)
                        {
                            let base = component
                                .formula
                                .as_value()
                                .ok_or("A damage formula is unavailable.")?;
                            component.formula =
                                SourcePresence::Value(add_formula(base, increment, steps)?);
                        }
                    }
                } else if !matches!(interval.damage, SourcePresence::Missing) {
                    return Err("Damage heightening is unavailable.".into());
                }
                let increment = match &interval.area {
                    SourcePresence::Missing => 0,
                    SourcePresence::Value(value) => {
                        value.as_u64().ok_or("Area heightening is unsupported.")?
                    }
                    _ => return Err("Area heightening is unavailable.".into()),
                };
                if increment > 0 {
                    if let SourcePresence::Value(area) = &mut system.area {
                        let value = match area.value.as_value() {
                            Some(StringOrNumber::Number(n)) => n.as_u64(),
                            Some(StringOrNumber::String(s)) => s.parse().ok(),
                            None => None,
                        }
                        .ok_or("The spell area cannot be heightened.")?;
                        let value = increment
                            .checked_mul(steps)
                            .and_then(|v| v.checked_add(value))
                            .ok_or("The heightened area is too large.")?;
                        area.value = SourcePresence::Value(StringOrNumber::Number(value.into()));
                    } else if !matches!(system.area, SourcePresence::Null) {
                        return Err("The spell area cannot be heightened.".into());
                    }
                }
            }
        }
        _ => return Err("Heightening rules are unavailable.".into()),
    }
    Ok(result)
}

/// Combine ordinary additive dice; retain more complex formulas symbolically.
/// This is formula display, not dice execution or contextual modifier evaluation.
fn add_formula(base: &str, increment: &str, steps: u64) -> Result<String, String> {
    fn dice(s: &str) -> Option<(u64, u64)> {
        let (count, faces) = s.trim().split_once('d')?;
        Some((count.parse().ok()?, faces.parse().ok()?))
    }
    if let (Some((a, faces)), Some((b, increment_faces))) = (dice(base), dice(increment))
        && faces == increment_faces
    {
        let count = b
            .checked_mul(steps)
            .and_then(|v| v.checked_add(a))
            .ok_or("The heightened damage is too large.")?;
        return Ok(format!("{count}d{faces}"));
    }
    if increment.trim().is_empty() || increment.trim() == "0" {
        return Ok(base.into());
    }
    Ok(format!("({base}) + {steps} × ({increment})"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use atlas_record::source_record::{ItemSourceView, SourceNodeView};
    use atlas_search::test_support::record;
    use serde_json::json;

    #[test]
    fn unsupported_heightening_does_not_claim_effective_mechanics() {
        for heightening in [
            json!({"type":"interval","interval":1,"area":"broken"}),
            json!({"type":"interval","interval":1,"area":2.5}),
            json!({"type":"interval","interval":1,"area":-5}),
            json!({"type":"interval","interval":1,"area":null}),
            json!({"type":"fixed","levels":{"2":{"description":{"value":"Changed rules"}}}}),
            json!({"type":"fixed","levels":{"2":{"heightening":{"interval":2}}}}),
        ] {
            let input = record(
                "spells",
                "Item",
                json!({
                    "_id":"aaaaaaaaaaaaaaaa","type":"spell","system":{
                        "level":{"value":1},"description":{"value":"Original rules"},
                        "area":{"type":"burst","value":10},"heightening":heightening
                    }
                }),
            );
            let SourceNodeView::Item(ItemSourceView::SpellSource(source)) =
                input.record.node_at(&[]).unwrap()
            else {
                panic!("spell")
            };
            assert!(heighten(source, 3).is_err(), "{heightening}");
        }
    }

    #[test]
    fn identical_description_patch_is_safe() {
        let input = record(
            "spells",
            "Item",
            json!({
                "_id":"aaaaaaaaaaaaaaaa","type":"spell","system":{
                    "level":{"value":1},"description":{"value":"Original rules","gm":"Private rules"},
                "heightening":{"type":"fixed","levels":{"2":{"description":{"value":"Original rules"},"heightening":{}}}}
                }
            }),
        );
        let SourceNodeView::Item(ItemSourceView::SpellSource(source)) =
            input.record.node_at(&[]).unwrap()
        else {
            panic!("spell")
        };
        assert!(heighten(source, 3).is_ok());
    }

    #[test]
    fn fixed_layers_merge_in_order_and_preserve_the_local_source() {
        let input = record(
            "spells",
            "Item",
            json!({"_id":"aaaaaaaaaaaaaaaa","type":"spell","system":{
                "level":{"value":1},"range":{"value":"30 feet"},
                "area":{"type":"burst","value":10},
                "damage":{"local":{"formula":"1d6","type":"fire"}},
                "heightening":{"type":"fixed","levels":{
                    "2":{"range":{"value":"60 feet"},"damage":{"local":{"formula":"2d6"}}},
                    "3":{"range":{"value":"120 feet"},"area":null}
                }}
            }}),
        );
        let SourceNodeView::Item(ItemSourceView::SpellSource(source)) =
            input.record.node_at(&[]).unwrap()
        else {
            panic!("spell");
        };
        let effective = heighten(source, 3).unwrap();
        let system = effective.system.as_value().unwrap();
        assert_eq!(
            system.range.as_value().unwrap().value.as_value().unwrap(),
            "120 feet"
        );
        assert!(matches!(system.area, SourcePresence::Null));
        let damage = &system.damage.as_value().unwrap().entries[0];
        assert_eq!(damage.0, "local");
        assert_eq!(damage.1.formula.as_value().unwrap(), "2d6");
        assert_eq!(
            damage.1.r#type,
            source
                .system
                .as_value()
                .unwrap()
                .damage
                .as_value()
                .unwrap()
                .entries[0]
                .1
                .r#type
        );
        assert_eq!(
            source
                .system
                .as_value()
                .unwrap()
                .range
                .as_value()
                .unwrap()
                .value
                .as_value()
                .unwrap(),
            "30 feet"
        );
    }

    #[test]
    fn interval_scales_damage_and_area_only_at_complete_steps() {
        let input = record(
            "spells",
            "Item",
            json!({"_id":"aaaaaaaaaaaaaaaa","type":"spell","system":{
                "level":{"value":1},"area":{"type":"burst","value":10},
                "damage":{"local":{"formula":"2d6","type":"fire"}},
                "heightening":{"type":"interval","interval":2,"area":5,"damage":{"local":"1d6"}}
            }}),
        );
        let SourceNodeView::Item(ItemSourceView::SpellSource(source)) =
            input.record.node_at(&[]).unwrap()
        else {
            panic!("spell");
        };
        for (rank, formula, area) in [
            (1, "2d6", 10),
            (2, "2d6", 10),
            (3, "3d6", 15),
            (4, "3d6", 15),
            (5, "4d6", 20),
        ] {
            let effective = heighten(source, rank).unwrap();
            let system = effective.system.as_value().unwrap();
            assert_eq!(
                system.damage.as_value().unwrap().entries[0]
                    .1
                    .formula
                    .as_value()
                    .unwrap(),
                formula,
                "rank {rank}"
            );
            assert_eq!(
                system.area.as_value().unwrap().value.as_value(),
                Some(&StringOrNumber::Number(area.into())),
                "rank {rank}"
            );
        }
        assert!(heighten(source, 0).is_err());
        assert!(heighten(source, 11).is_err());
    }
    #[test]
    fn heightening_combines_dice_and_preserves_other_expressions() {
        assert_eq!(add_formula("6d6", "2d6", 2).unwrap(), "10d6");
        assert_eq!(
            add_formula("1d4+1", "1d4+1", 2).unwrap(),
            "(1d4+1) + 2 × (1d4+1)"
        );
        assert!(add_formula("1d6", "18446744073709551615d6", 2).is_err());
    }
}
