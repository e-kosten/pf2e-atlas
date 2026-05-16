use atlas_domain::RecordKey;
use serde_json::Value;

use super::html::{html_cells, html_elements, html_text};
use crate::records::references::{
    extract_reference_candidates_from_text, record_by_key, reference_pack_and_locator,
    resolve_record_key,
};
use crate::records::{NormalizedRecord, RecordReferenceIndex};
use crate::source::normalize::{normalize_text, pointer_string};

pub(super) struct JournalRemasterChange {
    pub(super) remaster_record_key: RecordKey,
    pub(super) legacy_name: String,
    pub(super) source_ref: String,
}

pub(super) fn extract_remaster_journal_changes(
    record: &NormalizedRecord,
    index: &RecordReferenceIndex,
) -> Vec<JournalRemasterChange> {
    let Ok(raw) = serde_json::from_str::<Value>(&record.raw_json) else {
        return Vec::new();
    };
    let Some(pages) = raw.pointer("/pages").and_then(Value::as_array) else {
        return Vec::new();
    };

    let mut changes = Vec::new();
    for page in pages {
        let page_name = pointer_string(page, "/name").unwrap_or_else(|| "journal-page".to_string());
        let source_ref = format!("journal:{page_name}");
        let Some(content) = pointer_string(page, "/text/content") else {
            continue;
        };

        if page_name == "Remaster Changes" {
            for list_item in html_elements(&content, "li") {
                let targets = resolve_journal_targets(&list_item, index);
                if targets.len() != 1 {
                    continue;
                }
                let plain = html_text(&list_item);
                let old_segment = split_remaster_intro_alias_segment(&plain);
                for alias_text in split_alias_list_text(&old_segment) {
                    changes.push(JournalRemasterChange {
                        remaster_record_key: targets[0].clone(),
                        legacy_name: alias_text,
                        source_ref: source_ref.clone(),
                    });
                }
            }
        }

        for row in html_elements(&content, "tr") {
            let cells = html_cells(&row);
            if cells.len() < 2 {
                continue;
            }
            let status_cell = if cells.len() >= 4 {
                &cells[2]
            } else {
                "Renamed"
            };
            let status = normalize_text(&html_text(status_cell));
            if !matches!(status.as_str(), "renamed" | "merged" | "replaced") {
                continue;
            }

            let old_cell = &cells[0];
            let new_cell = cells.last().expect("row should have at least two cells");
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

            let old_text = html_text(old_cell);
            let Some(grouped_aliases) = expand_grouped_alias_text(&old_text, targets.len()) else {
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

fn resolve_journal_targets(cell_html: &str, index: &RecordReferenceIndex) -> Vec<RecordKey> {
    let candidates = extract_reference_candidates_from_text(cell_html);
    if candidates.is_empty() {
        let plain = html_text(cell_html);
        return resolve_record_key(None, &plain, index)
            .into_iter()
            .collect();
    }

    let mut targets = Vec::new();
    for candidate in candidates {
        let Some((pack_name, locator)) = reference_pack_and_locator(&candidate.raw_target) else {
            continue;
        };
        let Some(record_key) = resolve_record_key(Some(&pack_name), &locator, index) else {
            continue;
        };
        if record_by_key(index, &record_key)
            .is_some_and(|record| record.foundry_document_type != "JournalEntry")
        {
            targets.push(record_key);
        }
    }
    targets
}

fn resolve_alias_source_name(cell_html: &str, index: &RecordReferenceIndex) -> Option<String> {
    let direct_text = html_text(cell_html);
    if !direct_text.is_empty() && !cell_html.contains("@UUID[") {
        return Some(direct_text);
    }

    let candidate = extract_reference_candidates_from_text(cell_html)
        .into_iter()
        .next()?;
    let (pack_name, locator) = reference_pack_and_locator(&candidate.raw_target)?;
    let record_key = resolve_record_key(Some(&pack_name), &locator, index)?;
    record_by_key(index, &record_key).map(|record| record.name.clone())
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
