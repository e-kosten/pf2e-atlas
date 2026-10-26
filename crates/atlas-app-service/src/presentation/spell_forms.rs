//! Read-only casting forms use the same rank owner as their selected spell.
use super::*;
use atlas_foundry_model::generated::SpellSource;

fn difference<T: Clone + PartialEq>(base: &FactView<T>, form: &FactView<T>) -> FactView<T> {
    if base == form {
        unavailable()
    } else {
        form.clone()
    }
}

fn changes(base: &SpellPresentationView, form: &SpellPresentationView) -> SpellChangeView {
    SpellChangeView {
        cast: difference(&base.cast, &form.cast),
        range: difference(&base.range, &form.range),
        target: difference(&base.target, &form.target),
        area: difference(
            base.area.as_ref().unwrap_or(&fact(SourceFieldView::Null)),
            form.area.as_ref().unwrap_or(&fact(SourceFieldView::Null)),
        ),
        defense: difference(
            base.defense
                .as_ref()
                .unwrap_or(&fact(SourceFieldView::Null)),
            form.defense
                .as_ref()
                .unwrap_or(&fact(SourceFieldView::Null)),
        ),
        duration: difference(&base.duration, &form.duration),
        sustained: difference(&base.sustained, &form.sustained),
        traits: unavailable(),
        traditions: difference(&base.traditions, &form.traditions),
        damage: difference(&base.damage, &form.damage),
        // These are effective facts; repeating the raw interval here is not a result.
        heightening: unavailable(),
    }
}

pub(super) fn apply_forms(
    source: &SpellSource,
    selected: &SpellSource,
    rank: u8,
    view: &mut SpellPresentationView,
) {
    let Some(overlays) = source.system.as_value().and_then(|s| s.overlays.as_value()) else {
        return;
    };
    let Some(mut forms) = view.forms.value.take() else {
        return;
    };
    for form in &mut forms {
        let Some((_, overlay)) = overlays.entries.iter().find(|(id, _)| id == &form.id) else {
            continue;
        };
        let result = spell_rank::casting_form(source, overlay)
            .and_then(|base| spell_rank::heighten(&base, rank).map(|effective| (base, effective)));
        match result {
            Ok((base, effective)) => {
                form.effective_rank = Some(rank);
                let current = spell::authored_spell(&effective);
                let rank_changes = spell_changes::rank_changes(&base, &effective, &current, rank);
                form.rank_changes = (!rank_changes.is_empty()).then_some(rank_changes);
                let mut changes = changes(view, &current);
                // Identity traits live outside the family body; compare the checked sources.
                let traits = |source: &SpellSource| {
                    identifiers(
                        SourceFieldView::from(&source.system)
                            .and_then(|s| (&s.traits).into())
                            .and_then(|t| (&t.value).into()),
                    )
                };
                changes.traits = difference(&traits(selected), &traits(&effective));
                form.changes = FactView {
                    state: QueryFieldState::Value,
                    value: Some(changes),
                };
            }
            Err(reason) => {
                form.rank_note = Some(format!("Rank {rank} preview unavailable. {reason}"));
            }
        }
    }
    forms.sort_by_key(|form| {
        form.sort
            .value
            .as_ref()
            .and_then(serde_json::Number::as_i64)
            .unwrap_or(i64::MAX)
    });
    view.forms.value = Some(forms);
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn a_casting_form_scales_its_own_damage_then_compares_at_the_selected_rank() {
        let record = atlas_search::test_support::record(
            "spells",
            "Item",
            json!({"_id":"aaaaaaaaaaaaaaaa","type":"spell","system":{
                "level":{"value":1},"time":{"value":"1 to 3"},"damage":{"main":{"formula":"1d8","type":"vitality","kinds":["damage","healing"]}},
                "heightening":{"type":"interval","interval":1,"damage":{"main":"1d8"}},
                "overlays":{"living":{"overlayType":"override","sort":2,"system":{
                    "time":{"value":"2"},"damage":{"main":{"formula":"1d8+8","kinds":["healing"]}},
                    "heightening":{"damage":{"main":"1d8+8"}}
                }},"touch":{"overlayType":"override","sort":1,"system":{"time":{"value":"1"}}}}
            }}),
        );
        let SourceNodeView::Item(ItemSourceView::SpellSource(source)) =
            record.record.node_at(&[]).unwrap()
        else {
            panic!("spell");
        };
        let original = source.clone();
        let effective = spell_rank::heighten(source, 3).unwrap();
        let mut view = spell::authored_spell(&effective);
        apply_forms(source, &effective, 3, &mut view);
        let forms = view.forms.value.unwrap();
        assert_eq!(forms[0].id, "touch");
        assert_eq!(
            forms[0].changes.value.as_ref().unwrap().damage.state,
            QueryFieldState::NotApplicable
        );
        let living = &forms[1];
        assert_eq!(living.rank_note, None);
        let damage = living
            .changes
            .value
            .as_ref()
            .unwrap()
            .damage
            .value
            .as_ref()
            .unwrap();
        assert_eq!(
            damage[0].formula.value.as_deref(),
            Some("(1d8+8) + 2 × (1d8+8)")
        );
        assert_eq!(
            damage[0].kinds.value.as_ref().unwrap(),
            &vec!["healing".to_owned()]
        );
        let marker = &living.rank_changes.as_ref().unwrap()[0];
        assert_eq!(marker.base_value, "1d8+8 vitality; healing");
        assert_eq!(source, &original);
    }

    #[test]
    fn unsupported_form_scaling_preserves_the_authored_patch_with_a_local_reason() {
        let record = atlas_search::test_support::record(
            "spells",
            "Item",
            json!({"_id":"aaaaaaaaaaaaaaaa","type":"spell","system":{
                "level":{"value":1},"overlays":{"variant":{"overlayType":"override","sort":1,"system":{
                    "description":{"value":"Different rules"},"target":{"value":"2 creatures"}
                }}}
            }}),
        );
        let SourceNodeView::Item(ItemSourceView::SpellSource(source)) =
            record.record.node_at(&[]).unwrap()
        else {
            panic!("spell");
        };
        let mut view = spell::authored_spell(source);
        apply_forms(source, source, 3, &mut view);
        let form = &view.forms.value.as_ref().unwrap()[0];
        assert!(
            form.rank_note
                .as_ref()
                .unwrap()
                .contains("changes the spell description")
        );
        assert_eq!(
            form.changes.value.as_ref().unwrap().target.value.as_deref(),
            Some("2 creatures")
        );
        assert!(form.rank_changes.is_none());
    }
}
