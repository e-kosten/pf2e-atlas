use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::UNIX_EPOCH;

use atlas_index::{ARTIFACT_CONTRACT_VERSION, ARTIFACT_SCHEMA_VERSION, EXPECTED_SOURCE_KIND};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use crate::error::IngestError;
use crate::source::loader::{
    default_manifest_path, json_files, relative_source_path, resolve_pack_path,
};
use crate::source::model::Manifest;

pub const ARTIFACT_MANIFEST_VERSION: &str = "pf2e-atlas-artifact-manifest/v1";
pub const ADJACENT_ARTIFACT_MANIFEST_PATH: &str = "manifest.json";

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ArtifactManifest {
    pub manifest_version: String,
    pub artifact_contract_version: String,
    pub schema_version: String,
    pub source: ArtifactManifestSource,
    pub build: ArtifactManifestBuild,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ArtifactManifestSource {
    pub kind: String,
    pub root: String,
    pub signature: String,
    pub record_count: usize,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub git_commit: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fingerprint: Option<SourceFingerprint>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fingerprint_unavailable_reason: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ArtifactManifestBuild {
    pub artifact_record_count: usize,
    pub generated_record_count: usize,
    pub document_embedding_count: usize,
    pub embedding_model: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SourceFingerprint {
    pub kind: String,
    pub value: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourcePositionReport {
    pub git_commit: Option<String>,
    pub fingerprint: Option<SourceFingerprint>,
    pub unavailable_reason: Option<String>,
}

#[derive(Debug, Clone)]
pub(crate) struct ArtifactManifestInput {
    pub source_root: PathBuf,
    pub source_signature: String,
    pub source_record_count: usize,
    pub artifact_record_count: usize,
    pub generated_record_count: usize,
    pub document_embedding_count: usize,
    pub embedding_model: String,
    pub source_position: SourcePositionReport,
}

impl ArtifactManifest {
    pub(crate) fn new(input: ArtifactManifestInput) -> Self {
        Self {
            manifest_version: ARTIFACT_MANIFEST_VERSION.to_string(),
            artifact_contract_version: ARTIFACT_CONTRACT_VERSION.to_string(),
            schema_version: ARTIFACT_SCHEMA_VERSION.to_string(),
            source: ArtifactManifestSource {
                kind: EXPECTED_SOURCE_KIND.to_string(),
                root: input.source_root.display().to_string(),
                signature: input.source_signature,
                record_count: input.source_record_count,
                git_commit: input.source_position.git_commit,
                fingerprint: input.source_position.fingerprint,
                fingerprint_unavailable_reason: input.source_position.unavailable_reason,
            },
            build: ArtifactManifestBuild {
                artifact_record_count: input.artifact_record_count,
                generated_record_count: input.generated_record_count,
                document_embedding_count: input.document_embedding_count,
                embedding_model: input.embedding_model,
            },
        }
    }
}

pub fn adjacent_artifact_manifest_path(artifact_path: &Path) -> PathBuf {
    artifact_path
        .parent()
        .unwrap_or_else(|| Path::new("."))
        .join(ADJACENT_ARTIFACT_MANIFEST_PATH)
}

pub fn write_artifact_manifest(
    path: &Path,
    manifest: &ArtifactManifest,
) -> Result<(), IngestError> {
    let serialized = serde_json::to_string_pretty(manifest)
        .map_err(|error| IngestError::ArtifactWriteFailed(error.to_string()))?;
    fs::write(path, serialized).map_err(|error| IngestError::ArtifactWriteFailed(error.to_string()))
}

pub fn read_artifact_manifest(path: &Path) -> Result<ArtifactManifest, IngestError> {
    let serialized = fs::read_to_string(path)
        .map_err(|error| IngestError::SourceUnavailable(error.to_string()))?;
    serde_json::from_str(&serialized)
        .map_err(|error| IngestError::ManifestParseFailed(error.to_string()))
}

pub fn compute_source_position_report(
    source_root: &Path,
    manifest_path: Option<&Path>,
) -> SourcePositionReport {
    if source_root.join(".git").exists() {
        return match source_git_commit_if_clean(source_root) {
            Ok(commit) => SourcePositionReport {
                git_commit: Some(commit),
                fingerprint: None,
                unavailable_reason: None,
            },
            Err(commit_error) => match metadata_source_fingerprint(source_root, manifest_path) {
                Ok(fingerprint) => SourcePositionReport {
                    git_commit: None,
                    fingerprint: Some(fingerprint),
                    unavailable_reason: Some(commit_error.to_string()),
                },
                Err(fingerprint_error) => SourcePositionReport {
                    git_commit: None,
                    fingerprint: None,
                    unavailable_reason: Some(format!("{commit_error}; {fingerprint_error}")),
                },
            },
        };
    }
    match metadata_source_fingerprint(source_root, manifest_path) {
        Ok(fingerprint) => SourcePositionReport {
            git_commit: None,
            fingerprint: Some(fingerprint),
            unavailable_reason: None,
        },
        Err(error) => SourcePositionReport {
            git_commit: None,
            fingerprint: None,
            unavailable_reason: Some(error.to_string()),
        },
    }
}

pub fn compute_source_fingerprint(
    source_root: &Path,
    manifest_path: Option<&Path>,
) -> Result<SourceFingerprint, IngestError> {
    metadata_source_fingerprint(source_root, manifest_path)
}

pub fn source_git_commit(source_root: &Path) -> Result<String, IngestError> {
    git_output(source_root, ["rev-parse", "HEAD"])
}

pub fn source_git_commit_if_clean(source_root: &Path) -> Result<String, IngestError> {
    ensure_git_worktree_clean(source_root)?;
    source_git_commit(source_root)
}

fn ensure_git_worktree_clean(source_root: &Path) -> Result<(), IngestError> {
    let output = Command::new("git")
        .arg("-C")
        .arg(source_root)
        .args(["status", "--porcelain", "--untracked-files=normal"])
        .output()
        .map_err(|error| IngestError::SourceUnavailable(error.to_string()))?;
    if !output.status.success() {
        return Err(IngestError::SourceUnavailable(
            String::from_utf8_lossy(&output.stderr).trim().to_string(),
        ));
    }
    if !output.stdout.is_empty() {
        Err(IngestError::SourceUnavailable(
            "git source checkout has uncommitted or untracked changes".to_string(),
        ))
    } else {
        Ok(())
    }
}

fn git_output<const N: usize>(source_root: &Path, args: [&str; N]) -> Result<String, IngestError> {
    let output = Command::new("git")
        .arg("-C")
        .arg(source_root)
        .args(args)
        .output()
        .map_err(|error| IngestError::SourceUnavailable(error.to_string()))?;
    if !output.status.success() {
        return Err(IngestError::SourceUnavailable(
            String::from_utf8_lossy(&output.stderr).trim().to_string(),
        ));
    }
    Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
}

fn metadata_source_fingerprint(
    source_root: &Path,
    manifest_path: Option<&Path>,
) -> Result<SourceFingerprint, IngestError> {
    let manifest_path = manifest_path
        .map(Path::to_path_buf)
        .unwrap_or_else(|| default_manifest_path(source_root));
    let serialized = fs::read_to_string(&manifest_path)
        .map_err(|error| IngestError::SourceUnavailable(error.to_string()))?;
    let manifest_hash = sha256_hex(serialized.as_bytes());
    let manifest = serde_json::from_str::<Manifest>(&serialized)
        .map_err(|error| IngestError::ManifestParseFailed(error.to_string()))?;
    let mut fields = Vec::new();
    fields.push(relative_source_path(source_root, &manifest_path));
    fields.push(manifest_hash);
    collect_optional_file_metadata(
        source_root,
        &source_root.join("static/lang/en.json"),
        &mut fields,
    )?;

    for pack in manifest.packs {
        fields.push("pack".to_string());
        fields.push(pack.name.clone());
        fields.push(pack.label.clone());
        fields.push(pack.document_type.clone());
        fields.push(pack.path.clone());
        let pack_path = resolve_pack_path(source_root, &pack);
        let mut files = json_files(&pack_path)?;
        files.sort();
        fields.push(files.len().to_string());
        for file in files {
            let metadata = fs::metadata(&file)
                .map_err(|error| IngestError::SourceUnavailable(error.to_string()))?;
            fields.push(relative_source_path(source_root, &file));
            fields.push(metadata.len().to_string());
            fields.push(modified_nanos(&metadata).to_string());
        }
    }

    Ok(SourceFingerprint {
        kind: "file_metadata_v1".to_string(),
        value: fingerprint_hash("atlas-source-fingerprint-file-metadata-v1", fields.iter()),
    })
}

fn collect_optional_file_metadata(
    source_root: &Path,
    path: &Path,
    fields: &mut Vec<String>,
) -> Result<(), IngestError> {
    if !path.is_file() {
        return Ok(());
    }

    let metadata =
        fs::metadata(path).map_err(|error| IngestError::SourceUnavailable(error.to_string()))?;
    fields.push("file".to_string());
    fields.push(relative_source_path(source_root, path));
    fields.push(metadata.len().to_string());
    fields.push(modified_nanos(&metadata).to_string());
    Ok(())
}

fn modified_nanos(metadata: &fs::Metadata) -> u128 {
    metadata
        .modified()
        .ok()
        .and_then(|modified| modified.duration_since(UNIX_EPOCH).ok())
        .map(|duration| duration.as_nanos())
        .unwrap_or(0)
}

fn fingerprint_hash<'a>(prefix: &str, fields: impl IntoIterator<Item = &'a String>) -> String {
    let mut hasher = Sha256::new();
    hash_field(&mut hasher, prefix);
    for field in fields {
        hash_field(&mut hasher, field);
    }
    format!("sha256:{}", hex_lower(hasher.finalize()))
}

fn hash_field(hasher: &mut Sha256, value: &str) {
    hasher.update(value.len().to_string().as_bytes());
    hasher.update(b":");
    hasher.update(value.as_bytes());
    hasher.update(b"\n");
}

fn sha256_hex(bytes: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(bytes);
    hex_lower(hasher.finalize())
}

fn hex_lower(bytes: impl AsRef<[u8]>) -> String {
    bytes
        .as_ref()
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect()
}
