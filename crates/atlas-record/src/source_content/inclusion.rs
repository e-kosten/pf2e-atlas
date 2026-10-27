//! Structural composition of checked prepared HTML. Never parses Foundry macros.
use ego_tree::NodeRef;
use scraper::{ElementRef, Html, Node};
use std::collections::{BTreeMap, HashMap};

pub fn escape_content_text(value: &str) -> String {
    value
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}

/// Transient insertion HTML and checked source-navigation facts, not a source tree.
#[derive(Debug, Clone)]
pub struct PreparedHtmlInclusion {
    pub html: String,
    pub source_link: Option<PreparedInclusionSourceLink>,
}
#[derive(Debug, Clone)]
pub struct PreparedInclusionSourceLink {
    pub namespace: usize,
    pub matching_references: Vec<usize>,
    pub action: bool,
}

/// Only paragraph-isolated markers accept block insertions. Mixed inline content
/// retains its original link with a local explanation instead of invalid HTML.
pub fn compose_prepared_inclusions(
    html: &str,
    insertions: &BTreeMap<usize, PreparedHtmlInclusion>,
    omit_terminal_uuid: bool,
) -> String {
    let fragment = Html::parse_fragment(html);
    let root = fragment.root_element();
    let mut headings = HashMap::new();
    for paragraph in root
        .descendants()
        .filter_map(ElementRef::wrap)
        .filter(|element| element.value().name() == "p")
    {
        let Some(ordinal) = isolated_marker(*paragraph) else {
            continue;
        };
        let next_is_inclusion = adjacent_element(*paragraph, false)
            .and_then(|next| isolated_marker(*next))
            .is_some_and(|n| insertions.contains_key(&n));
        let Some(insertion) = insertions.get(&ordinal) else {
            continue;
        };
        let Some(source) = &insertion.source_link else {
            continue;
        };
        if let Some(previous) = adjacent_element(*paragraph, true) {
            if is_heading(previous.value().name()) && !next_is_inclusion {
                if !has_links_or_controls(previous) {
                    headings.insert(
                        previous.id(),
                        element_with_body(
                            previous,
                            previous.value().name(),
                            &source_heading_link(source, &previous.inner_html()),
                            false,
                        ),
                    );
                }
            } else if previous.value().name() == "p"
                && source.action
                && let Some(title) = action_heading(previous, source)
            {
                headings.insert(previous.id(), title);
            }
        }
    }
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
                &headings,
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
    insertions: &BTreeMap<usize, PreparedHtmlInclusion>,
    headings: &HashMap<ego_tree::NodeId, String>,
    omit_terminal: bool,
) -> String {
    if let Some(heading) = headings.get(&node.id()) {
        return heading.clone();
    }
    if let Some(ordinal) = isolated_marker(node) {
        if let Some(html) = insertions.get(&ordinal) {
            if ElementRef::wrap(node)
                .is_some_and(|element| element.value().attr("data-atlas-passage-target").is_some())
            {
                return inclusion_at_passage_target(&html.html);
            }
            return html.html.clone();
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
                out.push_str(&render(child, insertions, headings, false));
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

// Replacing an isolated Embed paragraph must retain its checked landing location.
// Only transient navigation metadata transfers to the insertion's existing root.
fn inclusion_at_passage_target(html: &str) -> String {
    let mut fragment = Html::parse_fragment(html);
    let start = fragment
        .root_element()
        .children()
        .find_map(ElementRef::wrap)
        .map(|element| element.id());
    if let Some(start) = start
        && let Some(mut node) = fragment.tree.get_mut(start)
        && let Node::Element(element) = node.value()
    {
        element
            .attrs
            .retain(|(name, _)| name.local.as_ref() != "data-atlas-passage-target");
        let mut name = element.name.clone();
        name.prefix = None;
        name.ns = Default::default();
        name.local = "data-atlas-passage-target".into();
        element.attrs.push((name, "".into()));
    }
    fragment.root_element().inner_html()
}

fn adjacent_element(node: NodeRef<'_, Node>, backwards: bool) -> Option<ElementRef<'_>> {
    let mut next = if backwards {
        node.prev_sibling()
    } else {
        node.next_sibling()
    };
    while let Some(candidate) = next {
        if !matches!(candidate.value(), Node::Text(t) if t.trim().is_empty()) {
            return ElementRef::wrap(candidate);
        }
        next = if backwards {
            candidate.prev_sibling()
        } else {
            candidate.next_sibling()
        };
    }
    None
}
fn is_heading(tag: &str) -> bool {
    matches!(tag, "h1" | "h2" | "h3" | "h4" | "h5" | "h6")
}
fn reference_ordinals(element: ElementRef<'_>) -> Vec<usize> {
    element
        .descendants()
        .filter_map(ElementRef::wrap)
        .filter(|child| child.id() != element.id())
        .filter_map(|child| child.value().attr("data-atlas-reference")?.parse().ok())
        .collect()
}
fn has_links_or_controls(element: ElementRef<'_>) -> bool {
    element
        .descendants()
        .filter_map(ElementRef::wrap)
        .filter(|child| child.id() != element.id())
        .any(|child| {
            child.value().name() == "a"
                || child.value().attr("data-atlas-reference").is_some()
                || child.value().attr("data-atlas-interaction").is_some()
        })
}
fn source_heading_link(source: &PreparedInclusionSourceLink, title: &str) -> String {
    format!(
        "<span data-atlas-reference=\"0\" data-atlas-inclusion=\"{}\">{title}</span>",
        source.namespace
    )
}
pub(super) fn element_with_body(
    element: ElementRef<'_>,
    tag: &str,
    body: &str,
    promoted: bool,
) -> String {
    let mut out = format!("<{tag}");
    for (name, value) in element.value().attrs() {
        out.push_str(&format!(" {name}=\"{}\"", escape_content_text(value)));
    }
    if promoted {
        out.push_str(" data-atlas-inclusion-heading=\"true\"");
    }
    out.push('>');
    out.push_str(body);
    if !matches!(
        tag,
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
        out.push_str(&format!("</{tag}>"));
    }
    out
}
fn action_heading(
    paragraph: ElementRef<'_>,
    source: &PreparedInclusionSourceLink,
) -> Option<String> {
    if paragraph
        .descendants()
        .filter_map(ElementRef::wrap)
        .filter(|child| child.value().attr("data-atlas-action-glyph").is_some())
        .count()
        != 1
    {
        return None;
    }
    let first = paragraph
        .children()
        .find(|n| !matches!(n.value(), Node::Text(t) if t.trim().is_empty()))?;
    let title = ElementRef::wrap(first)?;
    let refs = reference_ordinals(paragraph);
    let title_reference = title
        .value()
        .attr("data-atlas-reference")
        .and_then(|n| n.parse::<usize>().ok());
    let inner = if refs.len() == 1
        && title_reference.is_some_and(|n| source.matching_references.contains(&n))
    {
        // The original exact link stays in its host namespace.
        paragraph.inner_html()
    } else if refs.is_empty()
        && title.value().name() == "strong"
        && !has_links_or_controls(paragraph)
    {
        paragraph.inner_html().replacen(
            &title.html(),
            &source_heading_link(source, &title.inner_html()),
            1,
        )
    } else {
        return None;
    };
    let mut level = 3;
    let mut previous = paragraph.prev_sibling();
    while let Some(node) = previous {
        if let Some(heading) = ElementRef::wrap(node).filter(|e| is_heading(e.value().name())) {
            level = heading.value().name()[1..]
                .parse::<usize>()
                .ok()?
                .saturating_add(1)
                .min(6);
            break;
        }
        previous = node.prev_sibling();
    }
    Some(element_with_body(
        paragraph,
        &format!("h{level}"),
        &inner,
        true,
    ))
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
        let inserts = BTreeMap::from([(
            0,
            PreparedHtmlInclusion {
                html: "<section><p>Inserted rules</p></section>".into(),
                source_link: None,
            },
        )]);
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
    fn insertion(
        action: bool,
        matching_references: Vec<usize>,
    ) -> BTreeMap<usize, PreparedHtmlInclusion> {
        BTreeMap::from([(
            2,
            PreparedHtmlInclusion {
                html: "<section data-atlas-inclusion=\"9\"><p>Included rules.</p></section>".into(),
                source_link: Some(PreparedInclusionSourceLink {
                    namespace: 9,
                    matching_references,
                    action,
                }),
            },
        )])
    }
    #[test]
    fn heading_links_use_the_exact_inclusion_namespace_without_changing_the_heading() {
        let html = compose_prepared_inclusions(
            "<h3 id=\"paragon\"><strong>Paragon Benefit</strong></h3><p><span data-atlas-reference=\"2\">Embed</span></p>",
            &insertion(false, vec![]),
            false,
        );
        assert!(html.contains("<h3 id=\"paragon\"><span data-atlas-reference=\"0\" data-atlas-inclusion=\"9\"><strong>Paragon Benefit</strong></span></h3>"), "{html}");
        assert!(!html.contains("Source record"));
        assert!(html.contains("Included rules."));
    }
    #[test]
    fn action_title_lines_become_linked_headings_and_keep_glyphs_and_qualifiers() {
        for title in [
            "<span data-atlas-reference=\"1\">Reaction title</span>",
            "<strong>Reaction title</strong>",
        ] {
            let html = compose_prepared_inclusions(
                &format!(
                    "<p>{title} <span data-atlas-action-glyph=\"reaction\">reaction</span> (concentrate)</p><p><span data-atlas-reference=\"2\">Embed</span></p>"
                ),
                &insertion(true, vec![1]),
                false,
            );
            assert!(
                html.contains("<h3 data-atlas-inclusion-heading=\"true\">"),
                "{html}"
            );
            assert!(html.contains(
                "data-atlas-action-glyph=\"reaction\">reaction</span> (concentrate)</h3>"
            ));
            assert!(!html.contains("Source record"));
        }
    }
    #[test]
    fn prose_mentions_stay_inline_and_unheaded_insertions_remain_body_only() {
        let html = compose_prepared_inclusions(
            "<p>You gain <span data-atlas-reference=\"1\">Reaction title</span> <span data-atlas-action-glyph=\"reaction\">reaction</span>.</p><p><span data-atlas-reference=\"2\">Embed</span></p>",
            &insertion(true, vec![1]),
            false,
        );
        assert!(
            html.starts_with("<p>You gain ") && !html.contains("<h"),
            "{html}"
        );
        let untitled = compose_prepared_inclusions(
            "<p><span data-atlas-reference=\"2\">Embed</span></p>",
            &insertion(false, vec![]),
            false,
        );
        assert_eq!(
            untitled,
            "<section data-atlas-inclusion=\"9\"><p>Included rules.</p></section>"
        );
    }
    #[test]
    fn a_heading_before_multiple_consecutive_inclusions_is_not_assigned_to_one_target() {
        let mut inserts = insertion(false, vec![]);
        inserts.insert(
            3,
            PreparedHtmlInclusion {
                html: "<section>Other rules.</section>".into(),
                source_link: None,
            },
        );
        let html = compose_prepared_inclusions(
            "<h3>Combined benefits</h3><p><span data-atlas-reference=\"2\">First</span></p><p><span data-atlas-reference=\"3\">Second</span></p>",
            &inserts,
            false,
        );
        assert!(html.starts_with("<h3>Combined benefits</h3>"), "{html}");
        assert!(html.contains("Included rules.") && html.contains("Other rules."));
    }
}
