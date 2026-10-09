use scraper::{
    Html, Selector,
    node::{Node, Text},
};

/// Library formatting with two bounded DOM fixes for verified library gaps:
/// captions are ignored inside tables; li[value] does not reset numbering.
/// Prepared browser HTML remains untouched. This is not another renderer or
/// Foundry parser, and it never reparses authored source.
pub(super) fn plain_text(html: &str) -> Result<String, html2text::Error> {
    let mut fragment = Html::parse_fragment(html);
    if let Ok(selector) = Selector::parse("table > caption") {
        let captions = fragment
            .select(&selector)
            .filter_map(|caption| Some((caption.id(), caption.parent()?.id())))
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
    }
    if let Ok(selector) = Selector::parse("ol") {
        let lists = fragment
            .select(&selector)
            .filter_map(|list| {
                let items = list
                    .children()
                    .filter_map(scraper::ElementRef::wrap)
                    .filter(|element| element.value().name() == "li")
                    .map(|item| {
                        (
                            item.id(),
                            item.value()
                                .attr("value")
                                .and_then(|value| value.parse::<i64>().ok()),
                        )
                    })
                    .collect::<Vec<_>>();
                items.iter().any(|(_, value)| value.is_some()).then(|| {
                    (
                        list.id(),
                        list.value()
                            .attr("start")
                            .and_then(|value| value.parse::<i64>().ok())
                            .unwrap_or(1),
                        items,
                    )
                })
            })
            .collect::<Vec<_>>();
        for (list, mut ordinal, items) in lists {
            if let Some(mut node) = fragment.tree.get_mut(list)
                && let Node::Element(element) = node.value()
            {
                element.name.local = "div".into();
            }
            for (item, reset) in items {
                ordinal = reset.unwrap_or(ordinal);
                if let Some(mut node) = fragment.tree.get_mut(item) {
                    if let Node::Element(element) = node.value() {
                        element.name.local = "p".into();
                    }
                    node.prepend(Node::Text(Text {
                        text: format!("{ordinal}. ").into(),
                    }));
                }
                ordinal = ordinal.saturating_add(1);
            }
        }
    }
    html2text::config::plain_no_decorate()
        .string_from_read(fragment.root_element().inner_html().as_bytes(), 1_000_000)
        .map(|text| text.trim().to_string())
}
