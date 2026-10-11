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
    assert_eq!(s.cast.value.as_deref(), Some("1 to 3 actions"));
    assert_eq!(
        s.area.value.as_ref().unwrap().size.value.as_deref(),
        Some("30 feet")
    );
    assert_eq!(s.defense.value.as_ref().unwrap().basic.value, Some(true));
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
    assert_eq!(changes.cast.value.as_deref(), Some("2 actions"));
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
            record_key: local.navigation.record_key.clone(),
            owners: local.navigation.owners.clone(),
            fields: vec![],
            passage: None,
            source_fingerprint: local.navigation.source_fingerprint.clone(),
        })
        .unwrap();
    assert_eq!(selected.record.title, "Caster");
    assert_eq!(selected.presentation.identity.title, "Local Fireball");
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
}
