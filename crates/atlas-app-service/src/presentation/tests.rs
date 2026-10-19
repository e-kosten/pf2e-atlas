use super::*;
use crate::{executor::RetrievalExecutor, test_support::fixture_worker_with_executor};
use atlas_search::test_support::{open_source_fixture, record};
use serde_json::json;

fn actor(p: &RecordPresentationView) -> &ActorPresentationView {
    let RecordBodyView::Creature(a) = &p.body else {
        panic!("creature");
    };
    a
}

#[test]
fn optional_spell_structures_distinguish_missing_leaf_from_unavailable_system() {
    for (system, expected) in [
        (None, QueryFieldState::Missing),
        (Some(json!(null)), QueryFieldState::Null),
        (Some(json!(42)), QueryFieldState::Invalid),
        (Some(json!({})), QueryFieldState::NotApplicable),
        (
            Some(json!({"heightening":null,"overlays":null})),
            QueryFieldState::Null,
        ),
        (
            Some(json!({"heightening":42,"overlays":42})),
            QueryFieldState::Invalid,
        ),
    ] {
        let mut value = json!({"_id":"aaaaaaaaaaaaaaaa","type":"spell"});
        if let Some(system) = system {
            value["system"] = system;
        }
        let input = record("spells", "Item", value);
        let SourceNodeView::Item(ItemSourceView::SpellSource(source)) =
            SourceQueryView::new(input.record.source(), "spells", "").source
        else {
            panic!("spell")
        };
        let spell = spell::authored_spell(source);
        assert_eq!(spell.heightening.state, expected);
        assert_eq!(spell.forms.state, expected);
        if expected == QueryFieldState::NotApplicable {
            assert!(matches!(
                source.system.as_value().unwrap().heightening,
                atlas_foundry_model::SourcePresence::Missing
            ));
            assert!(matches!(
                source.system.as_value().unwrap().overlays,
                atlas_foundry_model::SourcePresence::Missing
            ));
        }
    }
}

#[test]
fn physical_prices_preserve_parent_and_currency_availability_without_mutating_source() {
    for (price, expected) in [
        (None, QueryFieldState::Missing),
        (Some(json!(null)), QueryFieldState::Null),
        (Some(json!(42)), QueryFieldState::Invalid),
        (Some(json!({"value":null})), QueryFieldState::Null),
        (Some(json!({"value":42})), QueryFieldState::Invalid),
        (
            Some(json!({"value":{"gp":0,"sp":null,"cp":"bad"}})),
            QueryFieldState::Value,
        ),
        (Some(json!({"value":{}})), QueryFieldState::Value),
    ] {
        let mut system = json!({"level":{"value":0}});
        if let Some(price) = price {
            system["price"] = price;
        }
        let input = record(
            "items",
            "Item",
            json!({"_id":"aaaaaaaaaaaaaaaa","type":"equipment","system":system}),
        );
        let node = SourceQueryView::new(input.record.source(), "items", "").source;
        let SourceNodeView::Item(item) = node else {
            panic!("item")
        };
        let authored = item.physical_price();
        let physical = physical(item);
        assert_eq!(physical.price.state, expected);
        assert_eq!(
            physical.price.state,
            field_state(authored.and_then(|p| (&p.value).into()).availability())
        );
        if let Some(price) = physical.price.value {
            assert_eq!(price.pp.state, QueryFieldState::Missing);
            if system["price"]["value"].get("gp").is_some() {
                assert_eq!(price.gp.value, Some(0.into()));
                assert_eq!(price.sp.state, QueryFieldState::Null);
                assert_eq!(price.cp.state, QueryFieldState::Invalid);
                assert_eq!(
                    authored
                        .value()
                        .unwrap()
                        .value
                        .as_value()
                        .unwrap()
                        .gp
                        .as_value()
                        .and_then(|n| n.as_i64()),
                    Some(0)
                );
            } else {
                assert_eq!(price.gp.state, QueryFieldState::Missing);
            }
        }
    }
}

#[test]
fn classification_uses_typed_rank_and_level_including_zero_and_unavailable() {
    let f = fixture_worker_with_executor(RetrievalExecutor::from_test_fixture_factory(
        1,
        16,
        || {
            Ok(open_source_fixture(
                vec![
                    record(
                        "items",
                        "Item",
                        json!({"_id":"aaaaaaaaaaaaaaaa","type":"equipment","name":"Zero Level","system":{"level":{"value":0}}}),
                    ),
                    record(
                        "spells",
                        "Item",
                        json!({"_id":"bbbbbbbbbbbbbbbb","type":"spell","name":"Unavailable Rank","system":{"level":{"value":null}}}),
                    ),
                ],
                false,
            )?)
        },
    ));
    let zero = f.worker.record_detail("items:aaaaaaaaaaaaaaaa").unwrap();
    assert_eq!(
        zero.presentation.classification,
        format!("{} 0", zero.presentation.identity.kind_label)
    );
    let unavailable = f.worker.record_detail("spells:bbbbbbbbbbbbbbbb").unwrap();
    assert_eq!(
        unavailable.presentation.classification,
        format!(
            "{} · rank unavailable",
            unavailable.presentation.identity.kind_label
        )
    );
}

#[test]
fn optional_spell_baselines_omit_absent_or_blank_text_only_under_a_known_system() {
    for system in [
        json!({}),
        json!({"requirements":null,"cost":null,"location":null}),
        json!({"requirements":"","cost":{"value":""},"location":{"value":""}}),
        json!({"requirements":" \n\t ","cost":{"value":"\t "},"location":{"value":" \n"}}),
    ] {
        let authored_requirement = system
            .get("requirements")
            .and_then(serde_json::Value::as_str)
            .map(str::to_owned);
        let input = record(
            "spells",
            "Item",
            json!({"_id":"aaaaaaaaaaaaaaaa","type":"spell","system":system}),
        );
        let SourceNodeView::Item(ItemSourceView::SpellSource(source)) =
            SourceQueryView::new(input.record.source(), "spells", "").source
        else {
            panic!("spell")
        };
        let spell = spell::authored_spell(source);
        assert!(spell.requirements.is_none());
        assert!(spell.cost.is_none());
        assert!(spell.casting_entry.is_none());
        assert_eq!(
            SourceFieldView::from(&source.system)
                .and_then(|s| (&s.requirements).into())
                .value()
                .map(String::as_str),
            authored_requirement.as_deref()
        );
    }
    for (system, expected) in [
        (None, QueryFieldState::Missing),
        (Some(json!(null)), QueryFieldState::Null),
        (Some(json!(42)), QueryFieldState::Invalid),
        (
            Some(json!({"requirements":42,"cost":{"value":42},"location":{"value":42}})),
            QueryFieldState::Invalid,
        ),
    ] {
        let mut value = json!({"_id":"aaaaaaaaaaaaaaaa","type":"spell"});
        if let Some(system) = system {
            value["system"] = system;
        }
        let input = record("spells", "Item", value);
        let SourceNodeView::Item(ItemSourceView::SpellSource(source)) =
            SourceQueryView::new(input.record.source(), "spells", "").source
        else {
            panic!("spell")
        };
        let spell = spell::authored_spell(source);
        for fact in [spell.requirements, spell.cost, spell.casting_entry] {
            let fact = fact.expect("unavailable parents and invalid fields remain explicit");
            assert_eq!(fact.state, expected);
        }
    }
    for value in ["0", "false", " 2 gp "] {
        let input = record(
            "spells",
            "Item",
            json!({"_id":"aaaaaaaaaaaaaaaa","type":"spell","system":{"requirements":value,"cost":{"value":value},"location":{"value":value}}}),
        );
        let SourceNodeView::Item(ItemSourceView::SpellSource(source)) =
            SourceQueryView::new(input.record.source(), "spells", "").source
        else {
            panic!("spell")
        };
        let spell = spell::authored_spell(source);
        for fact in [spell.requirements, spell.cost, spell.casting_entry] {
            let fact = fact.expect("nonempty authored values remain explicit");
            assert_eq!(fact.state, QueryFieldState::Value);
            assert_eq!(fact.value.as_deref(), Some(value));
        }
    }
}

#[test]
fn baseline_spell_area_omits_only_explicit_null_under_a_known_system() {
    for (system, expected) in [
        (None, Some(QueryFieldState::Missing)),
        (Some(json!(null)), Some(QueryFieldState::Null)),
        (Some(json!(42)), Some(QueryFieldState::Invalid)),
        (Some(json!({})), Some(QueryFieldState::Missing)),
        (
            Some(
                json!({"area":null,"overlays":{"form":{"overlayType":"override","system":{"area":null}}}}),
            ),
            None,
        ),
        (Some(json!({"area":42})), Some(QueryFieldState::Invalid)),
        (
            Some(json!({"area":{"type":"emanation","value":30}})),
            Some(QueryFieldState::Value),
        ),
    ] {
        let mut value = json!({"_id":"aaaaaaaaaaaaaaaa","type":"spell"});
        if let Some(system) = system {
            value["system"] = system;
        }
        let input = record("spells", "Item", value);
        let SourceNodeView::Item(ItemSourceView::SpellSource(source)) =
            SourceQueryView::new(input.record.source(), "spells", "").source
        else {
            panic!("spell")
        };
        let spell = spell::authored_spell(source);
        assert_eq!(spell.area.as_ref().map(|area| area.state), expected);
        if expected.is_none() {
            assert_eq!(
                SourceFieldView::from(&source.system)
                    .and_then(|system| (&system.area).into())
                    .availability(),
                atlas_record::source_record::FieldAvailability::Null
            );
            assert_eq!(
                spell.forms.value.as_ref().unwrap()[0]
                    .changes
                    .value
                    .as_ref()
                    .unwrap()
                    .area
                    .state,
                QueryFieldState::Null,
                "baseline no-area omission must not omit a form's explicit clear"
            );
        }
    }
}

#[test]
fn baseline_spell_defense_omits_only_explicit_null_under_a_known_system() {
    for (system, expected) in [
        (None, Some(QueryFieldState::Missing)),
        (Some(json!(null)), Some(QueryFieldState::Null)),
        (Some(json!(42)), Some(QueryFieldState::Invalid)),
        (Some(json!({})), Some(QueryFieldState::Missing)),
        (
            Some(
                json!({"defense":null,"overlays":{"form":{"overlayType":"override","system":{"defense":null}}}}),
            ),
            None,
        ),
        (Some(json!({"defense":42})), Some(QueryFieldState::Invalid)),
        (
            Some(json!({"defense":{"save":{"statistic":"reflex","basic":true}}})),
            Some(QueryFieldState::Value),
        ),
    ] {
        let mut value = json!({"_id":"aaaaaaaaaaaaaaaa","type":"spell"});
        if let Some(system) = system {
            value["system"] = system;
        }
        let input = record("spells", "Item", value);
        let SourceNodeView::Item(ItemSourceView::SpellSource(source)) =
            SourceQueryView::new(input.record.source(), "spells", "").source
        else {
            panic!("spell")
        };
        let spell = spell::authored_spell(source);
        assert_eq!(
            spell.defense.as_ref().map(|defense| defense.state),
            expected
        );
        if expected.is_none() {
            assert_eq!(
                SourceFieldView::from(&source.system)
                    .and_then(|system| (&system.defense).into())
                    .availability(),
                atlas_record::source_record::FieldAvailability::Null
            );
            assert_eq!(
                spell.forms.value.as_ref().unwrap()[0]
                    .changes
                    .value
                    .as_ref()
                    .unwrap()
                    .defense
                    .state,
                QueryFieldState::Null,
                "baseline no-defense omission must not omit a form's explicit clear"
            );
        }
    }
}

#[test]
fn authored_action_costs_are_typed_without_guessing_from_prose() {
    for (time, glyph, label) in [
        ("1", Some(ActionGlyphView::OneAction), "1 action"),
        ("2", Some(ActionGlyphView::TwoActions), "2 actions"),
        ("3", Some(ActionGlyphView::ThreeActions), "3 actions"),
        (" Reaction ", Some(ActionGlyphView::Reaction), "reaction"),
        ("0", Some(ActionGlyphView::FreeAction), "free action"),
        ("1 to 3", None, "1 to 3 actions"),
        ("1 minute", None, "1 minute"),
        ("2 to 2 rounds", None, "2 to 2 rounds"),
    ] {
        let input = record(
            "spells",
            "Item",
            json!({"_id":"aaaaaaaaaaaaaaaa","type":"spell","system":{"time":{"value":time}}}),
        );
        let SourceNodeView::Item(item @ ItemSourceView::SpellSource(source)) =
            SourceQueryView::new(input.record.source(), "spells", "").source
        else {
            panic!("spell")
        };
        let spell = spell::authored_spell(source);
        assert_eq!(spell.cast, item_action_cost(item));
        assert_eq!(
            spell.cast.value,
            Some(ActionCostView {
                glyph,
                label: label.into()
            })
        );
        assert!(item_usage(item).is_none());
    }
    for (kind, count, glyph, state) in [
        (
            "action",
            json!(1),
            Some(ActionGlyphView::OneAction),
            QueryFieldState::Value,
        ),
        (
            "action",
            json!(2),
            Some(ActionGlyphView::TwoActions),
            QueryFieldState::Value,
        ),
        (
            "action",
            json!(3),
            Some(ActionGlyphView::ThreeActions),
            QueryFieldState::Value,
        ),
        ("action", json!(4), None, QueryFieldState::Invalid),
        (
            "action",
            json!(1.0),
            Some(ActionGlyphView::OneAction),
            QueryFieldState::Value,
        ),
        ("action", json!(null), None, QueryFieldState::Null),
        ("action", json!("bad"), None, QueryFieldState::Invalid),
        (
            "free",
            json!(null),
            Some(ActionGlyphView::FreeAction),
            QueryFieldState::Value,
        ),
        (
            "reaction",
            json!(null),
            Some(ActionGlyphView::Reaction),
            QueryFieldState::Value,
        ),
        ("passive", json!(null), None, QueryFieldState::Value),
    ] {
        let input = record(
            "actions",
            "Item",
            json!({"_id":"aaaaaaaaaaaaaaaa","type":"action","system":{"actionType":{"value":kind},"actions":{"value":count}}}),
        );
        let SourceNodeView::Item(item) =
            SourceQueryView::new(input.record.source(), "actions", "").source
        else {
            panic!("item")
        };
        let cost = item_action_cost(item);
        assert_eq!(cost.state, state);
        assert_eq!(cost.value.as_ref().and_then(|v| v.glyph), glyph);
        if kind == "passive" {
            assert_eq!(cost.value.unwrap().label, "passive");
        }
    }
    for (system, state) in [
        (json!({}), QueryFieldState::Missing),
        (json!({"actionType":null}), QueryFieldState::Null),
        (
            json!({"actionType":{"value":"bad"}}),
            QueryFieldState::Invalid,
        ),
    ] {
        let input = record(
            "actions",
            "Item",
            json!({"_id":"aaaaaaaaaaaaaaaa","type":"action","system":system}),
        );
        let SourceNodeView::Item(item) =
            SourceQueryView::new(input.record.source(), "actions", "").source
        else {
            panic!("item")
        };
        assert_eq!(item_action_cost(item).state, state);
    }
}
#[test]
fn actor_collection_availability_is_independent_of_zero_and_empty() {
    for (items, expected) in [
        (None, QueryFieldState::Missing),
        (Some(json!([])), QueryFieldState::Value),
        (Some(json!(null)), QueryFieldState::Null),
        (Some(json!(17)), QueryFieldState::Invalid),
    ] {
        let mut source = json!({"_id":"aaaaaaaaaaaaaaaa","type":"npc","system":{"skills":{},"abilities":null,"attributes":{"speed":{"value":0,"otherSpeeds":[]}}}});
        if let Some(items) = items {
            source["items"] = items;
        }
        let input = record("actors", "Actor", source);
        let node = SourceQueryView::new(input.record.source(), "actors", "").source;
        let a = actor::authored_actor(&input.record, node, "fixture");
        assert_eq!(a.activities.state, expected);
        assert_eq!(a.skills.state, QueryFieldState::Value);
        assert!(a.skills.value.unwrap().is_empty());
        assert_eq!(a.abilities.state, QueryFieldState::Null);
        assert_eq!(a.land_speed.value, Some(0.into()));
        assert_eq!(a.movement.value, Some(vec![]));
    }
}
#[test]
fn custom_iwr_qualifiers_use_the_authored_label_without_evaluating_predicates() {
    let input = record(
        "actors",
        "Actor",
        json!({"_id":"aaaaaaaaaaaaaaaa","type":"npc","system":{"attributes":{"immunities":[{"type":"fire","exceptions":[{"label":"silver weapons","definition":["damage:material:silver"]}]}],"weaknesses":[{"type":"cold","value":2,"exceptions":[{"label":"while sheltered","definition":["sheltered"]}]}],"resistances":[{"type":"all-damage","value":10,"exceptions":[{"label":"blessed silver","definition":["damage:material:silver","blessed"]}],"doubleVs":[{"label":"unholy fire","definition":["damage:type:fire","unholy"]}]}]}}}),
    );
    let node = SourceQueryView::new(input.record.source(), "actors", "").source;
    let a = actor::authored_actor(&input.record, node, "fixture");
    assert_eq!(
        a.immunities.value.unwrap()[0].exceptions.value,
        Some(vec!["silver weapons".into()])
    );
    assert_eq!(
        a.weaknesses.value.unwrap()[0].exceptions.value,
        Some(vec!["while sheltered".into()])
    );
    let r = a.resistances.value.unwrap();
    assert_eq!(r[0].exceptions.value, Some(vec!["blessed silver".into()]));
    assert_eq!(r[0].double_against.value, Some(vec!["unholy fire".into()]));
}
#[test]
fn duplicate_casting_ids_preserve_ambiguous_authored_association() {
    let entry = json!({"_id":"eeeeeeeeeeeeeeee","type":"spellcastingEntry","name":"Same entry"});
    let input = record(
        "actors",
        "Actor",
        json!({"_id":"aaaaaaaaaaaaaaaa","type":"npc","items":[entry.clone(),entry,{"_id":"ssssssssssssssss","type":"spell","system":{"location":{"value":"eeeeeeeeeeeeeeee"}}}]}),
    );
    let node = SourceQueryView::new(input.record.source(), "actors", "").source;
    let a = actor::authored_actor(&input.record, node, "fixture");
    let activity = &a.activities.value.unwrap()[2];
    assert_eq!(
        activity.association.value.as_deref(),
        Some("eeeeeeeeeeeeeeee")
    );
    assert!(activity.casting_entry.is_none());
}
#[test]
fn authored_iwr_preserves_qualifiers_zero_and_unavailable_members() {
    let input = record(
        "actors",
        "Actor",
        json!({"_id":"aaaaaaaaaaaaaaaa","type":"npc","system":{"attributes":{"hp":{"max":0},"immunities":[],"weaknesses":null,"resistances":[{"type":"all-damage","value":10,"exceptions":["force","ghost-touch","vitality"],"doubleVs":["non-magical"]},{"type":"cold","value":"broken"}]},"saves":{"will":{"value":0}}}}),
    );
    let node = SourceQueryView::new(input.record.source(), "actors", "").source;
    let a = actor::authored_actor(&input.record, node, "fixture");
    assert_eq!(a.maximum_hp.value, Some(0.into()));
    assert_eq!(a.saves.will.value, Some(0.into()));
    assert_eq!(a.saves.reflex.state, QueryFieldState::Missing);
    assert_eq!(a.immunities.value, Some(vec![]));
    assert_eq!(a.weaknesses.state, QueryFieldState::Null);
    let r = a.resistances.value.unwrap();
    assert_eq!(r[0].damage_type.value.as_deref(), Some("all-damage"));
    assert_eq!(r[0].magnitude.value, Some(10.into()));
    assert_eq!(
        r[0].exceptions.value,
        Some(vec![
            "force".into(),
            "ghost-touch".into(),
            "vitality".into()
        ])
    );
    assert_eq!(r[0].double_against.value, Some(vec!["non-magical".into()]));
    assert_eq!(r[1].magnitude.state, QueryFieldState::Invalid);
}
#[test]
fn spells_preserve_authored_damage_forms_and_interval_without_evaluating() {
    let input = record(
        "spells",
        "Item",
        json!({"_id":"aaaaaaaaaaaaaaaa","name":"Heal","type":"spell","system":{"level":{"value":1},"time":{"value":"1 to 3"},"range":{"value":"varies"},"traits":{"value":["healing","vitality"],"traditions":["divine","primal"]},"duration":{"value":"","sustained":false},"area":{"type":"emanation","value":30},"defense":{"save":{"statistic":"fortitude","basic":true}},"damage":{"main":{"formula":"1d8","type":"vitality","kinds":["damage","healing"],"category":null,"materials":[],"applyMod":false}},"heightening":{"type":"interval","interval":1,"damage":{"main":"1d8"}},"overlays":{"living":{"name":"Against living","overlayType":"override","sort":1,"system":{"heightening":{"damage":{"main":"1d8+8"}},"defense":null,"time":{"value":"2"},"traits":{"value":["healing"],"traditions":["divine"]}}}}}}),
    );
    let SourceNodeView::Item(ItemSourceView::SpellSource(source)) =
        SourceQueryView::new(input.record.source(), "spells", "").source
    else {
        panic!("spell");
    };
    let s = spell::authored_spell(source);
    assert_eq!(
        s.cast.value.as_ref().map(|v| v.label.as_str()),
        Some("1 to 3 actions")
    );
    assert_eq!(
        s.area
            .as_ref()
            .unwrap()
            .value
            .as_ref()
            .unwrap()
            .size
            .value
            .as_deref(),
        Some("30 feet")
    );
    assert_eq!(
        s.defense
            .as_ref()
            .unwrap()
            .value
            .as_ref()
            .unwrap()
            .basic
            .value,
        Some(true)
    );
    assert_eq!(s.sustained.value, Some(false));
    let damage = &s.damage.value.unwrap()[0];
    assert_eq!(damage.formula.value.as_deref(), Some("1d8"));
    assert_eq!(damage.damage_type.value.as_deref(), Some("vitality"));
    assert_eq!(
        damage.kinds.value,
        Some(vec!["damage".into(), "healing".into()])
    );
    assert_eq!(damage.apply_modifier.value, Some(false));
    let SpellHeighteningView::Interval {
        interval, damage, ..
    } = s.heightening.value.unwrap()
    else {
        panic!("interval");
    };
    assert_eq!(interval.value, Some(1.into()));
    assert_eq!(
        damage.value.unwrap()[0].formula.value.as_deref(),
        Some("1d8")
    );
    let form = &s.forms.value.unwrap()[0];
    let changes = form.changes.value.as_ref().unwrap();
    assert_eq!(changes.defense.state, QueryFieldState::Null);
    assert_eq!(
        changes.cast.value.as_ref().map(|v| v.label.as_str()),
        Some("2 actions")
    );
    assert_eq!(changes.traits.value, Some(vec!["healing".into()]));
    assert_eq!(changes.range.state, QueryFieldState::Missing);
    assert_eq!(
        changes
            .heightening
            .value
            .as_ref()
            .unwrap()
            .damage
            .value
            .as_ref()
            .unwrap()[0]
            .formula
            .value
            .as_deref(),
        Some("1d8+8")
    );
    assert_eq!(
        changes.heightening.value.as_ref().unwrap().interval.state,
        QueryFieldState::Missing
    );
}
#[test]
fn fixed_heightening_keeps_rank_patch_and_invalid_enclosing_states() {
    let input = record(
        "spells",
        "Item",
        json!({"_id":"aaaaaaaaaaaaaaaa","type":"spell","system":{"heightening":{"type":"fixed","levels":{"3":{"range":{"value":"120 feet"},"damage":{"main":{"formula":"4d6","type":"fire","category":null,"materials":[]}}},"5":null}}}}),
    );
    let SourceNodeView::Item(ItemSourceView::SpellSource(source)) =
        SourceQueryView::new(input.record.source(), "spells", "").source
    else {
        panic!("spell");
    };
    let s = spell::authored_spell(source);
    let SpellHeighteningView::Fixed { levels } = s.heightening.value.unwrap() else {
        panic!("fixed");
    };
    let levels = levels.value.unwrap();
    assert_eq!(levels.len(), 2);
    assert_eq!(levels[0].rank, 3);
    let changes = levels[0].changes.value.as_ref().unwrap();
    assert_eq!(changes.range.value.as_deref(), Some("120 feet"));
    assert_eq!(
        changes.damage.value.as_ref().unwrap()[0]
            .formula
            .value
            .as_deref(),
        Some("4d6")
    );
    assert_eq!(levels[1].changes.state, QueryFieldState::Null);
    let input = record(
        "spells",
        "Item",
        json!({"_id":"bbbbbbbbbbbbbbbb","type":"spell","system":17}),
    );
    let SourceNodeView::Item(ItemSourceView::SpellSource(source)) =
        SourceQueryView::new(input.record.source(), "spells", "").source
    else {
        panic!("spell");
    };
    let s = spell::authored_spell(source);
    assert_eq!(s.range.state, QueryFieldState::Invalid);
    assert_eq!(s.damage.state, QueryFieldState::Invalid);
    assert_eq!(s.rank.state, QueryFieldState::Invalid);
}
#[test]
fn selected_local_spell_and_casting_entry_keep_customization_and_typed_association() {
    let spell = json!({"_id":"ssssssssssssssss","type":"spell","name":"Local Fireball","system":{"level":{"value":3},"time":{"value":"2"},"location":{"value":"eeeeeeeeeeeeeeee","heightenedLevel":6},"range":{"value":"250 feet"},"damage":{"custom":{"formula":"12d6","type":"fire","category":null,"materials":[]}}}});
    let actor_source = json!({"_id":"aaaaaaaaaaaaaaaa","type":"npc","name":"Caster","items":[{"_id":"eeeeeeeeeeeeeeee","type":"spellcastingEntry","name":"Arcane Prepared Spells","system":{"spelldc":{"dc":33,"value":23},"prepared":{"value":"prepared"},"tradition":{"value":"arcane"}}},spell]});
    let executor = RetrievalExecutor::from_test_fixture_factory(1, 16, move || {
        Ok(open_source_fixture(
            vec![record("actors", "Actor", actor_source.clone())],
            false,
        )?)
    });
    let f = fixture_worker_with_executor(executor);
    let d = f.worker.record_detail("actors:aaaaaaaaaaaaaaaa").unwrap();
    let a = actor(&d.presentation);
    let local = &a.activities.value.as_ref().unwrap()[1];
    assert_eq!(
        local.casting_entry.as_ref().unwrap(),
        &a.activities.value.as_ref().unwrap()[0].navigation
    );
    assert_eq!(
        a.activities.value.as_ref().unwrap()[0].attack.value,
        Some(23.into())
    );
    assert_eq!(
        a.activities.value.as_ref().unwrap()[0]
            .difficulty_class
            .value,
        Some(33.into())
    );
    let selected = f
        .worker
        .record_detail_at(RecordDetailRequest {
            spell_rank: None,
            record_key: local.navigation.record_key.clone(),
            owners: local.navigation.owners.clone(),
            fields: vec![],
            passage: None,
            source_fingerprint: local.navigation.source_fingerprint.clone(),
        })
        .unwrap();
    assert_eq!(selected.record.title, "Caster");
    assert_eq!(selected.presentation.identity.title, "Local Fireball");
    assert_eq!(selected.presentation.classification, "Spell 6");
    let RecordBodyView::Spell(s) = selected.presentation.body else {
        panic!("owned spell");
    };
    assert_eq!(s.authored_cast_rank.value, Some(6.into()));
    assert_eq!(
        s.damage.value.unwrap()[0].formula.value.as_deref(),
        Some("12d6")
    );
    let entry = &a.activities.value.as_ref().unwrap()[0];
    let selected = f
        .worker
        .record_detail_at(RecordDetailRequest {
            spell_rank: None,
            record_key: entry.navigation.record_key.clone(),
            owners: entry.navigation.owners.clone(),
            fields: vec![],
            passage: None,
            source_fingerprint: entry.navigation.source_fingerprint.clone(),
        })
        .unwrap();
    let RecordBodyView::Activity(entry) = selected.presentation.body else {
        panic!("owned casting entry");
    };
    assert_eq!(entry.attack.value, Some(23.into()));
    assert_eq!(entry.difficulty_class.value, Some(33.into()));
    assert_eq!(entry.preparation.value.as_deref(), Some("prepared"));
    assert_eq!(entry.casting_tradition.value.as_deref(), Some("arcane"));
    assert_eq!(selected.presentation.identity.kind_label, "Spellcasting");
    let context = entry.casting_context.unwrap();
    assert_eq!(context.parent_title, "Caster");
    assert!(context.parent.owners.is_empty());
    let members = context.spells.value.unwrap();
    assert_eq!(members.len(), 1);
    assert_eq!(members[0].title, "Local Fireball");
    assert_eq!(members[0].base_rank.value, Some(3.into()));
    assert_eq!(members[0].authored_cast_rank.value, Some(6.into()));
    assert_eq!(members[0].navigation, local.navigation);
    assert_eq!(
        members[0].damage.value.as_ref().unwrap()[0]
            .formula
            .value
            .as_deref(),
        Some("12d6")
    );
    assert!(
        a.activities.value.as_ref().unwrap()[0]
            .casting_context
            .is_none()
    );
}

#[test]
fn hazard_primary_defense_presence_requires_known_hp_and_ac_and_preserves_zero_saves() {
    for (system, state, presence) in [
        (
            json!({"attributes":{"hp":{"max":0},"ac":{"value":0}},"saves":{"will":{"value":0}}}),
            QueryFieldState::Value,
            Some(false),
        ),
        (
            json!({"attributes":{"hp":{"max":1},"ac":{"value":0}}}),
            QueryFieldState::Value,
            Some(true),
        ),
        (
            json!({"attributes":{"hp":{"max":0},"ac":{"value":14}}}),
            QueryFieldState::Value,
            Some(true),
        ),
        (
            json!({"attributes":{"hp":{"max":0}}}),
            QueryFieldState::Missing,
            None,
        ),
        (
            json!({"attributes":{"hp":{"max":0},"ac":null}}),
            QueryFieldState::Null,
            None,
        ),
        (
            json!({"attributes":{"hp":{"max":0},"ac":{"value":"broken"}}}),
            QueryFieldState::Invalid,
            None,
        ),
        (json!(null), QueryFieldState::Null, None),
    ] {
        let input = record(
            "hazards",
            "Actor",
            json!({"_id":"aaaaaaaaaaaaaaaa","type":"hazard","system":system}),
        );
        let node = SourceQueryView::new(input.record.source(), "hazards", "").source;
        let defense = hazard_defense_block(node);
        assert_eq!(defense.state, state);
        assert_eq!(defense.value, presence);
        if presence == Some(false) {
            let authored = actor::authored_actor(&input.record, node, "fixture");
            assert_eq!(authored.armor_class.value, Some(0.into()));
            assert_eq!(authored.maximum_hp.value, Some(0.into()));
            assert_eq!(authored.saves.will.value, Some(0.into()));
        }
    }
}

#[test]
fn hazard_save_omission_requires_the_complete_known_zero_blank_detail_shape() {
    let baseline = json!({"attributes":{"hp":{"max":0},"ac":{"value":0}},"saves":{"fortitude":{"value":0,"saveDetail":""},"reflex":{"value":0,"saveDetail":" \n"},"will":{"value":0,"saveDetail":""}}});
    let mut cases = vec![(baseline.clone(), false)];
    for value in [json!(null), json!(42), json!("resistance to magic")] {
        let mut system = baseline.clone();
        system["saves"]["will"]["saveDetail"] = value;
        cases.push((system, true));
    }
    let mut system = baseline.clone();
    system["saves"]["will"]
        .as_object_mut()
        .unwrap()
        .remove("saveDetail");
    cases.push((system, true));
    for value in [json!(1), json!(null), json!("broken")] {
        let mut system = baseline.clone();
        system["saves"]["will"]["value"] = value;
        cases.push((system, true));
    }
    for value in [json!(1), json!(null)] {
        let mut system = baseline.clone();
        system["attributes"]["hp"]["max"] = value;
        cases.push((system, true));
    }
    for (system, expected) in cases {
        let input = record(
            "hazards",
            "Actor",
            json!({"_id":"aaaaaaaaaaaaaaaa","type":"hazard","system":system}),
        );
        let node = SourceQueryView::new(input.record.source(), "hazards", "").source;
        assert_eq!(hazard_show_saves(node), expected);
        let actor = actor::authored_actor(&input.record, node, "fixture");
        assert_eq!(actor.saves.fortitude.value, Some(0.into()));
        assert_eq!(actor.saves.reflex.value, Some(0.into()));
    }
}

#[test]
fn encounter_save_adjustments_restore_an_omitted_legacy_hazard_block() {
    use crate::encounters::mechanics::{StatBlockView, StatValueView};
    let executor = RetrievalExecutor::from_test_fixture_factory(1, 16, || {
        Ok(open_source_fixture(
            vec![record(
                "hazards",
                "Actor",
                json!({"_id":"aaaaaaaaaaaaaaaa","name":"Haunt","type":"hazard","system":{"attributes":{"hp":{"max":0},"ac":{"value":0}},"saves":{"fortitude":{"value":0,"saveDetail":""},"reflex":{"value":0,"saveDetail":""},"will":{"value":0,"saveDetail":""}}}}),
            )],
            false,
        ).unwrap())
    });
    let fixture = fixture_worker_with_executor(executor);
    let detail = fixture
        .worker
        .record_detail("hazards:aaaaaaaaaaaaaaaa")
        .unwrap();
    let RecordBodyView::Hazard(authored) = &detail.presentation.body else {
        panic!("hazard")
    };
    assert!(!authored.show_saves);
    for (effective, explanation, expected) in [(0, false, false), (1, false, true), (0, true, true)]
    {
        let block = StatBlockView {
            record_key: detail.record.record_key.clone(),
            title: "Haunt".into(),
            level: None,
            adjusted_level: None,
            values: vec![StatValueView {
                target: "save.will".into(),
                label: "Will".into(),
                base_value: 0.into(),
                adjusted_value: effective.into(),
                modifiers: vec![],
                suppressed_modifiers: if explanation {
                    vec![StatModifierView {
                        source: "test".into(),
                        label: "Suppressed effect".into(),
                        modifier_type: StatModifierTypeView::Status,
                        value: 1,
                    }]
                } else {
                    vec![]
                },
            }],
            speeds: vec![],
            action_budget: None,
            activities: vec![],
            unapplied_effects: vec![],
        };
        let composed = compose_participant(detail.presentation.clone(), &block);
        let RecordBodyView::Hazard(hazard) = composed.body else {
            panic!("hazard")
        };
        assert_eq!(hazard.show_saves, expected);
        assert_eq!(hazard.actor.saves.will.value, Some(effective.into()));
    }
}

#[test]
fn selected_duplicate_casting_entry_does_not_guess_members_and_rejects_stale_owner() {
    let entry = json!({"_id":"eeeeeeeeeeeeeeee","type":"spellcastingEntry","name":"Duplicate entry","system":{"spelldc":{"dc":25,"value":15}}});
    let source = json!({"_id":"aaaaaaaaaaaaaaaa","type":"npc","name":"Caster","items":[entry.clone(),entry,{"_id":"ssssssssssssssss","type":"spell","name":"Unresolved local spell","system":{"level":{"value":2},"location":{"value":"eeeeeeeeeeeeeeee"}}}]});
    let executor = RetrievalExecutor::from_test_fixture_factory(1, 16, move || {
        Ok(open_source_fixture(
            vec![record("actors", "Actor", source.clone())],
            false,
        )?)
    });
    let fixture = fixture_worker_with_executor(executor);
    let root = fixture
        .worker
        .record_detail("actors:aaaaaaaaaaaaaaaa")
        .unwrap();
    let activities = actor(&root.presentation).activities.value.as_ref().unwrap();
    assert!(activities[2].casting_entry.is_none());
    let navigation = &activities[0].navigation;
    let request = RecordDetailRequest {
        spell_rank: None,
        record_key: navigation.record_key.clone(),
        owners: navigation.owners.clone(),
        fields: vec![],
        passage: None,
        source_fingerprint: navigation.source_fingerprint.clone(),
    };
    assert!(request.source_fingerprint.is_some());
    let selected = fixture.worker.record_detail_at(request.clone()).unwrap();
    let RecordBodyView::Activity(entry) = selected.presentation.body else {
        panic!("entry");
    };
    assert!(
        entry
            .casting_context
            .unwrap()
            .spells
            .value
            .unwrap()
            .is_empty()
    );
    let stale = RecordDetailRequest {
        spell_rank: None,
        source_fingerprint: Some("previous-snapshot".into()),
        ..request
    };
    assert!(fixture.worker.record_detail_at(stale).is_err());
}

#[test]
fn activity_reference_details_use_selected_body_and_declared_content_not_names() {
    let items = json!([
        {"_id":"aaaaaaaaaaaaaaa1","type":"melee","name":"Staff","system":{"description":{"value":"","gm":""},"bonus":{"value":10},"damageRolls":{"one":{"damage":"1d4+2","damageType":"bludgeoning"}}}},
        {"_id":"aaaaaaaaaaaaaaa2","type":"melee","name":"Staff","system":{"description":{"value":"<p>Knocks the target back.</p>"}}},
        {"_id":"aaaaaaaaaaaaaaa3","type":"action","name":"Custom action","system":{"description":{"value":"<p>Additional action rules.</p>"}}},
        {"_id":"aaaaaaaaaaaaaaa4","type":"action","name":"Custom action","system":{"description":{"value":" \n\t "}}},
        {"_id":"aaaaaaaaaaaaaaa5","type":"weapon","name":"Staff","system":{"description":{"value":""}}},
        {"_id":"aaaaaaaaaaaaaaa6","type":"spell","name":"Spell","system":{"description":{"value":""},"level":{"value":1}}},
        {"_id":"aaaaaaaaaaaaaaa7","type":"melee","name":"Unknown description","system":{}},
        {"_id":"aaaaaaaaaaaaaaa8","type":"action","name":"Custom null body","system":{"description":{"value":null}}},
        {"_id":"aaaaaaaaaaaaaaa9","type":"action","name":"Custom invalid body","system":{"description":{"value":42}}},
        {"_id":"aaaaaaaaaaaaaa10","type":"melee","name":"Unavailable system","system":42},
        {"_id":"aaaaaaaaaaaaaa11","type":"melee","name":"Markup only","system":{"description":{"value":"<p></p>"}}},
        {"_id":"aaaaaaaaaaaaaa12","type":"melee","name":"GM details","system":{"description":{"value":"","gm":"<p>Additional GM detail.</p>"}}},
        {"_id":"aaaaaaaaaaaaaa13","type":"spellcastingEntry","name":"Spellcasting","system":{"description":{"value":""}}}
    ]);
    let input = record(
        "actors",
        "Actor",
        json!({"_id":"aaaaaaaaaaaaaaaa","type":"npc","items":items}),
    );
    let body = actor::authored_actor(&input.record, input.record.node_at(&[]).unwrap(), "fixture");
    let activities = body.activities.value.unwrap();
    assert_eq!(activities.len(), 13);
    let flags = activities
        .iter()
        .map(|activity| activity.has_reference_details)
        .collect::<Vec<_>>();
    assert_eq!(
        flags,
        [
            false, true, true, false, true, true, true, true, true, true, true, true, true
        ]
    );
    assert_eq!(activities[0].title, activities[1].title);
    assert_eq!(activities[2].title, activities[3].title);
    assert_eq!(activities[0].title, activities[4].title);
}

#[test]
fn reference_details_flag_agrees_with_actual_selected_spell_and_physical_dispatch() {
    let source = json!({"_id":"aaaaaaaaaaaaaaaa","type":"npc","items":[
        {"_id":"bbbbbbbbbbbbbbbb","type":"melee","name":"Staff","system":{"description":{"value":""}}},
        {"_id":"cccccccccccccccc","type":"weapon","name":"Staff","system":{"description":{"value":""}}},
        {"_id":"dddddddddddddddd","type":"spell","name":"Spell","system":{"description":{"value":""},"level":{"value":1}}}
    ]});
    let executor = RetrievalExecutor::from_test_fixture_factory(1, 16, move || {
        Ok(open_source_fixture(
            vec![record("actors", "Actor", source.clone())],
            false,
        )?)
    });
    let f = fixture_worker_with_executor(executor);
    let parent = f.worker.record_detail("actors:aaaaaaaaaaaaaaaa").unwrap();
    let activities = actor(&parent.presentation)
        .activities
        .value
        .as_ref()
        .unwrap();
    for (index, activity) in activities.iter().enumerate() {
        let selected = f
            .worker
            .record_detail_at(RecordDetailRequest {
                spell_rank: None,
                record_key: activity.navigation.record_key.clone(),
                owners: activity.navigation.owners.clone(),
                fields: vec![],
                passage: None,
                source_fingerprint: activity.navigation.source_fingerprint.clone(),
            })
            .unwrap();
        match (index, selected.presentation.body) {
            (0, RecordBodyView::Activity(summary)) => {
                assert!(!activity.has_reference_details);
                assert!(!summary.has_reference_details);
            }
            (1, RecordBodyView::PhysicalReference(_)) | (2, RecordBodyView::Spell(_)) => {
                assert!(activity.has_reference_details)
            }
            _ => panic!("selected body diverged from reference-details policy"),
        }
    }
}

#[test]
fn heightened_identity_uses_effective_traits_without_rewriting_source() {
    let source = json!({"_id":"aaaaaaaaaaaaaaaa","name":"Local spell","type":"spell","system":{
        "level":{"value":1},"traits":{"value":["fire"],"rarity":"common"},
        "range":{"value":"30 feet"},"heightening":{"type":"fixed","levels":{
            "3":{"traits":{"value":["cold"],"rarity":"rare"},"range":{"value":"60 feet"}}
        }}
    }});
    let f = fixture_worker_with_executor(RetrievalExecutor::from_test_fixture_factory(
        1,
        16,
        move || {
            Ok(open_source_fixture(
                vec![record("spells", "Item", source.clone())],
                false,
            )?)
        },
    ));
    let projected = f
        .worker
        .record_detail_at(RecordDetailRequest {
            record_key: "spells:aaaaaaaaaaaaaaaa".into(),
            spell_rank: Some(3),
            owners: vec![],
            fields: vec![],
            passage: None,
            source_fingerprint: None,
        })
        .unwrap();
    assert_eq!(
        projected.presentation.identity.rarity.as_deref(),
        Some("rare")
    );
    assert_eq!(
        projected
            .presentation
            .identity
            .traits
            .iter()
            .map(|t| t.value.as_str())
            .collect::<Vec<_>>(),
        vec!["cold"]
    );
    let original = f.worker.record_detail("spells:aaaaaaaaaaaaaaaa").unwrap();
    assert_eq!(
        original.presentation.identity.rarity.as_deref(),
        Some("common")
    );
    assert_eq!(original.presentation.identity.traits[0].value, "fire");
}

#[test]
fn unsupported_heightening_is_visible_and_does_not_claim_effective_rank() {
    for heightening in [
        json!({"type":"interval","interval":1,"area":2.5}),
        json!({"type":"fixed","levels":{"3":{"description":{"value":"Changed description"}}}}),
    ] {
        let source = json!({"_id":"aaaaaaaaaaaaaaaa","type":"spell","name":"Spell","system":{
            "level":{"value":1},"area":{"type":"burst","value":10},"heightening":heightening
        }});
        let f = fixture_worker_with_executor(RetrievalExecutor::from_test_fixture_factory(
            1,
            16,
            move || {
                Ok(open_source_fixture(
                    vec![record("spells", "Item", source.clone())],
                    false,
                )?)
            },
        ));
        let projected = f
            .worker
            .record_detail_at(RecordDetailRequest {
                record_key: "spells:aaaaaaaaaaaaaaaa".into(),
                spell_rank: Some(3),
                owners: vec![],
                fields: vec![],
                passage: None,
                source_fingerprint: None,
            })
            .unwrap();
        assert_eq!(projected.selected.spell_rank, None);
        let RecordBodyView::Spell(spell) = projected.presentation.body else {
            panic!("spell")
        };
        assert_eq!(spell.rank.value, Some(1.into()));
        assert!(spell.rank_note.unwrap().contains("preview is unavailable"));
    }
}

#[test]
fn reference_defense_notes_preserve_qualifiers_without_changing_statistics() {
    let input = record(
        "actors",
        "Actor",
        json!({
            "_id":"aaaaaaaaaaaaaaaa", "type":"npc",
            "system": {"attributes": {
                "ac":{"value":25,"details":"21 while broken"},
                "hp":{"max":80,"details":"void healing"},
                "allSaves":{"value":"+2 status vs. magic"}},
                "saves":{"fortitude":{"value":10,"saveDetail":"-2 without heartstone"},
                         "reflex":{"value":0,"saveDetail":""},"will":{"value":8,"saveDetail":42}}}
        }),
    );
    let body = actor::authored_actor(&input.record, input.record.node_at(&[]).unwrap(), "fixture");
    assert_eq!(body.armor_class.value, Some(25.into()));
    assert_eq!(body.saves.fortitude.value, Some(10.into()));
    assert_eq!(
        body.defense_notes.armor_class.value.as_deref(),
        Some("21 while broken")
    );
    assert_eq!(
        body.defense_notes.hit_points.value.as_deref(),
        Some("void healing")
    );
    assert_eq!(
        body.defense_notes.all_saves.value.as_deref(),
        Some("+2 status vs. magic")
    );
    assert_eq!(
        body.defense_notes.fortitude.value.as_deref(),
        Some("-2 without heartstone")
    );
    assert_eq!(body.defense_notes.reflex.value.as_deref(), Some(""));
    assert_eq!(body.defense_notes.will.state, QueryFieldState::Invalid);
}

#[test]
fn reference_frequency_uses_capacity_not_remaining_uses_and_preserves_invalid_interval() {
    for (interval, expected) in [
        (json!("PT1M"), QueryFieldState::Value),
        (json!("invented"), QueryFieldState::Invalid),
    ] {
        let input = record(
            "actors",
            "Actor",
            json!({
                "_id":"aaaaaaaaaaaaaaaa","type":"npc", "items":[{
                    "_id":"bbbbbbbbbbbbbbbb","type":"action","name":"Ward",
                    "system":{"frequency":{"max":3,"value":0,"per":interval}}
                }]
            }),
        );
        let body =
            actor::authored_actor(&input.record, input.record.node_at(&[]).unwrap(), "fixture");
        let activities = body.activities.value.unwrap();
        let frequency = activities[0].frequency.value.as_ref().unwrap();
        assert_eq!(frequency.maximum.value, Some(3.into()));
        assert_eq!(frequency.interval.state, expected);
        if expected == QueryFieldState::Value {
            assert_eq!(frequency.interval.value.as_deref(), Some("minute"));
        }
    }
}

#[test]
fn reference_casting_capacity_requires_focus_context_and_ritual_dc_stays_separate() {
    let input = record(
        "actors",
        "Actor",
        json!({
            "_id":"aaaaaaaaaaaaaaaa","type":"npc",
            "system":{"traits":{"size":{"value":"lg"}},"attributes":{"hardness":{"value":0}},
                "resources":{"focus":{"max":3,"value":0}},"spellcasting":{"rituals":{"dc":23}}},
            "items":[
                {"_id":"bbbbbbbbbbbbbbbb","type":"spellcastingEntry","system":{"prepared":{"value":"focus"},"spelldc":{"dc":27}}},
                {"_id":"cccccccccccccccc","type":"spellcastingEntry","system":{"prepared":{"value":"innate"},"spelldc":{"dc":25}}}
            ]
        }),
    );
    let body = actor::authored_actor(&input.record, input.record.node_at(&[]).unwrap(), "fixture");
    assert_eq!(body.size.value.as_deref(), Some("Large"));
    assert_eq!(body.hardness.value, Some(0.into()));
    assert_eq!(body.ritual_difficulty_class.value, Some(23.into()));
    let activities = body.activities.value.unwrap();
    assert_eq!(activities[0].focus_capacity.value, Some(3.into()));
    assert_eq!(activities[0].difficulty_class.value, Some(27.into()));
    assert_eq!(
        activities[1].focus_capacity.state,
        QueryFieldState::NotApplicable
    );
    assert_eq!(activities[1].difficulty_class.value, Some(25.into()));
}

#[test]
fn reference_optional_actor_facts_keep_missing_null_invalid_and_zero_distinct() {
    for (hardness, expected) in [
        (None, QueryFieldState::Missing),
        (Some(json!(null)), QueryFieldState::Null),
        (Some(json!("bad")), QueryFieldState::Invalid),
        (Some(json!({"value":0})), QueryFieldState::Value),
        (Some(json!({"value":8})), QueryFieldState::Value),
        (Some(json!({"value":null})), QueryFieldState::Null),
        (Some(json!({"value":"bad"})), QueryFieldState::Invalid),
        (Some(json!({})), QueryFieldState::Missing),
        (Some(json!(8)), QueryFieldState::Invalid),
    ] {
        let mut attrs = json!({});
        if let Some(value) = hardness {
            attrs["hardness"] = value;
        }
        let input = record(
            "actors",
            "Actor",
            json!({"_id":"aaaaaaaaaaaaaaaa","type":"npc","system":{"attributes":attrs}}),
        );
        let body =
            actor::authored_actor(&input.record, input.record.node_at(&[]).unwrap(), "fixture");
        assert_eq!(body.hardness.state, expected);
        if let Some(value) = attrs["hardness"]["value"].as_i64() {
            assert_eq!(body.hardness.value, Some(value.into()));
        }
        assert_eq!(body.ritual_difficulty_class.state, QueryFieldState::Missing);
        assert_eq!(body.size.state, QueryFieldState::Missing);
    }
}

#[test]
fn reference_self_effect_navigation_survives_storage_and_selected_activity_reading() {
    let f = fixture_worker_with_executor(RetrievalExecutor::from_test_fixture_factory(
        1,
        16,
        || {
            Ok(open_source_fixture(
                vec![
                    record(
                        "actors",
                        "Actor",
                        json!({"_id":"aaaaaaaaaaaaaaaa","type":"npc","items":[
                            {"_id":"cccccccccccccccc","type":"action","name":"Ward","system":{"selfEffect":{"name":"Ward effect","uuid":"Compendium.pf2e.effects.Item.bbbbbbbbbbbbbbbb"}}},
                            {"_id":"dddddddddddddddd","type":"melee","system":{"attackEffects":{"value":["unresolved-poison"]}}}
                        ]}),
                    ),
                    record(
                        "effects",
                        "Item",
                        json!({"_id":"bbbbbbbbbbbbbbbb","type":"effect","name":"Effect: Ward"}),
                    ),
                ],
                false,
            )?)
        },
    ));
    let detail = f.worker.record_detail("actors:aaaaaaaaaaaaaaaa").unwrap();
    let activities = actor(&detail.presentation)
        .activities
        .value
        .as_ref()
        .unwrap();
    let effect = activities[0].self_effect.value.as_ref().unwrap();
    assert_eq!(
        effect.navigation.as_ref().unwrap().record_key,
        "effects:bbbbbbbbbbbbbbbb"
    );
    assert_eq!(
        activities[1].strike_effects.value.as_ref().unwrap(),
        &["unresolved-poison"]
    );
    let selected = f
        .worker
        .record_detail_at(atlas_app_model::RecordDetailRequest {
            record_key: "actors:aaaaaaaaaaaaaaaa".into(),
            owners: activities[0].navigation.owners.clone(),
            spell_rank: None,
            fields: vec![],
            passage: None,
            source_fingerprint: activities[0].navigation.source_fingerprint.clone(),
        })
        .unwrap();
    let RecordBodyView::Activity(selected) = selected.presentation.body else {
        panic!("activity")
    };
    assert_eq!(
        selected.self_effect.value.unwrap().navigation,
        effect.navigation
    );
}

#[test]
fn ritual_casting_preserves_compound_checks_zero_casters_and_unavailability() {
    for (ritual, expected) in [
        (
            json!({"primary":{"check":"Religion (expert) or Occultism (master)"},"secondary":{"casters":0,"checks":"Medicine; Society"}}),
            QueryFieldState::Value,
        ),
        (json!(null), QueryFieldState::NotApplicable),
        (json!(42), QueryFieldState::Invalid),
    ] {
        let input = record(
            "spells",
            "Item",
            json!({"_id":"aaaaaaaaaaaaaaaa","type":"spell","system":{"ritual":ritual,"counteraction":true}}),
        );
        let SourceNodeView::Item(ItemSourceView::SpellSource(source)) =
            SourceQueryView::new(input.record.source(), "spells", "").source
        else {
            panic!("spell")
        };
        let view = spell::authored_spell(source);
        assert_eq!(view.ritual.state, expected);
        assert_eq!(view.counteraction.value, Some(true));
        if let Some(ritual) = view.ritual.value {
            assert_eq!(
                ritual.primary_check.value.as_deref(),
                Some("Religion (expert) or Occultism (master)")
            );
            assert_eq!(ritual.secondary_casters.value, Some(0.into()));
            assert_eq!(
                ritual.secondary_checks.value.as_deref(),
                Some("Medicine; Society")
            );
        }
    }
}

#[test]
fn effect_reference_duration_handles_special_units_and_invalid_timed_amounts() {
    for (unit, value, expected, state) in [
        (
            "encounter",
            -1,
            Some("Until the encounter ends"),
            QueryFieldState::Value,
        ),
        ("unlimited", -1, Some("Unlimited"), QueryFieldState::Value),
        ("rounds", 1, Some("1 round"), QueryFieldState::Value),
        ("minutes", 2, Some("2 minutes"), QueryFieldState::Value),
        ("rounds", -1, None, QueryFieldState::Invalid),
    ] {
        let input = record(
            "effects",
            "Item",
            json!({"_id":"aaaaaaaaaaaaaaaa","type":"effect","system":{"duration":{"unit":unit,"value":value,"sustained":true,"expiry":"turn-end"}}}),
        );
        let SourceNodeView::Item(item) =
            SourceQueryView::new(input.record.source(), "effects", "").source
        else {
            panic!("item")
        };
        let view = item_reference::effect_duration(item).value.unwrap();
        assert_eq!(view.duration.state, state);
        assert_eq!(view.duration.value.as_deref(), expected);
        assert_eq!(view.sustained.value, Some(true));
        assert_eq!(
            view.expiry.value.as_deref(),
            if matches!(unit, "encounter" | "unlimited") {
                None
            } else {
                Some("At the end of the turn")
            }
        );
    }
}

#[test]
fn feat_prerequisites_preserve_alternatives_and_missing_members() {
    let input = record(
        "feats",
        "Item",
        json!({"_id":"aaaaaaaaaaaaaaaa","type":"feat","system":{"prerequisites":{"value":[{"value":"trained in Medicine"},{"value":"Battle Medicine or Medic Dedication"},{}]}}}),
    );
    let SourceNodeView::Item(item) =
        SourceQueryView::new(input.record.source(), "feats", "").source
    else {
        panic!("item")
    };
    let view: Vec<_> = SourceNodeView::Item(item)
        .content_selections()
        .into_iter()
        .filter(|s| s.role == atlas_record::source_record::SourceContentRole::Prerequisite)
        .collect();
    assert_eq!(view.len(), 3);
    assert_eq!(
        view[0].text.value().map(String::as_str),
        Some("trained in Medicine")
    );
    assert_eq!(
        view[1].text.value().map(String::as_str),
        Some("Battle Medicine or Medic Dedication")
    );
    assert!(matches!(view[2].text, SourceFieldView::Missing));
    assert_eq!(view[2].field, "/system/prerequisites/value/2/value");
}

#[test]
fn rich_prerequisites_are_prepared_attributed_and_keep_unavailable_entries() {
    let fixture = fixture_worker_with_executor(RetrievalExecutor::from_test_fixture_factory(
        1,
        16,
        || {
            Ok(open_source_fixture(
                vec![
                    record(
                        "feats",
                        "Item",
                        json!({"_id":"aaaaaaaaaaaaaaaa","name":"Prerequisite test","type":"feat","system":{"prerequisites":{"value":[
                            {"value":"@UUID[Compendium.pf2e.feats.Item.bbbbbbbbbbbbbbbb]{Battle Medicine} or trained in Medicine"},
                            {"value":"<script>bad()</script><strong>expert</strong> in Medicine"}, {}, {"value":null}, {"value":42}, {"value":""}
                        ]}}}),
                    ),
                    record(
                        "feats",
                        "Item",
                        json!({"_id":"bbbbbbbbbbbbbbbb","name":"Battle Medicine","type":"feat","system":{}}),
                    ),
                ],
                false,
            )?)
        },
    ));
    let detail = fixture
        .worker
        .record_detail("feats:aaaaaaaaaaaaaaaa")
        .unwrap();
    let fields: Vec<_> = detail
        .presentation
        .content
        .iter()
        .filter(|f| f.role == "prerequisite")
        .collect();
    assert_eq!(fields.len(), 6);
    for (i, field) in fields.iter().enumerate() {
        assert_eq!(
            field.locator.field,
            format!("/system/prerequisites/value/{i}/value")
        );
    }
    let PreparedFieldBodyView::Html { html, .. } = &fields[0].body else {
        panic!("prepared prerequisite")
    };
    assert!(html.contains("Battle Medicine"));
    assert!(html.contains("or trained in Medicine"));
    assert!(!html.contains("@UUID"));
    assert!(detail.relationships.iter().any(|r| {
        r.target
            .as_ref()
            .is_some_and(|t| t.record_key == "feats:bbbbbbbbbbbbbbbb")
    }));
    let PreparedFieldBodyView::Html { html, .. } = &fields[1].body else {
        panic!("prepared HTML")
    };
    assert!(html.contains("<strong>expert</strong>"));
    assert!(!html.contains("<script"));
    for (index, expected) in [
        (2, QueryFieldState::Missing),
        (3, QueryFieldState::Null),
        (4, QueryFieldState::Invalid),
    ] {
        assert!(
            matches!(fields[index].body, PreparedFieldBodyView::Unavailable { state } if state == expected)
        );
    }
}

#[test]
fn journal_page_collections_distinguish_empty_missing_null_and_invalid() {
    for (pages, expected) in [
        (Some(json!([])), QueryFieldState::Value),
        (None, QueryFieldState::Missing),
        (Some(json!(null)), QueryFieldState::Null),
        (Some(json!(42)), QueryFieldState::Invalid),
    ] {
        let mut journal = json!({"_id":"aaaaaaaaaaaaaaaa","name":"Journal"});
        if let Some(pages) = pages {
            journal["pages"] = pages;
        }
        let fixture = fixture_worker_with_executor(RetrievalExecutor::from_test_fixture_factory(
            1,
            16,
            move || {
                Ok(open_source_fixture(
                    vec![record("journals", "JournalEntry", journal.clone())],
                    false,
                )?)
            },
        ));
        let detail = fixture
            .worker
            .record_detail("journals:aaaaaaaaaaaaaaaa")
            .unwrap();
        let RecordBodyView::Journal(view) = detail.presentation.body else {
            panic!("journal")
        };
        assert_eq!(view.pages.state, expected);
        if expected == QueryFieldState::Value {
            assert!(view.pages.value.unwrap().is_empty());
        }
    }
}

#[test]
fn unavailable_prerequisite_collections_remain_visible_in_selected_reference() {
    for (prerequisites, expected) in [
        (None, QueryFieldState::Missing),
        (Some(json!(null)), QueryFieldState::Null),
        (Some(json!({"value":42})), QueryFieldState::Invalid),
    ] {
        let mut system = json!({});
        if let Some(value) = prerequisites {
            system["prerequisites"] = value;
        }
        let fixture = fixture_worker_with_executor(RetrievalExecutor::from_test_fixture_factory(
            1,
            16,
            move || {
                Ok(open_source_fixture(
                    vec![record(
                        "feats",
                        "Item",
                        json!({"_id":"aaaaaaaaaaaaaaaa","type":"feat","name":"Unavailable requirements","system":system}),
                    )],
                    false,
                )?)
            },
        ));
        let detail = fixture
            .worker
            .record_detail("feats:aaaaaaaaaaaaaaaa")
            .unwrap();
        let field = detail
            .presentation
            .content
            .iter()
            .find(|f| f.role == "prerequisite")
            .unwrap();
        assert_eq!(field.locator.field, "/system/prerequisites/value");
        assert!(
            matches!(field.body, PreparedFieldBodyView::Unavailable { state } if state == expected)
        );
    }
}

#[test]
fn consumable_spell_navigation_keeps_exact_nested_identity_and_authored_rank() {
    for nested in [false, true] {
        let item = json!({"_id":"cccccccccccccccc","name":"Custom wand","type":"consumable","system":{
            "category":"wand","spell":{"_id":"ssssssssssssssss","name":"Customized Heal","type":"spell","system":{
                "level":{"value":1},"traits":{"value":["cantrip"]},"location":{"heightenedLevel":2},"description":{"value":"Local customization"}
            }}
        }});
        let value = if nested {
            json!({"_id":"aaaaaaaaaaaaaaaa","type":"npc","system":{"details":{"level":{"value":20}}},"items":[item]})
        } else {
            item
        };
        let input = record("test", if nested { "Actor" } else { "Item" }, value);
        let mut selected = vec![];
        if nested {
            input.record.visit_immediate_actor_items(|_, owner, _| {
                selected.push(owner.clone());
            });
        }
        let node = input.record.node_at(&selected).unwrap();
        let links = owned_links(&input.record, node, &selected, "fingerprint");
        assert_eq!(links.len(), 1);
        assert_eq!(links[0].title, "Customized Heal");
        assert_eq!(links[0].navigation.owners.len(), selected.len() + 1);
        let SourceNodeView::Item(ItemSourceView::SpellSource(spell)) =
            input.record.node_at(&links[0].navigation.owners).unwrap()
        else {
            panic!("spell")
        };
        assert_eq!(spell_context::ranks(&input.record, spell), vec![2]);
        assert_eq!(spell_context::base_rank(spell), Some(1));
    }
}

#[test]
fn empty_scroll_has_no_invented_spell_link() {
    let input = record(
        "test",
        "Item",
        json!({"_id":"cccccccccccccccc","type":"consumable","system":{"category":"scroll","spell":null}}),
    );
    assert!(
        owned_links(
            &input.record,
            input.record.node_at(&[]).unwrap(),
            &[],
            "fingerprint"
        )
        .is_empty()
    );
}
