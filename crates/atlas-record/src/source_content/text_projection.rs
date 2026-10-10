use scraper::{Html, Selector, node::Node};

/// Undecorated canonical words and useful block breaks from the library.
/// A bounded DOM fix preserves table captions ignored by the library.
/// Prepared browser HTML remains untouched. This is not another renderer or
/// Foundry parser, and it never reparses authored source.
pub fn plain_text(html: &str) -> Result<String, html2text::Error> {
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
    html2text::config::with_decorator(html2text::render::TrivialDecorator::new())
        .string_from_read(fragment.root_element().inner_html().as_bytes(), 1_000_000)
        .map(|text| text.trim().to_string())
}

#[cfg(test)]
mod tests {
    use super::plain_text;
    #[test]
    fn canonical_words_have_no_renderer_decoration() {
        let text=plain_text(r#"<h2>Dragon Form</h2><blockquote><strong>Bold</strong> <em>words</em></blockquote><ul><li>First</li><li>Second</li></ul><ol start="5"><li value="8">Third</li></ol><p><a href="https://example.com">Named link</a></p>"#).unwrap();
        for word in [
            "Dragon Form",
            "Bold words",
            "First",
            "Second",
            "Third",
            "Named link",
        ] {
            assert!(text.contains(word), "{text}");
        }
        for marker in [
            "##",
            "> ",
            "* ",
            "8.",
            "[Named link]",
            "https://example.com",
        ] {
            assert!(!text.contains(marker), "{text}");
        }
        assert!(text.contains('\n'));
    }
}
