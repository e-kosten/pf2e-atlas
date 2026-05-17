use std::collections::BTreeSet;

use atlas_domain::RecordFamily;

use crate::{
    ActorSideData, ItemSideData, MetricRow, NormalizedRecord, PresentationBadge,
    PresentationBadgeKind, PresentationBlock, PresentationFact, PresentationSection,
    PresentationSectionKind, SpellSideData,
    presentation_format::{
        action_count_text, activation_text, duration_text, format_ability_mods, format_area,
        format_bulk, format_list, format_number, format_price_cp, format_save, format_saves,
        format_skill_mods, format_speeds, format_stealth, humanize, metric_number_for_definition,
    },
};

pub fn build_record_presentation_document(
    record: &NormalizedRecord,
) -> crate::RecordPresentationDocument {
    let mut document = crate::RecordPresentationDocument {
        record_key: record.key.clone(),
        record_family: record.record_family,
        title: record.name.clone(),
        identity: identity_facts(record),
        badges: badges(record),
        sections: recipe_sections(record),
    };
    prune_empty_sections(&mut document.sections);
    document
}

fn recipe_sections(record: &NormalizedRecord) -> Vec<PresentationSection> {
    match record.record_family {
        RecordFamily::Spell => spell_sections(record),
        RecordFamily::Creature => creature_sections(record),
        RecordFamily::Equipment => equipment_sections(record),
        RecordFamily::Hazard => hazard_sections(record),
        RecordFamily::Feat | RecordFamily::Rule => feat_action_sections(record),
        _ => fallback_sections(record),
    }
}

fn spell_sections(record: &NormalizedRecord) -> Vec<PresentationSection> {
    vec![
        fact_section(
            PresentationSectionKind::Summary,
            spell_summary_facts(record, record.spell_data.as_ref()),
        ),
        description_section(record),
        details_section(record),
    ]
}

fn creature_sections(record: &NormalizedRecord) -> Vec<PresentationSection> {
    vec![
        fact_section(
            PresentationSectionKind::Summary,
            creature_summary_facts(record.actor_data.as_ref(), &record.metrics),
        ),
        fact_section(
            PresentationSectionKind::Defense,
            creature_defense_facts(record.actor_data.as_ref(), &record.metrics),
        ),
        fact_section(
            PresentationSectionKind::Movement,
            creature_movement_facts(record.actor_data.as_ref(), &record.metrics),
        ),
        fact_section(
            PresentationSectionKind::Offense,
            creature_offense_facts(record, record.spell_data.as_ref()),
        ),
        description_section(record),
        details_section(record),
    ]
}

fn equipment_sections(record: &NormalizedRecord) -> Vec<PresentationSection> {
    vec![
        fact_section(
            PresentationSectionKind::Summary,
            equipment_summary_facts(record, record.item_data.as_ref()),
        ),
        description_section(record),
        details_section(record),
    ]
}

fn hazard_sections(record: &NormalizedRecord) -> Vec<PresentationSection> {
    vec![
        fact_section(
            PresentationSectionKind::Summary,
            hazard_summary_facts(record.actor_data.as_ref(), &record.metrics),
        ),
        fact_section(
            PresentationSectionKind::Defense,
            creature_defense_facts(record.actor_data.as_ref(), &record.metrics),
        ),
        fact_section(
            PresentationSectionKind::Routine,
            hazard_routine_facts(
                record,
                record.actor_data.as_ref(),
                record.spell_data.as_ref(),
            ),
        ),
        description_section(record),
        details_section(record),
    ]
}

fn feat_action_sections(record: &NormalizedRecord) -> Vec<PresentationSection> {
    vec![
        fact_section(
            PresentationSectionKind::Summary,
            action_summary_facts(record),
        ),
        description_section(record),
        details_section(record),
    ]
}

fn fallback_sections(record: &NormalizedRecord) -> Vec<PresentationSection> {
    vec![
        fact_section(
            PresentationSectionKind::Summary,
            fallback_summary_facts(record),
        ),
        description_section(record),
        details_section(record),
    ]
}

fn identity_facts(record: &NormalizedRecord) -> Vec<PresentationFact> {
    [
        fact(
            "family",
            "Family",
            Some(humanize(record.record_family.as_str())),
        ),
        fact("type", "Type", Some(humanize(&record.foundry_record_type))),
        fact(
            "level",
            if record.record_family == RecordFamily::Spell {
                "Rank"
            } else {
                "Level"
            },
            record.level.map(|value| value.to_string()),
        ),
        fact("rarity", "Rarity", record.rarity.as_deref().map(humanize)),
        fact(
            "publication",
            "Publication",
            record.publication_title.as_deref().map(ToString::to_string),
        ),
    ]
    .into_iter()
    .flatten()
    .collect()
}

fn badges(record: &NormalizedRecord) -> Vec<PresentationBadge> {
    record
        .traits
        .iter()
        .map(|value| PresentationBadge {
            kind: PresentationBadgeKind::Trait,
            label: "Trait".to_string(),
            value: humanize(value),
        })
        .chain(
            record
                .taxonomy_families
                .iter()
                .map(|value| PresentationBadge {
                    kind: PresentationBadgeKind::Classification,
                    label: "Family".to_string(),
                    value: humanize(value),
                }),
        )
        .collect()
}

fn spell_summary_facts(
    record: &NormalizedRecord,
    spell: Option<&SpellSideData>,
) -> Vec<PresentationFact> {
    let mut facts = Vec::new();
    if let Some(spell) = spell {
        push_fact(
            &mut facts,
            "traditions",
            "Traditions",
            format_list(&spell.traditions),
        );
        push_fact(&mut facts, "range", "Range", spell.range_text.clone());
        push_fact(&mut facts, "area", "Area", format_area(spell));
        push_fact(&mut facts, "save", "Save", format_save(spell));
        push_fact(&mut facts, "duration", "Duration", duration_text(record));
        push_fact(&mut facts, "targets", "Targets", spell.target_text.clone());
        push_fact(
            &mut facts,
            "damage",
            "Damage",
            format_list(&spell.damage_types),
        );
        push_fact(
            &mut facts,
            "spellKinds",
            "Spell Kinds",
            format_list(&spell.spell_kinds),
        );
        push_fact(
            &mut facts,
            "sustained",
            "Sustained",
            spell.sustained.then(|| "Yes".to_string()),
        );
    } else {
        push_fact(&mut facts, "duration", "Duration", duration_text(record));
    }
    push_fact(&mut facts, "cast", "Cast", activation_text(record));
    facts
}

fn creature_summary_facts(
    actor: Option<&ActorSideData>,
    metrics: &[MetricRow],
) -> Vec<PresentationFact> {
    let mut facts = Vec::new();
    if let Some(actor) = actor {
        push_fact(
            &mut facts,
            "size",
            "Size",
            actor.size.as_deref().map(humanize),
        );
        push_fact(
            &mut facts,
            "languages",
            "Languages",
            format_list(&actor.languages),
        );
        push_fact(&mut facts, "senses", "Senses", format_list(&actor.senses));
    }
    push_fact(
        &mut facts,
        "perception",
        "Perception",
        metric_number_for_definition(metrics, crate::metrics::actor::PERCEPTION_MOD)
            .map(crate::presentation_format::format_modifier),
    );
    push_fact(&mut facts, "skills", "Skills", format_skill_mods(metrics));
    push_fact(
        &mut facts,
        "abilities",
        "Abilities",
        format_ability_mods(metrics),
    );
    facts
}

fn creature_defense_facts(
    actor: Option<&ActorSideData>,
    metrics: &[MetricRow],
) -> Vec<PresentationFact> {
    let mut facts = Vec::new();
    push_fact(
        &mut facts,
        "ac",
        "AC",
        metric_number_for_definition(metrics, crate::metrics::actor::ARMOR_CLASS)
            .map(format_number),
    );
    push_fact(
        &mut facts,
        "hp",
        "HP",
        metric_number_for_definition(metrics, crate::metrics::actor::HP_VALUE).map(format_number),
    );
    push_fact(
        &mut facts,
        "hardness",
        "Hardness",
        metric_number_for_definition(metrics, crate::metrics::actor::HARDNESS).map(format_number),
    );
    push_fact(&mut facts, "saves", "Saves", format_saves(metrics));
    if let Some(actor) = actor {
        push_fact(
            &mut facts,
            "immunities",
            "Immunities",
            format_list(&actor.immunities),
        );
        push_fact(
            &mut facts,
            "resistances",
            "Resistances",
            format_list(&actor.resistances),
        );
        push_fact(
            &mut facts,
            "weaknesses",
            "Weaknesses",
            format_list(&actor.weaknesses),
        );
    }
    facts
}

fn creature_movement_facts(
    actor: Option<&ActorSideData>,
    metrics: &[MetricRow],
) -> Vec<PresentationFact> {
    let mut facts = Vec::new();
    push_fact(&mut facts, "speed", "Speed", format_speeds(metrics));
    if let Some(actor) = actor {
        push_fact(
            &mut facts,
            "speedTypes",
            "Speed Types",
            format_list(&actor.speed_types),
        );
    }
    facts
}

fn creature_offense_facts(
    record: &NormalizedRecord,
    spell: Option<&SpellSideData>,
) -> Vec<PresentationFact> {
    let mut facts = Vec::new();
    if let Some(spell) = spell {
        push_fact(
            &mut facts,
            "damage",
            "Damage",
            format_list(&spell.damage_types),
        );
        push_fact(
            &mut facts,
            "spellKinds",
            "Spell Kinds",
            format_list(&spell.spell_kinds),
        );
        push_fact(&mut facts, "save", "Save", format_save(spell));
    }
    push_fact(
        &mut facts,
        "actionCost",
        "Action Cost",
        action_count_text(record.system_actions_value),
    );
    facts
}

fn equipment_summary_facts(
    record: &NormalizedRecord,
    item: Option<&ItemSideData>,
) -> Vec<PresentationFact> {
    let mut facts = Vec::new();
    let price_cp = item.and_then(|item| item.price_cp).or(record.price_cp);
    push_fact(&mut facts, "price", "Price", price_cp.map(format_price_cp));
    if let Some(item) = item {
        push_fact(&mut facts, "bulk", "Bulk", item.bulk_value.map(format_bulk));
        push_fact(
            &mut facts,
            "usage",
            "Usage",
            item.system_usage.as_deref().map(humanize),
        );
        push_fact(&mut facts, "hands", "Hands", item.hands_requirement.clone());
        push_fact(
            &mut facts,
            "baseItem",
            "Base Item",
            item.system_base_item.as_deref().map(humanize),
        );
        push_fact(
            &mut facts,
            "itemCategory",
            "Category",
            item.system_category.as_deref().map(humanize),
        );
        push_fact(
            &mut facts,
            "group",
            "Group",
            item.system_group.as_deref().map(humanize),
        );
        push_fact(
            &mut facts,
            "damage",
            "Damage",
            format_list(&item.damage_types),
        );
    }
    push_fact(
        &mut facts,
        "activation",
        "Activation",
        activation_text(record),
    );
    facts
}

fn hazard_summary_facts(
    actor: Option<&ActorSideData>,
    metrics: &[MetricRow],
) -> Vec<PresentationFact> {
    let mut facts = Vec::new();
    if let Some(actor) = actor {
        push_fact(
            &mut facts,
            "complexity",
            "Complexity",
            actor.is_complex.then(|| "Complex".to_string()),
        );
        push_fact(&mut facts, "disable", "Disable", actor.disable_text.clone());
        push_fact(
            &mut facts,
            "disableSkills",
            "Disable Skills",
            format_list(&actor.disable_skills),
        );
    }
    push_fact(&mut facts, "stealth", "Stealth", format_stealth(metrics));
    facts
}

fn hazard_routine_facts(
    record: &NormalizedRecord,
    actor: Option<&ActorSideData>,
    spell: Option<&SpellSideData>,
) -> Vec<PresentationFact> {
    let mut facts = Vec::new();
    if let Some(spell) = spell {
        push_fact(&mut facts, "range", "Range", spell.range_text.clone());
        push_fact(&mut facts, "area", "Area", format_area(spell));
        push_fact(&mut facts, "save", "Save", format_save(spell));
        push_fact(
            &mut facts,
            "damage",
            "Damage",
            format_list(&spell.damage_types),
        );
        push_fact(&mut facts, "targets", "Targets", spell.target_text.clone());
    }
    if let Some(actor) = actor {
        push_fact(&mut facts, "disable", "Disable", actor.disable_text.clone());
    }
    push_fact(&mut facts, "duration", "Duration", duration_text(record));
    facts
}

fn action_summary_facts(record: &NormalizedRecord) -> Vec<PresentationFact> {
    let mut facts = Vec::new();
    push_fact(
        &mut facts,
        "actionCost",
        "Action Cost",
        activation_text(record),
    );
    if let Some(spell) = record.spell_data.as_ref() {
        push_fact(&mut facts, "range", "Range", spell.range_text.clone());
        push_fact(&mut facts, "area", "Area", format_area(spell));
        push_fact(&mut facts, "save", "Save", format_save(spell));
        push_fact(&mut facts, "duration", "Duration", duration_text(record));
        push_fact(&mut facts, "targets", "Targets", spell.target_text.clone());
        push_fact(
            &mut facts,
            "damage",
            "Damage",
            format_list(&spell.damage_types),
        );
    }
    facts
}

fn fallback_summary_facts(record: &NormalizedRecord) -> Vec<PresentationFact> {
    let mut facts = Vec::new();
    push_fact(
        &mut facts,
        "actionCost",
        "Action Cost",
        activation_text(record),
    );
    push_fact(&mut facts, "duration", "Duration", duration_text(record));
    facts
}

fn details_section(record: &NormalizedRecord) -> PresentationSection {
    let facts = [
        fact(
            "taxonomyFamilies",
            "Families",
            format_list(&record.taxonomy_families),
        ),
        fact(
            "publicationFamily",
            "Publication Family",
            Some(humanize(record.publication_family.as_str())),
        ),
        fact("pack", "Pack", Some(record.pack_name.to_string())),
        fact(
            "foundryDocumentType",
            "Foundry Document Type",
            Some(record.foundry_document_type.clone()),
        ),
    ]
    .into_iter()
    .flatten()
    .collect();
    fact_section(PresentationSectionKind::Details, facts)
}

fn description_section(record: &NormalizedRecord) -> PresentationSection {
    let mut blocks = Vec::new();
    if let Some(document) = &record.description {
        blocks.push(PresentationBlock::Content(document.clone()));
    } else if let Some(document) = &record.blurb {
        blocks.push(PresentationBlock::Content(document.clone()));
    }
    blocks.extend(
        record
            .supplemental_content
            .iter()
            .filter(|content| content.contributes_to_search)
            .map(|content| {
                PresentationBlock::Content(labeled_content_document(
                    content.label.as_deref(),
                    &content.document,
                ))
            }),
    );
    PresentationSection::new(PresentationSectionKind::Description, blocks)
}

fn labeled_content_document(
    label: Option<&str>,
    document: &crate::ContentDocument,
) -> crate::ContentDocument {
    let Some(label) = label.filter(|value| !value.trim().is_empty()) else {
        return document.clone();
    };
    let mut blocks = vec![crate::ContentBlock::Heading {
        level: 3,
        content: vec![crate::ContentInline::Text {
            text: label.to_string(),
        }],
    }];
    blocks.extend(document.blocks.clone());
    crate::ContentDocument::new(blocks)
}

fn fact_section(
    kind: PresentationSectionKind,
    facts: Vec<PresentationFact>,
) -> PresentationSection {
    let blocks = if facts.is_empty() {
        Vec::new()
    } else {
        vec![PresentationBlock::FactList(dedupe_facts(facts))]
    };
    PresentationSection::new(kind, blocks)
}

fn prune_empty_sections(sections: &mut Vec<PresentationSection>) {
    sections.retain(|section| {
        section.blocks.iter().any(|block| match block {
            PresentationBlock::FactList(facts) => !facts.is_empty(),
            PresentationBlock::Prose(text) => !text.text.trim().is_empty(),
            PresentationBlock::Content(document) => !document.is_empty(),
            PresentationBlock::Relationships(relationships) => !relationships.is_empty(),
        })
    });
}

fn dedupe_facts(facts: Vec<PresentationFact>) -> Vec<PresentationFact> {
    let mut seen = BTreeSet::new();
    facts
        .into_iter()
        .filter(|fact| seen.insert(format!("{}:{}", fact.label, fact.value).to_lowercase()))
        .collect()
}

fn fact(
    key: impl Into<String>,
    label: impl Into<String>,
    value: Option<String>,
) -> Option<PresentationFact> {
    let value = value?;
    let value = value.trim();
    if value.is_empty() {
        return None;
    }
    Some(PresentationFact {
        key: key.into(),
        label: label.into(),
        value: value.to_string(),
    })
}

fn push_fact(
    facts: &mut Vec<PresentationFact>,
    key: impl Into<String>,
    label: impl Into<String>,
    value: Option<String>,
) {
    if let Some(fact) = fact(key, label, value) {
        facts.push(fact);
    }
}
