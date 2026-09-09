use atlas_domain::RecordKey;
use atlas_record::{
    FactValue, H8FieldValue, JournalPageEntry, JournalRecord, RichDocument, RichNode,
    iter_foundry_links, render_plain_text,
};

use crate::records::RecordReferenceIndex;
use crate::records::references::{record_by_key, reference_pack_and_locator, resolve_record_key};
use crate::source::normalize::normalize_text;

pub(super) struct JournalRemasterChange {
    pub(super) remaster_record_key: RecordKey,
    pub(super) legacy_name: String,
    pub(super) source_ref: String,
}

pub(super) fn extract_remaster_journal_changes(
    journal: &JournalRecord,
    index: &RecordReferenceIndex,
) -> Vec<JournalRemasterChange> {
    let mut changes = Vec::new();
    let FactValue::Value(H8FieldValue::Known(pages)) = &journal.pages else {
        return changes;
    };
    for page in pages {
        let JournalPageEntry::Page(page) = page else {
            continue;
        };
        let Some(page_name) = known(&page.name) else {
            continue;
        };
        let Some(document) = known(&page.text).and_then(|text| known(&text.content)) else {
            continue;
        };
        let source_ref = format!("journal:{page_name}");
        if page_name == "Remaster Changes" {
            for list_item in rich_elements(&document.nodes, "li") {
                let targets = resolve_journal_targets(list_item, index);
                if targets.len() != 1 {
                    continue;
                }
                let old_segment = split_remaster_intro_alias_segment(&rich_text(list_item));
                for alias_text in split_alias_list_text(&old_segment) {
                    changes.push(JournalRemasterChange {
                        remaster_record_key: targets[0].clone(),
                        legacy_name: alias_text,
                        source_ref: source_ref.clone(),
                    });
                }
            }
        }

        for row in rich_elements(&document.nodes, "tr") {
            let cells = direct_rich_elements(row, &["td", "th"]);
            if cells.len() < 2 {
                continue;
            }
            let status = cells
                .get(2)
                .map(|cell| normalize_text(&rich_text(cell)))
                .unwrap_or_else(|| "renamed".to_string());
            if !matches!(status.as_str(), "renamed" | "merged" | "replaced") {
                continue;
            }

            let old_cell = cells[0];
            let Some(new_cell) = cells.last().copied() else {
                continue;
            };
            let targets = resolve_journal_targets(new_cell, index);
            if targets.is_empty() {
                continue;
            }

            if targets.len() == 1 {
                let Some(old_name) = resolve_alias_source_name(old_cell, index) else {
                    continue;
                };
                changes.push(JournalRemasterChange {
                    remaster_record_key: targets[0].clone(),
                    legacy_name: old_name,
                    source_ref: source_ref.clone(),
                });
                continue;
            }

            let Some(grouped_aliases) =
                expand_grouped_alias_text(&rich_text(old_cell), targets.len())
            else {
                continue;
            };
            for (alias_text, target) in grouped_aliases.iter().zip(targets.iter()) {
                changes.push(JournalRemasterChange {
                    remaster_record_key: target.clone(),
                    legacy_name: alias_text.clone(),
                    source_ref: source_ref.clone(),
                });
            }
        }
    }

    changes
}

fn known<T>(value: &atlas_record::H8Fact<T>) -> Option<&T> {
    match value {
        FactValue::Value(H8FieldValue::Known(value)) => Some(value),
        FactValue::Missing | FactValue::Null | FactValue::Value(H8FieldValue::Unsupported(_)) => {
            None
        }
    }
}

fn resolve_journal_targets(nodes: &[RichNode], index: &RecordReferenceIndex) -> Vec<RecordKey> {
    let document = RichDocument::new(nodes.to_vec());
    let links = iter_foundry_links(&document).collect::<Vec<_>>();
    if links.is_empty() {
        return resolve_record_key(None, &rich_text(nodes), index)
            .into_iter()
            .collect();
    }

    links
        .into_iter()
        .filter_map(|link| {
            let (pack_name, locator) = reference_pack_and_locator(&link.source.authored_target)?;
            let record_key = resolve_record_key(Some(&pack_name), &locator, index)?;
            record_by_key(index, &record_key)
                .is_some_and(|record| record.foundry.document_type.as_str() != "JournalEntry")
                .then_some(record_key)
        })
        .collect()
}

fn resolve_alias_source_name(nodes: &[RichNode], index: &RecordReferenceIndex) -> Option<String> {
    let document = RichDocument::new(nodes.to_vec());
    let mut links = iter_foundry_links(&document);
    let Some(link) = links.next() else {
        let direct_text = rich_text(nodes);
        return (!direct_text.is_empty()).then_some(direct_text);
    };
    let (pack_name, locator) = reference_pack_and_locator(&link.source.authored_target)?;
    let record_key = resolve_record_key(Some(&pack_name), &locator, index)?;
    record_by_key(index, &record_key).map(|record| record.identity.name.clone())
}

fn rich_elements<'a>(nodes: &'a [RichNode], tag_name: &str) -> Vec<&'a [RichNode]> {
    let mut elements = Vec::new();
    collect_rich_elements(nodes, tag_name, &mut elements);
    elements
}

fn collect_rich_elements<'a>(
    nodes: &'a [RichNode],
    tag_name: &str,
    elements: &mut Vec<&'a [RichNode]>,
) {
    for node in nodes {
        if let RichNode::HtmlElement { tag, children, .. } = node {
            if tag == tag_name {
                elements.push(children);
            }
            collect_rich_elements(children, tag_name, elements);
        }
    }
}

fn direct_rich_elements<'a>(nodes: &'a [RichNode], tags: &[&str]) -> Vec<&'a [RichNode]> {
    nodes
        .iter()
        .filter_map(|node| {
            let RichNode::HtmlElement { tag, children, .. } = node else {
                return None;
            };
            tags.contains(&tag.as_str()).then_some(children.as_slice())
        })
        .collect()
}

fn rich_text(nodes: &[RichNode]) -> String {
    render_plain_text(&RichDocument::new(nodes.to_vec()))
}

fn split_remaster_intro_alias_segment(plain_text: &str) -> String {
    for delimiter in [
        " are merged into ",
        " is merged into ",
        " are now ",
        " is now ",
    ] {
        if let Some((segment, _)) = plain_text.split_once(delimiter) {
            return segment.trim().to_string();
        }
    }
    plain_text.trim().to_string()
}

fn split_alias_list_text(value: &str) -> Vec<String> {
    value
        .replace(" and ", ", ")
        .split(',')
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(str::to_string)
        .collect()
}

pub(super) fn expand_grouped_alias_text(
    alias_text: &str,
    expected_count: usize,
) -> Option<Vec<String>> {
    let open = alias_text.rfind('(')?;
    let close = alias_text.rfind(')')?;
    if close <= open {
        return None;
    }
    let base_name = alias_text[..open].trim();
    let variants = split_alias_list_text(&alias_text[open + 1..close]);
    if base_name.is_empty() || variants.len() != expected_count {
        return None;
    }
    Some(
        variants
            .into_iter()
            .map(|variant| format!("{base_name} ({variant})"))
            .collect(),
    )
}
