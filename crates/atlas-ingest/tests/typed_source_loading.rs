use std::fs;
use std::path::PathBuf;
use std::sync::atomic::{AtomicUsize, Ordering};

use atlas_foundry_model::FoundryDocumentSource;
use atlas_foundry_model::{ActorSourcePF2e, ItemSourcePF2e, SourcePresence};
use atlas_ingest::{SourceLoadFailureStage, load_foundry_documents};
use serde_json::json;
use sha2::{Digest, Sha256};

struct Source(PathBuf);
impl Source {
    fn new(packs: serde_json::Value) -> Self {
        static NEXT: AtomicUsize = AtomicUsize::new(0);
        let root = std::env::temp_dir().join(format!(
            "atlas typed source {} {}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir_all(&root).unwrap();
        fs::write(
            root.join("module.json"),
            json!({"packs": packs}).to_string(),
        )
        .unwrap();
        Self(root)
    }
    fn write(&self, path: &str, bytes: impl AsRef<[u8]>) {
        let path = self.0.join(path);
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(path, bytes).unwrap();
    }
}
impl Drop for Source {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}
fn pack(name: &str, kind: &str) -> serde_json::Value {
    json!({"name":name,"label":name,"type":kind,"path":format!("packs/{name}")})
}

#[test]
fn dispatches_every_registered_family_and_all_five_document_kinds() {
    let source = Source::new(json!([
        pack("actors", "Actor"),
        pack("items", "Item"),
        pack("journals", "JournalEntry"),
        pack("macros", "Macro"),
        pack("tables", "RollTable")
    ]));
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
        source.write(
            &format!("packs/actors/{family}.json"),
            json!({"type":family}).to_string(),
        );
    }
    for family in [
        "action",
        "affliction",
        "ancestry",
        "armor",
        "background",
        "backpack",
        "book",
        "campaignFeature",
        "class",
        "condition",
        "consumable",
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
        source.write(
            &format!("packs/items/{family}.json"),
            json!({"type":family}).to_string(),
        );
    }
    source.write("packs/journals/entry.json", br#"{"pages":[]}"#);
    source.write(
        "packs/macros/macro.json",
        br#"{"type":"script","command":"return 1;"}"#,
    );
    source.write("packs/tables/table.json", br#"{"results":[]}"#);
    let loaded = load_foundry_documents(&source.0, None).unwrap();
    let report = loaded.report();
    assert_eq!(report.pack_count, 5);
    assert_eq!(report.counts.discovered_files, 35);
    assert_eq!(report.counts.modeled_documents, 35);
    assert_eq!(report.counts.diagnostic_free_documents, 35);
    assert_eq!(report.counts.quarantined_files, 0);
    assert!(report.diagnostics.is_empty());
    assert!(report.failures.is_empty());
    for (pack, expected) in
        loaded
            .packs
            .iter()
            .zip(["Actor", "Item", "JournalEntry", "Macro", "RollTable"])
    {
        assert_eq!(pack.document_type, expected);
        for document in &pack.documents {
            let kind = match document.admission.model.as_ref().unwrap() {
                FoundryDocumentSource::Actor(_) => "Actor",
                FoundryDocumentSource::Item(_) => "Item",
                FoundryDocumentSource::JournalEntry(_) => "JournalEntry",
                FoundryDocumentSource::Macro(_) => "Macro",
                FoundryDocumentSource::RollTable(_) => "RollTable",
            };
            assert_eq!(kind, expected);
        }
    }
}

#[test]
fn preserves_authored_bytes_repeated_members_and_partial_fields_without_defaults() {
    let source = Source::new(json!([pack("items", "Item")]));
    let bytes = br#" { "type":"ancestry", "name":null, "system":{"items":{"feature":{"uuid":"Compendium.pf2e.features.Item.x","level":"1"}}}, "future":9007199254740993, "future":null } "#;
    source.write("packs/items/ancestry.json", bytes);
    let loaded = load_foundry_documents(&source.0, None).unwrap();
    let document = &loaded.packs[0].documents[0];
    assert_eq!(document.bytes, bytes);
    assert_eq!(
        document.content_hash,
        format!("{:x}", Sha256::digest(bytes))
    );
    assert_eq!(
        document.admission.raw,
        atlas_foundry_model::parse_source_value(bytes).unwrap()
    );
    assert_eq!(document.provenance.source_path, "packs/items/ancestry.json");
    assert_eq!(document.provenance.pack_name, "items");
    assert_eq!(document.provenance.document_type, "Item");
    let FoundryDocumentSource::Item(item) = document.admission.model.as_ref().unwrap() else {
        panic!()
    };
    let ItemSourcePF2e::AncestrySource(ancestry) = item.as_ref() else {
        panic!()
    };
    assert!(matches!(ancestry._id, SourcePresence::Missing));
    assert!(matches!(ancestry.name, SourcePresence::Null));
    let feature = &ancestry
        .system
        .as_value()
        .unwrap()
        .items
        .as_value()
        .unwrap()
        .entries[0]
        .1;
    assert!(matches!(feature.level, SourcePresence::Invalid(_)));
    assert!(feature.level.as_value().is_none());
    let report = loaded.report();
    assert_eq!(report.counts.partial_documents, 1);
    assert_eq!(report.counts.diagnostics, 1);
    assert_eq!(
        report.diagnostics[0].context.record_key,
        "items:packs/items/ancestry.json"
    );
    assert_eq!(
        report.diagnostics[0].json_path,
        "$.system.items[\"feature\"].level"
    );
    assert!(!source.0.join("artifact.sqlite").exists());
}

#[test]
fn preserves_embedded_identities_order_and_spellcasting_links_despite_invalid_preparation() {
    let source = Source::new(json!([pack("actors", "Actor")]));
    source.write("packs/actors/caster.json", br#"{"type":"npc","_id":"actor","items":[{"type":"spellcastingEntry","_id":"entry","system":{"slots":{"slot1":{"prepared":["darkness (at will)",{"id":"spell"}]}}}},{"type":"spell","_id":"spell","system":{"location":{"value":"entry"}}}]}"#);
    let loaded = load_foundry_documents(&source.0, None).unwrap();
    let FoundryDocumentSource::Actor(actor) = loaded.packs[0].documents[0]
        .admission
        .model
        .as_ref()
        .unwrap()
    else {
        panic!()
    };
    let ActorSourcePF2e::NPCSource(actor) = actor.as_ref() else {
        panic!()
    };
    let items = actor.items.as_value().unwrap();
    assert_eq!(items.len(), 2);
    let ItemSourcePF2e::SpellcastingEntrySource(entry) = &items[0] else {
        panic!()
    };
    let ItemSourcePF2e::SpellSource(spell) = &items[1] else {
        panic!()
    };
    assert_eq!(entry._id.as_value().unwrap(), "entry");
    assert_eq!(spell._id.as_value().unwrap(), "spell");
    assert_eq!(
        spell
            .system
            .as_value()
            .unwrap()
            .location
            .as_value()
            .unwrap()
            .value
            .as_value()
            .unwrap(),
        "entry"
    );
    let system = serde_json::to_value(entry.system.as_value().unwrap()).unwrap();
    assert_eq!(
        system["slots"]["value"]["slot1"]["value"]["prepared"]["invalid"]["values"][0]["Array"]
            .as_array()
            .unwrap()
            .len(),
        2
    );
    assert_eq!(
        loaded.report().diagnostics[0].json_path,
        "$.items[0].system.slots.slot1.prepared[0]"
    );
}

#[test]
fn unknown_or_ambiguous_roots_stay_raw_without_becoming_quarantine() {
    let source = Source::new(json!([
        pack("items", "Item"),
        pack("future", "FutureDocument")
    ]));
    for (file, bytes) in [
        ("unknown", r#"{"type":"future"}"#),
        ("ambiguous", r#"{"type":"action","type":"spell"}"#),
        ("missing", r#"{}"#),
    ] {
        source.write(&format!("packs/items/{file}.json"), bytes);
    }
    source.write("packs/future/doc.json", b"{}");
    let loaded = load_foundry_documents(&source.0, None).unwrap();
    let report = loaded.report();
    assert_eq!(report.counts.discovered_files, 4);
    assert_eq!(report.counts.retained_documents, 4);
    assert_eq!(report.counts.raw_only_documents, 4);
    assert_eq!(report.counts.modeled_documents, 0);
    assert_eq!(report.counts.quarantined_files, 0);
    assert_eq!(report.diagnostics.len(), 4);
    assert!(
        loaded
            .packs
            .iter()
            .flat_map(|p| &p.documents)
            .all(|d| d.admission.model.is_none())
    );
}

#[test]
fn quarantines_bad_envelopes_with_original_bytes_and_reports_missing_packs() {
    let source = Source::new(json!([pack("items", "Item"), pack("missing", "Item")]));
    let bad = [b"{".as_slice(), b"[]", b"null", b"{} true", b"\xff"];
    for (index, bytes) in bad.iter().enumerate() {
        source.write(&format!("packs/items/{index}.json"), bytes);
    }
    source.write("packs/items/valid.json", br#"{"type":"action"}"#);
    let loaded = load_foundry_documents(&source.0, None).unwrap();
    let report = loaded.report();
    assert_eq!(report.counts.discovered_files, 6);
    assert_eq!(report.counts.retained_documents, 1);
    assert_eq!(report.counts.quarantined_files, 5);
    assert_eq!(report.counts.unavailable_packs, 1);
    for (file, bytes) in loaded.packs[0].quarantined_files.iter().zip(bad) {
        assert_eq!(file.bytes.as_deref(), Some(bytes));
        assert_eq!(
            file.content_hash.as_ref().unwrap(),
            &format!("{:x}", Sha256::digest(bytes))
        );
        assert_eq!(file.failure.stage, SourceLoadFailureStage::Parse);
    }
    assert_eq!(report.failures.len(), 6);
    assert_eq!(
        loaded.packs[1].discovery_failure.as_ref().unwrap().stage,
        SourceLoadFailureStage::Discovery
    );
}

#[test]
fn shares_manifest_resolution_namespaced_pack_discovery_and_folder_exclusion() {
    let source = Source::new(json!([pack("items", "Item")]));
    let alternate = source.0.join("alternate manifest.json");
    fs::rename(source.0.join("module.json"), &alternate).unwrap();
    source.write("packs/pf2e/items/nested/z.json", br#"{"type":"action"}"#);
    source.write("packs/pf2e/items/a.json", br#"{"type":"spell"}"#);
    source.write("packs/pf2e/items/_folders.json", b"[]");
    source.write("packs/pf2e/items/readme.txt", b"not JSON");
    assert!(load_foundry_documents(&source.0, None).is_err());
    let loaded = load_foundry_documents(&source.0, Some(&alternate)).unwrap();
    assert_eq!(loaded.manifest_path, alternate);
    assert_eq!(
        loaded.manifest_content_hash,
        format!("{:x}", Sha256::digest(fs::read(&alternate).unwrap()))
    );
    assert_eq!(
        loaded.packs[0].resolved_path,
        source.0.join("packs/pf2e/items")
    );
    assert_eq!(
        loaded.packs[0]
            .documents
            .iter()
            .map(|d| d.provenance.source_path.as_str())
            .collect::<Vec<_>>(),
        vec!["packs/pf2e/items/a.json", "packs/pf2e/items/nested/z.json"]
    );
    assert_eq!(
        loaded.report(),
        load_foundry_documents(&source.0, Some(&alternate))
            .unwrap()
            .report()
    );
}

#[test]
fn unavailable_roots_and_malformed_manifests_are_errors() {
    let source = Source::new(json!([]));
    assert!(load_foundry_documents(source.0.join("nonexistent"), None).is_err());
    assert!(
        load_foundry_documents(&source.0, None)
            .unwrap()
            .packs
            .is_empty()
    );
    source.write("module.json", b"{");
    assert!(load_foundry_documents(&source.0, None).is_err());
}

#[cfg(unix)]
#[test]
fn read_failures_are_explicit_without_claiming_retained_bytes() {
    let source = Source::new(json!([pack("items", "Item")]));
    fs::create_dir_all(source.0.join("packs/items")).unwrap();
    std::os::unix::fs::symlink(
        "nonexistent-target",
        source.0.join("packs/items/broken.json"),
    )
    .unwrap();
    let loaded = load_foundry_documents(&source.0, None).unwrap();
    let file = &loaded.packs[0].quarantined_files[0];
    assert_eq!(file.failure.stage, SourceLoadFailureStage::Read);
    assert!(file.bytes.is_none());
    assert!(file.content_hash.is_none());
    assert_eq!(loaded.report().counts.quarantined_files, 1);
}
