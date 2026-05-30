use std::collections::BTreeMap;
use std::path::Path;

use atlas_embedding::{EmbeddingRuntimeConfig, ReusableDocumentEmbedding};
use atlas_index::{AtlasIndex, DocumentEmbeddingCacheReader};

pub(crate) fn load_reusable_document_embeddings(
    path: &Path,
    config: &EmbeddingRuntimeConfig,
) -> Result<BTreeMap<String, ReusableDocumentEmbedding>, String> {
    let index = AtlasIndex::open_read_only(path).map_err(|error| error.to_string())?;
    index
        .load_reusable_document_embeddings(config.model_spec())
        .map_err(|error| error.to_string())
}
