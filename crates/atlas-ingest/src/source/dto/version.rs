use serde::Serialize;

use super::{SourceDiagnostic, SourceDiagnosticKind, SourceIdentity};

pub const PF2E_SOURCE_CONTRACT_VERSION: &str = "pf2e-serialized-source/v1";
pub const PF2E_SOURCE_PINNED_SYSTEM_ID: &str = "pf2e";
pub const PF2E_SOURCE_PINNED_SYSTEM_VERSION: &str = "6.12.4";
pub const PF2E_SOURCE_PINNED_COMMIT: &str = "4cbdaa37d6c33e9519561bae2c59a23e0288cbce";
pub const PF2E_SOURCE_PINNED_SIGNATURE: &str =
    "foundry-pf2e:sha256:dd78d67f5b6d25bf65e30ca4da66af76e7a31e1e7d990562f139154b1752603a";

/// Version identity emitted with source-boundary diagnostics and build inputs.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct SourceVersionMetadata {
    contract_version: &'static str,
    system_id: String,
    system_version: String,
    upstream_commit: String,
}

impl SourceVersionMetadata {
    pub const fn contract_version(&self) -> &'static str {
        self.contract_version
    }

    pub fn system_id(&self) -> &str {
        &self.system_id
    }

    pub fn system_version(&self) -> &str {
        &self.system_version
    }

    pub fn upstream_commit(&self) -> &str {
        &self.upstream_commit
    }
}

pub fn pinned_source_version_metadata() -> SourceVersionMetadata {
    SourceVersionMetadata {
        contract_version: PF2E_SOURCE_CONTRACT_VERSION,
        system_id: PF2E_SOURCE_PINNED_SYSTEM_ID.to_string(),
        system_version: PF2E_SOURCE_PINNED_SYSTEM_VERSION.to_string(),
        upstream_commit: PF2E_SOURCE_PINNED_COMMIT.to_string(),
    }
}

pub fn validate_pinned_source_version(
    identity: &SourceIdentity,
    system_id: &str,
    system_version: &str,
    upstream_commit: &str,
) -> Result<SourceVersionMetadata, SourceDiagnostic> {
    let expected = pinned_source_version_metadata();
    if system_id == expected.system_id
        && system_version == expected.system_version
        && upstream_commit == expected.upstream_commit
    {
        return Ok(expected);
    }

    Err(SourceDiagnostic::new(
        SourceDiagnosticKind::UnsupportedSourceVersion,
        identity,
        "$source_version",
        format!(
            "system={} version={} commit={}",
            expected.system_id, expected.system_version, expected.upstream_commit
        ),
        format!("system={system_id} version={system_version} commit={upstream_commit}"),
    ))
}
