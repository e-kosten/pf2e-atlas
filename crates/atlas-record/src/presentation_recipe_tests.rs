use std::collections::BTreeMap;

use atlas_domain::{
    MetricDomain, PackName, PublicationCategory, Rarity, RecordId, RecordKey, RecordKind,
};

use crate::{
    ActivationTimeSourceField, ActorMechanics, AtlasRecord, ContentSourceKind,
    FoundryDocumentMechanics, FoundryDocumentType, FoundryRecordInfo, FoundryRecordType,
    ItemMechanics, ItemTypeMechanics, MetricDefinition, MetricRow, MetricValue, NormalizedTime,
    PresentationBlock, PresentationSection, PresentationSectionKind, RecordActivationTiming,
    RecordClassification, RecordContent, RecordContentDocument, RecordIdentity, RecordMechanics,
    RecordProvenance, RecordPublication, RecordRequirements, RecordTaxonomy, RecordTiming,
    RecordVisibility, RichDocument, RichNode, SpellDefense, SpellMechanics, SpellRange,
    SpellTarget, build_record_presentation_document, metrics,
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
fn spell_recipe_builds_summary_before_description() {
    let mut record = base_record(RecordKind::Spell);
    record.mechanics.document = FoundryDocumentMechanics::Item(ItemMechanics {
        foundry_type: Some(ItemTypeMechanics::Spell(SpellMechanics {
            traditions: vec!["divine".to_string(), "primal".to_string()],
            kinds: vec!["cantrip".to_string()],
            range: Some(SpellRange {
                text: "30 feet".to_string(),
                distance: Some(30.0),
            }),
            target: Some(SpellTarget {
                text: "1 creature".to_string(),
            }),
            area: None,
            defense: Some(SpellDefense {
                save: Some("fortitude".to_string()),
                basic: true,
            }),
            sustained: false,
            damage_types: vec!["vitality".to_string()],
        })),
        ..ItemMechanics::default()
    });

    let document = build_record_presentation_document(&record);

    assert_eq!(document.kind, RecordKind::Spell);
    assert_eq!(document.identity[0].value, "Spell");
    assert_eq!(document.identity[2].label, "Rank");
    assert_eq!(document.sections[0].kind, PresentationSectionKind::Summary);
    assert_eq!(
        document.sections[1].kind,
        PresentationSectionKind::Description
    );
    assert_section_facts_include(&document.sections[0], "Traditions", "Divine, Primal");
    assert_section_facts_include(&document.sections[0], "Save", "basic Fortitude");
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
    let mut record = base_record(RecordKind::Spell);
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
fn creature_recipe_groups_defense_movement_and_offense_sections() {
    let mut record = base_record(RecordKind::Creature);
    record.mechanics.document = FoundryDocumentMechanics::Actor(ActorMechanics {
        size: Some("medium".to_string()),
        languages: vec!["common".to_string()],
        speed_types: vec!["land".to_string()],
        senses: vec!["darkvision".to_string()],
        immunities: vec!["poison".to_string()],
        resistances: vec!["fire".to_string()],
        weaknesses: vec!["cold iron".to_string()],
        disable_text: None,
        disable_skills: Vec::new(),
        is_complex: false,
    });
    record.mechanics.metrics = vec![
        defined_metric(metrics::actor::PERCEPTION_MOD, 9.0),
        metric(&metrics::actor::ability::mod_key("str"), 4.0),
        defined_metric(metrics::actor::ARMOR_CLASS, 19.0),
        defined_metric(metrics::actor::HP_VALUE, 45.0),
        metric(&metrics::actor::save::mod_key("fort"), 12.0),
        metric(&metrics::actor::save::mod_key("ref"), 8.0),
        metric(&metrics::actor::save::mod_key("will"), 7.0),
        metric(&metrics::actor::speed::value_key("land"), 25.0),
    ];

    let document = build_record_presentation_document(&record);

    assert_eq!(
        document
            .sections
            .iter()
            .map(|section| section.kind)
            .collect::<Vec<_>>(),
        vec![
            PresentationSectionKind::Summary,
            PresentationSectionKind::Defense,
            PresentationSectionKind::Movement,
            PresentationSectionKind::Offense,
            PresentationSectionKind::Description,
            PresentationSectionKind::Details,
        ]
    );
    assert_section_facts_include(&document.sections[1], "AC", "19");
    assert_section_facts_include(&document.sections[1], "Saves", "Fort +12, Ref +8, Will +7");
    assert_section_facts_include(&document.sections[2], "Speed", "Land 25 feet");
}

#[test]
fn hazard_recipe_drops_empty_sections_and_keeps_disable_routine() {
    let mut record = base_record(RecordKind::Hazard);
    record.content.documents.clear();
    record.mechanics.document = FoundryDocumentMechanics::Actor(ActorMechanics {
        size: None,
        languages: Vec::new(),
        speed_types: Vec::new(),
        senses: Vec::new(),
        immunities: Vec::new(),
        resistances: Vec::new(),
        weaknesses: Vec::new(),
        disable_text: Some("thievery to disable the needle launcher".to_string()),
        disable_skills: vec!["thievery".to_string()],
        is_complex: true,
    });
    record.mechanics.metrics = vec![
        defined_metric(metrics::actor::STEALTH_DC, 22.0),
        defined_metric(metrics::actor::ARMOR_CLASS, 18.0),
    ];

    let document = build_record_presentation_document(&record);

    assert!(
        !document
            .sections
            .iter()
            .any(|section| section.kind == PresentationSectionKind::Description)
    );
    assert_section_facts_include(&document.sections[0], "Complexity", "Complex");
    assert!(
        document
            .sections
            .iter()
            .any(|section| section.kind == PresentationSectionKind::Routine)
    );
}

fn metric(key: &str, value: f64) -> MetricRow {
    MetricRow {
        domain: MetricDomain::Actor,
        key: key.to_string(),
        value: MetricValue::Number(value),
    }
}

fn defined_metric(definition: MetricDefinition, value: f64) -> MetricRow {
    metric(
        definition
            .exact_key()
            .expect("test metric definition should have a static key"),
        value,
    )
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
