use super::{ContentBlock, ContentDocument, ContentInline};

pub fn render_plain_text(document: &ContentDocument) -> String {
    let mut output = String::new();
    render_blocks_plain(&document.blocks, &mut output);
    trim_trailing_blank_lines(output)
}

pub fn render_markdown_like(document: &ContentDocument) -> String {
    let mut output = String::new();
    render_blocks_markdown(&document.blocks, &mut output, 0);
    trim_trailing_blank_lines(output)
}

pub(crate) fn render_inlines_plain(inlines: &[ContentInline]) -> String {
    let mut output = String::new();
    for inline in inlines {
        render_inline_plain(inline, &mut output);
    }
    normalize_inline_spacing(output)
}

pub(crate) fn render_inlines_markdown(inlines: &[ContentInline]) -> String {
    let mut output = String::new();
    for inline in inlines {
        render_inline_markdown(inline, &mut output);
    }
    normalize_inline_spacing(output)
}

fn render_blocks_plain(blocks: &[ContentBlock], output: &mut String) {
    for block in blocks {
        match block {
            ContentBlock::Heading { content, .. } | ContentBlock::Paragraph { content } => {
                push_line(output, &render_inlines_plain(content));
            }
            ContentBlock::List { items, .. } => {
                for item in items {
                    render_blocks_plain(item, output);
                }
            }
            ContentBlock::Table {
                caption,
                headers,
                rows,
            } => {
                if let Some(caption) = caption {
                    push_line(output, &render_inlines_plain(caption));
                }
                if !headers.is_empty() {
                    push_line(output, &render_table_row_plain(headers));
                }
                for row in rows {
                    let cells: Vec<String> = row
                        .iter()
                        .map(|cell| render_inlines_plain(cell))
                        .filter(|cell| !cell.is_empty())
                        .collect();
                    push_line(output, &cells.join(" | "));
                }
            }
            ContentBlock::Callout { title, blocks } | ContentBlock::RuleBlock { title, blocks } => {
                if let Some(title) = title {
                    push_line(output, &render_inlines_plain(title));
                }
                render_blocks_plain(blocks, output);
            }
            ContentBlock::DefinitionList { items } => {
                for item in items {
                    push_line(output, &render_inlines_plain(&item.term));
                    render_blocks_plain(&item.definition, output);
                }
            }
            ContentBlock::Separator => output.push('\n'),
        }
    }
}

fn render_blocks_markdown(blocks: &[ContentBlock], output: &mut String, indent: usize) {
    for block in blocks {
        match block {
            ContentBlock::Heading { level, content } => {
                let level = (*level).clamp(1, 6) as usize;
                push_line(
                    output,
                    &format!("{} {}", "#".repeat(level), render_inlines_markdown(content)),
                );
                output.push('\n');
            }
            ContentBlock::Paragraph { content } => {
                push_line(output, &render_inlines_markdown(content));
                output.push('\n');
            }
            ContentBlock::List { ordered, items } => {
                for (index, item) in items.iter().enumerate() {
                    let prefix = if *ordered {
                        format!("{}.", index + 1)
                    } else {
                        "-".to_string()
                    };
                    render_list_item_markdown(output, indent, &prefix, item);
                }
                output.push('\n');
            }
            ContentBlock::Table {
                caption,
                headers,
                rows,
            } => {
                if let Some(caption) = caption {
                    push_line(
                        output,
                        &format!("Table: {}", render_inlines_markdown(caption)),
                    );
                }
                if !headers.is_empty() {
                    push_line(
                        output,
                        &format!("| {} |", render_table_row_markdown(headers)),
                    );
                    push_line(
                        output,
                        &format!(
                            "| {} |",
                            headers
                                .iter()
                                .map(|_| "---")
                                .collect::<Vec<_>>()
                                .join(" | ")
                        ),
                    );
                }
                for row in rows {
                    let cells: Vec<String> = row
                        .iter()
                        .map(|cell| render_inlines_markdown(cell))
                        .collect();
                    push_line(output, &format!("| {} |", cells.join(" | ")));
                }
                output.push('\n');
            }
            ContentBlock::Callout { title, blocks } | ContentBlock::RuleBlock { title, blocks } => {
                if let Some(title) = title {
                    push_line(output, &format!("> {}", render_inlines_markdown(title)));
                }
                render_blocks_markdown(blocks, output, indent);
                output.push('\n');
            }
            ContentBlock::DefinitionList { items } => {
                for item in items {
                    push_line(output, &format!("{}:", render_inlines_markdown(&item.term)));
                    render_blocks_markdown(&item.definition, output, indent + 2);
                }
                output.push('\n');
            }
            ContentBlock::Separator => {
                push_line(output, "---");
                output.push('\n');
            }
        }
    }
}

fn render_list_item_markdown(
    output: &mut String,
    indent: usize,
    prefix: &str,
    blocks: &[ContentBlock],
) {
    let rendered = {
        let mut nested = String::new();
        render_blocks_markdown(blocks, &mut nested, indent + 2);
        trim_trailing_blank_lines(nested)
    };
    let mut lines = rendered.lines();
    if let Some(first) = lines.next() {
        push_line(
            output,
            &format!("{}{} {}", " ".repeat(indent), prefix, first),
        );
    }
    for line in lines {
        push_line(output, &format!("{}  {}", " ".repeat(indent), line));
    }
}

fn render_inline_plain(inline: &ContentInline, output: &mut String) {
    match inline {
        ContentInline::Text { text } | ContentInline::Code { text } => output.push_str(text),
        ContentInline::Strong { content } | ContentInline::Emphasis { content } => {
            for inline in content {
                render_inline_plain(inline, output);
            }
        }
        ContentInline::Break => output.push(' '),
        ContentInline::Reference { reference } => {
            if let Some(label) = &reference.label {
                output.push_str(&render_inlines_plain(label));
            } else if let Some(name) = &reference.resolved_name {
                output.push_str(name);
            } else if let Some(key) = &reference.resolved_key {
                output.push_str(&key.to_string());
            }
        }
        ContentInline::Roll { label, formula, .. } => {
            output.push_str(label.as_deref().unwrap_or(formula));
        }
        ContentInline::Template { label, .. } => output.push_str(label),
        ContentInline::Macro { label, raw } => output.push_str(label.as_deref().unwrap_or(raw)),
        ContentInline::ActionGlyph { action } => output.push_str(action),
        ContentInline::Icon { label, name } => {
            output.push_str(label.as_deref().unwrap_or(name));
        }
    }
}

fn render_inline_markdown(inline: &ContentInline, output: &mut String) {
    match inline {
        ContentInline::Text { text } => output.push_str(text),
        ContentInline::Strong { content } => {
            output.push_str("**");
            for inline in content {
                render_inline_markdown(inline, output);
            }
            output.push_str("**");
        }
        ContentInline::Emphasis { content } => {
            output.push('*');
            for inline in content {
                render_inline_markdown(inline, output);
            }
            output.push('*');
        }
        ContentInline::Code { text } => {
            output.push('`');
            output.push_str(text);
            output.push('`');
        }
        ContentInline::Break => output.push('\n'),
        ContentInline::Reference { reference } => {
            let label = reference
                .label
                .as_deref()
                .map(render_inlines_markdown)
                .or_else(|| reference.resolved_name.clone())
                .or_else(|| reference.resolved_key.as_ref().map(ToString::to_string))
                .unwrap_or_default();
            if let Some(key) = &reference.resolved_key {
                output.push_str(&format!("[{label}](record:{key})"));
            } else {
                output.push_str(&label);
            }
        }
        ContentInline::Roll { label, formula, .. } => {
            output.push_str(label.as_deref().unwrap_or(formula));
        }
        ContentInline::Template { label, .. } => output.push_str(label),
        ContentInline::Macro { label, raw } => output.push_str(label.as_deref().unwrap_or(raw)),
        ContentInline::ActionGlyph { action } => output.push_str(&format!("[{action}]")),
        ContentInline::Icon { label, name } => {
            output.push_str(label.as_deref().unwrap_or(name));
        }
    }
}

fn render_table_row_plain(cells: &[Vec<ContentInline>]) -> String {
    cells
        .iter()
        .map(|cell| render_inlines_plain(cell))
        .filter(|cell| !cell.is_empty())
        .collect::<Vec<_>>()
        .join(" | ")
}

fn render_table_row_markdown(cells: &[Vec<ContentInline>]) -> String {
    cells
        .iter()
        .map(|cell| render_inlines_markdown(cell))
        .collect::<Vec<_>>()
        .join(" | ")
}

fn push_line(output: &mut String, line: &str) {
    let line = line.trim();
    if !line.is_empty() {
        output.push_str(line);
        output.push('\n');
    }
}

fn normalize_inline_spacing(value: String) -> String {
    value.split_whitespace().collect::<Vec<_>>().join(" ")
}

fn trim_trailing_blank_lines(mut value: String) -> String {
    while value.ends_with('\n') {
        value.pop();
    }
    value
}

#[cfg(test)]
mod tests {
    use atlas_domain::RecordKey;

    use super::*;
    use crate::content::{ContentReference, ContentReferenceLocator};

    #[test]
    fn plain_text_keeps_readable_reference_labels() {
        let document = ContentDocument::new(vec![ContentBlock::Paragraph {
            content: vec![
                ContentInline::Text {
                    text: "Cast ".to_string(),
                },
                ContentInline::Reference {
                    reference: ContentReference {
                        label: Some(vec![ContentInline::Text {
                            text: "Heal".to_string(),
                        }]),
                        locator: ContentReferenceLocator::Unknown {
                            raw: "@UUID[...]".to_string(),
                        },
                        resolved_key: Some(RecordKey::parse("spells:Heal").expect("key parses")),
                        resolved_name: Some("Heal".to_string()),
                    },
                },
                ContentInline::Text {
                    text: ".".to_string(),
                },
            ],
        }]);

        assert_eq!(render_plain_text(&document), "Cast Heal.");
    }

    #[test]
    fn markdown_like_preserves_structure_and_links() {
        let document = ContentDocument::new(vec![
            ContentBlock::Heading {
                level: 2,
                content: vec![ContentInline::Text {
                    text: "Effect".to_string(),
                }],
            },
            ContentBlock::List {
                ordered: false,
                items: vec![vec![ContentBlock::Paragraph {
                    content: vec![ContentInline::Strong {
                        content: vec![ContentInline::Text {
                            text: "Critical".to_string(),
                        }],
                    }],
                }]],
            },
        ]);

        assert_eq!(
            render_markdown_like(&document),
            "## Effect\n\n- **Critical**"
        );
    }

    #[test]
    fn markdown_like_uses_resolved_name_for_unlabeled_links() {
        let document = ContentDocument::new(vec![ContentBlock::Paragraph {
            content: vec![ContentInline::Reference {
                reference: ContentReference {
                    label: None,
                    locator: ContentReferenceLocator::FoundryUuid {
                        raw_target: "@UUID[...]".to_string(),
                    },
                    resolved_key: Some(RecordKey::parse("feats-srd:abc123").expect("key parses")),
                    resolved_name: Some("Guardian's Deflection".to_string()),
                },
            }],
        }]);

        assert_eq!(render_plain_text(&document), "Guardian's Deflection");
        assert_eq!(
            render_markdown_like(&document),
            "[Guardian's Deflection](record:feats-srd:abc123)"
        );
    }
}
