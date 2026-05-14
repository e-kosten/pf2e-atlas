use atlas_domain::{
    MetricDomain, PackName, PublicationFamily, RecordFamily, RecordId, RecordKey, TextStatus,
};

use crate::{
    ActorSideData, MetricRow, MetricValue, NormalizedRecord, PresentationBlock,
    PresentationSection, PresentationSectionKind, SpellSideData,
    build_record_presentation_document,
};

fn base_record(record_family: RecordFamily) -> NormalizedRecord {
    NormalizedRecord {
        key: RecordKey::new(
            PackName::new("test-pack").expect("pack should parse"),
            RecordId::new("TestRecord").expect("id should parse"),
        ),
        id: RecordId::new("TestRecord").expect("id should parse"),
        name: "Test Record".to_string(),
        normalized_name: "test record".to_string(),
        record_family,
        pack_name: PackName::new("test-pack").expect("pack should parse"),
        pack_label: "Test Pack".to_string(),
        foundry_document_type: "Item".to_string(),
        foundry_record_type: record_family.as_str().to_string(),
        level: Some(3),
        rarity: Some("uncommon".to_string()),
        traits: vec!["healing".to_string(), "vitality".to_string()],
        system_category: None,
        system_group: None,
        system_base_item: None,
        system_usage: None,
        system_price_json: None,
        system_actions_value: Some(2),
        system_time_value: None,
        system_duration_value: None,
        price_cp: None,
        activation_time: None,
        duration: None,
        metrics: Vec::new(),
        actor_data: None,
        item_data: None,
        spell_data: None,
        publication_title: Some("Player Core".to_string()),
        publication_remaster: true,
        description_text: Some("Restores vitality to a wounded ally.".to_string()),
        blurb_text: None,
        publication_family: PublicationFamily::Core,
        folder_id: None,
        taxonomy_families: vec!["support".to_string()],
        variant_group_key: None,
        variant_base_name: None,
        variant_label: None,
        variant_axes: Vec::new(),
        variant_confidence: None,
        variant_source: "none".to_string(),
        source_path: "packs/test-pack/TestRecord.json".to_string(),
        text_status: TextStatus::Resolved,
        is_default_visible: true,
        search_text_projection: "Test Record healing vitality".to_string(),
        reference_candidates: Vec::new(),
        raw_json: "{}".to_string(),
    }
}

#[test]
fn spell_recipe_builds_summary_before_description() {
    let mut record = base_record(RecordFamily::Spell);
    record.spell_data = Some(SpellSideData {
        traditions: vec!["divine".to_string(), "primal".to_string()],
        spell_kinds: vec!["cantrip".to_string()],
        range_text: Some("30 feet".to_string()),
        range_value: Some(30.0),
        target_text: Some("1 creature".to_string()),
        area_type: None,
        area_value: None,
        save_type: Some("fortitude".to_string()),
        sustained: false,
        basic_save: true,
        damage_types: vec!["vitality".to_string()],
    });

    let document = build_record_presentation_document(&record);

    assert_eq!(document.record_family, RecordFamily::Spell);
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
fn creature_recipe_groups_defense_movement_and_offense_sections() {
    let mut record = base_record(RecordFamily::Creature);
    record.actor_data = Some(ActorSideData {
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
    record.metrics = vec![
        metric("perception.mod", 9.0),
        metric("ability.str.mod", 4.0),
        metric("ac.value", 19.0),
        metric("hp.value", 45.0),
        metric("save.fort.mod", 12.0),
        metric("save.ref.mod", 8.0),
        metric("save.will.mod", 7.0),
        metric("speed.land.value", 25.0),
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
    let mut record = base_record(RecordFamily::Hazard);
    record.description_text = None;
    record.actor_data = Some(ActorSideData {
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
    record.metrics = vec![metric("stealth.dc", 22.0), metric("ac.value", 18.0)];

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

fn assert_section_facts_include(section: &PresentationSection, label: &str, value: &str) {
    let has_fact = section.blocks.iter().any(|block| match block {
        PresentationBlock::FactList(facts) => facts
            .iter()
            .any(|fact| fact.label == label && fact.value == value),
        PresentationBlock::Prose(_) | PresentationBlock::Relationships(_) => false,
    });
    assert!(
        has_fact,
        "expected section {:?} to contain {label}: {value}",
        section.kind
    );
}
