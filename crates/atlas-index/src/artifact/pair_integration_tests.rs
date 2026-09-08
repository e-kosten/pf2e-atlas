use std::path::{Path, PathBuf};

use sha2::{Digest, Sha256};

use crate::SqliteIndexReader;

#[test]
fn first_generation_materialization_opens_through_bound_public_reader() {
    let root = unique_root("linux-first-generation-open");
    std::fs::create_dir_all(&root).unwrap();
    let artifact = root.join("index.sqlite");
    create_valid_generation(&artifact, "Linux First Open");
    let digest = sha256(&artifact);
    write_manifest(&root.join("manifest.json"), &digest);
    let generation = super::pair::generation_path(&artifact, &digest);
    assert!(!generation.exists());

    let reader = SqliteIndexReader::open_read_only(&artifact)
        .expect("the first installed generation must open through the public reader");
    assert_eq!(reader.verified_artifact_sha256(), Some(digest.as_str()));
    assert_eq!(reader.check().unwrap().status, crate::ValidationStatus::Ok);
    assert_eq!(reader.inspect().unwrap().records.total_records, 3);
    let records = reader.load_records().unwrap();
    assert_eq!(records.len(), 3);
    assert_eq!(records[0].identity.name, "Linux First Open 1");
    assert!(generation.exists());

    drop(reader);
    std::fs::remove_dir_all(root).unwrap();
}

fn create_valid_generation(path: &Path, name: &str) {
    crate::tests::create_valid_artifact_database(&path.to_path_buf()).unwrap();
    let connection = rusqlite::Connection::open(path).unwrap();
    for index in 1..=3 {
        let record_key = format!("actions:testAction{index}");
        let record_name = format!("{name} {index}");
        connection
            .execute(
                "UPDATE records SET name = ?1, normalized_name = ?2 WHERE record_key = ?3",
                (&record_name, record_name.to_lowercase(), record_key),
            )
            .unwrap();
    }
}

fn write_manifest(path: &Path, digest: &str) {
    std::fs::write(
        path,
        format!(
            r#"{{"manifest_version":"{}","artifact_contract_version":"{}","schema_version":"{}","build":{{"artifact_sha256":"{}"}}}}"#,
            crate::ARTIFACT_MANIFEST_VERSION,
            crate::ARTIFACT_CONTRACT_VERSION,
            crate::ARTIFACT_SCHEMA_VERSION,
            digest,
        ),
    )
    .unwrap();
}

fn sha256(path: &Path) -> String {
    let bytes = std::fs::read(path).unwrap();
    format!("{:x}", Sha256::digest(bytes))
}

fn unique_root(name: &str) -> PathBuf {
    std::env::temp_dir().join(format!(
        "atlas-pair-integration-{name}-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ))
}
