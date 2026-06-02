use std::collections::BTreeSet;

use atlas_domain::{PackName, PublicationFamily, RecordFamily, RecordKey};

use crate::{
    ActorSideData, ContentDocument, ItemSideData, MetricRow, NormalizedRecord, PersistedRecord,
    PresentationBadge, PresentationBadgeKind, PresentationBlock, PresentationFact,
    PresentationSection, PresentationSectionKind, SpellSideData, SupplementalContentDocument,
    presentation_format::{
        action_count_text, activation_text, duration_text, format_ability_mods, format_area,
        format_bulk, format_list, format_number, format_price_cp, format_save, format_saves,
        format_skill_mods, format_speeds, format_stealth, humanize, metric_number_for_definition,
    },
};

pub trait RecordPresentationSource {
    fn key(&self) -> &RecordKey;
    fn name(&self) -> &str;
    fn record_family(&self) -> RecordFamily;
    fn pack_name(&self) -> &PackName;
    fn foundry_document_type(&self) -> &str;
    fn foundry_record_type(&self) -> &str;
    fn level(&self) -> Option<i64>;
    fn rarity(&self) -> Option<&str>;
    fn traits(&self) -> &[String];
    fn prerequisites(&self) -> &[String];
    fn system_actions_value(&self) -> Option<i64>;
    fn price_cp(&self) -> Option<i64>;
    fn activation_time(&self) -> Option<&crate::NormalizedTime>;
    fn duration(&self) -> Option<&crate::NormalizedTime>;
    fn metrics(&self) -> &[MetricRow];
    fn actor_data(&self) -> Option<&ActorSideData>;
    fn item_data(&self) -> Option<&ItemSideData>;
    fn spell_data(&self) -> Option<&SpellSideData>;
    fn publication_title(&self) -> Option<&str>;
    fn description(&self) -> Option<&ContentDocument>;
    fn blurb(&self) -> Option<&ContentDocument>;
    fn supplemental_content(&self) -> &[SupplementalContentDocument];
    fn publication_family(&self) -> PublicationFamily;
    fn taxonomy_families(&self) -> &[String];
}

impl RecordPresentationSource for NormalizedRecord {
    fn key(&self) -> &RecordKey {
        &self.key
    }
    fn name(&self) -> &str {
        &self.name
    }
    fn record_family(&self) -> RecordFamily {
        self.record_family
    }
    fn pack_name(&self) -> &PackName {
        &self.pack_name
    }
    fn foundry_document_type(&self) -> &str {
        &self.foundry_document_type
    }
    fn foundry_record_type(&self) -> &str {
        &self.foundry_record_type
    }
    fn level(&self) -> Option<i64> {
        self.level
    }
    fn rarity(&self) -> Option<&str> {
        self.rarity.as_deref()
    }
    fn traits(&self) -> &[String] {
        &self.traits
    }
    fn prerequisites(&self) -> &[String] {
        &self.prerequisites
    }
    fn system_actions_value(&self) -> Option<i64> {
        self.system_actions_value
    }
    fn price_cp(&self) -> Option<i64> {
        self.price_cp
    }
    fn activation_time(&self) -> Option<&crate::NormalizedTime> {
        self.activation_time.as_ref()
    }
    fn duration(&self) -> Option<&crate::NormalizedTime> {
        self.duration.as_ref()
    }
    fn metrics(&self) -> &[MetricRow] {
        &self.metrics
    }
    fn actor_data(&self) -> Option<&ActorSideData> {
        self.actor_data.as_ref()
    }
    fn item_data(&self) -> Option<&ItemSideData> {
        self.item_data.as_ref()
    }
    fn spell_data(&self) -> Option<&SpellSideData> {
        self.spell_data.as_ref()
    }
    fn publication_title(&self) -> Option<&str> {
        self.publication_title.as_deref()
    }
    fn description(&self) -> Option<&ContentDocument> {
        self.description.as_ref()
    }
    fn blurb(&self) -> Option<&ContentDocument> {
        self.blurb.as_ref()
    }
    fn supplemental_content(&self) -> &[SupplementalContentDocument] {
        &self.supplemental_content
    }
    fn publication_family(&self) -> PublicationFamily {
        self.publication_family
    }
    fn taxonomy_families(&self) -> &[String] {
        &self.taxonomy_families
    }
}

impl RecordPresentationSource for PersistedRecord {
    fn key(&self) -> &RecordKey {
        &self.key
    }
    fn name(&self) -> &str {
        &self.name
    }
    fn record_family(&self) -> RecordFamily {
        self.record_family
    }
    fn pack_name(&self) -> &PackName {
        &self.pack_name
    }
    fn foundry_document_type(&self) -> &str {
        &self.foundry_document_type
    }
    fn foundry_record_type(&self) -> &str {
        &self.foundry_record_type
    }
    fn level(&self) -> Option<i64> {
        self.level
    }
    fn rarity(&self) -> Option<&str> {
        self.rarity.as_deref()
    }
    fn traits(&self) -> &[String] {
        &self.traits
    }
    fn prerequisites(&self) -> &[String] {
        &self.prerequisites
    }
    fn system_actions_value(&self) -> Option<i64> {
        self.system_actions_value
    }
    fn price_cp(&self) -> Option<i64> {
        self.price_cp
    }
    fn activation_time(&self) -> Option<&crate::NormalizedTime> {
        self.activation_time.as_ref()
    }
    fn duration(&self) -> Option<&crate::NormalizedTime> {
        self.duration.as_ref()
    }
    fn metrics(&self) -> &[MetricRow] {
        &self.metrics
    }
    fn actor_data(&self) -> Option<&ActorSideData> {
        self.actor_data.as_ref()
    }
    fn item_data(&self) -> Option<&ItemSideData> {
        self.item_data.as_ref()
    }
    fn spell_data(&self) -> Option<&SpellSideData> {
        self.spell_data.as_ref()
    }
    fn publication_title(&self) -> Option<&str> {
        self.publication_title.as_deref()
    }
    fn description(&self) -> Option<&ContentDocument> {
        self.description.as_ref()
    }
    fn blurb(&self) -> Option<&ContentDocument> {
        self.blurb.as_ref()
    }
    fn supplemental_content(&self) -> &[SupplementalContentDocument] {
        &self.supplemental_content
    }
    fn publication_family(&self) -> PublicationFamily {
        self.publication_family
    }
    fn taxonomy_families(&self) -> &[String] {
        &self.taxonomy_families
    }
}

pub fn build_record_presentation_document(
    record: &(impl RecordPresentationSource + ?Sized),
) -> crate::RecordPresentationDocument {
    build_record_presentation_document_with_content_filter(record, |_| true)
}

pub fn build_record_presentation_document_with_content_filter(
    record: &(impl RecordPresentationSource + ?Sized),
    include_supplemental_content: impl Fn(&SupplementalContentDocument) -> bool + Copy,
) -> crate::RecordPresentationDocument {
    let mut document = crate::RecordPresentationDocument {
        record_key: record.key().clone(),
        record_family: record.record_family(),
        title: record.name().to_string(),
        identity: identity_facts(record),
        badges: badges(record),
        sections: recipe_sections(record, include_supplemental_content),
    };
    prune_empty_sections(&mut document.sections);
    document
}

fn recipe_sections(
    record: &(impl RecordPresentationSource + ?Sized),
    include_supplemental_content: impl Fn(&SupplementalContentDocument) -> bool + Copy,
) -> Vec<PresentationSection> {
    match record.record_family() {
        RecordFamily::Spell => spell_sections(record, include_supplemental_content),
        RecordFamily::Creature => creature_sections(record, include_supplemental_content),
        RecordFamily::Equipment => equipment_sections(record, include_supplemental_content),
        RecordFamily::Hazard => hazard_sections(record, include_supplemental_content),
        RecordFamily::Feat | RecordFamily::Rule => {
            feat_action_sections(record, include_supplemental_content)
        }
        _ => fallback_sections(record, include_supplemental_content),
    }
}

fn spell_sections(
    record: &(impl RecordPresentationSource + ?Sized),
    include_supplemental_content: impl Fn(&SupplementalContentDocument) -> bool + Copy,
) -> Vec<PresentationSection> {
    vec![
        fact_section(
            PresentationSectionKind::Summary,
            spell_summary_facts(record, record.spell_data()),
        ),
        description_section(record, include_supplemental_content),
        details_section(record),
    ]
}

fn creature_sections(
    record: &(impl RecordPresentationSource + ?Sized),
    include_supplemental_content: impl Fn(&SupplementalContentDocument) -> bool + Copy,
) -> Vec<PresentationSection> {
    vec![
        fact_section(
            PresentationSectionKind::Summary,
            creature_summary_facts(record.actor_data(), record.metrics()),
        ),
        fact_section(
            PresentationSectionKind::Defense,
            creature_defense_facts(record.actor_data(), record.metrics()),
        ),
        fact_section(
            PresentationSectionKind::Movement,
            creature_movement_facts(record.actor_data(), record.metrics()),
        ),
        fact_section(
            PresentationSectionKind::Offense,
            creature_offense_facts(record, record.spell_data()),
        ),
        description_section(record, include_supplemental_content),
        details_section(record),
    ]
}

fn equipment_sections(
    record: &(impl RecordPresentationSource + ?Sized),
    include_supplemental_content: impl Fn(&SupplementalContentDocument) -> bool + Copy,
) -> Vec<PresentationSection> {
    vec![
        fact_section(
            PresentationSectionKind::Summary,
            equipment_summary_facts(record, record.item_data()),
        ),
        description_section(record, include_supplemental_content),
        details_section(record),
    ]
}

fn hazard_sections(
    record: &(impl RecordPresentationSource + ?Sized),
    include_supplemental_content: impl Fn(&SupplementalContentDocument) -> bool + Copy,
) -> Vec<PresentationSection> {
    vec![
        fact_section(
            PresentationSectionKind::Summary,
            hazard_summary_facts(record.actor_data(), record.metrics()),
        ),
        fact_section(
            PresentationSectionKind::Defense,
            creature_defense_facts(record.actor_data(), record.metrics()),
        ),
        fact_section(
            PresentationSectionKind::Routine,
            hazard_routine_facts(record, record.actor_data(), record.spell_data()),
        ),
        description_section(record, include_supplemental_content),
        details_section(record),
    ]
}

fn feat_action_sections(
    record: &(impl RecordPresentationSource + ?Sized),
    include_supplemental_content: impl Fn(&SupplementalContentDocument) -> bool + Copy,
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
    record: &(impl RecordPresentationSource + ?Sized),
    include_supplemental_content: impl Fn(&SupplementalContentDocument) -> bool + Copy,
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

fn identity_facts(record: &(impl RecordPresentationSource + ?Sized)) -> Vec<PresentationFact> {
    [
        fact(
            "family",
            "Family",
            Some(humanize(record.record_family().as_str())),
        ),
        fact("type", "Type", Some(humanize(record.foundry_record_type()))),
        fact(
            "level",
            if record.record_family() == RecordFamily::Spell {
                "Rank"
            } else {
                "Level"
            },
            record.level().map(|value| value.to_string()),
        ),
        fact("rarity", "Rarity", record.rarity().map(humanize)),
        fact(
            "publication",
            "Publication",
            record.publication_title().map(ToString::to_string),
        ),
    ]
    .into_iter()
    .flatten()
    .collect()
}

fn badges(record: &(impl RecordPresentationSource + ?Sized)) -> Vec<PresentationBadge> {
    record
        .traits()
        .iter()
        .map(|value| PresentationBadge {
            kind: PresentationBadgeKind::Trait,
            label: "Trait".to_string(),
            value: humanize(value),
        })
        .chain(
            record
                .taxonomy_families()
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
    record: &(impl RecordPresentationSource + ?Sized),
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
        push_fact(
            &mut facts,
            "duration",
            "Duration",
            duration_text(record.duration()),
        );
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
        push_fact(
            &mut facts,
            "duration",
            "Duration",
            duration_text(record.duration()),
        );
    }
    push_fact(
        &mut facts,
        "cast",
        "Cast",
        activation_text(record.activation_time(), record.system_actions_value()),
    );
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
    record: &(impl RecordPresentationSource + ?Sized),
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
        action_count_text(record.system_actions_value()),
    );
    facts
}

fn equipment_summary_facts(
    record: &(impl RecordPresentationSource + ?Sized),
    item: Option<&ItemSideData>,
) -> Vec<PresentationFact> {
    let mut facts = Vec::new();
    let price_cp = item.and_then(|item| item.price_cp).or(record.price_cp());
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
        activation_text(record.activation_time(), record.system_actions_value()),
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
    record: &(impl RecordPresentationSource + ?Sized),
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
    push_fact(
        &mut facts,
        "duration",
        "Duration",
        duration_text(record.duration()),
    );
    facts
}

fn action_summary_facts(
    record: &(impl RecordPresentationSource + ?Sized),
) -> Vec<PresentationFact> {
    let mut facts = Vec::new();
    push_fact(
        &mut facts,
        "prerequisites",
        "Prerequisites",
        format_prerequisites(record.prerequisites()),
    );
    push_fact(
        &mut facts,
        "actionCost",
        "Action Cost",
        activation_text(record.activation_time(), record.system_actions_value()),
    );
    if let Some(spell) = record.spell_data() {
        push_fact(&mut facts, "range", "Range", spell.range_text.clone());
        push_fact(&mut facts, "area", "Area", format_area(spell));
        push_fact(&mut facts, "save", "Save", format_save(spell));
        push_fact(
            &mut facts,
            "duration",
            "Duration",
            duration_text(record.duration()),
        );
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

fn format_prerequisites(values: &[String]) -> Option<String> {
    let values = values
        .iter()
        .map(|value| value.trim())
        .filter(|value| !value.is_empty())
        .collect::<Vec<_>>();
    (!values.is_empty()).then(|| values.join(", "))
}

fn fallback_summary_facts(
    record: &(impl RecordPresentationSource + ?Sized),
) -> Vec<PresentationFact> {
    let mut facts = Vec::new();
    push_fact(
        &mut facts,
        "actionCost",
        "Action Cost",
        activation_text(record.activation_time(), record.system_actions_value()),
    );
    push_fact(
        &mut facts,
        "duration",
        "Duration",
        duration_text(record.duration()),
    );
    facts
}

fn details_section(record: &(impl RecordPresentationSource + ?Sized)) -> PresentationSection {
    let facts = [
        fact(
            "taxonomyFamilies",
            "Families",
            format_list(record.taxonomy_families()),
        ),
        fact(
            "publicationFamily",
            "Publication Family",
            Some(humanize(record.publication_family().as_str())),
        ),
        fact("pack", "Pack", Some(record.pack_name().to_string())),
        fact(
            "foundryDocumentType",
            "Foundry Document Type",
            Some(record.foundry_document_type().to_string()),
        ),
    ]
    .into_iter()
    .flatten()
    .collect();
    fact_section(PresentationSectionKind::Details, facts)
}

fn description_section(
    record: &(impl RecordPresentationSource + ?Sized),
    include_supplemental_content: impl Fn(&SupplementalContentDocument) -> bool + Copy,
) -> PresentationSection {
    let mut blocks = Vec::new();
    if let Some(document) = record.description() {
        blocks.push(PresentationBlock::Content(document.clone()));
    } else if let Some(document) = record.blurb() {
        blocks.push(PresentationBlock::Content(document.clone()));
    }
    blocks.extend(
        record
            .supplemental_content()
            .iter()
            .filter(|content| {
                content.contributes_to_search && include_supplemental_content(content)
            })
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
