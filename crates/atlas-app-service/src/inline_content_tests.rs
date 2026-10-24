use crate::{executor::RetrievalExecutor, test_support::*};
use atlas_app_model::*;
use atlas_search::test_support::{open_source_fixture, record};
use serde_json::json;

fn fixture(host: &str, target: &str) -> FixtureWorker {
    fixture_worker_with_executor(RetrievalExecutor::from_test_fixture_factory(1, 16, {
        let host = host.to_string();
        let target = target.to_string();
        move || {
            Ok(open_source_fixture(
                vec![
                    record(
                        "spells",
                        "Item",
                        json!({"_id":"hostSpell1000000","name":"Host spell","type":"spell","system":{"description":{"value":host}}}),
                    ),
                    record(
                        "features",
                        "Item",
                        json!({"_id":"featTarget100000","name":"Target feature","type":"feat","system":{"level":{"value":2},"traits":{"value":["fighter"],"rarity":"uncommon"},"prerequisites":{"value":[{"value":"Know the basics"}]},"publication":{"title":"Test Source"},"description":{"value":target}}}),
                    ),
                    record(
                        "actions",
                        "Item",
                        json!({"_id":"actionTarget1000","name":"Nested reaction","type":"action","system":{"description":{"value":"<p>Trigger: enemy moves. @Check[reflex|dc:22] @Damage[2d6[fire]]</p>"}}}),
                    ),
                ],
                false,
            )?)
        }
    }))
}
fn body(detail: &RecordDetailView) -> (&str, &[PreparedInclusionView]) {
    let field = detail
        .presentation
        .content
        .iter()
        .find(|f| f.role == "description")
        .unwrap();
    let PreparedFieldBodyView::Html {
        html, inclusions, ..
    } = &field.body
    else {
        panic!("html")
    };
    (html, inclusions)
}
#[test]
fn inline_content_composes_nested_occurrences_with_exact_binding_sources() {
    let f = fixture(
        "<h2>Host rules</h2><p>@UUID[Compendium.pf2e.features.Item.featTarget100000]</p><p>@Embed[Compendium.pf2e.features.Item.featTarget100000 inline header traits publication journalLink=false]</p><p>@Embed[Compendium.pf2e.features.Item.featTarget100000 inline hr=false]</p><p>Host ends.</p>",
        "<p>Feature introduction.</p><p>@Embed[Compendium.pf2e.actions.Item.actionTarget1000 inline]</p><p>@UUID[Compendium.pf2e.actions.Item.actionTarget1000]</p>",
    );
    let detail = f.worker.record_detail("spells:hostSpell1000000").unwrap();
    let (html, bindings) = body(&detail);
    assert_eq!(html.matches("Feature introduction.").count(), 2, "{html}");
    assert_eq!(html.matches("Trigger: enemy moves.").count(), 2);
    assert_eq!(html.matches("Test Source").count(), 1);
    assert_eq!(html.matches("<hr>").count(), 1);
    assert_eq!(html.matches("Know the basics").count(), 2);
    assert!(html.contains("Host ends.") && html.contains("Level 2") && html.contains("uncommon"));
    let namespaces: std::collections::BTreeSet<_> = bindings.iter().map(|b| b.namespace).collect();
    assert_eq!(namespaces.len(), bindings.len());
    let controls: Vec<_> = bindings.iter().filter(|b| !b.controls.is_empty()).collect();
    assert_eq!(controls.len(), 2);
    assert!(
        controls
            .iter()
            .all(|b| b.source.record.to_string() == "actions:actionTarget1000")
    );
    assert!(controls.iter().all(|b| b.controls.iter().any(|c|matches!(c.control,ContentControlKindView::Damage{ref formula,..} if formula=="2d6[fire]"))));
    // One decode per distinct inclusion target, even when the same fragment repeats.
    assert_eq!(f.worker.last_read_metrics().source_body_decodes, 3);
    assert!(detail.relationships.iter().any(|r| {
        r.kind == "uuid"
            && r.target
                .as_ref()
                .is_some_and(|t| t.record_key == "features:featTarget100000")
    }));
}
#[test]
fn inline_content_handles_cycles_missing_and_unknown_options_without_losing_host() {
    for (target, options, message) in [
        (
            "<p>@Embed[Compendium.pf2e.features.Item.featTarget100000 inline]</p>",
            "inline",
            "already included",
        ),
        (
            "<p>Rules.</p>",
            "inline unknown=true",
            "option is unavailable",
        ),
    ] {
        let f = fixture(
            &format!(
                "<p>Before.</p><p>@Embed[Compendium.pf2e.features.Item.featTarget100000 {options}]</p><p>After.</p>"
            ),
            target,
        );
        let detail = f.worker.record_detail("spells:hostSpell1000000").unwrap();
        let (html, _) = body(&detail);
        assert!(
            html.contains("Before.") && html.contains("After.") && html.contains(message),
            "{html}"
        );
    }
    let f = fixture(
        "<p>@Embed[Compendium.pf2e.features.Item.missingItem10000 inline]</p>",
        "<p>Rules</p>",
    );
    let detail = f.worker.record_detail("spells:hostSpell1000000").unwrap();
    assert!(body(&detail).0.contains("Included content is unavailable"));
}
#[test]
fn inline_content_budget_is_shared_across_sibling_occurrences() {
    let f = fixture(
        &"<p>@Embed[Compendium.pf2e.features.Item.featTarget100000 inline]</p>".repeat(70),
        "<p>Repeated source rule.</p>",
    );
    let detail = f.worker.record_detail("spells:hostSpell1000000").unwrap();
    let (html, _) = body(&detail);
    assert_eq!(html.matches("Repeated source rule.").count(), 64);
    assert_eq!(html.matches("More content is available").count(), 6);
    assert_eq!(f.worker.last_read_metrics().source_body_decodes, 2);
}

#[test]
fn inline_content_links_existing_headings_and_leaves_untitled_content_as_prose() {
    for (host, titled) in [
        (
            "<h3>Paragon Benefit</h3><p>@Embed[Compendium.pf2e.features.Item.featTarget100000 inline]</p>",
            true,
        ),
        (
            "<p>@Embed[Compendium.pf2e.features.Item.featTarget100000 inline]</p>",
            false,
        ),
    ] {
        let f = fixture(host, "<p>Benefit rules.</p>");
        let detail = f.worker.record_detail("spells:hostSpell1000000").unwrap();
        let (html, bindings) = body(&detail);
        assert!(!html.contains("Source record</span>"), "{html}");
        assert_eq!(html.contains("<h3>"), titled, "{html}");
        if titled {
            assert!(html.contains("Paragon Benefit</span></h3>"));
        }
        let source = &bindings
            .iter()
            .find(|b| b.namespace == 1)
            .unwrap()
            .references[0];
        assert_eq!(
            source.target.as_ref().unwrap().record_key,
            "features:featTarget100000"
        );
        assert!(html.contains("Benefit rules."));
    }
}
