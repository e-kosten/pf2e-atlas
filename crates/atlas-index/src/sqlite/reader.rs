use std::cell::{RefCell, RefMut};
use std::fs::File;
use std::path::{Path, PathBuf};
use std::time::{Instant, UNIX_EPOCH};

use diesel::connection::SimpleConnection;
use diesel::{Connection as DieselConnection, SqliteConnection};
use rusqlite::{Connection, OpenFlags};

use crate::IndexValidationError;
use crate::artifact::metadata::{
    ARTIFACT_CONTRACT_VERSION, ARTIFACT_METADATA_TABLE, ARTIFACT_SCHEMA_VERSION,
    artifact_metadata_keys,
};
use crate::artifact::pair::{
    GenerationLease, PairLock, VerifiedArtifactFile, adjacent_manifest_path,
    open_verified_generation,
};
use crate::read::search::vector::register_sqlite_vec_extension;

#[derive(Debug, Clone, PartialEq, Eq)]
struct VerifiedArtifactGenerationIdentity {
    canonical_artifact_path: PathBuf,
    generation_path: PathBuf,
    file_identity: String,
    bytes: u64,
    trusted_sha256: String,
    modified_unix_nanos: Option<u128>,
}

pub struct SqliteIndexReader {
    path: PathBuf,
    _verified_artifact_sha256: Option<String>,
    verified_generation: Option<VerifiedArtifactGenerationIdentity>,
    reader_acquisition_ms: u128,
    reader_visible_sha_pass_count: u64,
    reader_generation_sha_pass_count: u64,
    compatibility_stamp_check_count: u64,
    diesel_connection: RefCell<SqliteConnection>,
    validation_connection: RefCell<Connection>,
    _artifact_file: File,
    // This field must remain after the SQLite handles and retained file so an
    // obsolete generation is removed only after this reader releases it.
    _generation_lease: Option<GenerationLease>,
}

impl SqliteIndexReader {
    pub fn open_read_only(path: impl AsRef<Path>) -> Result<Self, IndexValidationError> {
        Self::open_bound_read_only_with_hook(path.as_ref(), || {})
    }

    fn open_bound_read_only_with_hook(
        path: &Path,
        after_verification: impl FnOnce(),
    ) -> Result<Self, IndexValidationError> {
        let acquisition_started = Instant::now();
        let path = path.to_path_buf();
        let manifest_path = adjacent_manifest_path(&path);
        let pair_lock = PairLock::shared(&manifest_path)?;
        let verified = VerifiedArtifactFile::open(&path, &manifest_path)?;
        let (generation, generation_lease, materialization) =
            open_verified_generation(&path, &manifest_path, &verified)?;
        after_verification();
        let database_url = immutable_read_only_sqlite_uri(&generation.file, &generation.path)?;
        let (diesel_connection, validation_connection) = open_connections(&database_url)?;
        validate_compatibility_stamps(
            &validation_connection,
            &verified.artifact_contract_version,
            &verified.schema_version,
        )?;
        let verified_generation = verified_generation_identity(
            &path,
            &generation.path,
            &generation.file,
            verified.sha256.clone(),
        )?;
        drop(pair_lock);
        Ok(Self {
            path,
            _verified_artifact_sha256: Some(verified.sha256),
            verified_generation: Some(verified_generation),
            reader_acquisition_ms: acquisition_started.elapsed().as_millis(),
            reader_visible_sha_pass_count: 0,
            reader_generation_sha_pass_count: materialization.verify_sha_pass_count,
            compatibility_stamp_check_count: 1,
            diesel_connection: RefCell::new(diesel_connection),
            validation_connection: RefCell::new(validation_connection),
            _artifact_file: generation.file,
            _generation_lease: Some(generation_lease),
        })
    }

    #[cfg(test)]
    pub(crate) fn open_unpublished_read_only(
        path: impl AsRef<Path>,
    ) -> Result<Self, IndexValidationError> {
        let path = path.as_ref().to_path_buf();
        let artifact_file = File::open(&path)
            .map_err(|error| IndexValidationError::Unavailable(error.to_string()))?;
        let database_url = immutable_read_only_sqlite_uri(&artifact_file, &path)?;
        let (diesel_connection, validation_connection) = open_connections(&database_url)?;
        Ok(Self {
            path,
            _verified_artifact_sha256: None,
            verified_generation: None,
            reader_acquisition_ms: 0,
            reader_visible_sha_pass_count: 0,
            reader_generation_sha_pass_count: 0,
            compatibility_stamp_check_count: 0,
            diesel_connection: RefCell::new(diesel_connection),
            validation_connection: RefCell::new(validation_connection),
            _artifact_file: artifact_file,
            _generation_lease: None,
        })
    }

    #[cfg(test)]
    pub(crate) fn open_unpublished_read_only_with_vectors(
        path: impl AsRef<Path>,
    ) -> Result<Self, IndexValidationError> {
        register_sqlite_vec_extension().map_err(IndexValidationError::Unavailable)?;
        Self::open_unpublished_read_only(path)
    }

    pub fn open_read_only_with_vectors(
        path: impl AsRef<Path>,
    ) -> Result<Self, IndexValidationError> {
        register_sqlite_vec_extension().map_err(IndexValidationError::Unavailable)?;
        Self::open_read_only(path)
    }

    pub fn path(&self) -> &Path {
        &self.path
    }

    pub(crate) fn validation_connection(
        &self,
    ) -> Result<RefMut<'_, Connection>, IndexValidationError> {
        Ok(self.validation_connection.borrow_mut())
    }

    pub(crate) fn with_diesel_connection<T>(
        &self,
        f: impl FnOnce(&mut SqliteConnection) -> T,
    ) -> T {
        f(&mut self.diesel_connection.borrow_mut())
    }

    fn verified_generation_identity(
        &self,
    ) -> Result<&VerifiedArtifactGenerationIdentity, IndexValidationError> {
        let identity = self.verified_generation.as_ref().ok_or_else(|| {
            IndexValidationError::Unavailable(
                "validation composition requires a manifest-verified artifact generation"
                    .to_string(),
            )
        })?;
        self.verify_generation_unchanged(identity)?;
        Ok(identity)
    }

    fn verify_generation_unchanged(
        &self,
        expected: &VerifiedArtifactGenerationIdentity,
    ) -> Result<(), IndexValidationError> {
        let metadata = self
            ._artifact_file
            .metadata()
            .map_err(|error| IndexValidationError::Unavailable(error.to_string()))?;
        if metadata.len() != expected.bytes
            || file_identity(&metadata) != expected.file_identity
            || modified_unix_nanos(&metadata) != expected.modified_unix_nanos
            || self.verified_generation.as_ref() != Some(expected)
        {
            return Err(IndexValidationError::Unavailable(
                "verified artifact generation changed after validation binding".to_string(),
            ));
        }
        Ok(())
    }

    #[doc(hidden)]
    pub fn validate_generation_binding(&self) -> Result<(), IndexValidationError> {
        self.verified_generation_identity().map(|_| ())
    }

    #[doc(hidden)]
    pub fn verified_generation_evidence(&self) -> Result<serde_json::Value, IndexValidationError> {
        let identity = self.verified_generation_identity()?;
        Ok(serde_json::json!({
            "canonical_artifact_path": identity.canonical_artifact_path,
            "generation_path": identity.generation_path,
            "file_identity": identity.file_identity,
            "bytes": identity.bytes,
            "trusted_sha256": identity.trusted_sha256,
            "reader_acquisition_ms": self.reader_acquisition_ms,
            "reader_visible_sha_pass_count": self.reader_visible_sha_pass_count,
            "reader_generation_sha_pass_count": self.reader_generation_sha_pass_count,
            "compatibility_stamp_check_count": self.compatibility_stamp_check_count,
        }))
    }

    #[cfg(test)]
    pub(crate) fn verified_artifact_sha256(&self) -> Option<&str> {
        self._verified_artifact_sha256.as_deref()
    }
}

fn validate_compatibility_stamps(
    connection: &Connection,
    manifest_contract_version: &str,
    manifest_schema_version: &str,
) -> Result<(), IndexValidationError> {
    let sql = format!(
        "SELECT key, value FROM {ARTIFACT_METADATA_TABLE} WHERE key IN (?1, ?2) ORDER BY key"
    );
    let mut statement = connection.prepare(&sql).map_err(|error| {
        IndexValidationError::Unavailable(format!(
            "artifact compatibility metadata is unavailable: {error}; rebuild the artifact"
        ))
    })?;
    let rows = statement
        .query_map(
            [
                artifact_metadata_keys::ARTIFACT_CONTRACT_VERSION,
                artifact_metadata_keys::SCHEMA_VERSION,
            ],
            |row| Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?)),
        )
        .map_err(|error| IndexValidationError::Unavailable(error.to_string()))?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|error| IndexValidationError::Unavailable(error.to_string()))?;

    if rows.len() != 2 {
        return Err(IndexValidationError::Unavailable(
            "artifact compatibility metadata is missing or duplicated; rebuild the artifact with `atlas index build`"
                .to_string(),
        ));
    }
    let values = rows
        .into_iter()
        .collect::<std::collections::BTreeMap<_, _>>();
    let contract_version = values
        .get(artifact_metadata_keys::ARTIFACT_CONTRACT_VERSION)
        .ok_or_else(|| unsupported_stamp("artifact contract", "missing"))?;
    let schema_version = values
        .get(artifact_metadata_keys::SCHEMA_VERSION)
        .ok_or_else(|| unsupported_stamp("schema", "missing"))?;
    if contract_version != ARTIFACT_CONTRACT_VERSION
        || contract_version != manifest_contract_version
    {
        return Err(unsupported_stamp("artifact contract", contract_version));
    }
    if schema_version != ARTIFACT_SCHEMA_VERSION || schema_version != manifest_schema_version {
        return Err(unsupported_stamp("schema", schema_version));
    }
    Ok(())
}

fn unsupported_stamp(kind: &str, actual: &str) -> IndexValidationError {
    IndexValidationError::Unavailable(format!(
        "artifact {kind} version `{actual}` is unsupported or does not match its manifest; rebuild the artifact with `atlas index build`"
    ))
}

fn verified_generation_identity(
    artifact_path: &Path,
    generation_path: &Path,
    artifact_file: &File,
    trusted_sha256: String,
) -> Result<VerifiedArtifactGenerationIdentity, IndexValidationError> {
    let metadata = artifact_file
        .metadata()
        .map_err(|error| IndexValidationError::Unavailable(error.to_string()))?;
    let canonical_artifact_path = std::fs::canonicalize(artifact_path)
        .map_err(|error| IndexValidationError::Unavailable(error.to_string()))?;
    let canonical_generation_path = std::fs::canonicalize(generation_path)
        .map_err(|error| IndexValidationError::Unavailable(error.to_string()))?;
    Ok(VerifiedArtifactGenerationIdentity {
        canonical_artifact_path,
        generation_path: canonical_generation_path,
        file_identity: file_identity(&metadata),
        bytes: metadata.len(),
        trusted_sha256,
        modified_unix_nanos: modified_unix_nanos(&metadata),
    })
}

#[cfg(unix)]
fn file_identity(metadata: &std::fs::Metadata) -> String {
    use std::os::unix::fs::MetadataExt;

    format!("dev:{}:ino:{}", metadata.dev(), metadata.ino())
}

#[cfg(windows)]
fn file_identity(metadata: &std::fs::Metadata) -> String {
    use std::os::windows::fs::MetadataExt;

    format!(
        "volume:{}:file-index:{}",
        metadata.volume_serial_number().unwrap_or_default(),
        metadata.file_index().unwrap_or_default()
    )
}

#[cfg(not(any(unix, windows)))]
fn file_identity(metadata: &std::fs::Metadata) -> String {
    format!("unsupported:bytes:{}", metadata.len())
}

fn modified_unix_nanos(metadata: &std::fs::Metadata) -> Option<u128> {
    metadata
        .modified()
        .ok()?
        .duration_since(UNIX_EPOCH)
        .ok()
        .map(|duration| duration.as_nanos())
}

fn open_connections(
    database_url: &str,
) -> Result<(SqliteConnection, Connection), IndexValidationError> {
    let mut diesel_connection = SqliteConnection::establish(database_url)
        .map_err(|error| IndexValidationError::Unavailable(error.to_string()))?;
    diesel_connection
        .batch_execute("PRAGMA query_only = ON")
        .map_err(|error| IndexValidationError::Unavailable(error.to_string()))?;
    let validation_connection = Connection::open_with_flags(
        database_url,
        OpenFlags::SQLITE_OPEN_READ_ONLY | OpenFlags::SQLITE_OPEN_URI,
    )
    .map_err(|error| IndexValidationError::Unavailable(error.to_string()))?;
    Ok((diesel_connection, validation_connection))
}

fn immutable_read_only_sqlite_uri(
    artifact_file: &File,
    display_path: &Path,
) -> Result<String, IndexValidationError> {
    #[cfg(unix)]
    {
        use std::os::fd::AsRawFd;

        let _ = display_path;
        Ok(format!(
            "file:/dev/fd/{}?mode=ro&immutable=1",
            artifact_file.as_raw_fd()
        ))
    }

    #[cfg(not(unix))]
    read_only_sqlite_uri(display_path)
}

#[cfg(not(unix))]
fn read_only_sqlite_uri(path: &Path) -> Result<String, IndexValidationError> {
    let path = path.to_str().ok_or_else(|| {
        IndexValidationError::Unavailable(format!(
            "SQLite artifact path is not valid UTF-8: {}",
            path.display()
        ))
    })?;
    let mut escaped = String::with_capacity(path.len());
    for ch in path.chars() {
        match ch {
            '?' => escaped.push_str("%3f"),
            '#' => escaped.push_str("%23"),
            '%' => escaped.push_str("%25"),
            _ => escaped.push(ch),
        }
    }
    Ok(format!("file:{escaped}?mode=ro&immutable=1"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use atlas_domain::RecordKey;
    use sha2::{Digest, Sha256};

    #[test]
    fn old_unbound_manifest_is_rejected_with_rebuild_guidance() {
        let root = std::env::temp_dir().join(format!("atlas-old-pair-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&root);
        std::fs::create_dir_all(&root).unwrap();
        let path = root.join("index.sqlite");
        rusqlite::Connection::open(&path).unwrap();
        std::fs::write(root.join("manifest.json"), r#"{"build":{}}"#).unwrap();
        let error = match SqliteIndexReader::open_read_only(&path) {
            Ok(_) => panic!("old unbound manifest must be rejected"),
            Err(error) => error,
        };
        assert!(error.to_string().contains("rebuild"));
        std::fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn unsupported_manifest_envelope_version_is_rejected() {
        let root = unique_root("old-manifest-envelope");
        std::fs::create_dir_all(&root).unwrap();
        let path = root.join("index.sqlite");
        create_valid_generation(&path, "Old Envelope");
        std::fs::write(
            root.join("manifest.json"),
            format!(
                r#"{{"manifest_version":"pf2e-atlas-artifact-manifest/v2","artifact_contract_version":"{}","schema_version":"{}","build":{{"artifact_sha256":"{}"}}}}"#,
                crate::ARTIFACT_CONTRACT_VERSION,
                crate::ARTIFACT_SCHEMA_VERSION,
                sha256(&path),
            ),
        )
        .unwrap();

        let error = match SqliteIndexReader::open_read_only(&path) {
            Ok(_) => panic!("old manifest envelope must be rejected"),
            Err(error) => error,
        };
        assert!(error.to_string().contains("manifest contract"));
        assert!(error.to_string().contains("rebuild"));
        std::fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn missing_manifest_hard_link_fails_closed() {
        let root = unique_root("missing-manifest-hard-link");
        let published = root.join("published");
        let unbound = root.join("unbound");
        std::fs::create_dir_all(&published).unwrap();
        std::fs::create_dir_all(&unbound).unwrap();
        let source = published.join("index.sqlite");
        sqlite_with_marker(&source, "bound");
        write_manifest(&published.join("manifest.json"), &source);
        let hard_link = unbound.join("index.sqlite");
        std::fs::hard_link(&source, &hard_link).unwrap();

        let error = match SqliteIndexReader::open_read_only(&hard_link) {
            Ok(_) => panic!("an otherwise valid hard link without its manifest must fail"),
            Err(error) => error,
        };
        assert!(
            error
                .to_string()
                .contains("required adjacent manifest is missing")
        );
        assert!(error.to_string().contains("rebuild"));
        std::fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn warm_keyed_reads_skip_global_scans_and_full_file_hashes() {
        let root = unique_root("warm-keyed-read");
        std::fs::create_dir_all(&root).unwrap();
        let artifact = root.join("index.sqlite");
        let manifest = root.join("manifest.json");
        create_valid_generation(&artifact, "Warm");
        let connection = rusqlite::Connection::open(&artifact).unwrap();
        connection
            .execute(
                "UPDATE records SET record_kind = 'creature', foundry_document_type = 'Actor', foundry_record_type = 'npc' WHERE record_key = 'actions:testAction1'",
                [],
            )
            .unwrap();
        crate::test_support::insert_minimal_canonical_npc_body(
            &connection,
            "actions:testAction1",
            20,
            30,
            30,
            8,
        )
        .unwrap();
        drop(connection);
        write_manifest(&manifest, &artifact);

        drop(SqliteIndexReader::open_read_only(&artifact).unwrap());
        crate::read::records::reset_canonical_coherence_scan_count();
        let reader = SqliteIndexReader::open_read_only(&artifact).unwrap();
        let evidence = reader.verified_generation_evidence().unwrap();
        assert_eq!(evidence["reader_visible_sha_pass_count"], 0);
        assert_eq!(evidence["reader_generation_sha_pass_count"], 0);
        assert_eq!(evidence["compatibility_stamp_check_count"], 1);

        let key = RecordKey::parse("actions:testAction1").unwrap();
        for _ in 0..2 {
            let hydrated = reader
                .load_hydrated_records_by_key(std::slice::from_ref(&key))
                .unwrap();
            assert_eq!(hydrated.len(), 1);
        }
        assert_eq!(crate::read::records::canonical_coherence_scan_count(), 0);

        drop(reader);
        std::fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn incompatible_sqlite_version_stamp_is_rejected_on_open() {
        let root = unique_root("incompatible-stamp");
        std::fs::create_dir_all(&root).unwrap();
        let artifact = root.join("index.sqlite");
        create_valid_generation(&artifact, "Old Contract");
        let connection = rusqlite::Connection::open(&artifact).unwrap();
        connection
            .execute(
                "UPDATE artifact_metadata SET value = 'pf2e-atlas-artifact/v2' WHERE key = 'artifact_contract_version'",
                [],
            )
            .unwrap();
        drop(connection);
        write_manifest(&root.join("manifest.json"), &artifact);

        let error = match SqliteIndexReader::open_read_only(&artifact) {
            Ok(_) => panic!("old artifact contract must be rejected"),
            Err(error) => error,
        };
        assert!(error.to_string().contains("artifact contract"));
        assert!(error.to_string().contains("rebuild"));
        std::fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn keyed_canonical_decode_reports_local_mutation_normally() {
        let root = unique_root("typed-decode-mutation");
        std::fs::create_dir_all(&root).unwrap();
        let artifact = root.join("index.sqlite");
        create_valid_generation(&artifact, "Mutation");
        let connection = rusqlite::Connection::open(&artifact).unwrap();
        crate::test_support::insert_minimal_canonical_npc_body(
            &connection,
            "actions:testAction1",
            20,
            30,
            30,
            8,
        )
        .unwrap();
        connection
            .execute(
                "UPDATE canonical_creature_records SET canonical_json = '{' WHERE record_key = 'actions:testAction1'",
                [],
            )
            .unwrap();
        drop(connection);
        write_manifest(&root.join("manifest.json"), &artifact);

        let reader = SqliteIndexReader::open_read_only(&artifact).unwrap();
        let key = RecordKey::parse("actions:testAction1").unwrap();
        let error = reader
            .load_canonical_record_bodies_by_key(&[key])
            .expect_err("malformed canonical JSON must fail typed decode");
        assert!(matches!(error, crate::RecordLoadError::InvalidData(_)));

        drop(reader);
        std::fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn generation_binding_allows_publish_while_old_reader_stays_coherent() {
        let root = unique_root("verified-generation");
        std::fs::create_dir_all(&root).unwrap();
        let artifact = root.join("index.sqlite");
        let manifest = root.join("manifest.json");
        let replacement_artifact = root.join("replacement.sqlite");
        let replacement_manifest = root.join("replacement.json");
        create_valid_generation(&artifact, "Verified Old");
        create_valid_generation(&replacement_artifact, "Visible New");
        write_manifest(&manifest, &artifact);
        write_manifest(&replacement_manifest, &replacement_artifact);
        let old_hash = sha256(&artifact);
        let new_hash = sha256(&replacement_artifact);
        let old_generation = crate::artifact::pair::generation_path(&artifact, &old_hash);
        let new_generation = crate::artifact::pair::generation_path(&artifact, &new_hash);
        let (attempted_tx, attempted_rx) = std::sync::mpsc::channel();
        let (published_tx, published_rx) = std::sync::mpsc::channel();

        let reader = SqliteIndexReader::open_bound_read_only_with_hook(&artifact, || {
            let target_artifact = artifact.clone();
            let target_manifest = manifest.clone();
            std::thread::spawn(move || {
                let contention_probe = std::fs::OpenOptions::new()
                    .read(true)
                    .write(true)
                    .open(crate::artifact::pair::lock_path(&target_manifest))
                    .unwrap();
                let contention_error = contention_probe
                    .try_lock()
                    .expect_err("reader acquisition must hold the shared target lock");
                assert!(matches!(
                    contention_error,
                    std::fs::TryLockError::WouldBlock
                ));
                attempted_tx.send(()).unwrap();
                let receipt = crate::ArtifactPublicationReceipt::issue_test(
                    &replacement_artifact,
                    &target_artifact,
                )
                .unwrap();
                published_tx
                    .send(crate::publish_artifact_pair(
                        receipt,
                        &replacement_manifest,
                        &target_artifact,
                        &target_manifest,
                    ))
                    .unwrap();
            });
            attempted_rx.recv().unwrap();
        })
        .unwrap();

        published_rx
            .recv_timeout(std::time::Duration::from_secs(10))
            .unwrap()
            .unwrap();
        assert_reader_generation(&reader, "Verified Old", &old_hash);
        drop(reader);
        assert!(!old_generation.exists());
        assert!(new_generation.exists());

        let reader = SqliteIndexReader::open_read_only(&artifact).unwrap();
        assert_reader_generation(&reader, "Visible New", &new_hash);
        drop(reader);
        assert_eq!(sha256(&artifact), new_hash);
        std::fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn deep_receipt_preserves_independent_outputs_with_one_coherence_pass() {
        let root = unique_root("deep-receipt-equality");
        std::fs::create_dir_all(&root).unwrap();
        let artifact = root.join("index.sqlite");
        let manifest = root.join("manifest.json");
        create_valid_generation(&artifact, "Receipt");
        write_manifest(&manifest, &artifact);

        let independent = SqliteIndexReader::open_read_only(&artifact).unwrap();
        let expected_check = independent.check().unwrap();
        let expected_inspect = independent.inspect().unwrap();
        let expected_deep = independent
            .validate_target(crate::ValidationTarget::BaseOnly)
            .unwrap();

        crate::artifact::validation::reset_deep_coherence_validation_count();
        let composed = SqliteIndexReader::open_read_only(&artifact).unwrap();
        let actual_check = composed.check().unwrap();
        let actual_deep = composed
            .validate_target(crate::ValidationTarget::BaseOnly)
            .unwrap();
        let actual_inspect = composed
            .inspect_with_validation_report(actual_deep.clone())
            .unwrap();

        assert_eq!(actual_check, expected_check);
        assert_eq!(actual_inspect, expected_inspect);
        assert_eq!(actual_deep, expected_deep);
        assert_eq!(
            crate::artifact::validation::deep_coherence_validation_count(),
            1
        );
        drop(composed);
        drop(independent);
        std::fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn deep_receipt_preserves_independent_failure_payload_and_order() {
        let root = unique_root("deep-receipt-error-equality");
        std::fs::create_dir_all(&root).unwrap();
        let artifact = root.join("index.sqlite");
        let manifest = root.join("manifest.json");
        create_valid_generation(&artifact, "Receipt Error");
        let connection = rusqlite::Connection::open(&artifact).unwrap();
        connection.execute("DROP TABLE item_records", []).unwrap();
        drop(connection);
        write_manifest(&manifest, &artifact);

        let independent = SqliteIndexReader::open_read_only(&artifact).unwrap();
        let expected_error = independent.inspect().unwrap_err().to_string();

        crate::artifact::validation::reset_deep_coherence_validation_count();
        let composed = SqliteIndexReader::open_read_only(&artifact).unwrap();
        let deep = composed
            .validate_target(crate::ValidationTarget::BaseOnly)
            .unwrap();
        let actual_error = composed
            .inspect_with_validation_report(deep)
            .unwrap_err()
            .to_string();

        assert_eq!(actual_error, expected_error);
        assert_eq!(
            crate::artifact::validation::deep_coherence_validation_count(),
            1
        );
        drop(composed);
        drop(independent);
        std::fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn generation_evidence_distinguishes_reader_bindings() {
        let root = unique_root("deep-receipt-cross-generation");
        let first_root = root.join("first");
        let second_root = root.join("second");
        std::fs::create_dir_all(&first_root).unwrap();
        std::fs::create_dir_all(&second_root).unwrap();
        let first = first_root.join("index.sqlite");
        let second = second_root.join("index.sqlite");
        create_valid_generation(&first, "First");
        create_valid_generation(&second, "Second");
        write_manifest(&first_root.join("manifest.json"), &first);
        write_manifest(&second_root.join("manifest.json"), &second);

        let first_reader = SqliteIndexReader::open_read_only(&first).unwrap();
        let second_reader = SqliteIndexReader::open_read_only(&second).unwrap();
        let first_evidence = first_reader.verified_generation_evidence().unwrap();
        let second_evidence = second_reader.verified_generation_evidence().unwrap();
        assert_ne!(first_evidence, second_evidence);
        first_reader.validate_generation_binding().unwrap();
        second_reader.validate_generation_binding().unwrap();

        drop(second_reader);
        drop(first_reader);
        std::fs::remove_dir_all(root).unwrap();
    }

    #[cfg(unix)]
    #[test]
    fn deep_receipt_detects_retained_generation_tamper_without_path_reopen() {
        let root = unique_root("deep-receipt-tamper");
        std::fs::create_dir_all(&root).unwrap();
        let artifact = root.join("index.sqlite");
        create_valid_generation(&artifact, "Tamper");
        write_manifest(&root.join("manifest.json"), &artifact);
        let reader = SqliteIndexReader::open_read_only(&artifact).unwrap();
        let evidence = reader.verified_generation_evidence().unwrap();
        let generation_path = evidence["generation_path"].as_str().unwrap();
        let bytes = evidence["bytes"].as_u64().unwrap();

        let file = std::fs::OpenOptions::new()
            .append(true)
            .open(generation_path)
            .unwrap();
        file.set_len(bytes + 1).unwrap();
        drop(file);
        let error = reader
            .validate_generation_binding()
            .expect_err("retained generation drift must invalidate the receipt");
        assert!(error.to_string().contains("generation changed"));

        drop(reader);
        std::fs::remove_dir_all(root).unwrap();
    }

    fn assert_reader_generation(reader: &SqliteIndexReader, name: &str, hash: &str) {
        assert_eq!(reader.verified_artifact_sha256(), Some(hash));
        assert_eq!(reader.check().unwrap().status, crate::ValidationStatus::Ok);
        assert_eq!(
            reader.validate().unwrap().status,
            crate::ValidationStatus::Ok
        );
        assert_eq!(reader.inspect().unwrap().records.total_records, 3);
        let hydrated = reader.load_hydrated_records().unwrap();
        assert_eq!(hydrated.len(), 3);
        assert_eq!(hydrated[0].record.identity.name, format!("{name} 1"));
        let records = reader.load_records().unwrap();
        assert_eq!(records[0].identity.name, format!("{name} 1"));
        let validation_name: String = reader
            .validation_connection()
            .unwrap()
            .query_row(
                "SELECT name FROM records WHERE record_key = 'actions:testAction1'",
                [],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(validation_name, format!("{name} 1"));
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

    fn unique_root(name: &str) -> PathBuf {
        std::env::temp_dir().join(format!(
            "atlas-reader-{name}-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ))
    }

    fn sqlite_with_marker(path: &Path, marker: &str) {
        let connection = rusqlite::Connection::open(path).unwrap();
        connection
            .execute_batch("CREATE TABLE marker(value TEXT NOT NULL);")
            .unwrap();
        connection
            .execute("INSERT INTO marker(value) VALUES (?1)", [marker])
            .unwrap();
    }

    fn write_manifest(path: &Path, artifact: &Path) {
        std::fs::write(
            path,
            format!(
                r#"{{"manifest_version":"{}","artifact_contract_version":"{}","schema_version":"{}","build":{{"artifact_sha256":"{}"}}}}"#,
                crate::ARTIFACT_MANIFEST_VERSION,
                crate::ARTIFACT_CONTRACT_VERSION,
                crate::ARTIFACT_SCHEMA_VERSION,
                sha256(artifact),
            ),
        )
        .unwrap();
    }

    fn sha256(path: &Path) -> String {
        format!("{:x}", Sha256::digest(std::fs::read(path).unwrap()))
    }
}
