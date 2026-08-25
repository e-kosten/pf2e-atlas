use std::fmt;

use serde::Serialize;

use super::version::{
    PF2E_SOURCE_CONTRACT_VERSION, PF2E_SOURCE_PINNED_COMMIT, PF2E_SOURCE_PINNED_SYSTEM_ID,
    PF2E_SOURCE_PINNED_SYSTEM_VERSION,
};

/// Stable identity attached to every source-boundary diagnostic.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct SourceIdentity {
    pub record_key: String,
    pub source_path: String,
}

impl SourceIdentity {
    pub fn new(record_key: impl Into<String>, source_path: impl Into<String>) -> Self {
        Self {
            record_key: record_key.into(),
            source_path: source_path.into(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum SourceDiagnosticKind {
    MalformedShape,
    UnknownDiscriminator,
    InvalidParentContext,
    UnsupportedSourceVersion,
}

/// A source-contract failure with enough context to fix the exact record.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct SourceDiagnostic {
    pub kind: SourceDiagnosticKind,
    pub source_contract_version: &'static str,
    pub source_system_id: &'static str,
    pub source_system_version: &'static str,
    pub source_upstream_commit: &'static str,
    #[serde(flatten)]
    details: Box<SourceDiagnosticDetails>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
struct SourceDiagnosticDetails {
    record_key: String,
    source_path: String,
    json_path: String,
    expected_shape: String,
    actual_shape: String,
}

impl SourceDiagnostic {
    pub(crate) fn new(
        kind: SourceDiagnosticKind,
        identity: &SourceIdentity,
        json_path: impl Into<String>,
        expected_shape: impl Into<String>,
        actual_shape: impl Into<String>,
    ) -> Self {
        Self {
            kind,
            source_contract_version: PF2E_SOURCE_CONTRACT_VERSION,
            source_system_id: PF2E_SOURCE_PINNED_SYSTEM_ID,
            source_system_version: PF2E_SOURCE_PINNED_SYSTEM_VERSION,
            source_upstream_commit: PF2E_SOURCE_PINNED_COMMIT,
            details: Box::new(SourceDiagnosticDetails {
                record_key: identity.record_key.clone(),
                source_path: identity.source_path.clone(),
                json_path: json_path.into(),
                expected_shape: expected_shape.into(),
                actual_shape: actual_shape.into(),
            }),
        }
    }

    pub fn record_key(&self) -> &str {
        &self.details.record_key
    }

    pub fn source_path(&self) -> &str {
        &self.details.source_path
    }

    pub fn json_path(&self) -> &str {
        &self.details.json_path
    }

    pub fn expected_shape(&self) -> &str {
        &self.details.expected_shape
    }

    pub fn actual_shape(&self) -> &str {
        &self.details.actual_shape
    }
}

impl fmt::Display for SourceDiagnostic {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "PF2e serialized-Source diagnostic for record {} at {} ({}): expected {}, found {}",
            self.record_key(),
            self.source_path(),
            self.json_path(),
            self.expected_shape(),
            self.actual_shape()
        )
    }
}

impl std::error::Error for SourceDiagnostic {}
