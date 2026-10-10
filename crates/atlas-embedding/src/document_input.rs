use sha2::{Digest, Sha256};

/// Execution policy and exact input determine reusable inference, independently
/// of the number of source occurrences which use the vector.
pub fn embedding_reuse_key(input: &str) -> String {
    let spec = crate::default_embedding_model_spec();
    let envelope = format!(
        "{}\0{}\0{}\0{}\0{}\0{}\0{}\0{}",
        spec.model_id,
        spec.model_revision,
        spec.model_sha256,
        spec.tokenizer_sha256,
        spec.pooling.as_str(),
        spec.normalization.as_str(),
        crate::EMBEDDING_UNIT_POLICY_VERSION,
        input
    );
    hash_document_embedding_input(&envelope)
}

pub fn hash_document_embedding_input(input: &str) -> String {
    hash_bytes(input.as_bytes())
}

pub(crate) fn hash_bytes(input: &[u8]) -> String {
    let digest = Sha256::digest(input);
    let mut encoded = String::with_capacity(digest.len() * 2);
    const HEX: &[u8; 16] = b"0123456789abcdef";
    for byte in digest {
        encoded.push(HEX[(byte >> 4) as usize] as char);
        encoded.push(HEX[(byte & 0x0f) as usize] as char);
    }
    encoded
}
