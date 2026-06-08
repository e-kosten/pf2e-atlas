use super::{
    FoundryLink, FoundryLinkBehavior, FoundryNode, RichDocument, RichLinkTarget, RichNode,
};

pub fn render_plain_text(document: &RichDocument) -> String {
    let mut output = String::new();
    render_nodes_plain(&document.nodes, &mut output);
    normalize_block_spacing(output)
}

pub fn render_markdown_like(document: &RichDocument) -> String {
    let mut output = String::new();
    render_nodes_markdown(&document.nodes, &mut output, 0);
    trim_trailing_blank_lines(output)
}

pub(crate) fn render_nodes_plain_text(nodes: &[RichNode]) -> String {
    let mut output = String::new();
    render_nodes_plain(nodes, &mut output);
    normalize_inline_spacing(output)
}

pub(crate) fn render_nodes_markdown_text(nodes: &[RichNode]) -> String {
    let mut output = String::new();
    render_nodes_markdown_inline(nodes, &mut output);
    normalize_inline_spacing(output)
}

fn render_nodes_plain(nodes: &[RichNode], output: &mut String) {
    for node in nodes {
        render_node_plain(node, output);
    }
}

fn render_node_plain(node: &RichNode, output: &mut String) {
    match node {
        RichNode::Text { text } => output.push_str(text),
        RichNode::HtmlElement { tag, children, .. } => {
            if is_blockish_tag(tag) || tag == "br" {
                output.push('\n');
            }
            render_nodes_plain(children, output);
            if is_blockish_tag(tag) {
                output.push('\n');
            } else if tag == "td" || tag == "th" {
                output.push_str(" | ");
            }
        }
        RichNode::FoundryLink { link } => render_foundry_link_plain(link, output),
        RichNode::Foundry { node } => render_foundry_node_plain(node, output),
    }
}

fn render_nodes_markdown(nodes: &[RichNode], output: &mut String, indent: usize) {
    for node in nodes {
        render_node_markdown(node, output, indent);
    }
}

fn render_node_markdown(node: &RichNode, output: &mut String, indent: usize) {
    match node {
        RichNode::Text { text } => output.push_str(text),
        RichNode::HtmlElement {
            tag,
            attributes: _,
            children,
        } => match tag.as_str() {
            "h1" | "h2" | "h3" | "h4" | "h5" | "h6" => {
                let level = heading_level(tag).unwrap_or(1).clamp(1, 6) as usize;
                push_line(
                    output,
                    &format!(
                        "{} {}",
                        "#".repeat(level),
                        render_nodes_markdown_text(children)
                    ),
                );
                output.push('\n');
            }
            "p" | "div" | "section" | "article" | "blockquote" => {
                let text = render_nodes_markdown_text(children);
                if !text.is_empty() {
                    push_line(output, &text);
                    output.push('\n');
                }
            }
            "ul" | "ol" => {
                render_list_markdown(tag == "ol", children, output, indent);
                output.push('\n');
            }
            "li" => {
                push_line(
                    output,
                    &format!(
                        "{}- {}",
                        " ".repeat(indent),
                        render_nodes_markdown_text(children)
                    ),
                );
            }
            "table" => {
                render_table_markdown(children, output);
                output.push('\n');
            }
            "hr" => {
                push_line(output, "---");
                output.push('\n');
            }
            "br" => output.push('\n'),
            "strong" | "b" => {
                output.push_str("**");
                render_nodes_markdown_inline(children, output);
                output.push_str("**");
            }
            "em" | "i" => {
                output.push('*');
                render_nodes_markdown_inline(children, output);
                output.push('*');
            }
            "code" => {
                output.push('`');
                render_nodes_markdown_inline(children, output);
                output.push('`');
            }
            _ => render_nodes_markdown(children, output, indent),
        },
        RichNode::FoundryLink { link } => render_foundry_link_markdown(link, output),
        RichNode::Foundry { node } => render_foundry_node_markdown(node, output),
    }
}

fn render_nodes_markdown_inline(nodes: &[RichNode], output: &mut String) {
    for node in nodes {
        match node {
            RichNode::HtmlElement { tag, children, .. } if tag == "br" => output.push(' '),
            RichNode::HtmlElement { tag, children, .. } if tag == "strong" || tag == "b" => {
                output.push_str("**");
                render_nodes_markdown_inline(children, output);
                output.push_str("**");
            }
            RichNode::HtmlElement { tag, children, .. } if tag == "em" || tag == "i" => {
                output.push('*');
                render_nodes_markdown_inline(children, output);
                output.push('*');
            }
            RichNode::HtmlElement { tag, children, .. } if tag == "code" => {
                output.push('`');
                render_nodes_markdown_inline(children, output);
                output.push('`');
            }
            RichNode::HtmlElement { children, .. } => {
                render_nodes_markdown_inline(children, output)
            }
            other => render_node_markdown(other, output, 0),
        }
    }
}

fn render_list_markdown(ordered: bool, children: &[RichNode], output: &mut String, indent: usize) {
    let mut index = 1;
    for child in children {
        let RichNode::HtmlElement { tag, children, .. } = child else {
            continue;
        };
        if tag != "li" {
            continue;
        }
        let prefix = if ordered {
            format!("{index}.")
        } else {
            "-".to_string()
        };
        index += 1;
        let rendered = render_nodes_markdown_text(children);
        push_line(
            output,
            &format!("{}{} {}", " ".repeat(indent), prefix, rendered),
        );
    }
}

fn render_table_markdown(children: &[RichNode], output: &mut String) {
    let mut rows = Vec::new();
    let mut caption = None;
    collect_table_rows(children, &mut rows, &mut caption);
    if let Some(caption) = caption {
        push_line(output, &format!("Table: {caption}"));
    }
    for (index, row) in rows.iter().enumerate() {
        push_line(output, &format!("| {} |", row.join(" | ")));
        if index == 0 {
            push_line(
                output,
                &format!(
                    "| {} |",
                    row.iter().map(|_| "---").collect::<Vec<_>>().join(" | ")
                ),
            );
        }
    }
}

fn collect_table_rows(
    nodes: &[RichNode],
    rows: &mut Vec<Vec<String>>,
    caption: &mut Option<String>,
) {
    for node in nodes {
        let RichNode::HtmlElement { tag, children, .. } = node else {
            continue;
        };
        match tag.as_str() {
            "caption" => *caption = Some(render_nodes_plain_text(children)),
            "tr" => {
                let cells = children
                    .iter()
                    .filter_map(|child| match child {
                        RichNode::HtmlElement { tag, children, .. }
                            if tag == "td" || tag == "th" =>
                        {
                            Some(render_nodes_plain_text(children))
                        }
                        _ => None,
                    })
                    .collect::<Vec<_>>();
                if !cells.is_empty() {
                    rows.push(cells);
                }
            }
            _ => collect_table_rows(children, rows, caption),
        }
    }
}

fn render_foundry_link_plain(link: &FoundryLink, output: &mut String) {
    output.push_str(&link_display_text(link));
}

fn render_foundry_link_markdown(link: &FoundryLink, output: &mut String) {
    let label = link_display_text(link);
    if let RichLinkTarget::Record { key, .. } = &link.target {
        match link.behavior {
            FoundryLinkBehavior::Reference => {
                output.push_str(&format!("[{label}](record:{key})"));
            }
            FoundryLinkBehavior::Embed { .. } => {
                output.push_str(&format!("[{label}](record:{key})"));
            }
        }
    } else {
        output.push_str(&label);
    }
}

fn render_foundry_node_plain(node: &FoundryNode, output: &mut String) {
    output.push_str(&foundry_node_display_text(node));
}

fn render_foundry_node_markdown(node: &FoundryNode, output: &mut String) {
    output.push_str(&foundry_node_display_text(node));
}

pub(crate) fn link_display_text(link: &FoundryLink) -> String {
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
        FoundryNode::Localize {
            key,
            label,
            resolved,
        } => label_text(label)
            .or_else(|| {
                resolved
                    .as_deref()
                    .map(render_nodes_plain_text)
                    .filter(|resolved| !resolved.trim().is_empty())
            })
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

fn is_blockish_tag(tag: &str) -> bool {
    matches!(
        tag,
        "p" | "div"
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
            | "caption"
            | "h1"
            | "h2"
            | "h3"
            | "h4"
            | "h5"
            | "h6"
            | "hr"
    )
}

fn push_line(output: &mut String, line: &str) {
    let line = line.trim();
    if !line.is_empty() {
        output.push_str(line);
    }
    output.push('\n');
}

fn normalize_block_spacing(output: String) -> String {
    trim_trailing_blank_lines(
        output
            .lines()
            .map(|line| normalize_inline_spacing(line.to_string()))
            .filter(|line| !line.is_empty())
            .collect::<Vec<_>>()
            .join("\n"),
    )
}

fn normalize_inline_spacing(output: String) -> String {
    output.split_whitespace().collect::<Vec<_>>().join(" ")
}

fn trim_trailing_blank_lines(mut output: String) -> String {
    while output.ends_with('\n') || output.ends_with(' ') {
        output.pop();
    }
    output
}
