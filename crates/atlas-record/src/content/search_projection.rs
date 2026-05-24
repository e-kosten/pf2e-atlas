use atlas_domain::TimeKind;

use crate::{ContentBlock, ContentDocument, ContentFtsField, NormalizedRecord, label_for_row};

use super::ContentReference;
use super::render::{render_inlines_plain, render_plain_text};

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct RecordFtsProjection {
    pub title: String,
    pub aliases: String,
    pub traits: String,
    pub precision_terms: String,
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
    record: &NormalizedRecord,
    aliases: &[String],
) -> RecordFtsProjection {
    let mut projection = RecordFtsProjection {
        title: record.name.clone(),
        aliases: aliases.join("\n"),
        traits: record.traits.join(" "),
        precision_terms: precision_terms(record),
        ..RecordFtsProjection::default()
    };
    append_structured_terms(record, &mut projection);

    if let Some(document) = &record.description {
        append_document(document, ContentFtsField::Body, &mut projection);
    }
    if let Some(document) = &record.blurb {
        append_document(document, ContentFtsField::Body, &mut projection);
    }
    for supplemental in &record.supplemental_content {
        if !supplemental.contributes_to_search {
            continue;
        }
        append_document(
            &supplemental.document,
            supplemental.source_kind.fts_field(),
            &mut projection,
        );
    }

    projection
}

fn precision_terms(record: &NormalizedRecord) -> String {
    let mut terms = TermCollector::default();
    terms.add_slug(record.record_family.as_str());
    terms.add_slug(&record.foundry_record_type);
    terms.render()
}

fn append_structured_terms(record: &NormalizedRecord, projection: &mut RecordFtsProjection) {
    let mut taxonomy = TermCollector::default();
    taxonomy.add_slug(record.record_family.as_str());
    taxonomy.add_slug(&record.foundry_record_type);
    taxonomy.add_slugs(&record.taxonomy_families);
    taxonomy.add_optional_slug(record.system_category.as_deref());
    taxonomy.add_optional_slug(record.system_group.as_deref());
    taxonomy.add_optional_slug(record.system_base_item.as_deref());
    if let Some(item) = record.item_data.as_ref() {
        taxonomy.add_optional_slug(item.system_category.as_deref());
        taxonomy.add_optional_slug(item.system_group.as_deref());
        taxonomy.add_optional_slug(item.system_base_item.as_deref());
    }
    if let Some(spell) = record.spell_data.as_ref() {
        taxonomy.add_slugs(&spell.spell_kinds);
    }
    taxonomy.add_optional_text(record.variant_base_name.as_deref());
    taxonomy.add_optional_text(record.variant_label.as_deref());
    taxonomy.add_slugs(&record.variant_axes);
    append_text(&mut projection.taxonomy_terms, &taxonomy.render());

    let mut constraints = TermCollector::default();
    constraints.add_texts(&record.prerequisites);
    if let Some(time) = record.activation_time.as_ref() {
        constraints.add_text(&time.text);
        append_action_cost_terms(time.kind, time.actions, &mut constraints);
    }
    if let Some(item) = record.item_data.as_ref() {
        constraints.add_optional_text(item.hands_requirement.as_deref());
    }
    append_text(&mut projection.constraint_terms, &constraints.render());

    let mut mechanics = TermCollector::default();
    if let Some(level) = record.level {
        mechanics.add_text(&format!("level {level}"));
        mechanics.add_text(&ordinal_phrase(level, "level"));
        if record.record_family == atlas_domain::RecordFamily::Spell {
            mechanics.add_text(&format!("rank {level}"));
            mechanics.add_text(&ordinal_phrase(level, "rank"));
        }
    }
    mechanics.add_optional_slug(record.rarity.as_deref());
    if let Some(duration) = record.duration.as_ref() {
        mechanics.add_text(&duration.text);
    }
    if let Some(spell) = record.spell_data.as_ref() {
        mechanics.add_slugs(&spell.traditions);
        mechanics.add_optional_text(spell.range_text.as_deref());
        mechanics.add_optional_text(spell.target_text.as_deref());
        mechanics.add_optional_slug(spell.area_type.as_deref());
        mechanics.add_optional_slug(spell.save_type.as_deref());
        mechanics.add_slugs(&spell.damage_types);
        if spell.sustained {
            mechanics.add_text("sustained");
        }
        if spell.basic_save {
            mechanics.add_text("basic save");
        }
    }
    if let Some(item) = record.item_data.as_ref() {
        mechanics.add_optional_text(item.system_usage.as_deref());
        mechanics.add_slugs(&item.damage_types);
    }
    if let Some(actor) = record.actor_data.as_ref() {
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
    append_text(&mut projection.mechanic_terms, &mechanics.render());

    let mut source = TermCollector::default();
    source.add_optional_text(record.publication_title.as_deref());
    if record.publication_family != atlas_domain::PublicationFamily::Unknown {
        source.add_slug(record.publication_family.as_str());
    }
    source.add_text(&record.pack_label);
    append_text(&mut projection.source_terms, &source.render());

    let mut metrics = TermCollector::default();
    for metric in &record.metrics {
        let label = label_for_row(metric);
        metrics.add_text(&label.label);
        if let Some(short_label) = label.short_label.as_deref() {
            metrics.add_text(short_label);
        }
    }
    append_text(&mut projection.metric_terms, &metrics.render());
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

fn append_document(
    document: &ContentDocument,
    field: ContentFtsField,
    projection: &mut RecordFtsProjection,
) {
    append_text(
        &mut projection.headings,
        &collect_headings(document).join("\n"),
    );
    match field {
        ContentFtsField::Body => append_text(&mut projection.body, &render_plain_text(document)),
        ContentFtsField::Facts => append_text(&mut projection.facts, &render_plain_text(document)),
        ContentFtsField::EmbeddedContent => {
            append_text(
                &mut projection.embedded_content,
                &render_plain_text(document),
            );
        }
    }
    append_text(
        &mut projection.references,
        &collect_reference_labels(document).join("\n"),
    );
}

fn collect_headings(document: &ContentDocument) -> Vec<String> {
    let mut headings = Vec::new();
    for block in &document.blocks {
        collect_block_headings(block, &mut headings);
    }
    headings
}

fn collect_block_headings(block: &ContentBlock, headings: &mut Vec<String>) {
    match block {
        ContentBlock::Heading { content, .. } => headings.push(render_inlines_plain(content)),
        ContentBlock::List { items, .. } => {
            for item in items {
                for block in item {
                    collect_block_headings(block, headings);
                }
            }
        }
        ContentBlock::Callout { blocks, .. } | ContentBlock::RuleBlock { blocks, .. } => {
            for block in blocks {
                collect_block_headings(block, headings);
            }
        }
        ContentBlock::DefinitionList { items } => {
            for item in items {
                for block in &item.definition {
                    collect_block_headings(block, headings);
                }
            }
        }
        ContentBlock::Table {
            caption: Some(caption),
            ..
        } => headings.push(render_inlines_plain(caption)),
        ContentBlock::Paragraph { .. } | ContentBlock::Table { .. } | ContentBlock::Separator => {}
    }
}

fn collect_reference_labels(document: &ContentDocument) -> Vec<String> {
    crate::iter_content_references(document)
        .filter_map(reference_label)
        .collect()
}

fn reference_label(reference: &ContentReference) -> Option<String> {
    reference
        .label
        .as_deref()
        .map(render_inlines_plain)
        .filter(|label| !label.is_empty())
        .or_else(|| reference.resolved_name.clone())
        .or_else(|| reference.resolved_key.as_ref().map(ToString::to_string))
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
    use atlas_domain::{
        MetricDomain, PackName, PublicationFamily, RecordFamily, RecordId, RecordKey, TimeKind,
    };

    use crate::{
        ActorSideData, ContentInline, ContentSourceKind, ContentVisibility, ItemSideData,
        MetricRow, MetricValue, NormalizedTime, SpellSideData, SupplementalContentDocument,
    };

    use super::*;

    #[test]
    fn fts_projection_splits_body_facts_and_embedded_content() {
        let mut record = base_record();
        record.description = Some(ContentDocument::new(vec![
            ContentBlock::Heading {
                level: 2,
                content: vec![ContentInline::Text {
                    text: "Effect".to_string(),
                }],
            },
            ContentBlock::Paragraph {
                content: vec![ContentInline::Text {
                    text: "Main body".to_string(),
                }],
            },
        ]));
        record
            .supplemental_content
            .push(SupplementalContentDocument {
                source_kind: ContentSourceKind::Disable,
                visibility: ContentVisibility::Public,
                contributes_to_search: true,
                contributes_to_references: true,
                label: None,
                document: text_document("Disable text"),
            });
        record
            .supplemental_content
            .push(SupplementalContentDocument {
                source_kind: ContentSourceKind::EmbeddedItemDescription,
                visibility: ContentVisibility::Public,
                contributes_to_search: true,
                contributes_to_references: true,
                label: Some("Jaws".to_string()),
                document: text_document("Embedded attack text"),
            });

        let projection = build_record_fts_projection(&record, &["Alias".to_string()]);

        assert_eq!(projection.title, "Test Record");
        assert_eq!(projection.aliases, "Alias");
        assert_eq!(projection.traits, "healing vitality");
        assert_eq!(projection.precision_terms, "spell");
        assert_eq!(projection.headings, "Effect");
        assert_eq!(projection.body, "Effect\nMain body");
        assert_eq!(projection.facts, "Disable text");
        assert_eq!(projection.embedded_content, "Embedded attack text");
    }

    #[test]
    fn fts_projection_adds_deterministic_structured_terms_without_duplicating_traits() {
        let mut record = base_record();
        record.record_family = RecordFamily::Spell;
        record.foundry_record_type = "spell".to_string();
        record.level = Some(2);
        record.rarity = Some("uncommon".to_string());
        record.prerequisites = vec!["expert in Medicine".to_string()];
        record.activation_time = Some(NormalizedTime {
            kind: TimeKind::Actions,
            actions: Some(2),
            duration_value: None,
            duration_unit: None,
            text: "2".to_string(),
        });
        record.duration = Some(NormalizedTime {
            kind: TimeKind::Duration,
            actions: None,
            duration_value: None,
            duration_unit: None,
            text: "1 minute".to_string(),
        });
        record.taxonomy_families = vec!["focus_spell".to_string()];
        record.pack_name = PackName::new("internal-spell-pack").expect("pack parses");
        record.publication_title = Some("Pathfinder Player Core".to_string());
        record.publication_family = PublicationFamily::Core;
        record.pack_label = "Spells".to_string();
        record.actor_data = Some(ActorSideData {
            size: Some("medium".to_string()),
            languages: vec!["common".to_string()],
            speed_types: vec!["fly".to_string()],
            senses: vec!["darkvision".to_string()],
            immunities: vec!["fire".to_string()],
            resistances: Vec::new(),
            weaknesses: Vec::new(),
            disable_text: None,
            disable_skills: vec!["thievery".to_string()],
            is_complex: true,
        });
        record.item_data = Some(ItemSideData {
            system_category: Some("weapon".to_string()),
            system_base_item: Some("longsword".to_string()),
            system_group: Some("sword".to_string()),
            system_usage: Some("held in one hand".to_string()),
            price_cp: None,
            bulk_value: None,
            hands_requirement: Some("one hand".to_string()),
            damage_types: vec!["slashing".to_string()],
        });
        record.spell_data = Some(SpellSideData {
            traditions: vec!["divine".to_string()],
            spell_kinds: vec!["focus".to_string()],
            range_text: Some("30 feet".to_string()),
            range_value: Some(30.0),
            target_text: Some("1 ally".to_string()),
            area_type: Some("burst".to_string()),
            area_value: None,
            save_type: Some("will".to_string()),
            sustained: true,
            basic_save: true,
            damage_types: vec!["vitality".to_string()],
        });
        record.metrics = vec![MetricRow {
            domain: MetricDomain::Actor,
            key: "speed.fly.value".to_string(),
            value: MetricValue::Number(60.0),
        }];

        let projection = build_record_fts_projection(&record, &[]);

        assert_eq!(projection.traits, "healing vitality");
        assert_eq!(projection.precision_terms, "spell");
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

    fn base_record() -> NormalizedRecord {
        NormalizedRecord {
            key: RecordKey::new(
                PackName::new("test-pack").expect("pack parses"),
                RecordId::new("TestRecord").expect("id parses"),
            ),
            id: RecordId::new("TestRecord").expect("id parses"),
            name: "Test Record".to_string(),
            normalized_name: "test record".to_string(),
            record_family: RecordFamily::Spell,
            pack_name: PackName::new("test-pack").expect("pack parses"),
            pack_label: "Test Pack".to_string(),
            foundry_document_type: "Item".to_string(),
            foundry_record_type: "spell".to_string(),
            level: None,
            rarity: None,
            traits: vec!["healing".to_string(), "vitality".to_string()],
            prerequisites: Vec::new(),
            system_category: None,
            system_group: None,
            system_base_item: None,
            system_usage: None,
            system_price_json: None,
            system_actions_value: None,
            system_time_value: None,
            system_duration_value: None,
            price_cp: None,
            activation_time: None,
            duration: None,
            metrics: Vec::new(),
            actor_data: None,
            item_data: None,
            spell_data: None,
            publication_title: None,
            publication_remaster: false,
            description: None,
            blurb: None,
            supplemental_content: Vec::new(),
            publication_family: PublicationFamily::Unknown,
            folder_id: None,
            taxonomy_families: Vec::new(),
            variant_group_key: None,
            variant_base_name: None,
            variant_label: None,
            variant_axes: Vec::new(),
            variant_confidence: None,
            variant_source: "none".to_string(),
            source_path: "test.json".to_string(),
            is_default_visible: true,
            raw_json: "{}".to_string(),
        }
    }

    fn text_document(text: &str) -> ContentDocument {
        ContentDocument::new(vec![ContentBlock::Paragraph {
            content: vec![ContentInline::Text {
                text: text.to_string(),
            }],
        }])
    }
}
