use atlas_record::{
    ContentDocument, ContentInline, ContentReferenceLocator, ContentSourceKind,
    iter_content_references, render_plain_text,
};

use crate::generated::afflictions::AfflictionFamily;
use crate::records::variants;
use crate::records::{AtlasRecord, EmbeddedItemFact, SourceRecordFacts};
use crate::source::normalize::normalize_text;

pub(super) fn detect_affliction_family(
    traits: &[String],
    system_category: Option<&str>,
) -> Option<AfflictionFamily> {
    let system_category = system_category.map(normalize_text);
    if traits.iter().any(|trait_value| trait_value == "disease")
        || system_category.as_deref() == Some("disease")
    {
        return Some(AfflictionFamily::Disease);
    }
    if traits.iter().any(|trait_value| trait_value == "poison")
        || system_category.as_deref() == Some("poison")
    {
        return Some(AfflictionFamily::Poison);
    }
    if traits.iter().any(|trait_value| trait_value == "curse")
        || system_category.as_deref() == Some("curse")
    {
        return Some(AfflictionFamily::Curse);
    }
    None
}

pub(super) fn affliction_family_label(family: AfflictionFamily) -> &'static str {
    match family {
        AfflictionFamily::Curse => "curse",
        AfflictionFamily::Disease => "disease",
        AfflictionFamily::Poison => "poison",
    }
}

pub(super) fn has_affliction_shape(document: Option<&ContentDocument>) -> bool {
    let Some(description) = document
        .map(render_plain_text)
        .filter(|value| !value.trim().is_empty())
    else {
        return false;
    };
    let normalized = normalize_text(&description);
    normalized.contains("saving throw") && normalized.contains("stage 1")
}

pub(super) fn record_affliction_document(record: &AtlasRecord) -> Option<ContentDocument> {
    record
        .content
        .description()
        .cloned()
        .or_else(|| supplemental_document(record, ContentSourceKind::DetailsFieldDescription))
        .or_else(|| supplemental_document(record, ContentSourceKind::PublicNotes))
        .or_else(|| record.content.blurb().cloned())
}

pub(super) fn embedded_item_affliction_document(
    item: &EmbeddedItemFact,
    source_facts: &SourceRecordFacts,
) -> Option<ContentDocument> {
    item.content_refs
        .iter()
        .find(|content_ref| {
            matches!(
                content_ref.source_kind,
                ContentSourceKind::EmbeddedItemDescription
                    | ContentSourceKind::EmbeddedSpellDescription
            )
        })
        .and_then(|content_ref| source_facts.source_content.get(&content_ref.local_key))
        .map(|content| content.document.clone())
}

pub(super) fn parse_compendium_source(value: &str) -> Option<(String, String)> {
    let parts = value.split('.').collect::<Vec<_>>();
    if parts.len() >= 5 && parts.first() == Some(&"Compendium") && parts.get(1) == Some(&"pf2e") {
        return Some((normalize_text(parts.get(2)?), normalize_text(parts.last()?)));
    }
    None
}

pub(super) fn extract_linked_names(document: Option<&ContentDocument>) -> Vec<String> {
    let Some(document) = document else {
        return Vec::new();
    };
    variants::sorted_unique(
        iter_content_references(document)
            .filter_map(|candidate| {
                candidate
                    .label
                    .as_ref()
                    .and_then(|label| {
                        let text = render_inlines_plain_text(label).trim().to_string();
                        (!text.is_empty()).then_some(text)
                    })
                    .or_else(|| fallback_linked_name(&candidate.locator))
            })
            .collect(),
    )
}

fn supplemental_document(
    record: &AtlasRecord,
    source_kind: ContentSourceKind,
) -> Option<ContentDocument> {
    record
        .content
        .documents
        .iter()
        .find(|content| content.source_kind == source_kind)
        .map(|content| content.document.clone())
}

fn render_inlines_plain_text(inlines: &[ContentInline]) -> String {
    let mut text = String::new();
    for inline in inlines {
        match inline {
            ContentInline::Text { text: value } | ContentInline::Code { text: value } => {
                text.push_str(value);
            }
            ContentInline::Strong { content } | ContentInline::Emphasis { content } => {
                text.push_str(&render_inlines_plain_text(content));
            }
            ContentInline::Reference { reference } => {
                if let Some(label) = &reference.label {
                    text.push_str(&render_inlines_plain_text(label));
                }
            }
            ContentInline::Break => text.push(' '),
            ContentInline::Roll { label, formula, .. } => {
                text.push_str(label.as_ref().unwrap_or(formula));
            }
            ContentInline::Template { label, .. }
            | ContentInline::Icon {
                label: Some(label), ..
            } => {
                text.push_str(label);
            }
            ContentInline::Macro {
                label: Some(label), ..
            } => {
                text.push_str(label);
            }
            ContentInline::Macro { label: None, .. }
            | ContentInline::ActionGlyph { .. }
            | ContentInline::Icon { label: None, .. } => {}
        }
    }
    text
}

fn fallback_linked_name(locator: &ContentReferenceLocator) -> Option<String> {
    let raw = match locator {
        ContentReferenceLocator::FoundryUuid { raw_target }
        | ContentReferenceLocator::Compendium { raw_target } => raw_target.as_str(),
        ContentReferenceLocator::PackAndLocator { locator, .. } => locator.as_str(),
        ContentReferenceLocator::Unknown { raw } => raw.as_str(),
    };
    let tail = raw.split('.').next_back()?.replace(['-', '_'], " ");
    let trimmed = tail.trim();
    (!trimmed.is_empty()).then(|| trimmed.to_string())
}
