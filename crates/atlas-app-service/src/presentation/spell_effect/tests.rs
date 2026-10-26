use super::*;
use crate::{executor::RetrievalExecutor, test_support::fixture_worker_with_executor};
use atlas_search::test_support::{open_source_fixture, record};
use serde_json::{Value, json};
fn bear() -> Value {
    serde_json::from_str(include_str!(
        "../../../tests/fixtures/spell-effect-animal-form-bear.json"
    ))
    .unwrap()
}
fn enlarge() -> Value {
    serde_json::from_str(include_str!(
        "../../../tests/fixtures/spell-effect-enlarge.json"
    ))
    .unwrap()
}
fn worker(effect: Value) -> crate::test_support::FixtureWorker {
    fixture_worker_with_executor(RetrievalExecutor::from_test_fixture_factory(
        1,
        16,
        move || {
            Ok(open_source_fixture(
                vec![
                    record("effects", "Item", effect.clone()),
                    record(
                        "spells",
                        "Item",
                        json!({"_id":"ssssssssssssssss","type":"spell","name":"Test spell","system":{"level":{"value":2},"description":{"value":format!("<p>@UUID[Compendium.pf2e.effects.Item.{}]{{Bear}}</p>",effect["_id"].as_str().unwrap())}}}),
                    ),
                ],
                false,
            )?)
        },
    ))
}
fn request(key: &str, rank: Option<u8>) -> RecordDetailRequest {
    RecordDetailRequest {
        record_key: key.into(),
        spell_rank: rank,
        owners: vec![],
        fields: vec![],
        passage: None,
        source_fingerprint: None,
    }
}
fn preview(view: &RecordDetailView) -> &SpellEffectPreviewView {
    let RecordBodyView::Activity(a) = &view.presentation.body else {
        panic!("effect activity")
    };
    a.spell_effect.as_ref().unwrap()
}
fn fact_value(view: &SpellEffectPreviewView, name: &str) -> String {
    view.facts
        .iter()
        .find(|f| f.label == name)
        .unwrap()
        .value
        .value
        .clone()
        .unwrap()
}
#[test]
fn exact_effect_link_carries_selected_rank_without_replacing_authored_destination() {
    let fixture = worker(bear());
    let spell = fixture
        .worker
        .record_detail_at(request("spells:ssssssssssssssss", Some(4)))
        .unwrap();
    let link = spell
        .relationships
        .iter()
        .find(|r| r.origin == "content")
        .unwrap();
    let authored = link.target.as_ref().unwrap();
    let context = link.contextual_target.as_ref().unwrap();
    assert_eq!(authored.record_key, context.record_key);
    assert_eq!(authored.spell_rank, None);
    assert_eq!(context.spell_rank, Some(4));
    let detail = fixture
        .worker
        .record_detail_at(request(&context.record_key, context.spell_rank))
        .unwrap();
    assert_eq!(detail.selected.spell_rank, Some(4));
    let p = preview(&detail);
    assert!(p.unavailable.is_none());
    assert_eq!(fact_value(p, "Size"), "Large");
    assert_eq!(fact_value(p, "Temporary HP"), "15");
    assert_eq!(fact_value(p, "AC"), "18 + your level");
    assert!(detail.presentation.content.iter().any(|f|matches!(&f.body,PreparedFieldBodyView::Html{html,..} if html.contains("Medium animal battle form"))));
}
#[test]
fn battle_form_brackets_replace_each_other_and_preserve_base_dice_types_and_traits() {
    let fixture = worker(bear());
    for (rank, hp, size, attack, damage) in [
        (2, "5", "Medium", "+9", "2d8+1 piercing"),
        (3, "10", "Medium", "+14", "2d8+5 piercing"),
        (4, "15", "Large", "+16", "2d8+9 piercing"),
        (5, "20", "Huge", "+18", "4d8+7 piercing"),
        (6, "20", "Huge", "+18", "4d8+7 piercing"),
    ] {
        let detail = fixture
            .worker
            .record_detail_at(request("effects:gQnDKDeBTtjwOWAk", Some(rank)))
            .unwrap();
        let p = preview(&detail);
        assert!(p.unavailable.is_none(), "{rank}: {:?}", p.unavailable);
        assert_eq!(fact_value(p, "Temporary HP"), hp);
        assert_eq!(fact_value(p, "Size"), size);
        let jaws = p.strikes.iter().find(|s| s.id == "jaws").unwrap();
        assert_eq!(jaws.attack.value.value.as_deref(), Some(attack));
        assert_eq!(jaws.damage.value.value.as_deref(), Some(damage));
        assert!(jaws.traits.iter().any(|t| t.value == "unarmed"));
        assert_eq!(jaws.attack.base_value.is_some(), rank > 2);
        assert!(
            jaws.attack
                .qualifier
                .as_ref()
                .unwrap()
                .contains("if higher")
        );
    }
}
#[test]
fn enlarge_preserves_both_eligible_versions_and_labels_fixed_reach_honestly() {
    let fixture = worker(enlarge());
    for (rank, expected) in [
        (2, vec!["Large"]),
        (4, vec!["Huge"]),
        (6, vec!["Huge", "Large"]),
    ] {
        let detail = fixture
            .worker
            .record_detail_at(request("effects:sPCWrhUHqlbGhYSD", Some(rank)))
            .unwrap();
        let p = preview(&detail);
        assert!(p.unavailable.is_none(), "{:?}", p.unavailable);
        if expected.len() == 1 {
            assert!(p.choices.is_empty());
            assert_eq!(fact_value(p, "Size"), expected[0]);
            assert!(p.facts.iter().any(|f| f.label == "Reach (fixed)"));
        } else {
            assert_eq!(
                p.choices
                    .iter()
                    .map(|c| c.label.as_str())
                    .collect::<Vec<_>>(),
                expected
            );
            assert!(p.facts.is_empty());
        }
    }
}
#[test]
fn invalid_rank_brackets_and_actor_dependent_choices_remain_unavailable() {
    let mut broken = bear();
    broken["system"]["rules"][0]["brackets"][2]["value"]["strikes"]["jaws"]["damage"]["dice"] =
        json!("not a die count");
    let fixture = worker(broken);
    let detail = fixture
        .worker
        .record_detail_at(request("effects:gQnDKDeBTtjwOWAk", Some(4)))
        .unwrap();
    assert!(preview(&detail).unavailable.is_some());
    assert!(preview(&detail).facts.is_empty());
    let mut conditional = enlarge();
    conditional["system"]["rules"][0]["choices"][0]["predicate"] = json!(["self:effect:unmodeled"]);
    let fixture = worker(conditional);
    let detail = fixture
        .worker
        .record_detail_at(request("effects:sPCWrhUHqlbGhYSD", Some(6)))
        .unwrap();
    assert!(preview(&detail).unavailable.is_some());
    assert!(preview(&detail).choices.is_empty());
}
