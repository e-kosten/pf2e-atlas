use atlas_index::SourceArtifactBuildContext;
use atlas_ingest::{BuildArtifactOptions, CapturedBuildInputs, IngestError};

pub(crate) fn capture_build_inputs(
    options: BuildArtifactOptions,
) -> Result<CapturedBuildInputs, IngestError> {
    let started = std::time::Instant::now();
    let captured = CapturedBuildInputs::capture(options);
    tracing::info!(target:"atlas_progress",phase="check_source_freshness",
        elapsed_seconds=started.elapsed().as_secs_f64(),success=captured.is_ok(),"Captured source inputs");
    captured
}
pub(crate) fn source_is_fresh(
    inputs: &CapturedBuildInputs,
    context: &SourceArtifactBuildContext,
) -> bool {
    let fingerprint = inputs.fingerprint();
    fingerprint.value == context.source_fingerprint
        && fingerprint.file_count == context.source_file_count
        && context
            .reference_preparation
            .as_ref()
            .is_some_and(|preparation| {
                preparation.recipe == atlas_ingest::REFERENCE_PREPARATION_RECIPE
                    && preparation.input_hash == inputs.reference_inputs_identity()
                    && preparation.coverage == atlas_index::ReferenceCoverage::Complete
            })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn changed_reference_selection_is_not_fresh_when_foundry_bytes_are_unchanged() {
        let temporary = tempfile::tempdir().unwrap();
        let source = temporary.path().join("source");
        std::fs::create_dir_all(source.join("static/lang")).unwrap();
        std::fs::write(source.join("static/lang/en.json"), "{}").unwrap();
        std::fs::write(source.join("module.json"), r#"{"id":"custom","packs":[]}"#).unwrap();
        let options = |revision: &str| BuildArtifactOptions {
            source_root: source.clone(),
            output_path: temporary.path().join("workspace.sqlite"),
            manifest_path: None,
            locale: "en".into(),
            search_profiles: vec![],
            reuse_embeddings: true,
            reuse_passage_layouts: true,
            embedding_batch_size: 32,
            reference_supplement: atlas_ingest::ReferenceSupplementSelection::Absent {
                revision: revision.into(),
            },
        };
        let first = CapturedBuildInputs::capture(options("disabled/v1")).unwrap();
        let changed = CapturedBuildInputs::capture(options("disabled/v2")).unwrap();
        assert_eq!(first.fingerprint().value, changed.fingerprint().value);
        assert_eq!(
            first.fingerprint().file_count,
            changed.fingerprint().file_count
        );
        let mut published = atlas_search::test_support::context(false);
        published.source_fingerprint = first.fingerprint().value.clone();
        published.source_file_count = first.fingerprint().file_count;
        let (expected_entry_count, entry_keys_sha256) =
            atlas_index::reference_inventory_identity(std::iter::empty::<
                atlas_domain::DefinitionKey,
            >())
            .unwrap();
        published.reference_preparation = Some(atlas_index::ReferencePreparation {
            recipe: atlas_ingest::REFERENCE_PREPARATION_RECIPE.into(),
            input_hash: first.reference_inputs_identity().into(),
            coverage: atlas_index::ReferenceCoverage::Complete,
            source_package: Some("custom".into()),
            pack_systems: Default::default(),
            expected_entry_count,
            entry_keys_sha256,
        });
        assert!(source_is_fresh(&first, &published));
        assert!(!source_is_fresh(&changed, &published));
        published.reference_preparation.as_mut().unwrap().recipe =
            "reference-definitions/v2".into();
        assert!(
            !source_is_fresh(&first, &published),
            "changed derived prose must be prepared again even when captured source bytes are unchanged"
        );
    }
}
