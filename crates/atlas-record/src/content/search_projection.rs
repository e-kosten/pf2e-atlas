use atlas_domain::{RecordKind, TimeKind};

use crate::{
    AtlasRecord, CreatureActionCost, CreatureDamageKind, CreatureNumber, CreatureRecord,
    CreatureResourceAmount, CreatureSourceScalar, FactValue, HazardRecord, MechanicActivityFamily,
    MechanicBaseValue, MechanicFact, MechanicSourceFamily, MechanicSurface, PresentationBadge,
    PresentationBadgeKind, PresentationBlock, PresentationFact, PresentationSection,
    PresentationSectionKind, RecordBody, RecordContentDocument, RecordPresentationDocument,
    RichLinkTarget, SpellHeightening, SpellRecord, SpellRule, SpellRulePredicate, SpellSourceValue,
    build_hazard_presentation_document, build_record_presentation_document_with_content_filter,
    label_for_row, presentation_recipe::searchable_content_sections, project_creature_facts,
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
    let canonical_hazard = canonical_hazard_for_record(record, canonical_body);
    let canonical_spell = canonical_spell_for_record(record, canonical_body);
    append_structured_terms(
        record,
        canonical_creature,
        canonical_hazard,
        canonical_spell,
        &mut projection,
    );
    if let Some(hazard) = canonical_hazard {
        append_hazard_owned_content(hazard, &mut projection);
    }
    if let Some(spell) = canonical_spell {
        append_spell_content_fts(spell, &mut projection);
    }

    projection
}

pub fn build_search_presentation_document_with_content_filter(
    record: &AtlasRecord,
    canonical_body: Option<&RecordBody>,
    include_supplemental_content: impl Fn(&RecordContentDocument) -> bool + Copy,
) -> RecordPresentationDocument {
    if let Some(hazard) = canonical_hazard_for_record(record, canonical_body) {
        return build_hazard_presentation_document(hazard, |content| {
            if !content.source_kind.default_contributes_to_search() {
                return false;
            }
            let adapted = RecordContentDocument {
                source_kind: content.source_kind,
                label: content.label.clone(),
                document: content.document.clone(),
            };
            include_supplemental_content(&adapted)
        });
    }
    if let Some(spell) = canonical_spell_for_record(record, canonical_body) {
        return canonical_spell_search_document(spell);
    }
    if matches!(canonical_body, Some(RecordBody::Spell(_))) {
        return RecordPresentationDocument {
            record_key: record.identity.key.clone(),
            kind: atlas_domain::RecordKind::Spell,
            title: record.identity.name.clone(),
            identity: Vec::new(),
            badges: Vec::new(),
            sections: Vec::new(),
        };
    }
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

fn canonical_spell_search_document(spell: &SpellRecord) -> RecordPresentationDocument {
    let definition = &spell.definition;
    let mut badges = Vec::new();
    let mut classification_facts = Vec::new();
    if let Some(classification) = known_spell_fact(&definition.classification) {
        if let Some(rank) = known_spell_fact(&classification.rank) {
            classification_facts.push(presentation_fact("spell.rank", "Rank", rank.to_string()));
        }
        if let Some(traits) = known_spell_fact(&classification.traits) {
            badges.extend(traits.iter().map(|value| PresentationBadge {
                kind: PresentationBadgeKind::Trait,
                label: "Trait".to_string(),
                value: value.as_str().to_string(),
            }));
        }
        if let Some(traditions) = known_spell_fact(&classification.traditions) {
            classification_facts.push(presentation_fact(
                "spell.traditions",
                "Traditions",
                traditions
                    .iter()
                    .map(|value| value.as_str())
                    .collect::<Vec<_>>()
                    .join(", "),
            ));
        }
    }

    let mut summary = Vec::new();
    if let Some(casting) = known_spell_fact(&definition.casting) {
        push_spell_fact(&mut summary, "spell.casting.time", "Cast", &casting.time);
        push_spell_fact(&mut summary, "spell.casting.cost", "Cost", &casting.cost);
        push_spell_fact(
            &mut summary,
            "spell.casting.requirements",
            "Requirements",
            &casting.requirements,
        );
        if known_spell_fact(&casting.counteraction).copied() == Some(true) {
            summary.push(presentation_fact(
                "spell.casting.counteraction",
                "Counteraction",
                "yes",
            ));
        }
    }
    if let Some(targeting) = known_spell_fact(&definition.targeting) {
        push_spell_fact(&mut summary, "spell.target", "Target", &targeting.target);
        if let Some(range) = known_spell_fact(&targeting.range) {
            summary.push(presentation_fact(
                "spell.range",
                "Range",
                range.authored_text.clone(),
            ));
        }
        if let Some(area) = known_spell_fact(&targeting.area) {
            let value = [
                known_spell_fact(&area.value).map(ToString::to_string),
                known_spell_fact(&area.area_type).map(|value| value.as_str().to_string()),
                known_spell_fact(&area.details).cloned(),
            ]
            .into_iter()
            .flatten()
            .collect::<Vec<_>>()
            .join(" ");
            if !value.is_empty() {
                summary.push(presentation_fact("spell.area", "Area", value));
            }
        }
    }
    if let Some(duration) = known_spell_fact(&definition.duration) {
        push_spell_fact(&mut summary, "spell.duration", "Duration", &duration.value);
        if known_spell_fact(&duration.sustained).copied() == Some(true) {
            summary.push(presentation_fact("spell.sustained", "Sustained", "yes"));
        }
    }

    let mut defense_facts = Vec::new();
    if let Some(defense) = known_spell_fact(&definition.defense) {
        if let Some(passive) = known_spell_fact(&defense.passive) {
            defense_facts.push(presentation_fact(
                "spell.defense.passive",
                "Passive defense",
                passive.as_str(),
            ));
        }
        if let Some(save) = known_spell_fact(&defense.save)
            && let Some(statistic) = known_spell_fact(&save.statistic)
        {
            let value = if known_spell_fact(&save.basic).copied() == Some(true) {
                format!("{} (basic)", statistic.as_str())
            } else {
                statistic.as_str().to_string()
            };
            defense_facts.push(presentation_fact("spell.defense.save", "Save", value));
        }
    }

    let mut offense = Vec::new();
    if let Some(damage) = known_spell_fact(&definition.damage) {
        for member in damage {
            let value = [
                known_spell_fact(&member.value.formula).cloned(),
                known_spell_fact(&member.value.damage_type).cloned(),
                known_spell_fact(&member.value.category).cloned(),
            ]
            .into_iter()
            .flatten()
            .collect::<Vec<_>>()
            .join(" ");
            if !value.is_empty() {
                offense.push(presentation_fact("spell.damage", "Damage", value));
            }
        }
    }

    let mut details = Vec::new();
    if let Some(heightening) = known_spell_fact(&definition.heightening) {
        match heightening {
            SpellHeightening::Interval(value) => {
                if let Some(interval) = known_spell_fact(&value.interval) {
                    details.push(presentation_fact(
                        "spell.heightening.interval",
                        "Heightened",
                        format!("every {interval} ranks"),
                    ));
                }
            }
            SpellHeightening::Fixed(values) => {
                for value in values {
                    if let SpellSourceValue::Known(rank) = value.rank {
                        details.push(presentation_fact(
                            "spell.heightening.fixed",
                            "Heightened",
                            format!("rank {rank}"),
                        ));
                    }
                }
            }
        }
    }
    if let Some(ritual) = known_spell_fact(&definition.ritual) {
        push_spell_fact(
            &mut details,
            "spell.ritual.primary_check",
            "Primary check",
            &ritual.primary_check,
        );
        if let Some(value) = known_spell_fact(&ritual.secondary_casters) {
            details.push(presentation_fact(
                "spell.ritual.secondary_casters",
                "Secondary casters",
                value.to_string(),
            ));
        }
        push_spell_fact(
            &mut details,
            "spell.ritual.secondary_checks",
            "Secondary checks",
            &ritual.secondary_checks,
        );
    }
    if let Some(rules) = known_spell_fact(&definition.rules) {
        for (index, rule) in rules.iter().enumerate() {
            if let Some((label, value)) = spell_rule_search_value(&rule.rule) {
                details.push(presentation_fact(
                    format!("spell.rule.{index}"),
                    label,
                    value,
                ));
            }
        }
    }

    let sections = [
        (
            PresentationSectionKind::Classification,
            classification_facts,
        ),
        (PresentationSectionKind::Summary, summary),
        (PresentationSectionKind::Defense, defense_facts),
        (PresentationSectionKind::Offense, offense),
        (PresentationSectionKind::Details, details),
    ]
    .into_iter()
    .filter_map(|(kind, facts)| {
        (!facts.is_empty())
            .then(|| PresentationSection::new(kind, vec![PresentationBlock::FactList(facts)]))
    })
    .collect();

    RecordPresentationDocument {
        record_key: spell.identity.record_key.clone(),
        kind: RecordKind::Spell,
        title: spell.identity.name.clone(),
        identity: Vec::new(),
        badges,
        sections,
    }
}

fn append_spell_content_fts(spell: &SpellRecord, projection: &mut RecordFtsProjection) {
    let document = canonical_spell_search_document(spell);
    for section in &document.sections {
        for block in &section.blocks {
            if let PresentationBlock::FactList(facts) = block {
                for fact in facts {
                    append_text(
                        &mut projection.facts,
                        &format!("{} {}", fact.label, fact.value),
                    );
                    append_text(&mut projection.mechanic_terms, &fact.value);
                }
            }
        }
    }
    for content in &spell.definition.content.documents {
        if content.visibility != crate::ContentVisibility::Public
            || !content.source_kind.default_contributes_to_search()
            || !matches!(
                content.duplicate_status,
                crate::DuplicateContentStatus::Unique
            )
        {
            continue;
        }
        if let Some(label) = &content.label {
            append_text(&mut projection.headings, label);
        }
        let text = crate::render_plain_text(&content.document);
        match content.source_kind.fts_field() {
            crate::ContentFtsField::Body => append_text(&mut projection.body, &text),
            crate::ContentFtsField::Facts => append_text(&mut projection.facts, &text),
            crate::ContentFtsField::EmbeddedContent => {
                append_text(&mut projection.embedded_content, &text)
            }
        }
        for reference in &content.reference_occurrences {
            if reference.visibility != crate::ContentVisibility::Public {
                continue;
            }
            if let Some(label) = &reference.label {
                append_text(&mut projection.references, label);
            }
            if let Some(name) = reference.target.display_name() {
                append_text(&mut projection.references, name);
            }
            if let Some(key) = reference.target.record_key() {
                append_text(&mut projection.references, &key.to_string());
            }
        }
    }
}

fn presentation_fact(
    key: impl Into<String>,
    label: impl Into<String>,
    value: impl Into<String>,
) -> PresentationFact {
    PresentationFact {
        key: key.into(),
        label: label.into(),
        value: value.into(),
    }
}

fn push_spell_fact(
    facts: &mut Vec<PresentationFact>,
    key: &str,
    label: &str,
    fact: &crate::SpellFact<String>,
) {
    if let Some(value) = known_spell_fact(fact) {
        facts.push(presentation_fact(key, label, value.clone()));
    }
}

fn known_spell_fact<T>(fact: &crate::SpellFact<T>) -> Option<&T> {
    match fact {
        FactValue::Value(SpellSourceValue::Known(value)) => Some(value),
        FactValue::Missing
        | FactValue::Null
        | FactValue::Value(SpellSourceValue::Unsupported(_)) => None,
    }
}

fn spell_rule_search_value(rule: &SpellRule) -> Option<(&'static str, String)> {
    let mut values = Vec::new();
    let label = match rule {
        SpellRule::DamageDice(rule) => {
            push_known(&mut values, &rule.selector);
            push_known(&mut values, &rule.dice_number);
            push_known(&mut values, &rule.die_size);
            push_known(&mut values, &rule.damage_type);
            push_known_bool(&mut values, "hide if disabled", &rule.hide_if_disabled);
            push_predicates(&mut values, &rule.predicate);
            "Damage dice rule"
        }
        SpellRule::EphemeralEffect(rule) => {
            push_known_list(&mut values, &rule.selectors);
            push_known(&mut values, &rule.uuid);
            push_predicates(&mut values, &rule.predicate);
            "Ephemeral effect rule"
        }
        SpellRule::DamageAlteration(rule) => {
            push_known(&mut values, &rule.mode);
            push_known(&mut values, &rule.property);
            push_known_list(&mut values, &rule.selectors);
            push_known(&mut values, &rule.slug);
            push_known(&mut values, &rule.value);
            push_predicates(&mut values, &rule.predicate);
            "Damage alteration rule"
        }
        SpellRule::RollOption(rule) => {
            push_known(&mut values, &rule.domain);
            push_known(&mut values, &rule.label);
            push_known(&mut values, &rule.option);
            push_known(&mut values, &rule.placement);
            push_known_bool(&mut values, "toggleable", &rule.toggleable);
            push_predicates(&mut values, &rule.predicate);
            if let Some(suboptions) = known_spell_fact(&rule.suboptions) {
                for suboption in suboptions {
                    push_known(&mut values, &suboption.label);
                    push_known(&mut values, &suboption.value);
                }
            }
            "Roll option rule"
        }
        SpellRule::ItemAlteration(rule) => {
            push_known(&mut values, &rule.item_id);
            push_known(&mut values, &rule.mode);
            push_known(&mut values, &rule.property);
            push_known(&mut values, &rule.value);
            push_predicates(&mut values, &rule.predicate);
            "Item alteration rule"
        }
        SpellRule::Unsupported(_) => return None,
    };
    Some((label, values.join(" ")))
}

fn push_known(values: &mut Vec<String>, fact: &crate::SpellFact<String>) {
    if let Some(value) = known_spell_fact(fact) {
        values.push(value.clone());
    }
}

fn push_known_list(values: &mut Vec<String>, fact: &crate::SpellFact<Vec<String>>) {
    if let Some(items) = known_spell_fact(fact) {
        values.extend(items.iter().cloned());
    }
}

fn push_known_bool(values: &mut Vec<String>, label: &str, fact: &crate::SpellFact<bool>) {
    if let Some(value) = known_spell_fact(fact) {
        values.push(format!("{label} {value}"));
    }
}

fn push_predicates(values: &mut Vec<String>, fact: &crate::SpellFact<Vec<SpellRulePredicate>>) {
    if let Some(predicates) = known_spell_fact(fact) {
        for predicate in predicates {
            match predicate {
                SpellRulePredicate::Term(value) => values.push(value.clone()),
                SpellRulePredicate::Or(items) => values.extend(items.iter().cloned()),
                SpellRulePredicate::Unsupported(_) => {}
            }
        }
    }
}

fn canonical_spell_for_record<'a>(
    record: &AtlasRecord,
    canonical_body: Option<&'a RecordBody>,
) -> Option<&'a SpellRecord> {
    let spell = canonical_body?.as_spell()?;
    (spell.identity.record_key == record.identity.key).then_some(spell)
}

fn canonical_creature_for_record<'a>(
    record: &AtlasRecord,
    canonical_body: Option<&'a RecordBody>,
) -> Option<&'a CreatureRecord> {
    match canonical_body? {
        RecordBody::Creature(creature) if creature.identity.record_key == record.identity.key => {
            Some(creature)
        }
        RecordBody::Creature(_) | RecordBody::Hazard(_) | RecordBody::Spell(_) => None,
    }
}

fn canonical_hazard_for_record<'a>(
    record: &AtlasRecord,
    canonical_body: Option<&'a RecordBody>,
) -> Option<&'a HazardRecord> {
    match canonical_body? {
        RecordBody::Hazard(hazard) if hazard.identity.record_key == record.identity.key => {
            Some(hazard)
        }
        RecordBody::Creature(_) | RecordBody::Hazard(_) | RecordBody::Spell(_) => None,
    }
}

fn append_structured_terms(
    record: &AtlasRecord,
    canonical_creature: Option<&CreatureRecord>,
    canonical_hazard: Option<&HazardRecord>,
    canonical_spell: Option<&SpellRecord>,
    projection: &mut RecordFtsProjection,
) {
    let generic_mechanics = (!matches!(
        record.classification.kind,
        RecordKind::Creature | RecordKind::Hazard | RecordKind::Spell
    ))
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
    if record.classification.kind != RecordKind::Spell
        && let Some(level) = record.classification.level
    {
        mechanics.add_text(&format!("level {level}"));
        mechanics.add_text(&ordinal_phrase(level, "level"));
    }
    if let Some(spell) = canonical_spell {
        if let FactValue::Value(SpellSourceValue::Known(classification)) =
            &spell.definition.classification
        {
            if let FactValue::Value(SpellSourceValue::Known(rank)) = &classification.rank {
                mechanics.add_text(&format!("rank {rank}"));
                mechanics.add_text(&ordinal_phrase(i64::from(*rank), "rank"));
            }
            if let FactValue::Value(SpellSourceValue::Known(traditions)) =
                &classification.traditions
            {
                mechanics.add_slugs(
                    &traditions
                        .iter()
                        .map(|value| value.as_str().to_string())
                        .collect::<Vec<_>>(),
                );
            }
            if let FactValue::Value(SpellSourceValue::Known(traits)) = &classification.traits {
                let traits = traits
                    .iter()
                    .map(|value| value.as_str().to_string())
                    .collect::<Vec<_>>();
                projection.traits = traits.join(" ");
                mechanics.add_slugs(&traits);
            }
        }
        if let FactValue::Value(SpellSourceValue::Known(casting)) = &spell.definition.casting {
            mechanics.add_optional_text(known_spell_fact(&casting.time).map(String::as_str));
            mechanics.add_optional_text(known_spell_fact(&casting.cost).map(String::as_str));
            mechanics
                .add_optional_text(known_spell_fact(&casting.requirements).map(String::as_str));
            if known_spell_fact(&casting.counteraction).copied() == Some(true) {
                mechanics.add_text("counteraction");
            }
        }
        if let FactValue::Value(SpellSourceValue::Known(targeting)) = &spell.definition.targeting {
            if let FactValue::Value(SpellSourceValue::Known(range)) = &targeting.range {
                mechanics.add_text(&range.authored_text);
            }
            if let FactValue::Value(SpellSourceValue::Known(target)) = &targeting.target {
                mechanics.add_text(target);
            }
            if let Some(area) = known_spell_fact(&targeting.area) {
                mechanics.add_optional_slug(
                    known_spell_fact(&area.area_type).map(|value| value.as_str()),
                );
                mechanics.add_optional_text(known_spell_fact(&area.details).map(String::as_str));
            }
        }
        if let Some(defense) = known_spell_fact(&spell.definition.defense) {
            mechanics
                .add_optional_slug(known_spell_fact(&defense.passive).map(|value| value.as_str()));
            if let Some(save) = known_spell_fact(&defense.save) {
                mechanics.add_optional_slug(
                    known_spell_fact(&save.statistic).map(|value| value.as_str()),
                );
                if known_spell_fact(&save.basic).copied() == Some(true) {
                    mechanics.add_text("basic save");
                }
            }
        }
        if let Some(damage) = known_spell_fact(&spell.definition.damage) {
            for member in damage {
                mechanics
                    .add_optional_text(known_spell_fact(&member.value.formula).map(String::as_str));
                mechanics.add_optional_slug(
                    known_spell_fact(&member.value.damage_type).map(String::as_str),
                );
                mechanics.add_optional_slug(
                    known_spell_fact(&member.value.category).map(String::as_str),
                );
                if let Some(kinds) = known_spell_fact(&member.value.kinds) {
                    mechanics.add_slugs(kinds);
                }
            }
        }
        if let Some(duration) = known_spell_fact(&spell.definition.duration) {
            mechanics.add_optional_text(known_spell_fact(&duration.value).map(String::as_str));
            if known_spell_fact(&duration.sustained).copied() == Some(true) {
                mechanics.add_text("sustained");
            }
        }
        if let Some(ritual) = known_spell_fact(&spell.definition.ritual) {
            mechanics
                .add_optional_text(known_spell_fact(&ritual.primary_check).map(String::as_str));
            mechanics
                .add_optional_text(known_spell_fact(&ritual.secondary_checks).map(String::as_str));
        }
    }
    mechanics.add_optional_slug(record.classification.rarity.map(|rarity| rarity.as_str()));
    if let Some(duration) = record.timing.duration_time() {
        mechanics.add_text(&duration.text);
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
    } else if let Some(hazard) = canonical_hazard {
        let canonical = crate::project_hazard_facts(hazard);
        mechanics.add_texts(&canonical.mechanic_terms);
        for metric in &canonical.metrics {
            let label = label_for_row(metric);
            metrics.add_text(&label.label);
            metrics.add_optional_text(label.short_label.as_deref());
        }
        if canonical.conveniences.detection_dc.is_some() {
            metrics.add_text("Detection DC Stealth DC");
        }
        if canonical.conveniences.broken_threshold.is_some() {
            metrics.add_text("Broken Threshold");
        }
        if canonical.conveniences.initiative_suggestion.is_some() {
            metrics.add_text("Stealth Initiative");
        }
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

fn append_hazard_owned_content(hazard: &HazardRecord, projection: &mut RecordFtsProjection) {
    let mut documents = hazard.content.documents.iter().collect::<Vec<_>>();
    documents.sort_by_key(|document| (document.authored_order, document.id.content_key.as_str()));
    for document in documents {
        let rendered = crate::render_plain_text(&document.document);
        let target = match document.source_kind.fts_field() {
            crate::ContentFtsField::Body => &mut projection.body,
            crate::ContentFtsField::Facts => &mut projection.facts,
            crate::ContentFtsField::EmbeddedContent => &mut projection.embedded_content,
        };
        append_text(target, &rendered);
        for occurrence in &document.reference_occurrences {
            let target = match &occurrence.target {
                RichLinkTarget::Record { key, name } => format!("{name}\n{key}"),
                RichLinkTarget::LocalContent { content_key, label } => {
                    label.clone().unwrap_or_else(|| content_key.clone())
                }
                RichLinkTarget::External { target, label } => {
                    label.clone().unwrap_or_else(|| target.clone())
                }
                RichLinkTarget::Unresolved {
                    target,
                    fallback_label,
                } => format!("{fallback_label}\n{target}"),
            };
            append_text(&mut projection.references, &target);
        }
    }
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
        ItemMechanics, MetricRow, MetricValue, NormalizedTime, RecordActivationTiming,
        RecordClassification, RecordContent, RecordContentDocument, RecordDurationTiming,
        RecordIdentity, RecordMechanics, RecordProvenance, RecordPublication, RecordRequirements,
        RecordTaxonomy, RecordTiming, RecordVisibility, RichDocument, RichNode, SpellIdentity,
        SpellProvenance, SpellRecord, SpellSourceId,
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
    fn canonical_families_without_bodies_cannot_fall_back_to_generic_mechanics() {
        for kind in [RecordKind::Creature, RecordKind::Hazard] {
            let mut record = base_record();
            record.classification.kind = kind;
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
    }

    #[test]
    fn canonical_spell_body_blocks_generic_mechanics_from_search_and_presentation() {
        let mut record = base_record();
        record.mechanics.document = FoundryDocumentMechanics::Item(ItemMechanics {
            category: Some("legacy-secret-category".to_string()),
            ..ItemMechanics::default()
        });
        record.mechanics.metrics.push(MetricRow {
            domain: MetricDomain::Actor,
            key: "speed.fly.value".to_string(),
            value: MetricValue::Number(999.0),
        });
        let body = RecordBody::Spell(SpellRecord::new(
            SpellIdentity {
                record_key: record.identity.key.clone(),
                source_id: SpellSourceId::new("TestRecord").expect("source id"),
                name: "Canonical Spell".to_string(),
            },
            SpellProvenance {
                source_path: "packs/spells/test.json".to_string(),
                source_contract_version: "fixture".to_string(),
                source_system_version: "6.12.4".to_string(),
                source_upstream_commit: "fixture".to_string(),
                standalone_location: FactValue::Null,
            },
        ));

        let projection = build_search_fts_projection(&record, &[], Some(&body));
        assert!(!projection.mechanic_terms.contains("legacy-secret"));
        assert!(!projection.taxonomy_terms.contains("legacy-secret"));
        let presentation =
            build_search_presentation_document_with_content_filter(&record, Some(&body), |_| true);
        assert_eq!(presentation.title, "Canonical Spell");
        assert!(presentation.sections.is_empty());
    }

    #[test]
    fn generic_equipment_fts_adds_structured_terms_without_duplicating_traits() {
        let mut record = base_record();
        record.classification.kind = RecordKind::Equipment;
        record.foundry.record_type = FoundryRecordType::Weapon;
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
            PackName::new("internal-equipment-pack").expect("pack parses"),
            RecordId::new("TestRecord").expect("id parses"),
        );
        record.publication.title = Some("Pathfinder Player Core".to_string());
        record.publication.category = PublicationCategory::Core;
        record.foundry.pack_label = "Equipment".to_string();
        record.mechanics.document = FoundryDocumentMechanics::Item(ItemMechanics {
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
        assert!(projection.mechanic_terms.contains("2nd level"));
        assert!(projection.mechanic_terms.contains("uncommon"));
        assert!(projection.source_terms.contains("Pathfinder Player Core"));
        assert!(projection.source_terms.contains("Equipment"));
        assert!(!projection.source_terms.contains("internal-equipment-pack"));
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
