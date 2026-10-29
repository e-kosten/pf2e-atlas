//! Browser wire compatibility, independent of implementation/build provenance.
use sha2::{Digest, Sha256};
use std::{
    error::Error,
    fs,
    path::{Path, PathBuf},
};

pub const METADATA: &str = "crates/atlas-web/api-contract.json";

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
    let bindings = metadata["bindings"]
        .as_str()
        .filter(|path| !path.is_empty())
        .ok_or("API bindings must name the generated binding directory")?;
    let binding_root = root.join(bindings);
    let mut files = Vec::new();
    collect(root, &binding_root, &mut files)?;
    if files.is_empty() {
        return Err("API binding directory contains no generated TypeScript contracts".into());
    }
    files.sort();
    let mut hash = Sha256::new();
    hash.update(b"atlas-web-protocol\0");
    hash.update(revision.to_string().as_bytes());
    hash.update([0]);
    let mut watched_paths = vec![metadata_path, binding_root];
    for file in files {
        let path = root.join(&file);
        hash.update(file.as_bytes());
        hash.update([0]);
        hash.update(fs::read_to_string(&path)?.replace("\r\n", "\n").as_bytes());
        hash.update([0]);
        watched_paths.push(path);
    }
    Ok(BrowserContract {
        identifier: format!("atlas-web-v2-{:x}", hash.finalize()),
        watched_paths,
    })
}

fn collect(root: &Path, path: &Path, files: &mut Vec<String>) -> std::io::Result<()> {
    if path.is_dir() {
        for entry in fs::read_dir(path)? {
            collect(root, &entry?.path(), files)?;
        }
    } else if path.extension().is_some_and(|ext| ext == "ts") {
        files.push(
            path.strip_prefix(root)
                .map_err(std::io::Error::other)?
                .components()
                .map(|part| part.as_os_str().to_string_lossy())
                .collect::<Vec<_>>()
                .join("/"),
        );
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn fixture() -> tempfile::TempDir {
        let root = tempfile::tempdir().unwrap();
        fs::create_dir_all(root.path().join("crates/atlas-web")).unwrap();
        fs::create_dir(root.path().join("bindings")).unwrap();
        fs::write(
            root.path().join(METADATA),
            r#"{"protocol_revision":7,"bindings":"bindings"}"#,
        )
        .unwrap();
        // Create in reverse path order to verify stable collection order.
        fs::write(
            root.path().join("bindings/B.ts"),
            "export type B = \"ready\" | \"busy\";\n",
        )
        .unwrap();
        fs::write(
            root.path().join("bindings/A.ts"),
            "export type A = { value: string };\r\n",
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
            "atlas-web-v2-16d06f503005aaab173401cb8ced6199baf44f44398b368eb96407cf49dedf01"
        );
        fs::write(
            root.path().join("bindings/A.ts"),
            "export type A = { value: string };\n",
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
    fn changed_added_and_removed_bindings_invalidate_the_fingerprint() {
        let root = fixture();
        let before = load(root.path()).unwrap().identifier;
        let file = root.path().join("bindings/A.ts");
        fs::write(&file, "export type A = { value: bigint };\n").unwrap();
        let changed = load(root.path()).unwrap().identifier;
        assert_ne!(changed, before);
        fs::write(root.path().join("bindings/C.ts"), "export type C = A;\n").unwrap();
        let added = load(root.path()).unwrap().identifier;
        assert_ne!(added, changed);
        fs::remove_file(file).unwrap();
        assert_ne!(load(root.path()).unwrap().identifier, added);
    }

    #[test]
    fn protocol_revision_covers_semantics_absent_from_typescript() {
        let root = fixture();
        let before = load(root.path()).unwrap().identifier;
        fs::write(
            root.path().join(METADATA),
            r#"{"protocol_revision":8,"bindings":"bindings"}"#,
        )
        .unwrap();
        assert_ne!(load(root.path()).unwrap().identifier, before);
    }

    #[test]
    fn metadata_formatting_and_equivalent_numeric_json_preserve_the_identifier() {
        let root = fixture();
        let before = load(root.path()).unwrap().identifier;
        fs::write(
            root.path().join(METADATA),
            "{ \"bindings\": \"bindings\", \"protocol_revision\": 7.0 }\n",
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
                format!(r#"{{"protocol_revision":{revision},"bindings":"bindings"}}"#),
            )
            .unwrap();
            assert!(load(root.path()).is_err(), "revision {revision}");
        }
        fs::write(
            root.path().join(METADATA),
            r#"{"protocol_revision":7,"bindings":"bindings"}"#,
        )
        .unwrap();
        fs::remove_file(root.path().join("bindings/A.ts")).unwrap();
        fs::remove_file(root.path().join("bindings/B.ts")).unwrap();
        assert!(load(root.path()).is_err());
    }
}
