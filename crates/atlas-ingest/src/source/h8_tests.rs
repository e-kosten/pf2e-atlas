use std::path::{Path, PathBuf};

use atlas_domain::{PackName, RecordKind};
use atlas_record::{
    ContentChildIdentity, ContentOrigin, FactValue, H8FieldValue, JournalPageEntry, RecordBody,
    RichDocument, RichLinkTarget, RichNode, TableResultEntry, iter_foundry_links,
    render_plain_text,
};

use super::ManifestPack;
use super::dto::{
    SerializedSourceMember, SourcePresence, parse_journal_source, parse_roll_table_source,
    parse_serialized_source_object,
};
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
fn h8_journal_container_facts_preserve_four_states_and_duplicate_evidence() {
    let source = br#"{
      "_id":"journal1","name":"Journal","pages":[
        {"_id":"missing","name":"Missing","type":"text"},
        {"_id":"null","name":"Null","type":"text","image":null,"system":null},
        {"_id":"empty","name":"Empty","type":"text","image":{},"system":{}},
        {"_id":"populated","name":"Populated","type":"text",
         "image":{"caption":"Authored"},"system":{"future":true}},
        {"_id":"wrong","name":"Wrong shape","type":"text","image":false,"system":0},
        {"_id":"duplicate-image","name":"Duplicate image","type":"text",
         "image":{},"image":{"future":1}},
        {"_id":"duplicate-system","name":"Duplicate system","type":"text",
         "system":{},"system":{"future":2}}
      ]
    }"#;
    let root = parse_serialized_source_object(source).expect("lossless source tree");
    let dto = parse_journal_source(&root).expect("journal source DTO");
    let SourcePresence::Value(dto_pages) = dto.pages else {
        panic!("journal DTO pages");
    };
    assert!(matches!(
        dto_pages[0].object().expect("missing page").member("image"),
        SerializedSourceMember::Missing
    ));
    assert!(matches!(
        dto_pages[1].object().expect("null page").member("image"),
        SerializedSourceMember::Null
    ));
    let SerializedSourceMember::Value(empty_image) =
        dto_pages[2].object().expect("empty page").member("image")
    else {
        panic!("empty DTO image object");
    };
    assert_eq!(empty_image.compact_json(), "{}");
    let SerializedSourceMember::Duplicate(duplicate_images) = dto_pages[5]
        .object()
        .expect("duplicate image page")
        .member("image")
    else {
        panic!("duplicate DTO image members");
    };
    assert_eq!(
        duplicate_images
            .iter()
            .map(|value| value.compact_json())
            .collect::<Vec<_>>(),
        vec!["{}".to_string(), r#"{"future":1}"#.to_string()]
    );

    let loaded = normalize(
        "JournalEntry",
        "journals",
        "packs/journals/container-states.json",
        source,
    )
    .expect("container source states normalize without losing valid siblings");

    let Some(RecordBody::Journal(journal)) = loaded.facts.canonical_body.as_ref() else {
        panic!("journal body");
    };
    let FactValue::Value(H8FieldValue::Known(pages)) = &journal.pages else {
        panic!("pages");
    };
    assert_eq!(pages.len(), 7);

    let JournalPageEntry::Page(missing) = &pages[0] else {
        panic!("missing page");
    };
    assert_eq!(missing.image_source, FactValue::Missing);
    assert_eq!(missing.source_system, FactValue::Missing);

    let JournalPageEntry::Page(null) = &pages[1] else {
        panic!("null page");
    };
    assert_eq!(null.image_source, FactValue::Null);
    assert_eq!(null.source_system, FactValue::Null);

    let JournalPageEntry::Page(empty) = &pages[2] else {
        panic!("empty page");
    };
    let FactValue::Value(H8FieldValue::Known(empty_image)) = &empty.image_source else {
        panic!("empty image must be known");
    };
    assert_eq!(empty_image.compact_json, "{}");
    let FactValue::Value(H8FieldValue::Known(empty_system)) = &empty.source_system else {
        panic!("empty system must be known");
    };
    assert_eq!(empty_system.compact_json, "{}");

    let JournalPageEntry::Page(populated) = &pages[3] else {
        panic!("populated page");
    };
    let FactValue::Value(H8FieldValue::Known(populated_image)) = &populated.image_source else {
        panic!("populated image object must remain exact");
    };
    assert_eq!(populated_image.compact_json, r#"{"caption":"Authored"}"#);
    let FactValue::Value(H8FieldValue::Unsupported(populated_system)) = &populated.source_system
    else {
        panic!("populated system object must be exact unsupported evidence");
    };
    assert_eq!(populated_system.value, r#"{"future":true}"#);

    let JournalPageEntry::Page(wrong) = &pages[4] else {
        panic!("wrong-shape page");
    };
    let FactValue::Value(H8FieldValue::Unsupported(wrong_image)) = &wrong.image_source else {
        panic!("wrong-shape image must be exact unsupported evidence");
    };
    assert_eq!(wrong_image.value, "false");
    let FactValue::Value(H8FieldValue::Unsupported(wrong_system)) = &wrong.source_system else {
        panic!("wrong-shape system must be exact unsupported evidence");
    };
    assert_eq!(wrong_system.value, "0");

    let JournalPageEntry::Unsupported(duplicate_image) = &pages[5] else {
        panic!("duplicate image localizes to its page");
    };
    assert!(
        duplicate_image
            .exact_source
            .compact_json
            .contains(r#""image":{},"image":{"future":1}"#)
    );
    let JournalPageEntry::Unsupported(duplicate_system) = &pages[6] else {
        panic!("duplicate system localizes to its page");
    };
    assert!(
        duplicate_system
            .exact_source
            .compact_json
            .contains(r#""system":{},"system":{"future":2}"#)
    );
}

#[test]
fn h8_parent_child_containers_preserve_presence_and_reject_malformed_shapes() {
    for (document_type, pack, container, body_kind) in [
        ("JournalEntry", "journals", "pages", RecordKind::Journal),
        (
            "RollTable",
            "rollable-tables",
            "results",
            RecordKind::RollTable,
        ),
    ] {
        let prefix = if document_type == "JournalEntry" {
            r#"{"_id":"record1","name":"Record""#
        } else {
            r#"{"_id":"record1","name":"Record","formula":"1d1""#
        };
        for (case, suffix) in [
            ("missing", "}".to_string()),
            ("null", format!(r#","{container}":null}}"#)),
            ("empty", format!(r#","{container}":[]}}"#)),
        ] {
            let source = format!("{prefix}{suffix}");
            let root = parse_serialized_source_object(source.as_bytes())
                .expect("lossless parent container source tree");
            let dto_presence = if document_type == "JournalEntry" {
                parse_journal_source(&root)
                    .expect("journal source DTO")
                    .pages
            } else {
                parse_roll_table_source(&root)
                    .expect("roll table source DTO")
                    .results
            };
            match (case, &dto_presence) {
                ("missing", SourcePresence::Missing) | ("null", SourcePresence::Null) => {}
                ("empty", SourcePresence::Value(values)) if values.is_empty() => {}
                _ => panic!("unexpected {document_type} {case} DTO presence"),
            }
            let loaded = normalize(
                document_type,
                pack,
                &format!("packs/{pack}/{case}.json"),
                source.as_bytes(),
            )
            .expect("container source presence normalizes");
            assert_eq!(loaded.record.classification.kind, body_kind);
            match (
                case,
                loaded
                    .facts
                    .canonical_body
                    .as_ref()
                    .expect("canonical body"),
            ) {
                ("missing", RecordBody::Journal(value)) => {
                    assert_eq!(value.pages, FactValue::Missing)
                }
                ("null", RecordBody::Journal(value)) => {
                    assert_eq!(value.pages, FactValue::Null)
                }
                ("empty", RecordBody::Journal(value)) => {
                    assert_eq!(
                        value.pages,
                        FactValue::Value(H8FieldValue::Known(Vec::new()))
                    )
                }
                ("missing", RecordBody::RollTable(value)) => {
                    assert_eq!(value.results, FactValue::Missing)
                }
                ("null", RecordBody::RollTable(value)) => {
                    assert_eq!(value.results, FactValue::Null)
                }
                ("empty", RecordBody::RollTable(value)) => {
                    assert_eq!(
                        value.results,
                        FactValue::Value(H8FieldValue::Known(Vec::new()))
                    )
                }
                _ => panic!("unexpected H8 container body"),
            }
        }

        for (case, suffix, expected) in [
            (
                "wrong-shape",
                format!(r#","{container}":{{}}}}"#),
                format!("$.{container} must be an array"),
            ),
            (
                "duplicate",
                format!(r#","{container}":[],"{container}":[]}}"#),
                format!("$.{container} is duplicated"),
            ),
        ] {
            let source = format!("{prefix}{suffix}");
            let root = parse_serialized_source_object(source.as_bytes())
                .expect("lossless malformed parent container source tree");
            let dto_error = if document_type == "JournalEntry" {
                parse_journal_source(&root)
                    .expect_err("journal DTO must reject malformed container")
            } else {
                parse_roll_table_source(&root)
                    .expect_err("roll table DTO must reject malformed container")
            };
            assert!(
                dto_error.contains(&expected),
                "unexpected {document_type} {case} DTO error: {dto_error}"
            );
            let error = normalize(
                document_type,
                pack,
                &format!("packs/{pack}/{case}.json"),
                source.as_bytes(),
            )
            .expect_err("malformed parent container must fail closed");
            assert!(
                error.to_string().contains(&expected),
                "unexpected {document_type} {case} error: {error}"
            );
        }
    }
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

#[test]
fn h8_reference_resolution_copies_authoritative_documents_to_all_known_fields() {
    let journal = normalize(
        "JournalEntry",
        "journals",
        "packs/journals/reference-contract.json",
        br#"{
          "_id":"journal1","name":"Journal","pages":[
            {"_id":"target-page","name":"Target Page","type":"text",
             "text":{"content":"<p>Target.</p>","format":1}},
            {"_id":"source-page","name":"Source Page","type":"text",
             "text":{"content":"<p>@UUID[Compendium.pf2e.journals.JournalEntry.journal1.JournalEntryPage.target-page]{Target Page}</p>","format":1}}
          ]
        }"#,
    )
    .expect("journal source normalizes");
    let table = normalize(
        "RollTable",
        "rollable-tables",
        "packs/rollable-tables/reference-contract.json",
        br#"{
          "_id":"table1","name":"Table",
          "description":"<p>@UUID[Compendium.pf2e.journals.JournalEntry.journal1.JournalEntryPage.target-page]{Target Page}</p>",
          "formula":"1d1","replacement":false,"displayRoll":true,
          "results":[
            {"_id":"result-1","type":"text",
             "text":"<p>@UUID[Compendium.pf2e.journals.JournalEntry.journal1.JournalEntryPage.target-page]{Target Page}</p>",
             "weight":1,"range":[1,1],"drawn":false}
          ]
        }"#,
    )
    .expect("roll-table source normalizes");
    let mut records = vec![journal, table];
    let Some(RecordBody::Journal(journal)) = records[0].facts.canonical_body.as_mut() else {
        panic!("journal body");
    };
    let mut wrong_page_document = journal.content.documents[1].clone();
    let wrong_page_locator = match &wrong_page_document.origin {
        ContentOrigin::ChildField { locator, .. } => locator.clone(),
        _ => panic!("page content origin"),
    };
    wrong_page_document.origin = ContentOrigin::ChildField {
        locator: wrong_page_locator,
        relative_source_path: "$.pages[99].text.content".to_string(),
    };
    wrong_page_document.document = RichDocument::new(vec![RichNode::Text {
        text: "Wrong page document".to_string(),
    }]);
    wrong_page_document.refresh_derived_state();
    journal.content.documents.insert(0, wrong_page_document);

    let Some(RecordBody::RollTable(table)) = records[1].facts.canonical_body.as_mut() else {
        panic!("roll-table body");
    };
    let mut wrong_description = table.content.documents[0].clone();
    wrong_description.origin = ContentOrigin::RecordField {
        source_kind: wrong_description.source_kind,
        relative_source_path: "$.wrong-description".to_string(),
    };
    wrong_description.document = RichDocument::new(vec![RichNode::Text {
        text: "Wrong table description".to_string(),
    }]);
    wrong_description.refresh_derived_state();
    let mut wrong_result = table.content.documents[1].clone();
    let wrong_result_locator = match &wrong_result.origin {
        ContentOrigin::ChildField { locator, .. } => locator.clone(),
        _ => panic!("result content origin"),
    };
    wrong_result.origin = ContentOrigin::ChildField {
        locator: wrong_result_locator,
        relative_source_path: "$.results[99].text".to_string(),
    };
    wrong_result.document = RichDocument::new(vec![RichNode::Text {
        text: "Wrong table result".to_string(),
    }]);
    wrong_result.refresh_derived_state();
    table.content.documents.insert(0, wrong_result);
    table.content.documents.insert(0, wrong_description);

    let index = crate::records::references::build_record_reference_index(&records);
    crate::records::references::resolve_content_references(&mut records, &index);

    let target = |document: &atlas_record::RichDocument| {
        let link = iter_foundry_links(document)
            .next()
            .expect("resolved document keeps its authored link");
        let RichLinkTarget::RecordChild { key, locator, .. } = &link.target else {
            panic!("known H8 field must receive the resolved child target");
        };
        assert_eq!(key.to_string(), "journals:journal1");
        assert_eq!(locator.parent, *key);
        assert!(matches!(
            &locator.identity,
            ContentChildIdentity::Stable(id) if id.as_str() == "target-page"
        ));
    };

    let Some(RecordBody::Journal(journal)) = records[0].facts.canonical_body.as_ref() else {
        panic!("journal body");
    };
    let FactValue::Value(H8FieldValue::Known(pages)) = &journal.pages else {
        panic!("journal pages");
    };
    let page = pages
        .iter()
        .filter_map(|entry| match entry {
            JournalPageEntry::Page(page) => Some(page),
            JournalPageEntry::Unsupported(_) => None,
        })
        .find(|page| {
            matches!(
                &page.source_id,
                FactValue::Value(H8FieldValue::Known(id)) if id.as_str() == "source-page"
            )
        })
        .expect("source page");
    let FactValue::Value(H8FieldValue::Known(text)) = &page.text else {
        panic!("page text");
    };
    let FactValue::Value(H8FieldValue::Known(document)) = &text.content else {
        panic!("page content");
    };
    target(document);

    let Some(RecordBody::RollTable(table)) = records[1].facts.canonical_body.as_ref() else {
        panic!("roll-table body");
    };
    let FactValue::Value(H8FieldValue::Known(description)) = &table.description else {
        panic!("table description");
    };
    target(description);
    let FactValue::Value(H8FieldValue::Known(results)) = &table.results else {
        panic!("table results");
    };
    let TableResultEntry::Result(result) = &results[0] else {
        panic!("table result");
    };
    let FactValue::Value(H8FieldValue::Known(text)) = &result.text else {
        panic!("result text");
    };
    target(text);
    for retained in [
        "Wrong page document",
        "Wrong table description",
        "Wrong table result",
    ] {
        assert!(records.iter().any(|record| {
            record
                .facts
                .canonical_body
                .as_ref()
                .and_then(|body| match body {
                    RecordBody::Journal(journal) => Some(&journal.content),
                    RecordBody::RollTable(table) => Some(&table.content),
                    _ => None,
                })
                .is_some_and(|content| {
                    content
                        .documents
                        .iter()
                        .any(|document| render_plain_text(&document.document) == retained)
                })
        }));
    }
}
