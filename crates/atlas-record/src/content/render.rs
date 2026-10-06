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

pub(crate) fn foundry_node_display_text(node: &FoundryNode) -> String {
    match node {
        FoundryNode::Check {
            label,
            statistic,
            options,
        } => check_display_text(
            label,
            statistic.as_deref(),
            options.get("dc").map(String::as_str),
        ),
        FoundryNode::Damage { label, formula, .. } => {
            label_text(label).unwrap_or_else(|| formula.clone())
        }
        FoundryNode::InlineCommand {
            label, arguments, ..
        } => label_text(label).unwrap_or_else(|| arguments.clone()),
        FoundryNode::Template {
            label,
            shape,
            options,
        } => label_text(label)
            .or_else(|| template_display_text(shape.as_deref(), options))
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

fn template_display_text(
    shape: Option<&str>,
    options: &std::collections::BTreeMap<String, String>,
) -> Option<String> {
    let shape = shape.filter(|shape| matches!(*shape, "burst" | "cone" | "emanation" | "line"))?;
    let Some(distance) = options.get("distance") else {
        return Some(shape.to_string());
    };
    if distance.is_empty() || !distance.bytes().all(|byte| byte.is_ascii_digit()) {
        return Some(shape.to_string());
    }
    let Ok(distance) = distance.parse::<u32>() else {
        return Some(shape.to_string());
    };
    if distance == 0 {
        return Some(shape.to_string());
    }
    Some(format!("{distance}-foot {shape}"))
}

fn check_display_text(
    label: &Option<Vec<RichNode>>,
    statistic: Option<&str>,
    difficulty_class: Option<&str>,
) -> String {
    let base = label_text(label).or_else(|| statistic.map(capitalize_first));
    match (
        base,
        difficulty_class.filter(|value| !value.trim().is_empty()),
    ) {
        (Some(base), Some(dc)) => format!("{base} DC {dc}"),
        (Some(base), None) => base,
        (None, Some(dc)) => format!("DC {dc}"),
        (None, None) => String::new(),
    }
}

fn capitalize_first(value: &str) -> String {
    let mut characters = value.chars();
    let Some(first) = characters.next() else {
        return String::new();
    };
    first.to_uppercase().chain(characters).collect()
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

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use super::*;

    #[test]
    fn check_display_preserves_structured_dc_across_plain_and_markdown_renderers() {
        let check = |statistic: Option<&str>, dc: Option<&str>, label: Option<&str>| {
            let mut options = BTreeMap::from([("traits".to_string(), "secret".to_string())]);
            if let Some(dc) = dc {
                options.insert("dc".to_string(), dc.to_string());
            }
            RichNode::Foundry {
                node: FoundryNode::Check {
                    statistic: statistic.map(str::to_string),
                    options,
                    label: label.map(|text| {
                        vec![RichNode::Text {
                            text: text.to_string(),
                        }]
                    }),
                },
            }
        };
        let document = RichDocument::new(vec![
            check(Some("fortitude"), Some("28"), None),
            RichNode::Text {
                text: " / ".to_string(),
            },
            check(Some("reflex"), None, None),
            RichNode::Text {
                text: " / ".to_string(),
            },
            check(Some("will"), Some("30"), Some("Custom save")),
            RichNode::Text {
                text: " / ".to_string(),
            },
            check(None, Some("20"), None),
        ]);
        let expected = "Fortitude DC 28 / Reflex / Custom save DC 30 / DC 20";

        assert_eq!(render_plain_text(&document), expected);
        assert_eq!(render_markdown_like(&document), expected);
    }

    #[test]
    fn template_display_uses_typed_shape_and_distance_without_overriding_labels() {
        let template = |shape: Option<&str>, distance: Option<&str>, label: Option<&str>| {
            let mut options = BTreeMap::new();
            if let Some(distance) = distance {
                options.insert("distance".to_string(), distance.to_string());
            }
            RichDocument::new(vec![RichNode::Foundry {
                node: FoundryNode::Template {
                    shape: shape.map(str::to_string),
                    options,
                    label: label.map(|text| {
                        vec![RichNode::Text {
                            text: text.to_string(),
                        }]
                    }),
                },
            }])
        };

        let document = template(Some("emanation"), Some("30"), None);
        let authored = document.clone();
        let hash = crate::ContentHash::for_document(&document);
        assert_eq!(
            crate::render_presentation_content_plain_text(&crate::project_presentation_content(
                &document
            )),
            "30-foot emanation"
        );
        assert_eq!(document, authored);
        assert_eq!(crate::ContentHash::for_document(&document), hash);
        for render in [render_plain_text, render_markdown_like] {
            assert_eq!(
                render(&template(Some("emanation"), Some("30"), None)),
                "30-foot emanation"
            );
            assert_eq!(render(&template(Some("burst"), None, None)), "burst");
            assert_eq!(
                render(&template(Some("emanation"), Some("many"), None)),
                "emanation"
            );
            assert_eq!(
                render(&template(Some("future-shape"), Some("30"), None)),
                ""
            );
            assert_eq!(
                render(&template(
                    Some("emanation"),
                    Some("30"),
                    Some("Nearby allies")
                )),
                "Nearby allies"
            );
        }
    }
}
