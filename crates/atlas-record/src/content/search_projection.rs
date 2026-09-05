use atlas_domain::TimeKind;

use crate::{
    AtlasRecord, CreatureActionCost, CreatureDamageKind, CreatureNumber, CreatureRecord,
    CreatureResourceAmount, CreatureSourceScalar, FactValue, MechanicActivityFamily,
    MechanicBaseValue, MechanicFact, MechanicSourceFamily, MechanicSurface, PresentationBlock,
    PresentationFact, PresentationSection, PresentationSectionKind, RecordBody,
    RecordContentDocument, RecordPresentationDocument,
    build_record_presentation_document_with_content_filter, label_for_row,
    presentation_recipe::searchable_content_sections, project_creature_facts,
    project_creature_mechanics,
};

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct RecordFtsProjection {
    pub title: String,
    pub aliases: String,
    pub traits: String,
    pub taxonomy_terms: String,
    pub constraint_terms: String,
    pub mechanic_terms: String,
    pub source_terms: String,
    pub metric_terms: String,
    pub headings: String,
    pub body: String,
    pub facts: String,
    pub references: String,
    pub embedded_content: String,
}

pub fn build_record_fts_projection(
    record: &AtlasRecord,
    aliases: &[String],
) -> RecordFtsProjection {
    build_search_fts_projection(record, aliases, None)
}

pub fn build_search_fts_projection(
    record: &AtlasRecord,
    aliases: &[String],
    canonical_body: Option<&RecordBody>,
) -> RecordFtsProjection {
    let mut projection = RecordFtsProjection {
        title: record.identity.name.clone(),
        aliases: aliases.join("\n"),
        traits: record.classification.traits.join(" "),
        ..RecordFtsProjection::default()
    };
    let canonical_creature = canonical_creature_for_record(record, canonical_body);
    append_structured_terms(record, canonical_creature, &mut projection);

    projection
}

pub fn build_search_presentation_document_with_content_filter(
    record: &AtlasRecord,
    canonical_body: Option<&RecordBody>,
    include_supplemental_content: impl Fn(&RecordContentDocument) -> bool + Copy,
) -> RecordPresentationDocument {
    let mut document = build_record_presentation_document_with_content_filter(
        record,
        include_supplemental_content,
    );
    let Some(creature) = canonical_creature_for_record(record, canonical_body) else {
        return document;
    };

    document.sections.retain(|section| {
        !matches!(
            section.kind,
            PresentationSectionKind::Summary
                | PresentationSectionKind::Defense
                | PresentationSectionKind::Movement
                | PresentationSectionKind::Offense
        )
    });
    let mut canonical_sections = canonical_mechanics_search_projection(creature).sections;
    canonical_sections.extend(searchable_content_sections(
        record,
        include_supplemental_content,
    ));
    document.sections = canonical_sections;
    document
}

fn canonical_creature_for_record<'a>(
    record: &AtlasRecord,
    canonical_body: Option<&'a RecordBody>,
) -> Option<&'a CreatureRecord> {
    let RecordBody::Creature(creature) = canonical_body?;
    (creature.identity.record_key == record.identity.key).then_some(creature)
}

fn append_structured_terms(
    record: &AtlasRecord,
    canonical_creature: Option<&CreatureRecord>,
    projection: &mut RecordFtsProjection,
) {
    let generic_mechanics = (record.classification.kind != atlas_domain::RecordKind::Creature)
        .then_some(&record.mechanics);
    let mut taxonomy = TermCollector::default();
    taxonomy.add_slug(record.classification.kind.as_str());
    taxonomy.add_slug(record.foundry.record_type.as_str());
    taxonomy.add_slugs(&record.classification.taxonomy.inferred_groups);
    if let Some(item) = generic_mechanics.and_then(crate::RecordMechanics::item) {
        taxonomy.add_optional_slug(item.category.as_deref());
        taxonomy.add_optional_slug(item.group.as_deref());
        taxonomy.add_optional_slug(item.base_item.as_deref());
    }
    if let Some(spell) = generic_mechanics.and_then(crate::RecordMechanics::spell) {
        taxonomy.add_slugs(&spell.kinds);
    }
    if let Some(variant) = record.variant.as_ref() {
        taxonomy.add_text(&variant.base_name);
        taxonomy.add_optional_text(variant.label.as_deref());
        taxonomy.add_slugs(&variant.axes);
    }
    append_text(&mut projection.taxonomy_terms, &taxonomy.render());

    let mut constraints = TermCollector::default();
    constraints.add_texts(&record.requirements.prerequisites);
    if let Some(time) = record.timing.activation_time() {
        constraints.add_text(&time.text);
        append_action_cost_terms(time.kind, time.actions, &mut constraints);
    }
    if let Some(item) = generic_mechanics.and_then(crate::RecordMechanics::item) {
        constraints.add_optional_text(item.hands_requirement.as_deref());
    }
    append_text(&mut projection.constraint_terms, &constraints.render());

    let mut mechanics = TermCollector::default();
    if let Some(level) = record.classification.level {
        mechanics.add_text(&format!("level {level}"));
        mechanics.add_text(&ordinal_phrase(level, "level"));
        if record.classification.kind == atlas_domain::RecordKind::Spell {
            mechanics.add_text(&format!("rank {level}"));
            mechanics.add_text(&ordinal_phrase(level, "rank"));
        }
    }
    mechanics.add_optional_slug(record.classification.rarity.map(|rarity| rarity.as_str()));
    if let Some(duration) = record.timing.duration_time() {
        mechanics.add_text(&duration.text);
    }
    if let Some(spell) = generic_mechanics.and_then(crate::RecordMechanics::spell) {
        mechanics.add_slugs(&spell.traditions);
        mechanics.add_optional_text(spell.range.as_ref().map(|range| range.text.as_str()));
        mechanics.add_optional_text(spell.target.as_ref().map(|target| target.text.as_str()));
        mechanics.add_optional_slug(
            spell
                .area
                .as_ref()
                .and_then(|area| area.kind.as_ref())
                .map(String::as_str),
        );
        mechanics.add_optional_slug(
            spell
                .defense
                .as_ref()
                .and_then(|defense| defense.save.as_ref())
                .map(String::as_str),
        );
        mechanics.add_slugs(&spell.damage_types);
        if spell.sustained {
            mechanics.add_text("sustained");
        }
        if spell.defense.as_ref().is_some_and(|defense| defense.basic) {
            mechanics.add_text("basic save");
        }
    }
    if let Some(item) = generic_mechanics.and_then(crate::RecordMechanics::item) {
        mechanics.add_optional_text(item.usage.as_deref());
        mechanics.add_slugs(&item.damage_types);
    }
    if let Some(actor) = generic_mechanics.and_then(crate::RecordMechanics::actor) {
        mechanics.add_optional_slug(actor.size.as_deref());
        mechanics.add_slugs(&actor.languages);
        mechanics.add_slugs(&actor.speed_types);
        mechanics.add_slugs(&actor.senses);
        mechanics.add_slugs(&actor.immunities);
        mechanics.add_slugs(&actor.resistances);
        mechanics.add_slugs(&actor.weaknesses);
        mechanics.add_slugs(&actor.disable_skills);
        if actor.is_complex {
            mechanics.add_text("complex");
        }
    }
    let mut source = TermCollector::default();
    source.add_optional_text(record.publication.title.as_deref());
    if record.publication.category != atlas_domain::PublicationCategory::Unknown {
        source.add_slug(record.publication.category.as_str());
    }
    source.add_text(&record.foundry.pack_label);
    append_text(&mut projection.source_terms, &source.render());

    let mut metrics = TermCollector::default();
    if let Some(creature) = canonical_creature {
        let canonical = canonical_mechanics_search_projection(creature);
        mechanics.add_text(&canonical.mechanic_terms);
        metrics.add_text(&canonical.metric_terms);
    } else if let Some(generic_mechanics) = generic_mechanics {
        for metric in &generic_mechanics.metrics {
            let label = label_for_row(metric);
            metrics.add_text(&label.label);
            if let Some(short_label) = label.short_label.as_deref() {
                metrics.add_text(short_label);
            }
        }
    }
    append_text(&mut projection.mechanic_terms, &mechanics.render());
    append_text(&mut projection.metric_terms, &metrics.render());
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct CanonicalMechanicsSearchProjection {
    mechanic_terms: String,
    metric_terms: String,
    sections: Vec<PresentationSection>,
}

fn canonical_mechanics_search_projection(
    creature: &CreatureRecord,
) -> CanonicalMechanicsSearchProjection {
    let mechanics = project_creature_mechanics(creature);
    let mut mechanic_terms = TermCollector::default();
    let mut metric_terms = TermCollector::default();
    let mut summary = Vec::new();
    let mut defense = Vec::new();
    let mut movement = Vec::new();
    let mut offense = Vec::new();

    if let FactValue::Value(level) = mechanics.level {
        mechanic_terms.add_text(&format!("level {level}"));
        mechanic_terms.add_text(&ordinal_phrase(level, "level"));
        summary.push(PresentationFact {
            key: "level".to_string(),
            label: "Level".to_string(),
            value: level.to_string(),
        });
    }

    let canonical_facts = project_creature_facts(creature);
    for metric in &canonical_facts.metrics {
        let label = label_for_row(metric);
        metric_terms.add_text(&label.label);
        metric_terms.add_optional_text(label.short_label.as_deref());
    }
    let size = creature
        .size
        .value
        .as_value()
        .map(|value| value.as_source());
    let languages = creature
        .languages
        .value
        .as_value()
        .and_then(|value| value.values.as_value())
        .map(|values| {
            values
                .iter()
                .map(|value| value.as_str().to_string())
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();
    let speed_types = creature
        .movement
        .value
        .as_value()
        .map(|values| {
            values
                .iter()
                .filter_map(|value| movement_mode_slug(&value.mode))
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();
    let senses = creature
        .perception
        .value
        .as_value()
        .and_then(|value| value.senses.as_value())
        .map(|values| {
            values
                .iter()
                .map(|value| value.sense_type.as_str().to_string())
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();
    let defenses = creature.defenses.value.as_value();
    let immunities = canonical_iwr_terms(defenses.and_then(|value| value.immunities.as_value()));
    let resistances = canonical_iwr_terms(defenses.and_then(|value| value.resistances.as_value()));
    let weaknesses = canonical_iwr_terms(defenses.and_then(|value| value.weaknesses.as_value()));
    mechanic_terms.add_optional_slug(size);
    mechanic_terms.add_slugs(&languages);
    mechanic_terms.add_slugs(&speed_types);
    mechanic_terms.add_slugs(&senses);
    mechanic_terms.add_slugs(&immunities);
    mechanic_terms.add_slugs(&resistances);
    mechanic_terms.add_slugs(&weaknesses);
    push_canonical_list_fact(&mut summary, "actor.languages", "Languages", &languages);
    push_canonical_list_fact(&mut summary, "actor.senses", "Senses", &senses);
    push_canonical_list_fact(&mut defense, "actor.immunities", "Immunities", &immunities);
    push_canonical_list_fact(
        &mut defense,
        "actor.resistances",
        "Resistances",
        &resistances,
    );
    push_canonical_list_fact(&mut defense, "actor.weaknesses", "Weaknesses", &weaknesses);

    for fact in &mechanics.facts {
        add_canonical_fact_terms(fact, None, &mut mechanic_terms, &mut metric_terms);
        if let Some(presentation_fact) = canonical_presentation_fact(fact, None) {
            match presentation_section_kind(fact.facets.family) {
                PresentationSectionKind::Defense => defense.push(presentation_fact),
                PresentationSectionKind::Movement => movement.push(presentation_fact),
                PresentationSectionKind::Offense => offense.push(presentation_fact),
                _ => summary.push(presentation_fact),
            }
        }
    }

    for activity in &mechanics.activities {
        let family = activity_family_label(activity.family);
        mechanic_terms.add_text(family);
        mechanic_terms.add_text(&activity.label);
        metric_terms.add_text(&activity.label);
        offense.push(PresentationFact {
            key: format!("activity.{}", activity.occurrence_id.as_str()),
            label: family.to_string(),
            value: activity.label.clone(),
        });
        for fact in &activity.facts {
            add_canonical_fact_terms(
                fact,
                Some(&activity.label),
                &mut mechanic_terms,
                &mut metric_terms,
            );
            if let Some(fact) = canonical_presentation_fact(fact, Some(&activity.label)) {
                offense.push(fact);
            }
        }
    }

    CanonicalMechanicsSearchProjection {
        mechanic_terms: mechanic_terms.render(),
        metric_terms: metric_terms.render(),
        sections: [
            (PresentationSectionKind::Summary, summary),
            (PresentationSectionKind::Defense, defense),
            (PresentationSectionKind::Movement, movement),
            (PresentationSectionKind::Offense, offense),
        ]
        .into_iter()
        .filter_map(|(kind, facts)| {
            (!facts.is_empty())
                .then(|| PresentationSection::new(kind, vec![PresentationBlock::FactList(facts)]))
        })
        .collect(),
    }
}

fn canonical_iwr_terms(entries: Option<&Vec<crate::CreatureIwr>>) -> Vec<String> {
    entries
        .into_iter()
        .flatten()
        .map(|entry| entry.iwr_type.as_str().to_string())
        .collect()
}

fn movement_mode_slug(mode: &crate::CreatureMovementMode) -> Option<String> {
    Some(match mode {
        crate::CreatureMovementMode::Land => "land".to_string(),
        crate::CreatureMovementMode::Burrow => "burrow".to_string(),
        crate::CreatureMovementMode::Climb => "climb".to_string(),
        crate::CreatureMovementMode::Fly => "fly".to_string(),
        crate::CreatureMovementMode::Swim => "swim".to_string(),
        crate::CreatureMovementMode::Unsupported(_) => return None,
    })
}

fn push_canonical_list_fact(
    facts: &mut Vec<PresentationFact>,
    key: &str,
    label: &str,
    values: &[String],
) {
    if !values.is_empty() {
        facts.push(PresentationFact {
            key: key.to_string(),
            label: label.to_string(),
            value: values.join(", "),
        });
    }
}

fn presentation_section_kind(family: MechanicSourceFamily) -> PresentationSectionKind {
    match family {
        MechanicSourceFamily::Defense => PresentationSectionKind::Defense,
        MechanicSourceFamily::Movement => PresentationSectionKind::Movement,
        MechanicSourceFamily::Strike
        | MechanicSourceFamily::Spellcasting
        | MechanicSourceFamily::Spell
        | MechanicSourceFamily::Action => PresentationSectionKind::Offense,
        MechanicSourceFamily::Awareness
        | MechanicSourceFamily::Skill
        | MechanicSourceFamily::Ability
        | MechanicSourceFamily::Resource
        | MechanicSourceFamily::Unsupported => PresentationSectionKind::Summary,
    }
}

fn add_canonical_fact_terms(
    fact: &MechanicFact,
    activity_label: Option<&str>,
    mechanic_terms: &mut TermCollector,
    metric_terms: &mut TermCollector,
) {
    if let Some(activity_label) = activity_label {
        mechanic_terms.add_text(activity_label);
    }
    mechanic_terms.add_text(&fact.label);
    mechanic_terms.add_text(&fact.target.id());
    mechanic_terms.add_text(mechanic_family_label(fact.facets.family));
    mechanic_terms.add_text(mechanic_surface_label(fact.facets.surface));
    metric_terms.add_text(&fact.label);
    metric_terms.add_text(&humanize_slug(&fact.target.id()));
    if let Some(value) = canonical_mechanic_value_text(&fact.value) {
        mechanic_terms.add_text(&format!("{} {value}", fact.label));
    }
}

fn canonical_presentation_fact(
    fact: &MechanicFact,
    activity_label: Option<&str>,
) -> Option<PresentationFact> {
    let value = canonical_mechanic_value_text(&fact.value)?;
    Some(PresentationFact {
        key: fact.target.id(),
        label: activity_label
            .map(|activity| format!("{activity} {}", fact.label))
            .unwrap_or_else(|| fact.label.clone()),
        value,
    })
}

fn canonical_mechanic_value_text(value: &MechanicBaseValue) -> Option<String> {
    match value {
        MechanicBaseValue::Integer(value) => fact_integer(value),
        MechanicBaseValue::Number(value) => value.as_value().and_then(|value| match value {
            CreatureNumber::Integer(value) => Some(value.to_string()),
            CreatureNumber::Unsupported(_) => None,
        }),
        MechanicBaseValue::ResourceAmount(value) => {
            value.as_value().and_then(|value| match value {
                CreatureResourceAmount::Integer(value) => Some(value.to_string()),
                CreatureResourceAmount::Unsupported(_) => None,
            })
        }
        MechanicBaseValue::ActionCost(value) => action_cost_text(value),
        MechanicBaseValue::Frequency(value) => value.as_value().and_then(|frequency| {
            let maximum = frequency.maximum.as_value()?;
            let period = frequency.period.as_value();
            Some(match period {
                Some(period) => format!("{maximum} per {period}"),
                None => maximum.to_string(),
            })
        }),
        MechanicBaseValue::Uses(value) => value
            .as_value()
            .and_then(|uses| uses.maximum.as_value())
            .map(ToString::to_string),
        MechanicBaseValue::Roll(value) => value
            .value
            .as_value()
            .map(|modifier| format_modifier(*modifier)),
        MechanicBaseValue::Damage(value) => damage_text(value),
        MechanicBaseValue::SourceInteger(value) => value.as_value().and_then(|value| match value {
            CreatureSourceScalar::Value(value) => Some(value.to_string()),
            CreatureSourceScalar::Unsupported(_) => None,
        }),
    }
}

fn fact_integer(value: &FactValue<i64>) -> Option<String> {
    value.as_value().map(ToString::to_string)
}

fn action_cost_text(value: &CreatureActionCost) -> Option<String> {
    match value {
        CreatureActionCost::Passive => Some("passive".to_string()),
        CreatureActionCost::Reaction => Some("reaction".to_string()),
        CreatureActionCost::FreeAction => Some("free action".to_string()),
        CreatureActionCost::Actions(actions) => Some(if *actions == 1 {
            "1 action".to_string()
        } else {
            format!("{actions} actions")
        }),
        CreatureActionCost::Time(value) => Some(value.clone()),
        CreatureActionCost::Unsupported(_) => None,
    }
}

fn damage_text(value: &crate::CreatureDamage) -> Option<String> {
    let mut parts = Vec::new();
    if let Some(formula) = value.formula.as_value() {
        parts.push(formula.clone());
    }
    if let Some(damage_type) = value.damage_type.as_value() {
        parts.push(humanize_slug(damage_type));
    }
    if let Some(category) = value.category.as_value() {
        parts.push(humanize_slug(category));
    }
    if let Some(kinds) = value.kinds.as_value() {
        parts.extend(kinds.iter().filter_map(|kind| match kind {
            CreatureDamageKind::Damage => Some("damage".to_string()),
            CreatureDamageKind::Healing => Some("healing".to_string()),
            CreatureDamageKind::Unsupported(_) => None,
        }));
    }
    (!parts.is_empty()).then(|| parts.join(" "))
}

fn format_modifier(value: i64) -> String {
    if value >= 0 {
        format!("+{value}")
    } else {
        value.to_string()
    }
}

fn activity_family_label(family: MechanicActivityFamily) -> &'static str {
    match family {
        MechanicActivityFamily::Strike => "Strike",
        MechanicActivityFamily::SpellcastingEntry => "Spellcasting",
        MechanicActivityFamily::Spell => "Spell",
        MechanicActivityFamily::Action => "Action",
        MechanicActivityFamily::Unsupported => "Unsupported activity",
    }
}

fn mechanic_family_label(family: MechanicSourceFamily) -> &'static str {
    match family {
        MechanicSourceFamily::Defense => "defense",
        MechanicSourceFamily::Awareness => "awareness",
        MechanicSourceFamily::Skill => "skill",
        MechanicSourceFamily::Ability => "ability",
        MechanicSourceFamily::Movement => "movement",
        MechanicSourceFamily::Resource => "resource",
        MechanicSourceFamily::Strike => "strike",
        MechanicSourceFamily::Spellcasting => "spellcasting",
        MechanicSourceFamily::Spell => "spell",
        MechanicSourceFamily::Action => "action",
        MechanicSourceFamily::Unsupported => "unsupported",
    }
}

fn mechanic_surface_label(surface: MechanicSurface) -> &'static str {
    match surface {
        MechanicSurface::RawModifier => "modifier",
        MechanicSurface::Check => "check",
        MechanicSurface::Dc => "difficulty class",
        MechanicSurface::ArmorClass => "armor class",
        MechanicSurface::SavingThrow => "saving throw",
        MechanicSurface::AttackRoll => "attack roll",
        MechanicSurface::Damage => "damage",
        MechanicSurface::HitPoints => "hit points",
        MechanicSurface::Movement => "speed",
        MechanicSurface::Resource => "resource maximum",
        MechanicSurface::ActionEconomy => "action economy",
        MechanicSurface::Frequency => "frequency",
        MechanicSurface::Uses => "uses",
        MechanicSurface::SpellSlot => "spell slot",
    }
}

fn append_action_cost_terms(kind: TimeKind, actions: Option<i64>, terms: &mut TermCollector) {
    match kind {
        TimeKind::Actions => {
            if let Some(actions) = actions {
                terms.add_text(&format!("{actions} action"));
                terms.add_text(&format!("{actions} actions"));
                if let Some(word) = action_count_word(actions) {
                    terms.add_text(&format!("{word} action"));
                    terms.add_text(&format!("{word} actions"));
                }
            }
        }
        TimeKind::Free => terms.add_text("free action"),
        TimeKind::Reaction => terms.add_text("reaction"),
        TimeKind::Duration | TimeKind::Variable | TimeKind::Other => {}
    }
}

fn action_count_word(actions: i64) -> Option<&'static str> {
    match actions {
        1 => Some("one"),
        2 => Some("two"),
        3 => Some("three"),
        _ => None,
    }
}

fn ordinal_phrase(value: i64, noun: &str) -> String {
    let suffix = match value % 100 {
        11..=13 => "th",
        _ => match value % 10 {
            1 => "st",
            2 => "nd",
            3 => "rd",
            _ => "th",
        },
    };
    format!("{value}{suffix} {noun}")
}

#[derive(Default)]
struct TermCollector {
    terms: Vec<String>,
}

impl TermCollector {
    fn add_optional_text(&mut self, value: Option<&str>) {
        if let Some(value) = value {
            self.add_text(value);
        }
    }

    fn add_optional_slug(&mut self, value: Option<&str>) {
        if let Some(value) = value {
            self.add_slug(value);
        }
    }

    fn add_texts(&mut self, values: &[String]) {
        for value in values {
            self.add_text(value);
        }
    }

    fn add_slugs(&mut self, values: &[String]) {
        for value in values {
            self.add_slug(value);
        }
    }

    fn add_slug(&mut self, value: &str) {
        self.add_text(value);
        let humanized = humanize_slug(value);
        if humanized != value {
            self.add_text(&humanized);
        }
    }

    fn add_text(&mut self, value: &str) {
        let value = value.trim();
        if value.is_empty() || self.terms.iter().any(|term| term == value) {
            return;
        }
        self.terms.push(value.to_string());
    }

    fn render(&self) -> String {
        self.terms.join("\n")
    }
}

fn humanize_slug(value: &str) -> String {
    value
        .replace(['_', '-'], " ")
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
}

fn append_text(target: &mut String, value: &str) {
    let value = value.trim();
    if value.is_empty() {
        return;
    }
    if !target.is_empty() {
        target.push('\n');
    }
    target.push_str(value);
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use atlas_domain::{
        MetricDomain, PackName, PublicationCategory, RecordId, RecordKey, RecordKind, TimeKind,
    };

    use crate::{
        ActivationTimeSourceField, ActorMechanics, ContentSourceKind, DurationTimeSourceField,
        FoundryDocumentMechanics, FoundryDocumentType, FoundryRecordInfo, FoundryRecordType,
        ItemMechanics, ItemTypeMechanics, MetricRow, MetricValue, NormalizedTime,
        RecordActivationTiming, RecordClassification, RecordContent, RecordContentDocument,
        RecordDurationTiming, RecordIdentity, RecordMechanics, RecordProvenance, RecordPublication,
        RecordRequirements, RecordTaxonomy, RecordTiming, RecordVisibility, RichDocument, RichNode,
        SpellArea, SpellDefense, SpellMechanics, SpellRange, SpellTarget,
    };

    use super::*;

    #[test]
    fn fts_projection_splits_body_facts_and_embedded_content() {
        let mut record = base_record();
        record.content.documents.push(RecordContentDocument {
            source_kind: ContentSourceKind::Description,
            label: None,
            document: RichDocument::new(vec![
                html_element("h2", vec![text_node("Effect")]),
                html_element("p", vec![text_node("Main body")]),
            ]),
        });
        record.content.documents.push(RecordContentDocument {
            source_kind: ContentSourceKind::Disable,
            label: None,
            document: text_document("Disable text"),
        });
        record.content.documents.push(RecordContentDocument {
            source_kind: ContentSourceKind::EmbeddedItemDescription,
            label: Some("Jaws".to_string()),
            document: text_document("Embedded attack text"),
        });

        let projection = build_record_fts_projection(&record, &["Alias".to_string()]);

        assert_eq!(projection.title, "Test Record");
        assert_eq!(projection.aliases, "Alias");
        assert_eq!(projection.traits, "healing vitality");
        assert_eq!(projection.headings, "");
        assert_eq!(projection.body, "");
        assert_eq!(projection.facts, "");
        assert_eq!(projection.embedded_content, "");
    }

    #[test]
    fn creature_without_canonical_body_cannot_fall_back_to_generic_mechanics() {
        let mut record = base_record();
        record.classification.kind = RecordKind::Creature;
        record.mechanics.metrics.push(MetricRow {
            domain: MetricDomain::Actor,
            key: "speed.fly.value".to_string(),
            value: MetricValue::Number(60.0),
        });
        record.mechanics.document = FoundryDocumentMechanics::Actor(ActorMechanics {
            size: Some("lg".to_string()),
            senses: vec!["darkvision".to_string()],
            ..ActorMechanics::default()
        });

        let projection = build_record_fts_projection(&record, &[]);

        assert!(!projection.mechanic_terms.contains("darkvision"));
        assert!(!projection.mechanic_terms.contains("lg"));
        assert!(!projection.metric_terms.contains("Fly Speed"));
    }

    #[test]
    fn fts_projection_adds_deterministic_structured_terms_without_duplicating_traits() {
        let mut record = base_record();
        record.classification.kind = RecordKind::Spell;
        record.foundry.record_type = FoundryRecordType::Spell;
        record.classification.level = Some(2);
        record.classification.rarity = Some(atlas_domain::Rarity::Uncommon);
        record.requirements.prerequisites = vec!["expert in Medicine".to_string()];
        record.timing.activation = Some(RecordActivationTiming {
            time: NormalizedTime {
                kind: TimeKind::Actions,
                actions: Some(2),
                duration_value: None,
                duration_unit: None,
                text: "2".to_string(),
            },
            source_field: ActivationTimeSourceField::ActionsValue,
        });
        record.timing.duration = Some(RecordDurationTiming {
            time: NormalizedTime {
                kind: TimeKind::Duration,
                actions: None,
                duration_value: None,
                duration_unit: None,
                text: "1 minute".to_string(),
            },
            source_field: DurationTimeSourceField::DurationValue,
        });
        record.classification.taxonomy.inferred_groups = vec!["focus_spell".to_string()];
        record.identity.key = RecordKey::new(
            PackName::new("internal-spell-pack").expect("pack parses"),
            RecordId::new("TestRecord").expect("id parses"),
        );
        record.publication.title = Some("Pathfinder Player Core".to_string());
        record.publication.category = PublicationCategory::Core;
        record.foundry.pack_label = "Spells".to_string();
        record.mechanics.document = FoundryDocumentMechanics::Item(ItemMechanics {
            foundry_type: Some(ItemTypeMechanics::Spell(SpellMechanics {
                traditions: vec!["divine".to_string()],
                kinds: vec!["focus".to_string()],
                range: Some(SpellRange {
                    text: "30 feet".to_string(),
                    distance: Some(30.0),
                }),
                target: Some(SpellTarget {
                    text: "1 ally".to_string(),
                }),
                area: Some(SpellArea {
                    kind: Some("burst".to_string()),
                    value: None,
                }),
                defense: Some(SpellDefense {
                    save: Some("will".to_string()),
                    basic: true,
                }),
                sustained: true,
                damage_types: vec!["vitality".to_string()],
            })),
            category: Some("weapon".to_string()),
            base_item: Some("longsword".to_string()),
            group: Some("sword".to_string()),
            usage: Some("held in one hand".to_string()),
            price_json: None,
            price_cp: None,
            bulk_value: None,
            hands_requirement: Some("one hand".to_string()),
            damage_types: vec!["slashing".to_string()],
        });
        record.mechanics.metrics = vec![MetricRow {
            domain: MetricDomain::Actor,
            key: "speed.fly.value".to_string(),
            value: MetricValue::Number(60.0),
        }];

        let projection = build_record_fts_projection(&record, &[]);

        assert_eq!(projection.traits, "healing vitality");
        assert!(projection.taxonomy_terms.contains("focus spell"));
        assert!(projection.taxonomy_terms.contains("weapon"));
        assert!(projection.constraint_terms.contains("expert in Medicine"));
        assert!(projection.constraint_terms.contains("two actions"));
        assert!(projection.constraint_terms.contains("one hand"));
        assert!(projection.mechanic_terms.contains("2nd rank"));
        assert!(projection.mechanic_terms.contains("uncommon"));
        assert!(projection.mechanic_terms.contains("basic save"));
        assert!(projection.source_terms.contains("Pathfinder Player Core"));
        assert!(projection.source_terms.contains("Spells"));
        assert!(!projection.source_terms.contains("internal-spell-pack"));
        assert!(projection.metric_terms.contains("Fly Speed"));
        assert!(!projection.metric_terms.contains("60"));
        assert!(!projection.mechanic_terms.contains("healing"));
        assert!(!projection.taxonomy_terms.contains("healing"));
    }

    fn base_record() -> AtlasRecord {
        AtlasRecord {
            identity: RecordIdentity {
                key: RecordKey::new(
                    PackName::new("test-pack").expect("pack parses"),
                    RecordId::new("TestRecord").expect("id parses"),
                ),
                name: "Test Record".to_string(),
            },
            classification: RecordClassification {
                kind: RecordKind::Spell,
                level: None,
                rarity: None,
                traits: vec!["healing".to_string(), "vitality".to_string()],
                taxonomy: RecordTaxonomy::default(),
            },
            foundry: FoundryRecordInfo {
                pack_label: "Test Pack".to_string(),
                document_type: FoundryDocumentType::Item,
                record_type: FoundryRecordType::Spell,
                folder_id: None,
            },
            provenance: RecordProvenance {
                source_path: "test.json".to_string(),
                raw_json: Some("{}".to_string()),
            },
            publication: RecordPublication {
                title: None,
                remaster: false,
                category: PublicationCategory::Unknown,
            },
            requirements: RecordRequirements::default(),
            timing: RecordTiming::default(),
            mechanics: RecordMechanics::default(),
            content: RecordContent::default(),
            variant: None,
            visibility: RecordVisibility::default(),
        }
    }

    fn text_document(text: &str) -> RichDocument {
        RichDocument::new(vec![html_element("p", vec![text_node(text)])])
    }

    fn html_element(tag: &str, children: Vec<RichNode>) -> RichNode {
        RichNode::HtmlElement {
            tag: tag.to_string(),
            attributes: BTreeMap::new(),
            children,
        }
    }

    fn text_node(text: &str) -> RichNode {
        RichNode::Text {
            text: text.to_string(),
        }
    }
}
