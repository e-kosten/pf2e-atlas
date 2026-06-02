use atlas_index::{IndexBuildInput, IndexBuildPack};

use crate::source::SourceLoad;

pub(crate) fn index_build_input(source: SourceLoad) -> IndexBuildInput {
    IndexBuildInput {
        source_signature: source.source_signature,
        source_record_count: source.source_record_count,
        packs: source
            .packs
            .into_iter()
            .map(|pack| IndexBuildPack {
                name: pack.name,
                label: pack.label,
                document_type: pack.document_type,
                declared_path: pack.declared_path,
                resolved_path: pack.resolved_path,
                record_count: pack.record_count,
            })
            .collect(),
        records: source
            .records
            .into_iter()
            .map(|loaded| loaded.record)
            .collect(),
        references: source.references,
        aliases: source.aliases,
        remaster_links: source.remaster_links,
        pending_document_embeddings: source.pending_document_embeddings,
        document_embeddings: source.document_embeddings,
    }
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use atlas_domain::{
        PackName, PublicationFamily, RecordFamily, RecordId, RecordKey, RemasterLinkSource,
    };
    use atlas_embedding::{
        DocumentEmbeddingTokenizationTelemetry, EmbeddingUnitKind, GeneratedDocumentEmbedding,
        PendingDocumentEmbedding,
    };
    use atlas_record::{
        AliasSource, ContentSourceKind, ContentVisibility, NormalizedRecord, RecordAlias,
        ReferenceEdge, RemasterLink,
    };

    use super::index_build_input;
    use crate::diagnostics::IngestDiagnostics;
    use crate::records::{LoadedSourceRecord, SourceConstructionFacts};
    use crate::source::{LoadedPack, SourceLoad};

    #[test]
    fn maps_source_load_to_index_build_input_without_changing_boundaries()
    -> Result<(), Box<dyn std::error::Error>> {
        let source_record = record("source-pack", "source-record", RecordFamily::Rule);
        let generated_record = record("generated-pack", "generated-record", RecordFamily::Rule);
        let source_key = source_record.key.clone();
        let generated_key = generated_record.key.clone();
        let source = SourceLoad {
            manifest_path: PathBuf::from("manifest.json"),
            source_signature: "foundry-pf2e:sha256:test".to_string(),
            source_record_count: 1,
            packs: vec![LoadedPack {
                name: PackName::new("source-pack").expect("pack name parses"),
                label: "Source Pack".to_string(),
                document_type: "Item".to_string(),
                declared_path: "packs/source-pack".to_string(),
                resolved_path: PathBuf::from("/tmp/source-pack"),
                record_count: 1,
            }],
            records: vec![
                LoadedSourceRecord::new(source_record, SourceConstructionFacts::empty()),
                LoadedSourceRecord::new(generated_record, SourceConstructionFacts::empty()),
            ],
            references: vec![ReferenceEdge {
                from_record_key: source_key.clone(),
                to_record_key: generated_key.clone(),
                display_text: Some("Generated Record".to_string()),
                reference_text: "@UUID[generated]".to_string(),
                source_kind: ContentSourceKind::Description,
                visibility: ContentVisibility::Public,
            }],
            aliases: vec![RecordAlias {
                canonical_record_key: source_key.clone(),
                alias_text: "Source Alias".to_string(),
                normalized_alias: "source alias".to_string(),
                source: AliasSource::Migration,
                source_ref: "migration".to_string(),
            }],
            remaster_links: vec![RemasterLink {
                remaster_record_key: source_key.clone(),
                legacy_record_key: generated_key.clone(),
                source: RemasterLinkSource::Migration,
                source_ref: "migration".to_string(),
            }],
            pending_document_embeddings: vec![PendingDocumentEmbedding::prepared(
                "source-pack.source-record#parent".to_string(),
                source_key.to_string(),
                EmbeddingUnitKind::Parent,
                None,
                0,
                "source record".to_string(),
                "pending-hash".to_string(),
            )],
            document_embeddings: vec![GeneratedDocumentEmbedding {
                embedding_unit_key: "source-pack.source-record#parent".to_string(),
                record_key: source_key.to_string(),
                unit_kind: EmbeddingUnitKind::Parent,
                label: None,
                ordinal: 0,
                input_hash: "generated-hash".to_string(),
                dimensions: 3,
                vector: vec![0.1, 0.2, 0.3],
            }],
            document_embedding_tokenization: DocumentEmbeddingTokenizationTelemetry::default(),
            diagnostics: IngestDiagnostics::default(),
            skipped_records: Vec::new(),
            warnings: Vec::new(),
        };

        let input = index_build_input(source);

        assert_eq!(input.source_signature, "foundry-pf2e:sha256:test");
        assert_eq!(input.source_record_count, 1);
        assert_eq!(input.artifact_record_count(), 2);
        assert_eq!(input.generated_record_count()?, 1);

        assert_eq!(input.packs.len(), 1);
        assert_eq!(input.packs[0].name.as_str(), "source-pack");
        assert_eq!(input.packs[0].label, "Source Pack");
        assert_eq!(input.packs[0].document_type, "Item");
        assert_eq!(input.packs[0].declared_path, "packs/source-pack");
        assert_eq!(
            input.packs[0].resolved_path,
            PathBuf::from("/tmp/source-pack")
        );
        assert_eq!(input.packs[0].record_count, 1);

        assert_eq!(input.records.len(), 2);
        assert_eq!(input.records[0].key, source_key);
        assert_eq!(input.records[1].key, generated_key);

        assert_eq!(input.references.len(), 1);
        assert_eq!(input.aliases.len(), 1);
        assert_eq!(input.remaster_links.len(), 1);
        assert_eq!(input.pending_document_embeddings.len(), 1);
        assert_eq!(input.document_embeddings.len(), 1);
        Ok(())
    }

    fn record(pack_name: &str, id: &str, record_family: RecordFamily) -> NormalizedRecord {
        let pack_name = PackName::new(pack_name).expect("pack name parses");
        let id = RecordId::new(id).expect("record id parses");
        NormalizedRecord {
            key: RecordKey::new(pack_name.clone(), id.clone()),
            id,
            name: "Test Record".to_string(),
            normalized_name: "test record".to_string(),
            record_family,
            pack_name,
            pack_label: "Test Pack".to_string(),
            foundry_document_type: "Item".to_string(),
            foundry_record_type: "action".to_string(),
            level: None,
            rarity: None,
            traits: Vec::new(),
            prerequisites: Vec::new(),
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
            description: None,
            blurb: None,
            supplemental_content: Vec::new(),
            publication_family: PublicationFamily::Unknown,
            folder_id: None,
            taxonomy_families: Vec::new(),
            variant_group_key: None,
            variant_base_name: None,
            variant_label: None,
            variant_axes: Vec::new(),
            variant_confidence: None,
            variant_source: "none".to_string(),
            source_path: "test.json".to_string(),
            is_default_visible: true,
            raw_json: "{}".to_string(),
        }
    }
}
