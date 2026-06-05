use atlas_record::{
    AtlasRecord, ContentSourceKind, RichDocument, iter_foundry_links, render_plain_text,
};

use crate::generated::afflictions::AfflictionFamily;
use crate::records::variants;
use crate::records::{EmbeddedItemFact, SourceRecordFacts};
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

pub(super) fn has_affliction_shape(document: Option<&RichDocument>) -> bool {
    let Some(description) = document
        .map(render_plain_text)
        .filter(|value| !value.trim().is_empty())
    else {
        return false;
    };
    let normalized = normalize_text(&description);
    normalized.contains("saving throw") && normalized.contains("stage 1")
}

pub(super) fn record_affliction_document(record: &AtlasRecord) -> Option<RichDocument> {
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
) -> Option<RichDocument> {
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

pub(super) fn extract_linked_names(document: Option<&RichDocument>) -> Vec<String> {
    let Some(document) = document else {
        return Vec::new();
    };
    variants::sorted_unique(
        iter_foundry_links(document)
            .filter_map(|candidate| {
                candidate
                    .label
                    .as_ref()
                    .map(|label| render_plain_text(&RichDocument::new(label.clone())))
                    .filter(|label| !label.trim().is_empty())
                    .or_else(|| candidate.target.display_name().map(ToOwned::to_owned))
            })
            .collect(),
    )
}

fn supplemental_document(
    record: &AtlasRecord,
    source_kind: ContentSourceKind,
) -> Option<RichDocument> {
    record
        .content
        .documents
        .iter()
        .find(|content| content.source_kind == source_kind)
        .map(|content| content.document.clone())
}
