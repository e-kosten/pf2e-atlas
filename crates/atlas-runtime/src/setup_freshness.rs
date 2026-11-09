use atlas_index::SourceArtifactBuildContext;
use atlas_ingest::{BuildArtifactOptions, CapturedBuildInputs, IngestError};

pub(crate) fn capture_build_inputs(
    options: BuildArtifactOptions,
) -> Result<CapturedBuildInputs, IngestError> {
    tracing::info!(target:"atlas_progress",phase="check_source_freshness","Capturing source inputs");
    let started = std::time::Instant::now();
    let captured = CapturedBuildInputs::capture(options);
    let message = if captured.is_ok() {
        "Captured source inputs"
    } else {
        "Source input capture failed"
    };
    tracing::info!(
        phase = "check_source_freshness",
        elapsed_seconds = started.elapsed().as_secs_f64(),
        success = captured.is_ok(),
        "{message}"
    );
    tracing::info!(target: "atlas_progress", complete = true, "Source capture ended");
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
    fn changed_reference_notice_is_not_fresh_when_foundry_bytes_are_unchanged() {
        let temporary = tempfile::tempdir().unwrap();
        let source = temporary.path().join("source");
        std::fs::create_dir_all(source.join("static/lang")).unwrap();
        std::fs::write(source.join("static/lang/en.json"), "{}").unwrap();
        std::fs::write(
            source.join("module.json"),
            r#"{"id":"pf2e","version":"8.6.1","packs":[]}"#,
        )
        .unwrap();
        let options = || BuildArtifactOptions {
            source_root: source.clone(),
            output_path: temporary.path().join("workspace.sqlite"),
            manifest_path: None,
            locale: "en".into(),
            search_profiles: vec![],
            reuse_passage_layouts: true,
            embedding_batch_size: 32,
        };
        let first = CapturedBuildInputs::capture(options()).unwrap();
        std::fs::create_dir_all(source.join("static/licenses")).unwrap();
        std::fs::write(
            source.join("static/licenses/ORCLicense.md"),
            "TX 9-307-067, fixture notice",
        )
        .unwrap();
        let changed = CapturedBuildInputs::capture(options()).unwrap();
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
            source_package: Some("pf2e".into()),
            pack_systems: Default::default(),
            expected_entry_count,
            entry_keys_sha256,
        });
        assert!(source_is_fresh(&first, &published));
        assert!(!source_is_fresh(&changed, &published));
        for old_recipe in ["reference-definitions/v2", "reference-definitions/v3"] {
            published.reference_preparation.as_mut().unwrap().recipe = old_recipe.into();
            assert!(
                !source_is_fresh(&first, &published),
                "changed derived prose must be prepared again even when captured source bytes are unchanged: {old_recipe}"
            );
        }
    }
}
