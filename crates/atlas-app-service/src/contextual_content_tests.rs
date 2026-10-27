use crate::{executor::RetrievalExecutor, test_support::fixture_worker_with_executor};
use atlas_app_model::*;
use atlas_search::test_support::{open_source_fixture, record};
use serde_json::json;

fn html(view: &RecordDetailView) -> &str {
    view.presentation
        .content
        .iter()
        .find_map(|f| match &f.body {
            PreparedFieldBodyView::Html { html, .. } => Some(html.as_str()),
            _ => None,
        })
        .unwrap()
}
fn roll(view: &RecordDetailView) -> RollContentRequest {
    view.presentation
        .content
        .iter()
        .find_map(|f| match &f.body {
            PreparedFieldBodyView::Html { controls, .. } => {
                controls.iter().find_map(|c| match &c.control {
                    ContentControlKindView::Damage { roll, .. }
                    | ContentControlKindView::Command { roll, .. } => roll.clone(),
                    _ => None,
                })
            }
            _ => None,
        })
        .unwrap()
}
fn selection(key: &str) -> RecordDetailRequest {
    RecordDetailRequest {
        record_key: key.into(),
        owners: vec![],
        fields: vec![],
        passage: None,
        spell_rank: None,
        source_fingerprint: None,
    }
}
#[test]
fn selected_rank_resolves_both_spell_aliases_without_borrowing_an_actor() {
    let f = fixture_worker_with_executor(RetrievalExecutor::from_test_fixture_factory(
        1,
        16,
        || {
            Ok(open_source_fixture(
                vec![record(
                    "spells",
                    "Item",
                    json!({"_id":"contextSpell0001","name":"Context spell","type":"spell","system":{"level":{"value":1},"description":{"value":"<p>@Damage[(max(8,@item.rank*2))d6[mental]]; @Damage[@item.level[fire]]; @Damage[(ceil(@actor.level/2))d6[untyped]]</p>"}}}),
                )],
                false,
            )?)
        },
    ));
    let mut request = selection("spells:contextSpell0001");
    request.spell_rank = Some(5);
    let view = f.worker.record_detail_at(request.clone()).unwrap();
    assert!(html(&view).contains(">10d6</span> mental damage"));
    assert!(html(&view).contains("5 fire damage"));
    assert!(html(&view).contains("half the creature’s level (rounded up)"));
    let canonical = f
        .worker
        .with_retrieval(&crate::ReadExecutionContext::uncancellable(), |r| {
            Ok(
                r.read_attributed_content(&[atlas_record::source_content::SourceContentLocator {
                    record: atlas_domain::RecordKey::parse("spells:contextSpell0001").unwrap(),
                    owners: vec![],
                    field: "/system/description/value".into(),
                }])?
                .content
                .fields[0]
                    .html
                    .clone()
                    .unwrap(),
            )
        })
        .unwrap();
    let section = atlas_record::source_record::select_prepared_html_sections(&canonical)
        .unwrap()
        .remove(0);
    let mut addressed = request.clone();
    addressed.fields = vec!["/system/description/value".into()];
    addressed.passage = Some(atlas_domain::SourcePassageAddress::HtmlSection {
        prepared_html_sha256: atlas_search::test_support::hash(canonical.as_bytes()),
        canonical_text_sha256: atlas_search::test_support::hash(section.text.as_bytes()),
        selection_version: atlas_record::source_record::SOURCE_CONTENT_SELECTION_VERSION.into(),
        section_ordinal: 0,
        label: section.label,
        chunk_bytes: atlas_domain::SourceByteRange {
            start: 0,
            end: section.text.len(),
        },
    });
    let addressed_view = f.worker.record_detail_at(addressed.clone()).unwrap();
    assert!(html(&addressed_view).contains("data-atlas-passage-target"));
    assert!(html(&addressed_view).contains(">10d6</span> mental damage"));
    if let Some(atlas_domain::SourcePassageAddress::HtmlSection {
        prepared_html_sha256,
        ..
    }) = &mut addressed.passage
    {
        *prepared_html_sha256 = "stale".into()
    }
    assert!(f.worker.record_detail_at(addressed).is_err());
    let offered = roll(&view);
    assert_eq!(offered.spell_rank, Some(5));
    let outcome = f.worker.roll_content(offered.clone()).unwrap();
    assert!((10..=60).contains(&outcome.components[0].outcome.total));
    request.spell_rank = Some(6);
    let higher = f.worker.record_detail_at(request).unwrap();
    assert!(html(&higher).contains(">12d6</span> mental damage"));
    let mut invalid = offered;
    invalid.spell_rank = Some(0);
    assert!(f.worker.roll_content(invalid).is_err());
}

#[test]
fn actor_parent_values_do_not_turn_standalone_feats_or_effect_defaults_into_context() {
    let f = fixture_worker_with_executor(RetrievalExecutor::from_test_fixture_factory(
        1,
        16,
        || {
            Ok(open_source_fixture(
                vec![
                    record(
                        "actors",
                        "Actor",
                        json!({"_id":"contextActor0001","name":"Creature","type":"npc","system":{"attributes":{"adjustment":null},"details":{"level":{"value":7},"publicNotes":"<p>@Damage[(ceil(@actor.level/2))d6[untyped]]</p><p>@Embed[Compendium.pf2e.feats.Item.contextFeat00001 inline]</p>"}},"items":[{"_id":"contextAction001","type":"action","name":"Breath","system":{"description":{"value":"<p>@Damage[(ceil(@actor.level/2))d6[untyped]]</p>"}}}]}),
                    ),
                    record(
                        "feats",
                        "Item",
                        json!({"_id":"contextFeat00001","name":"Standalone","type":"feat","system":{"level":{"value":7},"description":{"value":"<p>@Damage[(ceil(@actor.level/2))d6[untyped]]</p>"}}}),
                    ),
                    record(
                        "effects",
                        "Item",
                        json!({"_id":"contextEffect001","name":"Counter","type":"effect","system":{"badge":{"type":"counter","value":4},"description":{"value":"<p>@Damage[@item.badge.value[fire]]</p>"}}}),
                    ),
                ],
                false,
            )?)
        },
    ));
    let actor = f.worker.record_detail("actors:contextActor0001").unwrap();
    assert!(html(&actor).contains(">4d6</span> damage"));
    assert!(
        html(&actor).contains("half the creature’s level (rounded up)"),
        "foreign inclusion retains its own symbolic context"
    );
    let mut request = selection("actors:contextActor0001");
    request.owners = vec![atlas_domain::OwnedContentLocator {
        collection: "/items".into(),
        identity: atlas_domain::OwnedContentIdentity::Stable("contextAction001".into()),
    }];
    let owned = f.worker.record_detail_at(request).unwrap();
    assert!(html(&owned).contains(">4d6</span> damage"));
    let feat = f.worker.record_detail("feats:contextFeat00001").unwrap();
    assert!(html(&feat).contains("the creature’s level"));
    let effect = f.worker.record_detail("effects:contextEffect001").unwrap();
    assert!(html(&effect).contains("the effect’s counter value"));
}
