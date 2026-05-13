pub const ARTIFACT_METADATA_TABLE: &str = "artifact_metadata";
pub const LEGACY_METADATA_TABLE: &str = "metadata";
pub const ARTIFACT_CONTRACT_VERSION: &str = "pf2e-atlas-artifact/v1";
pub const ARTIFACT_SCHEMA_VERSION: &str = "1";

pub mod artifact_metadata_keys {
    pub const ADJACENT_MANIFEST_PATH: &str = "adjacent_manifest_path";
    pub const ARTIFACT_CONTRACT_VERSION: &str = "artifact_contract_version";
    pub const ARTIFACT_RECORD_COUNT: &str = "artifact_record_count";
    pub const CONTENT_HASH_ALGORITHM: &str = "content_hash_algorithm";
    pub const EMBEDDING_DIMENSIONS: &str = "embedding_dimensions";
    pub const EMBEDDING_DISTANCE_METRIC: &str = "embedding_distance_metric";
    pub const EMBEDDING_DOCUMENT_PREFIX: &str = "embedding_document_prefix";
    pub const EMBEDDING_DTYPE: &str = "embedding_dtype";
    pub const EMBEDDING_MODEL_ID: &str = "embedding_model_id";
    pub const EMBEDDING_MODEL_REVISION: &str = "embedding_model_revision";
    pub const EMBEDDING_NORMALIZATION: &str = "embedding_normalization";
    pub const EMBEDDING_POOLING: &str = "embedding_pooling";
    pub const EMBEDDING_PROVIDER_FAMILY: &str = "embedding_provider_family";
    pub const EMBEDDING_QUERY_PREFIX: &str = "embedding_query_prefix";
    pub const EMBEDDING_TOKENIZER_ID: &str = "embedding_tokenizer_id";
    pub const FTS_TOKENIZER: &str = "fts_tokenizer";
    pub const SCHEMA_VERSION: &str = "schema_version";
    pub const GENERATED_RECORD_COUNT: &str = "generated_record_count";
    pub const SOURCE_KIND: &str = "source_kind";
    pub const SOURCE_RECORD_COUNT: &str = "source_record_count";
    pub const SOURCE_SIGNATURE: &str = "source_signature";
}

pub const REQUIRED_ARTIFACT_METADATA_KEYS: &[&str] = &[
    artifact_metadata_keys::ARTIFACT_CONTRACT_VERSION,
    artifact_metadata_keys::SCHEMA_VERSION,
    artifact_metadata_keys::SOURCE_KIND,
    artifact_metadata_keys::SOURCE_SIGNATURE,
    artifact_metadata_keys::SOURCE_RECORD_COUNT,
    artifact_metadata_keys::ARTIFACT_RECORD_COUNT,
    artifact_metadata_keys::GENERATED_RECORD_COUNT,
    artifact_metadata_keys::CONTENT_HASH_ALGORITHM,
    artifact_metadata_keys::EMBEDDING_PROVIDER_FAMILY,
    artifact_metadata_keys::EMBEDDING_MODEL_ID,
    artifact_metadata_keys::EMBEDDING_MODEL_REVISION,
    artifact_metadata_keys::EMBEDDING_TOKENIZER_ID,
    artifact_metadata_keys::EMBEDDING_POOLING,
    artifact_metadata_keys::EMBEDDING_NORMALIZATION,
    artifact_metadata_keys::EMBEDDING_DIMENSIONS,
    artifact_metadata_keys::EMBEDDING_DTYPE,
    artifact_metadata_keys::EMBEDDING_DISTANCE_METRIC,
    artifact_metadata_keys::EMBEDDING_DOCUMENT_PREFIX,
    artifact_metadata_keys::EMBEDDING_QUERY_PREFIX,
    artifact_metadata_keys::FTS_TOKENIZER,
    artifact_metadata_keys::ADJACENT_MANIFEST_PATH,
];

pub const EXPECTED_SOURCE_KIND: &str = "foundry-pf2e";
pub const EXPECTED_CONTENT_HASH_ALGORITHM: &str = "sha256";
pub const EXPECTED_EMBEDDING_PROVIDER_FAMILY: &str = "transformers-js-minilm";
pub const EXPECTED_EMBEDDING_MODEL_ID: &str = "Xenova/all-MiniLM-L12-v2";
pub const EXPECTED_EMBEDDING_MODEL_REVISION: &str = "main";
pub const EXPECTED_EMBEDDING_TOKENIZER_ID: &str = "Xenova/all-MiniLM-L12-v2";
pub const EXPECTED_EMBEDDING_POOLING: &str = "mean";
pub const EXPECTED_EMBEDDING_NORMALIZATION: &str = "l2";
pub const EXPECTED_EMBEDDING_DIMENSIONS: &str = "384";
pub const EXPECTED_EMBEDDING_DTYPE: &str = "f32";
pub const EXPECTED_EMBEDDING_DISTANCE_METRIC: &str = "cosine";
pub const EXPECTED_EMBEDDING_DOCUMENT_PREFIX: &str = "";
pub const EXPECTED_EMBEDDING_QUERY_PREFIX: &str = "";
pub const EXPECTED_FTS_TOKENIZER: &str = "unicode61 remove_diacritics 2";
