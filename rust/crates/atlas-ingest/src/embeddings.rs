use std::collections::{BTreeMap, BTreeSet};

use atlas_embedding::{
    DocumentEmbeddingInputParts, EmbeddingError, TextEmbedder, build_document_embedding_input,
    hash_document_embedding_input,
};
use tracing::info;

use crate::{LoadedRecord, RecordAlias, RemasterLink};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PendingDocumentEmbedding {
    pub record_key: String,
    pub input_text: String,
    pub input_hash: String,
}

#[derive(Debug, Clone, PartialEq)]
pub struct GeneratedDocumentEmbedding {
    pub record_key: String,
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
        let input_text = build_document_embedding_input(DocumentEmbeddingInputParts {
            name: record.name.as_str(),
            traits: &record.traits,
            taxonomy_families: &record.taxonomy_families,
            description_text: record.description_text.as_deref(),
            aliases: &aliases,
        });
        let input_hash = hash_document_embedding_input(&input_text);
        pending.push(PendingDocumentEmbedding {
            record_key,
            input_text,
            input_hash,
        });
    }

    pending
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
            .and_then(|lookup| lookup.get(&entry.record_key))
            .filter(|reusable| reusable.input_hash == entry.input_hash)
        {
            reused_count += 1;
            generated[index] = Some(GeneratedDocumentEmbedding {
                record_key: entry.record_key.clone(),
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
                record_key: entry.record_key.clone(),
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
        assert!(pending[0].input_text.contains("Visible Record"));
        assert!(pending[0].input_text.contains("healing"));
        assert!(pending[0].input_text.contains("fixture family"));
        assert!(pending[0].input_text.contains("Legacy Visible"));
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
            PendingDocumentEmbedding {
                record_key: "packs:first".to_string(),
                input_text: "first input".to_string(),
                input_hash: "first-hash".to_string(),
            },
            PendingDocumentEmbedding {
                record_key: "packs:second".to_string(),
                input_text: "second input".to_string(),
                input_hash: "second-hash".to_string(),
            },
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
            PendingDocumentEmbedding {
                record_key: "packs:first".to_string(),
                input_text: "first input".to_string(),
                input_hash: "first-hash".to_string(),
            },
            PendingDocumentEmbedding {
                record_key: "packs:second".to_string(),
                input_text: "second input".to_string(),
                input_hash: "second-hash".to_string(),
            },
        ];
        let reusable = BTreeMap::from([(
            "packs:first".to_string(),
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
            PendingDocumentEmbedding {
                record_key: "packs:first".to_string(),
                input_text: "first input".to_string(),
                input_hash: "first-hash".to_string(),
            },
            PendingDocumentEmbedding {
                record_key: "packs:second".to_string(),
                input_text: "second input".to_string(),
                input_hash: "second-hash".to_string(),
            },
            PendingDocumentEmbedding {
                record_key: "packs:third".to_string(),
                input_text: "third input".to_string(),
                input_hash: "third-hash".to_string(),
            },
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
