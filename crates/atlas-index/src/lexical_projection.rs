//! Storage projection of record-owned lexical selections. A row describes exactly
//! one identity or section: conjunctions never borrow terms from a parent/sibling.
use crate::codec::sha256 as hash_document_embedding_input;
use crate::{SourceLexicalUnitInput, SourceLexicalUnitKind};
use atlas_domain::{SourceByteRange, SourcePassageAddress};
use atlas_record::source_record::{SOURCE_CONTENT_SELECTION_VERSION, SourceSearchSelection};
pub fn project_lexical_units(
    selection: &SourceSearchSelection,
    aliases: &[String],
) -> Vec<SourceLexicalUnitInput> {
    let mut output = Vec::new();
    if selection.identities.is_empty() {
        return output;
    }
    let root = selection
        .identities
        .iter()
        .find(|identity| identity.owners.is_empty());
    if let Some(root) = root {
        output.push(SourceLexicalUnitInput {
            record: selection.root.clone(),
            owners: vec![],
            field: None,
            address: None,
            kind: SourceLexicalUnitKind::RootName,
            identity_terms: root.name.clone(),
            alias_terms: String::new(),
            structured_terms: root.vocabulary.join(" "),
            definition_terms: String::new(),
        });
    }
    if let Some(root) = root {
        for alias in aliases {
            output.push(SourceLexicalUnitInput {
                record: selection.root.clone(),
                owners: vec![],
                field: None,
                address: None,
                kind: SourceLexicalUnitKind::RootAlias,
                identity_terms: String::new(),
                alias_terms: alias.clone(),
                structured_terms: root.vocabulary.join(" "),
                definition_terms: String::new(),
            });
        }
    }
    for identity in selection
        .identities
        .iter()
        .filter(|identity| !identity.owners.is_empty())
    {
        output.push(SourceLexicalUnitInput {
            record: selection.root.clone(),
            owners: identity.owners.clone(),
            field: None,
            address: None,
            kind: SourceLexicalUnitKind::OwnedName,
            identity_terms: String::new(),
            alias_terms: String::new(),
            structured_terms: identity.vocabulary.join(" "),
            definition_terms: identity.name.clone(),
        });
    }
    for field in &selection.fields {
        for section in field
            .sections
            .iter()
            .filter(|section| section.is_lexical_definition())
        {
            let kind = match section.kind {
                atlas_record::source_record::SourceSelectedSectionKind::Heading => {
                    SourceLexicalUnitKind::Heading
                }
                atlas_record::source_record::SourceSelectedSectionKind::DefinitionLabel => {
                    SourceLexicalUnitKind::DefinitionLabel
                }
                atlas_record::source_record::SourceSelectedSectionKind::Unlabeled => continue,
            };
            let address = field.prepared_html_sha256.as_ref().map(|html_hash| {
                SourcePassageAddress::HtmlSection {
                    prepared_html_sha256: html_hash.clone(),
                    canonical_text_sha256: hash_document_embedding_input(section.text.as_bytes()),
                    selection_version: SOURCE_CONTENT_SELECTION_VERSION.into(),
                    section_ordinal: section.section_ordinal,
                    label: section.label.clone(),
                    chunk_bytes: SourceByteRange {
                        start: 0,
                        end: section.text.len(),
                    },
                }
            });
            output.push(SourceLexicalUnitInput {
                record: selection.root.clone(),
                owners: field.locator.owners.clone(),
                field: Some(field.locator.field.clone()),
                address,
                kind,
                identity_terms: String::new(),
                alias_terms: String::new(),
                structured_terms: String::new(),
                definition_terms: section.label.clone().unwrap_or_default(),
            });
        }
    }
    output
}
