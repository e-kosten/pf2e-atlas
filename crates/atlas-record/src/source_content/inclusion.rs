//! Structural composition of checked prepared HTML. Never parses Foundry macros.
use ego_tree::NodeRef;
use scraper::{ElementRef, Html, Node};
use std::collections::BTreeMap;

pub fn escape_content_text(value: &str) -> String {
    value
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}

/// Only paragraph-isolated markers accept block insertions. Mixed inline content
/// retains its original link with a local explanation instead of invalid HTML.
pub fn compose_prepared_inclusions(
    html: &str,
    insertions: &BTreeMap<usize, String>,
    omit_terminal_uuid: bool,
) -> String {
    let fragment = Html::parse_fragment(html);
    let root = fragment.root_element();
    let last = root
        .children()
        .filter(|n| !matches!(n.value(), Node::Text(t) if t.trim().is_empty()))
        .last()
        .map(|n| n.id());
    root.children()
        .map(|node| {
            render(
                node,
                insertions,
                omit_terminal_uuid && Some(node.id()) == last,
            )
        })
        .collect()
}
fn isolated_marker(node: NodeRef<'_, Node>) -> Option<usize> {
    let e = ElementRef::wrap(node)?;
    if e.value().name() != "p" {
        return None;
    }
    let mut children = node
        .children()
        .filter(|n| !matches!(n.value(), Node::Text(t) if t.trim().is_empty()));
    let marker = ElementRef::wrap(children.next()?)?;
    if children.next().is_some() {
        return None;
    }
    marker.value().attr("data-atlas-reference")?.parse().ok()
}
fn render(
    node: NodeRef<'_, Node>,
    insertions: &BTreeMap<usize, String>,
    omit_terminal: bool,
) -> String {
    if let Some(ordinal) = isolated_marker(node) {
        if let Some(html) = insertions.get(&ordinal) {
            return html.clone();
        }
        // Caller verifies this ordinal is a terminal UUID reference, not an Embed.
        if omit_terminal {
            return String::new();
        }
    }
    match node.value() {
        Node::Text(text) => escape_content_text(text),
        Node::Element(e) => {
            let mut out = format!("<{}", e.name());
            for (name, value) in e.attrs() {
                out.push_str(&format!(" {name}=\"{}\"", escape_content_text(value)));
            }
            out.push('>');
            for child in node.children() {
                out.push_str(&render(child, insertions, false));
            }
            if !matches!(
                e.name(),
                "area"
                    | "base"
                    | "br"
                    | "col"
                    | "embed"
                    | "hr"
                    | "img"
                    | "input"
                    | "link"
                    | "meta"
                    | "param"
                    | "source"
                    | "track"
                    | "wbr"
            ) {
                out.push_str(&format!("</{}>", e.name()));
            }
            if e.attr("data-atlas-reference")
                .and_then(|v| v.parse::<usize>().ok())
                .is_some_and(|o| insertions.contains_key(&o))
            {
                out.push_str(" <span class=\"prepared-inclusion-unavailable\">Inline content unavailable in this position; open the reference.</span>");
            }
            out
        }
        _ => String::new(),
    }
}

/// A terminal UUID paragraph is suppressible only for the exact pinned option.
pub fn terminal_prepared_reference(html: &str) -> Option<usize> {
    let fragment = Html::parse_fragment(html);
    let last = fragment
        .root_element()
        .children()
        .filter(|n| !matches!(n.value(), Node::Text(t) if t.trim().is_empty()))
        .last()?;
    isolated_marker(last)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn replaces_whole_paragraph_and_preserves_mixed_content() {
        let inserts = BTreeMap::from([(0, "<section><p>Inserted rules</p></section>".into())]);
        let html = compose_prepared_inclusions(
            "<h2>Host</h2><p><span data-atlas-reference=\"0\">Target</span></p><p>After</p>",
            &inserts,
            false,
        );
        assert!(
            html.contains("</h2><section><p>Inserted rules</p></section><p>After</p>"),
            "{html}"
        );
        let mixed = compose_prepared_inclusions(
            "<p>Before <span data-atlas-reference=\"0\">Target</span> after</p>",
            &inserts,
            false,
        );
        assert!(!mixed.contains("<section>"));
        assert!(
            mixed.contains("Before ")
                && mixed.contains(" after")
                && mixed.contains("open the reference")
        );
    }
}
