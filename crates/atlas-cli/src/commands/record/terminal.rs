//! Read-time terminal formatting through html2text, with bounded HTML fixes for
//! its unsupported rules, table captions and explicit list-number resets.
use scraper::{
    ElementRef, Html, Selector,
    node::{Node, Text},
};
use std::io::IsTerminal;

#[derive(Clone, Copy)]
pub(crate) struct TerminalLayout {
    pub width: usize,
    pub styled: bool,
}
impl TerminalLayout {
    pub fn current(width: usize) -> Self {
        Self {
            width,
            styled: supports_styling(
                std::io::stdout().is_terminal(),
                std::env::var_os("NO_COLOR").is_some_and(|value| !value.is_empty()),
                std::env::var("TERM").ok().as_deref(),
            ),
        }
    }
}
fn supports_styling(tty: bool, no_color: bool, term: Option<&str>) -> bool {
    tty && !no_color && term != Some("dumb")
}

fn selector(value: &str) -> Result<Selector, String> {
    Selector::parse(value)
        .map_err(|error| format!("cannot select prepared HTML formatting: {error}"))
}
pub(crate) fn render_html(html: &str, layout: TerminalLayout) -> Result<String, String> {
    let mut fragment = Html::parse_fragment(html);
    let rules = fragment
        .select(&selector("hr")?)
        .map(|e| e.id())
        .collect::<Vec<_>>();
    for id in rules {
        if let Some(mut node) = fragment.tree.get_mut(id) {
            if let Node::Element(element) = node.value() {
                element.name.local = "p".into();
            }
            node.append(Node::Text(Text {
                text: "─".repeat(layout.width).into(),
            }));
        }
    }
    let captions = fragment
        .select(&selector("table > caption")?)
        .filter_map(|e| Some((e.id(), e.parent()?.id())))
        .collect::<Vec<_>>();
    for (caption, table) in captions {
        if let Some(mut node) = fragment.tree.get_mut(caption)
            && let Node::Element(element) = node.value()
        {
            element.name.local = "p".into();
        }
        if let Some(mut node) = fragment.tree.get_mut(table) {
            node.insert_id_before(caption);
        }
    }
    let lists = fragment
        .select(&selector("ol")?)
        .filter_map(|list| {
            let entries = list
                .children()
                .filter_map(ElementRef::wrap)
                .filter(|e| e.value().name() == "li")
                .map(|e| {
                    (
                        e.id(),
                        e.value().attr("value").and_then(|v| v.parse::<i64>().ok()),
                    )
                })
                .collect::<Vec<_>>();
            entries.iter().any(|(_, value)| value.is_some()).then(|| {
                (
                    list.id(),
                    list.value()
                        .attr("start")
                        .and_then(|v| v.parse::<i64>().ok())
                        .unwrap_or(1),
                    entries,
                )
            })
        })
        .collect::<Vec<_>>();
    for (list, mut ordinal, entries) in lists {
        if let Some(mut node) = fragment.tree.get_mut(list)
            && let Node::Element(element) = node.value()
        {
            element.name.local = "div".into();
        }
        let count = entries.len();
        for (index, (item, reset)) in entries.into_iter().enumerate() {
            ordinal = reset.unwrap_or(ordinal);
            if let Some(mut node) = fragment.tree.get_mut(item) {
                if let Node::Element(element) = node.value() {
                    element.name.local = "p".into();
                }
                node.prepend(Node::Text(Text {
                    text: format!("{ordinal}. ").into(),
                }));
            }
            if index + 1 < count {
                ordinal = ordinal
                    .checked_add(1)
                    .ok_or("prepared list numbering exceeds terminal range")?;
            }
        }
    }
    if layout.styled {
        // RichDecorator marks emphasis but deliberately keeps Markdown heading
        // prefixes. Use block emphasis for headings in the styled rendition;
        // html2text still owns wrapping, links, tables and all text extraction.
        let headings = fragment
            .select(&selector("h1,h2,h3,h4,h5,h6")?)
            .map(|node| node.id())
            .collect::<Vec<_>>();
        for id in headings {
            let children = fragment
                .tree
                .get(id)
                .map(|node| node.children().map(|child| child.id()).collect::<Vec<_>>())
                .unwrap_or_default();
            if let Some(mut node) = fragment.tree.get_mut(id) {
                let mut strong = node.value().clone();
                if let Node::Element(element) = &mut strong {
                    element.name.local = "strong".into();
                }
                if let Node::Element(element) = node.value() {
                    element.name.local = "div".into();
                }
                let mut emphasized = node.append(strong);
                for child in children {
                    emphasized.append_id(child);
                }
            }
        }
        return html2text::from_read_coloured(
            fragment.root_element().inner_html().as_bytes(),
            layout.width,
            |annotations, text| {
                use html2text::render::RichAnnotation;
                let codes: Vec<_> = annotations
                    .iter()
                    .filter_map(|annotation| match annotation {
                        RichAnnotation::Strong => Some("1"),
                        RichAnnotation::Emphasis => Some("3"),
                        RichAnnotation::Strikeout => Some("9"),
                        RichAnnotation::Code => Some("36"),
                        RichAnnotation::Link(_) => Some("4"),
                        _ => None,
                    })
                    .collect();
                if codes.is_empty() {
                    text.to_owned()
                } else {
                    format!("\x1b[{}m{text}\x1b[0m", codes.join(";"))
                }
            },
        )
        .map_err(|e| format!("cannot render prepared HTML: {e}"));
    }
    html2text::from_read(
        fragment.root_element().inner_html().as_bytes(),
        layout.width,
    )
    .map_err(|e| format!("cannot render prepared HTML: {e}"))
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn horizontal_rules_use_the_output_width_in_plain_and_styled_reading() {
        for width in [40, 80, 120] {
            for styled in [false, true] {
                let text = render_html(
                    "<p>Before</p><hr><p>After</p>",
                    TerminalLayout { width, styled },
                )
                .unwrap();
                let rule = "─".repeat(width);
                assert_eq!(text.lines().filter(|line| *line == rule).count(), 1);
                let before = text.find("Before").unwrap();
                let separator = text.find(&rule).unwrap();
                let after = text.find("After").unwrap();
                assert!(before < separator && separator < after);
            }
        }
    }
    #[test]
    fn styled_html_uses_annotations_without_markdown_prefixes() {
        let html = "<h2>Harrowkin Agency</h2><p><strong>Trigger</strong> A creature moves. <em>Once per round.</em></p><ul><li>Retain the list</li></ul>";
        let styled = render_html(
            html,
            TerminalLayout {
                width: 40,
                styled: true,
            },
        )
        .unwrap();
        assert!(styled.contains("\x1b[1mHarrowkin Agency\x1b[0m"));
        assert!(styled.contains("\x1b[1mTrigger\x1b[0m"));
        assert!(styled.contains("\x1b[3mOnce per\x1b[0m\n\x1b[3mround.\x1b[0m"));
        assert!(!styled.contains("##"));
        assert!(!styled.contains("**"));
        assert!(styled.contains("* Retain the list"));
        let plain = render_html(
            html,
            TerminalLayout {
                width: 40,
                styled: false,
            },
        )
        .unwrap();
        assert!(plain.contains("## Harrowkin Agency"));
        assert!(!plain.contains('\x1b'));
    }
    #[test]
    fn automatic_styling_respects_output_capabilities_and_opt_out() {
        assert!(supports_styling(true, false, Some("xterm-256color")));
        assert!(!supports_styling(false, false, Some("xterm-256color")));
        assert!(!supports_styling(true, true, Some("xterm-256color")));
        assert!(!supports_styling(true, false, Some("dumb")));
    }
}
