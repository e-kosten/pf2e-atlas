//! Source-backed fixtures for retrieval and runtime contract tests.
use atlas_domain::{SourceByteRange, SourcePassageAddress};
use atlas_foundry_model::{
    SNAPSHOT_VERSION, SOURCE_CONTRACT_ID, SourceContext, admit_document_source,
};
use atlas_index::*;
use atlas_record::{
    source_content::{CONTENT_INTERPRETATION_VERSION, ContentAudience, ContentVisibilityRule},
    source_record::{
        SOURCE_CONTENT_SELECTION_VERSION, SourceBackedRecord, SourceSelectedSectionKind,
        prepare_record_content, select_record_search,
    },
};
use serde_json::Value;
use sha2::{Digest, Sha256};
use std::collections::BTreeMap;
/// Owns the source artifact for the lifetime of a fixture service.
pub struct FixtureArtifact {
    directory: tempfile::TempDir,
}
impl FixtureArtifact {
    pub fn path(&self) -> std::path::PathBuf {
        self.directory.path().join("artifact.sqlite")
    }
}
pub fn open_source_fixture(
    records: Vec<SourceArtifactRecordInput>,
    semantic: bool,
) -> Result<(crate::AtlasRetrievalService, FixtureArtifact), crate::SearchError> {
    open_input_fixture(input(records, semantic))
}
/// Allows tests to supply verified alias/pair evidence and selected unit vectors.
pub fn open_input_fixture(
    input: IndexBuildInput,
) -> Result<(crate::AtlasRetrievalService, FixtureArtifact), crate::SearchError> {
    let artifact = FixtureArtifact {
        directory: tempfile::tempdir()
            .map_err(|e| crate::SearchError::query_failed(e.to_string()))?,
    };
    SqliteIndexWriter::new(artifact.path()).write(&input)?;
    let reader = if input.context.semantic_model.is_some() {
        SqliteIndexReader::open_read_only_with_vectors(artifact.path())?
    } else {
        SqliteIndexReader::open_read_only(artifact.path())?
    };
    Ok((
        crate::AtlasRetrievalService::from_prepared_index_without_embeddings(reader),
        artifact,
    ))
}
impl crate::AtlasRetrievalService {
    pub fn read_metrics(&self) -> SourceReadMetrics {
        self.index.read_metrics()
    }
    pub fn reset_read_metrics(&self) {
        self.index.reset_read_metrics();
    }
}
pub fn hash(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}
pub fn audience() -> ContentAudience {
    ContentAudience {
        include_gm: true,
        include_owner: true,
        implicit_check_dc: ContentVisibilityRule::All,
    }
}
pub fn context(semantic: bool) -> SourceArtifactBuildContext {
    SourceArtifactBuildContext {
        format_version: ARTIFACT_FORMAT_VERSION,
        snapshot_version: SNAPSHOT_VERSION,
        source_contract: SOURCE_CONTRACT_ID.into(),
        source_revision: None,
        source_fingerprint: "a".repeat(64),
        source_file_count: 1,
        indexing_locale: "en".into(),
        locale_catalog_sha256: "b".repeat(64),
        english_catalog_sha256: "b".repeat(64),
        localization_policy: LOCALIZATION_POLICY_VERSION.into(),
        audience: audience(),
        content_interpretation_version: CONTENT_INTERPRETATION_VERSION.into(),
        content_selection_version: SOURCE_CONTENT_SELECTION_VERSION.into(),
        asset_policy: ASSET_POLICY_VERSION.into(),
        relationship_policy: RELATIONSHIP_POLICY_VERSION.into(),
        query_projection_version: SOURCE_PROJECTION_VERSION.into(),
        query_catalog_version: query_capabilities().unwrap().version,
        lookup_version: SOURCE_LOOKUP_VERSION.into(),
        lexical_selection_version: LEXICAL_SELECTION_VERSION.into(),
        fts_tokenizer: "unicode61 remove_diacritics 2".into(),
        used_trait_labels: BTreeMap::new(),
        semantic_model: semantic.then(SourceSemanticModelIdentity::current),
    }
}
pub fn record(pack: &str, kind: &str, value: Value) -> SourceArtifactRecordInput {
    let bytes = serde_json::to_vec(&value).unwrap();
    let source = admit_document_source(kind, SourceContext::new(pack, "fixture", "$"), &bytes)
        .unwrap()
        .model
        .unwrap();
    let record = SourceBackedRecord::new(pack, source).unwrap();
    let content = prepare_record_content(&record, audience(), None, None);
    let relationships = atlas_record::source_record::resolve_source_relationships(&record, None);
    SourceArtifactRecordInput {
        source_path: format!("packs/{pack}/{}.json", value["_id"].as_str().unwrap()),
        content_hash: hash(&bytes),
        preparation_context_hash: String::new(),
        record,
        content,
        relationships,
        diagnostics: vec![],
    }
}
pub fn input(mut records: Vec<SourceArtifactRecordInput>, semantic: bool) -> IndexBuildInput {
    let mut references = atlas_record::source_record::SourceReferenceIndex::default();
    for record in &records {
        references.insert_source(record.record.key(), record.record.source());
    }
    for record in &mut records {
        record.content =
            prepare_record_content(&record.record, audience(), None, Some(&references));
        record.relationships = atlas_record::source_record::resolve_source_relationships(
            &record.record,
            Some(&references),
        );
    }
    let mut context = context(semantic);
    context.source_file_count = records.len();
    let ctxhash = context.sha256().unwrap();
    let packs = records
        .iter()
        .map(|r| {
            (
                r.record.key().pack().as_str().to_owned(),
                atlas_record::source_record::SourceNodeView::from(r.record.source())
                    .document_kind()
                    .to_owned(),
            )
        })
        .collect::<BTreeMap<_, _>>()
        .into_iter()
        .map(|(pack_id, document_kind)| IndexBuildPack {
            label: pack_id.clone(),
            declared_path: format!("packs/{pack_id}"),
            pack_id,
            document_kind,
        })
        .collect();
    let mut lexical_units = Vec::new();
    let mut semantic_units = Vec::new();
    for (index, r) in records.iter_mut().enumerate() {
        r.preparation_context_hash = ctxhash.clone();
        let selection = select_record_search(
            &r.record,
            &r.content,
            audience(),
            r.record.key().pack().as_str(),
            &BTreeMap::new(),
        )
        .unwrap();
        let root_name = selection
            .identities
            .iter()
            .find(|i| i.owners.is_empty())
            .map(|i| i.name.clone())
            .unwrap_or_default();
        let root_vocabulary = selection
            .identities
            .iter()
            .find(|i| i.owners.is_empty())
            .map(|i| i.vocabulary.join(" "))
            .unwrap_or_default();
        for identity in &selection.identities {
            lexical_units.push(SourceLexicalUnitInput {
                record: r.record.key().clone(),
                owners: identity.owners.clone(),
                field: None,
                address: None,
                kind: if identity.owners.is_empty() {
                    SourceLexicalUnitKind::RootName
                } else {
                    SourceLexicalUnitKind::OwnedName
                },
                identity_terms: root_name.clone(),
                alias_terms: String::new(),
                structured_terms: identity.vocabulary.join(" "),
                definition_terms: if identity.owners.is_empty() {
                    String::new()
                } else {
                    identity.name.clone()
                },
            });
        }
        let mut vector = vec![0.0; 384];
        vector[index % 384] = 1.0;
        if semantic && !selection.identity_required.is_empty() {
            semantic_units.push(SourceSemanticUnitInput {
                record: r.record.key().clone(),
                owners: vec![],
                field: None,
                address: SourcePassageAddress::Identity {},
                chunk_ordinal: 0,
                input_token_count: 8,
                input_hash: hash(selection.identity_required.as_bytes()),
                vector: vector.clone(),
            });
        }
        for field in selection.fields {
            for section in field.sections {
                let range = SourceByteRange {
                    start: 0,
                    end: section.text.len(),
                };
                let address = if let Some(htmlhash) = &field.prepared_html_sha256 {
                    SourcePassageAddress::HtmlSection {
                        prepared_html_sha256: htmlhash.clone(),
                        canonical_text_sha256: hash(section.text.as_bytes()),
                        selection_version: SOURCE_CONTENT_SELECTION_VERSION.into(),
                        section_ordinal: section.section_ordinal,
                        label: section.label.clone(),
                        chunk_bytes: range,
                    }
                } else {
                    SourcePassageAddress::PlainSection {
                        source_text_sha256: hash(section.text.as_bytes()),
                        selection_version: SOURCE_CONTENT_SELECTION_VERSION.into(),
                        section_ordinal: section.section_ordinal,
                        label: section.label.clone(),
                        chunk_bytes: range,
                    }
                };
                if section.is_lexical_definition() {
                    lexical_units.push(SourceLexicalUnitInput {
                        record: r.record.key().clone(),
                        owners: field.locator.owners.clone(),
                        field: Some(field.locator.field.clone()),
                        address: Some(address.clone()),
                        kind: match section.kind {
                            SourceSelectedSectionKind::Heading => SourceLexicalUnitKind::Heading,
                            SourceSelectedSectionKind::DefinitionLabel => {
                                SourceLexicalUnitKind::DefinitionLabel
                            }
                            _ => unreachable!(),
                        },
                        identity_terms: selection.identity_required.clone(),
                        alias_terms: String::new(),
                        structured_terms: root_vocabulary.clone(),
                        definition_terms: section.label.unwrap(),
                    });
                }
                if semantic {
                    semantic_units.push(SourceSemanticUnitInput {
                        record: r.record.key().clone(),
                        owners: field.locator.owners.clone(),
                        field: Some(field.locator.field.clone()),
                        address,
                        chunk_ordinal: 0,
                        input_token_count: 8,
                        input_hash: hash(section.text.as_bytes()),
                        vector: vector.clone(),
                    });
                }
            }
        }
    }
    IndexBuildInput {
        records,
        packs,
        context,
        lexical_units,
        semantic_units,
        aliases: vec![],
        remaster_pairs: vec![],
    }
}
