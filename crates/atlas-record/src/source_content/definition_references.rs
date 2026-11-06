//! Exact authored localization references to uniquely identified definition vocabulary.
use atlas_domain::{DefinitionKey, TraitKey};
#[cfg(test)]
use atlas_domain::{OwnedContentIdentity, OwnedContentLocator};
use atlas_foundry_model::generated::{FoundryTraitDefinition, trait_definitions};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};

use super::{
    ContentAudience, ContentVisibilityRule, SourceContentLocator,
    parsed_markup::{ParsedMacro, ParsedNode},
    parser::parse_foundry_content_with_localization,
};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TraitLocalizationReference {
    pub trait_key: TraitKey,
    pub locator: SourceContentLocator,
    pub localization_key: String,
    /// Exact interpretation-local occurrence path within the authored field.
    pub path: String,
    pub visible: bool,
}

/// Find authored Localize occurrences whose exact key identifies one trait.
/// Repeated/shared keys never create aliases or merge parameterized identities.
/// Resolved localization prose is not recursively treated as authored source.
pub fn trait_localization_references(
    locator: SourceContentLocator,
    markup: &str,
    audience: ContentAudience,
    visibility: ContentVisibilityRule,
) -> Vec<TraitLocalizationReference> {
    static KEYS: std::sync::OnceLock<DefinitionLocalizationMap> = std::sync::OnceLock::new();
    references_with_keys(
        locator,
        markup,
        KEYS.get_or_init(|| definition_keys(trait_definitions())),
        audience,
        visibility,
    )
}

fn definition_keys(definitions: &[FoundryTraitDefinition]) -> DefinitionLocalizationMap {
    let mut keys = DefinitionLocalizationMap::new();
    for definition in definitions {
        for key in [definition.label_key, definition.description_key]
            .into_iter()
            .flatten()
        {
            if let Ok(key_value) = TraitKey::new(definition.identifier) {
                keys.entry(key.to_owned())
                    .or_default()
                    .insert(key_value.into());
            }
        }
    }
    keys
}

#[cfg(test)]
fn references_with_definitions(
    locator: SourceContentLocator,
    markup: &str,
    definitions: &[FoundryTraitDefinition],
    audience: ContentAudience,
    visibility: ContentVisibilityRule,
) -> Vec<TraitLocalizationReference> {
    references_with_keys(
        locator,
        markup,
        &definition_keys(definitions),
        audience,
        visibility,
    )
}

fn references_with_keys(
    locator: SourceContentLocator,
    markup: &str,
    keys: &DefinitionLocalizationMap,
    audience: ContentAudience,
    visibility: ContentVisibilityRule,
) -> Vec<TraitLocalizationReference> {
    definition_localization_references(locator, markup, keys, audience, visibility)
        .into_iter()
        .filter_map(|reference| {
            reference
                .key
                .trait_key()
                .map(|trait_key| TraitLocalizationReference {
                    trait_key,
                    locator: reference.locator,
                    localization_key: reference.localization_key,
                    path: reference.path,
                    visible: reference.visible,
                })
        })
        .collect()
}

/// Exact authored localization keys supplied by the selected definition catalog.
pub type DefinitionLocalizationMap = BTreeMap<String, BTreeSet<DefinitionKey>>;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DefinitionLocalizationReference {
    pub key: DefinitionKey,
    pub locator: SourceContentLocator,
    pub localization_key: String,
    pub path: String,
    pub visible: bool,
}

/// Source-neutral evidence for standalone definition bodies, without synthetic records.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DefinitionLocalizationOccurrence {
    pub key: DefinitionKey,
    pub localization_key: String,
    pub path: String,
    pub visible: bool,
}

/// The existing authored parser owns traversal; expanded prose is not authored evidence.
pub fn definition_localization_references(
    locator: SourceContentLocator,
    markup: &str,
    keys: &DefinitionLocalizationMap,
    audience: ContentAudience,
    visibility: ContentVisibilityRule,
) -> Vec<DefinitionLocalizationReference> {
    definition_body_localization_references(markup, keys, audience, visibility)
        .into_iter()
        .map(|reference| DefinitionLocalizationReference {
            key: reference.key,
            locator: locator.clone(),
            localization_key: reference.localization_key,
            path: reference.path,
            visible: reference.visible,
        })
        .collect()
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AuthoredLocalizationOccurrence {
    pub localization_key: String,
    pub path: String,
    pub visible: bool,
}

pub fn definition_body_localization_references(
    markup: &str,
    keys: &DefinitionLocalizationMap,
    audience: ContentAudience,
    visibility: ContentVisibilityRule,
) -> Vec<DefinitionLocalizationOccurrence> {
    authored_localization_occurrences(markup, audience, visibility)
        .into_iter()
        .filter_map(|reference| {
            let identifiers = keys
                .get(&reference.localization_key)
                .filter(|keys| keys.len() == 1)?;
            Some(DefinitionLocalizationOccurrence {
                key: identifiers.first()?.clone(),
                localization_key: reference.localization_key,
                path: reference.path,
                visible: reference.visible,
            })
        })
        .collect()
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

#[cfg(test)]
mod tests {
    use super::*;

    fn public() -> ContentAudience {
        ContentAudience {
            include_gm: false,
            include_owner: false,
            implicit_check_dc: ContentVisibilityRule::All,
        }
    }

    #[test]
    fn authored_keys_preserve_owner_field_and_distinct_occurrences() {
        let locator = SourceContentLocator {
            record: "bestiary:test".parse().unwrap(),
            owners: vec![OwnedContentLocator {
                collection: "/items".into(),
                identity: OwnedContentIdentity::Stable("strike".into()),
            }],
            field: "/system/description/value".into(),
        };
        let refs = trait_localization_references(
            locator.clone(),
            "<p>@Localize[PF2E.TraitDescriptionAgile] @Localize[PF2E.TraitDescriptionAgile]</p> Agile @Localize[not-a-trait] @Localize[PF2E.TraitDescriptionDeadly]",
            public(),
            ContentVisibilityRule::All,
        );
        assert_eq!(
            refs.len(),
            2,
            "shared Deadly keys and label prose do not establish exact identities"
        );
        assert!(
            refs.iter()
                .all(|r| r.trait_key.identifier() == "agile" && r.locator == locator)
        );
        assert_ne!(refs[0].path, refs[1].path);
        let hidden = trait_localization_references(
            locator.clone(),
            "<p data-visibility='gm'>@Localize[PF2E.TraitDescriptionAgile]</p><p data-visibility='unexpected'>@Localize[PF2E.TraitDescriptionAgile]</p>",
            public(),
            ContentVisibilityRule::All,
        );
        assert_eq!(hidden.len(), 2);
        assert!(hidden.iter().all(|reference| !reference.visible));
        let hidden = trait_localization_references(
            locator,
            "@Localize[PF2E.TraitDescriptionAgile]",
            public(),
            ContentVisibilityRule::None,
        );
        assert!(!hidden[0].visible);
    }

    #[test]
    fn repeated_mapping_for_one_identity_is_not_ambiguous_but_shared_label_keys_are() {
        let definitions = [
            FoundryTraitDefinition {
                identifier: "one",
                label_key: Some("same"),
                description_key: Some("same"),
                catalogs: &[],
                source_locations: &[],
            },
            FoundryTraitDefinition {
                identifier: "two",
                label_key: Some("shared"),
                description_key: None,
                catalogs: &[],
                source_locations: &[],
            },
            FoundryTraitDefinition {
                identifier: "three",
                label_key: Some("shared"),
                description_key: None,
                catalogs: &[],
                source_locations: &[],
            },
        ];
        let refs = references_with_definitions(
            SourceContentLocator {
                record: "test:id".parse().unwrap(),
                owners: vec![],
                field: "/description".into(),
            },
            "@Localize[same] @Localize[shared]<script>@Localize[same]</script>",
            &definitions,
            public(),
            ContentVisibilityRule::All,
        );
        assert_eq!(refs.len(), 1);
        assert_eq!(refs[0].trait_key.identifier(), "one");
    }
}
