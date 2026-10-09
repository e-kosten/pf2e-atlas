use std::collections::BTreeMap;

use atlas_domain::{PackName, RecordId, RecordKey};

use super::*;

fn locator() -> SourceContentLocator {
    SourceContentLocator {
        record: RecordKey::new(
            PackName::new("spells-srd").unwrap(),
            RecordId::new("abcdefghijklmnop").unwrap(),
        ),
        owners: Vec::new(),
        field: "/system/description/value".into(),
    }
}

fn audience() -> ContentAudience {
    ContentAudience {
        include_gm: true,
        include_owner: true,
        implicit_check_dc: ContentVisibilityRule::All,
    }
}

fn prepare(markup: &str) -> PreparedSourceContent {
    prepare_source_content(
        locator(),
        markup,
        audience(),
        ContentVisibilityRule::All,
        None,
        None,
    )
    .unwrap()
}

struct Context(BTreeMap<String, String>);
impl LocalizationResolver for Context {
    fn localized_value(&self, key: &str) -> Option<&str> {
        self.0.get(key).map(String::as_str)
    }
}
impl ContentReferenceResolver for Context {
    fn resolve_reference(
        &self,
        _source: &SourceContentLocator,
        target: &str,
    ) -> Option<ResolvedContentReference> {
        (target == "Compendium.pf2e.spells-srd.Item.abcdefghijklmnop").then(|| {
            ResolvedContentReference {
                target: ContentReferenceTarget::Record {
                    key: locator().record,
                },
                display_name: Some("Heal".into()),
            }
        })
    }
}

#[test]
fn preserves_html_unicode_and_mechanics_without_a_persisted_tree() {
    let markup = "<h2>Élan — 力</h2><p>↔ @Check[type:reflex|basic|dc:21] @Template[type:cone|distance:30|width:5] @Damage[2d6[fire]] [[/r 1d20+4]]</p>";
    let content = prepare(markup);
    assert!(content.html.contains("<h2>Élan — 力</h2>"));
    assert!(content.text.contains("DC 21 basic reflex"));
    assert!(content.text.contains("30-foot cone (width 5 feet)"));
    assert!(content.text.contains("2d6[fire]"));
    assert!(content.text.contains("/r 1d20+4"));
    assert_eq!(content.interactions.len(), 4);
    assert_eq!(content.locator, locator());
    let ContentInteractionKind::Check { statistic, options } = &content.interactions[0].kind else {
        panic!("check expected");
    };
    assert_eq!(statistic.as_deref(), Some("reflex"));
    assert!(options.contains_key("basic"));
    assert!(!options.contains_key("reflex"));
    let value = serde_json::to_value(&content).unwrap();
    assert!(value.get("nodes").is_none());
    assert!(value.get("document").is_none());
    assert_eq!(
        serde_json::from_value::<PreparedSourceContent>(value).unwrap(),
        content
    );
}

#[test]
fn labels_remain_authored_while_mechanics_are_available_to_controls() {
    let content = prepare("@Check[fortitude|dc:21]{Resist the poison} @Damage[2d6[fire]]{Burn}");
    assert_eq!(content.text, "Resist the poison Burn");
    assert_eq!(content.interactions.len(), 2);
    assert!(matches!(&content.interactions[0].kind,
        ContentInteractionKind::Check { options, .. } if options["dc"] == "21"));
}

#[test]
fn localization_uses_the_localized_body_and_retains_ignored_label_references() {
    let context = Context(BTreeMap::from([
        (
            "Outer".into(),
            "<p>@Localize[Inner] @UUID[Compendium.pf2e.spells-srd.Item.abcdefghijklmnop]</p>"
                .into(),
        ),
        ("Inner".into(), "Heals you.".into()),
    ]));
    let content = prepare_source_content(
        locator(),
        "@Localize[Outer]{@UUID[ignored]{Label}}",
        audience(),
        ContentVisibilityRule::All,
        Some(&context),
        Some(&context),
    )
    .unwrap();
    assert_eq!(content.text, "Heals you. Heal");
    assert_eq!(content.references.len(), 2);
    assert!(!content.references[0].visible);
    assert_eq!(content.references[0].authored_target, "ignored");
    assert!(content.references[1].visible);
    assert!(matches!(
        content.references[1].resolution,
        ContentReferenceResolution::Resolved(ContentReferenceTarget::Record { .. })
    ));
    assert!(
        content
            .diagnostics
            .iter()
            .any(|issue| issue.code == ContentDiagnosticCode::IgnoredLocalizationLabel)
    );
    assert!(!content.html.contains("/record/"));
}

#[test]
fn missing_recursive_localization_remains_literal_and_diagnosed() {
    let context = Context(BTreeMap::from([("Loop".into(), "@Localize[Loop]".into())]));
    let content = prepare_source_content(
        locator(),
        "@Localize[Loop] @Localize[Missing]",
        audience(),
        ContentVisibilityRule::All,
        Some(&context),
        None,
    )
    .unwrap();
    assert_eq!(content.text, "@Localize[Loop] @Localize[Missing]");
    assert_eq!(
        content
            .diagnostics
            .iter()
            .filter(|issue| issue.code == ContentDiagnosticCode::UnresolvedLocalization)
            .count(),
        2
    );
}

#[test]
fn occurrences_are_complete_before_visibility_and_interactions_are_projected() {
    let markup = "<p>@UUID[public]{Public}</p><p data-visibility='gm'>@UUID[secret]{Secret} @Check[reflex|dc:99]</p><p data-visibility='owner'>@UUID[owner]</p><p data-visibility='none'>@UUID[never]</p><p>@Check[fortitude|dc:20|showDC:gm]</p>";
    let public = ContentAudience {
        include_gm: false,
        include_owner: false,
        implicit_check_dc: ContentVisibilityRule::None,
    };
    let content = prepare_source_content(
        locator(),
        markup,
        public,
        ContentVisibilityRule::All,
        None,
        None,
    )
    .unwrap();
    assert_eq!(content.references.len(), 4);
    assert_eq!(
        content
            .references
            .iter()
            .filter(|reference| reference.visible)
            .count(),
        1
    );
    assert!(!content.html.contains("Secret"));
    assert!(!content.text.contains("20"));
    assert!(!content.text.contains("99"));
    assert_eq!(content.interactions.len(), 1);
    assert!(
        matches!(&content.interactions[0].kind, ContentInteractionKind::Check { options, .. } if !options.contains_key("dc"))
    );
    let inspection = prepare_source_content(
        locator(),
        markup,
        audience(),
        ContentVisibilityRule::All,
        None,
        None,
    )
    .unwrap();
    assert_eq!(
        inspection
            .references
            .iter()
            .filter(|reference| reference.visible)
            .count(),
        3
    );
    assert_eq!(
        inspection
            .references
            .iter()
            .map(|r| (&r.path, &r.authored_target))
            .collect::<Vec<_>>(),
        content
            .references
            .iter()
            .map(|r| (&r.path, &r.authored_target))
            .collect::<Vec<_>>()
    );
    assert!(!inspection.text.contains("never"));
}

#[test]
fn sanitizer_and_facts_agree_on_blocked_links_and_ignore_forged_markers() {
    let content = prepare(
        "<p data-atlas-interaction='999' onclick='evil()'>safe <a href='java&#x73;cript:evil()'>bad</a> <a href='https://example.com'>okay</a></p><script>evil()</script>",
    );
    assert!(!content.html.contains("onclick"));
    assert!(!content.html.contains("javascript"));
    assert!(!content.html.contains("999"));
    assert!(!content.html.contains("<script"));
    assert_eq!(content.references.len(), 2);
    assert_eq!(
        content.references[0].resolution,
        ContentReferenceResolution::Blocked
    );
    assert!(matches!(
        content.references[1].resolution,
        ContentReferenceResolution::UnverifiedUrl { .. }
    ));
    assert!(!content.text.contains("evil()"));
}

#[test]
fn references_in_nested_labels_keep_separate_identity_without_nested_anchors() {
    let content =
        prepare("@UUID[outer]{Use @UUID[inner]{Inner}} @Embed[target inline hr=false]{Target}");
    assert_eq!(content.references.len(), 3);
    assert_eq!(content.references[0].ordinal, 0);
    assert_eq!(content.references[1].ordinal, 1);
    assert!(content.references[1].path.ends_with("/label/1"));
    assert!(
        matches!(&content.references[2].kind, ContentReferenceKind::Embed { options } if options["inline"] == "true" && options["hr"] == "false")
    );
    assert!(
        content
            .diagnostics
            .iter()
            .any(|issue| issue.code == ContentDiagnosticCode::EmbedNotExpanded)
    );
    assert!(content.html.contains("data-atlas-reference=\"1\""));
}

#[test]
fn unknown_and_dynamic_content_survives_without_runtime_evaluation() {
    let content = prepare(
        "@Mystery[odd]{Label} @Damage[(@actor.level+2)d6[fire]] @Check[type:reflex|dc:resolve(@actor.level)] <span class='action-glyph'>A</span> @Template[cone|distance:resolve(@actor.level)] <span class='pf2-icon'>R</span>",
    );
    assert!(content.text.contains("@Mystery[odd]{Label}"));
    assert!(content.text.contains("(@actor.level+2)d6[fire]"));
    assert!(content.text.contains("resolve(@actor.level)"));
    assert!(content.text.contains("[action glyph: A]"));
    assert!(content.text.contains("reaction"));
    for code in [
        ContentDiagnosticCode::UnknownMacro,
        ContentDiagnosticCode::RuntimeContextRequired,
        ContentDiagnosticCode::UnknownActionGlyph,
        ContentDiagnosticCode::InvalidTemplate,
    ] {
        assert!(
            content.diagnostics.iter().any(|issue| issue.code == code),
            "{code:?}"
        );
    }
}

#[test]
fn stable_locator_is_independent_of_display_output_and_snapshot_local_identity_is_explicit() {
    let mut owned = locator();
    owned.owners.push(OwnedContentLocator {
        collection: "/items".into(),
        identity: OwnedContentIdentity::Stable("abcdefghijklmnop".into()),
    });
    let first = prepare_source_content(
        owned.clone(),
        "<p>Old name</p>",
        audience(),
        ContentVisibilityRule::All,
        None,
        None,
    )
    .unwrap();
    let second = prepare_source_content(
        owned.clone(),
        "<p>New name</p>",
        audience(),
        ContentVisibilityRule::All,
        None,
        None,
    )
    .unwrap();
    assert_eq!(first.locator, second.locator);
    assert_ne!(first.text, second.text);
    owned.owners[0].identity = OwnedContentIdentity::SnapshotLocal { index: 0 };
    assert_ne!(owned, first.locator);
}

#[test]
fn typed_partial_source_and_owned_item_are_callable_without_ingest() {
    use atlas_foundry_model::{
        ActorSourcePF2e, ItemSourcePF2e, SourceContext, SourcePresence, admit_actor_source_pf2e,
    };
    let raw = br#"{"_id":"abcdefghijklmnop","name":"Fixture","type":"npc","system":{"details":{"level":{"value":"broken"}}},"items":[{"_id":"ponmlkjihgfedcba","name":"Owned spell","type":"spell","system":{"description":{"value":"<p>@Check[type:reflex|basic|dc:21]</p>"}}}]}"#;
    let admission =
        admit_actor_source_pf2e(SourceContext::new("test", "fixture", "$"), raw).unwrap();
    assert!(!admission.diagnostics.is_empty());
    let ActorSourcePF2e::NPCSource(actor) = admission.model.unwrap() else {
        panic!("NPC");
    };
    let system = actor.system.as_value().unwrap();
    assert!(matches!(
        system
            .details
            .as_value()
            .unwrap()
            .level
            .as_value()
            .unwrap()
            .value,
        SourcePresence::Invalid { .. }
    ));
    let items = actor.items.as_value().unwrap();
    let ItemSourcePF2e::SpellSource(spell) = &items[0] else {
        panic!("Spell");
    };
    let markup = spell
        .system
        .as_value()
        .unwrap()
        .description
        .as_value()
        .unwrap()
        .value
        .as_value()
        .unwrap();
    let mut owned = locator();
    owned.owners.push(OwnedContentLocator {
        collection: "/items".into(),
        identity: OwnedContentIdentity::Stable(spell._id.as_value().unwrap().clone()),
    });
    let content = prepare_source_content(
        owned,
        markup,
        audience(),
        ContentVisibilityRule::All,
        None,
        None,
    )
    .unwrap();
    assert_eq!(content.text, "DC 21 basic reflex");
    assert_eq!(content.interactions.len(), 1);
}

#[test]
fn meaningful_captions_and_explicit_numbering_survive_library_text_projection() {
    let content = prepare(
        "<h2>Costs</h2><ol start='4'><li>First</li><li value='7'>Next</li><li>Last</li></ol><table><caption>Targets @Damage[1d6[fire]]</caption><tr><th colspan='2'>Type</th></tr><tr><td rowspan='2'>Creature</td><td>Two</td></tr><tr><td>Three</td></tr></table>",
    );
    for text in [
        "Costs",
        "4. First",
        "7. Next",
        "8. Last",
        "Targets 1d6[fire]",
        "Creature",
        "Two",
        "Three",
    ] {
        assert!(
            content.text.contains(text),
            "{text:?} absent from {:?}",
            content.text
        );
    }
    for html in [
        "<caption>",
        "start=\"4\"",
        "value=\"7\"",
        "colspan=\"2\"",
        "rowspan=\"2\"",
    ] {
        assert!(content.html.contains(html), "{html:?}");
    }
    assert_eq!(content.interactions.len(), 1);
}

#[test]
fn typed_damage_suffixes_do_not_consume_the_inline_roll_terminator() {
    let content = prepare("[[/r 2d6[fire]]]{Flames} @Damage[formula:3d6[cold]|name:Frost]");
    assert_eq!(content.text, "Flames 3d6[cold]");
    assert!(
        matches!(&content.interactions[0].kind, ContentInteractionKind::Command { arguments, .. } if arguments == "2d6[fire]")
    );
    assert!(
        matches!(&content.interactions[1].kind, ContentInteractionKind::Damage { formula, options } if formula == "3d6[cold]" && options["name"] == "Frost")
    );
}

#[test]
fn unusual_glyph_contents_do_not_drop_nested_reference_or_interaction_markers() {
    let content = prepare(
        "<span class='action-glyph'>@UUID[target]{Label} @Damage[1d6[fire]] <a href='https://example.com'>Source</a></span>",
    );
    assert_eq!(content.text, "Label 1d6[fire] [Source]");
    assert!(content.html.contains("data-atlas-reference=\"0\""));
    assert!(content.html.contains("data-atlas-reference=\"1\""));
    assert!(content.html.contains("data-atlas-interaction=\"0\""));
}

#[test]
fn malformed_enrichments_remain_literal_and_are_reported() {
    let content = prepare("<p>Before @Check[will|dc:21 and [[/r 2d6[fire] after</p>");
    assert!(content.text.contains("@Check[will|dc:21"));
    assert!(content.text.contains("[[/r 2d6[fire]"));
    assert!(content.interactions.is_empty());
    assert!(
        content
            .diagnostics
            .iter()
            .any(|issue| issue.code == ContentDiagnosticCode::MalformedSyntax)
    );
}

#[test]
fn unsupported_macro_meaning_is_not_invented_and_label_references_remain_evidence() {
    let content = prepare("@Action[R]{@UUID[target]{Trigger}} @Trait[fire]");
    assert_eq!(content.references.len(), 1);
    assert!(!content.references[0].visible);
    assert!(content.text.contains("@Action[R]{@UUID[target]{Trigger}}"));
    assert!(content.text.contains("@Trait[fire]"));
    assert_eq!(
        content
            .diagnostics
            .iter()
            .filter(|issue| issue.code == ContentDiagnosticCode::UnknownMacro)
            .count(),
        2
    );
}
