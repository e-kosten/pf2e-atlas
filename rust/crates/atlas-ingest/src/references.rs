use std::collections::BTreeSet;

use atlas_domain::RecordKey;
use serde_json::Value;

use crate::normalize::{normalize_text, strip_markup};
use crate::{LoadedRecord, RecordReferenceIndex, ReferenceCandidate, ReferenceEdge};

pub(crate) fn build_record_reference_index(records: &[LoadedRecord]) -> RecordReferenceIndex {
    let mut index = RecordReferenceIndex::default();
    for record in records {
        index.by_key.insert(record.key.to_string(), record.clone());
        index.by_pack_id.insert(
            (
                record.pack_name.as_str().to_string(),
                record.id.as_str().to_string(),
            ),
            record.key.clone(),
        );
        index.by_pack_id.insert(
            (
                normalize_text(record.pack_name.as_str()),
                normalize_text(record.id.as_str()),
            ),
            record.key.clone(),
        );
        index
            .by_pack_name
            .entry((
                record.pack_name.as_str().to_string(),
                record.normalized_name.clone(),
            ))
            .or_default()
            .push(record.key.clone());
        index
            .by_name
            .entry(record.normalized_name.clone())
            .or_default()
            .push(record.key.clone());
    }
    index
}

pub(crate) fn resolve_reference_edges(
    records: &[LoadedRecord],
    index: &RecordReferenceIndex,
) -> Vec<ReferenceEdge> {
    let mut seen = BTreeSet::new();
    let mut references = Vec::new();
    for record in records {
        for candidate in &record.reference_candidates {
            let Some((pack_name, locator)) = reference_pack_and_locator(&candidate.raw_target)
            else {
                continue;
            };
            let to_record_key = index
                .by_pack_id
                .get(&(pack_name.clone(), locator.clone()))
                .or_else(|| {
                    index
                        .by_pack_name
                        .get(&(pack_name, normalize_text(&locator)))
                        .and_then(|matches| {
                            if matches.len() == 1 {
                                matches.first()
                            } else {
                                None
                            }
                        })
                });
            let Some(to_record_key) = to_record_key else {
                continue;
            };
            let dedupe_key = (
                record.key.to_string(),
                to_record_key.to_string(),
                candidate.reference_text.clone(),
            );
            if seen.insert(dedupe_key) {
                references.push(ReferenceEdge {
                    from_record_key: record.key.clone(),
                    to_record_key: to_record_key.clone(),
                    display_text: candidate.display_text.clone(),
                    reference_text: candidate.reference_text.clone(),
                });
            }
        }
    }

    references.sort_by(|left, right| {
        (
            left.from_record_key.to_string(),
            left.to_record_key.to_string(),
            left.reference_text.as_str(),
        )
            .cmp(&(
                right.from_record_key.to_string(),
                right.to_record_key.to_string(),
                right.reference_text.as_str(),
            ))
    });
    references
}

pub(crate) fn reference_pack_and_locator(raw_target: &str) -> Option<(String, String)> {
    let parts = raw_target.split('.').collect::<Vec<_>>();
    if parts.len() >= 5 && parts.first() == Some(&"Compendium") && parts.get(1) == Some(&"pf2e") {
        return Some((parts.get(2)?.to_string(), parts.last()?.to_string()));
    }
    if parts.len() >= 3 && parts.first() == Some(&"pf2e") {
        return Some((parts.get(1)?.to_string(), parts.last()?.to_string()));
    }
    None
}

pub(crate) fn resolve_record_key(
    pack_name: Option<&str>,
    locator_or_name: &str,
    index: &RecordReferenceIndex,
) -> Option<RecordKey> {
    let normalized = normalize_text(locator_or_name);
    if normalized.is_empty() {
        return None;
    }

    if let Some(pack_name) = pack_name {
        if let Some(record_key) = index
            .by_pack_id
            .get(&(pack_name.to_string(), locator_or_name.to_string()))
        {
            return Some(record_key.clone());
        }

        let matches = index
            .by_pack_name
            .get(&(pack_name.to_string(), normalized.clone()))?;
        return (matches.len() == 1).then(|| matches[0].clone());
    }

    let matches = index.by_name.get(&normalized)?;
    (matches.len() == 1).then(|| matches[0].clone())
}

pub(crate) fn record_by_key<'a>(
    index: &'a RecordReferenceIndex,
    record_key: &RecordKey,
) -> Option<&'a LoadedRecord> {
    index.by_key.get(&record_key.to_string())
}

pub(crate) fn extract_reference_candidates(raw: &Value) -> Vec<ReferenceCandidate> {
    let mut candidates = Vec::new();
    collect_reference_candidates(raw, &mut candidates);
    candidates
}

pub(crate) fn collect_reference_candidates(raw: &Value, candidates: &mut Vec<ReferenceCandidate>) {
    match raw {
        Value::Array(values) => {
            for value in values {
                collect_reference_candidates(value, candidates);
            }
        }
        Value::Object(values) => {
            for value in values.values() {
                collect_reference_candidates(value, candidates);
            }
        }
        Value::String(value) => candidates.extend(extract_reference_candidates_from_text(value)),
        _ => {}
    }
}

pub(crate) fn extract_reference_candidates_from_text(text: &str) -> Vec<ReferenceCandidate> {
    let mut candidates = Vec::new();
    let mut offset = 0;

    while offset < text.len() {
        let Some((start, prefix)) = next_reference_prefix(text, offset) else {
            break;
        };
        let target_start = start + prefix.len();
        let Some(close_relative) = text[target_start..].find(']') else {
            break;
        };
        let close = target_start + close_relative;
        let raw_target = text[target_start..close].to_string();
        let mut end = close + 1;
        let mut display_text = None;

        if text[end..].starts_with('{')
            && let Some(display_close_relative) = text[end + 1..].find('}')
        {
            let display_close = end + 1 + display_close_relative;
            let display = strip_markup(&text[end + 1..display_close]);
            if !display.is_empty() {
                display_text = Some(display);
            }
            end = display_close + 1;
        }

        candidates.push(ReferenceCandidate {
            raw_target,
            display_text,
            reference_text: text[start..end].to_string(),
        });
        offset = end;
    }

    candidates
}

pub(crate) fn next_reference_prefix(text: &str, offset: usize) -> Option<(usize, &'static str)> {
    ["@UUID[", "@Compendium["]
        .into_iter()
        .filter_map(|prefix| {
            text[offset..]
                .find(prefix)
                .map(|position| (offset + position, prefix))
        })
        .min_by_key(|(position, _)| *position)
}
