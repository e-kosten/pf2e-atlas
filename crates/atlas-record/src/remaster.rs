//! Authored remaster statements, separate from current reference resolution.
mod journal;
mod projection;
mod static_inputs;

use crate::source_content::SourceContentLocator;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

pub use journal::journal_declarations;
pub use projection::*;
pub use static_inputs::{ReviewedCorrespondence, ReviewedCorrespondences, file_declarations};

pub const ATLAS_CORRESPONDENCE_BYTES: &[u8] = include_bytes!("remaster/correspondences.json");
/// Metadata inputs have their own admission identity, independent of content rendering.
pub fn remaster_inputs_identity() -> String {
    hash(format!("{}:{}", REMASTER_RECIPE, hash(ATLAS_CORRESPONDENCE_BYTES)).as_bytes())
}
pub const REMASTER_RECIPE: &str = "source-remaster-statements/v2";

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RemasterChangeKind {
    Renamed,
    Merged,
    Replaced,
    Removed,
    MechanicsChanged,
    Discussion,
    Unclassified,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RemasterContext {
    pub section: String,
    pub namespace: String,
    pub qualifier: Option<String>,
    pub source_type: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RemasterReference {
    pub target: String,
    pub label: String,
    /// Ordinal in the selected source field; absent for static literal inputs.
    pub occurrence: Option<usize>,
    pub explicit_key: Option<atlas_domain::RecordKey>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RemasterSide {
    pub text: String,
    pub terms: Vec<String>,
    pub references: Vec<RemasterReference>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum RemasterSelector {
    TableRow { ordinal: usize },
    ListItem { ordinal: usize },
    Discussion { ordinal: usize },
    LiteralEntry { ordinal: usize },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum RemasterCitation {
    Journal {
        locator: SourceContentLocator,
        selector: RemasterSelector,
        authored_sha256: String,
    },
    CapturedFile {
        input_key: String,
        selector: RemasterSelector,
    },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RemasterDeclaration {
    pub id: String,
    pub kinds: Vec<RemasterChangeKind>,
    pub authored_status: Option<String>,
    pub context: RemasterContext,
    pub before: Vec<RemasterSide>,
    pub after: Vec<RemasterSide>,
    pub citations: Vec<RemasterCitation>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RemasterInputKind {
    Migration850,
    Migration887,
    AtlasCorrespondence,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RemasterOrigin {
    Foundry,
    AtlasReviewed,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RemasterSourceInput {
    pub key: String,
    pub kind: RemasterInputKind,
    pub origin: RemasterOrigin,
    pub revision: String,
    pub path: String,
    pub sha256: String,
    pub bytes: Vec<u8>,
}

impl RemasterSourceInput {
    pub fn new(
        kind: RemasterInputKind,
        origin: RemasterOrigin,
        revision: String,
        path: String,
        bytes: Vec<u8>,
    ) -> Self {
        let sha256 = hash(&bytes);
        let key = hash(format!("{kind:?}:{origin:?}:{revision}:{path}:{sha256}").as_bytes());
        Self {
            key,
            kind,
            origin,
            revision,
            path,
            sha256,
            bytes,
        }
    }
}

impl RemasterDeclaration {
    pub(crate) fn seal(mut self) -> Result<Self, RemasterError> {
        self.id.clear();
        self.id = hash(&serde_json::to_vec(&self)?);
        Ok(self)
    }
}

pub fn hash(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}

#[derive(Debug)]
pub enum RemasterError {
    Invalid(String),
    Json(serde_json::Error),
}
impl std::fmt::Display for RemasterError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Invalid(s) => f.write_str(s),
            Self::Json(e) => e.fmt(f),
        }
    }
}
impl std::error::Error for RemasterError {}
impl From<serde_json::Error> for RemasterError {
    fn from(value: serde_json::Error) -> Self {
        Self::Json(value)
    }
}

#[cfg(test)]
mod tests;
