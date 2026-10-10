use super::traversal::visit_source_nodes;
use super::*;
use crate::source_content::*;
use atlas_foundry_model::{SourceContext, admit_document_source, decode_snapshot, encode_snapshot};
use serde_json::{Value, json};

const ROOT: &str = "aaaaaaaaaaaaaaaa";
const CHILD: &str = "bbbbbbbbbbbbbbbb";
const OTHER: &str = "cccccccccccccccc";
#[test]
fn table_results_resolve_explicit_pack_pairs_and_retain_external_or_invalid_targets() {
    let table = source_record(
        "RollTable",
        json!({"_id":ROOT,"results":[
            {"_id":CHILD,"type":"pack","documentCollection":"pf2e.test","documentId":OTHER},
            {"_id":"dddddddddddddddd","type":"pack","documentCollection":"pf2e.test","documentId":ROOT},
            {"_id":"eeeeeeeeeeeeeeee","type":"pack","documentCollection":"other-system.test","documentId":OTHER},
            {"_id":"ffffffffffffffff","type":"document","documentCollection":"Actor","documentId":OTHER},
            {"_id":"gggggggggggggggg","type":"pack","documentCollection":"pf2e.macros","documentId":OTHER},
            {"_id":"hhhhhhhhhhhhhhhh","type":"pack","documentCollection":"pf2e.test","documentId":42},
            {"_id":"iiiiiiiiiiiiiiii","type":"text","documentCollection":"pf2e.test","documentId":OTHER},
            {"_id":"jjjjjjjjjjjjjjjj","type":"document","documentCollection":"pf2e.test","documentId":OTHER}
        ]}),
    );
    let target = source_record("Item", json!({"_id":OTHER,"type":"equipment"}));
    let macro_source = admitted(
        "Macro",
        json!({"_id":OTHER,"type":"script","command":"return;"}),
    );
    let macro_record = SourceBackedRecord::new("macros", macro_source).unwrap();
    let mut index = SourceReferenceIndex::default();
    for r in [&table, &target, &macro_record] {
        index.insert_source(r.key(), r.source());
    }
    let edges = resolve_source_relationships(&table, Some(&index));
    assert_eq!(edges.len(), 7);
    assert!(
        edges
            .iter()
            .all(|e| e.kind == SourceRelationshipKind::TableResult
                && e.locator.field == "/documentId"
                && e.locator.owners.len() == 1)
    );
    assert!(
        matches!(&edges[0].resolution,ContentReferenceResolution::Resolved(ContentReferenceTarget::Record{key}) if key==target.key())
    );
    assert!(
        matches!(&edges[1].resolution,ContentReferenceResolution::Resolved(ContentReferenceTarget::Record{key}) if key==table.key())
    );
    assert_eq!(edges[2].resolution, ContentReferenceResolution::Unresolved);
    assert_eq!(edges[3].resolution, ContentReferenceResolution::Unresolved);
    assert!(
        matches!(&edges[4].resolution,ContentReferenceResolution::Resolved(ContentReferenceTarget::Record{key}) if key==macro_record.key())
    );
    assert!(matches!(
        edges[5].availability,
        FieldAvailability::Invalid { .. }
    ));
    assert_eq!(edges[6].authored_target, edges[0].authored_target);
    assert_eq!(edges[6].availability, FieldAvailability::Value);
    assert_eq!(edges[6].resolution, ContentReferenceResolution::Unresolved);
}
fn admitted(kind: &str, value: Value) -> atlas_foundry_model::FoundryDocumentSource {
    let bytes = serde_json::to_vec(&value).unwrap();
    admit_document_source(kind, SourceContext::new("fixture", "test", "$"), &bytes)
        .unwrap()
        .model
        .unwrap()
}
fn public() -> ContentAudience {
    ContentAudience {
        include_gm: false,
        include_owner: false,
        implicit_check_dc: ContentVisibilityRule::Gm,
    }
}
fn source_record(kind: &str, value: Value) -> SourceBackedRecord {
    let source = admitted(kind, value);
    SourceBackedRecord::new("test", source).unwrap()
}
fn item(family: &str) -> Value {
    json!({"_id":ROOT,"name":"Élan @Damage[1d6]","type":family,"system":{"description":{"value":"<p>Authored — 力 @Damage[1d6]</p>","gm":"@UUID[Compendium.pf2e.test.Item.aaaaaaaaaaaaaaaa] @Check[fortitude|dc:20]"},"publication":{"title":"Test","remaster":false}}})
}

#[test]
fn owned_lookup_checks_sibling_identity_instead_of_choosing_duplicate_ids() {
    let record = source_record(
        "Actor",
        json!({
            "_id": ROOT, "type": "npc", "items": [
                {"_id": CHILD, "name": "First", "type": "spell"},
                {"_id": CHILD, "name": "Second", "type": "spell"},
                {"_id": OTHER, "name": "Unique", "type": "spell"}
            ]
        }),
    );
    let owner = |identity| {
        vec![OwnedContentLocator {
            collection: "/items".into(),
            identity,
        }]
    };
    assert!(
        record
            .node_at(&owner(OwnedContentIdentity::Stable(CHILD.into())))
            .is_none()
    );
    assert_eq!(
        record
            .node_at(&owner(OwnedContentIdentity::SnapshotLocal { index: 1 }))
            .unwrap()
            .name()
            .value()
            .map(String::as_str),
        Some("Second")
    );
    assert_eq!(
        record
            .node_at(&owner(OwnedContentIdentity::Stable(OTHER.into())))
            .unwrap()
            .name()
            .value()
            .map(String::as_str),
        Some("Unique")
    );
    assert!(
        record
            .node_at(&owner(OwnedContentIdentity::SnapshotLocal { index: 99 }))
            .is_none()
    );
    assert!(matches!(
        record.node_at(&[]),
        Some(SourceNodeView::Actor(_))
    ));
    let mut items = Vec::new();
    assert!(
        record
            .visit_immediate_actor_items(|index, owner, item| {
                items.push((index, owner.clone(), item.family()));
            })
            .value()
            .is_some()
    );
    assert_eq!(items.len(), 3);
    assert_eq!(
        items[0].1.identity,
        OwnedContentIdentity::SnapshotLocal { index: 0 }
    );
    assert_eq!(
        items[1].1.identity,
        OwnedContentIdentity::SnapshotLocal { index: 1 }
    );
    assert_eq!(items[2].0, 2);
    assert_eq!(
        items[2].1.identity,
        OwnedContentIdentity::Stable(OTHER.into())
    );
}

#[test]
fn dispatches_all_eight_actor_and_twenty_four_item_families() {
    for family in [
        "army",
        "character",
        "familiar",
        "hazard",
        "loot",
        "npc",
        "party",
        "vehicle",
    ] {
        let record = source_record(
            "Actor",
            json!({"_id":ROOT,"name":family,"type":family,"system":{"traits":{"value":["new-trait"],"rarity":"rare"}},"items":[]}),
        );
        assert_eq!(
            SourceNodeView::from(record.source()).source_type().value(),
            Some(family)
        );
        assert_eq!(
            SourceNodeView::from(record.source())
                .actor_items()
                .value()
                .unwrap()
                .len(),
            0
        );
        let query = SourceQueryView::new(record.source(), "test", "Test");
        let applicable = matches!(family, "army" | "npc" | "hazard" | "vehicle");
        assert_eq!(query.traits().value().is_some(), applicable);
        assert_eq!(query.rarity().value().is_some(), applicable);
    }
    for family in [
        "action",
        "affliction",
        "ancestry",
        "armor",
        "background",
        "book",
        "campaignFeature",
        "class",
        "condition",
        "consumable",
        "backpack",
        "deity",
        "effect",
        "equipment",
        "feat",
        "heritage",
        "kit",
        "lore",
        "melee",
        "shield",
        "spell",
        "spellcastingEntry",
        "treasure",
        "weapon",
    ] {
        let mut input = item(family);
        input["system"]["traits"] = json!({"value":["new-trait"],"rarity":"rare"});
        // Foundry declares Treasure's value as never[]: only [] is valid.
        if family == "treasure" {
            input["system"]["traits"]["value"] = json!([]);
        }
        let record = source_record("Item", input);
        let content = prepare_record_content(&record, public(), None, None);
        let query = SourceQueryView::new(record.source(), "test", "Test pack");
        assert_eq!(query.source.source_type().value(), Some(family));
        assert_eq!(query.publication_remaster().value(), Some(false));
        let body = content
            .iter()
            .find(|c| c.locator().field == "/system/description/value")
            .unwrap();
        assert!(
            matches!(&body.status,SourceContentStatus::Prepared(p) if p.interactions.len()==1 && p.text=="Authored — 力 1d6")
        );
        let traits_applicable = !matches!(family, "class" | "deity" | "lore" | "spellcastingEntry");
        let rarity_applicable = !matches!(
            family,
            "action"
                | "affliction"
                | "campaignFeature"
                | "condition"
                | "deity"
                | "effect"
                | "kit"
                | "lore"
                | "melee"
                | "spellcastingEntry"
        );
        assert_eq!(
            query.traits().value().is_some(),
            traits_applicable,
            "{family}"
        );
        assert_eq!(
            query.rarity().value().is_some(),
            rarity_applicable,
            "{family}"
        );
    }
}

#[test]
fn whole_field_visibility_retains_hidden_occurrences_and_plain_names_do_not_parse_macros() {
    let source = admitted("Item", item("spell"));
    let key = source_record_key("test", &source).unwrap();
    let mut resolver = SourceReferenceIndex::default();
    resolver.insert_source(&key, &source);
    let record = SourceBackedRecord::new(key.pack().as_str(), source).unwrap();
    let content = prepare_record_content(&record, public(), None, Some(&resolver));
    let gm = content
        .iter()
        .find(|c| c.locator().field == "/system/description/gm")
        .unwrap();
    let SourceContentStatus::Prepared(gm) = &gm.status else {
        panic!("prepared");
    };
    assert!(gm.html.is_empty() && gm.text.is_empty() && gm.interactions.is_empty());
    assert_eq!(gm.references.len(), 1);
    assert!(!gm.references[0].visible);
    assert_eq!(gm.references[0].audiences, ["gm"]);
    assert!(matches!(
        gm.references[0].resolution,
        ContentReferenceResolution::Resolved(_)
    ));
    assert!(!content.iter().any(|c| c.role == SourceContentRole::Name));
    assert!(
        record
            .text_sources(&content, public(), "Test")
            .iter()
            .any(|t| t.text == "Élan @Damage[1d6]")
    );
    assert!(
        !record
            .text_sources(&content, public(), "Test")
            .iter()
            .any(|t| t.text.contains("fortitude"))
    );
}

#[test]
fn source_equality_and_checked_snapshot_roundtrip_preserve_invalid_and_additional_fields() {
    let mut input = item("spell");
    input["system"]["level"] = json!({"value":"3"});
    input["unknown"] = json!({"z":[1,null,{"x":false}],"a":2});
    let source = admitted("Item", input);
    let snapshot = encode_snapshot(&source).unwrap();
    let key = source_record_key("test", &source).unwrap();
    let record = SourceBackedRecord::new(key.pack().as_str(), source).unwrap();
    assert_eq!(encode_snapshot(record.source()).unwrap(), snapshot);
    assert_eq!(&decode_snapshot(&snapshot).unwrap(), record.source());
    let SourceNodeView::Item(item) = SourceNodeView::from(record.source()) else {
        panic!();
    };
    assert!(matches!(item.spell_rank(), SourceFieldView::Invalid(_)));
    let content = prepare_record_content(&record, public(), None, None);
    let relationships = resolve_source_relationships(&record, None);
    assert_eq!(encode_snapshot(record.source()).unwrap(), snapshot);
    let bytes = serde_json::to_vec(&content).unwrap();
    assert_eq!(
        serde_json::from_slice::<Vec<SourceContentOutcome>>(&bytes).unwrap(),
        content
    );
    let bytes = serde_json::to_vec(&relationships).unwrap();
    assert_eq!(
        serde_json::from_slice::<Vec<SourceRelationshipOccurrence>>(&bytes).unwrap(),
        relationships
    );
}

#[test]
fn projections_preserve_baselines_open_sets_zero_false_and_ancestor_states() {
    let record = source_record(
        "Actor",
        json!({"_id":ROOT,"type":"npc","system":{"details":{"level":{"value":0},"publication":{"remaster":false}},"attributes":{"ac":{"value":21},"hp":{"max":80,"value":12}},"traits":{"value":["brand-new-provider-trait"],"rarity":"rare"}},"items":[]}),
    );
    let query = SourceQueryView::new(record.source(), "test", "Test");
    assert_eq!(query.actor().level().value().unwrap().as_i64(), Some(0));
    assert_eq!(
        query.actor().hp_maximum().value().unwrap().as_i64(),
        Some(80)
    );
    assert_eq!(
        query.actor().armor_class().value().unwrap().as_i64(),
        Some(21)
    );
    assert_eq!(
        query.traits().value().unwrap(),
        ["brand-new-provider-trait"]
    );
    assert_eq!(query.publication_remaster().value(), Some(false));
    assert_eq!(query.pack_label, "Test");
    for (system, availability) in [
        (json!({}), FieldAvailability::Missing),
        (json!(null), FieldAvailability::Null),
        (
            json!(3),
            FieldAvailability::Invalid {
                source_path: "$.system".into(),
            },
        ),
    ] {
        let record = source_record("Actor", json!({"_id":ROOT,"type":"npc","system":system}));
        let query = SourceQueryView::new(record.source(), "test", "Test");
        assert_eq!(query.actor().hp_maximum().availability(), availability);
    }
    let hazard = source_record(
        "Actor",
        json!({"_id":ROOT,"type":"hazard","system":{"details":{"isComplex":false},"attributes":{"hardness":0}}}),
    );
    let query = SourceQueryView::new(hazard.source(), "test", "Test");
    assert_eq!(query.actor().hazard_complexity().value(), Some(false));
    assert_eq!(
        query.actor().hazard_hardness().value().unwrap().as_u64(),
        Some(0)
    );
}

#[test]
fn empty_unknown_and_not_applicable_are_different() {
    let empty = source_record(
        "Item",
        json!({"_id":ROOT,"type":"spell","system":{"traits":{"value":[],"traditions":[]},"level":{"value":3},"location":{"heightenedLevel":8}}}),
    );
    let SourceNodeView::Item(view) = SourceNodeView::from(empty.source()) else {
        panic!();
    };
    assert_eq!(view.traits().value(), Some([].as_slice()));
    assert_eq!(view.spell_traditions().value(), Some([].as_slice()));
    assert_eq!(view.spell_rank().value().unwrap().as_u64(), Some(3));
    let missing = source_record("Item", json!({"_id":ROOT,"type":"spell","system":{}}));
    let query = SourceQueryView::new(missing.source(), "test", "Test");
    assert!(matches!(query.traits(), SourceFieldView::Missing));
    let deity = source_record(
        "Item",
        json!({"_id":ROOT,"type":"deity","system":{"traits":{"otherTags":[]}}}),
    );
    assert!(matches!(
        SourceQueryView::new(deity.source(), "test", "Test").traits(),
        SourceFieldView::NotApplicable
    ));
}

#[test]
fn duplicate_and_missing_child_ids_are_snapshot_local_in_authored_order() {
    let source = admitted(
        "Actor",
        json!({"_id":ROOT,"type":"npc","items":[{"_id":CHILD,"type":"spell"},{"_id":CHILD,"type":"spell"},{"type":"spell"},{"_id":"bad","type":"spell"},{"_id":OTHER,"type":"spell"}]}),
    );
    let key = source_record_key("test", &source).unwrap();
    let mut paths = Vec::new();
    visit_source_nodes(&source, |owners, _| paths.push(owners.to_vec()));
    assert_eq!(paths.len(), 6);
    for (index, owners) in paths.iter().skip(1).enumerate() {
        assert_eq!(
            owners[0].identity,
            if index == 4 {
                OwnedContentIdentity::Stable(OTHER.into())
            } else {
                OwnedContentIdentity::SnapshotLocal { index }
            }
        );
    }
    let record = SourceBackedRecord::new(key.pack().as_str(), source).unwrap();
    assert_eq!(source_owned_document_count(&record), 5);
}

#[test]
fn invalid_whole_collection_does_not_salvage_neighbors() {
    let record = source_record(
        "Actor",
        json!({"_id":ROOT,"type":"npc","items":[{"_id":CHILD,"type":"spell"},{"type":"unknown-family"}]}),
    );
    let query = SourceQueryView::new(record.source(), "test", "Test");
    assert!(matches!(query.actor().items(), SourceFieldView::Invalid(_)));
    assert_eq!(source_owned_document_count(&record), 0);
    let mut visited = 0;
    visit_source_nodes(record.source(), |_, _| visited += 1);
    assert_eq!(visited, 1);
}

#[test]
fn owned_spell_predicates_require_the_same_child() {
    // Test extraction/scope, not a second production filter evaluator.
    let matching = |input: Value| {
        let source = admitted("Actor", input);
        SourceNodeView::from(&source)
            .actor_items()
            .value()
            .unwrap()
            .iter()
            .any(|child| {
                let view = ItemSourceView::from(child);
                view.family() == "spell"
                    && view
                        .spell_rank()
                        .value()
                        .is_some_and(|rank| rank.as_u64().is_some_and(|rank| rank >= 3))
                    && view
                        .traits()
                        .value()
                        .is_some_and(|traits| traits.iter().any(|t| t == "fire"))
            })
    };
    let spell = |rank, traits| json!({"type":"spell","system":{"level":{"value":rank},"traits":{"value":traits}}});
    assert!(matching(
        json!({"_id":ROOT,"type":"npc","items":[spell(3,json!(["fire"]))]})
    ));
    assert!(!matching(
        json!({"_id":ROOT,"type":"npc","items":[spell(3,json!(["cold"])),spell(1,json!(["fire"]))]})
    ));
}

#[test]
fn nested_physical_and_consumable_spell_bodies_stay_owned_and_unchanged() {
    let source = admitted(
        "Item",
        json!({"_id":ROOT,"type":"weapon","system":{"subitems":[{"_id":CHILD,"type":"equipment","system":{"subitems":[{"_id":OTHER,"type":"consumable","system":{"spell":{"_id":ROOT,"type":"spell","system":{"description":{"value":"Child prose"}}}}}]}}]}}),
    );
    let before = encode_snapshot(&source).unwrap();
    let key = source_record_key("test", &source).unwrap();
    let record = SourceBackedRecord::new(key.pack().as_str(), source).unwrap();
    let mut depths = Vec::new();
    visit_source_nodes(record.source(), |owners, _| depths.push(owners.len()));
    assert_eq!(source_owned_document_count(&record), 3);
    assert_eq!(depths, [0, 1, 2, 3]);
    let content = prepare_record_content(&record, public(), None, None);
    assert_eq!(encode_snapshot(record.source()).unwrap(), before);
    let child = record
        .text_sources(&content, public(), "Test")
        .into_iter()
        .find(|s| s.text == "Child prose")
        .unwrap();
    assert_eq!(child.owners.len(), 3);
}

#[test]
fn resolves_owned_pages_and_actor_items_and_keeps_repeated_slot_occurrences() {
    let source = admitted(
        "Actor",
        json!({"_id":ROOT,"type":"npc","items":[{"_id":CHILD,"type":"spell","system":{"location":{"value":OTHER}}},{"_id":OTHER,"type":"spellcastingEntry","system":{"slots":{"slot3":{"prepared":[{"id":CHILD},{"id":CHILD}]}}}}]}),
    );
    let key = source_record_key("test", &source).unwrap();
    let mut index = SourceReferenceIndex::default();
    index.insert_source(&key, &source);
    let record = SourceBackedRecord::new(key.pack().as_str(), source).unwrap();
    let relationships = resolve_source_relationships(&record, Some(&index));
    let slots = relationships
        .iter()
        .filter(|r| r.kind == SourceRelationshipKind::PreparedSpell)
        .collect::<Vec<_>>();
    assert_eq!(slots.len(), 2);
    assert_ne!(slots[0].locator.field, slots[1].locator.field);
    assert!(slots.iter().all(|r| matches!(
        r.resolution,
        ContentReferenceResolution::Resolved(ContentReferenceTarget::OwnedNode { .. })
    )));
    let journal = admitted(
        "JournalEntry",
        json!({"_id":ROOT,"pages":[{"_id":CHILD,"name":"Page","type":"text","text":{"format":1,"content":"@UUID[Compendium.pf2e.journals.JournalEntry.aaaaaaaaaaaaaaaa.JournalEntryPage.bbbbbbbbbbbbbbbb]"}}]}),
    );
    let key = source_record_key("journals", &journal).unwrap();
    index.insert_source(&key, &journal);
    let journal = SourceBackedRecord::new(key.pack().as_str(), journal).unwrap();
    let content = prepare_record_content(&journal, public(), None, Some(&index));
    assert!(content.iter().any(|c|matches!(&c.status,SourceContentStatus::Prepared(p) if p.references.iter().any(|r|matches!(r.resolution,ContentReferenceResolution::Resolved(ContentReferenceTarget::OwnedNode {..}))))));
}

#[test]
fn ambiguous_names_missing_ids_and_duplicate_roots_never_pick_a_winner() {
    let mut index = SourceReferenceIndex::default();
    let source = admitted("Item", json!({"_id":ROOT,"type":"spell","name":"Same"}));
    let key = source_record_key("test", &source).unwrap();
    index.insert_source(&key, &source);
    let other = admitted("Item", json!({"_id":OTHER,"type":"spell","name":"Same"}));
    let other_key = source_record_key("test", &other).unwrap();
    index.insert_source(&other_key, &other);
    let locator = SourceContentLocator {
        record: key.clone(),
        owners: vec![],
        field: "/system/description/value".into(),
    };
    assert!(
        index
            .resolve_reference(&locator, "pf2e.test.Same")
            .is_none()
    );
    assert!(
        index
            .resolve_reference(&locator, "Compendium.pf2e.test.Item.Same")
            .is_none()
    );
    assert!(
        index
            .resolve_reference(&locator, "Compendium.pf2e.test.Item.aaaaaaaaaaaaaaaa")
            .is_some()
    );
    index.insert_source(&key, &source);
    assert!(
        index
            .resolve_reference(&locator, "Compendium.pf2e.test.Item.aaaaaaaaaaaaaaaa")
            .is_none()
    );
}

#[test]
fn journal_page_fragments_resolve_identity_and_retain_authored_navigation() {
    let target = format!(
        "Compendium.pf2e.journals.JournalEntry.{ROOT}.JournalEntryPage.{CHILD}#basic-spellcasting-feat"
    );
    let source = admitted(
        "JournalEntry",
        json!({"_id":ROOT,"pages":[{"_id":CHILD,"type":"text","text":{"format":1,"content":format!("@UUID[{target}]{{Heading}}")}}]}),
    );
    let key = source_record_key("journals", &source).unwrap();
    let mut index = SourceReferenceIndex::default();
    index.insert_source(&key, &source);
    let record = SourceBackedRecord::new(key.pack().as_str(), source).unwrap();
    let content = prepare_record_content(&record, public(), None, Some(&index));
    let reference = content
        .iter()
        .find_map(|content| {
            if let SourceContentStatus::Prepared(prepared) = &content.status {
                prepared.references.first()
            } else {
                None
            }
        })
        .unwrap();
    assert_eq!(reference.authored_target, target);
    assert!(matches!(&reference.resolution,
        ContentReferenceResolution::Resolved(ContentReferenceTarget::OwnedNode { key: resolved, owners })
        if resolved == &key && owners[0].identity == OwnedContentIdentity::Stable(CHILD.into())));
    let locator = content[0].locator();
    assert!(
        index
            .resolve_reference(
                locator,
                &format!(
                    "Compendium.pf2e.journals.JournalEntry.{ROOT}.JournalEntryPage.{OTHER}#heading"
                )
            )
            .is_none()
    );
    assert!(
        index
            .resolve_reference(
                locator,
                &format!("Compendium.pf2e.journals.JournalEntry.{ROOT}#heading")
            )
            .is_none()
    );
    index.insert_source(&key, record.source());
    assert!(index.resolve_reference(locator, &target).is_none());
}

#[test]
fn relative_page_ids_use_the_current_journal_and_never_guess_other_scopes() {
    let source = admitted(
        "JournalEntry",
        json!({"_id":ROOT,"pages":[
            {"_id":CHILD,"type":"text","text":{"format":1,"content":format!("@UUID[.{OTHER}] @UUID[.{OTHER}#heading]")}},
            {"_id":OTHER,"name":"Sibling","type":"text"}
        ]}),
    );
    let key = source_record_key("journals", &source).unwrap();
    let mut index = SourceReferenceIndex::default();
    index.insert_source(&key, &source);
    let record = SourceBackedRecord::new(key.pack().as_str(), source).unwrap();
    let content = prepare_record_content(&record, public(), None, Some(&index));
    let prepared = content
        .iter()
        .find_map(|content| match &content.status {
            SourceContentStatus::Prepared(p) if p.references.len() == 2 => Some(p),
            _ => None,
        })
        .unwrap();
    assert_eq!(prepared.references[0].authored_target, format!(".{OTHER}"));
    assert_eq!(
        prepared.references[1].authored_target,
        format!(".{OTHER}#heading")
    );
    assert!(prepared.references.iter().all(|r| matches!(&r.resolution,
        ContentReferenceResolution::Resolved(ContentReferenceTarget::OwnedNode { key: resolved, owners })
        if resolved == &key && owners[0].identity == OwnedContentIdentity::Stable(OTHER.into()))));
    let root_locator = SourceContentLocator {
        record: key,
        owners: vec![],
        field: "/name".into(),
    };
    assert!(
        index
            .resolve_reference(&root_locator, &format!(".{OTHER}"))
            .is_none()
    );
    assert!(
        index
            .resolve_reference(&prepared.locator, ".dddddddddddddddd")
            .is_none()
    );
    assert!(
        index
            .resolve_reference(&prepared.locator, &format!("..{OTHER}"))
            .is_none()
    );

    let duplicate = admitted(
        "JournalEntry",
        json!({"_id":ROOT,"pages":[{"_id":OTHER},{"_id":OTHER}]}),
    );
    let mut index = SourceReferenceIndex::default();
    index.insert_source(&root_locator.record, &duplicate);
    assert!(
        index
            .resolve_reference(&prepared.locator, &format!(".{OTHER}"))
            .is_none()
    );
}

#[test]
fn journal_formats_do_not_default_to_html_and_plain_captions_bypass_macros() {
    for (format, expected) in [
        (Some(json!(1)), "prepared"),
        (Some(json!(2)), "unsupported"),
        (None, "unavailable"),
        (Some(json!(null)), "unavailable"),
        (Some(json!(9)), "unavailable"),
    ] {
        let mut text = json!({"content":"","markdown":"Markdown"});
        if let Some(format) = format {
            text["format"] = format;
        }
        let record = source_record(
            "JournalEntry",
            json!({"_id":ROOT,"pages":[{"_id":CHILD,"type":"text","text":text,"image":{"caption":"@UUID[foo] <b>literal</b>"}}]}),
        );
        let content = prepare_record_content(&record, public(), None, None);
        let body = content
            .iter()
            .find(|c| c.locator().field.starts_with("/text/"))
            .unwrap();
        assert!(match (&body.status, expected) {
            (SourceContentStatus::Prepared(p), "prepared") =>
                p.html.is_empty() && p.text.is_empty(),
            (SourceContentStatus::UnsupportedFormat { .. }, "unsupported") => true,
            (SourceContentStatus::FormatUnavailable { .. }, "unavailable") => true,
            _ => false,
        });
        assert!(!content.iter().any(|c| c.role == SourceContentRole::Caption));
        assert!(
            record
                .text_sources(&content, public(), "Test")
                .iter()
                .any(|t| t.text == "@UUID[foo] <b>literal</b>")
        );
    }
}

#[test]
fn macro_commands_are_source_data_and_table_results_have_owned_identity() {
    let macro_record = source_record(
        "Macro",
        json!({"_id":ROOT,"type":"script","name":"Run","command":"@UUID[foo]; console.log('x')"}),
    );
    let content = prepare_record_content(&macro_record, public(), None, None);
    assert!(content.is_empty());
    let text = macro_record.text_sources(&content, public(), "Test");
    assert!(text.iter().any(|t| t.text == "Run"));
    assert!(!text.iter().any(|t| t.text.contains("console.log")));
    let table = source_record(
        "RollTable",
        json!({"_id":ROOT,"description":"<p>Table</p>","results":[{"_id":CHILD,"text":"@Damage[1d6]"}]}),
    );
    assert_eq!(source_owned_document_count(&table), 1);
    let content = prepare_record_content(&table, public(), None, None);
    assert!(
        content
            .iter()
            .any(|c| c.role == SourceContentRole::TableResult)
    );
}

#[test]
fn party_html_and_army_declared_trait_states_are_selected() {
    let party = source_record(
        "Actor",
        json!({"_id":ROOT,"type":"party","system":{"details":{"description":"<p>@Damage[1d6]</p>"}}}),
    );
    let content = prepare_record_content(&party, public(), None, None);
    assert!(content.iter().any(|c|matches!(&c.status,SourceContentStatus::Prepared(p) if p.text=="1d6"&&p.interactions.len()==1)));
    for (traits, expected) in [
        (
            json!({"value":["new-army-trait"],"rarity":"rare"}),
            FieldAvailability::Value,
        ),
        (json!(null), FieldAvailability::Null),
        (
            json!(3),
            FieldAvailability::Invalid {
                source_path: "$.system.traits".into(),
            },
        ),
        (json!({}), FieldAvailability::Missing),
    ] {
        let army = source_record(
            "Actor",
            json!({"_id":ROOT,"type":"army","system":{"traits":traits}}),
        );
        let query = SourceQueryView::new(army.source(), "test", "Test");
        assert_eq!(query.traits().availability(), expected);
        assert_eq!(query.rarity().availability(), expected);
        if let Some(traits) = query.traits().value() {
            assert_eq!(traits, ["new-army-trait"]);
        }
    }
}

#[test]
fn qualified_authored_names_resolve_and_names_never_poison_ids() {
    let mut index = SourceReferenceIndex::default();
    let source = admitted("Item", json!({"_id":ROOT,"type":"spell","name":ROOT}));
    let key = source_record_key("test", &source).unwrap();
    index.insert_source(&key, &source);
    let locator = SourceContentLocator {
        record: key.clone(),
        owners: vec![],
        field: "/system/description/value".into(),
    };
    assert!(
        index
            .resolve_reference(&locator, &format!("Compendium.pf2e.test.{ROOT}"))
            .is_some()
    );
    let other = admitted("Item", json!({"_id":OTHER,"type":"spell","name":ROOT}));
    index.insert_source(&source_record_key("test", &other).unwrap(), &other);
    assert!(
        matches!(index.resolve_reference(&locator,&format!("Compendium.pf2e.test.{ROOT}")).unwrap().target,ContentReferenceTarget::Record {key:k} if k==key)
    );
    let named = admitted(
        "Item",
        json!({"_id":CHILD,"type":"spell","name":"Name with. punctuation"}),
    );
    index.insert_source(&source_record_key("test", &named).unwrap(), &named);
    assert!(
        index
            .resolve_reference(&locator, "Compendium.pf2e.test.Item.Name with. punctuation")
            .is_some()
    );
    assert!(
        index
            .resolve_reference(&locator, "pf2e.test.Item.Name with. punctuation")
            .is_some()
    );
}

#[test]
fn grants_and_provenance_are_typed_and_casting_references_check_target_family() {
    let source = admitted(
        "Actor",
        json!({"_id":ROOT,"type":"npc","items":[
            {"_id":CHILD,"type":"spell","_stats":{"compendiumSource":"Compendium.pf2e.test.Item.Spell"},"system":{"location":{"value":OTHER}},"flags":{"pf2e":{"itemGrants":{"with/slash":{"id":OTHER}}}}},
            {"_id":OTHER,"type":"weapon","flags":{"pf2e":{"grantedBy":{"id":CHILD}}}},
            {"_id":"dddddddddddddddd","type":"spellcastingEntry","system":{"slots":{"slot3":{"prepared":[{"id":OTHER}]}}}}
        ]}),
    );
    let key = source_record_key("actors", &source).unwrap();
    let mut index = SourceReferenceIndex::default();
    index.insert_source(&key, &source);
    let spell = admitted("Item", json!({"_id":ROOT,"type":"spell","name":"Spell"}));
    index.insert_source(&source_record_key("test", &spell).unwrap(), &spell);
    let record = SourceBackedRecord::new(key.pack().as_str(), source).unwrap();
    let relations = resolve_source_relationships(&record, Some(&index));
    for kind in [
        SourceRelationshipKind::ItemGrant,
        SourceRelationshipKind::GrantedBy,
        SourceRelationshipKind::CompendiumProvenance,
    ] {
        assert!(
            relations.iter().any(|r| r.kind == kind
                && matches!(r.resolution, ContentReferenceResolution::Resolved(_)))
        );
    }
    assert!(
        relations
            .iter()
            .any(|r| r.locator.field.contains("with~1slash"))
    );
    for kind in [
        SourceRelationshipKind::SpellcastingEntry,
        SourceRelationshipKind::PreparedSpell,
    ] {
        assert!(
            relations
                .iter()
                .any(|r| r.kind == kind && r.resolution == ContentReferenceResolution::Unresolved)
        );
    }
}

#[test]
fn resolved_embed_cycles_are_occurrences_without_expansion_and_markers_stay_local() {
    let source = admitted(
        "Item",
        json!({"_id":ROOT,"type":"spell","name":"Spell","system":{"description":{"value":"@Embed[Compendium.pf2e.test.Item.Spell inline] @UUID[Compendium.pf2e.test.Item.Spell]"}}}),
    );
    let key = source_record_key("test", &source).unwrap();
    let mut index = SourceReferenceIndex::default();
    index.insert_source(&key, &source);
    let record = SourceBackedRecord::new(key.pack().as_str(), source).unwrap();
    let content = prepare_record_content(&record, public(), None, Some(&index));
    let field = content
        .iter()
        .find(|c| c.locator().field == "/system/description/value")
        .unwrap();
    let SourceContentStatus::Prepared(p) = &field.status else {
        panic!();
    };
    assert_eq!(p.references.len(), 2);
    assert!(
        p.references
            .iter()
            .all(|r| matches!(r.resolution, ContentReferenceResolution::Resolved(_)))
    );
    assert!(
        matches!(&p.references[0].kind,ContentReferenceKind::Embed {options} if options.contains_key("inline"))
    );
    assert_eq!(p.text, "Spell Spell");
    assert!(
        p.diagnostics
            .iter()
            .any(|d| d.code == ContentDiagnosticCode::EmbedNotExpanded)
    );
}

#[test]
fn root_identity_rejections_and_ordered_additional_pairs_survive() {
    for (id, expected) in [
        (json!(null), SourceIdentityError::NullId),
        (json!(7), SourceIdentityError::RejectedId),
        (json!(""), SourceIdentityError::InvalidId),
    ] {
        let source = admitted("Item", json!({"_id":id,"type":"spell"}));
        assert_eq!(source_record_key("test", &source), Err(expected));
    }
    let bytes=br#"{"_id":"aaaaaaaaaaaaaaaa","type":"spell","extra":{"z":1,"z":2,"a":false},"system":{"description":{"value":""}}}"#;
    let source = admit_document_source("Item", SourceContext::new("test", "fixture", "$"), bytes)
        .unwrap()
        .model
        .unwrap();
    let snapshot = encode_snapshot(&source).unwrap();
    let key = source_record_key("test", &source).unwrap();
    let record = SourceBackedRecord::new(key.pack().as_str(), source).unwrap();
    assert_eq!(encode_snapshot(record.source()).unwrap(), snapshot);
    assert_eq!(&decode_snapshot(&snapshot).unwrap(), record.source());
    assert_eq!(
        source_record_key("bad:pack", record.source()),
        Err(SourceIdentityError::InvalidPack)
    );
}

#[test]
fn biography_gates_use_the_authored_section_and_unknown_visibility_retains_hidden_references() {
    let source = admitted(
        "Actor",
        json!({"_id":ROOT,"type":"character","system":{"details":{"biography":{"allies":"@UUID[Compendium.pf2e.test.Item.Spell]","attitude":"@Damage[1d6]","visibility":{"campaign":false,"personality":true}}}}}),
    );
    let key = source_record_key("actors", &source).unwrap();
    let mut index = SourceReferenceIndex::default();
    let spell = admitted("Item", json!({"_id":ROOT,"name":"Spell","type":"spell"}));
    index.insert_source(&source_record_key("test", &spell).unwrap(), &spell);
    let record = SourceBackedRecord::new(key.pack().as_str(), source).unwrap();
    let content = prepare_record_content(&record, public(), None, Some(&index));
    let allies = content
        .iter()
        .find(|c| c.locator().field.ends_with("/allies"))
        .unwrap();
    assert_eq!(allies.visibility, ContentVisibilityRule::Owner);
    assert!(
        matches!(&allies.status,SourceContentStatus::Prepared(p) if p.text.is_empty()&&p.references.len()==1&&!p.references[0].visible)
    );
    assert!(
        !content
            .iter()
            .any(|c| c.locator().field.ends_with("/attitude"))
    );
    assert!(
        record
            .text_sources(&content, public(), "Test")
            .iter()
            .any(|t| t.text == "@Damage[1d6]")
    );
    let missing = source_record(
        "Actor",
        json!({"_id":ROOT,"type":"character","system":{"details":{"biography":{"backstory":"@UUID[Compendium.pf2e.test.Item.Spell]"}}}}),
    );
    let content = prepare_record_content(&missing, public(), None, None);
    let backstory = content
        .iter()
        .find(|c| c.locator().field.ends_with("/backstory"))
        .unwrap();
    assert_eq!(
        backstory.visibility_availability,
        FieldAvailability::Missing
    );
    assert!(
        matches!(&backstory.status,SourceContentStatus::Prepared(p) if p.html.is_empty()&&p.text.is_empty()&&p.references.len()==1&&!p.references[0].visible)
    );
}

#[test]
fn preparation_is_sparse_but_preserves_present_empty_and_hidden_fields() {
    for description in [
        json!({}),
        json!(null),
        json!(3),
        json!({"value":null}),
        json!({"value":7}),
    ] {
        let record = source_record(
            "Item",
            json!({"_id":ROOT,"type":"spell","system":{"description":description}}),
        );
        let before = encode_snapshot(record.source()).unwrap();
        assert!(prepare_record_content(&record, public(), None, None).is_empty());
        assert_eq!(encode_snapshot(record.source()).unwrap(), before);
    }
    let record = source_record(
        "Item",
        json!({"_id":ROOT,"name":"Plain name","type":"spell","system":{"description":{"value":"","gm":"Private text"}}}),
    );
    let before = encode_snapshot(record.source()).unwrap();
    let content = prepare_record_content(&record, public(), None, None);
    assert_eq!(content.len(), 2);
    let empty = content
        .iter()
        .find(|c| c.locator().field == "/system/description/value")
        .unwrap();
    assert!(
        matches!(&empty.status,SourceContentStatus::Prepared(p) if p.html.is_empty() && p.text.is_empty())
    );
    let hidden = content
        .iter()
        .find(|c| c.locator().field == "/system/description/gm")
        .unwrap();
    assert_eq!(hidden.visibility, ContentVisibilityRule::Gm);
    assert!(
        matches!(&hidden.status,SourceContentStatus::Prepared(p) if p.html.is_empty() && p.text.is_empty())
    );
    assert!(!content.iter().any(|c| c.locator().field == "/name"));
    assert_eq!(encode_snapshot(record.source()).unwrap(), before);
}

#[test]
fn actor_collection_access_preserves_empty_and_unavailable_states_without_inventory() {
    for (items, expected) in [
        (None, FieldAvailability::Missing),
        (Some(json!(null)), FieldAvailability::Null),
        (
            Some(json!(3)),
            FieldAvailability::Invalid {
                source_path: "$.items".into(),
            },
        ),
        (Some(json!([])), FieldAvailability::Value),
    ] {
        let mut input = json!({"_id":ROOT,"type":"npc"});
        if let Some(items) = items {
            input["items"] = items;
        }
        let record = source_record("Actor", input);
        let query = SourceQueryView::new(record.source(), "test", "Test");
        let items = query.actor().items();
        assert_eq!(items.availability(), expected);
        if let Some(items) = items.value() {
            assert!(items.is_empty());
        }
        assert_eq!(source_owned_document_count(&record), 0);
    }
    let item = source_record("Item", json!({"_id":ROOT,"type":"spell"}));
    assert!(matches!(
        SourceQueryView::new(item.source(), "test", "Test")
            .actor()
            .items(),
        SourceFieldView::NotApplicable
    ));
}

#[test]
fn explicit_preparation_context_can_change_output_without_changing_source() {
    struct Locale(&'static str);
    impl LocalizationResolver for Locale {
        fn localized_value(&self, key: &str) -> Option<&str> {
            (key == "Greeting").then_some(self.0)
        }
    }
    let record = source_record(
        "Item",
        json!({"_id":ROOT,"type":"spell","system":{"description":{"value":"@Localize[Greeting]","gm":"Private text"}}}),
    );
    let before = encode_snapshot(record.source()).unwrap();
    let english = prepare_record_content(&record, public(), Some(&Locale("Hello")), None);
    let other_locale = prepare_record_content(&record, public(), Some(&Locale("Bonjour")), None);
    let gm_audience = ContentAudience {
        include_gm: true,
        ..public()
    };
    let gm = prepare_record_content(&record, gm_audience, Some(&Locale("Hello")), None);
    let text = |content: &[SourceContentOutcome], field: &str| {
        content
            .iter()
            .find_map(|c| match &c.status {
                SourceContentStatus::Prepared(p) if c.locator().field == field => {
                    Some(p.text.clone())
                }
                _ => None,
            })
            .unwrap()
    };
    assert_eq!(text(&english, "/system/description/value"), "Hello");
    assert_eq!(text(&other_locale, "/system/description/value"), "Bonjour");
    assert_eq!(text(&english, "/system/description/gm"), "");
    assert_eq!(text(&gm, "/system/description/gm"), "Private text");
    assert_eq!(encode_snapshot(record.source()).unwrap(), before);
}

#[test]
fn absent_journal_text_has_no_format_failure_outcome() {
    for text in [
        json!({"format":2}),
        json!({"format":9}),
        json!({"content":null}),
        json!({"content":7}),
    ] {
        let record = source_record(
            "JournalEntry",
            json!({"_id":ROOT,"pages":[{"_id":CHILD,"type":"text","text":text}]}),
        );
        assert!(prepare_record_content(&record, public(), None, None).is_empty());
    }
}

#[test]
fn checked_construction_derives_identity_and_preserves_the_complete_body() {
    let source = admitted("Item", item("spell"));
    let before = encode_snapshot(&source).unwrap();
    let record = SourceBackedRecord::new("spells", source).unwrap();
    assert_eq!(record.key().pack().as_str(), "spells");
    assert_eq!(record.key().id().as_str(), ROOT);
    assert_eq!(encode_snapshot(record.source()).unwrap(), before);
    for pack in ["", "bad pack", "bad:pack"] {
        let source = admitted("Item", item("spell"));
        let before = source.clone();
        let error = SourceBackedRecord::new(pack, source).unwrap_err();
        assert_eq!(error.reason, SourceIdentityError::InvalidPack);
        assert_eq!(error.source, before);
    }
    for (id, expected) in [
        (None, SourceIdentityError::MissingId),
        (Some(json!(null)), SourceIdentityError::NullId),
        (Some(json!(7)), SourceIdentityError::RejectedId),
        (Some(json!("bad")), SourceIdentityError::InvalidId),
    ] {
        let mut input = json!({"type":"spell"});
        if let Some(id) = id {
            input["_id"] = id;
        }
        let source = admitted("Item", input);
        let before = source.clone();
        let error = SourceBackedRecord::new("spells", source).unwrap_err();
        assert_eq!(error.reason, expected);
        assert_eq!(error.source, before);
    }
}

#[test]
fn traversal_unwinds_owner_paths_between_nested_children_and_root_siblings() {
    let source = admitted(
        "Actor",
        json!({"_id":ROOT,"type":"npc","items":[
            {"_id":CHILD,"type":"weapon","system":{"subitems":[
                {"_id":OTHER,"type":"equipment","system":{"subitems":[]}}
            ]}},
            {"_id":OTHER,"type":"spell"}
        ]}),
    );
    let mut visits = Vec::new();
    visit_source_nodes(&source, |owners, node| {
        visits.push((node.id().value().unwrap().as_str(), owners.to_vec()));
    });
    let locator = |collection: &str, id: &str| OwnedContentLocator {
        collection: collection.into(),
        identity: OwnedContentIdentity::Stable(id.into()),
    };
    assert_eq!(
        visits,
        vec![
            (ROOT, vec![]),
            (CHILD, vec![locator("/items", CHILD)]),
            (
                OTHER,
                vec![locator("/items", CHILD), locator("/system/subitems", OTHER)]
            ),
            (OTHER, vec![locator("/items", OTHER)]),
        ]
    );
}
