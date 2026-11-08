//! Display evidence derived from the checked passage, independent of search input.
use super::{
    SourceSelectedSection, SourceSelectedSectionKind, SourceSelectionError, checked_html_section,
    hash, heading_level, partition_prepared_html, plain, recover_source_passage,
};
use atlas_domain::{SourceByteRange, SourcePassageAddress};
use ego_tree::NodeId;
use scraper::{ElementRef, Html, Selector};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum SourceMatchPreview {
    Text {
        blocks: Vec<String>,
        truncated: bool,
    },
    Table {
        columns: Vec<String>,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourceMatchEvidence {
    pub preview: Option<SourceMatchPreview>,
    pub parent_heading: Option<String>,
}

/// Project multiple addresses for the same field using one prepared partition.
/// An ambiguous layout yields a destination without an excerpt.
pub fn recover_source_match_previews(
    html: Option<&str>,
    plain_source: Option<&str>,
    addresses: &[&SourcePassageAddress],
) -> Result<Vec<Option<SourceMatchEvidence>>, SourceSelectionError> {
    if !addresses
        .iter()
        .any(|address| matches!(address, SourcePassageAddress::HtmlSection { .. }))
    {
        return addresses
            .iter()
            .map(|address| {
                let text = recover_source_passage(None, plain_source, address)?;
                Ok(Some(SourceMatchEvidence {
                    preview: compact_blocks(text.lines(), false),
                    parent_heading: None,
                }))
            })
            .collect();
    }
    let html = html.ok_or_else(|| SourceSelectionError("prepared HTML is unavailable".into()))?;
    let document = Html::parse_fragment(html);
    let partition = partition_prepared_html(&document, html)?;
    let prepared_hash = hash(html);
    let selectors = PreviewSelectors::new()?;
    let mut layouts = BTreeMap::new();
    addresses
        .iter()
        .map(|address| {
            let (section, range) =
                checked_html_section(&prepared_hash, &partition.sections, address)?;
            let slice = range
                .extract(&section.text)
                .map_err(|e| SourceSelectionError(e.to_string()))?;
            let layout = layouts.entry(section.section_ordinal).or_insert_with(|| {
                let nodes = partition
                    .section_nodes
                    .iter()
                    .filter_map(|(node, ordinal)| {
                        (*ordinal == section.section_ordinal).then_some(*node)
                    });
                section_layout(&document, section, nodes, &selectors)
            });
            let preview = match layout {
                SectionLayout::Table(columns) => Some(SourceMatchPreview::Table {
                    columns: columns.clone(),
                }),
                SectionLayout::Prose { heading_end } => {
                    if range.start < *heading_end {
                        let text = &section.text[range.end.min(*heading_end)..range.end];
                        compact_blocks(text.lines(), false)
                    } else {
                        compact_blocks(slice.lines(), false)
                    }
                }
                SectionLayout::Aligned(blocks) => aligned_preview(section, range, blocks),
                SectionLayout::Unavailable => None,
            };
            let parent_heading = section
                .heading_chain
                .iter()
                .rev()
                .find(|heading| section.label.as_ref() != Some(*heading))
                .cloned();
            Ok(Some(SourceMatchEvidence {
                preview,
                parent_heading,
            }))
        })
        .collect()
}

struct PreviewSelectors {
    table: Selector,
    header_rows: Selector,
    rows: Selector,
    interaction: Selector,
}
impl PreviewSelectors {
    fn new() -> Result<Self, SourceSelectionError> {
        let parse = |selector| {
            Selector::parse(selector).map_err(|err| SourceSelectionError(err.to_string()))
        };
        Ok(Self {
            table: parse("table")?,
            header_rows: parse(":scope > thead > tr")?,
            rows: parse(":scope > tr, :scope > tbody > tr")?,
            interaction: parse("[data-atlas-interaction]")?,
        })
    }
}

enum SectionLayout {
    Table(Vec<String>),
    Prose { heading_end: usize },
    Aligned(Vec<AlignedBlock>),
    Unavailable,
}

#[derive(Clone, Copy)]
enum BlockKind {
    Heading,
    Prose,
    Omit,
}
struct AlignedBlock {
    range: SourceByteRange,
    kind: BlockKind,
}

fn section_layout(
    document: &Html,
    section: &SourceSelectedSection,
    nodes: impl Iterator<Item = NodeId>,
    selectors: &PreviewSelectors,
) -> SectionLayout {
    let nodes = nodes
        .filter_map(|id| document.tree.get(id))
        .collect::<Vec<_>>();
    if nodes.is_empty() {
        return SectionLayout::Unavailable;
    }
    let mut tables = Vec::new();
    let mut other_content = false;
    let mut needs_alignment = false;
    let mut heading_end = 0;
    for node in &nodes {
        let Some(element) = ElementRef::wrap(*node) else {
            other_content |= node
                .value()
                .as_text()
                .is_some_and(|text| !text.trim().is_empty());
            continue;
        };
        if heading_level(element.value().name()).is_some()
            || matches!(element.value().name(), "hr" | "caption")
        {
            if heading_end == 0
                && is_display_heading(section, element)
                && let Some(heading) = section.text.lines().next()
                && section.label.as_deref() == Some(heading)
            {
                heading_end = heading.len();
            }
            continue;
        }
        if element.value().name() == "table" {
            tables.push(element);
            needs_alignment = true;
        } else {
            other_content = true;
            needs_alignment |= element.select(&selectors.table).next().is_some()
                || has_unresolved_interaction(element, selectors);
        }
    }
    if !other_content && let [table] = tables.as_slice() {
        return SectionLayout::Table(table_columns(*table, selectors));
    }
    if !needs_alignment {
        return SectionLayout::Prose { heading_end };
    }
    // Independently rendered nodes must cover the same canonical text in order.
    // If the renderer combined nodes or changed their layout, decline the preview.
    let mut cursor = 0;
    let mut blocks = Vec::new();
    for node in nodes {
        let (text, kind) = if let Some(element) = ElementRef::wrap(node) {
            let Ok(text) = plain(&element.html()) else {
                return SectionLayout::Unavailable;
            };
            let kind = if is_display_heading(section, element) && blocks.is_empty() {
                BlockKind::Heading
            } else if matches!(element.value().name(), "table" | "hr")
                || element.select(&selectors.table).next().is_some()
                || has_unresolved_interaction(element, selectors)
            {
                BlockKind::Omit
            } else {
                BlockKind::Prose
            };
            (text, kind)
        } else if let Some(text) = node.value().as_text() {
            (text.trim().to_owned(), BlockKind::Prose)
        } else {
            continue;
        };
        if text.is_empty() {
            continue;
        }
        let rest = &section.text[cursor..];
        let tail = rest.trim_start();
        let start = cursor + rest.len() - tail.len();
        if !tail.starts_with(&text) {
            return SectionLayout::Unavailable;
        }
        cursor = start + text.len();
        blocks.push(AlignedBlock {
            range: SourceByteRange { start, end: cursor },
            kind,
        });
    }
    if section.text[cursor..].trim().is_empty() {
        SectionLayout::Aligned(blocks)
    } else {
        SectionLayout::Unavailable
    }
}

fn is_display_heading(section: &SourceSelectedSection, element: ElementRef<'_>) -> bool {
    section.kind == SourceSelectedSectionKind::Heading
        && heading_level(element.value().name()).is_some()
        && section.label.as_deref() == Some(element.text().collect::<String>().trim())
}

fn has_unresolved_interaction(element: ElementRef<'_>, selectors: &PreviewSelectors) -> bool {
    std::iter::once(element)
        .filter(|node| node.value().attr("data-atlas-interaction").is_some())
        .chain(element.select(&selectors.interaction))
        .any(|node| {
            // Prepared contextual interactions with an exposed Foundry operand
            // are unusable as excerpts. The full reader owns their interpretation.
            node.text().any(|text| {
                ["@actor.", "@item.", "@self.", "@kingdom."]
                    .iter()
                    .any(|prefix| text.contains(prefix))
            })
        })
}

fn table_columns(table: ElementRef<'_>, selectors: &PreviewSelectors) -> Vec<String> {
    if table.select(&selectors.table).next().is_some() {
        return Vec::new();
    }
    let mut header = table.select(&selectors.header_rows);
    let row = match header.next() {
        Some(row) if header.next().is_none() => Some(row),
        Some(_) => None,
        None => table.select(&selectors.rows).next(),
    };
    let Some(row) = row else { return Vec::new() };
    let cells = row.child_elements().collect::<Vec<_>>();
    if cells.is_empty() || cells.len() > 8 {
        return Vec::new();
    }
    cells
        .into_iter()
        .map(|cell| {
            if cell.value().name() != "th"
                || cell
                    .value()
                    .attr("scope")
                    .is_some_and(|scope| scope != "col")
                || cell
                    .value()
                    .attr("colspan")
                    .is_some_and(|value| value != "1")
                || cell
                    .value()
                    .attr("rowspan")
                    .is_some_and(|value| value != "1")
                || has_unresolved_interaction(cell, selectors)
            {
                return None;
            }
            let text = cell.text().collect::<String>();
            let label = text.split_whitespace().collect::<Vec<_>>().join(" ");
            (!label.is_empty() && label.chars().count() <= 40).then_some(label)
        })
        .collect::<Option<_>>()
        .unwrap_or_default()
}

fn aligned_preview(
    section: &SourceSelectedSection,
    window: SourceByteRange,
    blocks: &[AlignedBlock],
) -> Option<SourceMatchPreview> {
    let mut omitted = false;
    let mut text = Vec::new();
    for block in blocks {
        let start = window.start.max(block.range.start);
        let end = window.end.min(block.range.end);
        if start >= end {
            continue;
        }
        match block.kind {
            BlockKind::Heading => {}
            BlockKind::Omit => omitted = true,
            BlockKind::Prose if !omitted || text.is_empty() => text.push(&section.text[start..end]),
            BlockKind::Prose => break,
        }
    }
    compact_blocks(text.into_iter().flat_map(str::lines), omitted)
}

fn compact_blocks<'a>(
    lines: impl IntoIterator<Item = &'a str>,
    mut truncated: bool,
) -> Option<SourceMatchPreview> {
    const LIMIT: usize = 240;
    const MAX_BLOCKS: usize = 5;
    let mut remaining = LIMIT;
    let mut blocks = Vec::new();
    for line in lines {
        let line = line.split_whitespace().collect::<Vec<_>>().join(" ");
        if line.is_empty() {
            continue;
        }
        if remaining == 0 || blocks.len() == MAX_BLOCKS {
            truncated = true;
            break;
        }
        if let Some((end, _)) = line.char_indices().nth(remaining) {
            if blocks.is_empty() {
                let end = line[..end]
                    .rfind(' ')
                    .filter(|end| *end != 0)
                    .unwrap_or(end);
                blocks.push(line[..end].trim_end().to_owned());
            }
            truncated = true;
            break;
        }
        remaining -= line.chars().count();
        blocks.push(line);
    }
    (!blocks.is_empty()).then_some(SourceMatchPreview::Text { blocks, truncated })
}

#[cfg(test)]
mod tests;
