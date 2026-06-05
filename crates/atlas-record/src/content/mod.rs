use std::collections::BTreeMap;

use atlas_domain::RecordKey;
use serde::{Deserialize, Serialize};

mod render;
mod search_projection;
mod section_tree;
mod traversal;

pub use render::{render_markdown_like, render_plain_text};
pub use search_projection::{RecordFtsProjection, build_record_fts_projection};
pub use section_tree::{ContentSectionNode, ContentSectionOrigin, build_content_section_tree};
pub use traversal::{FoundryLinkIter, iter_foundry_links, visit_foundry_links_mut};

#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RichDocument {
    pub nodes: Vec<RichNode>,
}

impl RichDocument {
    pub fn new(nodes: Vec<RichNode>) -> Self {
        Self { nodes }
    }

    pub fn is_empty(&self) -> bool {
        self.nodes.is_empty()
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "camelCase")]
pub enum RichNode {
    Text {
        text: String,
    },
    HtmlElement {
        tag: String,
        attributes: BTreeMap<String, Option<String>>,
        children: Vec<RichNode>,
    },
    FoundryLink {
        link: FoundryLink,
    },
    Foundry {
        node: FoundryNode,
    },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FoundryLink {
    pub target: RichLinkTarget,
    pub label: Option<Vec<RichNode>>,
    pub source: FoundryLinkSource,
    pub behavior: FoundryLinkBehavior,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "camelCase")]
pub enum RichLinkTarget {
    Record {
        key: RecordKey,
        name: String,
    },
    LocalContent {
        content_key: String,
        label: Option<String>,
    },
    External {
        target: String,
        label: Option<String>,
    },
    Unresolved {
        target: String,
        fallback_label: String,
    },
}

impl RichLinkTarget {
    pub fn record_key(&self) -> Option<&RecordKey> {
        match self {
            Self::Record { key, .. } => Some(key),
            Self::LocalContent { .. } | Self::External { .. } | Self::Unresolved { .. } => None,
        }
    }

    pub fn display_name(&self) -> Option<&str> {
        match self {
            Self::Record { name, .. } => Some(name),
            Self::LocalContent { label, .. } | Self::External { label, .. } => label.as_deref(),
            Self::Unresolved { fallback_label, .. } => Some(fallback_label),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FoundryLinkSource {
    pub macro_kind: FoundryLinkMacroKind,
    pub authored_target: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub relation: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FoundryLinkMacroKind {
    Uuid,
    Compendium,
    Embed,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "camelCase")]
pub enum FoundryLinkBehavior {
    Reference,
    Embed {
        inline: bool,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        hr: Option<bool>,
        #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
        options: BTreeMap<String, String>,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ReferenceRelationKind {
    Reference,
    Embed,
}

impl ReferenceRelationKind {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Reference => "reference",
            Self::Embed => "embed",
        }
    }

    pub fn from_canonical(value: &str) -> Option<Self> {
        match value {
            "reference" => Some(Self::Reference),
            "embed" => Some(Self::Embed),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(
    tag = "kind",
    rename_all = "camelCase",
    rename_all_fields = "camelCase"
)]
pub enum FoundryNode {
    Check {
        statistic: Option<String>,
        options: BTreeMap<String, String>,
        label: Option<Vec<RichNode>>,
    },
    Damage {
        formula: String,
        options: BTreeMap<String, String>,
        damage_parts: Vec<DamagePart>,
        label: Option<Vec<RichNode>>,
    },
    InlineCommand {
        command: String,
        arguments: String,
        options: BTreeMap<String, String>,
        label: Option<Vec<RichNode>>,
    },
    Template {
        shape: Option<String>,
        options: BTreeMap<String, String>,
        label: Option<Vec<RichNode>>,
    },
    ActionGlyph {
        action: String,
    },
    Trait {
        traits: Vec<String>,
        label: Option<Vec<RichNode>>,
    },
    Localize {
        key: String,
        value: Option<Vec<RichNode>>,
    },
    UnknownFoundry {
        name: String,
        body: Option<String>,
        label: Option<Vec<RichNode>>,
        raw: String,
    },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DamagePart {
    pub formula: String,
    pub damage_type: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RecordContentDocument {
    pub source_kind: ContentSourceKind,
    pub label: Option<String>,
    pub document: RichDocument,
}

impl RecordContentDocument {
    pub const fn visibility(&self) -> ContentVisibility {
        self.source_kind.default_visibility()
    }

    pub const fn contributes_to_search(&self) -> bool {
        self.source_kind.default_contributes_to_search()
    }

    pub const fn contributes_to_reference_occurrences(&self) -> bool {
        self.source_kind
            .default_contributes_to_reference_occurrences()
    }

    pub const fn contributes_to_default_backlinks(&self) -> bool {
        self.source_kind.contributes_to_default_backlinks()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ContentSourceKind {
    Description,
    Blurb,
    Disable,
    Routine,
    Reset,
    StealthDetails,
    DetailsFieldDescription,
    PublicNotes,
    GmNotes,
    PrivateNotes,
    EmbeddedItemDescription,
    EmbeddedSpellDescription,
    GeneratedAffliction,
}

impl ContentSourceKind {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Description => "description",
            Self::Blurb => "blurb",
            Self::Disable => "disable",
            Self::Routine => "routine",
            Self::Reset => "reset",
            Self::StealthDetails => "stealth_details",
            Self::DetailsFieldDescription => "details_field_description",
            Self::PublicNotes => "public_notes",
            Self::GmNotes => "gm_notes",
            Self::PrivateNotes => "private_notes",
            Self::EmbeddedItemDescription => "embedded_item_description",
            Self::EmbeddedSpellDescription => "embedded_spell_description",
            Self::GeneratedAffliction => "generated_affliction",
        }
    }

    pub fn from_canonical(value: &str) -> Option<Self> {
        match value {
            "description" => Some(Self::Description),
            "blurb" => Some(Self::Blurb),
            "disable" => Some(Self::Disable),
            "routine" => Some(Self::Routine),
            "reset" => Some(Self::Reset),
            "stealth_details" => Some(Self::StealthDetails),
            "details_field_description" => Some(Self::DetailsFieldDescription),
            "public_notes" => Some(Self::PublicNotes),
            "gm_notes" => Some(Self::GmNotes),
            "private_notes" => Some(Self::PrivateNotes),
            "embedded_item_description" => Some(Self::EmbeddedItemDescription),
            "embedded_spell_description" => Some(Self::EmbeddedSpellDescription),
            "generated_affliction" => Some(Self::GeneratedAffliction),
            _ => None,
        }
    }

    pub const fn default_visibility(self) -> ContentVisibility {
        match self {
            Self::GmNotes => ContentVisibility::GmOnly,
            Self::PrivateNotes => ContentVisibility::Private,
            _ => ContentVisibility::Public,
        }
    }

    pub const fn contributes_to_default_retrieval(self) -> bool {
        match self.default_visibility() {
            ContentVisibility::Public => true,
            ContentVisibility::GmOnly
            | ContentVisibility::Private
            | ContentVisibility::Internal => false,
        }
    }

    pub const fn contributes_to_default_backlinks(self) -> bool {
        match self {
            Self::EmbeddedItemDescription
            | Self::EmbeddedSpellDescription
            | Self::GmNotes
            | Self::PrivateNotes => false,
            _ => self.contributes_to_default_retrieval(),
        }
    }

    pub const fn fts_field(self) -> ContentFtsField {
        match self {
            Self::Disable | Self::Routine | Self::Reset | Self::StealthDetails => {
                ContentFtsField::Facts
            }
            Self::EmbeddedItemDescription | Self::EmbeddedSpellDescription => {
                ContentFtsField::EmbeddedContent
            }
            _ => ContentFtsField::Body,
        }
    }

    pub const fn default_contributes_to_search(self) -> bool {
        self.contributes_to_default_retrieval()
    }

    pub const fn default_contributes_to_reference_occurrences(self) -> bool {
        self.contributes_to_default_retrieval()
    }

    pub const fn is_embedded(self) -> bool {
        matches!(
            self,
            Self::EmbeddedItemDescription | Self::EmbeddedSpellDescription
        )
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ContentVisibility {
    Public,
    GmOnly,
    Private,
    Internal,
}

impl ContentVisibility {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Public => "public",
            Self::GmOnly => "gm_only",
            Self::Private => "private",
            Self::Internal => "internal",
        }
    }

    pub fn from_canonical(value: &str) -> Option<Self> {
        match value {
            "public" => Some(Self::Public),
            "gm_only" => Some(Self::GmOnly),
            "private" => Some(Self::Private),
            "internal" => Some(Self::Internal),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum ContentFtsField {
    Body,
    Facts,
    EmbeddedContent,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rich_document_serializes_enums_with_stable_tags() {
        let document = RichDocument::new(vec![
            RichNode::HtmlElement {
                tag: "h2".to_string(),
                attributes: BTreeMap::new(),
                children: vec![RichNode::Text {
                    text: "Spell Effect".to_string(),
                }],
            },
            RichNode::HtmlElement {
                tag: "p".to_string(),
                attributes: BTreeMap::new(),
                children: vec![
                    RichNode::Text {
                        text: "Use ".to_string(),
                    },
                    RichNode::FoundryLink {
                        link: FoundryLink {
                            label: Some(vec![RichNode::Text {
                                text: "Heal".to_string(),
                            }]),
                            target: RichLinkTarget::Record {
                                key: RecordKey::parse("spells-srd:rfZpqmj0AIIdkVIs")
                                    .expect("record key parses"),
                                name: "Heal".to_string(),
                            },
                            source: FoundryLinkSource {
                                macro_kind: FoundryLinkMacroKind::Uuid,
                                authored_target: "Compendium.pf2e.spells-srd.Item.rfZpqmj0AIIdkVIs"
                                    .to_string(),
                                relation: None,
                            },
                            behavior: FoundryLinkBehavior::Reference,
                        },
                    },
                    RichNode::Text {
                        text: " to restore vitality.".to_string(),
                    },
                ],
            },
        ]);

        let encoded = serde_json::to_value(&document).expect("document serializes");

        assert_eq!(encoded["nodes"][0]["kind"], "htmlElement");
        assert_eq!(encoded["nodes"][1]["children"][1]["kind"], "foundryLink");
        assert_eq!(
            encoded["nodes"][1]["children"][1]["link"]["target"]["key"],
            "spells-srd:rfZpqmj0AIIdkVIs"
        );

        let decoded: RichDocument = serde_json::from_value(encoded).expect("document deserializes");
        assert_eq!(decoded, document);
    }

    #[test]
    fn source_kind_policies_are_stable() {
        assert_eq!(ContentSourceKind::Description.as_str(), "description");
        assert_eq!(
            ContentSourceKind::GmNotes.default_visibility(),
            ContentVisibility::GmOnly
        );
        assert!(!ContentSourceKind::PrivateNotes.contributes_to_default_retrieval());
        assert!(!ContentSourceKind::EmbeddedItemDescription.contributes_to_default_backlinks());
        assert!(
            ContentSourceKind::EmbeddedItemDescription
                .default_contributes_to_reference_occurrences()
        );
        let embedded_document = RecordContentDocument {
            source_kind: ContentSourceKind::EmbeddedSpellDescription,
            label: None,
            document: RichDocument::default(),
        };
        assert!(embedded_document.contributes_to_reference_occurrences());
        assert!(!embedded_document.contributes_to_default_backlinks());
        assert_eq!(
            ContentSourceKind::EmbeddedSpellDescription.fts_field(),
            ContentFtsField::EmbeddedContent
        );
    }
}
