use std::{error::Error, fmt};

use serde::{Deserialize, Serialize};
use serde_json::value::RawValue;

use crate::FoundryDocumentSource;

pub use crate::generated::SOURCE_CONTRACT_ID;
/// Increment when snapshot encoding or generated value-shape policy changes.
pub const SNAPSHOT_VERSION: u32 = 1;
// Ordered source pairs and generated presence/union tags expand the authored
// JSON reader's 128-container bound. Bound decoded trees as well as Serde's
// stack usage, since normal Rust Drop/Debug remain recursive.
const MAX_SNAPSHOT_DEPTH: usize = 1024;

#[derive(Serialize)]
struct SnapshotRef<'a> {
    version: u32,
    source_contract: &'static str,
    model: &'a FoundryDocumentSource,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct SnapshotHeader<'a> {
    version: u32,
    source_contract: String,
    #[serde(borrow)]
    model: &'a RawValue,
}

#[derive(Debug)]
pub enum SnapshotError {
    InvalidSnapshot(serde_json::Error),
    UnsupportedVersion(u32),
    SourceContractMismatch(String),
    NestingLimit,
}

impl fmt::Display for SnapshotError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidSnapshot(error) => write!(f, "Invalid typed source snapshot: {error}"),
            Self::UnsupportedVersion(version) => write!(
                f,
                "Unsupported source snapshot version {version}; rebuild required"
            ),
            Self::SourceContractMismatch(identity) => write!(
                f,
                "Unsupported source contract {identity}; rebuild required"
            ),
            Self::NestingLimit => write!(f, "Typed source snapshot exceeds nesting limit"),
        }
    }
}

impl Error for SnapshotError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::InvalidSnapshot(error) => Some(error),
            _ => None,
        }
    }
}

/// Encode the admitted model only. Raw source, filesystem provenance and a
/// diagnostic index remain separate concerns; invalid fields retain diagnostics.
pub fn encode_snapshot(model: &FoundryDocumentSource) -> Result<Vec<u8>, SnapshotError> {
    let mut bytes = Vec::new();
    let mut serializer = serde_json::Serializer::new(&mut bytes);
    SnapshotRef {
        version: SNAPSHOT_VERSION,
        source_contract: SOURCE_CONTRACT_ID,
        model,
    }
    .serialize(serde_stacker::Serializer::new(&mut serializer))
    .map_err(SnapshotError::InvalidSnapshot)?;
    check_depth(&bytes)?;
    Ok(bytes)
}

/// Decode a typed model without running authored JSON parsing or admission.
pub fn decode_snapshot(bytes: &[u8]) -> Result<FoundryDocumentSource, SnapshotError> {
    check_depth(bytes)?;
    let header: SnapshotHeader<'_> =
        serde_json::from_slice(bytes).map_err(SnapshotError::InvalidSnapshot)?;
    if header.version != SNAPSHOT_VERSION {
        return Err(SnapshotError::UnsupportedVersion(header.version));
    }
    if header.source_contract != SOURCE_CONTRACT_ID {
        return Err(SnapshotError::SourceContractMismatch(
            header.source_contract,
        ));
    }
    let mut decoder = serde_json::Deserializer::from_str(header.model.get());
    decoder.disable_recursion_limit();
    let model = FoundryDocumentSource::deserialize(serde_stacker::Deserializer::new(&mut decoder))
        .map_err(SnapshotError::InvalidSnapshot)?;
    decoder.end().map_err(SnapshotError::InvalidSnapshot)?;
    Ok(model)
}

// Only a nesting bound: Serde still validates JSON grammar, strings and tags.
fn check_depth(bytes: &[u8]) -> Result<(), SnapshotError> {
    let (mut depth, mut quoted, mut escaped) = (0usize, false, false);
    for &byte in bytes {
        if quoted {
            if escaped {
                escaped = false;
            } else if byte == b'\\' {
                escaped = true;
            } else if byte == b'"' {
                quoted = false;
            }
        } else {
            match byte {
                b'"' => quoted = true,
                b'[' | b'{' => {
                    depth += 1;
                    if depth > MAX_SNAPSHOT_DEPTH {
                        return Err(SnapshotError::NestingLimit);
                    }
                }
                b']' | b'}' => depth = depth.saturating_sub(1),
                _ => {}
            }
        }
    }
    Ok(())
}
