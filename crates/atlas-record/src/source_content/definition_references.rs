//! Authored localization references used by contextual definitions.
use serde::{Deserialize, Serialize};

use super::{
    ContentAudience, ContentVisibilityRule,
    parsed_markup::{ParsedMacro, ParsedNode},
    parser::parse_foundry_content_with_localization,
};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AuthoredLocalizationOccurrence {
    pub localization_key: String,
    pub path: String,
    pub visible: bool,
}

pub fn authored_localization_occurrences(
    markup: &str,
    audience: ContentAudience,
    visibility: ContentVisibilityRule,
) -> Vec<AuthoredLocalizationOccurrence> {
    let parsed = parse_foundry_content_with_localization(markup, None);
    let mut references = Vec::new();
    fn visit(
        nodes: &[ParsedNode],
        parent: &str,
        references: &mut Vec<AuthoredLocalizationOccurrence>,
        audience: ContentAudience,
        visible: bool,
    ) {
        for (index, node) in nodes.iter().enumerate() {
            let path = format!("{parent}/{index}");
            let mut visible = visible;
            let children = match node {
                ParsedNode::HtmlElement {
                    tag,
                    attributes,
                    children,
                } => {
                    if matches!(tag.as_str(), "script" | "style" | "iframe" | "object") {
                        continue;
                    }
                    if let Some(value) =
                        attributes.get("data-visibility").and_then(Option::as_deref)
                    {
                        visible = visible
                            && ContentVisibilityRule::from_source(value)
                                .is_some_and(|rule| audience.permits(rule));
                    }
                    Some((children.as_slice(), path.clone()))
                }
                ParsedNode::ParsedLink { link } => link
                    .label
                    .as_deref()
                    .map(|label| (label, format!("{path}/label"))),
                ParsedNode::Foundry { node } => {
                    let label = match node {
                        ParsedMacro::Localize { key, label, .. } => {
                            references.push(AuthoredLocalizationOccurrence {
                                localization_key: key.clone(),
                                path: path.clone(),
                                visible,
                            });
                            visible = false; // Custom Localize labels are ignored by preparation.
                            label
                        }
                        ParsedMacro::UnknownFoundry { label, .. } => {
                            visible = false;
                            label
                        }
                        ParsedMacro::Check { label, .. }
                        | ParsedMacro::Damage { label, .. }
                        | ParsedMacro::InlineCommand { label, .. }
                        | ParsedMacro::Template { label, .. } => label,
                    };
                    label
                        .as_deref()
                        .map(|label| (label, format!("{path}/label")))
                }
                ParsedNode::Text { .. } => None,
            };
            if let Some((children, path)) = children {
                visit(children, &path, references, audience, visible);
            }
        }
    }
    visit(
        &parsed.document.nodes,
        "",
        &mut references,
        audience,
        audience.permits(visibility),
    );
    references
}
