use super::{FormulaContext, contextual_damage_text};
use crate::source_content::{
    ContentInteraction, ContentInteractionKind, ContentInteractionLabel, escape_content_text,
};
use scraper::{Html, Node};
use std::collections::BTreeMap;

/// Replace only generated interaction bodies in one checked field namespace.
/// Authored labels and all reference markers remain owned by the source.
pub fn render_contextual_interactions(
    html: &str,
    interactions: &[ContentInteraction],
    context: &FormulaContext,
) -> String {
    let replacements: BTreeMap<_, _> = interactions
        .iter()
        .filter_map(|i| {
            let body = match (&i.kind, &i.label) {
                (
                    ContentInteractionKind::Damage { formula, options },
                    ContentInteractionLabel::Damage | ContentInteractionLabel::HealingPoints,
                ) => contextual_damage_text(
                    formula,
                    context,
                    options.contains_key("shortLabel")
                        || matches!(i.label, ContentInteractionLabel::HealingPoints),
                )?
                .html(),
                (
                    ContentInteractionKind::Check { statistic, options },
                    ContentInteractionLabel::Check {
                        noun,
                        secret_in_prose,
                    },
                ) => escape_content_text(&crate::source_content::read_only_text::check_text(
                    statistic.as_deref().unwrap_or("[missing check type]"),
                    options,
                    noun.as_deref(),
                    *secret_in_prose,
                    context,
                )),
                (
                    ContentInteractionKind::Command {
                        command, arguments, ..
                    },
                    ContentInteractionLabel::Roll,
                ) => super::contextual_roll_text(command, arguments, context)?.html(),
                _ => return None,
            };
            Some((i.ordinal, body))
        })
        .collect();
    if replacements.is_empty() {
        return html.into();
    }
    let fragment = Html::parse_fragment(html);
    fn render(node: ego_tree::NodeRef<'_, Node>, replacements: &BTreeMap<usize, String>) -> String {
        match node.value() {
            Node::Text(text) => escape_content_text(text),
            Node::Element(element) => {
                let replacement = element
                    .attr("data-atlas-interaction")
                    .and_then(|n| n.parse::<usize>().ok())
                    .and_then(|n| replacements.get(&n));
                let body = replacement.cloned().unwrap_or_else(|| {
                    node.children()
                        .map(|child| render(child, replacements))
                        .collect()
                });
                let Some(element_ref) = scraper::ElementRef::wrap(node) else {
                    return String::new();
                };
                crate::source_content::inclusion::element_with_body(
                    element_ref,
                    element.name(),
                    &body,
                    false,
                )
            }
            _ => String::new(),
        }
    }
    fragment
        .root_element()
        .children()
        .map(|node| render(node, &replacements))
        .collect()
}
