use std::path::Path;

use atlas_domain::PackName;
use atlas_record::{ContentSourceKind, render_plain_text};
use serde_json::json;

use super::normalize_record;
use crate::records::JournalPageSkipReason;
use crate::source::ManifestPack;

#[test]
fn normalizes_source_facts_embedded_content_refs_and_journal_pages() {
    let raw = json!({
        "_id": "host1",
        "name": "Host Record",
        "type": "npc",
        "_stats": { "compendiumSource": "Compendium.pf2e.bestiary.Actor.host1" },
        "items": [
            {
                "_id": "bite1",
                "name": "Ghoul Fever",
                "type": "action",
                "_stats": { "compendiumSource": "Compendium.pf2e.afflictions.Item.ghoul-fever" },
                "system": {
                    "slug": "ghoul-fever",
                    "category": "offensive",
                    "publication": { "remaster": false },
                    "traits": { "value": ["disease"] },
                    "description": {
                        "value": "<p><strong>Saving Throw</strong> Fortitude</p><p><strong>Stage 1</strong> Sickened</p>"
                    }
                }
            },
            {
                "_id": "spell1",
                "name": "Staged Spell",
                "type": "spell",
                "system": {
                    "traits": { "value": ["curse"] },
                    "spell": {
                        "system": {
                            "description": { "value": "<p>Nested spell text.</p>" }
                        }
                    }
                }
            }
        ],
        "pages": [
            {
                "_id": "page1",
                "name": "Rules",
                "text": { "content": "<h2>Rules</h2><p>Page body.</p>" }
            },
            {
                "_id": "page2",
                "name": "Image",
                "type": "image"
            },
            {
                "_id": "page3",
                "name": "Empty",
                "text": { "content": "   " }
            },
            {
                "_id": "page4",
                "name": "Unknown Empty",
                "text": { "content": "<unknown></unknown>" }
            }
        ],
        "system": {
            "slug": "host-record",
            "details": {
                "level": { "value": 1 },
                "publication": { "title": "Fixture" }
            },
            "traits": { "rarity": "common", "value": ["undead"] }
        }
    });
    let loaded = normalize_record(
        &manifest_pack("Actor"),
        &PackName::new("bestiary".to_string()).expect("pack name"),
        Path::new("packs/bestiary/host.json"),
        Path::new("."),
        raw,
    )
    .expect("record normalizes");

    let facts = &loaded.facts.source_facts;
    assert_eq!(facts.slug.as_deref(), Some("host-record"));
    assert_eq!(
        facts.compendium_source.as_deref(),
        Some("Compendium.pf2e.bestiary.Actor.host1")
    );
    assert_eq!(facts.embedded_items.len(), 2);

    let affliction = &facts.embedded_items[0];
    assert_eq!(affliction.item_id, "bite1");
    assert_eq!(affliction.normalized_name, "ghoul fever");
    assert_eq!(affliction.foundry_item_type, "action");
    assert_eq!(affliction.system_category.as_deref(), Some("offensive"));
    assert_eq!(affliction.traits, vec!["disease"]);
    assert_eq!(affliction.slug.as_deref(), Some("ghoul-fever"));
    assert!(affliction.raw_provenance.is_some());
    assert_eq!(affliction.content_refs.len(), 1);
    assert_eq!(
        affliction.content_refs[0].source_kind,
        ContentSourceKind::EmbeddedItemDescription
    );
    assert_eq!(
        affliction.content_refs[0].local_key,
        "#item:bite1:description"
    );
    let affliction_content = facts
        .source_content
        .get("#item:bite1:description")
        .expect("embedded description is source content");
    assert_eq!(
        render_plain_text(&affliction_content.document),
        "Saving Throw Fortitude\nStage 1 Sickened"
    );

    let spell = &facts.embedded_items[1];
    assert_eq!(
        spell.content_refs[0].local_key,
        "#item:spell1:spell-description"
    );
    assert_eq!(
        facts
            .source_content
            .get("#item:spell1:spell-description")
            .map(|content| render_plain_text(&content.document))
            .as_deref(),
        Some("Nested spell text.")
    );

    assert_eq!(facts.journal_pages.len(), 1);
    assert_eq!(facts.journal_pages[0].page_id.as_deref(), Some("page1"));
    assert_eq!(facts.journal_pages[0].normalized_name, "rules");
    assert_eq!(facts.journal_pages[0].source_ref, "journal:Rules");
    assert_eq!(
        render_plain_text(&facts.journal_pages[0].document),
        "Rules\nPage body."
    );
    assert_eq!(facts.skipped_journal_pages.len(), 3);
    assert_eq!(
        facts.skipped_journal_pages[0].reason,
        JournalPageSkipReason::MissingTextContent
    );
    assert_eq!(
        facts.skipped_journal_pages[1].reason,
        JournalPageSkipReason::EmptyTextContent
    );
    assert_eq!(
        facts.skipped_journal_pages[2].reason,
        JournalPageSkipReason::EmptyParsedDocument
    );
}

fn manifest_pack(document_type: &str) -> ManifestPack {
    ManifestPack {
        name: "bestiary".to_string(),
        label: "Bestiary".to_string(),
        document_type: document_type.to_string(),
        path: "packs/bestiary".to_string(),
    }
}
