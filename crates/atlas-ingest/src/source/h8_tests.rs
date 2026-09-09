use std::path::{Path, PathBuf};

use atlas_domain::{PackName, RecordKind};
use atlas_record::{
    ContentChildIdentity, FactValue, H8FieldValue, JournalPageEntry, RecordBody, RichLinkTarget,
    TableResultEntry, iter_foundry_links, render_plain_text,
};

use super::ManifestPack;
use super::normalize::normalize_record_from_source_bytes;

fn normalize(
    document_type: &str,
    pack: &str,
    relative: &str,
    source: &[u8],
) -> Result<crate::records::LoadedSourceRecord, crate::error::IngestError> {
    normalize_record_from_source_bytes(
        &ManifestPack {
            name: pack.to_string(),
            label: pack.to_string(),
            document_type: document_type.to_string(),
            path: format!("packs/{pack}"),
        },
        &PackName::new(pack).expect("pack"),
        Path::new(relative),
        Path::new("."),
        source,
        None,
    )
}

#[test]
fn h8_journal_preserves_four_states_exact_objects_and_local_child_failure() {
    let loaded = normalize(
        "JournalEntry",
        "journals",
        "packs/journals/contract.json",
        br#"{
          "_id":"journal1","name":"Journal","folder":null,"sort":0,
          "ownership":{"member":1,"member":2},
          "pages":[
            {"_id":"page-a","name":"Alpha","type":"text","sort":0,
             "title":{"show":true,"level":2},
             "text":{"content":"<p>Alpha text.</p>","format":1},
             "image":{},"video":{"controls":true,"volume":0.5},"src":null,
             "system":{"member":false,"member":0}},
            {"_id":"page-b","name":"Broken image","type":"image",
             "src":"first.webp","src":"second.webp","image":{},"text":{},"video":{},"system":{}}
          ]
        }"#,
    )
    .expect("journal source normalizes");

    assert_eq!(loaded.record.classification.kind, RecordKind::Journal);
    let Some(RecordBody::Journal(journal)) = loaded.facts.canonical_body.as_ref() else {
        panic!("journal body");
    };
    assert_eq!(journal.source_metadata.folder, FactValue::Null);
    assert!(matches!(
        journal.source_metadata.sort,
        FactValue::Value(H8FieldValue::Known(0))
    ));
    let FactValue::Value(H8FieldValue::Known(ownership)) = &journal.source_metadata.ownership
    else {
        panic!("ownership exact object");
    };
    assert_eq!(ownership.compact_json, r#"{"member":1,"member":2}"#);

    let FactValue::Value(H8FieldValue::Known(pages)) = &journal.pages else {
        panic!("pages");
    };
    let JournalPageEntry::Page(page) = &pages[0] else {
        panic!("first page");
    };
    assert!(matches!(
        page.locator.identity,
        ContentChildIdentity::Stable(_)
    ));
    let FactValue::Value(H8FieldValue::Unsupported(system)) = &page.source_system else {
        panic!("populated page system object must remain conspicuously unsupported");
    };
    assert_eq!(
        system.value, r#"{"member":false,"member":0}"#,
        "duplicate descendants retain their corresponding values and order"
    );
    let FactValue::Value(H8FieldValue::Known(image)) = &page.image_source else {
        panic!("empty image container must remain known empty provenance");
    };
    assert_eq!(image.compact_json, "{}");
    let FactValue::Value(H8FieldValue::Unsupported(video)) = &page.video else {
        panic!("inactive populated video metadata must not become active video semantics");
    };
    assert_eq!(video.value, r#"{"controls":true,"volume":0.5}"#);
    let JournalPageEntry::Unsupported(unsupported) = &pages[1] else {
        panic!("duplicate child field localizes to one unsupported child");
    };
    assert!(
        unsupported
            .exact_source
            .compact_json
            .contains(r#""src":"first.webp","src":"second.webp""#)
    );
    assert_eq!(pages.len(), 2, "valid siblings remain present");
    assert_eq!(journal.content.documents.len(), 1);
    assert_eq!(
        render_plain_text(&journal.content.documents[0].document),
        "Alpha text."
    );
}

#[test]
fn h8_journal_page_system_distinguishes_empty_known_from_populated_unsupported() {
    let loaded = normalize(
        "JournalEntry",
        "journals",
        "packs/journals/page-system.json",
        br#"{
          "_id":"journal1","name":"Journal","pages":[
            {"_id":"empty","name":"Empty","type":"text","system":{}},
            {"_id":"populated","name":"Populated","type":"text",
             "system":{"member":false,"member":0}}
          ]
        }"#,
    )
    .expect("journal page system facts normalize");

    let Some(RecordBody::Journal(journal)) = loaded.facts.canonical_body.as_ref() else {
        panic!("journal body");
    };
    let FactValue::Value(H8FieldValue::Known(pages)) = &journal.pages else {
        panic!("pages");
    };
    let JournalPageEntry::Page(empty) = &pages[0] else {
        panic!("empty-system page");
    };
    let FactValue::Value(H8FieldValue::Known(empty_system)) = &empty.source_system else {
        panic!("empty object must remain known empty");
    };
    assert_eq!(empty_system.compact_json, "{}");

    let JournalPageEntry::Page(populated) = &pages[1] else {
        panic!("populated-system page");
    };
    let FactValue::Value(H8FieldValue::Unsupported(populated_system)) = &populated.source_system
    else {
        panic!("populated object must be exact unsupported evidence");
    };
    assert_eq!(populated_system.value, r#"{"member":false,"member":0}"#);
}

#[test]
fn h8_journal_localizes_unknown_typed_object_members_with_exact_duplicate_evidence() {
    let loaded = normalize(
        "JournalEntry",
        "journals",
        "packs/journals/nested-unknown.json",
        br#"{
          "_id":"journal1","name":"Journal","pages":[
            {"_id":"page-title","name":"Unknown title","type":"text",
             "title":{"show":true,"future":"authored"},
             "text":{"content":"<p>Not projected.</p>"}},
            {"_id":"page-text","name":"Unknown text","type":"text",
             "text":{"content":"<p>Not projected.</p>",
                     "future":{"value":1,"value":2}}},
            {"_id":"page-valid","name":"Valid","type":"text",
             "text":{"content":"<p>Valid sibling.</p>"}},
            {"_id":"page-no-name","type":"text",
             "text":{"content":"<p>Must not receive a fabricated title.</p>"}}
          ]
        }"#,
    )
    .expect("unknown nested members localize to their owning children");

    let Some(RecordBody::Journal(journal)) = loaded.facts.canonical_body.as_ref() else {
        panic!("journal body");
    };
    let FactValue::Value(H8FieldValue::Known(pages)) = &journal.pages else {
        panic!("pages");
    };
    let JournalPageEntry::Unsupported(title) = &pages[0] else {
        panic!("unknown title member must make only that page unsupported");
    };
    assert!(
        title
            .exact_source
            .compact_json
            .contains(r#""future":"authored""#)
    );
    let JournalPageEntry::Unsupported(text) = &pages[1] else {
        panic!("unknown text member must make only that page unsupported");
    };
    assert!(
        text.exact_source
            .compact_json
            .contains(r#""future":{"value":1,"value":2}"#),
        "ordered duplicate descendants must remain exact"
    );
    assert!(matches!(pages[2], JournalPageEntry::Page(_)));
    let JournalPageEntry::Unsupported(missing_name) = &pages[3] else {
        panic!("a missing required page name must remain locally unavailable");
    };
    assert!(
        missing_name
            .exact_source
            .compact_json
            .contains(r#""_id":"page-no-name""#)
    );
    assert_eq!(journal.content.documents.len(), 1);
    assert_eq!(
        render_plain_text(&journal.content.documents[0].document),
        "Valid sibling."
    );
}

#[test]
fn h8_journal_retains_unknown_text_format_as_exact_unsupported() {
    let loaded = normalize(
        "JournalEntry",
        "journals",
        "packs/journals/text-format.json",
        br#"{"_id":"journal1","name":"Journal","pages":[
          {"_id":"page-a","name":"Alpha","type":"text",
           "text":{"content":"<p>Alpha.</p>","format":3}}
        ]}"#,
    )
    .expect("unknown text format remains a typed unsupported fact");
    let Some(RecordBody::Journal(journal)) = loaded.facts.canonical_body.as_ref() else {
        panic!("journal body");
    };
    let FactValue::Value(H8FieldValue::Known(pages)) = &journal.pages else {
        panic!("pages");
    };
    let JournalPageEntry::Page(page) = &pages[0] else {
        panic!("page remains addressable");
    };
    let FactValue::Value(H8FieldValue::Known(text)) = &page.text else {
        panic!("text owner");
    };
    let FactValue::Value(H8FieldValue::Unsupported(format)) = &text.format else {
        panic!("unknown format must be unsupported");
    };
    assert_eq!(format.value, "3");
    assert_eq!(journal.content.documents.len(), 1);
}

#[test]
fn h8_roll_table_localizes_duplicate_result_image_and_never_selects_a_value() {
    let loaded = normalize(
        "RollTable",
        "rollable-tables",
        "packs/rollable-tables/contract.json",
        br#"{
          "_id":"table1","name":"Table","description":"<p>Table description.</p>",
          "formula":"1d2","replacement":false,"displayRoll":true,"img":null,
          "results":[
            {"_id":"result-a","type":"text","text":"<p>Alpha.</p>","documentId":null,
             "weight":1,"range":[1,1],"drawn":false,"img":null},
            {"_id":"result-b","type":"text","text":"<p>Broken.</p>","documentId":null,
             "weight":1,"range":[2,2],"drawn":false,"img":"a.webp","img":"b.webp"},
            {"_id":"result-c","type":"text","text":"<p>Omega.</p>","documentId":null,
             "weight":1,"range":[3,3],"drawn":false,"img":null}
          ]
        }"#,
    )
    .expect("roll table source normalizes");

    let Some(RecordBody::RollTable(table)) = loaded.facts.canonical_body.as_ref() else {
        panic!("roll-table body");
    };
    let FactValue::Value(H8FieldValue::Known(results)) = &table.results else {
        panic!("results");
    };
    assert!(matches!(results[0], TableResultEntry::Result(_)));
    let TableResultEntry::Unsupported(unsupported) = &results[1] else {
        panic!("duplicate image result must be localized");
    };
    assert!(matches!(
        unsupported.locator.identity,
        ContentChildIdentity::Stable(_)
    ));
    assert!(
        unsupported
            .exact_source
            .compact_json
            .contains(r#""img":"a.webp","img":"b.webp""#)
    );
    assert!(matches!(results[2], TableResultEntry::Result(_)));
    assert_eq!(
        table.content.documents.len(),
        3,
        "description and both valid result documents remain"
    );
}

#[test]
fn h8_duplicate_parent_image_and_duplicate_dispatch_fail_closed() {
    for (document_type, pack, source, expected) in [
        (
            "RollTable",
            "rollable-tables",
            br#"{"_id":"table1","name":"Table","img":"a","img":"b","results":[]}"#.as_slice(),
            "$.img is duplicated",
        ),
        (
            "JournalEntry",
            "journals",
            br#"{"_id":"journal1","name":"Journal","pages":[{"_id":"p","type":"text","type":"image"}]}"#.as_slice(),
            "duplicate journal page dispatch",
        ),
    ] {
        let error = normalize(document_type, pack, "packs/fixture.json", source)
            .expect_err("critical duplicate must reject its parent");
        assert!(error.to_string().contains(expected), "{error}");
    }
}

#[test]
fn h8_duplicate_and_fallback_like_child_ids_never_collide() {
    let loaded = normalize(
        "JournalEntry",
        "journals",
        "packs/journals/identity.json",
        br#"{"_id":"journal1","name":"Journal","pages":[
          {"_id":"fallback-1","name":"Stable","type":"text","text":{"content":"stable"}},
          {"_id":"duplicate","name":"First","type":"text","text":{"content":"first"}},
          {"_id":"duplicate","name":"Second","type":"text","text":{"content":"second"}},
          {"name":"Missing","type":"text","text":{"content":"missing"}}
        ]}"#,
    )
    .expect("identity fixture");
    let Some(RecordBody::Journal(journal)) = loaded.facts.canonical_body.as_ref() else {
        panic!("journal body");
    };
    let FactValue::Value(H8FieldValue::Known(pages)) = &journal.pages else {
        panic!("pages");
    };
    let identities = pages
        .iter()
        .map(|entry| match entry {
            JournalPageEntry::Page(page) => &page.locator.identity,
            JournalPageEntry::Unsupported(page) => &page.locator.identity,
        })
        .collect::<Vec<_>>();
    assert!(
        matches!(identities[0], ContentChildIdentity::Stable(id) if id.as_str() == "fallback-1")
    );
    assert!(matches!(
        identities[1],
        ContentChildIdentity::Unstable { source_ordinal: 1 }
    ));
    assert!(matches!(
        identities[2],
        ContentChildIdentity::Unstable { source_ordinal: 2 }
    ));
    assert!(matches!(
        identities[3],
        ContentChildIdentity::Unstable { source_ordinal: 3 }
    ));
}

#[test]
fn h8_zero_observed_variants_retain_metadata_without_media_or_roll_execution() {
    let loaded = normalize(
        "JournalEntry",
        "journals",
        "packs/journals/zero-variants.json",
        br#"{"_id":"journal1","name":"Journal","pages":[
          {"_id":"image-1","name":"Image","type":"image","src":"image.webp",
           "image":{"caption":"Caption"},"text":{},"video":{},"system":{}},
          {"_id":"pdf-1","name":"PDF","type":"pdf","src":"document.pdf",
           "image":{},"text":{},"video":{},"system":{}},
          {"_id":"video-0","name":"Video zero","type":"video","src":"video.webm",
           "image":{},"text":{},"video":{"volume":0,"controls":false},"system":{}},
          {"_id":"video-1","name":"Video one","type":"video","src":"video.webm",
           "image":{},"text":{},"video":{"volume":1,"controls":true},"system":{}},
          {"_id":"video-low","name":"Video low","type":"video","src":"video.webm",
           "image":{},"text":{},"video":{"volume":-0.1},"system":{}},
          {"_id":"video-high","name":"Video high","type":"video","src":"video.webm",
           "image":{},"text":{},"video":{"volume":1.1},"system":{}}
        ]}"#,
    )
    .expect("declared journal variants normalize");
    let Some(RecordBody::Journal(journal)) = loaded.facts.canonical_body.as_ref() else {
        panic!("journal body");
    };
    let FactValue::Value(H8FieldValue::Known(pages)) = &journal.pages else {
        panic!("journal pages");
    };
    for (index, expected_kind, expected_source) in [
        (0, atlas_record::JournalPageKind::Image, "image.webp"),
        (1, atlas_record::JournalPageKind::Pdf, "document.pdf"),
        (2, atlas_record::JournalPageKind::Video, "video.webm"),
    ] {
        let JournalPageEntry::Page(page) = &pages[index] else {
            panic!("variant page");
        };
        assert_eq!(
            page.page_kind,
            FactValue::Value(H8FieldValue::Known(expected_kind))
        );
        assert!(matches!(
            &page.source,
            FactValue::Value(H8FieldValue::Known(source)) if source.as_str() == expected_source
        ));
    }
    for (index, expected) in [(2, "0"), (3, "1")] {
        let JournalPageEntry::Page(page) = &pages[index] else {
            panic!("video page");
        };
        let FactValue::Value(H8FieldValue::Known(video)) = &page.video else {
            panic!("video metadata");
        };
        assert!(matches!(
            &video.volume,
            FactValue::Value(H8FieldValue::Known(value)) if value.canonical == expected
        ));
    }
    for index in [4, 5] {
        let JournalPageEntry::Page(page) = &pages[index] else {
            panic!("video page");
        };
        let FactValue::Value(H8FieldValue::Known(video)) = &page.video else {
            panic!("video metadata");
        };
        assert!(matches!(
            video.volume,
            FactValue::Value(H8FieldValue::Unsupported(_))
        ));
    }

    let loaded = normalize(
        "RollTable",
        "rollable-tables",
        "packs/rollable-tables/document-variant.json",
        br#"{"_id":"table1","name":"Table","formula":"1d1","results":[
          {"_id":"result-1","type":"document","text":"Document target",
           "documentCollection":"JournalEntry","documentId":"journal-id",
           "weight":1,"range":[1,1],"drawn":false,"img":"result.webp"}
        ]}"#,
    )
    .expect("declared document result normalizes");
    let Some(RecordBody::RollTable(table)) = loaded.facts.canonical_body.as_ref() else {
        panic!("roll-table body");
    };
    let FactValue::Value(H8FieldValue::Known(results)) = &table.results else {
        panic!("table results");
    };
    let TableResultEntry::Result(result) = &results[0] else {
        panic!("document result");
    };
    assert_eq!(
        result.result_kind,
        FactValue::Value(H8FieldValue::Known(atlas_record::TableResultKind::Document))
    );
    assert!(matches!(
        &result.target.document_id,
        FactValue::Value(H8FieldValue::Known(id)) if id.as_str() == "journal-id"
    ));
}

#[test]
fn h8_stable_child_identity_survives_reorder_and_remains_parent_scoped() {
    let source = |reverse: bool| {
        if reverse {
            br#"{"_id":"journal1","name":"Journal","pages":[
              {"_id":"page-b","name":"B","type":"text","text":{"content":"B"}},
              {"_id":"page-a","name":"A","type":"text","text":{"content":"A"}}
            ]}"#
            .as_slice()
        } else {
            br#"{"_id":"journal1","name":"Journal","pages":[
              {"_id":"page-a","name":"A","type":"text","text":{"content":"A"}},
              {"_id":"page-b","name":"B","type":"text","text":{"content":"B"}}
            ]}"#
            .as_slice()
        }
    };
    let locators = |pack: &str, reverse: bool| {
        let loaded = normalize(
            "JournalEntry",
            pack,
            "packs/journals/reorder.json",
            source(reverse),
        )
        .expect("journal source");
        let Some(RecordBody::Journal(journal)) = loaded.facts.canonical_body.as_ref() else {
            panic!("journal body");
        };
        let FactValue::Value(H8FieldValue::Known(pages)) = &journal.pages else {
            panic!("pages");
        };
        pages
            .iter()
            .map(|entry| match entry {
                JournalPageEntry::Page(page) => (page.source_id.clone(), page.locator.clone()),
                JournalPageEntry::Unsupported(_) => panic!("supported page"),
            })
            .collect::<Vec<_>>()
    };
    let first = locators("journals", false);
    let reordered = locators("journals", true);
    for (source_id, locator) in &first {
        assert_eq!(
            reordered
                .iter()
                .find(|(candidate, _)| candidate == source_id)
                .map(|(_, locator)| locator),
            Some(locator),
            "stable locator must not depend on authored order"
        );
    }
    let other_parent = locators("other-journals", false);
    assert_ne!(first[0].1, other_parent[0].1);
}

#[test]
fn h8_real_loader_resolves_hero_point_result_to_exact_journal_page() {
    let Some(source_root) = std::env::var_os("PF2E_SOURCE_REPOSITORY") else {
        return;
    };
    let source_root = PathBuf::from(source_root);
    let fixture = std::env::temp_dir().join(format!(
        "atlas-h8-loader-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("clock")
            .as_nanos()
    ));
    let journals = fixture.join("packs/journals");
    let tables = fixture.join("packs/rollable-tables");
    std::fs::create_dir_all(&journals).expect("journal pack");
    std::fs::create_dir_all(&tables).expect("table pack");
    std::fs::copy(
        source_root.join("packs/journals/hero-point-deck.json"),
        journals.join("hero-point-deck.json"),
    )
    .expect("copy exact journal fixture");
    std::fs::copy(
        source_root.join("packs/rollable-tables/hero-point-deck.json"),
        tables.join("hero-point-deck.json"),
    )
    .expect("copy exact table fixture");
    std::fs::write(
        fixture.join("module.json"),
        br#"{"packs":[
          {"name":"journals","label":"Journals","type":"JournalEntry","path":"packs/journals"},
          {"name":"rollable-tables","label":"Roll Tables","type":"RollTable","path":"packs/rollable-tables"}
        ]}"#,
    )
    .expect("fixture manifest");

    let loaded = super::loader::load_foundry_source_records(&fixture, None)
        .expect("production loader accepts exact pinned fixtures");
    assert_eq!(loaded.records.len(), 2);
    let mut records = loaded.records;
    let index = crate::records::references::build_record_reference_index(&records);
    crate::records::references::resolve_content_references(&mut records, &index);
    let table = records
        .iter()
        .find(|record| record.record.identity.key.to_string() == "rollable-tables:zgZoI7h0XjjJrrNK")
        .expect("Hero Point table");
    let Some(RecordBody::RollTable(table_body)) = table.facts.canonical_body.as_ref() else {
        panic!("table body");
    };
    let link = table_body
        .content
        .documents
        .iter()
        .flat_map(|document| iter_foundry_links(&document.document))
        .find(|link| link.source.authored_target.contains("quxPxuMub8k6abzN"))
        .expect("Ancestral Might link");
    let RichLinkTarget::RecordChild { key, locator, .. } = &link.target else {
        panic!("link must resolve to a typed journal child");
    };
    assert_eq!(key.to_string(), "journals:BSp4LUSaOmUyjBko");
    assert_eq!(locator.parent, *key);
    assert!(matches!(
        &locator.identity,
        ContentChildIdentity::Stable(id) if id.as_str() == "quxPxuMub8k6abzN"
    ));
    let FactValue::Value(H8FieldValue::Known(results)) = &table_body.results else {
        panic!("table results");
    };
    let result_link = results
        .iter()
        .filter_map(|entry| match entry {
            TableResultEntry::Result(result) => Some(result),
            TableResultEntry::Unsupported(_) => None,
        })
        .filter_map(|result| match &result.text {
            FactValue::Value(H8FieldValue::Known(document)) => Some(document),
            _ => None,
        })
        .flat_map(iter_foundry_links)
        .find(|link| link.source.authored_target.contains("quxPxuMub8k6abzN"))
        .expect("typed table result keeps the resolved Ancestral Might link");
    assert!(matches!(
        &result_link.target,
        RichLinkTarget::RecordChild {
            key: result_key,
            locator: result_locator,
            ..
        } if result_key == key && result_locator == locator
    ));
    let edges = crate::records::references::resolve_reference_edges(&records);
    assert!(edges.iter().any(|edge| {
        edge.from_record_key == table.record.identity.key
            && edge.to_record_key == *key
            && edge.target_child.as_ref() == Some(locator)
            && edge.source_child.is_some()
    }));
    std::fs::remove_dir_all(fixture).expect("fixture cleanup");
}
