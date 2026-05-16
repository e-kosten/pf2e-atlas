use std::collections::{BTreeMap, BTreeSet};

use atlas_embedding::{
    DocumentEmbeddingSource, PendingDocumentEmbedding, build_document_embedding_units,
};
use atlas_record::build_record_presentation_document;

use crate::records::{LoadedSourceRecord, RecordAlias, RemasterLink};

pub(crate) mod generation;

pub(crate) fn build_pending_document_embeddings(
    records: &[LoadedSourceRecord],
    aliases: &[RecordAlias],
    remaster_links: &[RemasterLink],
) -> Vec<PendingDocumentEmbedding> {
    let aliases_by_key = aliases_by_record_key(aliases);
    let hidden_record_keys = remaster_links
        .iter()
        .map(|link| link.legacy_record_key.to_string())
        .collect::<BTreeSet<_>>();

    let sources = records
        .iter()
        .filter_map(|loaded| {
            let record = &loaded.record;
            let record_key = record.key.to_string();
            if !record.is_default_visible || hidden_record_keys.contains(&record_key) {
                return None;
            }
            Some(DocumentEmbeddingSource {
                record_key,
                record_name: record.name.clone(),
                document: build_record_presentation_document(record),
                aliases: aliases_by_key
                    .get(&record.key.to_string())
                    .cloned()
                    .unwrap_or_default(),
                source_description_markup: loaded.facts.source_description_markup.clone(),
            })
        })
        .collect::<Vec<_>>();

    build_document_embedding_units(&sources)
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
