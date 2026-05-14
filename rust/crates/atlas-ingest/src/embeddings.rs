use std::collections::{BTreeMap, BTreeSet};

use atlas_embedding::{
    EmbeddingError, EmbeddingInputChunk, EmbeddingInputSection, EmbeddingInputTokenization,
    EmbeddingSectionTruncation, TextEmbedder, TextEmbeddingTokenizer,
    hash_document_embedding_input, render_embedding_chunks_for_embedding,
    render_presentation_document_embedding_chunks,
};
use atlas_record::build_record_presentation_document;
use serde_json::Value;
use tracing::info;

use crate::embedding_units::{
    extract_structured_embedding_units, strip_markup_for_embedding_units,
};
use crate::{LoadedRecord, RecordAlias, RemasterLink};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PendingDocumentEmbedding {
    pub embedding_unit_key: String,
    pub record_key: String,
    pub unit_kind: EmbeddingUnitKind,
    pub label: Option<String>,
    pub ordinal: usize,
    pub input_chunks: Vec<EmbeddingInputChunk>,
    pub input_text: String,
    pub input_hash: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum EmbeddingUnitKind {
    Parent,
    HeadingSection,
    TitledOption,
    ActivationBlock,
}

impl EmbeddingUnitKind {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Parent => "parent",
            Self::HeadingSection => "heading_section",
            Self::TitledOption => "titled_option",
            Self::ActivationBlock => "activation_block",
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct GeneratedDocumentEmbedding {
    pub embedding_unit_key: String,
    pub record_key: String,
    pub unit_kind: EmbeddingUnitKind,
    pub label: Option<String>,
    pub ordinal: usize,
    pub input_hash: String,
    pub dimensions: usize,
    pub vector: Vec<f32>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ReusableDocumentEmbedding {
    pub input_hash: String,
    pub dimensions: usize,
    pub vector: Vec<f32>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct GeneratedDocumentEmbeddings {
    pub embeddings: Vec<GeneratedDocumentEmbedding>,
    pub reused_count: usize,
    pub generated_count: usize,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct DocumentEmbeddingTokenizationTelemetry {
    pub document_count: usize,
    pub truncated_document_count: usize,
    pub max_token_count: Option<usize>,
    pub max_observed_token_count: usize,
    pub total_observed_token_count: usize,
    pub total_tokens_over_limit: usize,
    pub section_truncations: Vec<DocumentEmbeddingSectionTruncation>,
    pub truncated_examples: Vec<DocumentEmbeddingTruncationExample>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DocumentEmbeddingTruncationExample {
    pub record_key: String,
    pub token_count: usize,
    pub max_token_count: usize,
    pub truncated_sections: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DocumentEmbeddingSectionTruncation {
    pub section: String,
    pub document_count: usize,
    pub dropped_chunk_count: usize,
}

pub(crate) fn build_pending_document_embeddings(
    records: &[LoadedRecord],
    aliases: &[RecordAlias],
    remaster_links: &[RemasterLink],
) -> Vec<PendingDocumentEmbedding> {
    let aliases_by_key = aliases_by_record_key(aliases);
    let hidden_record_keys = remaster_links
        .iter()
        .map(|link| link.legacy_record_key.to_string())
        .collect::<BTreeSet<_>>();
    let mut pending = Vec::new();

    for record in records {
        let record_key = record.key.to_string();
        if !record.is_default_visible || hidden_record_keys.contains(&record_key) {
            continue;
        }
        let aliases = aliases_by_key.get(&record_key).cloned().unwrap_or_default();
        pending.extend(build_record_embedding_units(record, &aliases));
    }

    pending
}

fn build_record_embedding_units(
    record: &LoadedRecord,
    aliases: &[String],
) -> Vec<PendingDocumentEmbedding> {
    let record_key = record.key.to_string();
    let raw_description = raw_description_markup(record);
    let compact_description = raw_description
        .as_deref()
        .map(strip_markup_for_embedding_units)
        .filter(|value| !value.trim().is_empty());
    let mut compact_record = record.clone();
    if compact_description.is_some() {
        compact_record.description_text = compact_description;
    }
    let document = build_record_presentation_document(&compact_record);
    let parent_chunks = document_embedding_input_chunks(&document, aliases);
    let mut units = vec![pending_embedding_unit(
        format!("{record_key}#parent"),
        record_key.clone(),
        EmbeddingUnitKind::Parent,
        None,
        0,
        parent_chunks,
    )];

    if let Some(markup) = raw_description {
        let child_units = extract_structured_embedding_units(&record.name, &markup);
        let context = child_embedding_context_chunks(&document);
        for child in child_units {
            let mut chunks = context.clone();
            chunks.push(EmbeddingInputChunk::line(
                EmbeddingInputSection::Description,
                format!("Section: {}", child.label),
            ));
            chunks.push(EmbeddingInputChunk::truncatable_line(
                EmbeddingInputSection::Description,
                format!("Description: {}", child.body),
            ));
            units.push(pending_embedding_unit(
                format!("{record_key}#{}:{}", child.kind.as_str(), child.ordinal),
                record_key.clone(),
                child.kind,
                Some(child.label),
                child.ordinal,
                chunks,
            ));
        }
    }

    units
}

fn pending_embedding_unit(
    embedding_unit_key: String,
    record_key: String,
    unit_kind: EmbeddingUnitKind,
    label: Option<String>,
    ordinal: usize,
    input_chunks: Vec<EmbeddingInputChunk>,
) -> PendingDocumentEmbedding {
    let input_text = render_embedding_chunks_for_embedding(&input_chunks);
    let input_hash = hash_document_embedding_input(&input_text);
    PendingDocumentEmbedding {
        embedding_unit_key,
        record_key,
        unit_kind,
        label,
        ordinal,
        input_chunks,
        input_text,
        input_hash,
    }
}

fn document_embedding_input_chunks(
    document: &atlas_record::RecordPresentationDocument,
    aliases: &[String],
) -> Vec<EmbeddingInputChunk> {
    let mut chunks = render_presentation_document_embedding_chunks(document);
    if !aliases.is_empty() {
        chunks.push(EmbeddingInputChunk::truncatable_line(
            EmbeddingInputSection::Aliases,
            format!("Aliases: {}", aliases.join(", ")),
        ));
    }
    chunks
}

fn child_embedding_context_chunks(
    document: &atlas_record::RecordPresentationDocument,
) -> Vec<EmbeddingInputChunk> {
    render_presentation_document_embedding_chunks(document)
        .into_iter()
        .filter(|chunk| {
            matches!(
                chunk.section,
                EmbeddingInputSection::Identity
                    | EmbeddingInputSection::Traits
                    | EmbeddingInputSection::Classification
                    | EmbeddingInputSection::Summary
            )
        })
        .collect()
}

fn raw_description_markup(record: &LoadedRecord) -> Option<String> {
    let raw = serde_json::from_str::<Value>(&record.raw_json).ok()?;
    raw.pointer("/system/description/value")
        .and_then(Value::as_str)
        .map(ToOwned::to_owned)
}

pub fn apply_document_embedding_token_budget(
    pending: &mut [PendingDocumentEmbedding],
    tokenizer: &TextEmbeddingTokenizer,
) -> Result<DocumentEmbeddingTokenizationTelemetry, EmbeddingError> {
    let inputs = pending
        .iter()
        .map(|entry| entry.input_text.as_str())
        .collect::<Vec<_>>();
    let tokenizations = tokenizer.analyze_document_inputs(&inputs)?;
    let mut truncated_sections_by_record =
        BTreeMap::<String, Vec<EmbeddingSectionTruncation>>::new();
    for (entry, tokenization) in pending.iter_mut().zip(tokenizations.iter()) {
        if !tokenization.truncated {
            continue;
        }
        let budgeted = tokenizer.budget_document_input(&entry.input_chunks)?;
        if !budgeted.truncated_sections.is_empty() {
            truncated_sections_by_record
                .insert(entry.record_key.clone(), budgeted.truncated_sections);
        }
        entry.input_text = budgeted.text;
        entry.input_hash = hash_document_embedding_input(&entry.input_text);
    }
    Ok(summarize_document_embedding_tokenization(
        pending,
        &tokenizations,
        &truncated_sections_by_record,
    ))
}

pub fn generate_document_embeddings(
    pending: &[PendingDocumentEmbedding],
    embedder: &mut TextEmbedder,
) -> Result<Vec<GeneratedDocumentEmbedding>, EmbeddingError> {
    Ok(
        generate_document_embeddings_with_reuse_using(pending, None, |input| {
            embedder.embed_document(input)
        })?
        .embeddings,
    )
}

pub fn generate_document_embeddings_with_reuse(
    pending: &[PendingDocumentEmbedding],
    reusable_embeddings: Option<&BTreeMap<String, ReusableDocumentEmbedding>>,
    embedder: &mut TextEmbedder,
) -> Result<GeneratedDocumentEmbeddings, EmbeddingError> {
    generate_document_embeddings_with_reuse_using_batch(pending, reusable_embeddings, 1, |inputs| {
        embedder.embed_documents(inputs)
    })
}

pub fn generate_document_embeddings_with_reuse_using<E>(
    pending: &[PendingDocumentEmbedding],
    reusable_embeddings: Option<&BTreeMap<String, ReusableDocumentEmbedding>>,
    mut embed_document: impl FnMut(&str) -> Result<Vec<f32>, E>,
) -> Result<GeneratedDocumentEmbeddings, E> {
    generate_document_embeddings_with_reuse_using_batch(pending, reusable_embeddings, 1, |inputs| {
        inputs
            .iter()
            .map(|input| embed_document(input))
            .collect::<Result<Vec<_>, _>>()
    })
}

pub fn generate_document_embeddings_with_reuse_using_batch<E>(
    pending: &[PendingDocumentEmbedding],
    reusable_embeddings: Option<&BTreeMap<String, ReusableDocumentEmbedding>>,
    batch_size: usize,
    mut embed_documents: impl FnMut(&[&str]) -> Result<Vec<Vec<f32>>, E>,
) -> Result<GeneratedDocumentEmbeddings, E> {
    let total = pending.len();
    let progress_interval = embedding_progress_interval(total);
    let batch_size = batch_size.max(1);
    let mut generated = vec![None; total];
    let mut pending_generation_indices = Vec::new();
    let mut reused_count = 0;
    let mut generated_count = 0;
    let mut last_reported = 0;
    for (index, entry) in pending.iter().enumerate() {
        if let Some(reusable) = reusable_embeddings
            .and_then(|lookup| lookup.get(&entry.embedding_unit_key))
            .filter(|reusable| reusable.input_hash == entry.input_hash)
        {
            reused_count += 1;
            generated[index] = Some(GeneratedDocumentEmbedding {
                embedding_unit_key: entry.embedding_unit_key.clone(),
                record_key: entry.record_key.clone(),
                unit_kind: entry.unit_kind,
                label: entry.label.clone(),
                ordinal: entry.ordinal,
                input_hash: entry.input_hash.clone(),
                dimensions: reusable.dimensions,
                vector: reusable.vector.clone(),
            });
            continue;
        }
        pending_generation_indices.push(index);
    }

    for chunk in pending_generation_indices.chunks(batch_size) {
        let current = chunk.last().copied().unwrap_or(0) + 1;
        let record_key = pending[current - 1].record_key.as_str();
        let inputs = chunk
            .iter()
            .map(|index| pending[*index].input_text.as_str())
            .collect::<Vec<_>>();
        let vectors = embed_documents(&inputs)?;
        debug_assert_eq!(vectors.len(), chunk.len());
        for (chunk_index, vector) in vectors.into_iter().enumerate() {
            let index = chunk[chunk_index];
            let entry = &pending[index];
            generated_count += 1;
            generated[index] = Some(GeneratedDocumentEmbedding {
                embedding_unit_key: entry.embedding_unit_key.clone(),
                record_key: entry.record_key.clone(),
                unit_kind: entry.unit_kind,
                label: entry.label.clone(),
                ordinal: entry.ordinal,
                input_hash: entry.input_hash.clone(),
                dimensions: vector.len(),
                vector,
            });
        }
        if last_reported == 0 || current - last_reported >= progress_interval || current == total {
            info!(target: "atlas_progress",
                phase = "document_embeddings",
                current = current as u64,
                total = total as u64,
                "Prepared document embedding batch through: {record_key}",
                record_key = record_key
            );
            last_reported = current;
        }
    }
    if pending_generation_indices.is_empty() && total > 0 {
        info!(target: "atlas_progress",
            phase = "document_embeddings",
            current = total as u64,
            total = total as u64,
            "Prepared document embeddings from reusable cache"
        );
    }

    Ok(GeneratedDocumentEmbeddings {
        embeddings: generated
            .into_iter()
            .map(|entry| entry.expect("every pending embedding is generated or reused"))
            .collect(),
        reused_count,
        generated_count,
    })
}

fn embedding_progress_interval(total: usize) -> usize {
    (total / 100).clamp(25, 500)
}

fn summarize_document_embedding_tokenization(
    pending: &[PendingDocumentEmbedding],
    tokenizations: &[EmbeddingInputTokenization],
    truncated_sections_by_record: &BTreeMap<String, Vec<EmbeddingSectionTruncation>>,
) -> DocumentEmbeddingTokenizationTelemetry {
    debug_assert_eq!(pending.len(), tokenizations.len());
    let max_token_count = tokenizations
        .iter()
        .find_map(|tokenization| tokenization.max_token_count);
    let max_observed_token_count = tokenizations
        .iter()
        .map(|tokenization| tokenization.token_count)
        .max()
        .unwrap_or(0);
    let total_observed_token_count = tokenizations
        .iter()
        .map(|tokenization| tokenization.token_count)
        .sum();
    let total_tokens_over_limit = tokenizations
        .iter()
        .filter_map(|tokenization| {
            let max_token_count = tokenization.max_token_count?;
            Some(tokenization.token_count.saturating_sub(max_token_count))
        })
        .sum();
    let truncated_document_count = tokenizations
        .iter()
        .filter(|tokenization| tokenization.truncated)
        .count();
    let mut truncated_examples = pending
        .iter()
        .zip(tokenizations.iter())
        .filter_map(|(entry, tokenization)| {
            let max_token_count = tokenization.max_token_count?;
            tokenization
                .truncated
                .then(|| DocumentEmbeddingTruncationExample {
                    record_key: entry.record_key.clone(),
                    token_count: tokenization.token_count,
                    max_token_count,
                    truncated_sections: truncated_sections_by_record
                        .get(&entry.record_key)
                        .map(|sections| {
                            sections
                                .iter()
                                .map(|section| section.section.as_str().to_string())
                                .collect()
                        })
                        .unwrap_or_default(),
                })
        })
        .collect::<Vec<_>>();
    truncated_examples.sort_by(|left, right| {
        right
            .token_count
            .cmp(&left.token_count)
            .then_with(|| left.record_key.cmp(&right.record_key))
    });
    truncated_examples.truncate(10);
    let section_truncations = summarize_section_truncations(truncated_sections_by_record);

    DocumentEmbeddingTokenizationTelemetry {
        document_count: pending.len(),
        truncated_document_count,
        max_token_count,
        max_observed_token_count,
        total_observed_token_count,
        total_tokens_over_limit,
        section_truncations,
        truncated_examples,
    }
}

fn summarize_section_truncations(
    truncated_sections_by_record: &BTreeMap<String, Vec<EmbeddingSectionTruncation>>,
) -> Vec<DocumentEmbeddingSectionTruncation> {
    let mut by_section = BTreeMap::<String, (usize, usize)>::new();
    for sections in truncated_sections_by_record.values() {
        for section in sections {
            let entry = by_section
                .entry(section.section.as_str().to_string())
                .or_insert((0, 0));
            entry.0 += 1;
            entry.1 += section.dropped_chunk_count;
        }
    }
    by_section
        .into_iter()
        .map(|(section, (document_count, dropped_chunk_count))| {
            DocumentEmbeddingSectionTruncation {
                section,
                document_count,
                dropped_chunk_count,
            }
        })
        .collect()
}

fn aliases_by_record_key(aliases: &[RecordAlias]) -> BTreeMap<String, Vec<String>> {
    let mut by_key = BTreeMap::<String, Vec<String>>::new();
    for alias in aliases {
        by_key
            .entry(alias.canonical_record_key.to_string())
            .or_default()
            .push(alias.alias_text.clone());
    }
    for aliases in by_key.values_mut() {
        aliases.sort();
        aliases.dedup();
    }
    by_key
}

#[cfg(test)]
mod tests {
    use atlas_domain::{PackName, RecordFamily, RecordId, RecordKey};

    use super::*;
    use crate::record_model::{AliasSource, LoadedRecord};

    #[test]
    fn builds_pending_inputs_for_default_visible_records_only() {
        let visible = test_record("packs:visible1", "Visible Record", true);
        let hidden = test_record("packs:hidden1", "Hidden Record", false);
        let aliases = vec![RecordAlias {
            canonical_record_key: visible.key.clone(),
            alias_text: "Legacy Visible".to_string(),
            normalized_alias: "legacy visible".to_string(),
            source: AliasSource::Migration,
            source_ref: "fixture".to_string(),
        }];

        let pending = build_pending_document_embeddings(&[visible, hidden], &aliases, &[]);

        assert_eq!(pending.len(), 1);
        assert_eq!(pending[0].record_key, "packs:visible1");
        assert_eq!(
            pending[0].input_text,
            "Name: Visible Record\n\
Family: Rule\n\
Type: Action\n\
Traits: Healing\n\
Classification: Fixture family\n\
Description: Fixture description.\n\
Families: Fixture family\n\
Publication Family: Unknown\n\
Pack: packs\n\
Foundry Document Type: Item\n\
Aliases: Legacy Visible"
        );
    }

    #[test]
    fn excludes_remaster_hidden_legacy_records() {
        let legacy = test_record("packs:legacy1", "Legacy Record", true);
        let remaster = test_record("packs:remaster1", "Remaster Record", true);
        let remaster_link = RemasterLink {
            remaster_record_key: remaster.key.clone(),
            legacy_record_key: legacy.key.clone(),
            source: atlas_domain::RemasterLinkSource::Migration,
            source_ref: "fixture".to_string(),
        };

        let pending = build_pending_document_embeddings(&[legacy, remaster], &[], &[remaster_link]);

        assert_eq!(
            pending
                .iter()
                .map(|entry| entry.record_key.as_str())
                .collect::<Vec<_>>(),
            vec!["packs:remaster1"]
        );
    }

    #[test]
    fn generates_document_vectors_from_pending_inputs() {
        let pending = vec![
            pending_embedding_with_hash("packs:first", "first input", "first-hash"),
            pending_embedding_with_hash("packs:second", "second input", "second-hash"),
        ];

        let generated = generate_document_embeddings_with_reuse_using(&pending, None, |input| {
            Ok::<_, std::convert::Infallible>(vec![input.len() as f32, 1.0])
        })
        .expect("fixture embedding should succeed")
        .embeddings;

        assert_eq!(generated.len(), 2);
        assert_eq!(generated[0].record_key, "packs:first");
        assert_eq!(generated[0].input_hash, "first-hash");
        assert_eq!(generated[0].dimensions, 2);
        assert_eq!(generated[0].vector, vec![11.0, 1.0]);
        assert_eq!(generated[1].record_key, "packs:second");
        assert_eq!(generated[1].input_hash, "second-hash");
        assert_eq!(generated[1].dimensions, 2);
        assert_eq!(generated[1].vector, vec![12.0, 1.0]);
    }

    #[test]
    fn reuses_matching_document_vectors() {
        let pending = vec![
            pending_embedding_with_hash("packs:first", "first input", "first-hash"),
            pending_embedding_with_hash("packs:second", "second input", "second-hash"),
        ];
        let reusable = BTreeMap::from([(
            "packs:first#parent".to_string(),
            ReusableDocumentEmbedding {
                input_hash: "first-hash".to_string(),
                dimensions: 2,
                vector: vec![9.0, 1.0],
            },
        )]);

        let generated =
            generate_document_embeddings_with_reuse_using(&pending, Some(&reusable), |input| {
                Ok::<_, std::convert::Infallible>(vec![input.len() as f32, 1.0])
            })
            .expect("fixture embedding should succeed");

        assert_eq!(generated.reused_count, 1);
        assert_eq!(generated.generated_count, 1);
        assert_eq!(generated.embeddings[0].vector, vec![9.0, 1.0]);
        assert_eq!(generated.embeddings[1].vector, vec![12.0, 1.0]);
    }

    #[test]
    fn generates_missing_document_vectors_in_batches() {
        let pending = vec![
            pending_embedding_with_hash("packs:first", "first input", "first-hash"),
            pending_embedding_with_hash("packs:second", "second input", "second-hash"),
            pending_embedding_with_hash("packs:third", "third input", "third-hash"),
        ];
        let mut batch_lengths = Vec::new();

        let generated =
            generate_document_embeddings_with_reuse_using_batch(&pending, None, 2, |inputs| {
                batch_lengths.push(inputs.len());
                Ok::<_, std::convert::Infallible>(
                    inputs
                        .iter()
                        .map(|input| vec![input.len() as f32, inputs.len() as f32])
                        .collect(),
                )
            })
            .expect("fixture embedding should succeed");

        assert_eq!(batch_lengths, vec![2, 1]);
        assert_eq!(generated.generated_count, 3);
        assert_eq!(
            generated
                .embeddings
                .iter()
                .map(|entry| entry.record_key.as_str())
                .collect::<Vec<_>>(),
            vec!["packs:first", "packs:second", "packs:third"]
        );
        assert_eq!(generated.embeddings[0].vector, vec![11.0, 2.0]);
        assert_eq!(generated.embeddings[2].vector, vec![11.0, 1.0]);
    }

    #[test]
    fn summarizes_document_embedding_tokenization_truncation_examples() {
        let pending = vec![
            pending_embedding("packs:short", "short"),
            pending_embedding("packs:long", "long"),
            pending_embedding("packs:longer", "longer"),
        ];
        let tokenizations = vec![
            EmbeddingInputTokenization {
                token_count: 8,
                max_token_count: Some(10),
                truncated: false,
            },
            EmbeddingInputTokenization {
                token_count: 12,
                max_token_count: Some(10),
                truncated: true,
            },
            EmbeddingInputTokenization {
                token_count: 20,
                max_token_count: Some(10),
                truncated: true,
            },
        ];

        let truncated_sections_by_record = BTreeMap::from([(
            "packs:longer".to_string(),
            vec![EmbeddingSectionTruncation {
                section: EmbeddingInputSection::Description,
                dropped_chunk_count: 1,
            }],
        )]);

        let telemetry = summarize_document_embedding_tokenization(
            &pending,
            &tokenizations,
            &truncated_sections_by_record,
        );

        assert_eq!(telemetry.document_count, 3);
        assert_eq!(telemetry.truncated_document_count, 2);
        assert_eq!(telemetry.max_token_count, Some(10));
        assert_eq!(telemetry.max_observed_token_count, 20);
        assert_eq!(telemetry.total_observed_token_count, 40);
        assert_eq!(telemetry.total_tokens_over_limit, 12);
        assert_eq!(
            telemetry.section_truncations,
            vec![DocumentEmbeddingSectionTruncation {
                section: "description".to_string(),
                document_count: 1,
                dropped_chunk_count: 1,
            }]
        );
        assert_eq!(
            telemetry.truncated_examples,
            vec![
                DocumentEmbeddingTruncationExample {
                    record_key: "packs:longer".to_string(),
                    token_count: 20,
                    max_token_count: 10,
                    truncated_sections: vec!["description".to_string()],
                },
                DocumentEmbeddingTruncationExample {
                    record_key: "packs:long".to_string(),
                    token_count: 12,
                    max_token_count: 10,
                    truncated_sections: Vec::new(),
                },
            ]
        );
    }

    fn pending_embedding(record_key: &str, input_text: &str) -> PendingDocumentEmbedding {
        pending_embedding_with_hash(
            record_key,
            input_text,
            &hash_document_embedding_input(input_text),
        )
    }

    fn pending_embedding_with_hash(
        record_key: &str,
        input_text: &str,
        input_hash: &str,
    ) -> PendingDocumentEmbedding {
        PendingDocumentEmbedding {
            embedding_unit_key: format!("{record_key}#parent"),
            record_key: record_key.to_string(),
            unit_kind: EmbeddingUnitKind::Parent,
            label: None,
            ordinal: 0,
            input_chunks: vec![EmbeddingInputChunk::line(
                EmbeddingInputSection::Identity,
                input_text,
            )],
            input_text: input_text.to_string(),
            input_hash: input_hash.to_string(),
        }
    }

    fn test_record(key: &str, name: &str, is_default_visible: bool) -> LoadedRecord {
        let key = RecordKey::parse(key).expect("fixture key is valid");
        LoadedRecord {
            id: RecordId::new(key.id().to_string()).expect("fixture id is valid"),
            normalized_name: atlas_ingest_normalize_for_test(name),
            name: name.to_string(),
            record_family: RecordFamily::Rule,
            pack_name: PackName::new(key.pack().to_string()).expect("fixture pack is valid"),
            pack_label: "Fixture Pack".to_string(),
            foundry_document_type: "Item".to_string(),
            foundry_record_type: "action".to_string(),
            level: None,
            rarity: None,
            traits: vec!["healing".to_string()],
            system_category: None,
            system_group: None,
            system_base_item: None,
            system_usage: None,
            system_price_json: None,
            system_actions_value: None,
            system_time_value: None,
            system_duration_value: None,
            price_cp: None,
            activation_time: None,
            duration: None,
            metrics: Vec::new(),
            actor_data: None,
            item_data: None,
            spell_data: None,
            publication_title: None,
            publication_remaster: false,
            description_text: Some("Fixture description.".to_string()),
            blurb_text: None,
            publication_family: atlas_domain::PublicationFamily::Unknown,
            folder_id: None,
            taxonomy_families: vec!["fixture family".to_string()],
            variant_group_key: None,
            variant_base_name: None,
            variant_label: None,
            variant_axes: Vec::new(),
            variant_confidence: None,
            variant_source: "none".to_string(),
            source_path: "fixture.json".to_string(),
            text_status: atlas_domain::TextStatus::Resolved,
            is_default_visible,
            search_text_projection: name.to_string(),
            reference_candidates: Vec::new(),
            raw_json: "{}".to_string(),
            key,
        }
    }

    fn atlas_ingest_normalize_for_test(value: &str) -> String {
        value.to_lowercase().replace(' ', "-")
    }
}
