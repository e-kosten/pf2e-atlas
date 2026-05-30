use atlas_record::{
    ContentBlock, ContentDefinitionItem, ContentDocument, ContentInline, ContentReference,
    ContentReferenceLocator,
};

use super::content_diagnostics::{ContentParseDiagnostics, DroppedContentMacro};
use super::content_html::{HtmlNode, parse_html_fragment};

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ParsedContentDocument {
    pub(crate) document: ContentDocument,
    pub(crate) diagnostics: ContentParseDiagnostics,
}

pub(crate) fn parse_foundry_content(value: &str) -> ParsedContentDocument {
    let nodes = parse_html_fragment(value);
    let mut diagnostics = ContentParseDiagnostics::default();
    let blocks = nodes_to_blocks(&nodes, &mut diagnostics);

    ParsedContentDocument {
        document: ContentDocument::new(blocks),
        diagnostics,
    }
}

fn nodes_to_blocks(
    nodes: &[HtmlNode],
    diagnostics: &mut ContentParseDiagnostics,
) -> Vec<ContentBlock> {
    let mut blocks = Vec::new();
    let mut pending_inline = Vec::new();

    for node in nodes {
        let node_blocks = node_to_blocks(node, diagnostics);
        if node_blocks.is_empty() {
            pending_inline.extend(node_to_inlines(node, diagnostics));
        } else {
            push_pending_paragraph(&mut blocks, &mut pending_inline);
            blocks.extend(node_blocks);
        }
    }

    push_pending_paragraph(&mut blocks, &mut pending_inline);
    blocks
}

fn node_to_blocks(node: &HtmlNode, diagnostics: &mut ContentParseDiagnostics) -> Vec<ContentBlock> {
    match node {
        HtmlNode::Text(_) => Vec::new(),
        HtmlNode::Element { name, children } => match name.as_str() {
            "h1" | "h2" | "h3" | "h4" | "h5" | "h6" => vec![ContentBlock::Heading {
                level: name[1..].parse::<u8>().unwrap_or(1),
                content: nodes_to_inlines(children, diagnostics),
            }],
            "p" => paragraph_from_inlines(nodes_to_inlines(children, diagnostics)),
            "ul" | "ol" => vec![ContentBlock::List {
                ordered: name == "ol",
                items: children
                    .iter()
                    .filter_map(|child| list_item_blocks(child, diagnostics))
                    .collect(),
            }],
            "table" => table_block(children, diagnostics).into_iter().collect(),
            "dl" => vec![ContentBlock::DefinitionList {
                items: definition_items(children, diagnostics),
            }],
            "blockquote" => vec![ContentBlock::Callout {
                title: None,
                blocks: nodes_to_blocks(children, diagnostics),
            }],
            "section" | "article" | "div" | "main" | "body" => {
                nodes_to_blocks(children, diagnostics)
            }
            "hr" => vec![ContentBlock::Separator],
            "br" | "strong" | "b" | "em" | "i" | "span" | "a" | "code" => Vec::new(),
            other => {
                diagnostics.record_unsupported_tag(other);
                let blocks = nodes_to_blocks(children, diagnostics);
                if blocks.is_empty() {
                    paragraph_from_inlines(nodes_to_inlines(children, diagnostics))
                } else {
                    blocks
                }
            }
        },
    }
}

fn list_item_blocks(
    node: &HtmlNode,
    diagnostics: &mut ContentParseDiagnostics,
) -> Option<Vec<ContentBlock>> {
    let HtmlNode::Element { name, children } = node else {
        return None;
    };
    if name != "li" {
        return None;
    }
    let blocks = nodes_to_blocks(children, diagnostics);
    if blocks.is_empty() {
        Some(paragraph_from_inlines(nodes_to_inlines(
            children,
            diagnostics,
        )))
    } else {
        Some(blocks)
    }
}

fn table_block(
    children: &[HtmlNode],
    diagnostics: &mut ContentParseDiagnostics,
) -> Option<ContentBlock> {
    let caption = children.iter().find_map(|child| match child {
        HtmlNode::Element { name, children } if name == "caption" => {
            Some(nodes_to_inlines(children, diagnostics))
        }
        _ => None,
    });
    let table_rows = collect_table_rows(children, diagnostics);
    if table_rows.is_empty() && caption.is_none() {
        return None;
    }

    let mut headers = Vec::new();
    let mut rows = Vec::new();
    for row in table_rows {
        if headers.is_empty() && row.has_header_cells {
            headers = row.cells;
        } else {
            rows.push(row.cells);
        }
    }

    Some(ContentBlock::Table {
        caption,
        headers,
        rows,
    })
}

fn collect_table_rows(
    children: &[HtmlNode],
    diagnostics: &mut ContentParseDiagnostics,
) -> Vec<TableRow> {
    let mut rows = Vec::new();
    for child in children {
        match child {
            HtmlNode::Element { name, children } if name == "tr" => {
                if let Some(row) = table_row(children, diagnostics) {
                    rows.push(row);
                }
            }
            HtmlNode::Element { name, children }
                if matches!(name.as_str(), "thead" | "tbody" | "tfoot") =>
            {
                rows.extend(collect_table_rows(children, diagnostics));
            }
            _ => {}
        }
    }
    rows
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct TableRow {
    has_header_cells: bool,
    cells: Vec<Vec<ContentInline>>,
}

fn table_row(children: &[HtmlNode], diagnostics: &mut ContentParseDiagnostics) -> Option<TableRow> {
    let mut has_header_cells = false;
    let mut cells = Vec::new();
    for child in children {
        if let HtmlNode::Element { name, children } = child
            && matches!(name.as_str(), "th" | "td")
        {
            has_header_cells |= name == "th";
            cells.push(nodes_to_inlines(children, diagnostics));
        }
    }

    (!cells.is_empty()).then_some(TableRow {
        has_header_cells,
        cells,
    })
}

fn definition_items(
    children: &[HtmlNode],
    diagnostics: &mut ContentParseDiagnostics,
) -> Vec<ContentDefinitionItem> {
    let mut items = Vec::new();
    let mut pending_term: Option<Vec<ContentInline>> = None;

    for child in children {
        match child {
            HtmlNode::Element { name, children } if name == "dt" => {
                pending_term = Some(nodes_to_inlines(children, diagnostics));
            }
            HtmlNode::Element { name, children } if name == "dd" => {
                if let Some(term) = pending_term.take() {
                    items.push(ContentDefinitionItem {
                        term,
                        definition: nodes_to_blocks(children, diagnostics),
                    });
                }
            }
            _ => {}
        }
    }

    items
}

fn paragraph_from_inlines(content: Vec<ContentInline>) -> Vec<ContentBlock> {
    if content.is_empty() {
        Vec::new()
    } else {
        vec![ContentBlock::Paragraph { content }]
    }
}

fn push_pending_paragraph(blocks: &mut Vec<ContentBlock>, pending_inline: &mut Vec<ContentInline>) {
    if pending_inline.is_empty() {
        return;
    }

    blocks.push(ContentBlock::Paragraph {
        content: std::mem::take(pending_inline),
    });
}

fn nodes_to_inlines(
    nodes: &[HtmlNode],
    diagnostics: &mut ContentParseDiagnostics,
) -> Vec<ContentInline> {
    nodes
        .iter()
        .flat_map(|node| node_to_inlines(node, diagnostics))
        .collect()
}

fn node_to_inlines(
    node: &HtmlNode,
    diagnostics: &mut ContentParseDiagnostics,
) -> Vec<ContentInline> {
    match node {
        HtmlNode::Text(text) => parse_text_inlines(text, diagnostics),
        HtmlNode::Element { name, children } => match name.as_str() {
            "strong" | "b" => vec![ContentInline::Strong {
                content: nodes_to_inlines(children, diagnostics),
            }],
            "em" | "i" => vec![ContentInline::Emphasis {
                content: nodes_to_inlines(children, diagnostics),
            }],
            "code" => vec![ContentInline::Code {
                text: text_content(children),
            }],
            "br" => vec![ContentInline::Break],
            "img" => vec![ContentInline::Icon {
                name: "image".to_string(),
                label: None,
            }],
            "a" | "span" | "small" | "sub" | "sup" => nodes_to_inlines(children, diagnostics),
            _ if is_block_tag(name) => Vec::new(),
            other => {
                diagnostics.record_unsupported_tag(other);
                nodes_to_inlines(children, diagnostics)
            }
        },
    }
}

fn is_block_tag(name: &str) -> bool {
    matches!(
        name,
        "p" | "h1"
            | "h2"
            | "h3"
            | "h4"
            | "h5"
            | "h6"
            | "ul"
            | "ol"
            | "li"
            | "table"
            | "thead"
            | "tbody"
            | "tfoot"
            | "tr"
            | "th"
            | "td"
            | "dl"
            | "dt"
            | "dd"
            | "blockquote"
            | "section"
            | "article"
            | "div"
            | "main"
            | "body"
    )
}

fn text_content(nodes: &[HtmlNode]) -> String {
    nodes
        .iter()
        .map(|node| match node {
            HtmlNode::Text(text) => text.clone(),
            HtmlNode::Element { children, .. } => text_content(children),
        })
        .collect::<Vec<_>>()
        .join("")
}

fn parse_text_inlines(
    value: &str,
    diagnostics: &mut ContentParseDiagnostics,
) -> Vec<ContentInline> {
    let mut inlines = Vec::new();
    let mut offset = 0;

    while offset < value.len() {
        let rest = &value[offset..];
        if rest.starts_with("[[/r")
            && let Some(end_relative) = rest.find("]]")
        {
            let end = offset + end_relative + 2;
            let raw = &value[offset..end];
            let formula = raw
                .trim_start_matches("[[/r")
                .trim_end_matches("]]")
                .trim()
                .to_string();
            inlines.push(ContentInline::Roll {
                label: None,
                formula,
                raw: raw.to_string(),
            });
            offset = end;
            continue;
        }
        if rest.starts_with('@')
            && let Some(parsed) = parse_foundry_macro(value, offset, diagnostics)
        {
            if !parsed.inlines.is_empty() {
                inlines.extend(parsed.inlines);
            }
            offset = parsed.end;
            continue;
        }

        let next_macro = rest.find('@').unwrap_or(rest.len());
        let next_roll = rest.find("[[/r").unwrap_or(rest.len());
        let next_signal = next_macro.min(next_roll);
        if next_signal > 0 {
            push_text_inline(&mut inlines, &rest[..next_signal]);
            offset += next_signal;
        } else {
            let Some(character) = rest.chars().next() else {
                break;
            };
            push_text_inline(&mut inlines, &character.to_string());
            offset += character.len_utf8();
        }
    }

    merge_adjacent_text(inlines)
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct ParsedMacro {
    inlines: Vec<ContentInline>,
    end: usize,
}

fn parse_foundry_macro(
    value: &str,
    start: usize,
    diagnostics: &mut ContentParseDiagnostics,
) -> Option<ParsedMacro> {
    let rest = &value[start..];
    if !rest.starts_with('@') {
        return None;
    }

    let name_start = start + 1;
    let mut name_end = name_start;
    for (relative, character) in value[name_start..].char_indices() {
        if character.is_ascii_alphabetic() {
            name_end = name_start + relative + character.len_utf8();
        } else {
            break;
        }
    }
    if name_end == name_start || !value[name_end..].starts_with('[') {
        return None;
    }
    let name = &value[name_start..name_end];
    let body_start = name_end + 1;
    let body_end = balanced_close(value, body_start, '[', ']')?;
    let body = &value[body_start..body_end];
    let mut end = body_end + 1;
    let display = if value[end..].starts_with('{') {
        let display_start = end + 1;
        let display_end = balanced_close(value, display_start, '{', '}')?;
        end = display_end + 1;
        Some(&value[display_start..display_end])
    } else {
        None
    };

    let inlines = match name.to_ascii_lowercase().as_str() {
        "uuid" => vec![ContentInline::Reference {
            reference: ContentReference {
                label: display.map(|label| parse_text_inlines(label, diagnostics)),
                locator: ContentReferenceLocator::FoundryUuid {
                    raw_target: body.to_string(),
                },
                resolved_key: None,
                resolved_name: None,
            },
        }],
        "compendium" => vec![ContentInline::Reference {
            reference: ContentReference {
                label: display.map(|label| parse_text_inlines(label, diagnostics)),
                locator: ContentReferenceLocator::Compendium {
                    raw_target: body.to_string(),
                },
                resolved_key: None,
                resolved_name: None,
            },
        }],
        "damage" | "check" | "trait" => {
            let label = display
                .map(|display| parse_text_inlines(display, diagnostics))
                .unwrap_or_else(|| mechanic_signal(name, body));
            if label.is_empty() {
                dropped_macro(name, value, start, end, diagnostics);
            }
            label
        }
        "template" => {
            let template_kind = body
                .split('|')
                .next()
                .and_then(|part| part.strip_prefix("type:"))
                .map(ToOwned::to_owned);
            vec![ContentInline::Template {
                label: display
                    .map(ToOwned::to_owned)
                    .or_else(|| template_kind.clone())
                    .unwrap_or_else(|| body.to_string()),
                template_kind,
                raw: value[start..end].to_string(),
            }]
        }
        "action" => vec![ContentInline::ActionGlyph {
            action: display
                .map(ToOwned::to_owned)
                .unwrap_or_else(|| body.split('|').next().unwrap_or(body).to_string()),
        }],
        _ => {
            if let Some(display) = display {
                parse_text_inlines(display, diagnostics)
            } else {
                vec![ContentInline::Macro {
                    label: Some(name.to_string()),
                    raw: value[start..end].to_string(),
                }]
            }
        }
    };

    Some(ParsedMacro { inlines, end })
}

fn dropped_macro(
    name: &str,
    value: &str,
    start: usize,
    end: usize,
    diagnostics: &mut ContentParseDiagnostics,
) {
    diagnostics.dropped_macros.push(DroppedContentMacro {
        name: name.to_string(),
        raw: value[start..end].to_string(),
    });
}

fn mechanic_signal(name: &str, body: &str) -> Vec<ContentInline> {
    let terms = match name.to_ascii_lowercase().as_str() {
        "damage" => nested_bracket_signals(body),
        "check" => mechanic_terms(body.split('|').next().unwrap_or_default()),
        "trait" => mechanic_terms(body),
        _ => Vec::new(),
    };
    if terms.is_empty() {
        Vec::new()
    } else {
        vec![ContentInline::Text {
            text: terms.join(" "),
        }]
    }
}

fn nested_bracket_signals(value: &str) -> Vec<String> {
    let mut signals = Vec::new();
    let mut offset = 0;

    while offset < value.len() {
        let rest = &value[offset..];
        let Some(open_relative) = rest.find('[') else {
            break;
        };
        let body_start = offset + open_relative + 1;
        let Some(body_end) = balanced_close(value, body_start, '[', ']') else {
            break;
        };
        signals.extend(mechanic_terms(&value[body_start..body_end]));
        offset = body_end + 1;
    }

    signals
}

fn mechanic_terms(value: &str) -> Vec<String> {
    value
        .split(|character: char| {
            character.is_whitespace()
                || matches!(
                    character,
                    '|' | ':' | ';' | ',' | '=' | '[' | ']' | '(' | ')' | '{' | '}'
                )
        })
        .map(str::trim)
        .filter(|term| !term.is_empty())
        .filter(|term| !term.chars().all(|character| character.is_ascii_digit()))
        .filter(|term| !term.contains(char::is_numeric))
        .filter(|term| {
            !matches!(
                term.to_ascii_lowercase().as_str(),
                "dc" | "basic" | "save" | "saving" | "throw" | "damage"
            )
        })
        .map(ToOwned::to_owned)
        .collect()
}

fn balanced_close(value: &str, body_start: usize, open: char, close: char) -> Option<usize> {
    let mut depth = 1;
    for (relative, character) in value[body_start..].char_indices() {
        match character {
            current if current == open => depth += 1,
            current if current == close => {
                depth -= 1;
                if depth == 0 {
                    return Some(body_start + relative);
                }
            }
            _ => {}
        }
    }
    None
}

fn push_text_inline(inlines: &mut Vec<ContentInline>, text: &str) {
    if text.is_empty() {
        return;
    }
    inlines.push(ContentInline::Text {
        text: text.to_string(),
    });
}

fn merge_adjacent_text(inlines: Vec<ContentInline>) -> Vec<ContentInline> {
    let mut merged: Vec<ContentInline> = Vec::new();
    for inline in inlines {
        match (merged.last_mut(), inline) {
            (Some(ContentInline::Text { text: existing }), ContentInline::Text { text }) => {
                existing.push_str(&text);
            }
            (_, inline) => merged.push(inline),
        }
    }
    merged
}
