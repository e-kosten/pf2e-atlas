use crate::{ContentBlock, ContentDocument, ContentFtsField, NormalizedRecord};

use super::ContentReference;
use super::render::{render_inlines_plain, render_plain_text};

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct RecordFtsProjection {
    pub title: String,
    pub aliases: String,
    pub traits: String,
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
        ..RecordFtsProjection::default()
    };

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
    use atlas_domain::{PackName, PublicationFamily, RecordFamily, RecordId, RecordKey};

    use crate::{ContentInline, ContentSourceKind, ContentVisibility, SupplementalContentDocument};

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
        assert_eq!(projection.headings, "Effect");
        assert_eq!(projection.body, "Effect\nMain body");
        assert_eq!(projection.facts, "Disable text");
        assert_eq!(projection.embedded_content, "Embedded attack text");
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
