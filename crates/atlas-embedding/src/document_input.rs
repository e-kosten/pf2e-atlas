use sha2::{Digest, Sha256};

pub fn hash_document_embedding_input(input: &str) -> String {
    let digest = Sha256::digest(input.as_bytes());
    let mut encoded = String::with_capacity(digest.len() * 2);
    for byte in digest {
        encoded.push_str(&format!("{byte:02x}"));
    }
    encoded
}
