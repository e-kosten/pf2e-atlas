use super::*;
use crate::source_content::*;
use atlas_foundry_model::{SourceContext, admit_document_source, decode_snapshot, encode_snapshot};
use serde_json::{Value, json};

const ROOT: &str = "aaaaaaaaaaaaaaaa";
const CHILD: &str = "bbbbbbbbbbbbbbbb";
const OTHER: &str = "cccccccccccccccc";
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
fn enriched(kind: &str, value: Value) -> SourceBackedRecord {
    let source = admitted(kind, value);
    let key = source_record_key("test", &source).unwrap();
    enrich_source_record(key, source, public(), None, None)
}
fn item(family: &str) -> Value {
    json!({"_id":ROOT,"name":"Élan @Damage[1d6]","type":family,"system":{"description":{"value":"<p>Authored — 力 @Damage[1d6]</p>","gm":"@UUID[Compendium.pf2e.test.Item.aaaaaaaaaaaaaaaa] @Check[fortitude|dc:20]"},"publication":{"title":"Test","remaster":false}}})
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
        let record = enriched(
            "Actor",
            json!({"_id":ROOT,"name":family,"type":family,"system":{"traits":{"value":["new-trait"],"rarity":"rare"}},"items":[]}),
        );
        assert_eq!(
            SourceNodeView::from(&record.source).source_type().value(),
            Some(family)
        );
        assert_eq!(record.enrichment.collections[0].length, Some(0));
        let query = SourceQueryView::new(&record.source, "test", "Test");
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
        let record = enriched("Item", input);
        let query = SourceQueryView::new(&record.source, "test", "Test pack");
        assert_eq!(query.source.source_type().value(), Some(family));
        assert_eq!(query.publication_remaster().value(), Some(false));
        let body = record
            .enrichment
            .content
            .iter()
            .find(|c| c.locator.field == "/system/description/value")
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
    let record = enrich_source_record(key, source, public(), None, Some(&resolver));
    let gm = record
        .enrichment
        .content
        .iter()
        .find(|c| c.locator.field == "/system/description/gm")
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
    let name = &record.enrichment.content[0];
    assert!(
        matches!(&name.status,SourceContentStatus::Prepared(p) if p.text=="Élan @Damage[1d6]" && p.interactions.is_empty())
    );
    assert!(
        !record
            .text_sources("Test")
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
    let record = enrich_source_record(key, source, public(), None, None);
    assert_eq!(encode_snapshot(&record.source).unwrap(), snapshot);
    assert_eq!(decode_snapshot(&snapshot).unwrap(), record.source);
    let SourceNodeView::Item(item) = SourceNodeView::from(&record.source) else {
        panic!();
    };
    assert!(matches!(item.spell_rank(), SourceFieldView::Invalid(_)));
    let bytes = serde_json::to_vec(&record.enrichment).unwrap();
    assert_eq!(
        serde_json::from_slice::<SourceRecordEnrichment>(&bytes).unwrap(),
        record.enrichment
    );
}

#[test]
fn projections_preserve_baselines_open_sets_zero_false_and_ancestor_states() {
    let record = enriched(
        "Actor",
        json!({"_id":ROOT,"type":"npc","system":{"details":{"level":{"value":0},"publication":{"remaster":false}},"attributes":{"ac":{"value":21},"hp":{"max":80,"value":12}},"traits":{"value":["brand-new-provider-trait"],"rarity":"rare"}},"items":[]}),
    );
    let query = SourceQueryView::new(&record.source, "test", "Test");
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
        let record = enriched("Actor", json!({"_id":ROOT,"type":"npc","system":system}));
        let query = SourceQueryView::new(&record.source, "test", "Test");
        assert_eq!(query.actor().hp_maximum().availability(), availability);
    }
    let hazard = enriched(
        "Actor",
        json!({"_id":ROOT,"type":"hazard","system":{"details":{"isComplex":false},"attributes":{"hardness":0}}}),
    );
    let query = SourceQueryView::new(&hazard.source, "test", "Test");
    assert_eq!(query.actor().hazard_complexity().value(), Some(false));
    assert_eq!(
        query.actor().hazard_hardness().value().unwrap().as_u64(),
        Some(0)
    );
}

#[test]
fn empty_unknown_and_not_applicable_are_different() {
    let empty = enriched(
        "Item",
        json!({"_id":ROOT,"type":"spell","system":{"traits":{"value":[],"traditions":[]},"level":{"value":3},"location":{"heightenedLevel":8}}}),
    );
    let SourceNodeView::Item(view) = SourceNodeView::from(&empty.source) else {
        panic!();
    };
    assert_eq!(view.traits().value(), Some([].as_slice()));
    assert_eq!(view.spell_traditions().value(), Some([].as_slice()));
    assert_eq!(view.spell_rank().value().unwrap().as_u64(), Some(3));
    let missing = enriched("Item", json!({"_id":ROOT,"type":"spell","system":{}}));
    let query = SourceQueryView::new(&missing.source, "test", "Test");
    assert!(matches!(query.traits(), SourceFieldView::Missing));
    let deity = enriched(
        "Item",
        json!({"_id":ROOT,"type":"deity","system":{"traits":{"otherTags":[]}}}),
    );
    assert!(matches!(
        SourceQueryView::new(&deity.source, "test", "Test").traits(),
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
    let (nodes, collections) = source_nodes(&key, &source);
    assert_eq!(collections[0].length, Some(5));
    for (index, entry) in nodes.iter().skip(1).enumerate() {
        assert_eq!(entry.order, index);
        assert_eq!(
            entry.owners[0].identity,
            if index == 4 {
                OwnedContentIdentity::Stable(OTHER.into())
            } else {
                OwnedContentIdentity::SnapshotLocal { index }
            }
        );
    }
    let record = enrich_source_record(key, source, public(), None, None);
    assert_eq!(record.enrichment.owned_nodes.len(), 5);
}

#[test]
fn invalid_whole_collection_does_not_salvage_neighbors() {
    let record = enriched(
        "Actor",
        json!({"_id":ROOT,"type":"npc","items":[{"_id":CHILD,"type":"spell"},{"type":"unknown-family"}]}),
    );
    let query = SourceQueryView::new(&record.source, "test", "Test");
    assert!(matches!(query.actor().items(), SourceFieldView::Invalid(_)));
    assert!(record.enrichment.owned_nodes.is_empty());
    assert_eq!(record.enrichment.collections[0].length, None);
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
    let record = enrich_source_record(key, source, public(), None, None);
    assert_eq!(record.enrichment.owned_nodes.len(), 3);
    assert_eq!(record.enrichment.owned_nodes[2].owners.len(), 3);
    assert_eq!(encode_snapshot(&record.source).unwrap(), before);
    let child = record
        .text_sources("Test")
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
    let record = enrich_source_record(key.clone(), source, public(), None, Some(&index));
    let slots = record
        .enrichment
        .relationships
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
    let journal = enrich_source_record(key, journal, public(), None, Some(&index));
    assert!(journal.enrichment.content.iter().any(|c|matches!(&c.status,SourceContentStatus::Prepared(p) if p.references.iter().any(|r|matches!(r.resolution,ContentReferenceResolution::Resolved(ContentReferenceTarget::OwnedNode {..}))))));
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
    let record = enrich_source_record(key.clone(), source, public(), None, Some(&index));
    let reference = record
        .enrichment
        .content
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
    let locator = &record.enrichment.content[0].locator;
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
    index.insert_source(&key, &record.source);
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
    let record = enrich_source_record(key.clone(), source, public(), None, Some(&index));
    let prepared = record
        .enrichment
        .content
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
        let record = enriched(
            "JournalEntry",
            json!({"_id":ROOT,"pages":[{"_id":CHILD,"type":"text","text":text,"image":{"caption":"@UUID[foo] <b>literal</b>"}}]}),
        );
        let body = record
            .enrichment
            .content
            .iter()
            .find(|c| c.locator.field.starts_with("/text/"))
            .unwrap();
        assert!(match (&body.status, expected) {
            (SourceContentStatus::Prepared(p), "prepared") =>
                p.html.is_empty() && p.text.is_empty(),
            (SourceContentStatus::UnsupportedFormat, "unsupported") => true,
            (SourceContentStatus::FormatUnavailable(_), "unavailable") => true,
            _ => false,
        });
        let caption = record
            .enrichment
            .content
            .iter()
            .find(|c| c.role == SourceContentRole::Caption)
            .unwrap();
        assert!(
            matches!(&caption.status,SourceContentStatus::Prepared(p) if p.text=="@UUID[foo] <b>literal</b>"&&p.references.is_empty()&&p.html.contains("&lt;b&gt;"))
        );
    }
}

#[test]
fn macro_commands_are_source_data_and_table_results_have_owned_identity() {
    let macro_record = enriched(
        "Macro",
        json!({"_id":ROOT,"type":"script","name":"Run","command":"@UUID[foo]; console.log('x')"}),
    );
    assert_eq!(macro_record.enrichment.content.len(), 1);
    let table = enriched(
        "RollTable",
        json!({"_id":ROOT,"description":"<p>Table</p>","results":[{"_id":CHILD,"text":"@Damage[1d6]"}]}),
    );
    assert_eq!(table.enrichment.owned_nodes.len(), 1);
    assert!(
        table
            .enrichment
            .content
            .iter()
            .any(|c| c.role == SourceContentRole::TableResult)
    );
}

#[test]
fn party_html_and_army_declared_trait_states_are_selected() {
    let party = enriched(
        "Actor",
        json!({"_id":ROOT,"type":"party","system":{"details":{"description":"<p>@Damage[1d6]</p>"}}}),
    );
    assert!(party.enrichment.content.iter().any(|c|matches!(&c.status,SourceContentStatus::Prepared(p) if p.text=="1d6"&&p.interactions.len()==1)));
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
        let army = enriched(
            "Actor",
            json!({"_id":ROOT,"type":"army","system":{"traits":traits}}),
        );
        let query = SourceQueryView::new(&army.source, "test", "Test");
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
    let record = enrich_source_record(key, source, public(), None, Some(&index));
    let relations = &record.enrichment.relationships;
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
    let record = enrich_source_record(key, source, public(), None, Some(&index));
    let field = record
        .enrichment
        .content
        .iter()
        .find(|c| c.locator.field == "/system/description/value")
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
    let record = enrich_source_record(key, source, public(), None, None);
    assert_eq!(encode_snapshot(&record.source).unwrap(), snapshot);
    assert_eq!(decode_snapshot(&snapshot).unwrap(), record.source);
    assert_eq!(
        source_record_key("bad:pack", &record.source),
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
    let record = enrich_source_record(key, source, public(), None, Some(&index));
    let allies = record
        .enrichment
        .content
        .iter()
        .find(|c| c.locator.field.ends_with("/allies"))
        .unwrap();
    assert_eq!(allies.visibility, ContentVisibilityRule::Owner);
    assert!(
        matches!(&allies.status,SourceContentStatus::Prepared(p) if p.text.is_empty()&&p.references.len()==1&&!p.references[0].visible)
    );
    let attitude = record
        .enrichment
        .content
        .iter()
        .find(|c| c.locator.field.ends_with("/attitude"))
        .unwrap();
    assert!(
        matches!(&attitude.status,SourceContentStatus::Prepared(p) if p.text=="@Damage[1d6]"&&p.interactions.is_empty())
    );
    let missing = enriched(
        "Actor",
        json!({"_id":ROOT,"type":"character","system":{"details":{"biography":{"backstory":"@UUID[Compendium.pf2e.test.Item.Spell]"}}}}),
    );
    let backstory = missing
        .enrichment
        .content
        .iter()
        .find(|c| c.locator.field.ends_with("/backstory"))
        .unwrap();
    assert_eq!(
        backstory.visibility_availability,
        FieldAvailability::Missing
    );
    assert!(
        matches!(&backstory.status,SourceContentStatus::Prepared(p) if p.html.is_empty()&&p.text.is_empty()&&p.references.len()==1&&!p.references[0].visible)
    );
}
