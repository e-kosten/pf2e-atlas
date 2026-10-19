//! Product-build compatibility, separate from permissive developer source diagnosis.
use crate::IngestError;
use atlas_foundry_model::generated::{REFERENCE_SOURCE_COMMIT, REFERENCE_SOURCE_VERSION};
use std::path::Path;

/// Validate a PF2e system manifest before preparation. Module versions are not
/// PF2e versions: custom modules continue through ordinary checked admission.
pub fn validate_source_compatibility(
    root: &Path,
    manifest: Option<&Path>,
) -> Result<(), IngestError> {
    let path = manifest
        .map(Path::to_owned)
        .unwrap_or_else(|| crate::source::discovery::default_manifest_path(root));
    let bytes = std::fs::read(&path).map_err(|e| IngestError::SourceUnavailable(e.to_string()))?;
    validate_manifest(&path, &bytes)
}

pub(crate) fn validate_manifest(path: &Path, bytes: &[u8]) -> Result<(), IngestError> {
    let value: serde_json::Value = serde_json::from_slice(bytes)
        .map_err(|e| IngestError::ManifestParseFailed(e.to_string()))?;
    let system_manifest = value.get("id").and_then(|v| v.as_str()) == Some("pf2e")
        || matches!(
            path.file_name().and_then(|v| v.to_str()),
            Some("system.json" | "system.pf2e.json")
        );
    if system_manifest {
        let version = value
            .get("version")
            .and_then(|v| v.as_str())
            .unwrap_or("unknown");
        if version != REFERENCE_SOURCE_VERSION {
            return Err(IngestError::SourceIncompatible(format!(
                "PF2e source version mismatch: expected {REFERENCE_SOURCE_VERSION} (reference commit {REFERENCE_SOURCE_COMMIT}), detected {version} in {}; use the pinned default cache or supply compatible source with --source; the supplied directory was not changed",
                path.display()
            )));
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn system_version_is_checked_but_module_version_is_not_a_system_version() {
        let temp = tempfile::tempdir().unwrap();
        let manifest = temp.path().join("module.json");
        for (json, compatible) in [
            (
                serde_json::json!({"id":"pf2e","version":REFERENCE_SOURCE_VERSION}),
                true,
            ),
            (serde_json::json!({"id":"pf2e","version":"7.9.1"}), false),
            (serde_json::json!({"id":"pf2e"}), false),
            (
                serde_json::json!({"id":"custom-module","version":"1.0.0"}),
                true,
            ),
        ] {
            std::fs::write(&manifest, json.to_string()).unwrap();
            assert_eq!(
                validate_source_compatibility(temp.path(), None).is_ok(),
                compatible
            );
        }
        let system = temp.path().join("system.pf2e.json");
        std::fs::write(&system, r#"{"version":"7.9.1"}"#).unwrap();
        let error = validate_source_compatibility(temp.path(), None)
            .unwrap_err()
            .to_string();
        assert!(error.contains(REFERENCE_SOURCE_VERSION));
        assert!(error.contains("7.9.1"));
        assert!(validate_source_compatibility(temp.path(), Some(&manifest)).is_ok());
    }
}
