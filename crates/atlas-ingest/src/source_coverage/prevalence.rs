use std::collections::BTreeSet;
use std::sync::OnceLock;

use serde::Deserialize;
use sha2::{Digest, Sha256};

use crate::source::dto::{
    PF2E_SOURCE_CONTRACT_VERSION, PF2E_SOURCE_PINNED_COMMIT, PF2E_SOURCE_PINNED_SIGNATURE,
};

use super::registry::{PF2E_TYPE_REGISTRY_SHA256, accepted_registry};
use super::{SourceLeafIdentity, SourceLeafSelector};

pub const PF2E_SOURCE_LEAF_PREVALENCE_VERSION: &str = "pf2e-source-leaf-prevalence/v1";
pub const PF2E_SOURCE_LEAF_PREVALENCE_SHA256: &str =
    "070c50eec2b31a51eb68afb4afdd1dfa42247eac17ea23e465561736a0e60097";

const PREVALENCE_BYTES: &[u8] = include_bytes!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../contracts/source-leaf-coverage/v1/prevalence.yaml"
));

#[derive(Debug, Deserialize)]
struct PrevalenceDocument {
    schema_version: String,
    source_contract_version: String,
    source_commit: String,
    source_signature: String,
    registry_sha256: String,
    entries: Vec<PrevalenceEntry>,
}

#[derive(Debug, Clone, Deserialize)]
pub(crate) struct PrevalenceEntry {
    pub entry_id: String,
    pub type_id: String,
    pub selector: SourceLeafSelector,
    pub normalized_path: String,
    pub record_count: usize,
    pub occurrence_count: usize,
}

impl PrevalenceEntry {
    pub(crate) fn identity(&self) -> SourceLeafIdentity {
        SourceLeafIdentity {
            type_id: self.type_id.clone(),
            selector: self.selector.clone(),
            normalized_path: self.normalized_path.clone(),
        }
    }
}

pub(crate) fn accepted_prevalence() -> Result<&'static [PrevalenceEntry], String> {
    static PREVALENCE: OnceLock<Result<Vec<PrevalenceEntry>, String>> = OnceLock::new();
    PREVALENCE
        .get_or_init(|| {
            let digest = format!("{:x}", Sha256::digest(PREVALENCE_BYTES));
            if digest != PF2E_SOURCE_LEAF_PREVALENCE_SHA256 {
                return Err(format!(
                    "accepted leaf prevalence digest mismatch: expected {PF2E_SOURCE_LEAF_PREVALENCE_SHA256}, found {digest}"
                ));
            }
            let document: PrevalenceDocument =
                yaml_serde::from_slice(PREVALENCE_BYTES).map_err(|error| error.to_string())?;
            if document.schema_version != PF2E_SOURCE_LEAF_PREVALENCE_VERSION
                || document.source_contract_version != PF2E_SOURCE_CONTRACT_VERSION
                || document.source_commit != PF2E_SOURCE_PINNED_COMMIT
                || document.source_signature != PF2E_SOURCE_PINNED_SIGNATURE
                || document.registry_sha256 != PF2E_TYPE_REGISTRY_SHA256
            {
                return Err("accepted leaf prevalence metadata is not bound to the source contract, pin, signature, and registry".to_string());
            }
            let registry = accepted_registry()?;
            let mut identities = BTreeSet::new();
            let mut entry_ids = BTreeSet::new();
            for entry in &document.entries {
                if entry.entry_id.trim().is_empty()
                    || !entry_ids.insert(entry.entry_id.as_str())
                    || !identities.insert(entry.identity())
                {
                    return Err("accepted leaf prevalence entries require unique non-empty identities and valid counts".to_string());
                }
                let registry_entry = registry
                    .entries
                    .iter()
                    .find(|registered| {
                        registered.type_id == entry.type_id
                            && registered.document_class == entry.selector.document_class
                            && registered.type_discriminator == entry.selector.type_discriminator
                            && registered.role == entry.selector.role
                            && registered.parent_context == entry.selector.parent_context
                    })
                    .ok_or_else(|| {
                        format!("leaf prevalence entry {} has no registry tuple", entry.entry_id)
                    })?;
                if registry_entry.corpus.count != entry.record_count {
                    return Err(format!(
                        "leaf prevalence entry {} record count {} differs from authenticated registry corpus count {}",
                        entry.entry_id, entry.record_count, registry_entry.corpus.count
                    ));
                }
            }
            Ok(document.entries)
        })
        .as_ref()
        .map(Vec::as_slice)
        .map_err(Clone::clone)
}

#[cfg(test)]
mod tests {
    use super::*;

    const HAZARD_LEDGERS: [&str; 4] = [
        include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../contracts/source-leaf-coverage/v1/actor-hazard.yaml"
        )),
        include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../contracts/source-leaf-coverage/v1/item-action-embedded-hazard.yaml"
        )),
        include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../contracts/source-leaf-coverage/v1/item-melee-embedded-hazard.yaml"
        )),
        include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../contracts/source-leaf-coverage/v1/item-consumable-embedded-hazard.yaml"
        )),
    ];

    const HAZARD_PREVALENCE_TYPE_IDS: [&str; 4] = [
        "actor--hazard--top-level--root--root--root",
        "item--action--embedded--actor--hazard--actor-items",
        "item--melee--embedded--actor--hazard--actor-items",
        "item--consumable--embedded--actor--hazard--actor-items",
    ];

    #[test]
    fn accepted_prevalence_is_digest_pin_registry_and_count_bound() {
        let entries = accepted_prevalence().expect("accepted prevalence");
        let action_name = entries
            .iter()
            .find(|entry| entry.entry_id == "item-action-top-level-name@4cbdaa37")
            .expect("action name prevalence");
        assert_eq!(action_name.record_count, 1_169);
        assert_eq!(action_name.occurrence_count, 1_169);
        let sparse_spell_rules = entries
            .iter()
            .find(|entry| entry.entry_id == "item-spell-top-level-rule-key@4cbdaa37")
            .expect("spell rule prevalence");
        assert_eq!(sparse_spell_rules.record_count, 1_716);
        assert_eq!(sparse_spell_rules.occurrence_count, 14);
    }

    #[test]
    fn h1_bc_hazard_ledgers_exactly_partition_the_authenticated_inventory() {
        let inventory = accepted_prevalence().expect("accepted prevalence");
        let inventory_hazard_identities = inventory
            .iter()
            .filter(|entry| HAZARD_PREVALENCE_TYPE_IDS.contains(&entry.type_id.as_str()))
            .map(PrevalenceEntry::identity)
            .collect::<BTreeSet<_>>();
        let declared_hazard_identities = HAZARD_LEDGERS
            .iter()
            .flat_map(|source| {
                super::super::parse_source_leaf_ledger(source)
                    .expect("hazard ledger parses")
                    .leaves
                    .into_iter()
                    .map(|leaf| leaf.normalized_path)
                    .collect::<Vec<_>>()
            })
            .collect::<Vec<_>>();

        let mut declared = BTreeSet::new();
        for source in HAZARD_LEDGERS {
            let ledger =
                super::super::parse_source_leaf_ledger(source).expect("hazard ledger parses");
            declared.extend(ledger.leaves.iter().map(|leaf| ledger.identity_for(leaf)));
        }

        assert_eq!(declared, inventory_hazard_identities);
        assert_eq!(declared_hazard_identities.len(), declared.len());
    }
}
