use std::collections::{BTreeMap, BTreeSet};

use atlas_embedding::EmbeddingUnitKind;

use crate::normalize::strip_markup;

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
enum MarkupBlock {
    Heading {
        level: u8,
        label: String,
    },
    Paragraph {
        text: String,
        first_label: Option<String>,
    },
    List(Vec<ListItem>),
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct ListItem {
    label: Option<String>,
    body: String,
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
    let blocks = markup_blocks(markup);
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

pub(crate) fn strip_markup_for_embedding_units(markup: &str) -> String {
    strip_markup(&drop_table_body_cells(markup))
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

fn markup_blocks(markup: &str) -> Vec<MarkupBlock> {
    let tokens = tokenize_markup(markup);
    let mut blocks = Vec::new();
    let mut index = 0;
    while index < tokens.len() {
        match tokens[index].as_str() {
            tag if heading_level(tag).is_some() && !tag.starts_with("</") => {
                let level = heading_level(tag).expect("checked heading level");
                let (inner, next) = collect_until_closing(&tokens, index + 1, &format!("h{level}"));
                let label = strip_markup_for_embedding_units(&inner.join(""));
                if !label.trim().is_empty() {
                    blocks.push(MarkupBlock::Heading { level, label });
                }
                index = next;
            }
            tag if tag_open_name(tag) == Some("p") => {
                let (inner, next) = collect_until_closing(&tokens, index + 1, "p");
                let html = inner.join("");
                let text = strip_markup_for_embedding_units(&html);
                if !text.trim().is_empty() {
                    blocks.push(MarkupBlock::Paragraph {
                        first_label: first_strong_label(&html),
                        text,
                    });
                }
                index = next;
            }
            tag if matches!(tag_open_name(tag), Some("ul" | "ol")) => {
                let (inner, next) =
                    collect_until_closing(&tokens, index + 1, tag_open_name(tag).unwrap());
                let items = list_items_from_tokens(&inner);
                if !items.is_empty() {
                    blocks.push(MarkupBlock::List(items));
                }
                index = next;
            }
            tag if tag_open_name(tag) == Some("table") => {
                let (inner, next) = collect_until_closing(&tokens, index + 1, "table");
                let table_text = render_table_for_embedding(&inner.join(""));
                if !table_text.is_empty() {
                    blocks.push(MarkupBlock::Paragraph {
                        text: table_text,
                        first_label: None,
                    });
                }
                index = next;
            }
            token if token.starts_with('<') => {
                index += 1;
            }
            _ => {
                let mut text = String::new();
                while index < tokens.len() && !tokens[index].starts_with('<') {
                    text.push_str(&tokens[index]);
                    index += 1;
                }
                let text = strip_markup_for_embedding_units(&text);
                if !text.trim().is_empty() {
                    blocks.push(MarkupBlock::Paragraph {
                        text,
                        first_label: None,
                    });
                }
            }
        }
    }
    blocks
}

fn list_items_from_tokens(tokens: &[String]) -> Vec<ListItem> {
    let mut items = Vec::new();
    let mut index = 0;
    while index < tokens.len() {
        match tag_open_name(&tokens[index]) {
            Some("li") => {
                let (inner, next) = collect_until_closing(tokens, index + 1, "li");
                let html = inner.join("");
                let label = first_strong_label(&html);
                let body = strip_markup_for_embedding_units(&html);
                if !body.trim().is_empty() {
                    items.push(ListItem { label, body });
                }
                index = next;
            }
            Some("ul" | "ol") => {
                let (_, next) = collect_until_closing(
                    tokens,
                    index + 1,
                    tag_open_name(&tokens[index]).unwrap(),
                );
                index = next;
            }
            _ => {
                index += 1;
            }
        }
    }
    items
}

fn render_blocks_for_unit(blocks: &[MarkupBlock]) -> String {
    blocks
        .iter()
        .map(|block| match block {
            MarkupBlock::Heading { label, .. } => label.clone(),
            MarkupBlock::Paragraph { text, .. } => text.clone(),
            MarkupBlock::List(items) => items
                .iter()
                .map(|item| item.body.as_str())
                .collect::<Vec<_>>()
                .join(" "),
        })
        .collect::<Vec<_>>()
        .join(" ")
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
}

fn drop_table_body_cells(markup: &str) -> String {
    let tokens = tokenize_markup(markup);
    let mut output = String::new();
    let mut index = 0;
    while index < tokens.len() {
        if tag_open_name(&tokens[index]) == Some("td") {
            let (_, next) = collect_until_closing(&tokens, index + 1, "td");
            index = next;
            continue;
        }
        output.push_str(&tokens[index]);
        index += 1;
    }
    output
}

fn render_table_for_embedding(table_markup: &str) -> String {
    let tokens = tokenize_markup(table_markup);
    let mut captions = Vec::new();
    let mut headers = Vec::new();
    let mut index = 0;
    while index < tokens.len() {
        match tag_open_name(&tokens[index]) {
            Some("caption") => {
                let (inner, next) = collect_until_closing(&tokens, index + 1, "caption");
                let text = strip_markup_for_embedding_units(&inner.join(""));
                if !text.is_empty() {
                    captions.push(text);
                }
                index = next;
            }
            Some("th") => {
                let (inner, next) = collect_until_closing(&tokens, index + 1, "th");
                let text = strip_markup_for_embedding_units(&inner.join(""));
                if !text.is_empty() {
                    headers.push(text);
                }
                index = next;
            }
            _ => index += 1,
        }
    }
    let mut parts = Vec::new();
    if !captions.is_empty() {
        parts.push(format!("Table: {}", captions.join(" | ")));
    }
    if !headers.is_empty() {
        parts.push(format!("Columns: {}", headers.join(" | ")));
    }
    parts.join(" ")
}

fn first_strong_label(html: &str) -> Option<String> {
    let tokens = tokenize_markup(html);
    let mut index = 0;
    while index < tokens.len() {
        if tag_open_name(&tokens[index]) == Some("strong") {
            let (inner, _) = collect_until_closing(&tokens, index + 1, "strong");
            let label = strip_markup_for_embedding_units(&inner.join(""));
            return (!label.trim().is_empty()).then_some(label);
        }
        if tokens[index].starts_with('<') || tokens[index].trim().is_empty() {
            index += 1;
            continue;
        }
        break;
    }
    None
}

fn tokenize_markup(markup: &str) -> Vec<String> {
    let mut tokens = Vec::new();
    let mut offset = 0;
    while offset < markup.len() {
        let rest = &markup[offset..];
        if rest.starts_with('<')
            && let Some(end) = rest.find('>')
        {
            tokens.push(rest[..=end].to_string());
            offset += end + 1;
            continue;
        }
        let next_tag = rest.find('<').unwrap_or(rest.len());
        if next_tag > 0 {
            tokens.push(rest[..next_tag].to_string());
            offset += next_tag;
        } else {
            offset += 1;
        }
    }
    tokens
}

fn collect_until_closing(tokens: &[String], start: usize, tag_name: &str) -> (Vec<String>, usize) {
    let mut depth = 1usize;
    let mut inner = Vec::new();
    let mut index = start;
    while index < tokens.len() {
        let token = &tokens[index];
        if tag_open_name(token) == Some(tag_name) {
            depth += 1;
        } else if tag_close_name(token) == Some(tag_name) {
            depth -= 1;
            if depth == 0 {
                return (inner, index + 1);
            }
        }
        inner.push(token.clone());
        index += 1;
    }
    (inner, index)
}

fn heading_level(tag: &str) -> Option<u8> {
    let name = tag_open_name(tag)?;
    let suffix = name.strip_prefix('h')?;
    let level = suffix.parse::<u8>().ok()?;
    (1..=6).contains(&level).then_some(level)
}

fn tag_open_name(tag: &str) -> Option<&str> {
    if !tag.starts_with('<') || tag.starts_with("</") || tag.starts_with("<!") {
        return None;
    }
    tag[1..]
        .split(|character: char| character.is_ascii_whitespace() || character == '>')
        .next()
        .map(|name| name.trim_end_matches('/'))
        .filter(|name| !name.is_empty())
}

fn tag_close_name(tag: &str) -> Option<&str> {
    if !tag.starts_with("</") {
        return None;
    }
    tag[2..]
        .split(|character: char| character.is_ascii_whitespace() || character == '>')
        .next()
        .filter(|name| !name.is_empty())
}

fn normalize_split_label(value: &str) -> String {
    strip_markup_for_embedding_units(value)
        .trim_matches(|character: char| {
            character.is_ascii_punctuation() || character.is_whitespace()
        })
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
        .to_lowercase()
}

fn normalized_token_count(value: &str) -> usize {
    normalize_split_label(value).split_whitespace().count()
}
