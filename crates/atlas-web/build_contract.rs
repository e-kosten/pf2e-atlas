//! Browser wire compatibility, independent of implementation/build provenance.
use sha2::{Digest, Sha256};
use std::{
    error::Error,
    fs,
    path::{Path, PathBuf},
};

pub const METADATA: &str = "crates/atlas-web/api-contract.json";

pub const SCHEMA: &str = "crates/atlas-web/openapi.json";

pub struct BrowserContract {
    pub identifier: String,
    pub watched_paths: Vec<PathBuf>,
}

pub fn load(root: &Path) -> Result<BrowserContract, Box<dyn Error>> {
    let metadata_path = root.join(METADATA);
    let metadata: serde_json::Value = serde_json::from_slice(&fs::read(&metadata_path)?)?;
    let revision = metadata["protocol_revision"]
        .as_f64()
        .filter(|revision| {
            revision.fract() == 0.0 && *revision >= 1.0 && *revision <= f64::from(u32::MAX)
        })
        .ok_or("API protocol_revision must be a positive 32-bit integer")?
        as u32;
    let schema_path = root.join(SCHEMA);
    let schema = fs::read_to_string(&schema_path)?.replace("\r\n", "\n");
    if schema.trim().is_empty() {
        return Err("OpenAPI document is empty".into());
    }
    let mut hash = Sha256::new();
    hash.update(b"atlas-web-openapi\0");
    hash.update(revision.to_string().as_bytes());
    hash.update([0]);
    hash.update(schema.as_bytes());
    Ok(BrowserContract {
        identifier: format!("atlas-web-v3-{:x}", hash.finalize()),
        watched_paths: vec![metadata_path, schema_path],
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn fixture() -> tempfile::TempDir {
        let root = tempfile::tempdir().unwrap();
        fs::create_dir_all(root.path().join("crates/atlas-web")).unwrap();
        fs::write(root.path().join(METADATA), r#"{"protocol_revision":7}"#).unwrap();
        fs::write(
            root.path().join(SCHEMA),
            "{\"openapi\":\"3.1.0\",\"paths\":{}}\r\n",
        )
        .unwrap();
        root
    }

    #[test]
    fn fingerprint_matches_shared_typescript_vector_and_normalizes_line_endings() {
        let root = fixture();
        let before = load(root.path()).unwrap();
        assert_eq!(
            before.identifier,
            "atlas-web-v3-08d4cc97bc6f443b0b8ffd103ae35b5684d30514cd77246159e8ba45eaa86854"
        );
        fs::write(
            root.path().join(SCHEMA),
            "{\"openapi\":\"3.1.0\",\"paths\":{}}\n",
        )
        .unwrap();
        assert_eq!(load(root.path()).unwrap().identifier, before.identifier);
    }

    #[test]
    fn implementation_and_test_changes_do_not_change_wire_compatibility() {
        let root = fixture();
        let before = load(root.path()).unwrap().identifier;
        for file in [
            "crates/atlas-app-service/src/rules.rs",
            "crates/atlas-domain/src/similarity.rs",
            "crates/atlas-web/src/tests.rs",
        ] {
            let path = root.path().join(file);
            fs::create_dir_all(path.parent().unwrap()).unwrap();
            fs::write(
                path,
                "// Internal implementation, test or comment change.\n",
            )
            .unwrap();
        }
        assert_eq!(load(root.path()).unwrap().identifier, before);
    }

    #[test]
    fn changed_schema_invalidates_the_fingerprint() {
        let root = fixture();
        let before = load(root.path()).unwrap().identifier;
        fs::write(
            root.path().join(SCHEMA),
            "{\"openapi\":\"3.1.0\",\"paths\":{\"/api/test\":{}}}\n",
        )
        .unwrap();
        assert_ne!(load(root.path()).unwrap().identifier, before);
    }

    #[test]
    fn protocol_revision_covers_semantics_absent_from_openapi() {
        let root = fixture();
        let before = load(root.path()).unwrap().identifier;
        fs::write(root.path().join(METADATA), r#"{"protocol_revision":8}"#).unwrap();
        assert_ne!(load(root.path()).unwrap().identifier, before);
    }

    #[test]
    fn metadata_formatting_and_equivalent_numeric_json_preserve_the_identifier() {
        let root = fixture();
        let before = load(root.path()).unwrap().identifier;
        fs::write(
            root.path().join(METADATA),
            "{ \"protocol_revision\": 7.0 }\n",
        )
        .unwrap();
        assert_eq!(load(root.path()).unwrap().identifier, before);
    }

    #[test]
    fn invalid_revision_or_empty_contracts_fail_instead_of_sharing_an_identifier() {
        let root = fixture();
        for revision in ["0", "-1", "1.5", "4294967296", "null", "\"1\""] {
            fs::write(
                root.path().join(METADATA),
                format!(r#"{{"protocol_revision":{revision}}}"#),
            )
            .unwrap();
            assert!(load(root.path()).is_err(), "revision {revision}");
        }
        fs::write(root.path().join(METADATA), r#"{"protocol_revision":7}"#).unwrap();
        fs::write(root.path().join(SCHEMA), "\n").unwrap();
        assert!(load(root.path()).is_err());
    }
}
