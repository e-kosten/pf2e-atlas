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
fn section_sidecar_preserves_original_body_without_inclusions() {
    let html = "<div><h2>First</h2><p>Text</p><h2>Second</h2><ul><li>Tail</li></ul></div>";
    let fixture = fixture(html, "<p>Unused target</p>");
    let detail = fixture
        .worker
        .record_detail("spells:hostSpell1000000")
        .unwrap();
    let field = detail
        .presentation
        .content
        .iter()
        .find(|field| field.role == "description")
        .unwrap();
    let PreparedFieldBodyView::Html {
        html: original,
        inclusions,
        ..
    } = &field.body
    else {
        panic!("html")
    };
    assert_eq!(original, html);
    assert!(inclusions.is_empty());
    let rendering = field.section_rendering.as_ref().unwrap();
    assert_eq!(
        rendering.selection_version,
        atlas_record::source_record::SOURCE_CONTENT_SELECTION_VERSION
    );
    assert!(rendering.html.contains("data-atlas-source-section=\"0\""));
    assert!(rendering.html.contains("data-atlas-source-section=\"1\""));
    assert_eq!(field.locator.record.to_string(), "spells:hostSpell1000000");
    assert_eq!(field.locator.field, "/system/description/value");
    assert!(field.locator.owners.is_empty());
    assert_eq!(fixture.worker.last_read_metrics().source_body_decodes, 1);
}

#[test]
fn inline_content_composes_nested_occurrences_with_exact_binding_sources() {
    let f = fixture(
        "<h2>Host rules</h2><p>@UUID[Compendium.pf2e.features.Item.featTarget100000]</p><p>@Embed[Compendium.pf2e.features.Item.featTarget100000 inline header traits publication journalLink=false]</p><p>@Embed[Compendium.pf2e.features.Item.featTarget100000 inline hr=false]</p><p>Host ends.</p>",
        "<p data-atlas-source-section=\"999\">Feature introduction.</p><p>@Embed[Compendium.pf2e.actions.Item.actionTarget1000 inline]</p><p>@UUID[Compendium.pf2e.actions.Item.actionTarget1000]</p>",
    );
    let detail = f.worker.record_detail("spells:hostSpell1000000").unwrap();
    let (html, bindings) = body(&detail);
    let rendering = detail
        .presentation
        .content
        .iter()
        .find(|field| field.role == "description")
        .unwrap()
        .section_rendering
        .as_ref()
        .unwrap();
    assert!(!html.contains("data-atlas-source-section"));
    assert!(!rendering.html.contains("999"));
    assert!(
        rendering
            .html
            .contains("<h2 data-atlas-source-section=\"0\">")
    );
    assert!(
        rendering
            .html
            .contains("<p data-atlas-source-section=\"0\">Host ends.</p>")
    );
    // Included target prose remains unannotated, retaining its original namespaces
    // and interaction/reference binding identities. No extra source read occurs.
    assert!(rendering.html.contains("<p>Feature introduction.</p>"));
    assert_eq!(rendering.html.matches("Feature introduction.").count(), 2);
    assert_eq!(rendering.html.matches("Trigger: enemy moves.").count(), 2);
    assert_eq!(
        rendering.html.matches("data-atlas-inclusion=").count(),
        html.matches("data-atlas-inclusion=").count()
    );

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

#[test]
fn inline_content_honors_all_observed_source_option_combinations() {
    // These are the seven combinations supplied by the pinned visible corpus.
    for (options, header, traits, publication, journal_link, rule) in [
        ("inline", false, false, false, true, true),
        ("inline hr=false", false, false, false, true, false),
        ("inline traits publication", false, true, true, true, true),
        (
            "inline traits publication journalLink=false",
            false,
            true,
            true,
            false,
            true,
        ),
        (
            "inline header traits publication",
            true,
            true,
            true,
            true,
            true,
        ),
        (
            "inline header traits publication journalLink=false",
            true,
            true,
            true,
            false,
            true,
        ),
        ("inline publication", false, false, true, true, true),
    ] {
        let f = fixture(
            &format!("<p>@Embed[Compendium.pf2e.features.Item.featTarget100000 {options}]</p>"),
            "<p>Complete feature rules.</p><p>@UUID[Compendium.pf2e.actions.Item.actionTarget1000]</p>",
        );
        let detail = f.worker.record_detail("spells:hostSpell1000000").unwrap();
        let (html, bindings) = body(&detail);
        assert!(html.contains("Complete feature rules.") && html.contains("Know the basics"));
        assert_eq!(html.contains("<h4>"), header, "{options}: {html}");
        assert_eq!(
            html.contains("prepared-inclusion__traits"),
            traits,
            "{options}"
        );
        assert_eq!(html.contains("Test Source"), publication, "{options}");
        assert_eq!(html.contains("<hr>"), rule, "{options}");
        // Stored relationship provenance remains available even when the authored
        // journalLink option omits its terminal marker from this reading fragment.
        let reference = bindings
            .iter()
            .flat_map(|b| &b.references)
            .find(|r| r.kind == "uuid")
            .unwrap();
        assert_eq!(
            reference.target.as_ref().unwrap().record_key,
            "actions:actionTarget1000"
        );
        assert_eq!(
            html.contains("Nested reaction"),
            journal_link,
            "{options}: {html}"
        );
    }
}

#[test]
fn inline_content_stops_at_the_depth_limit_without_losing_following_rules() {
    let f = fixture_worker_with_executor(RetrievalExecutor::from_test_fixture_factory(
        1,
        16,
        || {
            let mut records = vec![record(
                "spells",
                "Item",
                json!({
                    "_id":"hostSpell1000000", "name":"Host", "type":"spell",
                    "system":{"description":{"value":"<p>Before.</p><p>@Embed[Compendium.pf2e.actions.Item.depth00000000001 inline]</p><p>After.</p>"}}
                }),
            )];
            for depth in 1..=5 {
                records.push(record("actions", "Item", json!({
                "_id":format!("depth{depth:011}"), "name":format!("Depth {depth}"), "type":"action",
                "system":{"description":{"value":format!("<p>Rules at depth {depth}.</p><p>@Embed[Compendium.pf2e.actions.Item.depth{:011} inline]</p>", depth + 1)}}
            })));
            }
            Ok(open_source_fixture(records, false)?)
        },
    ));
    let detail = f.worker.record_detail("spells:hostSpell1000000").unwrap();
    let (html, bindings) = body(&detail);
    assert!(html.contains("Before.") && html.contains("After."));
    for depth in 1..=4 {
        assert!(html.contains(&format!("Rules at depth {depth}.")), "{html}");
    }
    assert!(!html.contains("Rules at depth 5."));
    assert!(html.contains("More content is available") && html.contains("Open reference"));
    assert!(bindings.iter().flat_map(|b| &b.references).any(|r| {
        r.target
            .as_ref()
            .is_some_and(|t| t.record_key == "actions:depth00000000005")
    }));
}

#[test]
fn inline_content_bounds_large_repeated_prose_and_retains_the_exact_destination() {
    let f = fixture(
        &format!(
            "{}<p>Host conclusion.</p>",
            "<p>@Embed[Compendium.pf2e.features.Item.featTarget100000 inline]</p>".repeat(12)
        ),
        &format!("<p>{}</p>", "Meaningful supplied rules. ".repeat(1500)),
    );
    let detail = f.worker.record_detail("spells:hostSpell1000000").unwrap();
    let (html, _) = body(&detail);
    assert!(html.contains("Meaningful supplied rules.") && html.contains("Host conclusion."));
    assert!(html.contains("More content is available") && html.contains("Open reference"));
    assert!(html.len() < 280 * 1024, "{} bytes", html.len());
    assert!(
        detail
            .relationships
            .iter()
            .filter(|r| r.kind == "embed")
            .all(|r| r
                .target
                .as_ref()
                .is_some_and(|t| t.record_key == "features:featTarget100000"))
    );
}

#[test]
fn inline_content_does_not_expand_hidden_inclusions_or_restore_hidden_target_text() {
    let f = fixture(
        "<p data-visibility='none'>@Embed[Compendium.pf2e.actions.Item.actionTarget1000 inline]</p><p>@Embed[Compendium.pf2e.features.Item.featTarget100000 inline]</p>",
        "<p>Visible rule.</p><p data-visibility='none'>Hidden rule. @Check[reflex|dc:99] @Embed[Compendium.pf2e.actions.Item.actionTarget1000 inline]</p>",
    );
    let detail = f.worker.record_detail("spells:hostSpell1000000").unwrap();
    let (html, bindings) = body(&detail);
    assert!(html.contains("Visible rule."));
    assert!(!html.contains("Hidden rule.") && !html.contains("Trigger: enemy moves."));
    assert!(
        !bindings
            .iter()
            .any(|b| b.source.record.to_string() == "actions:actionTarget1000")
    );
    assert!(bindings.iter().all(|b| b.controls.is_empty()));
    assert_eq!(f.worker.last_read_metrics().source_body_decodes, 2);
}
