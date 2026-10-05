use std::sync::OnceLock;

use serde::Deserialize;
use sha2::{Digest, Sha256};

use crate::source::dto::PF2E_SOURCE_PINNED_COMMIT;

use super::{SourceDocumentRole, SourceParentContextSelector};

pub const PF2E_TYPE_REGISTRY_VERSION: &str = "pf2e-type-registry/v1";
pub const PF2E_TYPE_REGISTRY_SHA256: &str =
    "e62b1d6797a72dd97297b5af05170477871712623fdb89497e6ab2100331d618";
pub const PF2E_TYPE_REGISTRY_ENTRY_COUNT: usize = 313;

const REGISTRY_BYTES: &[u8] = include_bytes!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../contracts/pf2e-type-registry.yaml"
));

#[derive(Debug, Deserialize)]
struct RegistryDocument {
    schema_version: String,
    source_identity: RegistrySourceIdentity,
    entries: Vec<RegistryEntry>,
}

#[derive(Debug, Deserialize)]
struct RegistrySourceIdentity {
    pf2e_commit: String,
}

#[derive(Debug, Clone, Deserialize)]
pub(crate) struct RegistryEntry {
    pub type_id: String,
    pub document_class: String,
    pub type_discriminator: String,
    pub role: SourceDocumentRole,
    pub parent_context: SourceParentContextSelector,
    pub corpus: RegistryCorpus,
}

#[derive(Debug, Clone, Deserialize)]
pub(crate) struct RegistryCorpus {
    pub count: usize,
}

#[derive(Debug)]
pub(crate) struct AcceptedRegistry {
    pub entries: Vec<RegistryEntry>,
}

pub(crate) fn accepted_registry() -> Result<&'static AcceptedRegistry, String> {
    static REGISTRY: OnceLock<Result<AcceptedRegistry, String>> = OnceLock::new();
    REGISTRY
        .get_or_init(|| {
            let digest = format!("{:x}", Sha256::digest(REGISTRY_BYTES));
            if digest != PF2E_TYPE_REGISTRY_SHA256 {
                return Err(format!(
                    "accepted registry digest mismatch: expected {PF2E_TYPE_REGISTRY_SHA256}, found {digest}"
                ));
            }
            let document: RegistryDocument =
                yaml_serde::from_slice(REGISTRY_BYTES).map_err(|error| error.to_string())?;
            if document.schema_version != PF2E_TYPE_REGISTRY_VERSION {
                return Err(format!(
                    "accepted registry schema mismatch: expected {PF2E_TYPE_REGISTRY_VERSION}, found {}",
                    document.schema_version
                ));
            }
            if document.source_identity.pf2e_commit != PF2E_SOURCE_PINNED_COMMIT {
                return Err(format!(
                    "accepted registry source commit mismatch: expected {PF2E_SOURCE_PINNED_COMMIT}, found {}",
                    document.source_identity.pf2e_commit
                ));
            }
            if document.entries.len() != PF2E_TYPE_REGISTRY_ENTRY_COUNT {
                return Err(format!(
                    "accepted registry entry count mismatch: expected {PF2E_TYPE_REGISTRY_ENTRY_COUNT}, found {}",
                    document.entries.len()
                ));
            }
            Ok(AcceptedRegistry {
                entries: document.entries,
            })
        })
        .as_ref()
        .map_err(Clone::clone)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn accepted_registry_is_digest_count_and_source_bound() {
        let registry = accepted_registry().expect("accepted registry");
        assert_eq!(registry.entries.len(), PF2E_TYPE_REGISTRY_ENTRY_COUNT);
        assert!(registry.entries.iter().any(|entry| {
            entry.type_id == "actor--npc--top-level--root--root--root"
                && entry.document_class == "Actor"
                && entry.type_discriminator == "npc"
                && entry.role == SourceDocumentRole::TopLevel
                && entry.parent_context == SourceParentContextSelector::root()
                && entry.corpus.count > 0
        }));
    }
}
