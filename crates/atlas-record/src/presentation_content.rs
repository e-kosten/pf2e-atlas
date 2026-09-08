use self::rank_damage::selected_damage_text;
use crate::content::foundry_node_display_text;
mod rank_damage;
use crate::{
    FoundryLink, FoundryLinkBehavior, FoundryNode, PresentationContent, PresentationContentBlock,
    PresentationInline, PresentationListItem, PresentationTableRow, RichDocument, RichLinkTarget,
    RichNode, render_plain_text,
};

pub fn project_presentation_content(document: &RichDocument) -> PresentationContent {
    PresentationContent::new(project_blocks(&document.nodes, &mut DisplayPolicy::Default))
}

/// App record surfaces may display structured damage parts without changing
/// canonical text, search presentation, or embedding inputs.
pub fn project_record_surface_content(document: &RichDocument) -> PresentationContent {
    project_record_surface_content_with_context(document, None).content
}

/// Context is supplied only by the successful selected-form resolver result.
#[derive(Debug, Clone)]
pub enum RecordSurfaceContentContext {
    Spell {
        form_id: crate::SpellFormId,
        cast_rank: u8,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RecordSurfaceContentIssueKind {
    MissingContext,
    UnsupportedDamage,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RecordSurfaceContentIssue {
    pub inline_index: usize,
    pub kind: RecordSurfaceContentIssueKind,
}

pub struct RecordSurfaceContentProjection {
    pub content: PresentationContent,
    pub issues: Vec<RecordSurfaceContentIssue>,
}

pub fn project_record_surface_content_with_context(
    document: &RichDocument,
    context: Option<&RecordSurfaceContentContext>,
) -> RecordSurfaceContentProjection {
    let mut policy = DisplayPolicy::RecordSurface {
        context,
        issues: Vec::new(),
        inline_index: 0,
    };
    let content = PresentationContent::new(project_blocks(&document.nodes, &mut policy));
    let issues = match policy {
        DisplayPolicy::RecordSurface { issues, .. } => issues,
        DisplayPolicy::Default => Vec::new(),
    };
    RecordSurfaceContentProjection { content, issues }
}

enum DisplayPolicy<'a> {
    Default,
    RecordSurface {
        context: Option<&'a RecordSurfaceContentContext>,
        issues: Vec<RecordSurfaceContentIssue>,
        inline_index: usize,
    },
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
            PresentationInline::Check { display, .. } => output.push_str(display),
            PresentationInline::LineBreak => output.push('\n'),
        }
    }
    output
}

fn project_blocks(
    nodes: &[RichNode],
    policy: &mut DisplayPolicy<'_>,
) -> Vec<PresentationContentBlock> {
    let mut blocks = Vec::new();
    for node in nodes {
        project_node_blocks(node, &mut blocks, policy);
    }
    blocks
}

fn project_node_blocks(
    node: &RichNode,
    blocks: &mut Vec<PresentationContentBlock>,
    policy: &mut DisplayPolicy<'_>,
) {
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
                let text = render_nodes_text(children, policy);
                if !text.is_empty() {
                    blocks.push(PresentationContentBlock::Heading {
                        level: heading_level(tag).unwrap_or(1).clamp(1, 6),
                        text,
                    });
                }
            }
            "p" | "blockquote" => push_paragraph(children, blocks, policy),
            "div" | "section" | "article" => {
                if children.iter().any(is_block_node) {
                    blocks.extend(project_blocks(children, policy));
                } else {
                    push_paragraph(children, blocks, policy);
                }
            }
            "ul" | "ol" => {
                let items = children
                    .iter()
                    .filter_map(|node| list_item(node, policy))
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
                if let Some((caption, rows)) = table_content(children, policy)
                    && !rows.is_empty()
                {
                    blocks.push(PresentationContentBlock::Table { caption, rows });
                }
            }
            "hr" => blocks.push(PresentationContentBlock::Rule),
            "br" => {}
            _ => blocks.extend(project_blocks(children, policy)),
        },
        RichNode::FoundryLink { link } => {
            blocks.push(PresentationContentBlock::Paragraph {
                spans: vec![reference_inline(link)],
            });
        }
        RichNode::Foundry { node } => {
            if let Some(span) = foundry_inline(node, policy) {
                blocks.push(PresentationContentBlock::Paragraph { spans: vec![span] });
            }
        }
    }
}

fn push_paragraph(
    children: &[RichNode],
    blocks: &mut Vec<PresentationContentBlock>,
    policy: &mut DisplayPolicy<'_>,
) {
    let spans = project_inline(children, policy);
    if !spans_are_empty(&spans) {
        blocks.push(PresentationContentBlock::Paragraph { spans });
    }
}

fn list_item(node: &RichNode, policy: &mut DisplayPolicy<'_>) -> Option<PresentationListItem> {
    let RichNode::HtmlElement { tag, children, .. } = node else {
        return None;
    };
    if tag != "li" {
        return None;
    }
    let blocks = if children.iter().any(is_block_node) {
        project_blocks(children, policy)
    } else {
        let spans = project_inline(children, policy);
        if spans_are_empty(&spans) {
            Vec::new()
        } else {
            vec![PresentationContentBlock::Paragraph { spans }]
        }
    };
    Some(PresentationListItem { blocks })
}

fn table_content(
    children: &[RichNode],
    policy: &mut DisplayPolicy<'_>,
) -> Option<(Option<String>, Vec<PresentationTableRow>)> {
    let mut rows = Vec::new();
    let mut caption = None;
    collect_table_content(children, &mut caption, &mut rows, policy);
    Some((caption, rows))
}

fn collect_table_content(
    nodes: &[RichNode],
    caption: &mut Option<String>,
    rows: &mut Vec<PresentationTableRow>,
    policy: &mut DisplayPolicy<'_>,
) {
    for node in nodes {
        let RichNode::HtmlElement { tag, children, .. } = node else {
            continue;
        };
        match tag.as_str() {
            "caption" => {
                let text = render_nodes_text(children, policy);
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
                            Some(PresentationContent::new(project_blocks(children, policy)))
                        }
                        _ => None,
                    })
                    .collect::<Vec<_>>();
                if !cells.is_empty() {
                    rows.push(PresentationTableRow { cells });
                }
            }
            _ => collect_table_content(children, caption, rows, policy),
        }
    }
}

fn project_inline(nodes: &[RichNode], policy: &mut DisplayPolicy<'_>) -> Vec<PresentationInline> {
    let mut spans = Vec::new();
    for node in nodes {
        project_node_inline(node, &mut spans, policy);
    }
    spans
}

fn project_node_inline(
    node: &RichNode,
    spans: &mut Vec<PresentationInline>,
    policy: &mut DisplayPolicy<'_>,
) {
    match node {
        RichNode::Text { text } => {
            if !text.is_empty() {
                spans.push(PresentationInline::Text { text: text.clone() });
            }
        }
        RichNode::HtmlElement { tag, children, .. } => match tag.as_str() {
            "br" => spans.push(PresentationInline::LineBreak),
            "strong" | "b" => {
                let nested = project_inline(children, policy);
                if !spans_are_empty(&nested) {
                    spans.push(PresentationInline::Strong { spans: nested });
                }
            }
            "em" | "i" => {
                let nested = project_inline(children, policy);
                if !spans_are_empty(&nested) {
                    spans.push(PresentationInline::Emphasis { spans: nested });
                }
            }
            "code" => {
                let text = render_nodes_text(children, policy);
                if !text.is_empty() {
                    spans.push(PresentationInline::Code { text });
                }
            }
            _ => {
                if is_blockish_tag(tag) {
                    let text = render_nodes_text(children, policy);
                    if !text.is_empty() {
                        spans.push(PresentationInline::Text { text });
                    }
                } else {
                    spans.extend(project_inline(children, policy));
                }
            }
        },
        RichNode::FoundryLink { link } => spans.push(reference_inline(link)),
        RichNode::Foundry { node } => {
            if let Some(span) = foundry_inline(node, policy) {
                spans.push(span);
            }
        }
    }
}

fn foundry_inline(
    node: &FoundryNode,
    policy: &mut DisplayPolicy<'_>,
) -> Option<PresentationInline> {
    let display = match policy {
        DisplayPolicy::Default => foundry_node_display_text(node),
        DisplayPolicy::RecordSurface {
            context,
            issues,
            inline_index,
        } => {
            let index = *inline_index;
            *inline_index += 1;
            match selected_damage_text(node, *context) {
                Ok(Some(text)) => text,
                Ok(None) => record_surface_foundry_text(node),
                Err(kind) => {
                    issues.push(RecordSurfaceContentIssue {
                        inline_index: index,
                        kind,
                    });
                    "Damage unavailable".to_string()
                }
            }
        }
    };
    if display.is_empty() {
        return None;
    }
    match node {
        FoundryNode::Check {
            statistic, options, ..
        } => Some(PresentationInline::Check {
            display,
            statistic: statistic.clone(),
            difficulty_class: options.get("dc").and_then(|value| value.parse().ok()),
        }),
        _ => Some(PresentationInline::Text { text: display }),
    }
}

fn render_nodes_text(nodes: &[RichNode], policy: &mut DisplayPolicy<'_>) -> String {
    match policy {
        DisplayPolicy::Default => render_nodes_plain_text(nodes),
        DisplayPolicy::RecordSurface { .. } => {
            render_spans_plain_text(&project_inline(nodes, policy))
                .trim()
                .to_string()
        }
    }
}

fn record_surface_foundry_text(node: &FoundryNode) -> String {
    let FoundryNode::Damage {
        damage_parts,
        label,
        ..
    } = node
    else {
        return foundry_node_display_text(node);
    };
    if label
        .as_deref()
        .map(render_nodes_plain_text)
        .is_some_and(|label| !label.trim().is_empty())
    {
        return foundry_node_display_text(node);
    }
    // These are the ordered parts already parsed by ingest. Validate display
    // completeness only; never parse a formula, options, or source text here.
    let safe_text = |value: &str| {
        !value.trim().is_empty()
            && !value
                .chars()
                .any(|ch| ch.is_control() || matches!(ch, '[' | ']'))
    };
    if damage_parts.is_empty()
        || damage_parts.iter().any(|part| {
            !safe_text(&part.formula)
                || part
                    .damage_type
                    .as_deref()
                    .is_some_and(|kind| !safe_text(kind))
        })
    {
        return foundry_node_display_text(node);
    }
    damage_parts
        .iter()
        .map(|part| match &part.damage_type {
            Some(kind) => format!("{} {}", part.formula, kind),
            None => part.formula.clone(),
        })
        .collect::<Vec<_>>()
        .join(", ")
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
        PresentationInline::Check { display, .. } => display.trim().is_empty(),
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
    fn record_surface_damage_policy_preserves_default_text_document_and_hash() {
        let damage = FoundryNode::Damage {
            formula: "10[bludgeoning],2d6[fire]".into(),
            options: BTreeMap::new(),
            label: None,
            damage_parts: vec![
                crate::DamagePart {
                    formula: "10".into(),
                    damage_type: Some("bludgeoning".into()),
                },
                crate::DamagePart {
                    formula: "2d6".into(),
                    damage_type: Some("fire".into()),
                },
            ],
        };
        let document = RichDocument::new(vec![RichNode::HtmlElement {
            tag: "p".into(),
            attributes: BTreeMap::new(),
            children: vec![
                RichNode::Foundry {
                    node: damage.clone(),
                },
                RichNode::Text {
                    text: " damage".into(),
                },
            ],
        }]);
        let before = document.clone();
        let hash = crate::ContentHash::for_document(&document);
        let plain = render_plain_text(&document);
        let markdown = crate::render_markdown_like(&document);
        let default = project_presentation_content(&document);
        assert_eq!(
            render_presentation_content_plain_text(&default),
            "10[bludgeoning],2d6[fire] damage"
        );
        assert_eq!(
            render_presentation_content_plain_text(&project_record_surface_content(&document)),
            "10 bludgeoning, 2d6 fire damage"
        );
        assert_eq!(document, before);
        assert_eq!(crate::ContentHash::for_document(&document), hash);
        assert_eq!(render_plain_text(&document), plain);
        assert_eq!(crate::render_markdown_like(&document), markdown);
        assert_eq!(project_presentation_content(&document), default);
        assert!(plain.contains("10[bludgeoning],2d6[fire]"));
        assert!(markdown.contains("10[bludgeoning],2d6[fire]"));
        let mut labelled = damage.clone();
        let FoundryNode::Damage { label, .. } = &mut labelled else {
            panic!("damage")
        };
        *label = Some(vec![RichNode::Text {
            text: "Authored damage label".into(),
        }]);
        assert_eq!(
            record_surface_foundry_text(&labelled),
            "Authored damage label"
        );
        for parts in [
            vec![],
            vec![crate::DamagePart {
                formula: "".into(),
                damage_type: Some("fire".into()),
            }],
            vec![crate::DamagePart {
                formula: "2d6".into(),
                damage_type: Some("".into()),
            }],
            vec![crate::DamagePart {
                formula: "2d6[fire]".into(),
                damage_type: None,
            }],
        ] {
            let mut malformed = damage.clone();
            let FoundryNode::Damage { damage_parts, .. } = &mut malformed else {
                panic!("damage")
            };
            *damage_parts = parts;
            assert_eq!(
                record_surface_foundry_text(&malformed),
                "10[bludgeoning],2d6[fire]"
            );
        }
        let bare = FoundryNode::Damage {
            formula: "7".into(),
            options: BTreeMap::new(),
            label: None,
            damage_parts: vec![crate::DamagePart {
                formula: "7".into(),
                damage_type: None,
            }],
        };
        assert_eq!(record_surface_foundry_text(&bare), "7");
    }

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

    #[test]
    fn projection_preserves_paragraph_check_dc_and_divider_order() {
        let document = RichDocument::new(vec![
            RichNode::HtmlElement {
                tag: "p".to_string(),
                attributes: BTreeMap::new(),
                children: vec![
                    RichNode::Text {
                        text: "Saving Throw ".to_string(),
                    },
                    RichNode::Foundry {
                        node: crate::FoundryNode::Check {
                            statistic: Some("fortitude".to_string()),
                            options: BTreeMap::from([("dc".to_string(), "28".to_string())]),
                            label: None,
                        },
                    },
                ],
            },
            RichNode::HtmlElement {
                tag: "hr".to_string(),
                attributes: BTreeMap::new(),
                children: Vec::new(),
            },
            RichNode::HtmlElement {
                tag: "p".to_string(),
                attributes: BTreeMap::new(),
                children: vec![RichNode::Text {
                    text: "After the divider.".to_string(),
                }],
            },
        ]);

        let content = project_presentation_content(&document);
        assert_eq!(content.blocks.len(), 3);
        assert!(matches!(
            &content.blocks[0],
            PresentationContentBlock::Paragraph { spans }
                if matches!(
                    spans.as_slice(),
                    [
                        PresentationInline::Text { text },
                        PresentationInline::Check {
                            display,
                            statistic: Some(statistic),
                            difficulty_class: Some(28),
                        },
                    ] if text == "Saving Throw "
                        && display == "Fortitude DC 28"
                        && statistic == "fortitude"
                )
        ));
        assert!(matches!(
            &content.blocks[0],
            PresentationContentBlock::Paragraph { spans }
                if render_spans_plain_text(spans) == "Saving Throw Fortitude DC 28"
        ));
        assert!(matches!(&content.blocks[1], PresentationContentBlock::Rule));
        assert!(matches!(
            &content.blocks[2],
            PresentationContentBlock::Paragraph { spans }
                if render_spans_plain_text(spans) == "After the divider."
        ));
    }
}
