use sha2::{Digest, Sha256};

impl crate::EmbeddingModelSpec {
    /// Binds raw document input to every pinned asset and execution policy.
    /// `hash_document_embedding_input` separately identifies raw source text.
    pub fn document_reuse_key(self, input: &str) -> String {
        self.document_reuse_key_with_segmentation_digest(
            input,
            crate::BASELINE_SEGMENTATION_TOKENIZER_SHA256,
        )
    }
    fn document_reuse_key_with_segmentation_digest(
        self,
        input: &str,
        segmentation_sha256: &str,
    ) -> String {
        let spec = self;
        let mut envelope = format!(
            "{}\0{}\0{}\0{}\0{}\0{}\0{}\0{}\0{}\0{}\0{}\0{}\0{}\0{}\0{}\0{}\0{}\0{}",
            spec.provider_family,
            spec.model_id,
            spec.model_revision,
            spec.model_sha256,
            spec.tokenizer_id,
            spec.tokenizer_sha256,
            spec.pooling.as_str(),
            spec.output_name,
            spec.normalization.as_str(),
            spec.dimensions,
            spec.dtype.as_str(),
            spec.distance_metric.as_str(),
            spec.max_input_tokens
                .map_or_else(|| "none".to_owned(), |n| n.to_string()),
            spec.document_prefix,
            spec.query_prefix,
            crate::EMBEDDING_UNIT_POLICY_VERSION,
            crate::EMBEDDING_INFERENCE_POLICY_VERSION,
            crate::BASELINE_INPUT_RECIPE,
        );
        envelope.push_str("\0segmentation-tokenizer-sha256\0");
        envelope.push_str(segmentation_sha256);
        for &(path, sha256) in spec.required_assets {
            envelope.push_str(&format!("\0{path}\0{sha256}"));
        }
        envelope.push('\0');
        envelope.push_str(&spec.document_input(input));
        hash_document_embedding_input(&envelope)
    }
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

#[cfg(test)]
mod tests {
    use crate::{
        BASELINE_SEGMENTATION_TOKENIZER_SHA256, SEARCH_PROFILES, baseline_tokenizer_cache_file,
        hash_document_embedding_input,
    };

    #[test]
    fn every_profile_reuse_key_binds_the_complete_shared_segmentation_digest() {
        let digest = BASELINE_SEGMENTATION_TOKENIZER_SHA256;
        assert_eq!(
            digest,
            "d241a60d5e8f04cc1b2b3e9ef7a4921b27bf526d9f6050ab90f9267a1f9e5c66"
        );
        assert_eq!(baseline_tokenizer_cache_file("unused").sha256, digest);
        // Preserve the recipe's abbreviated checksum to ensure it cannot mask a change.
        let changed = format!("{}{}", &digest[..8], "0".repeat(56));
        let raw = "Forest guardian\nA watchful guardian protects the forest.";
        for profile in SEARCH_PROFILES {
            let spec = profile.model_spec();
            let current = spec.document_reuse_key(raw);
            assert_eq!(
                current,
                spec.document_reuse_key_with_segmentation_digest(raw, digest)
            );
            assert_ne!(
                current,
                spec.document_reuse_key_with_segmentation_digest(raw, &changed),
                "{}",
                profile.id
            );
            assert_ne!(current, hash_document_embedding_input(raw));
        }
    }
}
