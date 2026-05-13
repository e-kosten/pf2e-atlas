use std::path::Path;

use atlas_domain::PackName;

use crate::references::{build_record_reference_index, resolve_reference_edges};
use crate::{
    DERIVED_AFFLICTION_INSTANCES_PACK_LABEL, DERIVED_AFFLICTION_INSTANCES_PACK_NAME,
    DERIVED_AFFLICTIONS_PACK_LABEL, DERIVED_AFFLICTIONS_PACK_NAME, IngestError, LoadedPack,
    SourceLoad, aliases, embeddings, generated_afflictions, source, variant_taxonomy, variants,
};

pub(crate) fn load_foundry_source(
    source_root: impl AsRef<Path>,
    manifest_path: Option<&Path>,
) -> Result<SourceLoad, IngestError> {
    let source_root = source_root.as_ref();
    let mut source = source::load_foundry_source_records(source_root, manifest_path)?;

    let reference_index = build_record_reference_index(&source.records);
    let generated_afflictions =
        generated_afflictions::build_generated_afflictions(&source.records, &reference_index);
    let generated_references = generated_afflictions.references.clone();
    if !generated_afflictions.records.is_empty() {
        let canonical_count = generated_afflictions
            .records
            .iter()
            .filter(|record| record.is_default_visible)
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
    }

    let reference_index = build_record_reference_index(&source.records);
    variant_taxonomy::assign_taxonomy_families(
        &mut source.records,
        &source.packs,
        &reference_index,
        &mut source.diagnostics,
    );
    variants::assign_variant_groups(
        &mut source.records,
        &reference_index,
        &mut source.diagnostics,
    );
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
    source.aliases =
        aliases::resolve_record_aliases(&source.records, &reference_index, source_root);
    source.remaster_links =
        aliases::resolve_remaster_links(&source.records, &reference_index, source_root);
    source.pending_document_embeddings = embeddings::build_pending_document_embeddings(
        &source.records,
        &source.aliases,
        &source.remaster_links,
    );

    Ok(source)
}
