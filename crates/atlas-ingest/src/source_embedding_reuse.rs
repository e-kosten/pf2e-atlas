//! Tokenizer-backed verification of old input/vector associations.
use crate::{IngestError, index_build_input::semantic_units};
use atlas_domain::RecordKey;
use atlas_embedding::{PreparedEmbeddingInput, TextEmbeddingTokenizer};
use atlas_index::{SourceSearchProfileIdentity, SourceSemanticUnitInput, SqliteIndexReader};
use std::collections::{BTreeMap, BTreeSet};

pub(crate) fn verified_reusable_vectors(
    reader: &SqliteIndexReader,
    inputs: &[PreparedEmbeddingInput],
    tokenizer: &TextEmbeddingTokenizer,
    native_tokenizer: &TextEmbeddingTokenizer,
    model: &SourceSearchProfileIdentity,
) -> Result<BTreeMap<String, Vec<f32>>, IngestError> {
    let label = atlas_embedding::search_profile(&model.profile_id)
        .map_err(|error| IngestError::DocumentEmbeddingFailed(error.to_string()))?
        .label;
    let hashes = inputs
        .iter()
        .map(|i| i.input_sha256.clone())
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect::<Vec<_>>();
    let mut checked_roots = BTreeMap::<RecordKey, Vec<SourceSemanticUnitInput>>::new();
    let mut by_hash = BTreeMap::new();
    let mut checked_hashes = 0;
    let total_hashes = hashes.len();
    tracing::info!(target:"atlas_progress", phase=%format!("{}:reuse", model.profile_id), current=0u64,total=total_hashes as u64,
        "{} [{}] reuse: verifying 0/{} unique raw-input hashes", label,model.profile_id,total_hashes);
    for hashes in hashes.chunks(1024) {
        let mut cursor = 0;
        loop {
            let candidates = reader
                .read_reuse_candidates(hashes, model, cursor, 512)
                .map_err(reuse_error)?;
            if candidates.is_empty() {
                break;
            }
            let new_roots = candidates
                .iter()
                .map(|c| c.location.record.clone())
                .filter(|key| !checked_roots.contains_key(key))
                .collect::<BTreeSet<_>>()
                .into_iter()
                .collect::<Vec<_>>();
            for roots in new_roots.chunks(128) {
                for selection in reader
                    .read_search_selections_for_reuse(roots)
                    .map_err(reuse_error)?
                {
                    let mut regenerated = vec![];
                    semantic_units(&selection, tokenizer, &mut regenerated, &mut 0)?;
                    checked_roots.insert(
                        selection.root,
                        regenerated
                            .into_iter()
                            .map(|p| {
                                crate::index_build_input::profile_pending(p, native_tokenizer)
                                    .map(|p| p.unit)
                            })
                            .collect::<Result<Vec<_>, _>>()?,
                    );
                }
            }
            for candidate in candidates {
                cursor = candidate.location.unit_id;
                let expected = checked_roots
                    .get(&candidate.location.record)
                    .and_then(|units| {
                        units.iter().find(|unit| {
                            unit.owners == candidate.location.owners
                                && unit.field == candidate.location.field
                                && Some(&unit.address) == candidate.location.address.as_ref()
                                && unit.chunk_ordinal == candidate.chunk_ordinal
                        })
                    });
                if !expected.is_some_and(|unit| {
                    unit.input_hash == candidate.input_hash
                        && unit.input_token_count == candidate.input_token_count
                }) {
                    return Err(IngestError::DocumentEmbeddingFailed(format!(
                        "reusable vector input association mismatch: {} unit {}",
                        candidate.location.record, candidate.location.unit_id
                    )));
                }
                if let Some(previous) =
                    by_hash.insert(candidate.input_hash, candidate.vector.clone())
                    && previous != candidate.vector
                {
                    return Err(IngestError::DocumentEmbeddingFailed(
                        "same verified input/model has inconsistent vectors".into(),
                    ));
                }
            }
        }
        checked_hashes += hashes.len();
        tracing::info!(target:"atlas_progress", phase=%format!("{}:reuse", model.profile_id), current=checked_hashes as u64,total=total_hashes as u64,
            "{} [{}] reuse: verified {}/{} unique raw-input hashes; {} reusable vectors",label,model.profile_id,checked_hashes,total_hashes,by_hash.len());
    }
    Ok(inputs
        .iter()
        .filter_map(|input| {
            by_hash
                .get(&input.input_sha256)
                .map(|vector| (input.reuse_key.clone(), vector.clone()))
        })
        .collect())
}
fn reuse_error(error: atlas_index::IndexError) -> IngestError {
    IngestError::DocumentEmbeddingFailed(format!("reusable vector validation failed: {error}"))
}
