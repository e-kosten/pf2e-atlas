//! Read-time terminal formatting through html2text, with bounded HTML fixes for
//! its unsupported rules, table captions and explicit list-number resets.
use scraper::{
    ElementRef, Html, Selector,
    node::{Node, Text},
};

fn selector(value: &str) -> Result<Selector, String> {
    Selector::parse(value)
        .map_err(|error| format!("cannot select prepared HTML formatting: {error}"))
}
pub(super) fn render_html(html: &str, width: usize) -> Result<String, String> {
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
                text: "────────────".into(),
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
    html2text::from_read(fragment.root_element().inner_html().as_bytes(), width)
        .map_err(|e| format!("cannot render prepared HTML: {e}"))
}
