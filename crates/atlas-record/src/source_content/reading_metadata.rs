//! Bounded reading metadata from explicit authored conventions.
use super::{escape_content_text, inclusion::element_with_body};
use ego_tree::NodeRef;
use scraper::{ElementRef, Html, Node};
use std::collections::BTreeMap;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RoutineHeader {
    pub actions: String,
    pub traits: Vec<String>,
    pub html: String,
}

/// Recognize only a complete parenthesized header in the first paragraph's
/// leading text. Keep its paragraph and every attributed marker in place.
pub fn routine_header(html: &str) -> Option<RoutineHeader> {
    let fragment = Html::parse_fragment(html);
    let root = fragment.root_element();
    let first = root
        .children()
        .find(|node| !matches!(node.value(), Node::Text(t) if t.trim().is_empty()))?;
    let paragraph = ElementRef::wrap(first)?;
    if paragraph.value().name() != "p" {
        return None;
    }
    let text_node = first.first_child()?;
    let Node::Text(text) = text_node.value() else {
        return None;
    };
    let trimmed = text.trim_start();
    let (header, tail) = trimmed.strip_prefix('(')?.split_once(')')?;
    if header.len() > 256 || !tail.is_empty() && !tail.starts_with(char::is_whitespace) {
        return None;
    }
    let (count, traits) = match header.split_once(';') {
        Some((count, traits)) => {
            let traits: Vec<_> = traits.split(',').map(str::trim).collect();
            if traits.iter().any(|t| {
                t.is_empty()
                    || !t
                        .bytes()
                        .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == b'-')
            }) {
                return None;
            }
            (count, traits.into_iter().map(str::to_owned).collect())
        }
        None => (header, vec![]),
    };
    let actions = action_budget(count.trim())?;
    let mut paragraph_body = escape_content_text(tail.trim_start());
    for child in text_node.next_siblings() {
        paragraph_body.push_str(&node_html(child));
    }
    let replacement = element_with_body(paragraph, "p", &paragraph_body, false);
    let html = root
        .children()
        .map(|node| {
            if node.id() == first.id() {
                replacement.clone()
            } else {
                node_html(node)
            }
        })
        .collect();
    Some(RoutineHeader {
        actions,
        traits,
        html,
    })
}

fn node_html(node: NodeRef<'_, Node>) -> String {
    match node.value() {
        Node::Text(text) => escape_content_text(text),
        Node::Element(_) => ElementRef::wrap(node).map(|e| e.html()).unwrap_or_default(),
        _ => String::new(),
    }
}

fn action_budget(text: &str) -> Option<String> {
    let normalized = text.to_ascii_lowercase();
    let count = normalized
        .strip_suffix(" actions")
        .or_else(|| normalized.strip_suffix(" action"))?;
    for (separator, display) in [(" to ", "–"), (" or ", " or ")] {
        if let Some((a, b)) = count.split_once(separator) {
            let (a, b) = (action_number(a)?, action_number(b)?);
            if a >= b {
                return None;
            }
            return Some(format!("{a}{display}{b} actions"));
        }
    }
    let n = action_number(count)?;
    Some(format!("{n} {}", if n == 1 { "action" } else { "actions" }))
}
fn action_number(text: &str) -> Option<u32> {
    let n = match text {
        "one" => 1,
        "two" => 2,
        "three" => 3,
        "four" => 4,
        "five" => 5,
        "six" => 6,
        "seven" => 7,
        "eight" => 8,
        "nine" => 9,
        "ten" => 10,
        "eleven" => 11,
        "twelve" => 12,
        _ if !text.is_empty() && text.bytes().all(|c| c.is_ascii_digit()) => text.parse().ok()?,
        _ => return None,
    };
    (n > 0).then_some(n)
}

/// Foundry's explicit trait override suppresses immediate Item inheritance.
/// Actor traits never enter this context. Unknown identifiers remain authored.
pub fn check_traits(options: &BTreeMap<String, String>, item_traits: &[String]) -> Vec<String> {
    let mut traits = Vec::new();
    let mut add = |value: &str| {
        for trait_ in value.split(',').map(str::trim).filter(|t| !t.is_empty()) {
            if !traits.iter().any(|t| t == trait_) {
                traits.push(trait_.to_owned());
            }
        }
    };
    if let Some(value) = options.get("traits") {
        add(value);
    }
    if let Some(value) = options.get("overrideTraits") {
        if !value.trim().is_empty() && !value.trim().eq_ignore_ascii_case("true") {
            add(value);
        }
    } else {
        for trait_ in item_traits {
            add(trait_);
        }
    }
    traits
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn routine_budgets_preserve_body_markers_and_support_authored_variants() {
        for (prefix, expected) in [
            ("1 action; illusion, mental, occult, visual", "1 action"),
            ("7 actions", "7 actions"),
            ("3 Actions", "3 actions"),
            ("twelve actions", "12 actions"),
            ("1 to 3 actions", "1–3 actions"),
            ("one or two actions; mental", "1 or 2 actions"),
        ] {
            let html = format!(
                "<p data-atlas-source-section=\"0\">({prefix}) Body <span data-atlas-interaction=\"0\">DC 26 Will save</span>.</p><p>Next.</p>"
            );
            let header = routine_header(&html).unwrap();
            assert_eq!(header.actions, expected);
            assert!(header.html.contains("data-atlas-source-section=\"0\""));
            assert!(header.html.contains("data-atlas-interaction=\"0\""));
            assert!(header.html.contains("Body "));
            assert!(!header.html.contains(&format!("({prefix})")));
        }
    }
    #[test]
    fn ambiguous_headers_and_nonleading_parentheses_stay_authored() {
        for html in [
            "<p>(many actions) Body</p>",
            "<p>(3 actions; see below) Body</p>",
            "<p>(0 actions) Body</p>",
            "<p>(3 to 1 actions) Body</p>",
            "<p>Body (3 actions)</p>",
            "<p><strong>(3 actions)</strong> Body</p>",
            "<h3>Title</h3><p>(3 actions) Body</p>",
            "<p>(3 actions)word</p>",
        ] {
            assert!(routine_header(html).is_none(), "{html}");
        }
    }
    #[test]
    fn check_traits_merge_deduplicate_override_and_retain_unknowns() {
        let item = vec!["mental".into(), "secret".into()];
        let options = BTreeMap::from([("traits".into(), "illusion,mental,homebrew-trait".into())]);
        assert_eq!(
            check_traits(&options, &item),
            ["illusion", "mental", "homebrew-trait", "secret"]
        );
        for value in ["", "true", "TRUE"] {
            let mut options = options.clone();
            options.insert("overrideTraits".into(), value.into());
            assert_eq!(
                check_traits(&options, &item),
                ["illusion", "mental", "homebrew-trait"]
            );
        }
        assert_eq!(
            check_traits(
                &BTreeMap::from([("overrideTraits".into(), "visual".into())]),
                &item
            ),
            ["visual"]
        );
        assert!(
            check_traits(
                &BTreeMap::from([("overrideTraits".into(), "".into())]),
                &item
            )
            .is_empty()
        );
    }
}
