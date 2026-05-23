use std::collections::BTreeMap;
use std::path::Path;

use atlas_embedding::{EmbeddingRuntimeConfig, ReusableDocumentEmbedding};
use atlas_index::{DocumentEmbeddingCacheReader, LadybugIndexReader, SqliteIndexReader};
use tracing::info;

use crate::source::model::BuildArtifactOptions;

pub(crate) struct ReusableDocumentEmbeddingCache {
    pub(crate) backend: &'static str,
    pub(crate) path: String,
    pub(crate) embeddings: BTreeMap<String, ReusableDocumentEmbedding>,
}

pub(crate) fn load_reusable_document_embeddings(
    options: &BuildArtifactOptions,
    config: &EmbeddingRuntimeConfig,
) -> Result<ReusableDocumentEmbeddingCache, String> {
    let mut errors = Vec::new();
    for candidate in embedding_cache_candidates(options) {
        if !candidate.path.exists() {
            continue;
        }
        match candidate.load(config) {
            Ok(reusable_embeddings) => {
                info!(
                    backend = candidate.backend,
                    path = %candidate.path.display(),
                    reusable_document_embeddings = reusable_embeddings.len(),
                    "loaded reusable document embeddings"
                );
                return Ok(ReusableDocumentEmbeddingCache {
                    backend: candidate.backend,
                    path: candidate.path.display().to_string(),
                    embeddings: reusable_embeddings,
                });
            }
            Err(error) => {
                info!(
                    backend = candidate.backend,
                    path = %candidate.path.display(),
                    reason = %error,
                    "document embedding reuse unavailable from candidate"
                );
                errors.push(format!(
                    "{} {}: {error}",
                    candidate.backend,
                    candidate.path.display()
                ));
            }
        }
    }
    if errors.is_empty() {
        Err("no existing embedding cache artifacts found".to_string())
    } else {
        Err(errors.join("; "))
    }
}

struct EmbeddingCacheCandidate<'a> {
    backend: &'static str,
    path: &'a Path,
}

impl EmbeddingCacheCandidate<'_> {
    fn load(
        &self,
        config: &EmbeddingRuntimeConfig,
    ) -> Result<BTreeMap<String, ReusableDocumentEmbedding>, String> {
        match self.backend {
            "sqlite" => load_sqlite_embeddings(self.path, config),
            "ladybug" => load_ladybug_embeddings(self.path, config),
            backend => Err(format!("unsupported embedding cache backend `{backend}`")),
        }
    }
}

fn embedding_cache_candidates(options: &BuildArtifactOptions) -> Vec<EmbeddingCacheCandidate<'_>> {
    let mut candidates = vec![EmbeddingCacheCandidate {
        backend: "sqlite",
        path: &options.output_path,
    }];
    if let Some(path) = &options.ladybug_output_path {
        candidates.push(EmbeddingCacheCandidate {
            backend: "ladybug",
            path,
        });
    }
    candidates
}

fn load_sqlite_embeddings(
    path: &Path,
    config: &EmbeddingRuntimeConfig,
) -> Result<BTreeMap<String, ReusableDocumentEmbedding>, String> {
    let index = SqliteIndexReader::open_read_only(path).map_err(|error| error.to_string())?;
    index
        .load_reusable_document_embeddings(config.model_spec())
        .map_err(|error| error.to_string())
}

fn load_ladybug_embeddings(
    path: &Path,
    config: &EmbeddingRuntimeConfig,
) -> Result<BTreeMap<String, ReusableDocumentEmbedding>, String> {
    let index = LadybugIndexReader::open(path).map_err(|error| error.to_string())?;
    index
        .load_reusable_document_embeddings(config.model_spec())
        .map_err(|error| error.to_string())
}
