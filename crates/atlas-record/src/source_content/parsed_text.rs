use super::parsed_markup::*;
pub fn render_plain_text(document: &ParsedMarkup) -> String {
    let mut output = String::new();
    render_nodes_plain(&document.nodes, &mut output);
    normalize_block_spacing(output)
}

pub(crate) fn render_nodes_plain_text(nodes: &[ParsedNode]) -> String {
    let mut output = String::new();
    render_nodes_plain(nodes, &mut output);
    normalize_inline_spacing(output)
}

fn render_nodes_plain(nodes: &[ParsedNode], output: &mut String) {
    for node in nodes {
        render_node_plain(node, output);
    }
}

fn render_node_plain(node: &ParsedNode, output: &mut String) {
    match node {
        ParsedNode::Text { text } => output.push_str(text),
        ParsedNode::HtmlElement { tag, children, .. } => {
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
        ParsedNode::ParsedLink { link } => render_foundry_link_plain(link, output),
        ParsedNode::Foundry { node } => render_foundry_node_plain(node, output),
    }
}

fn render_foundry_link_plain(link: &ParsedLink, output: &mut String) {
    output.push_str(&link_display_text(link));
}

fn render_foundry_node_plain(node: &ParsedMacro, output: &mut String) {
    output.push_str(&foundry_node_display_text(node));
}

fn link_display_text(link: &ParsedLink) -> String {
    if let Some(label) = label_text(&link.label) {
        return label;
    }
    let ParsedLinkTarget::Unresolved { fallback_label, .. } = &link.target;
    fallback_label.clone()
}

fn foundry_node_display_text(node: &ParsedMacro) -> String {
    match node {
        ParsedMacro::Check {
            label, statistic, ..
        } => label_text(label)
            .or_else(|| statistic.clone())
            .unwrap_or_default(),
        ParsedMacro::Damage { label, formula, .. } => {
            label_text(label).unwrap_or_else(|| formula.clone())
        }
        ParsedMacro::InlineCommand {
            label, arguments, ..
        } => label_text(label).unwrap_or_else(|| arguments.clone()),
        ParsedMacro::Template { label, shape, .. } => label_text(label)
            .or_else(|| shape.clone())
            .unwrap_or_default(),
        ParsedMacro::Localize {
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
        ParsedMacro::UnknownFoundry {
            label, body, name, ..
        } => label_text(label)
            .or_else(|| body.clone())
            .unwrap_or_else(|| name.clone()),
    }
}

fn label_text(label: &Option<Vec<ParsedNode>>) -> Option<String> {
    label
        .as_deref()
        .map(render_nodes_plain_text)
        .filter(|label| !label.trim().is_empty())
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
