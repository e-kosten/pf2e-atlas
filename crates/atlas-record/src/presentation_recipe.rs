use std::collections::BTreeSet;

use atlas_domain::RecordKind;

use crate::{
    ActorMechanics, AtlasRecord, ItemMechanics, MetricRow, PresentationBadge,
    PresentationBadgeKind, PresentationBlock, PresentationFact, PresentationSection,
    PresentationSectionKind, RecordContentDocument, SpellMechanics,
    presentation_content::project_presentation_content,
    presentation_format::{
        action_count_text, activation_text, duration_text, format_ability_mods, format_area,
        format_bulk, format_list, format_number, format_price_cp, format_save, format_saves,
        format_skill_mods, format_speeds, format_stealth, humanize, metric_number_for_definition,
    },
};

pub fn build_record_presentation_document(
    record: &AtlasRecord,
) -> crate::RecordPresentationDocument {
    build_record_presentation_document_with_content_filter(record, |_| true)
}

pub fn build_record_presentation_document_with_content_filter(
    record: &AtlasRecord,
    include_supplemental_content: impl Fn(&RecordContentDocument) -> bool + Copy,
) -> crate::RecordPresentationDocument {
    let mut document = crate::RecordPresentationDocument {
        record_key: record.identity.key.clone(),
        kind: record.classification.kind,
        title: record.identity.name.clone(),
        identity: identity_facts(record),
        badges: badges(record),
        sections: recipe_sections(record, include_supplemental_content),
    };
    prune_empty_sections(&mut document.sections);
    document
}

fn recipe_sections(
    record: &AtlasRecord,
    include_supplemental_content: impl Fn(&RecordContentDocument) -> bool + Copy,
) -> Vec<PresentationSection> {
    match record.classification.kind {
        RecordKind::Spell => spell_sections(record, include_supplemental_content),
        RecordKind::Creature => creature_sections(record, include_supplemental_content),
        RecordKind::Equipment => equipment_sections(record, include_supplemental_content),
        RecordKind::Hazard => hazard_sections(record, include_supplemental_content),
        RecordKind::Feat | RecordKind::Rule => {
            feat_action_sections(record, include_supplemental_content)
        }
        _ => fallback_sections(record, include_supplemental_content),
    }
}

fn spell_sections(
    record: &AtlasRecord,
    include_supplemental_content: impl Fn(&RecordContentDocument) -> bool + Copy,
) -> Vec<PresentationSection> {
    vec![
        fact_section(
            PresentationSectionKind::Summary,
            spell_summary_facts(record, record.mechanics.spell()),
        ),
        description_section(record, include_supplemental_content),
        details_section(record),
    ]
}

fn creature_sections(
    record: &AtlasRecord,
    include_supplemental_content: impl Fn(&RecordContentDocument) -> bool + Copy,
) -> Vec<PresentationSection> {
    vec![
        fact_section(
            PresentationSectionKind::Summary,
            creature_summary_facts(record.mechanics.actor(), &record.mechanics.metrics),
        ),
        fact_section(
            PresentationSectionKind::Defense,
            creature_defense_facts(record.mechanics.actor(), &record.mechanics.metrics),
        ),
        fact_section(
            PresentationSectionKind::Movement,
            creature_movement_facts(record.mechanics.actor(), &record.mechanics.metrics),
        ),
        fact_section(
            PresentationSectionKind::Offense,
            creature_offense_facts(record, record.mechanics.spell()),
        ),
        description_section(record, include_supplemental_content),
        details_section(record),
    ]
}

fn equipment_sections(
    record: &AtlasRecord,
    include_supplemental_content: impl Fn(&RecordContentDocument) -> bool + Copy,
) -> Vec<PresentationSection> {
    vec![
        fact_section(
            PresentationSectionKind::Summary,
            equipment_summary_facts(record, record.mechanics.item()),
        ),
        description_section(record, include_supplemental_content),
        details_section(record),
    ]
}

fn hazard_sections(
    record: &AtlasRecord,
    include_supplemental_content: impl Fn(&RecordContentDocument) -> bool + Copy,
) -> Vec<PresentationSection> {
    vec![
        fact_section(
            PresentationSectionKind::Summary,
            hazard_summary_facts(record.mechanics.actor(), &record.mechanics.metrics),
        ),
        fact_section(
            PresentationSectionKind::Defense,
            creature_defense_facts(record.mechanics.actor(), &record.mechanics.metrics),
        ),
        fact_section(
            PresentationSectionKind::Routine,
            hazard_routine_facts(record, record.mechanics.actor(), record.mechanics.spell()),
        ),
        description_section(record, include_supplemental_content),
        details_section(record),
    ]
}

fn feat_action_sections(
    record: &AtlasRecord,
    include_supplemental_content: impl Fn(&RecordContentDocument) -> bool + Copy,
) -> Vec<PresentationSection> {
    vec![
        fact_section(
            PresentationSectionKind::Summary,
            action_summary_facts(record),
        ),
        description_section(record, include_supplemental_content),
        details_section(record),
    ]
}

fn fallback_sections(
    record: &AtlasRecord,
    include_supplemental_content: impl Fn(&RecordContentDocument) -> bool + Copy,
) -> Vec<PresentationSection> {
    vec![
        fact_section(
            PresentationSectionKind::Summary,
            fallback_summary_facts(record),
        ),
        description_section(record, include_supplemental_content),
        details_section(record),
    ]
}

fn identity_facts(record: &AtlasRecord) -> Vec<PresentationFact> {
    [
        fact(
            "kind",
            "Kind",
            Some(humanize(record.classification.kind.as_str())),
        ),
        fact(
            "type",
            "Type",
            Some(humanize(record.foundry.record_type.as_str())),
        ),
        fact(
            "level",
            if record.classification.kind == RecordKind::Spell {
                "Rank"
            } else {
                "Level"
            },
            record.classification.level.map(|value| value.to_string()),
        ),
        fact(
            "rarity",
            "Rarity",
            record
                .classification
                .rarity
                .map(|rarity| humanize(rarity.as_str())),
        ),
        fact(
            "publication",
            "Publication",
            record.publication.title.as_deref().map(ToString::to_string),
        ),
    ]
    .into_iter()
    .flatten()
    .collect()
}

fn badges(record: &AtlasRecord) -> Vec<PresentationBadge> {
    record
        .classification
        .traits
        .iter()
        .map(|value| PresentationBadge {
            kind: PresentationBadgeKind::Trait,
            label: "Trait".to_string(),
            value: humanize(value),
        })
        .chain(
            record
                .classification
                .taxonomy
                .inferred_groups
                .iter()
                .map(|value| PresentationBadge {
                    kind: PresentationBadgeKind::Classification,
                    label: "Taxonomy".to_string(),
                    value: humanize(value),
                }),
        )
        .collect()
}

fn spell_summary_facts(
    record: &AtlasRecord,
    spell: Option<&SpellMechanics>,
) -> Vec<PresentationFact> {
    let mut facts = Vec::new();
    if let Some(spell) = spell {
        push_fact(
            &mut facts,
            "traditions",
            "Traditions",
            format_list(&spell.traditions),
        );
        push_fact(
            &mut facts,
            "range",
            "Range",
            spell.range.as_ref().map(|range| range.text.clone()),
        );
        push_fact(&mut facts, "area", "Area", format_area(spell));
        push_fact(&mut facts, "save", "Save", format_save(spell));
        push_fact(
            &mut facts,
            "duration",
            "Duration",
            duration_text(record.timing.duration_time()),
        );
        push_fact(
            &mut facts,
            "targets",
            "Targets",
            spell.target.as_ref().map(|target| target.text.clone()),
        );
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
            format_list(&spell.kinds),
        );
        push_fact(
            &mut facts,
            "sustained",
            "Sustained",
            spell.sustained.then(|| "Yes".to_string()),
        );
    } else {
        push_fact(
            &mut facts,
            "duration",
            "Duration",
            duration_text(record.timing.duration_time()),
        );
    }
    push_fact(
        &mut facts,
        "cast",
        "Cast",
        activation_text(
            record.timing.activation_time(),
            record.timing.activation_actions_value(),
        ),
    );
    facts
}

fn creature_summary_facts(
    actor: Option<&ActorMechanics>,
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
    actor: Option<&ActorMechanics>,
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
    actor: Option<&ActorMechanics>,
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
    record: &AtlasRecord,
    spell: Option<&SpellMechanics>,
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
            format_list(&spell.kinds),
        );
        push_fact(&mut facts, "save", "Save", format_save(spell));
    }
    push_fact(
        &mut facts,
        "actionCost",
        "Action Cost",
        action_count_text(record.timing.activation_actions_value()),
    );
    facts
}

fn equipment_summary_facts(
    record: &AtlasRecord,
    item: Option<&ItemMechanics>,
) -> Vec<PresentationFact> {
    let mut facts = Vec::new();
    let price_cp = item.and_then(|item| item.price_cp);
    push_fact(&mut facts, "price", "Price", price_cp.map(format_price_cp));
    if let Some(item) = item {
        push_fact(&mut facts, "bulk", "Bulk", item.bulk_value.map(format_bulk));
        push_fact(
            &mut facts,
            "usage",
            "Usage",
            item.usage.as_deref().map(humanize),
        );
        push_fact(&mut facts, "hands", "Hands", item.hands_requirement.clone());
        push_fact(
            &mut facts,
            "baseItem",
            "Base Item",
            item.base_item.as_deref().map(humanize),
        );
        push_fact(
            &mut facts,
            "itemCategory",
            "Category",
            item.category.as_deref().map(humanize),
        );
        push_fact(
            &mut facts,
            "group",
            "Group",
            item.group.as_deref().map(humanize),
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
        activation_text(
            record.timing.activation_time(),
            record.timing.activation_actions_value(),
        ),
    );
    facts
}

fn hazard_summary_facts(
    actor: Option<&ActorMechanics>,
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
    record: &AtlasRecord,
    actor: Option<&ActorMechanics>,
    spell: Option<&SpellMechanics>,
) -> Vec<PresentationFact> {
    let mut facts = Vec::new();
    if let Some(spell) = spell {
        push_fact(
            &mut facts,
            "range",
            "Range",
            spell.range.as_ref().map(|range| range.text.clone()),
        );
        push_fact(&mut facts, "area", "Area", format_area(spell));
        push_fact(&mut facts, "save", "Save", format_save(spell));
        push_fact(
            &mut facts,
            "damage",
            "Damage",
            format_list(&spell.damage_types),
        );
        push_fact(
            &mut facts,
            "targets",
            "Targets",
            spell.target.as_ref().map(|target| target.text.clone()),
        );
    }
    if let Some(actor) = actor {
        push_fact(&mut facts, "disable", "Disable", actor.disable_text.clone());
    }
    push_fact(
        &mut facts,
        "duration",
        "Duration",
        duration_text(record.timing.duration_time()),
    );
    facts
}

fn action_summary_facts(record: &AtlasRecord) -> Vec<PresentationFact> {
    let mut facts = Vec::new();
    push_fact(
        &mut facts,
        "prerequisites",
        "Prerequisites",
        format_prerequisites(&record.requirements.prerequisites),
    );
    push_fact(
        &mut facts,
        "actionCost",
        "Action Cost",
        activation_text(
            record.timing.activation_time(),
            record.timing.activation_actions_value(),
        ),
    );
    if let Some(spell) = record.mechanics.spell() {
        push_fact(
            &mut facts,
            "range",
            "Range",
            spell.range.as_ref().map(|range| range.text.clone()),
        );
        push_fact(&mut facts, "area", "Area", format_area(spell));
        push_fact(&mut facts, "save", "Save", format_save(spell));
        push_fact(
            &mut facts,
            "duration",
            "Duration",
            duration_text(record.timing.duration_time()),
        );
        push_fact(
            &mut facts,
            "targets",
            "Targets",
            spell.target.as_ref().map(|target| target.text.clone()),
        );
        push_fact(
            &mut facts,
            "damage",
            "Damage",
            format_list(&spell.damage_types),
        );
    }
    facts
}

fn format_prerequisites(values: &[String]) -> Option<String> {
    let values = values
        .iter()
        .map(|value| value.trim())
        .filter(|value| !value.is_empty())
        .collect::<Vec<_>>();
    (!values.is_empty()).then(|| values.join(", "))
}

fn fallback_summary_facts(record: &AtlasRecord) -> Vec<PresentationFact> {
    let mut facts = Vec::new();
    push_fact(
        &mut facts,
        "actionCost",
        "Action Cost",
        activation_text(
            record.timing.activation_time(),
            record.timing.activation_actions_value(),
        ),
    );
    push_fact(
        &mut facts,
        "duration",
        "Duration",
        duration_text(record.timing.duration_time()),
    );
    facts
}

fn details_section(record: &AtlasRecord) -> PresentationSection {
    let facts = [
        fact(
            "taxonomy",
            "Taxonomy",
            format_list(&record.classification.taxonomy.inferred_groups),
        ),
        fact(
            "publicationFamily",
            "Publication Family",
            Some(humanize(record.publication.category.as_str())),
        ),
        fact("pack", "Pack", Some(record.identity.pack().to_string())),
        fact(
            "foundryDocumentType",
            "Foundry Document Type",
            Some(record.foundry.document_type.as_str().to_string()),
        ),
    ]
    .into_iter()
    .flatten()
    .collect();
    fact_section(PresentationSectionKind::Details, facts)
}

fn description_section(
    record: &AtlasRecord,
    include_supplemental_content: impl Fn(&RecordContentDocument) -> bool + Copy,
) -> PresentationSection {
    let mut blocks = Vec::new();
    if let Some(document) = record.content.description() {
        blocks.push(PresentationBlock::Content(project_presentation_content(
            document,
        )));
    } else if let Some(document) = record.content.blurb() {
        blocks.push(PresentationBlock::Content(project_presentation_content(
            document,
        )));
    }
    blocks.extend(
        record
            .content
            .searchable_documents()
            .filter(|content| {
                content.contributes_to_search() && include_supplemental_content(content)
            })
            .map(|content| {
                PresentationBlock::Content(labeled_content(
                    content.label.as_deref(),
                    &content.document,
                ))
            }),
    );
    PresentationSection::new(PresentationSectionKind::Description, blocks)
}

fn labeled_content(
    label: Option<&str>,
    document: &crate::RichDocument,
) -> crate::PresentationContent {
    let mut content = project_presentation_content(document);
    let Some(label) = label.filter(|value| !value.trim().is_empty()) else {
        return content;
    };
    content.blocks.insert(
        0,
        crate::PresentationContentBlock::Heading {
            level: 3,
            text: label.to_string(),
        },
    );
    content
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
            PresentationBlock::Content(content) => !content.is_empty(),
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
