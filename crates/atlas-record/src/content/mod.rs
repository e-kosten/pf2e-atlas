use atlas_domain::RecordKey;
use serde::{Deserialize, Serialize};

mod render;
mod search_projection;
mod section_tree;
mod traversal;

pub use render::{render_markdown_like, render_plain_text};
pub use search_projection::{RecordFtsProjection, build_record_fts_projection};
pub use section_tree::{ContentSectionNode, ContentSectionOrigin, build_content_section_tree};
pub use traversal::{ContentReferenceIter, iter_content_references, visit_content_references_mut};

#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ContentDocument {
    pub blocks: Vec<ContentBlock>,
}

impl ContentDocument {
    pub fn new(blocks: Vec<ContentBlock>) -> Self {
        Self { blocks }
    }

    pub fn is_empty(&self) -> bool {
        self.blocks.is_empty()
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "camelCase")]
pub enum ContentBlock {
    Heading {
        level: u8,
        content: Vec<ContentInline>,
    },
    Paragraph {
        content: Vec<ContentInline>,
    },
    List {
        ordered: bool,
        items: Vec<Vec<ContentBlock>>,
    },
    Table {
        caption: Option<Vec<ContentInline>>,
        headers: Vec<Vec<ContentInline>>,
        rows: Vec<Vec<Vec<ContentInline>>>,
    },
    Callout {
        title: Option<Vec<ContentInline>>,
        blocks: Vec<ContentBlock>,
    },
    DefinitionList {
        items: Vec<ContentDefinitionItem>,
    },
    RuleBlock {
        title: Option<Vec<ContentInline>>,
        blocks: Vec<ContentBlock>,
    },
    Separator,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ContentDefinitionItem {
    pub term: Vec<ContentInline>,
    pub definition: Vec<ContentBlock>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "camelCase")]
pub enum ContentInline {
    Text {
        text: String,
    },
    Strong {
        content: Vec<ContentInline>,
    },
    Emphasis {
        content: Vec<ContentInline>,
    },
    Code {
        text: String,
    },
    Break,
    Reference {
        reference: ContentReference,
    },
    Roll {
        label: Option<String>,
        formula: String,
        raw: String,
    },
    Template {
        label: String,
        template_kind: Option<String>,
        raw: String,
    },
    Macro {
        label: Option<String>,
        raw: String,
    },
    ActionGlyph {
        action: String,
    },
    Icon {
        name: String,
        label: Option<String>,
    },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ContentReference {
    pub label: Option<Vec<ContentInline>>,
    pub locator: ContentReferenceLocator,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub resolved_key: Option<RecordKey>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub resolved_name: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "camelCase")]
pub enum ContentReferenceLocator {
    FoundryUuid { raw_target: String },
    Compendium { raw_target: String },
    PackAndLocator { pack_name: String, locator: String },
    Unknown { raw: String },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RecordContentDocument {
    pub source_kind: ContentSourceKind,
    pub label: Option<String>,
    pub document: ContentDocument,
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
    fn content_document_serializes_enums_with_stable_tags() {
        let document = ContentDocument::new(vec![
            ContentBlock::Heading {
                level: 2,
                content: vec![ContentInline::Text {
                    text: "Spell Effect".to_string(),
                }],
            },
            ContentBlock::Paragraph {
                content: vec![
                    ContentInline::Text {
                        text: "Use ".to_string(),
                    },
                    ContentInline::Reference {
                        reference: ContentReference {
                            label: Some(vec![ContentInline::Text {
                                text: "Heal".to_string(),
                            }]),
                            locator: ContentReferenceLocator::FoundryUuid {
                                raw_target: "Compendium.pf2e.spells-srd.Item.rfZpqmj0AIIdkVIs"
                                    .to_string(),
                            },
                            resolved_key: Some(
                                RecordKey::parse("spells-srd:rfZpqmj0AIIdkVIs")
                                    .expect("record key parses"),
                            ),
                            resolved_name: Some("Heal".to_string()),
                        },
                    },
                    ContentInline::Text {
                        text: " to restore vitality.".to_string(),
                    },
                ],
            },
        ]);

        let encoded = serde_json::to_value(&document).expect("document serializes");

        assert_eq!(encoded["blocks"][0]["kind"], "heading");
        assert_eq!(encoded["blocks"][1]["content"][1]["kind"], "reference");
        assert_eq!(
            encoded["blocks"][1]["content"][1]["reference"]["resolvedKey"],
            "spells-srd:rfZpqmj0AIIdkVIs"
        );

        let decoded: ContentDocument =
            serde_json::from_value(encoded).expect("document deserializes");
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
            document: ContentDocument::default(),
        };
        assert!(embedded_document.contributes_to_reference_occurrences());
        assert!(!embedded_document.contributes_to_default_backlinks());
        assert_eq!(
            ContentSourceKind::EmbeddedSpellDescription.fts_field(),
            ContentFtsField::EmbeddedContent
        );
    }
}
