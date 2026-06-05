use crate::{
    FoundryLink, FoundryLinkBehavior, FoundryNode, PresentationContent, PresentationContentBlock,
    PresentationInline, PresentationListItem, PresentationTableRow, RichDocument, RichLinkTarget,
    RichNode, render_plain_text,
};

pub(crate) fn project_presentation_content(document: &RichDocument) -> PresentationContent {
    PresentationContent::new(project_blocks(&document.nodes))
}

pub fn render_presentation_content_plain_text(content: &PresentationContent) -> String {
    content
        .blocks
        .iter()
        .filter_map(render_block_plain_text)
        .collect::<Vec<_>>()
        .join("\n\n")
}

fn render_block_plain_text(block: &PresentationContentBlock) -> Option<String> {
    match block {
        PresentationContentBlock::Heading { text, .. } => {
            (!text.trim().is_empty()).then(|| text.clone())
        }
        PresentationContentBlock::Paragraph { spans } => {
            let text = render_spans_plain_text(spans);
            (!text.trim().is_empty()).then_some(text)
        }
        PresentationContentBlock::List { ordered, items } => {
            let lines = items
                .iter()
                .enumerate()
                .filter_map(|(index, item)| {
                    let text = item
                        .blocks
                        .iter()
                        .filter_map(render_block_plain_text)
                        .collect::<Vec<_>>()
                        .join(" ");
                    if text.trim().is_empty() {
                        None
                    } else if *ordered {
                        Some(format!("{}. {text}", index + 1))
                    } else {
                        Some(format!("- {text}"))
                    }
                })
                .collect::<Vec<_>>();
            (!lines.is_empty()).then(|| lines.join("\n"))
        }
        PresentationContentBlock::Table { caption, rows } => {
            let mut lines = Vec::new();
            if let Some(caption) = caption.as_ref().filter(|value| !value.trim().is_empty()) {
                lines.push(caption.clone());
            }
            lines.extend(rows.iter().filter_map(|row| {
                let cells = row
                    .cells
                    .iter()
                    .map(render_presentation_content_plain_text)
                    .filter(|cell| !cell.trim().is_empty())
                    .collect::<Vec<_>>();
                (!cells.is_empty()).then(|| cells.join(" | "))
            }));
            (!lines.is_empty()).then(|| lines.join("\n"))
        }
        PresentationContentBlock::Rule => Some("---".to_string()),
    }
}

fn render_spans_plain_text(spans: &[PresentationInline]) -> String {
    let mut output = String::new();
    for span in spans {
        match span {
            PresentationInline::Text { text } | PresentationInline::Code { text } => {
                output.push_str(text);
            }
            PresentationInline::Strong { spans } | PresentationInline::Emphasis { spans } => {
                output.push_str(&render_spans_plain_text(spans));
            }
            PresentationInline::Reference { label, .. } => output.push_str(label),
            PresentationInline::LineBreak => output.push('\n'),
        }
    }
    output
}

fn project_blocks(nodes: &[RichNode]) -> Vec<PresentationContentBlock> {
    let mut blocks = Vec::new();
    for node in nodes {
        project_node_blocks(node, &mut blocks);
    }
    blocks
}

fn project_node_blocks(node: &RichNode, blocks: &mut Vec<PresentationContentBlock>) {
    match node {
        RichNode::Text { text } => {
            let text = text.trim();
            if !text.is_empty() {
                blocks.push(PresentationContentBlock::Paragraph {
                    spans: vec![PresentationInline::Text {
                        text: text.to_string(),
                    }],
                });
            }
        }
        RichNode::HtmlElement { tag, children, .. } => match tag.as_str() {
            "h1" | "h2" | "h3" | "h4" | "h5" | "h6" => {
                let text = render_nodes_plain_text(children);
                if !text.is_empty() {
                    blocks.push(PresentationContentBlock::Heading {
                        level: heading_level(tag).unwrap_or(1).clamp(1, 6),
                        text,
                    });
                }
            }
            "p" | "blockquote" => push_paragraph(children, blocks),
            "div" | "section" | "article" => {
                if children.iter().any(is_block_node) {
                    blocks.extend(project_blocks(children));
                } else {
                    push_paragraph(children, blocks);
                }
            }
            "ul" | "ol" => {
                let items = children
                    .iter()
                    .filter_map(list_item)
                    .filter(|item| !item.blocks.is_empty())
                    .collect::<Vec<_>>();
                if !items.is_empty() {
                    blocks.push(PresentationContentBlock::List {
                        ordered: tag == "ol",
                        items,
                    });
                }
            }
            "table" => {
                if let Some((caption, rows)) = table_content(children)
                    && !rows.is_empty()
                {
                    blocks.push(PresentationContentBlock::Table { caption, rows });
                }
            }
            "hr" => blocks.push(PresentationContentBlock::Rule),
            "br" => {}
            _ => blocks.extend(project_blocks(children)),
        },
        RichNode::FoundryLink { link } => {
            blocks.push(PresentationContentBlock::Paragraph {
                spans: vec![reference_inline(link)],
            });
        }
        RichNode::Foundry { node } => {
            let text = foundry_node_display_text(node);
            if !text.is_empty() {
                blocks.push(PresentationContentBlock::Paragraph {
                    spans: vec![PresentationInline::Text { text }],
                });
            }
        }
    }
}

fn push_paragraph(children: &[RichNode], blocks: &mut Vec<PresentationContentBlock>) {
    let spans = project_inline(children);
    if !spans_are_empty(&spans) {
        blocks.push(PresentationContentBlock::Paragraph { spans });
    }
}

fn list_item(node: &RichNode) -> Option<PresentationListItem> {
    let RichNode::HtmlElement { tag, children, .. } = node else {
        return None;
    };
    if tag != "li" {
        return None;
    }
    let blocks = if children.iter().any(is_block_node) {
        project_blocks(children)
    } else {
        let spans = project_inline(children);
        if spans_are_empty(&spans) {
            Vec::new()
        } else {
            vec![PresentationContentBlock::Paragraph { spans }]
        }
    };
    Some(PresentationListItem { blocks })
}

fn table_content(children: &[RichNode]) -> Option<(Option<String>, Vec<PresentationTableRow>)> {
    let mut rows = Vec::new();
    let mut caption = None;
    collect_table_content(children, &mut caption, &mut rows);
    Some((caption, rows))
}

fn collect_table_content(
    nodes: &[RichNode],
    caption: &mut Option<String>,
    rows: &mut Vec<PresentationTableRow>,
) {
    for node in nodes {
        let RichNode::HtmlElement { tag, children, .. } = node else {
            continue;
        };
        match tag.as_str() {
            "caption" => {
                let text = render_nodes_plain_text(children);
                if !text.is_empty() {
                    *caption = Some(text);
                }
            }
            "tr" => {
                let cells = children
                    .iter()
                    .filter_map(|child| match child {
                        RichNode::HtmlElement { tag, children, .. }
                            if tag == "td" || tag == "th" =>
                        {
                            Some(PresentationContent::new(project_blocks(children)))
                        }
                        _ => None,
                    })
                    .collect::<Vec<_>>();
                if !cells.is_empty() {
                    rows.push(PresentationTableRow { cells });
                }
            }
            _ => collect_table_content(children, caption, rows),
        }
    }
}

fn project_inline(nodes: &[RichNode]) -> Vec<PresentationInline> {
    let mut spans = Vec::new();
    for node in nodes {
        project_node_inline(node, &mut spans);
    }
    spans
}

fn project_node_inline(node: &RichNode, spans: &mut Vec<PresentationInline>) {
    match node {
        RichNode::Text { text } => {
            if !text.is_empty() {
                spans.push(PresentationInline::Text { text: text.clone() });
            }
        }
        RichNode::HtmlElement { tag, children, .. } => match tag.as_str() {
            "br" => spans.push(PresentationInline::LineBreak),
            "strong" | "b" => {
                let nested = project_inline(children);
                if !spans_are_empty(&nested) {
                    spans.push(PresentationInline::Strong { spans: nested });
                }
            }
            "em" | "i" => {
                let nested = project_inline(children);
                if !spans_are_empty(&nested) {
                    spans.push(PresentationInline::Emphasis { spans: nested });
                }
            }
            "code" => {
                let text = render_nodes_plain_text(children);
                if !text.is_empty() {
                    spans.push(PresentationInline::Code { text });
                }
            }
            _ => {
                if is_blockish_tag(tag) {
                    let text = render_nodes_plain_text(children);
                    if !text.is_empty() {
                        spans.push(PresentationInline::Text { text });
                    }
                } else {
                    spans.extend(project_inline(children));
                }
            }
        },
        RichNode::FoundryLink { link } => spans.push(reference_inline(link)),
        RichNode::Foundry { node } => {
            let text = foundry_node_display_text(node);
            if !text.is_empty() {
                spans.push(PresentationInline::Text { text });
            }
        }
    }
}

fn reference_inline(link: &FoundryLink) -> PresentationInline {
    PresentationInline::Reference {
        label: link_display_text(link),
        record_key: match &link.target {
            RichLinkTarget::Record { key, .. } => Some(key.clone()),
            RichLinkTarget::LocalContent { .. }
            | RichLinkTarget::External { .. }
            | RichLinkTarget::Unresolved { .. } => None,
        },
        embedded: matches!(link.behavior, FoundryLinkBehavior::Embed { .. }),
    }
}

fn link_display_text(link: &FoundryLink) -> String {
    link.label
        .as_deref()
        .map(render_nodes_plain_text)
        .filter(|label| !label.trim().is_empty())
        .or_else(|| link.target.display_name().map(ToOwned::to_owned))
        .unwrap_or_else(|| reference_display_fallback(&link.source.authored_target))
}

fn foundry_node_display_text(node: &FoundryNode) -> String {
    match node {
        FoundryNode::Check {
            label, statistic, ..
        } => label_text(label)
            .or_else(|| statistic.clone())
            .unwrap_or_default(),
        FoundryNode::Damage { label, formula, .. } => {
            label_text(label).unwrap_or_else(|| formula.clone())
        }
        FoundryNode::InlineCommand {
            label, arguments, ..
        } => label_text(label).unwrap_or_else(|| arguments.clone()),
        FoundryNode::Template { label, shape, .. } => label_text(label)
            .or_else(|| shape.clone())
            .unwrap_or_default(),
        FoundryNode::ActionGlyph { action } => action.clone(),
        FoundryNode::Trait { label, traits } => {
            label_text(label).unwrap_or_else(|| traits.join(" "))
        }
        FoundryNode::Localize { key, value } => value
            .as_deref()
            .map(render_nodes_plain_text)
            .filter(|value| !value.trim().is_empty())
            .unwrap_or_else(|| key.clone()),
        FoundryNode::UnknownFoundry {
            label, body, name, ..
        } => label_text(label)
            .or_else(|| body.clone())
            .unwrap_or_else(|| name.clone()),
    }
}

fn label_text(label: &Option<Vec<RichNode>>) -> Option<String> {
    label
        .as_deref()
        .map(render_nodes_plain_text)
        .filter(|label| !label.trim().is_empty())
}

fn render_nodes_plain_text(nodes: &[RichNode]) -> String {
    render_plain_text(&RichDocument::new(nodes.to_vec()))
}

fn reference_display_fallback(target: &str) -> String {
    let target = target.split_whitespace().next().unwrap_or(target);
    target
        .rsplit('.')
        .next()
        .unwrap_or(target)
        .replace(['-', '_'], " ")
        .trim()
        .to_string()
}

fn heading_level(tag: &str) -> Option<u8> {
    tag.strip_prefix('h')?.parse::<u8>().ok()
}

fn is_block_node(node: &RichNode) -> bool {
    matches!(
        node,
        RichNode::HtmlElement { tag, .. } if is_blockish_tag(tag)
    )
}

fn is_blockish_tag(tag: &str) -> bool {
    matches!(
        tag,
        "h1" | "h2"
            | "h3"
            | "h4"
            | "h5"
            | "h6"
            | "p"
            | "div"
            | "section"
            | "article"
            | "blockquote"
            | "ul"
            | "ol"
            | "li"
            | "table"
            | "thead"
            | "tbody"
            | "tfoot"
            | "tr"
            | "td"
            | "th"
            | "caption"
            | "hr"
    )
}

fn spans_are_empty(spans: &[PresentationInline]) -> bool {
    spans.iter().all(inline_is_empty)
}

fn inline_is_empty(span: &PresentationInline) -> bool {
    match span {
        PresentationInline::Text { text } | PresentationInline::Code { text } => {
            text.trim().is_empty()
        }
        PresentationInline::Strong { spans } | PresentationInline::Emphasis { spans } => {
            spans_are_empty(spans)
        }
        PresentationInline::Reference { label, .. } => label.trim().is_empty(),
        PresentationInline::LineBreak => false,
    }
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use atlas_domain::RecordKey;

    use super::*;
    use crate::{FoundryLinkMacroKind, FoundryLinkSource};

    #[test]
    fn projection_preserves_display_blocks_and_reference_targets() {
        let target_key = RecordKey::parse("actions:TreatWounds").expect("record key should parse");
        let document = RichDocument::new(vec![
            RichNode::HtmlElement {
                tag: "h2".to_string(),
                attributes: BTreeMap::new(),
                children: vec![RichNode::Text {
                    text: "Usage".to_string(),
                }],
            },
            RichNode::HtmlElement {
                tag: "p".to_string(),
                attributes: BTreeMap::new(),
                children: vec![
                    RichNode::Text {
                        text: "Attempt a ".to_string(),
                    },
                    RichNode::HtmlElement {
                        tag: "strong".to_string(),
                        attributes: BTreeMap::new(),
                        children: vec![RichNode::Text {
                            text: "Medicine".to_string(),
                        }],
                    },
                    RichNode::Text {
                        text: " check with ".to_string(),
                    },
                    RichNode::FoundryLink {
                        link: FoundryLink {
                            target: RichLinkTarget::Record {
                                key: target_key.clone(),
                                name: "Treat Wounds".to_string(),
                            },
                            label: Some(vec![RichNode::Text {
                                text: "Treat Wounds".to_string(),
                            }]),
                            source: FoundryLinkSource {
                                macro_kind: FoundryLinkMacroKind::Uuid,
                                authored_target: "Compendium.pf2e.actionspf2e.Item.TreatWounds"
                                    .to_string(),
                                relation: None,
                            },
                            behavior: FoundryLinkBehavior::Reference,
                        },
                    },
                    RichNode::Text {
                        text: " and review ".to_string(),
                    },
                    RichNode::FoundryLink {
                        link: FoundryLink {
                            target: RichLinkTarget::Record {
                                key: target_key.clone(),
                                name: "Treat Wounds".to_string(),
                            },
                            label: Some(vec![RichNode::Text {
                                text: "embedded treatment".to_string(),
                            }]),
                            source: FoundryLinkSource {
                                macro_kind: FoundryLinkMacroKind::Embed,
                                authored_target: "Compendium.pf2e.actionspf2e.Item.TreatWounds"
                                    .to_string(),
                                relation: None,
                            },
                            behavior: FoundryLinkBehavior::Embed {
                                inline: true,
                                hr: None,
                                options: BTreeMap::new(),
                            },
                        },
                    },
                    RichNode::Text {
                        text: ".".to_string(),
                    },
                ],
            },
        ]);

        let content = project_presentation_content(&document);

        assert!(matches!(
            &content.blocks[0],
            PresentationContentBlock::Heading { level: 2, text } if text == "Usage"
        ));
        let PresentationContentBlock::Paragraph { spans } = &content.blocks[1] else {
            panic!("expected paragraph");
        };
        assert!(spans.iter().any(|span| matches!(
            span,
            PresentationInline::Strong { spans }
                if matches!(&spans[0], PresentationInline::Text { text } if text == "Medicine")
        )));
        assert!(spans.iter().any(|span| matches!(
            span,
            PresentationInline::Reference {
                label,
                record_key: Some(record_key),
                embedded: false,
            } if label == "Treat Wounds" && record_key == &target_key
        )));
        assert!(spans.iter().any(|span| matches!(
            span,
            PresentationInline::Reference {
                label,
                record_key: Some(record_key),
                embedded: true,
            } if label == "embedded treatment" && record_key == &target_key
        )));
    }
}
