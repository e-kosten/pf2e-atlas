use std::collections::BTreeMap;

use atlas_domain::{PackName, PublicationCategory, Rarity, RecordId, RecordKey, RecordKind};

use crate::{
    ActivationTimeSourceField, AtlasRecord, ContentSourceKind, FoundryDocumentMechanics,
    FoundryDocumentType, FoundryRecordInfo, FoundryRecordType, ItemMechanics, NormalizedTime,
    PresentationBlock, PresentationSection, PresentationSectionKind, RecordActivationTiming,
    RecordClassification, RecordContent, RecordContentDocument, RecordIdentity, RecordMechanics,
    RecordProvenance, RecordPublication, RecordRequirements, RecordTaxonomy, RecordTiming,
    RecordVisibility, RichDocument, RichNode, build_record_presentation_document,
};

fn base_record(kind: RecordKind) -> AtlasRecord {
    AtlasRecord {
        identity: RecordIdentity {
            key: RecordKey::new(
                PackName::new("test-pack").expect("pack should parse"),
                RecordId::new("TestRecord").expect("id should parse"),
            ),
            name: "Test Record".to_string(),
        },
        classification: RecordClassification {
            kind,
            level: Some(3),
            rarity: Some(Rarity::Uncommon),
            traits: vec!["healing".to_string(), "vitality".to_string()],
            taxonomy: RecordTaxonomy {
                inferred_groups: vec!["support".to_string()],
            },
        },
        foundry: FoundryRecordInfo {
            pack_label: "Test Pack".to_string(),
            document_type: FoundryDocumentType::Item,
            record_type: FoundryRecordType::from_foundry(kind.as_str()),
            folder_id: None,
        },
        provenance: RecordProvenance {
            source_path: "packs/test-pack/TestRecord.json".to_string(),
            raw_json: Some("{}".to_string()),
        },
        publication: RecordPublication {
            title: Some("Player Core".to_string()),
            remaster: true,
            category: PublicationCategory::Core,
        },
        requirements: RecordRequirements::default(),
        timing: RecordTiming {
            activation: Some(RecordActivationTiming {
                time: NormalizedTime {
                    kind: atlas_domain::TimeKind::Actions,
                    actions: Some(2),
                    duration_value: None,
                    duration_unit: None,
                    text: "2".to_string(),
                },
                source_field: ActivationTimeSourceField::ActionsValue,
            }),
            duration: None,
        },
        mechanics: RecordMechanics::default(),
        content: RecordContent {
            documents: vec![RecordContentDocument {
                source_kind: ContentSourceKind::Description,
                label: None,
                document: text_document("Restores vitality to a wounded ally."),
            }],
        },
        variant: None,
        visibility: RecordVisibility::default(),
    }
}

#[test]
fn spell_recipe_has_no_generic_mechanics_or_content_fallback() {
    let mut record = base_record(RecordKind::Spell);
    record.mechanics.document = FoundryDocumentMechanics::Item(ItemMechanics {
        category: Some("legacy-spell-category".to_string()),
        ..ItemMechanics::default()
    });

    let document = build_record_presentation_document(&record);

    assert_eq!(document.kind, RecordKind::Spell);
    assert_eq!(document.identity[0].value, "Spell");
    assert_eq!(document.identity[2].label, "Rank");
    assert!(document.sections.is_empty());
}

#[test]
fn feat_recipe_surfaces_prerequisites_in_summary() {
    let mut record = base_record(RecordKind::Feat);
    record.requirements.prerequisites = vec![
        "trained in Medicine".to_string(),
        "Battle Medicine".to_string(),
    ];

    let document = build_record_presentation_document(&record);

    assert_eq!(document.sections[0].kind, PresentationSectionKind::Summary);
    assert_section_facts_include(
        &document.sections[0],
        "Prerequisites",
        "trained in Medicine, Battle Medicine",
    );
}

#[test]
fn description_section_does_not_duplicate_primary_content() {
    let mut record = base_record(RecordKind::Rule);
    record.content.documents.push(RecordContentDocument {
        source_kind: ContentSourceKind::PublicNotes,
        label: Some("Public Notes".to_string()),
        document: text_document("Bring a healer's kit."),
    });

    let document = build_record_presentation_document(&record);
    let description = document
        .sections
        .iter()
        .find(|section| section.kind == PresentationSectionKind::Description)
        .expect("description section");
    let content_blocks = description
        .blocks
        .iter()
        .filter(|block| matches!(block, PresentationBlock::Content(_)))
        .count();

    assert_eq!(
        content_blocks, 2,
        "description section should include one primary block plus one supplemental block"
    );
}

#[test]
fn details_section_does_not_expose_foundry_document_type() {
    let record = base_record(RecordKind::Equipment);

    let document = build_record_presentation_document(&record);

    assert!(!document.sections.iter().any(|section| {
        section
            .blocks
            .iter()
            .any(|block| block_has_fact_label(block, "Foundry Document Type"))
    }));
}

#[test]
fn generic_creature_recipe_is_absent_after_cutover() {
    let record = base_record(RecordKind::Creature);
    let document = build_record_presentation_document(&record);

    assert!(document.sections.is_empty());
}

#[test]
fn generic_hazard_recipe_is_absent_after_cutover() {
    let record = base_record(RecordKind::Hazard);
    let document = build_record_presentation_document(&record);

    assert!(document.sections.is_empty());
}

fn text_document(text: &str) -> RichDocument {
    RichDocument::new(vec![RichNode::HtmlElement {
        tag: "p".to_string(),
        attributes: BTreeMap::new(),
        children: vec![RichNode::Text {
            text: text.to_string(),
        }],
    }])
}

fn assert_section_facts_include(section: &PresentationSection, label: &str, value: &str) {
    let has_fact = section.blocks.iter().any(|block| match block {
        PresentationBlock::FactList(facts) => facts
            .iter()
            .any(|fact| fact.label == label && fact.value == value),
        PresentationBlock::Prose(_)
        | PresentationBlock::Content(_)
        | PresentationBlock::Relationships(_) => false,
    });
    assert!(
        has_fact,
        "expected section {:?} to contain {label}: {value}",
        section.kind
    );
}

fn block_has_fact_label(block: &PresentationBlock, label: &str) -> bool {
    match block {
        PresentationBlock::FactList(facts) => facts.iter().any(|fact| fact.label == label),
        PresentationBlock::Prose(_)
        | PresentationBlock::Content(_)
        | PresentationBlock::Relationships(_) => false,
    }
}
