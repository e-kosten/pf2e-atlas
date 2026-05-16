use std::collections::{BTreeMap, BTreeSet};

use crate::structured_units::markup::{
    ListItem, MarkupBlock, normalize_split_label, normalized_token_count, render_blocks_for_unit,
};
use crate::unit_kind::EmbeddingUnitKind;

const CHILD_SPLIT_DOCUMENT_TOKEN_THRESHOLD: usize = 384;
const CHILD_SPLIT_TARGET_TOKEN_COUNT: usize = 256;
const MIN_HEADING_SECTION_TOKENS: usize = 25;
const MIN_TITLED_OPTION_TOKENS: usize = 20;
const MAX_HEADING_LEVEL: u8 = 3;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct StructuredEmbeddingUnit {
    pub(crate) kind: EmbeddingUnitKind,
    pub(crate) label: String,
    pub(crate) ordinal: usize,
    pub(crate) body: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct TitledOptionCandidate {
    label: String,
    body: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct HeadingSectionRange {
    level: u8,
    label: String,
    label_is_valid: bool,
    ordinal: usize,
    start: usize,
    end: usize,
    body_token_count: usize,
}

pub(crate) fn extract_structured_embedding_units(
    record_name: &str,
    markup: &str,
) -> Vec<StructuredEmbeddingUnit> {
    let blocks = super::markup::markup_blocks(markup);
    if normalized_token_count(&render_blocks_for_unit(&blocks))
        <= CHILD_SPLIT_DOCUMENT_TOKEN_THRESHOLD
    {
        return Vec::new();
    }
    let normalized_record_name = normalize_split_label(record_name);
    let mut units = extract_adaptive_heading_units(&blocks, &normalized_record_name);
    if units.is_empty() {
        units.extend(extract_titled_option_units(&blocks, 0, blocks.len()));
    }
    units.sort_by_key(|unit| (unit.ordinal, unit.kind));
    reassign_ordinals_by_source_order(units)
}

fn reassign_ordinals_by_source_order(
    units: Vec<StructuredEmbeddingUnit>,
) -> Vec<StructuredEmbeddingUnit> {
    let mut counts = BTreeMap::<EmbeddingUnitKind, usize>::new();
    units
        .into_iter()
        .map(|mut unit| {
            let count = counts.entry(unit.kind).or_insert(0);
            *count += 1;
            unit.ordinal = *count;
            unit
        })
        .collect()
}

fn extract_adaptive_heading_units(
    blocks: &[MarkupBlock],
    normalized_record_name: &str,
) -> Vec<StructuredEmbeddingUnit> {
    dedupe_units(plan_heading_units(
        blocks,
        0,
        blocks.len(),
        normalized_record_name,
    ))
}

fn plan_heading_units(
    blocks: &[MarkupBlock],
    start: usize,
    end: usize,
    normalized_record_name: &str,
) -> Vec<StructuredEmbeddingUnit> {
    let sections = immediate_heading_sections(blocks, start, end, normalized_record_name);
    if sections.is_empty() {
        return Vec::new();
    }
    let mut units = Vec::new();

    for section in sections {
        if !section.label_is_valid {
            let nested = plan_heading_units(
                blocks,
                section.start + 1,
                section.end,
                normalized_record_name,
            );
            if nested.len() >= 2 {
                units.extend(nested);
                continue;
            }

            let titled_options =
                extract_titled_option_units(blocks, section.start + 1, section.end);
            if titled_options.len() >= 3 {
                units.extend(titled_options);
            }
            continue;
        }

        if section.body_token_count > CHILD_SPLIT_TARGET_TOKEN_COUNT {
            let nested = plan_heading_units(
                blocks,
                section.start + 1,
                section.end,
                normalized_record_name,
            );
            if nested.len() >= 2 {
                units.extend(nested);
                continue;
            }

            let titled_options =
                extract_titled_option_units(blocks, section.start + 1, section.end);
            if titled_options.len() >= 3 {
                units.extend(titled_options);
                continue;
            }
        }

        let body = render_blocks_for_unit(&blocks[section.start + 1..section.end]);
        units.push(StructuredEmbeddingUnit {
            kind: EmbeddingUnitKind::HeadingSection,
            label: section.label,
            ordinal: section.ordinal,
            body,
        });
    }

    units
}

fn immediate_heading_sections(
    blocks: &[MarkupBlock],
    start: usize,
    end: usize,
    normalized_record_name: &str,
) -> Vec<HeadingSectionRange> {
    let Some(level) = immediate_heading_level(blocks, start, end) else {
        return Vec::new();
    };
    let mut sections = Vec::new();
    let mut index = start;
    while index < end {
        let MarkupBlock::Heading {
            level: candidate_level,
            label,
        } = &blocks[index]
        else {
            index += 1;
            continue;
        };
        if *candidate_level != level {
            index += 1;
            continue;
        }

        let section_end = heading_section_end(blocks, index, end, level);
        let normalized_label = normalize_split_label(label);
        let body = render_blocks_for_unit(&blocks[index + 1..section_end]);
        let body_token_count = normalized_token_count(&body);
        if body_token_count >= MIN_HEADING_SECTION_TOKENS {
            sections.push(HeadingSectionRange {
                level,
                label: label.clone(),
                label_is_valid: !normalized_label.is_empty()
                    && normalized_label != normalized_record_name,
                ordinal: index + 1,
                start: index,
                end: section_end,
                body_token_count,
            });
        }
        index = section_end;
    }
    sections
}

fn immediate_heading_level(blocks: &[MarkupBlock], start: usize, end: usize) -> Option<u8> {
    blocks[start..end].iter().find_map(|block| {
        if let MarkupBlock::Heading { level, .. } = block
            && matches!(*level, 1..=MAX_HEADING_LEVEL)
        {
            return Some(*level);
        }
        None
    })
}

fn heading_section_end(blocks: &[MarkupBlock], index: usize, end: usize, level: u8) -> usize {
    blocks[index + 1..end]
        .iter()
        .position(|candidate| {
            matches!(
                candidate,
                MarkupBlock::Heading {
                    level: candidate_level,
                    ..
                } if *candidate_level <= level && matches!(*candidate_level, 1..=MAX_HEADING_LEVEL)
            )
        })
        .map(|relative| index + 1 + relative)
        .unwrap_or(end)
}

fn extract_titled_option_units(
    blocks: &[MarkupBlock],
    start: usize,
    end: usize,
) -> Vec<StructuredEmbeddingUnit> {
    let mut units = Vec::new();
    for (relative_index, block) in blocks[start..end].iter().enumerate() {
        let block_index = start + relative_index;
        let MarkupBlock::List(items) = block else {
            continue;
        };
        let candidates = titled_option_candidates_from_list(items);
        if candidates.len() < 3 {
            continue;
        }
        for (item_index, candidate) in candidates.into_iter().enumerate() {
            units.push(StructuredEmbeddingUnit {
                kind: EmbeddingUnitKind::TitledOption,
                label: candidate.label,
                ordinal: block_index * 10_000 + item_index + 1,
                body: candidate.body,
            });
        }
    }
    let mut index = start;
    while index < end {
        let start = index;
        let candidates = collect_titled_paragraph_candidates(blocks, index, end);
        if candidates.len() < 3 {
            index += candidates.len().max(1);
            continue;
        }
        let candidate_count = candidates.len();
        for (candidate_index, candidate) in candidates.into_iter().enumerate() {
            units.push(StructuredEmbeddingUnit {
                kind: EmbeddingUnitKind::TitledOption,
                label: candidate.label,
                ordinal: start * 10_000 + candidate_index + 1,
                body: candidate.body,
            });
        }
        index = start + candidate_count;
    }
    dedupe_units(units)
}

fn collect_titled_paragraph_candidates(
    blocks: &[MarkupBlock],
    start: usize,
    end: usize,
) -> Vec<TitledOptionCandidate> {
    let mut candidates = Vec::new();
    let mut index = start;
    while index < end {
        let Some(candidate) = titled_option_candidate_from_paragraph(&blocks[index]) else {
            break;
        };
        candidates.push(candidate);
        index += 1;
    }
    candidates
}

fn titled_option_candidates_from_list(items: &[ListItem]) -> Vec<TitledOptionCandidate> {
    items
        .iter()
        .filter_map(|item| {
            let label = clean_option_label(item.label.as_deref()?)?;
            (normalized_token_count(&item.body) >= MIN_TITLED_OPTION_TOKENS).then_some(
                TitledOptionCandidate {
                    label,
                    body: item.body.clone(),
                },
            )
        })
        .collect()
}

fn titled_option_candidate_from_paragraph(block: &MarkupBlock) -> Option<TitledOptionCandidate> {
    let MarkupBlock::Paragraph {
        text,
        first_label: Some(label),
    } = block
    else {
        return None;
    };
    let label = clean_option_label(label)?;
    (normalized_token_count(text) >= MIN_TITLED_OPTION_TOKENS).then_some(TitledOptionCandidate {
        label,
        body: text.clone(),
    })
}

fn clean_option_label(label: &str) -> Option<String> {
    let label = label.trim().trim_end_matches(':').trim();
    let normalized = normalize_split_label(label);
    (!normalized.is_empty() && !is_mechanics_result_label(&normalized)).then(|| label.to_string())
}

fn is_mechanics_result_label(normalized_label: &str) -> bool {
    matches!(
        normalized_label,
        "critical success" | "success" | "failure" | "critical failure"
    )
}

fn dedupe_units(units: Vec<StructuredEmbeddingUnit>) -> Vec<StructuredEmbeddingUnit> {
    let mut seen = BTreeSet::<(EmbeddingUnitKind, String, String)>::new();
    let mut deduped = Vec::new();
    for unit in units {
        let key = (
            unit.kind,
            normalize_split_label(&unit.label),
            normalize_split_label(&unit.body),
        );
        if seen.insert(key) {
            deduped.push(unit);
        }
    }
    deduped
}
