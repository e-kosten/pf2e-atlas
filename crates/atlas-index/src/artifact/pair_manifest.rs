use std::fs::File;
use std::path::{Path, PathBuf};

use serde::Deserialize;

use crate::artifact::metadata::{
    ARTIFACT_CONTRACT_VERSION, ARTIFACT_MANIFEST_VERSION, ARTIFACT_SCHEMA_VERSION,
};
use crate::artifact::pair::{sha256_file, write_error};
use crate::{IndexValidationError, IndexWriteError};

pub(crate) const ADJACENT_MANIFEST_FILE_NAME: &str = "manifest.json";

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(try_from = "String")]
pub(crate) struct ArtifactSha256(String);

impl ArtifactSha256 {
    fn parse(value: String) -> Result<Self, String> {
        if value.len() != 64
            || !value
                .bytes()
                .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
        {
            return Err(
                "artifact_sha256 must be exactly 64 lowercase hexadecimal characters".to_string(),
            );
        }
        Ok(Self(value))
    }

    pub(crate) fn as_str(&self) -> &str {
        &self.0
    }
}

impl std::fmt::Display for ArtifactSha256 {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(self.as_str())
    }
}

impl TryFrom<String> for ArtifactSha256 {
    type Error = String;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        Self::parse(value)
    }
}

#[derive(Deserialize)]
pub(super) struct PairManifest {
    pub(super) manifest_version: String,
    pub(super) artifact_contract_version: String,
    pub(super) schema_version: String,
    pub(super) build: PairManifestBuild,
}

#[derive(Deserialize)]
pub(super) struct PairManifestBuild {
    pub(super) artifact_sha256: ArtifactSha256,
}

pub(crate) fn adjacent_manifest_path(artifact_path: &Path) -> PathBuf {
    artifact_path
        .parent()
        .unwrap_or_else(|| Path::new("."))
        .join(ADJACENT_MANIFEST_FILE_NAME)
}

pub(crate) fn verify_pair_files(
    artifact_path: &Path,
    manifest_path: &Path,
) -> Result<String, IndexWriteError> {
    let manifest = read_manifest(manifest_path)
        .map_err(|error| IndexWriteError::WriteFailed(error.to_string()))?;
    let file = File::open(artifact_path).map_err(write_error)?;
    let sha256 = sha256_file(&file).map_err(write_error)?;
    if sha256 != manifest.build.artifact_sha256.as_str() {
        return Err(IndexWriteError::WriteFailed(pair_mismatch().to_string()));
    }
    Ok(sha256)
}

pub(crate) fn verify_manifest_digest(
    manifest_path: &Path,
    expected_sha256: &str,
) -> Result<(), IndexWriteError> {
    let manifest = read_manifest(manifest_path)
        .map_err(|error| IndexWriteError::WriteFailed(error.to_string()))?;
    if manifest.build.artifact_sha256.as_str() != expected_sha256 {
        return Err(IndexWriteError::ReceiptInvalidated(
            "adjacent manifest digest does not match the artifact publication receipt".to_string(),
        ));
    }
    Ok(())
}

pub(super) fn read_manifest(path: &Path) -> Result<PairManifest, IndexValidationError> {
    let bytes = std::fs::read(path).map_err(|error| {
        let message = if error.kind() == std::io::ErrorKind::NotFound {
            format!(
                "required adjacent manifest is missing at {}; rebuild the artifact and manifest together with `atlas index build`",
                path.display()
            )
        } else {
            format!("unable to read adjacent manifest {}: {error}", path.display())
        };
        IndexValidationError::Unavailable(message)
    })?;
    let manifest: PairManifest = serde_json::from_slice(&bytes).map_err(|error| {
        IndexValidationError::Unavailable(format!(
            "adjacent manifest is not a supported pair manifest: {error}; rebuild the artifact and manifest together"
        ))
    })?;
    if manifest.manifest_version != ARTIFACT_MANIFEST_VERSION {
        return Err(IndexValidationError::Unavailable(format!(
            "adjacent manifest contract `{}` is unsupported; rebuild the artifact and manifest together",
            manifest.manifest_version
        )));
    }
    if manifest.artifact_contract_version != ARTIFACT_CONTRACT_VERSION {
        return Err(IndexValidationError::Unavailable(format!(
            "adjacent manifest artifact contract `{}` is unsupported; rebuild the artifact and manifest together",
            manifest.artifact_contract_version
        )));
    }
    if manifest.schema_version != ARTIFACT_SCHEMA_VERSION {
        return Err(IndexValidationError::Unavailable(format!(
            "adjacent manifest schema `{}` is unsupported; rebuild the artifact and manifest together",
            manifest.schema_version
        )));
    }
    Ok(manifest)
}

fn pair_mismatch() -> IndexValidationError {
    IndexValidationError::Unavailable(
        "SQLite artifact and adjacent manifest are not a matching published pair; rebuild them together with `atlas index build`".to_string(),
    )
}
