use std::path::Path;

use atlas_domain::PackName;
use tracing::info;

use crate::diagnostics::{
    DERIVED_AFFLICTION_INSTANCES_PACK_LABEL, DERIVED_AFFLICTION_INSTANCES_PACK_NAME,
    DERIVED_AFFLICTIONS_PACK_LABEL, DERIVED_AFFLICTIONS_PACK_NAME,
};
use crate::embeddings;
use crate::error::IngestError;
use crate::generated::afflictions;
use crate::records::references::{build_record_reference_index, resolve_reference_edges};
use crate::records::{aliases, taxonomy, variants};
use crate::source::loader::load_foundry_source_records;
use crate::source::{LoadedPack, SourceLoad};

pub(crate) fn load_foundry_source(
    source_root: impl AsRef<Path>,
    manifest_path: Option<&Path>,
) -> Result<SourceLoad, IngestError> {
    let source_root = source_root.as_ref();
    info!(source = %source_root.display(), "loading Foundry source records");
    let mut source = load_foundry_source_records(source_root, manifest_path)?;
    info!(
        packs = source.packs.len(),
        source_records = source.records.len(),
        skipped_records = source.skipped_records.len(),
        "loaded Foundry source records"
    );

    info!("building reference index");
    let reference_index = build_record_reference_index(&source.records);
    info!("generating derived affliction records");
    let generated_afflictions =
        afflictions::build_generated_afflictions(&source.records, &reference_index);
    let generated_references = generated_afflictions.references.clone();
    if !generated_afflictions.records.is_empty() {
        let canonical_count = generated_afflictions
            .records
            .iter()
            .filter(|loaded| loaded.record.is_default_visible)
            .count();
        let instance_count = generated_afflictions.records.len() - canonical_count;
        source.diagnostics.generated_affliction_canonical_records = canonical_count;
        source.diagnostics.generated_affliction_instance_records = instance_count;
        source.diagnostics.generated_affliction_reference_edges =
            generated_afflictions.references.len();
        source.packs.push(LoadedPack {
            name: PackName::new(DERIVED_AFFLICTIONS_PACK_NAME.to_string()).map_err(|error| {
                IngestError::ManifestParseFailed(format!(
                    "invalid derived affliction pack: {error}"
                ))
            })?,
            label: DERIVED_AFFLICTIONS_PACK_LABEL.to_string(),
            document_type: "Item".to_string(),
            declared_path: "derived://afflictions".to_string(),
            resolved_path: source_root.join("derived-afflictions"),
            record_count: canonical_count,
        });
        source.packs.push(LoadedPack {
            name: PackName::new(DERIVED_AFFLICTION_INSTANCES_PACK_NAME.to_string()).map_err(
                |error| {
                    IngestError::ManifestParseFailed(format!(
                        "invalid derived affliction instance pack: {error}"
                    ))
                },
            )?,
            label: DERIVED_AFFLICTION_INSTANCES_PACK_LABEL.to_string(),
            document_type: "Item".to_string(),
            declared_path: "derived://affliction-instances".to_string(),
            resolved_path: source_root.join("derived-affliction-instances"),
            record_count: instance_count,
        });
        source.records.extend(generated_afflictions.records);
        info!(
            canonical = canonical_count,
            instances = instance_count,
            reference_edges = generated_afflictions.references.len(),
            "generated derived affliction records"
        );
    }

    info!(
        records = source.records.len(),
        "assigning taxonomy families"
    );
    let reference_index = build_record_reference_index(&source.records);
    taxonomy::assign_taxonomy_families(
        &mut source.records,
        &source.packs,
        &reference_index,
        &mut source.diagnostics,
    );
    info!(records = source.records.len(), "assigning variant groups");
    variants::assign_variant_groups(
        &mut source.records,
        &reference_index,
        &mut source.diagnostics,
    );
    info!("resolving reference edges");
    source.references = resolve_reference_edges(&source.records, &reference_index);
    source.references.extend(generated_references);
    source.references.sort_by(|left, right| {
        (
            left.from_record_key.to_string(),
            left.to_record_key.to_string(),
            left.reference_text.as_str(),
        )
            .cmp(&(
                right.from_record_key.to_string(),
                right.to_record_key.to_string(),
                right.reference_text.as_str(),
            ))
    });
    source.references.dedup_by(|left, right| {
        left.from_record_key == right.from_record_key
            && left.to_record_key == right.to_record_key
            && left.reference_text == right.reference_text
    });
    info!(
        reference_edges = source.references.len(),
        "resolved reference edges"
    );
    info!("resolving aliases and remaster links");
    source.aliases =
        aliases::resolve_record_aliases(&source.records, &reference_index, source_root);
    source.remaster_links =
        aliases::resolve_remaster_links(&source.records, &reference_index, source_root);
    info!(
        aliases = source.aliases.len(),
        remaster_links = source.remaster_links.len(),
        "resolved aliases and remaster links"
    );
    info!("preparing document embedding inputs");
    source.pending_document_embeddings = embeddings::build_pending_document_embeddings(
        &source.records,
        &source.aliases,
        &source.remaster_links,
    );
    info!(
        pending_document_embeddings = source.pending_document_embeddings.len(),
        "prepared document embedding inputs"
    );

    Ok(source)
}
