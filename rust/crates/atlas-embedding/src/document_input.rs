use std::fmt::Write;

use sha2::{Digest, Sha256};

use crate::text::normalize_embedding_text;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DocumentEmbeddingInputParts<'a> {
    pub name: &'a str,
    pub traits: &'a [String],
    pub taxonomy_families: &'a [String],
    pub description_text: Option<&'a str>,
    pub aliases: &'a [String],
}

pub fn build_document_embedding_input(parts: DocumentEmbeddingInputParts<'_>) -> String {
    let mut chunks = Vec::new();
    let mut seen = Vec::new();

    append_unique_text_chunk(&mut chunks, &mut seen, parts.name);
    for trait_value in parts.traits {
        append_unique_text_chunk(&mut chunks, &mut seen, trait_value);
    }
    for family in parts.taxonomy_families {
        append_unique_text_chunk(&mut chunks, &mut seen, family);
    }
    if let Some(description_text) = parts.description_text {
        append_unique_text_chunk(&mut chunks, &mut seen, description_text);
    }
    for alias in parts.aliases {
        append_unique_text_chunk(&mut chunks, &mut seen, alias);
    }

    chunks.join("\n")
}

pub fn hash_document_embedding_input(input: &str) -> String {
    let digest = Sha256::digest(input.as_bytes());
    let mut encoded = String::with_capacity(digest.len() * 2);
    for byte in digest {
        write!(&mut encoded, "{byte:02x}").expect("writing to a String cannot fail");
    }
    encoded
}

fn append_unique_text_chunk(chunks: &mut Vec<String>, seen: &mut Vec<String>, value: &str) {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        return;
    }
    let normalized = normalize_embedding_text(trimmed);
    if normalized.is_empty() || seen.contains(&normalized) {
        return;
    }
    seen.push(normalized);
    chunks.push(trimmed.to_string());
}
