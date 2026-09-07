use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Mutex, OnceLock};

use atlas_domain::{DetailLevel, PackName, RecordKey};
use atlas_record::{
    AtlasRecord, ConsumableSpellChild, ContentHash, ContentSourceKind, CreatureRecord,
    CreatureSkillKind, FactValue, HazardActionType, HazardCapability, HazardEntitySourceIdentity,
    HazardRecord, HazardSourceValue as CanonicalHazardSourceValue, OwnedRichContentDocument,
    PublicationLicense, RecordBody, RecordJson, RecordJsonOptions, RecordPresentationJson,
    RichDocument, RichLinkTarget, SpellDefinition, SpellFact, SpellRecord, SpellSourceContext,
    SpellSourceValue, UnsupportedSourceValue, record_json, visit_foundry_links_mut,
};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use sha2::{Digest, Sha256};

use crate::SourcePresence;
use crate::artifact_manifest::{
    ArtifactManifest, ArtifactManifestInput, SourcePositionReport, write_artifact_manifest,
};
use crate::diagnostics::IngestDiagnostics;
use crate::index_build_input::index_build_input;
use crate::records::SourceContentFact;
use crate::records::references::{build_record_reference_index, resolve_content_references};
use crate::source::dto::{
    HazardSourceField, HazardSourceValue as DtoHazardSourceValue, PF2E_SOURCE_CONTRACT_VERSION,
    PF2E_SOURCE_PINNED_COMMIT, PF2E_SOURCE_PINNED_SIGNATURE, SourceIdentity, SpellDocumentSource,
    SpellItemSource, VersionedHazardSource, VersionedItemSource, VersionedNpcSource,
    parse_hazard_source, parse_item_source, parse_npc_source, parse_serialized_source_object,
    parse_spell_document_source, pinned_source_version_metadata,
};
use crate::source::normalize::{
    normalize_record, normalize_record_from_source, parse_foundry_content_with_localization,
};
use crate::source::owned_content::finalize_spell_owned_content;
use crate::source::spells::finalize_consumable_spell_children;
use crate::source::{LoadedPack, ManifestPack, SourceLoad};

use super::spell_receipt_view::{canonical_child_view, canonical_spell_view, source_view};
use super::{
    CoverageContractError, CoverageFailureCode, FinalOwnerStage, FixtureContract,
    FixtureProvenance, PF2E_TYPE_REGISTRY_SHA256, SourceDocumentRole, SourceLeafCoverageLedger,
    SourceLeafIdentity, SourceLeafSelector, SourceParentContextSelector, lint_source_leaf_ledger,
    parse_source_leaf_ledger,
};

const ITEM_NAME_READER: &str = "source::dto::FullItemSource::name";
const NPC_ABILITY_MOD_READER: &str = "source::dto::NpcLegacyAbilitySource::mod";
const NPC_SKILLS_READER: &str = "source::dto::NpcCoreSource::skills";
const HAZARD_DISABLE_READER: &str = "source::dto::HazardLifecycleSource::disable";
const HAZARD_FOLDER_READER: &str = "source::dto::HazardSource::folder";
const HAZARD_TEMP_MAX_READER: &str = "source::dto::HazardHitPointsSource::temporary_maximum";
const HAZARD_ACTION_TYPE_READER: &str = "source::dto::HazardActionSource::action_type";
const HAZARD_DAMAGE_READER: &str = "source::dto::HazardDamageSource::damage";
const HAZARD_EXACT_PROMOTED_READER: &str = "source::dto::HazardSource::exact_promoted_leaf";
const HAZARD_EXACT_UNSUPPORTED_READER: &str =
    "source::dto::HazardSource::exact_typed_unsupported_leaf";
const HAZARD_EXACT_PROVENANCE_READER: &str = "source::dto::HazardSource::exact_provenance_leaf";
const HAZARD_HYDRATION_LEDGER_SOURCES: [&str; 4] = [
    include_str!("../../../../contracts/source-leaf-coverage/v1/actor-hazard.yaml"),
    include_str!("../../../../contracts/source-leaf-coverage/v1/item-action-embedded-hazard.yaml"),
    include_str!("../../../../contracts/source-leaf-coverage/v1/item-melee-embedded-hazard.yaml"),
    include_str!(
        "../../../../contracts/source-leaf-coverage/v1/item-consumable-embedded-hazard.yaml"
    ),
];
const ITEM_SPELL_READER: &str = "source::dto::parse_spell_document_source";
const CONSUMABLE_SPELL_CHILD_READER: &str = "source::dto::ConsumableSpellChildSource";
const SPELL_RAW_PROVENANCE_READER: &str = "AtlasRecord.provenance.raw_json";

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
#[allow(dead_code)]
pub(crate) enum SourceAccessorPurpose {
    Parser,
    ProvenanceReader,
    Inventory,
}

/// Opaque evidence emitted only by the sealed exact-accessor registry.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct SourceLeafReceipt {
    identity: SourceLeafIdentity,
    fixture: FixtureReference,
    source: SourcePresence<SourceLeafValue>,
    reader: ActualReadEvidence,
    observations: Vec<StageObservation>,
    #[serde(skip_serializing_if = "Option::is_none")]
    semantic_output: Option<SemanticOutputObservation>,
    evidence_digest: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SpellArtifactReceiptOperations {
    pub source_file_count: usize,
    pub compatible_sibling_group_count: usize,
    pub isolated_discriminator_group_count: usize,
}

impl SpellArtifactReceiptOperations {
    pub fn sqlite_cycle_count(self) -> usize {
        self.source_file_count
            + self.compatible_sibling_group_count
            + self.isolated_discriminator_group_count
    }
}

impl SourceLeafReceipt {
    pub fn identity(&self) -> &SourceLeafIdentity {
        &self.identity
    }
    pub(crate) fn fixture(&self) -> &FixtureReference {
        &self.fixture
    }
    pub(crate) fn source(&self) -> &SourcePresence<SourceLeafValue> {
        &self.source
    }
    pub(crate) fn integrity_is_valid(&self) -> bool {
        evidence_digest(
            &self.identity,
            &self.fixture,
            &self.source,
            &self.reader,
            &self.observations,
            &self.semantic_output,
        )
        .is_ok_and(|digest| self.evidence_digest == digest)
    }
    pub(crate) fn reader_id(&self) -> &str {
        &self.reader.reader_id
    }
    pub(crate) fn reader_binding(&self) -> &str {
        &self.reader.accessor_binding
    }
    pub(crate) fn reader_purpose(&self) -> SourceAccessorPurpose {
        self.reader.purpose
    }
    pub(crate) fn reader_mutation_digest(&self) -> &str {
        &self.reader.mutation_digest
    }
    pub(crate) fn observations(&self) -> &[StageObservation] {
        &self.observations
    }
    pub(crate) fn semantic_output(&self) -> Option<(bool, bool, &str)> {
        self.semantic_output.as_ref().map(|proof| {
            (
                proof.observed,
                proof.mutation_observed,
                proof.accessor_binding.as_str(),
            )
        })
    }
}

/// Loads a fixture from the pinned Git tree and executes its registered parser
/// and selected real owner stages. Callers supply no bytes, accessor types,
/// owner values, mutation sentinels, or success flags.
///
/// Declaration-equivalent accessor and builder APIs do not exist:
///
/// ```compile_fail
/// use atlas_ingest::{SourceLeafAccessor, SourceLeafReceiptBuilder};
/// ```
///
/// Fixture references cannot be self-authenticated from caller bytes:
///
/// ```compile_fail
/// use atlas_ingest::FixtureReference;
/// ```
///
/// A cloned caller-owned value cannot be submitted as every owner stage:
///
/// ```compile_fail
/// use atlas_ingest::SourceLeafReceipt;
/// struct OwnerValue(serde_json::Value);
/// let same_owner = OwnerValue(serde_json::json!({"value": 4}));
/// let _ = SourceLeafReceipt::capture(same_owner);
/// ```
pub fn capture_registered_source_leaf_receipt(
    ledger: &SourceLeafCoverageLedger,
    leaf_index: usize,
    fixture_index: usize,
    source_repository: &Path,
) -> Result<SourceLeafReceipt, CoverageContractError> {
    if let Some(failure) = lint_source_leaf_ledger(ledger).into_iter().next() {
        return Err(error(failure.code, failure.message));
    }
    let leaf = ledger.leaves.get(leaf_index).ok_or_else(|| {
        error(
            CoverageFailureCode::InvalidContract,
            format!("leaf index {leaf_index} is outside the ledger"),
        )
    })?;
    let fixture = leaf.fixtures.get(fixture_index).ok_or_else(|| {
        error(
            CoverageFailureCode::FixtureNotSourceGrounded,
            format!("fixture index {fixture_index} is outside the leaf"),
        )
    })?;
    let identity = ledger.identity_for(leaf);
    let registration = registration_for(&identity, leaf.reader.reader_id.as_deref())?;
    registration.capture(
        identity.clone(),
        ResolvedFixture::load(fixture, source_repository, &identity.selector)?,
        ledger,
        None,
    )
}

/// Captures one spell-ledger receipt set while sharing one baseline artifact
/// and partitioned compatible or discriminator mutation artifacts per source file.
pub fn capture_registered_spell_source_leaf_receipts(
    ledger: &SourceLeafCoverageLedger,
    source_repository: &Path,
) -> Result<(Vec<SourceLeafReceipt>, SpellArtifactReceiptOperations), CoverageContractError> {
    if let Some(failure) = lint_source_leaf_ledger(ledger).into_iter().next() {
        return Err(error(failure.code, failure.message));
    }
    let fixture_entries = registered_spell_fixture_entries(ledger)?;

    let (artifact_evidence, operations) =
        build_spell_artifact_evidence(ledger, source_repository, &fixture_entries)?;

    let mut receipts = Vec::new();
    for leaf in &ledger.leaves {
        for fixture in &leaf.fixtures {
            let identity = ledger.identity_for(leaf);
            let registration = registration_for(&identity, leaf.reader.reader_id.as_deref())?;
            receipts.push(registration.capture(
                identity.clone(),
                ResolvedFixture::load(fixture, source_repository, &identity.selector)?,
                ledger,
                artifact_evidence.get(&fixture.source_path),
            )?);
        }
    }
    Ok((receipts, operations))
}

fn registered_spell_fixture_entries(
    ledger: &SourceLeafCoverageLedger,
) -> Result<BTreeMap<String, Vec<(usize, usize)>>, CoverageContractError> {
    let mut fixture_entries = BTreeMap::<String, Vec<(usize, usize)>>::new();
    for (leaf_index, leaf) in ledger.leaves.iter().enumerate() {
        for (fixture_index, fixture) in leaf.fixtures.iter().enumerate() {
            let identity = ledger.identity_for(leaf);
            match registration_for(&identity, leaf.reader.reader_id.as_deref())? {
                RegisteredAccessor::ItemSpell(_) | RegisteredAccessor::ConsumableSpellChild(_) => {}
                _ => {
                    return Err(error(
                        CoverageFailureCode::InvalidContract,
                        "grouped spell artifact capture received a non-spell ledger",
                    ));
                }
            }
            fixture_entries
                .entry(fixture.source_path.clone())
                .or_default()
                .push((leaf_index, fixture_index));
        }
    }
    Ok(fixture_entries)
}

fn build_spell_artifact_evidence(
    ledger: &SourceLeafCoverageLedger,
    source_repository: &Path,
    fixture_entries: &BTreeMap<String, Vec<(usize, usize)>>,
) -> Result<
    (
        BTreeMap<String, SpellArtifactHydrationEvidence>,
        SpellArtifactReceiptOperations,
    ),
    CoverageContractError,
> {
    let mut artifact_evidence = BTreeMap::new();
    let mut operations = SpellArtifactReceiptOperations {
        source_file_count: 0,
        compatible_sibling_group_count: 0,
        isolated_discriminator_group_count: 0,
    };
    for (source_path, entries) in fixture_entries {
        let (first_leaf_index, first_fixture_index) = entries[0];
        let first_leaf = &ledger.leaves[first_leaf_index];
        let first_fixture = &first_leaf.fixtures[first_fixture_index];
        let first_identity = ledger.identity_for(first_leaf);
        let fixture =
            ResolvedFixture::load(first_fixture, source_repository, &first_identity.selector)?;
        let mut grouped_entries =
            BTreeMap::<SpellArtifactMutationGroup, Vec<(usize, usize)>>::new();
        for (leaf_index, fixture_index) in entries {
            let leaf = &ledger.leaves[*leaf_index];
            let fixture_contract = &leaf.fixtures[*fixture_index];
            let identity = ledger.identity_for(leaf);
            let registration = registration_for(&identity, leaf.reader.reader_id.as_deref())?;
            if let Some(group) = spell_artifact_mutation_group(
                &identity.normalized_path,
                registration,
                &fixture_contract.case_id,
            ) {
                grouped_entries
                    .entry(group)
                    .or_default()
                    .push((*leaf_index, *fixture_index));
            }
        }
        if grouped_entries.is_empty() {
            return Err(error(
                CoverageFailureCode::ArtifactHydrationMismatch,
                format!("spell artifact fixture `{source_path}` has no compatible mutation"),
            ));
        }
        let (_, baseline_input) =
            run_spell_pipeline(&fixture, fixture.raw.clone(), &fixture.serialized)?;
        let baseline_key = baseline_input.records[0].identity.key.clone();
        let baseline = persist_and_hydrate_spell(&baseline_input, &baseline_key)?;
        operations.source_file_count += 1;

        let mut mutations = BTreeMap::new();
        for (group, group_entries) in grouped_entries {
            let mut mutation_raw = fixture.raw.clone();
            for (leaf_index, fixture_index) in group_entries {
                let leaf = &ledger.leaves[leaf_index];
                let fixture_contract = &leaf.fixtures[fixture_index];
                let identity = ledger.identity_for(leaf);
                let registration = registration_for(&identity, leaf.reader.reader_id.as_deref())?;
                if !mutate_grouped_spell_artifact(
                    &mut mutation_raw,
                    &fixture.raw,
                    &identity.normalized_path,
                    registration,
                    &fixture_contract.case_id,
                )? {
                    return Err(error(
                        CoverageFailureCode::ArtifactHydrationMismatch,
                        format!(
                            "spell artifact fixture `{source_path}` grouped a non-mutation receipt"
                        ),
                    ));
                }
            }
            let mutation_serialized = serde_json::to_vec(&mutation_raw)
                .map_err(|message| error(CoverageFailureCode::ReaderNotObserved, message))?;
            let (_, mutation_input) =
                run_spell_pipeline(&fixture, mutation_raw, &mutation_serialized)?;
            let mutation_key = mutation_input.records[0].identity.key.clone();
            let mutation = persist_and_hydrate_spell(&mutation_input, &mutation_key)?;
            match &group {
                SpellArtifactMutationGroup::CompatibleSiblings => {
                    operations.compatible_sibling_group_count += 1;
                }
                SpellArtifactMutationGroup::IsolatedDiscriminator { .. } => {
                    operations.isolated_discriminator_group_count += 1;
                }
            }
            mutations.insert(group, SpellArtifactHydration::new(mutation));
        }
        artifact_evidence.insert(
            source_path.clone(),
            SpellArtifactHydrationEvidence {
                baseline: SpellArtifactHydration::new(baseline),
                mutations,
            },
        );
    }
    Ok((artifact_evidence, operations))
}

#[derive(Debug, Clone, Copy)]
enum RegisteredAccessor {
    ItemActionName,
    ActorNpcAbilityMod(AbilitySlot),
    ActorNpcShadowSkillBase,
    ActorHazardDisable,
    ActorHazardFolder,
    ActorHazardTemporaryMaximum,
    EmbeddedHazardActionType,
    EmbeddedHazardDamage,
    ActorHazardExact(HazardExactDisposition),
    ItemSpell(SpellLeafProbe),
    ConsumableSpellChild(ConsumableSpellChildLeafProbe),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum HazardExactDisposition {
    Promoted,
    TypedUnsupported,
    ProvenanceOnly,
}

#[derive(Debug, Clone, Copy)]
enum SpellLeafProbe {
    Rank,
    DamageFormula,
    OverlaySort,
    RuleKey,
    StandaloneLocation,
    Image,
    PublicationLicense,
    Mapped,
}

#[derive(Debug, Clone, Copy)]
enum ConsumableSpellChildLeafProbe {
    ChildId,
    StandaloneLocator,
    HeightenedRank,
    LocationValue,
    Image,
    SortProvenance,
    Slug,
    PublicationLicense,
    PublicationRemaster,
    PublicationTitle,
    Rarity,
    Mapped,
}

#[derive(Debug, Clone, Copy)]
enum AbilitySlot {
    Strength,
    Dexterity,
    Constitution,
    Intelligence,
    Wisdom,
    Charisma,
}

fn registration_for(
    identity: &SourceLeafIdentity,
    reader_id: Option<&str>,
) -> Result<RegisteredAccessor, CoverageContractError> {
    if identity.type_id == "item--action--top-level--root--root--root"
        && identity.selector.document_class == "Item"
        && identity.selector.type_discriminator == "action"
        && identity.normalized_path == "$.name"
        && reader_id == Some(ITEM_NAME_READER)
    {
        Ok(RegisteredAccessor::ItemActionName)
    } else if identity.type_id == "actor--npc--top-level--root--root--root"
        && identity.selector.document_class == "Actor"
        && identity.selector.type_discriminator == "npc"
        && reader_id == Some(NPC_ABILITY_MOD_READER)
        && let Some(slot) = ability_slot(&identity.normalized_path)
    {
        Ok(RegisteredAccessor::ActorNpcAbilityMod(slot))
    } else if identity.type_id == "actor--npc--top-level--root--root--root"
        && identity.selector.document_class == "Actor"
        && identity.selector.type_discriminator == "npc"
        && identity.normalized_path == "$.system.skills.*.base"
        && reader_id == Some(NPC_SKILLS_READER)
    {
        Ok(RegisteredAccessor::ActorNpcShadowSkillBase)
    } else if identity.type_id == "actor--hazard--top-level--root--root--root"
        && identity.normalized_path == "$.system.details.disable"
        && reader_id == Some(HAZARD_DISABLE_READER)
    {
        Ok(RegisteredAccessor::ActorHazardDisable)
    } else if identity.type_id == "actor--hazard--top-level--root--root--root"
        && identity.normalized_path == "$.folder"
        && reader_id == Some(HAZARD_FOLDER_READER)
    {
        Ok(RegisteredAccessor::ActorHazardFolder)
    } else if identity.type_id == "actor--hazard--top-level--root--root--root"
        && identity.normalized_path == "$.system.attributes.hp.tempmax"
        && reader_id == Some(HAZARD_TEMP_MAX_READER)
    {
        Ok(RegisteredAccessor::ActorHazardTemporaryMaximum)
    } else if identity.type_id == "item--action--embedded--actor--hazard--actor-items"
        && identity.normalized_path == "$.system.actionType.value"
        && reader_id == Some(HAZARD_ACTION_TYPE_READER)
    {
        Ok(RegisteredAccessor::EmbeddedHazardActionType)
    } else if identity.type_id == "item--melee--embedded--actor--hazard--actor-items"
        && identity.normalized_path == "$.system.damageRolls.*.damage"
        && reader_id == Some(HAZARD_DAMAGE_READER)
    {
        Ok(RegisteredAccessor::EmbeddedHazardDamage)
    } else if is_hazard_exact_selector(&identity.selector)
        && reader_id == Some(HAZARD_EXACT_PROMOTED_READER)
    {
        Ok(RegisteredAccessor::ActorHazardExact(
            HazardExactDisposition::Promoted,
        ))
    } else if is_hazard_exact_selector(&identity.selector)
        && reader_id == Some(HAZARD_EXACT_UNSUPPORTED_READER)
    {
        Ok(RegisteredAccessor::ActorHazardExact(
            HazardExactDisposition::TypedUnsupported,
        ))
    } else if is_hazard_exact_selector(&identity.selector)
        && reader_id == Some(HAZARD_EXACT_PROVENANCE_READER)
    {
        Ok(RegisteredAccessor::ActorHazardExact(
            HazardExactDisposition::ProvenanceOnly,
        ))
    } else if identity.type_id == "item--spell--top-level--root--root--root"
        && identity.selector.document_class == "Item"
        && identity.selector.type_discriminator == "spell"
        && identity.selector.role == SourceDocumentRole::TopLevel
        && let Some(probe) = spell_leaf_probe(&identity.normalized_path, reader_id)
    {
        Ok(RegisteredAccessor::ItemSpell(probe))
    } else if identity.type_id == "item--spell--child--item--consumable--consumable-system-spell"
        && identity.selector.document_class == "Item"
        && identity.selector.type_discriminator == "spell"
        && identity.selector.role == SourceDocumentRole::Child
        && let Some(probe) = consumable_spell_child_leaf_probe(&identity.normalized_path, reader_id)
    {
        Ok(RegisteredAccessor::ConsumableSpellChild(probe))
    } else {
        Err(error(
            CoverageFailureCode::ReaderNotObserved,
            format!(
                "no sealed semantic accessor is registered for {} {}",
                identity.normalized_path,
                reader_id.unwrap_or("<missing reader>")
            ),
        ))
    }
}

fn is_hazard_exact_selector(selector: &SourceLeafSelector) -> bool {
    (selector.document_class == "Actor"
        && selector.type_discriminator == "hazard"
        && selector.role == SourceDocumentRole::TopLevel)
        || (selector.document_class == "Item"
            && selector.role == SourceDocumentRole::Embedded
            && selector.parent_context.document_class.as_deref() == Some("Actor")
            && selector.parent_context.type_discriminator.as_deref() == Some("hazard")
            && selector.parent_context.relationship_path.as_deref() == Some("Actor.items"))
}

impl RegisteredAccessor {
    fn capture(
        self,
        identity: SourceLeafIdentity,
        fixture: ResolvedFixture,
        ledger: &SourceLeafCoverageLedger,
        artifact_evidence: Option<&SpellArtifactHydrationEvidence>,
    ) -> Result<SourceLeafReceipt, CoverageContractError> {
        match self {
            Self::ItemActionName => capture_item_name(identity, fixture),
            Self::ActorNpcAbilityMod(slot) => capture_actor_npc_ability(identity, fixture, slot),
            Self::ActorNpcShadowSkillBase => capture_actor_npc_shadow_skill(identity, fixture),
            Self::ActorHazardDisable => capture_hazard_disable(identity, fixture, ledger),
            Self::ActorHazardFolder => capture_hazard_folder(identity, fixture),
            Self::ActorHazardTemporaryMaximum => {
                capture_hazard_temporary_maximum(identity, fixture, ledger)
            }
            Self::EmbeddedHazardActionType => capture_hazard_action_type(identity, fixture, ledger),
            Self::EmbeddedHazardDamage => capture_hazard_damage(identity, fixture, ledger),
            Self::ActorHazardExact(disposition) => {
                capture_hazard_exact_leaf(identity, fixture, disposition, ledger)
            }
            Self::ItemSpell(probe) => {
                capture_item_spell_leaf(identity, fixture, probe, artifact_evidence)
            }
            Self::ConsumableSpellChild(probe) => {
                capture_consumable_spell_child_leaf(identity, fixture, probe, artifact_evidence)
            }
        }
    }
}

fn spell_leaf_probe(path: &str, reader_id: Option<&str>) -> Option<SpellLeafProbe> {
    match (path, reader_id) {
        ("$.system.level.value", Some(ITEM_SPELL_READER)) => Some(SpellLeafProbe::Rank),
        ("$.system.damage.*.formula", Some(ITEM_SPELL_READER)) => {
            Some(SpellLeafProbe::DamageFormula)
        }
        ("$.system.overlays.*.sort", Some(ITEM_SPELL_READER)) => Some(SpellLeafProbe::OverlaySort),
        ("$.system.rules[].key", Some(ITEM_SPELL_READER)) => Some(SpellLeafProbe::RuleKey),
        ("$.system.location", Some(ITEM_SPELL_READER)) => Some(SpellLeafProbe::StandaloneLocation),
        ("$.img", Some(ITEM_SPELL_READER)) => Some(SpellLeafProbe::Image),
        ("$.system.publication.license", Some(ITEM_SPELL_READER)) => {
            Some(SpellLeafProbe::PublicationLicense)
        }
        (_, Some(ITEM_SPELL_READER)) => Some(SpellLeafProbe::Mapped),
        _ => None,
    }
}

fn consumable_spell_child_leaf_probe(
    path: &str,
    reader_id: Option<&str>,
) -> Option<ConsumableSpellChildLeafProbe> {
    match (path, reader_id) {
        ("$._id", Some(CONSUMABLE_SPELL_CHILD_READER)) => {
            Some(ConsumableSpellChildLeafProbe::ChildId)
        }
        ("$.flags.core.sourceId", Some(CONSUMABLE_SPELL_CHILD_READER)) => {
            Some(ConsumableSpellChildLeafProbe::StandaloneLocator)
        }
        ("$.system.location.heightenedLevel", Some(CONSUMABLE_SPELL_CHILD_READER)) => {
            Some(ConsumableSpellChildLeafProbe::HeightenedRank)
        }
        ("$.system.location.value", Some(CONSUMABLE_SPELL_CHILD_READER)) => {
            Some(ConsumableSpellChildLeafProbe::LocationValue)
        }
        ("$.img", Some(CONSUMABLE_SPELL_CHILD_READER)) => {
            Some(ConsumableSpellChildLeafProbe::Image)
        }
        ("$.sort", Some(SPELL_RAW_PROVENANCE_READER)) => {
            Some(ConsumableSpellChildLeafProbe::SortProvenance)
        }
        ("$.system.slug", Some(CONSUMABLE_SPELL_CHILD_READER)) => {
            Some(ConsumableSpellChildLeafProbe::Slug)
        }
        ("$.system.publication.license", Some(CONSUMABLE_SPELL_CHILD_READER)) => {
            Some(ConsumableSpellChildLeafProbe::PublicationLicense)
        }
        ("$.system.publication.remaster", Some(CONSUMABLE_SPELL_CHILD_READER)) => {
            Some(ConsumableSpellChildLeafProbe::PublicationRemaster)
        }
        ("$.system.publication.title", Some(CONSUMABLE_SPELL_CHILD_READER)) => {
            Some(ConsumableSpellChildLeafProbe::PublicationTitle)
        }
        ("$.system.traits.rarity", Some(CONSUMABLE_SPELL_CHILD_READER)) => {
            Some(ConsumableSpellChildLeafProbe::Rarity)
        }
        (_, Some(CONSUMABLE_SPELL_CHILD_READER)) => Some(ConsumableSpellChildLeafProbe::Mapped),
        _ => None,
    }
}

fn ability_slot(path: &str) -> Option<AbilitySlot> {
    match path {
        "$.system.abilities.str.mod" => Some(AbilitySlot::Strength),
        "$.system.abilities.dex.mod" => Some(AbilitySlot::Dexterity),
        "$.system.abilities.con.mod" => Some(AbilitySlot::Constitution),
        "$.system.abilities.int.mod" => Some(AbilitySlot::Intelligence),
        "$.system.abilities.wis.mod" => Some(AbilitySlot::Wisdom),
        "$.system.abilities.cha.mod" => Some(AbilitySlot::Charisma),
        _ => None,
    }
}

fn capture_item_name(
    identity: SourceLeafIdentity,
    fixture: ResolvedFixture,
) -> Result<SourceLeafReceipt, CoverageContractError> {
    let source_name = fixture
        .raw
        .get("name")
        .and_then(Value::as_str)
        .ok_or_else(|| {
            error(
                CoverageFailureCode::ReaderNotObserved,
                "$.name is not a string",
            )
        })?;
    let excerpt =
        serde_json::to_vec(&Value::String(source_name.to_string())).map_err(|source_error| {
            error(
                CoverageFailureCode::ReceiptProvenanceInvalid,
                format!("failed to serialize the registered $.name excerpt: {source_error}"),
            )
        })?;
    if digest_bytes(&excerpt) != fixture.reference.excerpt_digest {
        return Err(error(
            CoverageFailureCode::ReceiptProvenanceInvalid,
            "registered $.name excerpt does not match its declaration digest",
        ));
    }
    let actual = run_item_name_pipeline(&fixture, fixture.raw.clone())?;
    let mut mutated_raw = fixture.raw.clone();
    mutated_raw["name"] = Value::String(format!("{source_name}\u{241f}source-leaf-mutation"));
    let mutation = run_item_name_pipeline(&fixture, mutated_raw)?;
    if actual == mutation {
        return Err(error(
            CoverageFailureCode::ReaderNotObserved,
            "registered pipeline did not observe its internal source mutation",
        ));
    }

    let source = string_leaf(actual.source_dto.clone());
    let observations = vec![
        stage(
            FinalOwnerStage::SourceDto,
            "FullItemSource.name",
            "source::dto::parse_item_source",
            actual.source_dto,
            &mutation.source_dto,
        )?,
        stage(
            FinalOwnerStage::Canonical,
            "AtlasRecord.identity.name",
            "source::normalize::normalize_record",
            actual.canonical,
            &mutation.canonical,
        )?,
        stage(
            FinalOwnerStage::PostProjection,
            "IndexBuildInput.records[].identity.name",
            "index_build_input::index_build_input",
            actual.post_projection,
            &mutation.post_projection,
        )?,
        stage(
            FinalOwnerStage::ArtifactHydration,
            "RetrievedRecord.record.identity.name",
            "atlas_index::hydrate_record_parts",
            actual.hydration,
            &mutation.hydration,
        )?,
        stage(
            FinalOwnerStage::PublicSurface,
            "RecordJson.name",
            "atlas_record::record_json",
            actual.public_surface,
            &mutation.public_surface,
        )?,
    ];
    let reader = ActualReadEvidence {
        reader_id: ITEM_NAME_READER.to_string(),
        accessor_binding: "sealed::ItemActionName".to_string(),
        purpose: SourceAccessorPurpose::Parser,
        mutation_digest: digest_serializable(&string_leaf(mutation.source_dto))?,
    };
    let semantic_output = None;
    let evidence_digest = evidence_digest(
        &identity,
        &fixture.reference,
        &source,
        &reader,
        &observations,
        &semantic_output,
    )?;
    Ok(SourceLeafReceipt {
        identity,
        fixture: fixture.reference,
        source,
        reader,
        observations,
        semantic_output,
        evidence_digest,
    })
}

fn capture_actor_npc_ability(
    identity: SourceLeafIdentity,
    fixture: ResolvedFixture,
    slot: AbilitySlot,
) -> Result<SourceLeafReceipt, CoverageContractError> {
    validate_actor_excerpt(&fixture)?;
    let source = ability_mod_from_raw(&fixture.raw, slot)?;
    let mut mutated_raw = fixture.raw.clone();
    let mutation_value = match mutated_raw.pointer(slot.mod_pointer()) {
        Some(Value::Number(number)) => Value::from(number.as_i64().unwrap_or_default() + 101),
        Some(Value::Null) => Value::from(101),
        _ => {
            return Err(error(
                CoverageFailureCode::ReaderNotObserved,
                format!("{} is not a number or explicit null", slot.mod_pointer()),
            ));
        }
    };
    let Some(mutated_slot) = mutated_raw.pointer_mut(slot.mod_pointer()) else {
        return Err(error(
            CoverageFailureCode::ReaderNotObserved,
            format!("{} disappeared before mutation", slot.mod_pointer()),
        ));
    };
    *mutated_slot = mutation_value;
    let source_mutation = ability_mod_from_raw(&mutated_raw, slot)?;
    let actual = run_npc_pipeline(&fixture, fixture.raw.clone())?;
    let mutation = run_npc_pipeline(&fixture, mutated_raw)?;
    let canonical_destination =
        format!("CreatureRecord.legacy_abilities.{}", slot.canonical_name());
    let post_destination = format!(
        "IndexBuildInput.canonical_bodies[].legacy_abilities.{}",
        slot.canonical_name()
    );
    let hydration_destination = format!(
        "RetrievedRecord.body.legacy_abilities.{}",
        slot.canonical_name()
    );
    let observations = vec![
        presence_stage(
            FinalOwnerStage::SourceDto,
            "NpcLegacyAbilitySource.mod",
            "source::dto::parse_npc_source",
            dto_ability_value(&actual.source_dto, slot),
            &dto_ability_value(&mutation.source_dto, slot),
        )?,
        presence_stage(
            FinalOwnerStage::Canonical,
            &canonical_destination,
            "source::npc_core::convert_npc_core",
            creature_ability_value(&actual.canonical, slot),
            &creature_ability_value(&mutation.canonical, slot),
        )?,
        presence_stage(
            FinalOwnerStage::PostProjection,
            &post_destination,
            "index_build_input::index_build_input",
            creature_ability_value(&actual.post_projection, slot),
            &creature_ability_value(&mutation.post_projection, slot),
        )?,
        presence_stage(
            FinalOwnerStage::ArtifactHydration,
            &hydration_destination,
            "atlas_index::hydrate_record_parts",
            creature_ability_value(&actual.hydration, slot),
            &creature_ability_value(&mutation.hydration, slot),
        )?,
    ];
    sealed_receipt(
        identity,
        fixture.reference,
        source,
        source_mutation,
        NPC_ABILITY_MOD_READER,
        "sealed::ActorNpcAbilityModProbe",
        observations,
    )
}

fn capture_actor_npc_shadow_skill(
    identity: SourceLeafIdentity,
    fixture: ResolvedFixture,
) -> Result<SourceLeafReceipt, CoverageContractError> {
    validate_actor_excerpt(&fixture)?;
    let skill_key = unsupported_skill_key(&fixture.raw)?;
    let source = shadow_skill_from_raw(&fixture.raw, &skill_key)?;
    let mut mutated_raw = fixture.raw.clone();
    let base_pointer = format!("/system/skills/{}/base", escape_json_pointer(&skill_key));
    let mutation_value = match mutated_raw.pointer(&base_pointer) {
        Some(Value::Number(number)) => Value::from(number.as_i64().unwrap_or_default() + 101),
        Some(Value::Null) => Value::from(101),
        _ => {
            return Err(error(
                CoverageFailureCode::ReaderNotObserved,
                format!("shadow skill {skill_key:?} has no number or explicit null base"),
            ));
        }
    };
    let Some(mutated_base) = mutated_raw.pointer_mut(&base_pointer) else {
        return Err(error(
            CoverageFailureCode::ReaderNotObserved,
            format!("shadow skill {skill_key:?} base disappeared before mutation"),
        ));
    };
    *mutated_base = mutation_value;
    let source_mutation = shadow_skill_from_raw(&mutated_raw, &skill_key)?;
    let actual = run_npc_pipeline(&fixture, fixture.raw.clone())?;
    let mutation = run_npc_pipeline(&fixture, mutated_raw)?;
    let observations = vec![
        presence_stage(
            FinalOwnerStage::SourceDto,
            "NpcCoreSource.skills[*].base",
            "source::dto::parse_npc_source",
            dto_shadow_skill_value(&actual.source_dto, &skill_key),
            &dto_shadow_skill_value(&mutation.source_dto, &skill_key),
        )?,
        presence_stage(
            FinalOwnerStage::Canonical,
            "CreatureRecord.skills[*].source_entries[*].modifier",
            "source::npc_core::convert_npc_core",
            creature_skill_value(&actual.canonical, &skill_key),
            &creature_skill_value(&mutation.canonical, &skill_key),
        )?,
        presence_stage(
            FinalOwnerStage::PostProjection,
            "IndexBuildInput.canonical_bodies[].skills[*].source_entries[*].modifier",
            "index_build_input::index_build_input",
            creature_skill_value(&actual.post_projection, &skill_key),
            &creature_skill_value(&mutation.post_projection, &skill_key),
        )?,
        presence_stage(
            FinalOwnerStage::ArtifactHydration,
            "RetrievedRecord.body.skills[*].source_entries[*].modifier",
            "atlas_index::hydrate_record_parts",
            creature_skill_value(&actual.hydration, &skill_key),
            &creature_skill_value(&mutation.hydration, &skill_key),
        )?,
        presence_stage(
            FinalOwnerStage::PublicSurface,
            "RecordPresentationJson.creature.skills[*].source_entries[*].modifier",
            "atlas_record::record_json",
            public_skill_value(&actual.public_surface, &skill_key),
            &public_skill_value(&mutation.public_surface, &skill_key),
        )?,
    ];
    sealed_receipt(
        identity,
        fixture.reference,
        source,
        source_mutation,
        NPC_SKILLS_READER,
        "sealed::ActorNpcShadowSkillBase",
        observations,
    )
}

fn capture_item_spell_leaf(
    identity: SourceLeafIdentity,
    fixture: ResolvedFixture,
    probe: SpellLeafProbe,
    artifact_evidence: Option<&SpellArtifactHydrationEvidence>,
) -> Result<SourceLeafReceipt, CoverageContractError> {
    if matches!(probe, SpellLeafProbe::Mapped) {
        return capture_mapped_spell_leaf(identity, fixture, false, artifact_evidence);
    }
    let source = raw_spell_leaf(&fixture, probe)?;
    validate_spell_excerpt(&fixture, &source)?;
    let mut mutated_raw = fixture.raw.clone();
    mutate_spell_leaf(&mut mutated_raw, false, probe, fixture.reference.case_id())?;
    let source_mutation =
        raw_spell_leaf_from_root(&mutated_raw, false, probe, fixture.reference.case_id())?;
    let (mut actual, actual_input) =
        run_spell_pipeline(&fixture, fixture.raw.clone(), &fixture.serialized)?;
    let mutation_serialized = serde_json::to_vec(&mutated_raw)
        .map_err(|message| error(CoverageFailureCode::ReaderNotObserved, message))?;
    let (mut mutation, mutation_input) =
        run_spell_pipeline(&fixture, mutated_raw, &mutation_serialized)?;
    let artifact_group = required_spell_artifact_mutation_group(
        &identity.normalized_path,
        RegisteredAccessor::ItemSpell(probe),
        fixture.reference.case_id(),
    )?;
    apply_spell_artifact_pair(
        &mut actual,
        &actual_input,
        &mut mutation,
        &mutation_input,
        artifact_evidence,
        &artifact_group,
    )?;

    let observations = vec![
        presence_stage(
            FinalOwnerStage::SourceDto,
            spell_source_destination(probe)?,
            "source::dto::parse_spell_document_source",
            spell_source_leaf(&actual.source, probe, fixture.reference.case_id())?,
            &spell_source_leaf(&mutation.source, probe, fixture.reference.case_id())?,
        )?,
        presence_stage(
            FinalOwnerStage::Canonical,
            spell_canonical_destination(probe)?,
            "source::spells::convert_standalone_spell",
            spell_definition_leaf(
                &actual
                    .canonical
                    .as_ref()
                    .ok_or_else(|| {
                        error(CoverageFailureCode::CanonicalMismatch, "missing spell body")
                    })?
                    .definition,
                probe,
                fixture.reference.case_id(),
            )?,
            &spell_definition_leaf(
                &mutation
                    .canonical
                    .as_ref()
                    .ok_or_else(|| {
                        error(
                            CoverageFailureCode::CanonicalMismatch,
                            "missing mutated spell body",
                        )
                    })?
                    .definition,
                probe,
                fixture.reference.case_id(),
            )?,
        )?,
        presence_stage(
            FinalOwnerStage::PostProjection,
            spell_post_destination(probe)?,
            "index_build_input::index_build_input",
            spell_definition_leaf(
                &actual
                    .post_projection
                    .as_ref()
                    .ok_or_else(|| {
                        error(
                            CoverageFailureCode::PostProjectionMismatch,
                            "missing projected spell body",
                        )
                    })?
                    .definition,
                probe,
                fixture.reference.case_id(),
            )?,
            &spell_definition_leaf(
                &mutation
                    .post_projection
                    .as_ref()
                    .ok_or_else(|| {
                        error(
                            CoverageFailureCode::PostProjectionMismatch,
                            "missing mutated projected spell body",
                        )
                    })?
                    .definition,
                probe,
                fixture.reference.case_id(),
            )?,
        )?,
        presence_stage(
            FinalOwnerStage::ArtifactHydration,
            &format!("ArtifactHydration::{}", spell_canonical_destination(probe)?),
            "atlas_index::hydrate_record_parts",
            spell_definition_leaf(
                &actual
                    .hydration
                    .as_ref()
                    .ok_or_else(|| {
                        error(
                            CoverageFailureCode::ArtifactHydrationMismatch,
                            "missing hydrated spell body",
                        )
                    })?
                    .definition,
                probe,
                fixture.reference.case_id(),
            )?,
            &spell_definition_leaf(
                &mutation
                    .hydration
                    .as_ref()
                    .ok_or_else(|| {
                        error(
                            CoverageFailureCode::ArtifactHydrationMismatch,
                            "missing mutated hydrated spell body",
                        )
                    })?
                    .definition,
                probe,
                fixture.reference.case_id(),
            )?,
        )?,
    ];
    sealed_receipt(
        identity,
        fixture.reference,
        source,
        source_mutation,
        ITEM_SPELL_READER,
        "sealed::ItemSpellLeaf",
        observations,
    )
}

fn capture_consumable_spell_child_leaf(
    identity: SourceLeafIdentity,
    fixture: ResolvedFixture,
    probe: ConsumableSpellChildLeafProbe,
    artifact_evidence: Option<&SpellArtifactHydrationEvidence>,
) -> Result<SourceLeafReceipt, CoverageContractError> {
    if matches!(probe, ConsumableSpellChildLeafProbe::Mapped) {
        return capture_mapped_spell_leaf(identity, fixture, true, artifact_evidence);
    }
    let source = raw_consumable_spell_child_leaf(&fixture, probe)?;
    validate_spell_excerpt(&fixture, &source)?;
    let mut mutated_raw = fixture.raw.clone();
    mutate_consumable_spell_child_leaf(&mut mutated_raw, probe, fixture.reference.case_id())?;
    let source_mutation = raw_consumable_spell_child_leaf_from_root(
        &mutated_raw,
        probe,
        fixture.reference.case_id(),
    )?;
    let (mut actual, actual_input) =
        run_spell_pipeline(&fixture, fixture.raw.clone(), &fixture.serialized)?;
    let mutation_serialized = serde_json::to_vec(&mutated_raw)
        .map_err(|message| error(CoverageFailureCode::ReaderNotObserved, message))?;
    let (mut mutation, mutation_input) =
        run_spell_pipeline(&fixture, mutated_raw, &mutation_serialized)?;

    if matches!(probe, ConsumableSpellChildLeafProbe::SortProvenance) {
        let observations = vec![presence_stage(
            FinalOwnerStage::DurableProvenance,
            "AtlasRecord.provenance.raw_json#/system/spell",
            "sealed::ConsumableSpellChildRawProvenance",
            raw_provenance_consumable_spell_child_leaf(
                &actual.raw_provenance,
                probe,
                fixture.reference.case_id(),
            )?,
            &raw_provenance_consumable_spell_child_leaf(
                &mutation.raw_provenance,
                probe,
                fixture.reference.case_id(),
            )?,
        )?];
        return provenance_receipt(
            identity,
            fixture.reference,
            source,
            source_mutation,
            "sealed::ConsumableSpellChildRawProvenance",
            observations,
        );
    }

    let artifact_group = required_spell_artifact_mutation_group(
        &identity.normalized_path,
        RegisteredAccessor::ConsumableSpellChild(probe),
        fixture.reference.case_id(),
    )?;
    apply_spell_artifact_pair(
        &mut actual,
        &actual_input,
        &mut mutation,
        &mutation_input,
        artifact_evidence,
        &artifact_group,
    )?;

    let actual_child = actual.child.as_ref().ok_or_else(|| {
        error(
            CoverageFailureCode::CanonicalMismatch,
            "missing canonical consumable spell child",
        )
    })?;
    let mutation_child = mutation.child.as_ref().ok_or_else(|| {
        error(
            CoverageFailureCode::CanonicalMismatch,
            "missing mutated canonical consumable spell child",
        )
    })?;
    let observations = vec![
        presence_stage(
            FinalOwnerStage::SourceDto,
            consumable_spell_child_source_destination(probe)?,
            "source::dto::parse_spell_document_source",
            consumable_spell_child_source_leaf(&actual.source, probe, fixture.reference.case_id())?,
            &consumable_spell_child_source_leaf(
                &mutation.source,
                probe,
                fixture.reference.case_id(),
            )?,
        )?,
        presence_stage(
            FinalOwnerStage::Canonical,
            consumable_spell_child_canonical_destination(probe)?,
            "source::spells::finalize_consumable_spell_children",
            consumable_spell_child_canonical_leaf(
                actual_child,
                probe,
                fixture.reference.case_id(),
            )?,
            &consumable_spell_child_canonical_leaf(
                mutation_child,
                probe,
                fixture.reference.case_id(),
            )?,
        )?,
        presence_stage(
            FinalOwnerStage::PostProjection,
            consumable_spell_child_post_destination(probe)?,
            "records::references::resolve_content_references",
            consumable_spell_child_canonical_leaf(
                actual.post_child.as_ref().ok_or_else(|| {
                    error(
                        CoverageFailureCode::PostProjectionMismatch,
                        "missing projected consumable spell child",
                    )
                })?,
                probe,
                fixture.reference.case_id(),
            )?,
            &consumable_spell_child_canonical_leaf(
                mutation.post_child.as_ref().ok_or_else(|| {
                    error(
                        CoverageFailureCode::PostProjectionMismatch,
                        "missing mutated projected consumable spell child",
                    )
                })?,
                probe,
                fixture.reference.case_id(),
            )?,
        )?,
        presence_stage(
            FinalOwnerStage::ArtifactHydration,
            &format!(
                "ArtifactHydration::{}",
                consumable_spell_child_canonical_destination(probe)?
            ),
            "atlas_index::hydrate_record_parts",
            consumable_spell_child_canonical_leaf(
                actual.hydrated_child.as_ref().ok_or_else(|| {
                    error(
                        CoverageFailureCode::ArtifactHydrationMismatch,
                        "missing hydrated consumable spell child",
                    )
                })?,
                probe,
                fixture.reference.case_id(),
            )?,
            &consumable_spell_child_canonical_leaf(
                mutation.hydrated_child.as_ref().ok_or_else(|| {
                    error(
                        CoverageFailureCode::ArtifactHydrationMismatch,
                        "missing mutated hydrated consumable spell child",
                    )
                })?,
                probe,
                fixture.reference.case_id(),
            )?,
        )?,
    ];
    sealed_receipt(
        identity,
        fixture.reference,
        source,
        source_mutation,
        CONSUMABLE_SPELL_CHILD_READER,
        "sealed::ConsumableSpellChildLeaf",
        observations,
    )
}

fn capture_mapped_spell_leaf(
    identity: SourceLeafIdentity,
    fixture: ResolvedFixture,
    child: bool,
    artifact_evidence: Option<&SpellArtifactHydrationEvidence>,
) -> Result<SourceLeafReceipt, CoverageContractError> {
    let raw_spell = if child {
        fixture.raw.pointer("/system/spell").ok_or_else(|| {
            error(
                CoverageFailureCode::ReaderNotObserved,
                "missing spell child",
            )
        })?
    } else {
        &fixture.raw
    };
    let selected = select_pattern_leaf(raw_spell, &identity.normalized_path)?;
    validate_spell_excerpt(&fixture, &selected.source)?;
    if identity.normalized_path == "$.type" {
        return capture_mapped_spell_type_leaf(
            identity,
            fixture,
            child,
            selected,
            artifact_evidence,
        );
    }

    let (mut actual, actual_input) =
        run_spell_pipeline(&fixture, fixture.raw.clone(), &fixture.serialized)?;
    let actual_source = mapped_spell_stage_leaf(
        &actual,
        &selected,
        &identity.normalized_path,
        child,
        MappedSpellStage::Source,
    )?;
    let actual_canonical = mapped_spell_stage_leaf(
        &actual,
        &selected,
        &identity.normalized_path,
        child,
        MappedSpellStage::Canonical,
    )?;
    let actual_post = mapped_spell_stage_leaf(
        &actual,
        &selected,
        &identity.normalized_path,
        child,
        MappedSpellStage::PostProjection,
    )?;
    let mut mutated_raw = fixture.raw.clone();
    let mutation_root = if child {
        mutated_raw.pointer_mut("/system/spell").ok_or_else(|| {
            error(
                CoverageFailureCode::ReaderNotObserved,
                "missing mutable spell child",
            )
        })?
    } else {
        &mut mutated_raw
    };
    mutate_selected_pattern_leaf(mutation_root, &selected, &identity.normalized_path)?;
    let mutated_selected = select_pattern_leaf(mutation_root, &identity.normalized_path)?;
    let mutation_serialized = serde_json::to_vec(&mutated_raw)
        .map_err(|message| error(CoverageFailureCode::ReaderNotObserved, message))?;
    let (mut mutation, mutation_input) =
        run_spell_pipeline(&fixture, mutated_raw, &mutation_serialized)?;
    let registration = if child {
        RegisteredAccessor::ConsumableSpellChild(ConsumableSpellChildLeafProbe::Mapped)
    } else {
        RegisteredAccessor::ItemSpell(SpellLeafProbe::Mapped)
    };
    let artifact_group = required_spell_artifact_mutation_group(
        &identity.normalized_path,
        registration,
        fixture.reference.case_id(),
    )?;
    apply_spell_artifact_pair(
        &mut actual,
        &actual_input,
        &mut mutation,
        &mutation_input,
        artifact_evidence,
        &artifact_group,
    )?;
    let actual_hydration = mapped_spell_stage_leaf(
        &actual,
        &selected,
        &identity.normalized_path,
        child,
        MappedSpellStage::ArtifactHydration,
    )?;
    let mutation_source = mapped_spell_stage_leaf(
        &mutation,
        &mutated_selected,
        &identity.normalized_path,
        child,
        MappedSpellStage::Source,
    )?;
    let mutation_canonical = mapped_spell_stage_leaf(
        &mutation,
        &mutated_selected,
        &identity.normalized_path,
        child,
        MappedSpellStage::Canonical,
    )?;
    let mutation_post = mapped_spell_stage_leaf(
        &mutation,
        &mutated_selected,
        &identity.normalized_path,
        child,
        MappedSpellStage::PostProjection,
    )?;
    let mutation_hydration = mapped_spell_stage_leaf(
        &mutation,
        &mutated_selected,
        &identity.normalized_path,
        child,
        MappedSpellStage::ArtifactHydration,
    )?;
    let content_equivalence = is_spell_content_path(&identity.normalized_path);
    let receipt_source = if content_equivalence {
        actual_source.clone()
    } else {
        selected.source
    };
    let receipt_mutation = if content_equivalence {
        mutation_source.clone()
    } else {
        mutated_selected.source
    };

    let (source_destination, canonical_destination, post_destination) =
        mapped_spell_destinations(&identity.normalized_path, child);
    let hydration_destination = format!("ArtifactHydration::{canonical_destination}");
    let observations = vec![
        presence_stage(
            FinalOwnerStage::SourceDto,
            &source_destination,
            "source_coverage::spell_receipt_view::source_view",
            actual_source,
            &mutation_source,
        )?,
        presence_stage(
            FinalOwnerStage::Canonical,
            &canonical_destination,
            "source_coverage::spell_receipt_view::canonical_view",
            actual_canonical,
            &mutation_canonical,
        )?,
        presence_stage(
            FinalOwnerStage::PostProjection,
            &post_destination,
            "index_build_input::index_build_input",
            actual_post,
            &mutation_post,
        )?,
        presence_stage(
            FinalOwnerStage::ArtifactHydration,
            &hydration_destination,
            "atlas_index::hydrate_record_parts",
            actual_hydration,
            &mutation_hydration,
        )?,
    ];
    sealed_receipt(
        identity,
        fixture.reference,
        receipt_source,
        receipt_mutation,
        if child {
            CONSUMABLE_SPELL_CHILD_READER
        } else {
            ITEM_SPELL_READER
        },
        "sealed::MappedSpellLeaf",
        observations,
    )
}

fn capture_mapped_spell_type_leaf(
    identity: SourceLeafIdentity,
    fixture: ResolvedFixture,
    child: bool,
    selected: SelectedPatternLeaf,
    artifact_evidence: Option<&SpellArtifactHydrationEvidence>,
) -> Result<SourceLeafReceipt, CoverageContractError> {
    let (mut actual, actual_input) =
        run_spell_pipeline(&fixture, fixture.raw.clone(), &fixture.serialized)?;
    apply_spell_artifact_baseline(&mut actual, &actual_input, artifact_evidence)?;
    let actual_source = mapped_spell_stage_leaf(
        &actual,
        &selected,
        &identity.normalized_path,
        child,
        MappedSpellStage::Source,
    )?;
    let actual_canonical = mapped_spell_stage_leaf(
        &actual,
        &selected,
        &identity.normalized_path,
        child,
        MappedSpellStage::Canonical,
    )?;
    let actual_post = mapped_spell_stage_leaf(
        &actual,
        &selected,
        &identity.normalized_path,
        child,
        MappedSpellStage::PostProjection,
    )?;
    let actual_hydration = mapped_spell_stage_leaf(
        &actual,
        &selected,
        &identity.normalized_path,
        child,
        MappedSpellStage::ArtifactHydration,
    )?;
    let mut mutated_root = if child {
        fixture
            .raw
            .pointer("/system/spell")
            .cloned()
            .ok_or_else(|| {
                error(
                    CoverageFailureCode::ReaderNotObserved,
                    "missing spell child",
                )
            })?
    } else {
        fixture.raw.clone()
    };
    mutate_selected_pattern_leaf(&mut mutated_root, &selected, &identity.normalized_path)?;
    let mutation = select_pattern_leaf(&mutated_root, &identity.normalized_path)?.source;
    let (source_destination, canonical_destination, post_destination) =
        mapped_spell_destinations(&identity.normalized_path, child);
    let hydration_destination = format!("ArtifactHydration::{canonical_destination}");
    let observations = vec![
        presence_stage(
            FinalOwnerStage::SourceDto,
            &source_destination,
            "source::dto::parse_spell_document_source::type_discriminator",
            actual_source,
            &SourcePresence::Missing,
        )?,
        presence_stage(
            FinalOwnerStage::Canonical,
            &canonical_destination,
            "source::normalize::normalize_record_from_source::spell_discriminator",
            actual_canonical,
            &SourcePresence::Missing,
        )?,
        presence_stage(
            FinalOwnerStage::PostProjection,
            &post_destination,
            "index_build_input::index_build_input",
            actual_post,
            &SourcePresence::Missing,
        )?,
        presence_stage(
            FinalOwnerStage::ArtifactHydration,
            &hydration_destination,
            "atlas_index::hydrate_record_parts::spell_discriminator",
            actual_hydration,
            &SourcePresence::Missing,
        )?,
    ];
    sealed_receipt(
        identity,
        fixture.reference,
        selected.source,
        mutation,
        if child {
            CONSUMABLE_SPELL_CHILD_READER
        } else {
            ITEM_SPELL_READER
        },
        "sealed::MappedSpellTypeLeaf",
        observations,
    )
}

#[derive(Debug, Clone, Copy)]
enum MappedSpellStage {
    Source,
    Canonical,
    PostProjection,
    ArtifactHydration,
}

fn mapped_spell_stage_leaf(
    pipeline: &SpellPipeline,
    selected: &SelectedPatternLeaf,
    normalized_path: &str,
    child: bool,
    stage: MappedSpellStage,
) -> Result<SourcePresence<SourceLeafValue>, CoverageContractError> {
    if matches!(
        normalized_path,
        "$.system.description.value" | "$.system.description.gm"
    ) {
        return mapped_spell_content_leaf(pipeline, selected, normalized_path, child, stage);
    }
    let view = match (child, stage) {
        (_, MappedSpellStage::Source) => source_view(&pipeline.source, &pipeline.common_source)?,
        (false, MappedSpellStage::Canonical) => canonical_spell_view(
            &pipeline.record,
            pipeline.canonical.as_ref().ok_or_else(|| {
                error(CoverageFailureCode::CanonicalMismatch, "missing spell body")
            })?,
        )?,
        (false, MappedSpellStage::PostProjection) => canonical_spell_view(
            &pipeline.record,
            pipeline.post_projection.as_ref().ok_or_else(|| {
                error(
                    CoverageFailureCode::PostProjectionMismatch,
                    "missing projected spell body",
                )
            })?,
        )?,
        (false, MappedSpellStage::ArtifactHydration) => canonical_spell_view(
            pipeline.hydrated_record.as_ref().ok_or_else(|| {
                error(
                    CoverageFailureCode::ArtifactHydrationMismatch,
                    "missing hydrated spell record",
                )
            })?,
            pipeline.hydration.as_ref().ok_or_else(|| {
                error(
                    CoverageFailureCode::ArtifactHydrationMismatch,
                    "missing hydrated spell body",
                )
            })?,
        )?,
        (true, MappedSpellStage::Canonical) => {
            canonical_child_view(pipeline.child.as_ref().ok_or_else(|| {
                error(
                    CoverageFailureCode::CanonicalMismatch,
                    "missing canonical spell child",
                )
            })?)?
        }
        (true, MappedSpellStage::PostProjection) => {
            canonical_child_view(pipeline.post_child.as_ref().ok_or_else(|| {
                error(
                    CoverageFailureCode::PostProjectionMismatch,
                    "missing projected spell child",
                )
            })?)?
        }
        (true, MappedSpellStage::ArtifactHydration) => {
            canonical_child_view(pipeline.hydrated_child.as_ref().ok_or_else(|| {
                error(
                    CoverageFailureCode::ArtifactHydrationMismatch,
                    "missing hydrated spell child",
                )
            })?)?
        }
    };
    selected_leaf_from_view(&view, selected)
}

fn mapped_spell_content_leaf(
    pipeline: &SpellPipeline,
    _selected: &SelectedPatternLeaf,
    normalized_path: &str,
    child: bool,
    stage: MappedSpellStage,
) -> Result<SourcePresence<SourceLeafValue>, CoverageContractError> {
    let source_path = if child {
        format!("$.system.spell{}", normalized_path.trim_start_matches('$'))
    } else {
        normalized_path.to_string()
    };
    let source_kind = spell_content_source_kind(normalized_path, child).ok_or_else(|| {
        error(
            stage.mismatch_code(),
            format!("no typed rich-content source kind for {normalized_path}"),
        )
    })?;
    let document = match stage {
        MappedSpellStage::Source if child => pipeline
            .source_content
            .iter()
            .find(|content| {
                content.source_kind == source_kind
                    && content.nested_source_id.as_deref()
                        == spell_child_source_id(&pipeline.source)
                    && content
                        .relative_source_path
                        .ends_with(".system.spell.system.description.value")
            })
            .map(|content| &content.document),
        MappedSpellStage::Source => pipeline
            .source_content
            .iter()
            .find(|content| {
                content.relative_source_path == source_path && content.source_kind == source_kind
            })
            .map(|content| &content.document),
        MappedSpellStage::Canonical if child => pipeline
            .child
            .as_ref()
            .and_then(|child| {
                owned_child_content_document(
                    &child.definition.content,
                    child.child_id.as_str(),
                    source_kind,
                )
            })
            .map(|document| validated_owned_document(document, stage))
            .transpose()?,
        MappedSpellStage::PostProjection if child => pipeline
            .post_child
            .as_ref()
            .and_then(|child| {
                owned_child_content_document(
                    &child.definition.content,
                    child.child_id.as_str(),
                    source_kind,
                )
            })
            .map(|document| validated_owned_document(document, stage))
            .transpose()?,
        MappedSpellStage::ArtifactHydration if child => pipeline
            .hydrated_child
            .as_ref()
            .and_then(|child| {
                owned_child_content_document(
                    &child.definition.content,
                    child.child_id.as_str(),
                    source_kind,
                )
            })
            .map(|document| validated_owned_document(document, stage))
            .transpose()?,
        MappedSpellStage::Canonical => pipeline
            .canonical
            .as_ref()
            .and_then(|spell| {
                owned_content_document(&spell.definition.content, &source_path, source_kind)
            })
            .map(|document| validated_owned_document(document, stage))
            .transpose()?,
        MappedSpellStage::PostProjection => pipeline
            .post_projection
            .as_ref()
            .and_then(|spell| {
                owned_content_document(&spell.definition.content, &source_path, source_kind)
            })
            .map(|document| validated_owned_document(document, stage))
            .transpose()?,
        MappedSpellStage::ArtifactHydration => pipeline
            .hydration
            .as_ref()
            .and_then(|spell| {
                owned_content_document(&spell.definition.content, &source_path, source_kind)
            })
            .map(|document| validated_owned_document(document, stage))
            .transpose()?,
    }
    .ok_or_else(|| {
        error(
            stage.mismatch_code(),
            format!("typed rich-content owner did not retain {source_path}"),
        )
    })?;
    rich_document_leaf(document, stage)
}

fn is_spell_content_path(normalized_path: &str) -> bool {
    matches!(
        normalized_path,
        "$.system.description.value" | "$.system.description.gm"
    )
}

fn rich_document_leaf(
    document: &RichDocument,
    stage: MappedSpellStage,
) -> Result<SourcePresence<SourceLeafValue>, CoverageContractError> {
    let mut authored_semantics = document.clone();
    visit_foundry_links_mut(&mut authored_semantics, |link| {
        link.target = RichLinkTarget::Unresolved {
            target: link.source.authored_target.clone(),
            fallback_label: String::new(),
        };
    });
    Ok(SourcePresence::Value(SourceLeafValue {
        json_type: SourceJsonType::String,
        value: None,
        stable_digest: Some(
            digest_serializable(&authored_semantics)
                .map_err(|message| error(stage.mismatch_code(), message.message))?,
        ),
        member_kind: None,
        member_identity: None,
        ordinal: None,
        multiplicity: 1,
        unsupported: None,
    }))
}

fn owned_content_document<'a>(
    content: &'a atlas_record::OwnedRichContent,
    source_path: &str,
    source_kind: ContentSourceKind,
) -> Option<&'a OwnedRichContentDocument> {
    content.documents.iter().find(|document| {
        document.provenance.field_or_pointer_family == source_path
            && document.source_kind == source_kind
    })
}

fn owned_child_content_document<'a>(
    content: &'a atlas_record::OwnedRichContent,
    child_id: &str,
    source_kind: ContentSourceKind,
) -> Option<&'a OwnedRichContentDocument> {
    content.documents.iter().find(|document| {
        document.source_kind == source_kind
            && document.provenance.nested_source_id.as_deref() == Some(child_id)
            && document
                .provenance
                .field_or_pointer_family
                .ends_with(".system.spell.system.description.value")
    })
}

fn spell_child_source_id(source: &SpellDocumentSource) -> Option<&str> {
    match source {
        SpellDocumentSource::ConsumableChild(child) => Some(child.spell.id.as_str()),
        SpellDocumentSource::Standalone(_) => None,
    }
}

fn validated_owned_document(
    document: &OwnedRichContentDocument,
    stage: MappedSpellStage,
) -> Result<&RichDocument, CoverageContractError> {
    let actual_hash = ContentHash::for_document(&document.document);
    if actual_hash != document.content_hash {
        return Err(error(
            stage.mismatch_code(),
            format!(
                "typed rich-content owner has stale content hash {} for {}",
                document.content_hash.as_str(),
                document.provenance.field_or_pointer_family
            ),
        ));
    }
    Ok(&document.document)
}

fn spell_content_source_kind(normalized_path: &str, child: bool) -> Option<ContentSourceKind> {
    match (child, normalized_path) {
        (false, "$.system.description.value") => Some(ContentSourceKind::Description),
        (false, "$.system.description.gm") => Some(ContentSourceKind::GmNotes),
        (true, "$.system.description.value") => Some(ContentSourceKind::EmbeddedSpellDescription),
        _ => None,
    }
}

impl MappedSpellStage {
    const fn mismatch_code(self) -> CoverageFailureCode {
        match self {
            Self::Source => CoverageFailureCode::DtoMismatch,
            Self::Canonical => CoverageFailureCode::CanonicalMismatch,
            Self::PostProjection => CoverageFailureCode::PostProjectionMismatch,
            Self::ArtifactHydration => CoverageFailureCode::ArtifactHydrationMismatch,
        }
    }
}

fn mapped_spell_destinations(path: &str, child: bool) -> (String, String, String) {
    let (source, canonical) = if child {
        match path {
            "$._id" => (
                "ConsumableSpellChildSource.spell.id".to_string(),
                "ConsumableSpellChild.child_id".to_string(),
            ),
            "$.name" => (
                "ConsumableSpellChildSource.spell.name".to_string(),
                "ConsumableSpellChild.name".to_string(),
            ),
            "$.type" => (
                "parse_spell_document_source.child_type_discriminator".to_string(),
                "ConsumableSpellChild.definition (spell discriminator)".to_string(),
            ),
            "$.flags.core.sourceId" => (
                "ConsumableSpellChildSource.standalone_locator".to_string(),
                "ConsumableSpellChild.standalone_locator".to_string(),
            ),
            "$.system.description.value" => (
                "SourceConstructionFacts.source_facts.content_sources[$.system.spell.system.description.value]".to_string(),
                "ConsumableSpellChild.definition.content[$.system.spell.system.description.value]".to_string(),
            ),
            _ => (
                format!("ConsumableSpellChildSource.spell::{path}"),
                format!("ConsumableSpellChild.definition::{path}"),
            ),
        }
    } else {
        match path {
            "$._id" => (
                "SpellItemSource.id".to_string(),
                "SpellRecord.identity.source_id".to_string(),
            ),
            "$.name" => (
                "SpellItemSource.name".to_string(),
                "SpellRecord.identity.name".to_string(),
            ),
            "$.type" => (
                "VersionedItemSource.source.item_type".to_string(),
                "RecordBody::Spell discriminator".to_string(),
            ),
            "$.folder" => (
                "VersionedItemSource.source.folder".to_string(),
                "AtlasRecord.foundry.folder_id".to_string(),
            ),
            "$.system.publication.title" => (
                "SpellItemSource.publication_title".to_string(),
                "AtlasRecord.publication.title".to_string(),
            ),
            "$.system.publication.remaster" => (
                "SpellItemSource.publication_remaster".to_string(),
                "AtlasRecord.publication.remaster".to_string(),
            ),
            "$.system.traits.rarity" => (
                "SpellItemSource.rarity".to_string(),
                "AtlasRecord.classification.rarity".to_string(),
            ),
            "$.system.description.value" | "$.system.description.gm" => (
                format!("SourceConstructionFacts.source_facts.content_sources[{path}]"),
                format!("SpellDefinition.content[{path}]"),
            ),
            _ => (
                format!("SpellItemSource::{path}"),
                format!("SpellDefinition::{path}"),
            ),
        }
    };
    let post = format!("PostProjection::{canonical}");
    (source, canonical, post)
}

fn provenance_receipt(
    identity: SourceLeafIdentity,
    fixture: FixtureReference,
    source: SourcePresence<SourceLeafValue>,
    source_mutation: SourcePresence<SourceLeafValue>,
    accessor_binding: &str,
    observations: Vec<StageObservation>,
) -> Result<SourceLeafReceipt, CoverageContractError> {
    if source == source_mutation {
        return Err(error(
            CoverageFailureCode::ReaderNotObserved,
            "registered provenance mutation did not alter the selected source leaf",
        ));
    }
    let reader = ActualReadEvidence {
        reader_id: SPELL_RAW_PROVENANCE_READER.to_string(),
        accessor_binding: accessor_binding.to_string(),
        purpose: SourceAccessorPurpose::ProvenanceReader,
        mutation_digest: digest_serializable(&source_mutation)?,
    };
    let semantic_output = Some(SemanticOutputObservation {
        accessor_binding: format!("{accessor_binding}::no-semantic-fallback"),
        observed: false,
        mutation_observed: true,
    });
    let evidence_digest = evidence_digest(
        &identity,
        &fixture,
        &source,
        &reader,
        &observations,
        &semantic_output,
    )?;
    Ok(SourceLeafReceipt {
        identity,
        fixture,
        source,
        reader,
        observations,
        semantic_output,
        evidence_digest,
    })
}

#[derive(Debug)]
struct SpellPipeline {
    source: SpellDocumentSource,
    common_source: VersionedItemSource,
    source_content: Vec<SourceContentFact>,
    record: AtlasRecord,
    canonical: Option<SpellRecord>,
    child: Option<ConsumableSpellChild>,
    post_projection: Option<SpellRecord>,
    post_child: Option<ConsumableSpellChild>,
    hydrated_record: Option<AtlasRecord>,
    hydration: Option<SpellRecord>,
    hydrated_child: Option<ConsumableSpellChild>,
    raw_provenance: Value,
}

#[derive(Debug, Clone)]
struct SpellArtifactHydration {
    record: AtlasRecord,
    body: Option<SpellRecord>,
    child: Option<ConsumableSpellChild>,
}

#[derive(Debug, Clone)]
struct SpellArtifactHydrationPair {
    baseline: SpellArtifactHydration,
    mutation: SpellArtifactHydration,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
enum SpellArtifactMutationGroup {
    CompatibleSiblings,
    IsolatedDiscriminator {
        normalized_path: String,
        case_id: String,
    },
}

#[derive(Debug, Clone)]
struct SpellArtifactHydrationEvidence {
    baseline: SpellArtifactHydration,
    mutations: BTreeMap<SpellArtifactMutationGroup, SpellArtifactHydration>,
}

impl SpellArtifactHydrationPair {
    fn new(
        baseline: atlas_record::RetrievedRecord,
        mutation: atlas_record::RetrievedRecord,
    ) -> Self {
        Self {
            baseline: SpellArtifactHydration::new(baseline),
            mutation: SpellArtifactHydration::new(mutation),
        }
    }
}

impl SpellArtifactHydrationEvidence {
    fn pair_for(
        &self,
        group: &SpellArtifactMutationGroup,
    ) -> Result<SpellArtifactHydrationPair, CoverageContractError> {
        let mutation = self.mutations.get(group).ok_or_else(|| {
            error(
                CoverageFailureCode::ArtifactHydrationMismatch,
                format!("missing grouped spell artifact mutation for {group:?}"),
            )
        })?;
        Ok(SpellArtifactHydrationPair {
            baseline: self.baseline.clone(),
            mutation: mutation.clone(),
        })
    }
}

impl SpellArtifactHydration {
    fn new(hydrated: atlas_record::RetrievedRecord) -> Self {
        Self {
            record: hydrated.record,
            body: hydrated.body.and_then(|body| match body {
                RecordBody::Spell(spell) => Some(spell),
                RecordBody::Creature(_) | RecordBody::Hazard(_) => None,
            }),
            child: hydrated.spell_children.into_iter().next(),
        }
    }

    fn apply_to(&self, pipeline: &mut SpellPipeline) {
        pipeline.hydrated_record = Some(self.record.clone());
        pipeline.hydration = self.body.clone();
        pipeline.hydrated_child = self.child.clone();
    }
}

fn apply_spell_artifact_pair(
    actual: &mut SpellPipeline,
    actual_input: &atlas_index::IndexBuildInput,
    mutation: &mut SpellPipeline,
    mutation_input: &atlas_index::IndexBuildInput,
    evidence: Option<&SpellArtifactHydrationEvidence>,
    group: &SpellArtifactMutationGroup,
) -> Result<(), CoverageContractError> {
    let pair = if let Some(evidence) = evidence {
        evidence.pair_for(group)?
    } else {
        let actual_key = spell_input_record_key(actual_input)?;
        let mutation_key = spell_input_record_key(mutation_input)?;
        SpellArtifactHydrationPair::new(
            persist_and_hydrate_spell(actual_input, actual_key)?,
            persist_and_hydrate_spell(mutation_input, mutation_key)?,
        )
    };
    pair.baseline.apply_to(actual);
    pair.mutation.apply_to(mutation);
    Ok(())
}

fn apply_spell_artifact_baseline(
    actual: &mut SpellPipeline,
    actual_input: &atlas_index::IndexBuildInput,
    evidence: Option<&SpellArtifactHydrationEvidence>,
) -> Result<(), CoverageContractError> {
    let baseline = if let Some(evidence) = evidence {
        evidence.baseline.clone()
    } else {
        SpellArtifactHydration::new(persist_and_hydrate_spell(
            actual_input,
            spell_input_record_key(actual_input)?,
        )?)
    };
    baseline.apply_to(actual);
    Ok(())
}

fn spell_input_record_key(
    input: &atlas_index::IndexBuildInput,
) -> Result<&RecordKey, CoverageContractError> {
    input
        .records
        .first()
        .map(|record| &record.identity.key)
        .ok_or_else(|| {
            error(
                CoverageFailureCode::ArtifactHydrationMismatch,
                "spell receipt input has no generic record",
            )
        })
}

fn run_spell_pipeline(
    fixture: &ResolvedFixture,
    raw: Value,
    serialized: &[u8],
) -> Result<(SpellPipeline, atlas_index::IndexBuildInput), CoverageContractError> {
    let pack = fixture
        .reference
        .record_key
        .split_once(':')
        .map(|(pack, _)| pack)
        .ok_or_else(|| {
            error(
                CoverageFailureCode::FixtureNotSourceGrounded,
                "invalid spell fixture record key",
            )
        })?;
    let pack_name = PackName::new(pack.to_string()).map_err(|message| {
        error(
            CoverageFailureCode::FixtureNotSourceGrounded,
            message.to_string(),
        )
    })?;
    let manifest_pack = ManifestPack {
        name: pack.to_string(),
        label: "spell source-leaf fixture".to_string(),
        document_type: "Item".to_string(),
        path: fixture
            .reference
            .source_path
            .rsplit_once('/')
            .map_or("packs", |(parent, _)| parent)
            .to_string(),
    };
    let source_identity = SourceIdentity::new(
        fixture.reference.record_key.clone(),
        fixture.reference.source_path.clone(),
    );
    let serialized_source = parse_serialized_source_object(serialized)
        .map_err(|message| error(CoverageFailureCode::ReaderNotObserved, message))?;
    let common_source = parse_item_source(
        pinned_source_version_metadata(),
        source_identity.clone(),
        None,
        raw.clone(),
    )
    .map_err(|message| error(CoverageFailureCode::ReaderNotObserved, message))?;
    let source = parse_spell_document_source(&serialized_source, &source_identity)
        .map_err(|message| error(CoverageFailureCode::ReaderNotObserved, message))?
        .ok_or_else(|| {
            error(
                CoverageFailureCode::ReaderNotObserved,
                "registered source did not parse as a spell definition or consumable spell child",
            )
        })?;
    let loaded = normalize_record_from_source(
        &manifest_pack,
        &pack_name,
        Path::new(&fixture.reference.source_path),
        Path::new("."),
        serialized_source,
        None,
    )
    .map_err(|message| error(CoverageFailureCode::ReaderNotObserved, message))?;
    let mut records = vec![loaded];
    let index = build_record_reference_index(&records);
    finalize_consumable_spell_children(&mut records, &index)
        .map_err(|message| error(CoverageFailureCode::CanonicalMismatch, message))?;
    finalize_spell_owned_content(&mut records);
    let canonical = records[0]
        .facts
        .canonical_body
        .as_ref()
        .and_then(RecordBody::as_spell)
        .cloned();
    let child = records[0].facts.canonical_spell_children.first().cloned();
    resolve_content_references(&mut records, &index);
    let loaded = records.remove(0);
    let record = loaded.record.clone();
    let source_content = loaded.facts.source_facts.content_sources.clone();
    let post_child = loaded.facts.canonical_spell_children.first().cloned();
    let raw_provenance = loaded
        .record
        .provenance
        .raw_json
        .as_deref()
        .ok_or_else(|| {
            error(
                CoverageFailureCode::ProvenanceNotDurable,
                "normalized spell record has no exact raw provenance",
            )
        })
        .and_then(|value| {
            serde_json::from_str(value)
                .map_err(|message| error(CoverageFailureCode::ProvenanceNotDurable, message))
        })?;
    let input = index_build_input(SourceLoad {
        manifest_path: PathBuf::from("static/system.json"),
        source_signature: PF2E_SOURCE_PINNED_SIGNATURE.to_string(),
        source_record_count: 1,
        packs: vec![LoadedPack {
            name: pack_name,
            label: manifest_pack.label,
            document_type: manifest_pack.document_type,
            declared_path: manifest_pack.path.clone(),
            resolved_path: PathBuf::from(manifest_pack.path),
            record_count: 1,
        }],
        records: vec![loaded],
        references: Vec::new(),
        aliases: Vec::new(),
        remaster_links: Vec::new(),
        pending_document_embeddings: Vec::new(),
        document_embeddings: Vec::new(),
        document_embedding_tokenization: Default::default(),
        diagnostics: IngestDiagnostics::default(),
        skipped_records: Vec::new(),
        warnings: Vec::new(),
    });
    let post_projection = input
        .canonical_bodies
        .first()
        .and_then(RecordBody::as_spell)
        .cloned();
    Ok((
        SpellPipeline {
            source,
            common_source,
            source_content,
            record,
            canonical,
            child,
            post_projection,
            post_child,
            hydrated_record: None,
            hydration: None,
            hydrated_child: None,
            raw_provenance,
        },
        input,
    ))
}

fn persist_and_hydrate_spell(
    input: &atlas_index::IndexBuildInput,
    key: &RecordKey,
) -> Result<atlas_record::RetrievedRecord, CoverageContractError> {
    let nonce = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_err(|message| error(CoverageFailureCode::ArtifactHydrationMismatch, message))?
        .as_nanos();
    let root = std::env::temp_dir().join(format!(
        "pf2e-atlas-h2-receipt-{}-{nonce}",
        std::process::id()
    ));
    std::fs::create_dir_all(&root)
        .map_err(|message| error(CoverageFailureCode::ArtifactHydrationMismatch, message))?;
    let artifact = root.join("index.sqlite");
    let result = (|| {
        let receipt = atlas_index::IndexArtifactWriter::write(
            &atlas_index::SqliteIndexWriter::new(artifact.clone()),
            input,
            atlas_embedding::EmbeddingModelId::BgeSmallEnV15,
        )
        .map_err(|message| error(CoverageFailureCode::ArtifactHydrationMismatch, message))?;
        let manifest = serde_json::json!({
            "manifest_version": atlas_index::ARTIFACT_MANIFEST_VERSION,
            "artifact_contract_version": atlas_index::ARTIFACT_CONTRACT_VERSION,
            "schema_version": atlas_index::ARTIFACT_SCHEMA_VERSION,
            "build": { "artifact_sha256": receipt.artifact_sha256() },
        });
        std::fs::write(
            root.join("manifest.json"),
            serde_json::to_vec(&manifest).map_err(|message| {
                error(CoverageFailureCode::ArtifactHydrationMismatch, message)
            })?,
        )
        .map_err(|message| error(CoverageFailureCode::ArtifactHydrationMismatch, message))?;
        let reader = atlas_index::SqliteIndexReader::open_read_only(&artifact)
            .map_err(|message| error(CoverageFailureCode::ArtifactHydrationMismatch, message))?;
        let mut hydrated = reader
            .load_hydrated_records_by_key(std::slice::from_ref(key))
            .map_err(|message| error(CoverageFailureCode::ArtifactHydrationMismatch, message))?;
        if hydrated.len() != 1 {
            return Err(error(
                CoverageFailureCode::ArtifactHydrationMismatch,
                format!(
                    "artifact hydration returned {} rows for `{key}`",
                    hydrated.len()
                ),
            ));
        }
        Ok(hydrated.remove(0))
    })();
    let _ = std::fs::remove_dir_all(&root);
    result
}

fn raw_spell_leaf(
    fixture: &ResolvedFixture,
    probe: SpellLeafProbe,
) -> Result<SourcePresence<SourceLeafValue>, CoverageContractError> {
    raw_spell_leaf_from_root(&fixture.raw, false, probe, fixture.reference.case_id())
}

fn raw_spell_leaf_from_root(
    root: &Value,
    child: bool,
    probe: SpellLeafProbe,
    case_id: &str,
) -> Result<SourcePresence<SourceLeafValue>, CoverageContractError> {
    let spell = if child {
        root.pointer("/system/spell").ok_or_else(|| {
            error(
                CoverageFailureCode::ReaderNotObserved,
                "missing spell child",
            )
        })?
    } else {
        root
    };
    match probe {
        SpellLeafProbe::Rank => scalar_pointer_leaf(spell, "/system/level/value"),
        SpellLeafProbe::DamageFormula => {
            let (key, _) = spell_member_for_case(probe, case_id)?;
            map_pointer_leaf(
                spell,
                key,
                &format!("/system/damage/{}/formula", escape_json_pointer(key)),
            )
        }
        SpellLeafProbe::OverlaySort => {
            let (key, _) = spell_member_for_case(probe, case_id)?;
            map_pointer_leaf(
                spell,
                key,
                &format!("/system/overlays/{}/sort", escape_json_pointer(key)),
            )
        }
        SpellLeafProbe::RuleKey => {
            let (key, ordinal) = spell_member_for_case(probe, case_id)?;
            array_pointer_leaf(
                spell,
                &format!("{ordinal}:{key}"),
                ordinal,
                &format!("/system/rules/{ordinal}/key"),
            )
        }
        SpellLeafProbe::StandaloneLocation => scalar_pointer_leaf(spell, "/system/location"),
        SpellLeafProbe::Image => scalar_pointer_leaf(spell, "/img"),
        SpellLeafProbe::PublicationLicense => {
            scalar_pointer_leaf(spell, "/system/publication/license")
        }
        SpellLeafProbe::Mapped => mapped_probe_error("raw spell leaf"),
    }
}

fn raw_consumable_spell_child_leaf(
    fixture: &ResolvedFixture,
    probe: ConsumableSpellChildLeafProbe,
) -> Result<SourcePresence<SourceLeafValue>, CoverageContractError> {
    raw_consumable_spell_child_leaf_from_root(&fixture.raw, probe, fixture.reference.case_id())
}

fn raw_consumable_spell_child_leaf_from_root(
    root: &Value,
    probe: ConsumableSpellChildLeafProbe,
    _case_id: &str,
) -> Result<SourcePresence<SourceLeafValue>, CoverageContractError> {
    let spell = root.pointer("/system/spell").ok_or_else(|| {
        error(
            CoverageFailureCode::ReaderNotObserved,
            "missing spell child",
        )
    })?;
    scalar_pointer_leaf(spell, consumable_spell_child_pointer(probe)?)
}

fn raw_provenance_consumable_spell_child_leaf(
    raw: &Value,
    probe: ConsumableSpellChildLeafProbe,
    case_id: &str,
) -> Result<SourcePresence<SourceLeafValue>, CoverageContractError> {
    raw_consumable_spell_child_leaf_from_root(raw, probe, case_id)
}

fn mutate_spell_leaf(
    root: &mut Value,
    child: bool,
    probe: SpellLeafProbe,
    case_id: &str,
) -> Result<(), CoverageContractError> {
    let spell = if child {
        root.pointer_mut("/system/spell").ok_or_else(|| {
            error(
                CoverageFailureCode::ReaderNotObserved,
                "missing spell child",
            )
        })?
    } else {
        root
    };
    match probe {
        SpellLeafProbe::Rank => mutate_spell_rank(spell, "/system/level/value"),
        SpellLeafProbe::DamageFormula => {
            let (key, _) = spell_member_for_case(probe, case_id)?;
            mutate_pointer(
                spell,
                &format!("/system/damage/{}/formula", escape_json_pointer(key)),
            )
        }
        SpellLeafProbe::OverlaySort => {
            let (key, _) = spell_member_for_case(probe, case_id)?;
            mutate_pointer(
                spell,
                &format!("/system/overlays/{}/sort", escape_json_pointer(key)),
            )
        }
        SpellLeafProbe::RuleKey => {
            let (_, ordinal) = spell_member_for_case(probe, case_id)?;
            mutate_pointer(spell, &format!("/system/rules/{ordinal}/key"))
        }
        SpellLeafProbe::StandaloneLocation => {
            let system = spell
                .pointer_mut("/system")
                .and_then(Value::as_object_mut)
                .ok_or_else(|| error(CoverageFailureCode::ReaderNotObserved, "missing system"))?;
            system.insert(
                "location".to_string(),
                serde_json::json!({"value": "source-leaf-mutation"}),
            );
            Ok(())
        }
        SpellLeafProbe::Image => mutate_pointer(spell, "/img"),
        SpellLeafProbe::PublicationLicense => mutate_pointer(spell, "/system/publication/license"),
        SpellLeafProbe::Mapped => mapped_probe_error("spell mutation"),
    }
}

fn mutate_grouped_spell_artifact(
    mutation_root: &mut Value,
    baseline_root: &Value,
    normalized_path: &str,
    registration: RegisteredAccessor,
    case_id: &str,
) -> Result<bool, CoverageContractError> {
    match registration {
        RegisteredAccessor::ItemSpell(SpellLeafProbe::Mapped) => {
            if normalized_path == "$.type" {
                return Ok(false);
            }
            let selected = select_pattern_leaf(baseline_root, normalized_path)?;
            mutate_selected_pattern_leaf(mutation_root, &selected, normalized_path)?;
        }
        RegisteredAccessor::ItemSpell(probe) => {
            mutate_spell_leaf(mutation_root, false, probe, case_id)?;
        }
        RegisteredAccessor::ConsumableSpellChild(ConsumableSpellChildLeafProbe::Mapped) => {
            if normalized_path == "$.type" {
                return Ok(false);
            }
            let baseline_child = baseline_root.pointer("/system/spell").ok_or_else(|| {
                error(
                    CoverageFailureCode::ReaderNotObserved,
                    "missing baseline consumable spell child",
                )
            })?;
            let mutation_child = mutation_root.pointer_mut("/system/spell").ok_or_else(|| {
                error(
                    CoverageFailureCode::ReaderNotObserved,
                    "missing mutable consumable spell child",
                )
            })?;
            let selected = select_pattern_leaf(baseline_child, normalized_path)?;
            mutate_selected_pattern_leaf(mutation_child, &selected, normalized_path)?;
        }
        RegisteredAccessor::ConsumableSpellChild(ConsumableSpellChildLeafProbe::SortProvenance) => {
            return Ok(false);
        }
        RegisteredAccessor::ConsumableSpellChild(probe) => {
            mutate_consumable_spell_child_leaf(mutation_root, probe, case_id)?;
        }
        RegisteredAccessor::ItemActionName
        | RegisteredAccessor::ActorNpcAbilityMod(_)
        | RegisteredAccessor::ActorNpcShadowSkillBase
        | RegisteredAccessor::ActorHazardDisable
        | RegisteredAccessor::ActorHazardFolder
        | RegisteredAccessor::ActorHazardTemporaryMaximum
        | RegisteredAccessor::EmbeddedHazardActionType
        | RegisteredAccessor::EmbeddedHazardDamage
        | RegisteredAccessor::ActorHazardExact(_) => {
            return Err(error(
                CoverageFailureCode::InvalidContract,
                "grouped spell artifact mutation received a non-spell accessor",
            ));
        }
    }
    Ok(true)
}

fn required_spell_artifact_mutation_group(
    normalized_path: &str,
    registration: RegisteredAccessor,
    case_id: &str,
) -> Result<SpellArtifactMutationGroup, CoverageContractError> {
    spell_artifact_mutation_group(normalized_path, registration, case_id).ok_or_else(|| {
        error(
            CoverageFailureCode::ArtifactHydrationMismatch,
            format!("spell receipt {case_id} has no artifact mutation group"),
        )
    })
}

fn spell_artifact_mutation_group(
    normalized_path: &str,
    registration: RegisteredAccessor,
    case_id: &str,
) -> Option<SpellArtifactMutationGroup> {
    if normalized_path == "$.type"
        || matches!(
            registration,
            RegisteredAccessor::ConsumableSpellChild(ConsumableSpellChildLeafProbe::SortProvenance)
        )
    {
        return None;
    }
    if matches!(
        normalized_path,
        "$.system.heightening.type"
            | "$.system.overlays.*.overlayType"
            | "$.system.overlays.*.system.heightening.type"
            | "$.system.rules[].key"
    ) {
        return Some(SpellArtifactMutationGroup::IsolatedDiscriminator {
            normalized_path: normalized_path.to_string(),
            case_id: case_id.to_string(),
        });
    }
    Some(SpellArtifactMutationGroup::CompatibleSiblings)
}

fn mutate_consumable_spell_child_leaf(
    root: &mut Value,
    probe: ConsumableSpellChildLeafProbe,
    _case_id: &str,
) -> Result<(), CoverageContractError> {
    let spell = root.pointer_mut("/system/spell").ok_or_else(|| {
        error(
            CoverageFailureCode::ReaderNotObserved,
            "missing spell child",
        )
    })?;
    mutate_pointer(spell, consumable_spell_child_pointer(probe)?)
}

fn mutate_pointer(root: &mut Value, pointer: &str) -> Result<(), CoverageContractError> {
    let value = root.pointer_mut(pointer).ok_or_else(|| {
        error(
            CoverageFailureCode::ReaderNotObserved,
            format!("mutation pointer {pointer} is missing"),
        )
    })?;
    *value = match value {
        Value::Null => Value::String("source-leaf-mutation".to_string()),
        Value::Bool(current) => Value::Bool(!*current),
        Value::String(current) => Value::String(format!("{current}\u{241f}source-leaf-mutation")),
        Value::Number(current) => Value::from(current.as_i64().unwrap_or_default() + 101),
        other => {
            return Err(error(
                CoverageFailureCode::ReaderNotObserved,
                format!("mutation pointer {pointer} has unsupported value {other}"),
            ));
        }
    };
    Ok(())
}

fn mutate_spell_rank(root: &mut Value, pointer: &str) -> Result<(), CoverageContractError> {
    let value = root.pointer_mut(pointer).ok_or_else(|| {
        error(
            CoverageFailureCode::ReaderNotObserved,
            format!("mutation pointer {pointer} is missing"),
        )
    })?;
    let current = value.as_u64().ok_or_else(|| {
        error(
            CoverageFailureCode::ReaderNotObserved,
            format!("mutation pointer {pointer} is not an unsigned spell rank"),
        )
    })?;
    *value = Value::from(if current >= 10 {
        current - 1
    } else {
        current + 1
    });
    Ok(())
}

fn scalar_pointer_leaf(
    root: &Value,
    pointer: &str,
) -> Result<SourcePresence<SourceLeafValue>, CoverageContractError> {
    match root.pointer(pointer) {
        None => Ok(SourcePresence::Missing),
        Some(Value::Null) => Ok(SourcePresence::Null),
        Some(value) => source_leaf_value(value.clone(), None, None, None),
    }
}

fn map_pointer_leaf(
    root: &Value,
    key: &str,
    pointer: &str,
) -> Result<SourcePresence<SourceLeafValue>, CoverageContractError> {
    match root.pointer(pointer) {
        None => Ok(SourcePresence::Missing),
        Some(Value::Null) => Ok(SourcePresence::Null),
        Some(value) => source_leaf_value(
            value.clone(),
            Some(SourceMemberKind::Map),
            Some(key.to_string()),
            None,
        ),
    }
}

fn array_pointer_leaf(
    root: &Value,
    identity: &str,
    ordinal: usize,
    pointer: &str,
) -> Result<SourcePresence<SourceLeafValue>, CoverageContractError> {
    match root.pointer(pointer) {
        None => Ok(SourcePresence::Missing),
        Some(Value::Null) => Ok(SourcePresence::Null),
        Some(value) => source_leaf_value(
            value.clone(),
            Some(SourceMemberKind::Array),
            Some(identity.to_string()),
            Some(ordinal),
        ),
    }
}

fn source_leaf_value(
    value: Value,
    member_kind: Option<SourceMemberKind>,
    member_identity: Option<String>,
    ordinal: Option<usize>,
) -> Result<SourcePresence<SourceLeafValue>, CoverageContractError> {
    let json_type = match &value {
        Value::Null => SourceJsonType::Null,
        Value::Bool(_) => SourceJsonType::Boolean,
        Value::Number(_) => SourceJsonType::Number,
        Value::String(_) => SourceJsonType::String,
        Value::Array(_) => SourceJsonType::Array,
        Value::Object(_) => SourceJsonType::Object,
    };
    Ok(SourcePresence::Value(SourceLeafValue {
        json_type,
        value: Some(value),
        stable_digest: None,
        member_kind,
        member_identity,
        ordinal,
        multiplicity: 1,
        unsupported: None,
    }))
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum SelectedPathStep {
    Key(String),
    Index(usize),
}

#[derive(Debug, Clone)]
struct SelectedPatternLeaf {
    steps: Vec<SelectedPathStep>,
    source: SourcePresence<SourceLeafValue>,
}

fn select_pattern_leaf(
    root: &Value,
    pattern: &str,
) -> Result<SelectedPatternLeaf, CoverageContractError> {
    let suffix = pattern.strip_prefix("$.").ok_or_else(|| {
        error(
            CoverageFailureCode::ReaderNotObserved,
            format!("spell coverage path must begin with $.: {pattern}"),
        )
    })?;
    let mut tokens = Vec::new();
    let mut has_map = false;
    let mut has_array = false;
    for segment in suffix.split('.') {
        if segment == "*" {
            has_map = true;
            tokens.push(PatternToken::AnyMapKey);
        } else if let Some(key) = segment.strip_suffix("[]") {
            has_array = true;
            if !key.is_empty() {
                tokens.push(PatternToken::Key(key.to_string()));
            }
            tokens.push(PatternToken::AnyArrayIndex);
        } else {
            tokens.push(PatternToken::Key(segment.to_string()));
        }
    }
    let mut steps = Vec::new();
    let value = select_pattern_value(root, &tokens, &mut steps).ok_or_else(|| {
        error(
            CoverageFailureCode::ReaderNotObserved,
            format!("fixture has no value for mapped spell leaf {pattern}"),
        )
    })?;
    let map_identity = steps
        .iter()
        .filter_map(|step| match step {
            SelectedPathStep::Key(key) => Some(key.as_str()),
            SelectedPathStep::Index(_) => None,
        })
        .collect::<Vec<_>>()
        .join("/");
    let ordinal = steps.iter().rev().find_map(|step| match step {
        SelectedPathStep::Index(index) => Some(*index),
        SelectedPathStep::Key(_) => None,
    });
    let member_kind = match (has_map, has_array) {
        (false, false) => None,
        (false, true) => Some(SourceMemberKind::Array),
        (true, false) => Some(SourceMemberKind::Map),
        (true, true) => Some(SourceMemberKind::Nested),
    };
    let source = match value {
        Value::Null => SourcePresence::Null,
        value => source_leaf_value(
            value.clone(),
            member_kind,
            member_kind.map(|_| map_identity),
            ordinal,
        )?,
    };
    Ok(SelectedPatternLeaf { steps, source })
}

#[derive(Debug, Clone)]
enum PatternToken {
    Key(String),
    AnyMapKey,
    AnyArrayIndex,
}

fn select_pattern_value<'a>(
    value: &'a Value,
    tokens: &[PatternToken],
    steps: &mut Vec<SelectedPathStep>,
) -> Option<&'a Value> {
    let Some((token, rest)) = tokens.split_first() else {
        return Some(value);
    };
    match token {
        PatternToken::Key(key) => {
            let next = value.as_object()?.get(key)?;
            steps.push(SelectedPathStep::Key(key.clone()));
            let selected = select_pattern_value(next, rest, steps);
            if selected.is_none() {
                steps.pop();
            }
            selected
        }
        PatternToken::AnyMapKey => {
            for (key, next) in value.as_object()? {
                steps.push(SelectedPathStep::Key(key.clone()));
                if let Some(selected) = select_pattern_value(next, rest, steps) {
                    return Some(selected);
                }
                steps.pop();
            }
            None
        }
        PatternToken::AnyArrayIndex => {
            for (index, next) in value.as_array()?.iter().enumerate() {
                steps.push(SelectedPathStep::Index(index));
                if let Some(selected) = select_pattern_value(next, rest, steps) {
                    return Some(selected);
                }
                steps.pop();
            }
            None
        }
    }
}

fn selected_leaf_from_view(
    view: &Value,
    selected: &SelectedPatternLeaf,
) -> Result<SourcePresence<SourceLeafValue>, CoverageContractError> {
    let mut value = view;
    for step in &selected.steps {
        value = match step {
            SelectedPathStep::Key(key) => value.as_object().and_then(|map| map.get(key)),
            SelectedPathStep::Index(index) => value.as_array().and_then(|array| array.get(*index)),
        }
        .ok_or_else(|| {
            error(
                CoverageFailureCode::CanonicalMismatch,
                format!("typed spell owner lacks selected path step {step:?}"),
            )
        })?;
    }
    match (&selected.source, value) {
        (SourcePresence::Null, Value::Null) => Ok(SourcePresence::Null),
        (SourcePresence::Value(expected), value) => source_leaf_value(
            value.clone(),
            expected.member_kind,
            expected.member_identity.clone(),
            expected.ordinal,
        ),
        _ => Err(error(
            CoverageFailureCode::CanonicalMismatch,
            "typed spell owner changed selected source presence",
        )),
    }
}

fn mutate_selected_pattern_leaf(
    root: &mut Value,
    selected: &SelectedPatternLeaf,
    normalized_path: &str,
) -> Result<(), CoverageContractError> {
    let mut value = root;
    for step in &selected.steps {
        value = match step {
            SelectedPathStep::Key(key) => value.as_object_mut().and_then(|map| map.get_mut(key)),
            SelectedPathStep::Index(index) => {
                value.as_array_mut().and_then(|array| array.get_mut(*index))
            }
        }
        .ok_or_else(|| {
            error(
                CoverageFailureCode::ReaderNotObserved,
                format!("mutation lost selected spell path step {step:?}"),
            )
        })?;
    }
    *value = match (normalized_path, &*value) {
        ("$.system.traits.rarity", Value::String(current)) => Value::String(
            if current == "common" {
                "uncommon"
            } else {
                "common"
            }
            .to_string(),
        ),
        ("$._id", Value::String(current)) => Value::String(format!("{current}x")),
        ("$.system.level.value", Value::Number(current)) => {
            let current = current.as_u64().unwrap_or_default();
            Value::from(if current >= 10 {
                current - 1
            } else {
                current + 1
            })
        }
        (_, _) => match value {
            Value::Null => Value::String("source-leaf-mutation".to_string()),
            Value::Bool(current) => Value::Bool(!*current),
            Value::Number(current) => Value::from(current.as_i64().unwrap_or_default() + 1),
            Value::String(current) => {
                Value::String(format!("{current}\u{241f}source-leaf-mutation"))
            }
            Value::Array(_) | Value::Object(_) => {
                return Err(error(
                    CoverageFailureCode::ReaderNotObserved,
                    "mapped spell leaf selected a non-leaf container",
                ));
            }
        },
    };
    Ok(())
}

fn spell_member_for_case(
    probe: SpellLeafProbe,
    case_id: &str,
) -> Result<(&'static str, usize), CoverageContractError> {
    let member = match (probe, case_id) {
        (SpellLeafProbe::DamageFormula, "heal-damage-0") => ("0", 0),
        (SpellLeafProbe::OverlaySort, "heal-overlay-living") => ("37gy7l19tik74o4s", 0),
        (SpellLeafProbe::OverlaySort, "heal-overlay-touch") => ("7qdtetowq348s9oc", 1),
        (SpellLeafProbe::OverlaySort, "heal-overlay-emanation") => ("7vbvdrv2cl87sqta", 2),
        (SpellLeafProbe::OverlaySort, "heal-overlay-undead") => ("lfxcoz2d3f8j2zq1", 3),
        (SpellLeafProbe::RuleKey, "phase-bolt-rule-0") => ("EphemeralEffect", 0),
        (SpellLeafProbe::RuleKey, "qi-blast-rule-0") => ("RollOption", 0),
        (SpellLeafProbe::RuleKey, "qi-blast-rule-1") => ("DamageAlteration", 1),
        (SpellLeafProbe::RuleKey, "qi-blast-rule-2") => ("ItemAlteration", 2),
        (SpellLeafProbe::RuleKey, "qi-blast-rule-3") => ("ItemAlteration", 3),
        _ => {
            return Err(error(
                CoverageFailureCode::FixtureNotSourceGrounded,
                format!("case {case_id} is not registered for {probe:?}"),
            ));
        }
    };
    Ok(member)
}

fn consumable_spell_child_pointer(
    probe: ConsumableSpellChildLeafProbe,
) -> Result<&'static str, CoverageContractError> {
    Ok(match probe {
        ConsumableSpellChildLeafProbe::ChildId => "/_id",
        ConsumableSpellChildLeafProbe::StandaloneLocator => "/flags/core/sourceId",
        ConsumableSpellChildLeafProbe::HeightenedRank => "/system/location/heightenedLevel",
        ConsumableSpellChildLeafProbe::LocationValue => "/system/location/value",
        ConsumableSpellChildLeafProbe::Image => "/img",
        ConsumableSpellChildLeafProbe::SortProvenance => "/sort",
        ConsumableSpellChildLeafProbe::Slug => "/system/slug",
        ConsumableSpellChildLeafProbe::PublicationLicense => "/system/publication/license",
        ConsumableSpellChildLeafProbe::PublicationRemaster => "/system/publication/remaster",
        ConsumableSpellChildLeafProbe::PublicationTitle => "/system/publication/title",
        ConsumableSpellChildLeafProbe::Rarity => "/system/traits/rarity",
        ConsumableSpellChildLeafProbe::Mapped => {
            return mapped_probe_error("consumable spell child pointer");
        }
    })
}

fn spell_source_leaf(
    source: &SpellDocumentSource,
    probe: SpellLeafProbe,
    case_id: &str,
) -> Result<SourcePresence<SourceLeafValue>, CoverageContractError> {
    let SpellDocumentSource::Standalone(source) = source else {
        return Err(error(
            CoverageFailureCode::DtoMismatch,
            "top-level spell receipt parsed a consumable child",
        ));
    };
    spell_item_leaf(source, probe, case_id)
}

fn spell_definition_leaf(
    definition: &SpellDefinition,
    probe: SpellLeafProbe,
    case_id: &str,
) -> Result<SourcePresence<SourceLeafValue>, CoverageContractError> {
    match probe {
        SpellLeafProbe::Rank => spell_fact_u8_leaf(&definition.classification, |classification| {
            &classification.rank
        }),
        SpellLeafProbe::DamageFormula => {
            let (key, _) = spell_member_for_case(probe, case_id)?;
            spell_damage_formula_leaf(&definition.damage, key)
        }
        SpellLeafProbe::OverlaySort => {
            let (key, _) = spell_member_for_case(probe, case_id)?;
            spell_overlay_sort_leaf(&definition.overlays, key)
        }
        SpellLeafProbe::RuleKey => {
            let (_, ordinal) = spell_member_for_case(probe, case_id)?;
            spell_rule_key_leaf(&definition.rules, ordinal)
        }
        SpellLeafProbe::StandaloneLocation => {
            unsupported_fact_leaf(&definition.provenance.standalone_location)
        }
        SpellLeafProbe::Image => {
            spell_fact_string_like_leaf(&definition.source_context.image, String::as_str)
        }
        SpellLeafProbe::PublicationLicense => spell_fact_string_like_leaf(
            &definition.source_context.publication_license,
            PublicationLicense::as_str,
        ),
        SpellLeafProbe::Mapped => mapped_probe_error("canonical spell leaf"),
    }
}

fn spell_item_leaf(
    source: &SpellItemSource,
    probe: SpellLeafProbe,
    case_id: &str,
) -> Result<SourcePresence<SourceLeafValue>, CoverageContractError> {
    match probe {
        SpellLeafProbe::Rank => spell_fact_u8_leaf(&source.classification, |classification| {
            &classification.rank
        }),
        SpellLeafProbe::DamageFormula => {
            let (key, _) = spell_member_for_case(probe, case_id)?;
            spell_damage_formula_leaf(&source.damage, key)
        }
        SpellLeafProbe::OverlaySort => {
            let (key, _) = spell_member_for_case(probe, case_id)?;
            spell_overlay_sort_leaf(&source.overlays, key)
        }
        SpellLeafProbe::RuleKey => {
            let (_, ordinal) = spell_member_for_case(probe, case_id)?;
            spell_rule_key_leaf(&source.rules, ordinal)
        }
        SpellLeafProbe::StandaloneLocation => unsupported_fact_leaf(&source.location_provenance),
        SpellLeafProbe::Image => spell_fact_string_like_leaf(&source.image, String::as_str),
        SpellLeafProbe::PublicationLicense => {
            spell_fact_string_like_leaf(&source.publication_license, PublicationLicense::as_str)
        }
        SpellLeafProbe::Mapped => mapped_probe_error("spell DTO leaf"),
    }
}

fn consumable_spell_child_source_leaf(
    source: &SpellDocumentSource,
    probe: ConsumableSpellChildLeafProbe,
    _case_id: &str,
) -> Result<SourcePresence<SourceLeafValue>, CoverageContractError> {
    let SpellDocumentSource::ConsumableChild(source) = source else {
        return Err(error(
            CoverageFailureCode::DtoMismatch,
            "consumable-child receipt parsed a standalone spell",
        ));
    };
    match probe {
        ConsumableSpellChildLeafProbe::ChildId => scalar_string_leaf(source.spell.id.clone()),
        ConsumableSpellChildLeafProbe::StandaloneLocator => {
            spell_fact_string_like_leaf(&source.standalone_locator, |locator| locator.as_str())
        }
        ConsumableSpellChildLeafProbe::HeightenedRank => {
            spell_fact_u8_leaf(&source.spell.location, |location| &location.heightened_rank)
        }
        ConsumableSpellChildLeafProbe::LocationValue => {
            spell_fact_string_leaf(&source.spell.location, |location| &location.value)
        }
        ConsumableSpellChildLeafProbe::Image => {
            spell_fact_string_like_leaf(&source.spell.image, String::as_str)
        }
        ConsumableSpellChildLeafProbe::Slug => {
            spell_fact_string_like_leaf(&source.spell.slug, String::as_str)
        }
        ConsumableSpellChildLeafProbe::PublicationLicense => spell_fact_string_like_leaf(
            &source.spell.publication_license,
            PublicationLicense::as_str,
        ),
        ConsumableSpellChildLeafProbe::PublicationRemaster => {
            spell_fact_bool_leaf(&source.spell.publication_remaster)
        }
        ConsumableSpellChildLeafProbe::PublicationTitle => {
            spell_fact_string_like_leaf(&source.spell.publication_title, String::as_str)
        }
        ConsumableSpellChildLeafProbe::Rarity => {
            spell_fact_string_like_leaf(&source.spell.rarity, |rarity| rarity.as_str())
        }
        ConsumableSpellChildLeafProbe::SortProvenance => Err(error(
            CoverageFailureCode::DtoMismatch,
            "provenance-only child sort has no semantic DTO owner",
        )),
        ConsumableSpellChildLeafProbe::Mapped => mapped_probe_error("consumable spell DTO leaf"),
    }
}

fn consumable_spell_child_canonical_leaf(
    child: &ConsumableSpellChild,
    probe: ConsumableSpellChildLeafProbe,
    _case_id: &str,
) -> Result<SourcePresence<SourceLeafValue>, CoverageContractError> {
    match probe {
        ConsumableSpellChildLeafProbe::ChildId => {
            scalar_string_leaf(child.child_id.as_str().to_string())
        }
        ConsumableSpellChildLeafProbe::StandaloneLocator => {
            spell_fact_string_like_leaf(&child.standalone_locator, |locator| locator.as_str())
        }
        ConsumableSpellChildLeafProbe::HeightenedRank => {
            spell_fact_u8_leaf(&child.location, |location| &location.heightened_rank)
        }
        ConsumableSpellChildLeafProbe::LocationValue => {
            spell_fact_string_leaf(&child.location, |location| &location.value)
        }
        ConsumableSpellChildLeafProbe::Image => {
            spell_fact_string_like_leaf(&child.definition.source_context.image, String::as_str)
        }
        ConsumableSpellChildLeafProbe::PublicationLicense => spell_fact_string_like_leaf(
            &child.definition.source_context.publication_license,
            PublicationLicense::as_str,
        ),
        ConsumableSpellChildLeafProbe::Slug
        | ConsumableSpellChildLeafProbe::PublicationRemaster
        | ConsumableSpellChildLeafProbe::PublicationTitle
        | ConsumableSpellChildLeafProbe::Rarity => {
            consumable_source_context_leaf(&child.definition.source_context, probe)
        }
        ConsumableSpellChildLeafProbe::SortProvenance => Err(error(
            CoverageFailureCode::CanonicalMismatch,
            "provenance-only child sort has no semantic canonical owner",
        )),
        ConsumableSpellChildLeafProbe::Mapped => {
            mapped_probe_error("canonical consumable spell leaf")
        }
    }
}

fn consumable_source_context_leaf(
    source: &SpellSourceContext,
    probe: ConsumableSpellChildLeafProbe,
) -> Result<SourcePresence<SourceLeafValue>, CoverageContractError> {
    let child = match &source.consumable_child {
        FactValue::Missing => return Ok(SourcePresence::Missing),
        FactValue::Null => return Ok(SourcePresence::Null),
        FactValue::Value(child) => child,
    };
    match probe {
        ConsumableSpellChildLeafProbe::Slug => {
            spell_fact_string_like_leaf(&child.slug, String::as_str)
        }
        ConsumableSpellChildLeafProbe::PublicationRemaster => {
            spell_fact_bool_leaf(&child.publication_remaster)
        }
        ConsumableSpellChildLeafProbe::PublicationTitle => {
            spell_fact_string_like_leaf(&child.publication_title, String::as_str)
        }
        ConsumableSpellChildLeafProbe::Rarity => {
            spell_fact_string_like_leaf(&child.rarity, |rarity| rarity.as_str())
        }
        _ => Err(error(
            CoverageFailureCode::CanonicalMismatch,
            "leaf is not owned by consumable spell source context",
        )),
    }
}

fn spell_fact_u8_leaf<T>(
    outer: &SpellFact<T>,
    select: impl FnOnce(&T) -> &SpellFact<u8>,
) -> Result<SourcePresence<SourceLeafValue>, CoverageContractError> {
    let fact = match outer {
        FactValue::Missing => return Ok(SourcePresence::Missing),
        FactValue::Null => return Ok(SourcePresence::Null),
        FactValue::Value(SpellSourceValue::Unsupported(value)) => {
            return unsupported_spell_source_value_leaf(value);
        }
        FactValue::Value(SpellSourceValue::Known(value)) => select(value),
    };
    match fact {
        FactValue::Missing => Ok(SourcePresence::Missing),
        FactValue::Null => Ok(SourcePresence::Null),
        FactValue::Value(SpellSourceValue::Known(value)) => scalar_number_leaf(i64::from(*value)),
        FactValue::Value(SpellSourceValue::Unsupported(value)) => {
            unsupported_spell_source_value_leaf(value)
        }
    }
}

fn spell_fact_string_leaf<T>(
    outer: &SpellFact<T>,
    select: impl FnOnce(&T) -> &SpellFact<String>,
) -> Result<SourcePresence<SourceLeafValue>, CoverageContractError> {
    let fact = match outer {
        FactValue::Missing => return Ok(SourcePresence::Missing),
        FactValue::Null => return Ok(SourcePresence::Null),
        FactValue::Value(SpellSourceValue::Unsupported(value)) => {
            return unsupported_spell_source_value_leaf(value);
        }
        FactValue::Value(SpellSourceValue::Known(value)) => select(value),
    };
    spell_fact_string_like_leaf(fact, String::as_str)
}

fn spell_fact_string_like_leaf<T>(
    fact: &SpellFact<T>,
    value: impl FnOnce(&T) -> &str,
) -> Result<SourcePresence<SourceLeafValue>, CoverageContractError> {
    match fact {
        FactValue::Missing => Ok(SourcePresence::Missing),
        FactValue::Null => Ok(SourcePresence::Null),
        FactValue::Value(SpellSourceValue::Known(known)) => {
            scalar_string_leaf(value(known).to_string())
        }
        FactValue::Value(SpellSourceValue::Unsupported(value)) => {
            unsupported_spell_source_value_leaf(value)
        }
    }
}

fn spell_fact_bool_leaf(
    fact: &SpellFact<bool>,
) -> Result<SourcePresence<SourceLeafValue>, CoverageContractError> {
    match fact {
        FactValue::Missing => Ok(SourcePresence::Missing),
        FactValue::Null => Ok(SourcePresence::Null),
        FactValue::Value(SpellSourceValue::Known(value)) => {
            source_leaf_value(Value::Bool(*value), None, None, None)
        }
        FactValue::Value(SpellSourceValue::Unsupported(value)) => {
            unsupported_spell_source_value_leaf(value)
        }
    }
}

fn spell_damage_formula_leaf(
    damage: &SpellFact<Vec<atlas_record::SpellOrderedMember<atlas_record::SpellDamage>>>,
    key: &str,
) -> Result<SourcePresence<SourceLeafValue>, CoverageContractError> {
    let members = match damage {
        FactValue::Missing => return Ok(SourcePresence::Missing),
        FactValue::Null => return Ok(SourcePresence::Null),
        FactValue::Value(SpellSourceValue::Unsupported(value)) => {
            return unsupported_spell_source_value_leaf(value);
        }
        FactValue::Value(SpellSourceValue::Known(value)) => value,
    };
    let member = members
        .iter()
        .find(|member| member.key == key)
        .ok_or_else(|| {
            error(
                CoverageFailureCode::ReaderNotObserved,
                format!("damage member {key} is missing"),
            )
        })?;
    spell_fact_map_string_leaf(&member.value.formula, key)
}

fn spell_overlay_sort_leaf(
    overlays: &SpellFact<Vec<atlas_record::SpellOverlay>>,
    key: &str,
) -> Result<SourcePresence<SourceLeafValue>, CoverageContractError> {
    let overlays = match overlays {
        FactValue::Missing => return Ok(SourcePresence::Missing),
        FactValue::Null => return Ok(SourcePresence::Null),
        FactValue::Value(SpellSourceValue::Unsupported(value)) => {
            return unsupported_spell_source_value_leaf(value);
        }
        FactValue::Value(SpellSourceValue::Known(value)) => value,
    };
    let overlay = overlays
        .iter()
        .find(|overlay| overlay.key == key)
        .ok_or_else(|| {
            error(
                CoverageFailureCode::ReaderNotObserved,
                format!("overlay member {key} is missing"),
            )
        })?;
    match &overlay.sort {
        FactValue::Missing => Ok(SourcePresence::Missing),
        FactValue::Null => Ok(SourcePresence::Null),
        FactValue::Value(SpellSourceValue::Known(value)) => Ok(map_number_leaf(key, *value)),
        FactValue::Value(SpellSourceValue::Unsupported(value)) => {
            unsupported_spell_source_value_leaf(value)
        }
    }
}

fn spell_rule_key_leaf(
    rules: &SpellFact<Vec<atlas_record::SpellRuleElement>>,
    ordinal: usize,
) -> Result<SourcePresence<SourceLeafValue>, CoverageContractError> {
    let rules = match rules {
        FactValue::Missing => return Ok(SourcePresence::Missing),
        FactValue::Null => return Ok(SourcePresence::Null),
        FactValue::Value(SpellSourceValue::Unsupported(value)) => {
            return unsupported_spell_source_value_leaf(value);
        }
        FactValue::Value(SpellSourceValue::Known(value)) => value,
    };
    let rule = rules.get(ordinal).ok_or_else(|| {
        error(
            CoverageFailureCode::ReaderNotObserved,
            format!("rule ordinal {ordinal} is missing"),
        )
    })?;
    array_string_leaf(
        &format!("{ordinal}:{}", rule.authored_key),
        ordinal,
        rule.authored_key.clone(),
    )
}

fn spell_fact_map_string_leaf(
    fact: &SpellFact<String>,
    key: &str,
) -> Result<SourcePresence<SourceLeafValue>, CoverageContractError> {
    match fact {
        FactValue::Missing => Ok(SourcePresence::Missing),
        FactValue::Null => Ok(SourcePresence::Null),
        FactValue::Value(SpellSourceValue::Known(value)) => map_string_leaf(key, value.clone()),
        FactValue::Value(SpellSourceValue::Unsupported(value)) => {
            unsupported_spell_source_value_leaf(value)
        }
    }
}

fn unsupported_fact_leaf(
    fact: &FactValue<UnsupportedSourceValue>,
) -> Result<SourcePresence<SourceLeafValue>, CoverageContractError> {
    match fact {
        FactValue::Missing => Ok(SourcePresence::Missing),
        FactValue::Null => Ok(SourcePresence::Null),
        FactValue::Value(value) => unsupported_spell_source_value_leaf(value),
    }
}

fn unsupported_spell_source_value_leaf(
    value: &UnsupportedSourceValue,
) -> Result<SourcePresence<SourceLeafValue>, CoverageContractError> {
    let parsed =
        serde_json::from_str(&value.value).unwrap_or_else(|_| Value::String(value.value.clone()));
    let json_type = source_json_type(&parsed);
    Ok(SourcePresence::Value(SourceLeafValue {
        json_type,
        value: Some(parsed.clone()),
        stable_digest: None,
        member_kind: None,
        member_identity: None,
        ordinal: None,
        multiplicity: 1,
        unsupported: Some(TypedUnsupportedValue {
            value: parsed,
            reason: format!("{:?}", value.reason),
        }),
    }))
}

fn scalar_string_leaf(
    value: String,
) -> Result<SourcePresence<SourceLeafValue>, CoverageContractError> {
    source_leaf_value(Value::String(value), None, None, None)
}

fn scalar_number_leaf(
    value: i64,
) -> Result<SourcePresence<SourceLeafValue>, CoverageContractError> {
    source_leaf_value(Value::from(value), None, None, None)
}

fn map_string_leaf(
    key: &str,
    value: String,
) -> Result<SourcePresence<SourceLeafValue>, CoverageContractError> {
    source_leaf_value(
        Value::String(value),
        Some(SourceMemberKind::Map),
        Some(key.to_string()),
        None,
    )
}

fn array_string_leaf(
    identity: &str,
    ordinal: usize,
    value: String,
) -> Result<SourcePresence<SourceLeafValue>, CoverageContractError> {
    source_leaf_value(
        Value::String(value),
        Some(SourceMemberKind::Array),
        Some(identity.to_string()),
        Some(ordinal),
    )
}

fn capture_hazard_disable(
    identity: SourceLeafIdentity,
    fixture: ResolvedFixture,
    ledger: &SourceLeafCoverageLedger,
) -> Result<SourceLeafReceipt, CoverageContractError> {
    let raw_value = fixture
        .raw
        .pointer("/system/details/disable")
        .cloned()
        .ok_or_else(|| {
            error(
                CoverageFailureCode::ReaderNotObserved,
                "missing hazard disable",
            )
        })?;
    validate_hazard_excerpt(&fixture, &raw_value)?;
    let source_text = raw_value.as_str().ok_or_else(|| {
        error(
            CoverageFailureCode::ReaderNotObserved,
            "hazard disable is not a string",
        )
    })?;
    let mut mutated_raw = fixture.raw.clone();
    let mutated_text = format!("{source_text}\u{241f}source-leaf-mutation");
    let target = mutated_raw
        .pointer_mut("/system/details/disable")
        .ok_or_else(|| {
            error(
                CoverageFailureCode::ReaderNotObserved,
                "validated hazard disable pointer disappeared before mutation",
            )
        })?;
    *target = Value::String(mutated_text.clone());
    let actual = run_hazard_pipeline(&fixture, fixture.raw.clone(), ledger, &identity)?;
    let mutation = run_hazard_pipeline(&fixture, mutated_raw, ledger, &identity)?;
    let source = dto_hazard_string(&hazard_lifecycle(&actual.source_dto)?.disable);
    let source_mutation = dto_hazard_string(&hazard_lifecycle(&mutation.source_dto)?.disable);
    let hydration_source =
        dto_hazard_string(&hazard_lifecycle(&actual.hydration_source_dto)?.disable);
    let hydration_source_mutation =
        dto_hazard_string(&hazard_lifecycle(&mutation.hydration_source_dto)?.disable);
    let hydration_text = actual
        .hydration_raw
        .pointer("/system/details/disable")
        .and_then(Value::as_str)
        .ok_or_else(|| {
            error(
                CoverageFailureCode::ArtifactHydrationMismatch,
                "baseline hydration source omitted hazard disable",
            )
        })?;
    let hydration_mutated_text = mutation
        .hydration_raw
        .pointer("/system/details/disable")
        .and_then(Value::as_str)
        .ok_or_else(|| {
            error(
                CoverageFailureCode::ArtifactHydrationMismatch,
                "grouped hydration source omitted hazard disable",
            )
        })?;
    let observations = vec![
        presence_stage(
            FinalOwnerStage::SourceDto,
            "HazardLifecycleSource.disable",
            "source::dto::parse_hazard_source",
            source.clone(),
            &source_mutation,
        )?,
        presence_stage(
            FinalOwnerStage::Canonical,
            "HazardRecord.lifecycle.disable",
            "source::hazard_core::convert_hazard_core",
            hazard_rich_document_leaf(
                source_text,
                &actual.canonical.lifecycle,
                "/system/details/disable",
            )?,
            &hazard_rich_document_leaf(
                &mutated_text,
                &mutation.canonical.lifecycle,
                "/system/details/disable",
            )?,
        )?,
        presence_stage(
            FinalOwnerStage::PostProjection,
            "IndexBuildInput.canonical_bodies[].lifecycle.disable",
            "index_build_input::index_build_input",
            hazard_rich_document_leaf(
                source_text,
                &actual.post_projection.lifecycle,
                "/system/details/disable",
            )?,
            &hazard_rich_document_leaf(
                &mutated_text,
                &mutation.post_projection.lifecycle,
                "/system/details/disable",
            )?,
        )?,
        owner_presence_stage(
            FinalOwnerStage::ArtifactHydration,
            "RetrievedRecord.body.RecordBody::Hazard.HazardRecord lifecycle/classification/publication field ($.system.details.disable)",
            "SqliteIndexReader::load_hydrated_records_by_key grouped hazard disable",
            &hydration_source,
            &hydration_source_mutation,
            (
                hazard_rich_document_leaf(
                    hydration_text,
                    &actual.hydration.lifecycle,
                    "/system/details/disable",
                )?,
                Some(hazard_rich_document_leaf(
                    hydration_mutated_text,
                    &mutation.hydration.lifecycle,
                    "/system/details/disable",
                )?),
            ),
            (
                &owner_fingerprint("baseline hydrated lifecycle", &actual.hydration.lifecycle),
                &owner_fingerprint("grouped hydrated lifecycle", &mutation.hydration.lifecycle),
            ),
        )?,
    ];
    sealed_receipt(
        identity,
        fixture.reference,
        source,
        source_mutation,
        HAZARD_DISABLE_READER,
        "sealed::ActorHazardDisable",
        observations,
    )
}

fn capture_hazard_folder(
    identity: SourceLeafIdentity,
    fixture: ResolvedFixture,
) -> Result<SourceLeafReceipt, CoverageContractError> {
    let raw_value = fixture.raw.pointer("/folder").cloned().ok_or_else(|| {
        error(
            CoverageFailureCode::ReaderNotObserved,
            "missing hazard editor folder",
        )
    })?;
    validate_hazard_excerpt(&fixture, &raw_value)?;
    let folder = raw_value.as_str().ok_or_else(|| {
        error(
            CoverageFailureCode::ReaderNotObserved,
            "hazard editor folder is not a string",
        )
    })?;
    let mut mutated_raw = fixture.raw.clone();
    let mutated_folder = format!("{folder}-source-leaf-mutation");
    let target = mutated_raw.pointer_mut("/folder").ok_or_else(|| {
        error(
            CoverageFailureCode::ReaderNotObserved,
            "validated hazard folder pointer disappeared before mutation",
        )
    })?;
    *target = Value::String(mutated_folder.clone());
    let actual = run_hazard_pipeline_core(&fixture, fixture.raw.clone())?;
    let mutation = run_hazard_pipeline_core(&fixture, mutated_raw)?;
    let source = dto_hazard_string(&actual.source_dto.source.folder);
    let source_mutation = dto_hazard_string(&mutation.source_dto.source.folder);
    let observation = presence_stage(
        FinalOwnerStage::DurableProvenance,
        "HazardRecord.provenance.source_folder",
        "HazardProvenance.source_folder",
        canonical_hazard_string(&actual.canonical.provenance.source_folder),
        &canonical_hazard_string(&mutation.canonical.provenance.source_folder),
    )?;
    let reader = ActualReadEvidence {
        reader_id: HAZARD_FOLDER_READER.to_string(),
        accessor_binding: "sealed::ActorHazardFolder".to_string(),
        purpose: SourceAccessorPurpose::ProvenanceReader,
        mutation_digest: digest_serializable(&source_mutation)?,
    };
    let semantic_output = Some(SemanticOutputObservation {
        accessor_binding: "HazardRecord gameplay/presentation projection excludes source_folder"
            .to_string(),
        observed: false,
        mutation_observed: true,
    });
    let observations = vec![observation];
    let evidence_digest = evidence_digest(
        &identity,
        &fixture.reference,
        &source,
        &reader,
        &observations,
        &semantic_output,
    )?;
    Ok(SourceLeafReceipt {
        identity,
        fixture: fixture.reference,
        source,
        reader,
        observations,
        semantic_output,
        evidence_digest,
    })
}

fn capture_hazard_temporary_maximum(
    identity: SourceLeafIdentity,
    fixture: ResolvedFixture,
    ledger: &SourceLeafCoverageLedger,
) -> Result<SourceLeafReceipt, CoverageContractError> {
    let raw_value = fixture
        .raw
        .pointer("/system/attributes/hp/tempmax")
        .cloned()
        .ok_or_else(|| {
            error(
                CoverageFailureCode::ReaderNotObserved,
                "missing hazard hp.tempmax",
            )
        })?;
    validate_hazard_excerpt(&fixture, &raw_value)?;
    let value = raw_value.as_i64().ok_or_else(|| {
        error(
            CoverageFailureCode::ReaderNotObserved,
            "hazard hp.tempmax is not an integer",
        )
    })?;
    let mut mutated_raw = fixture.raw.clone();
    let target = mutated_raw
        .pointer_mut("/system/attributes/hp/tempmax")
        .ok_or_else(|| {
            error(
                CoverageFailureCode::ReaderNotObserved,
                "validated hazard tempmax pointer disappeared before mutation",
            )
        })?;
    *target = Value::from(value + 101);
    let actual = run_hazard_pipeline(&fixture, fixture.raw.clone(), ledger, &identity)?;
    let mutation = run_hazard_pipeline(&fixture, mutated_raw, ledger, &identity)?;
    let source = number_leaf(value);
    let source_mutation = number_leaf(value + 101);
    let hydration_source =
        dto_hazard_number(&hazard_hit_points(&actual.hydration_source_dto)?.temporary_maximum);
    let hydration_source_mutation =
        dto_hazard_number(&hazard_hit_points(&mutation.hydration_source_dto)?.temporary_maximum);
    let observations = vec![
        presence_stage(
            FinalOwnerStage::SourceDto,
            "HazardHitPointsSource.temporary_maximum",
            "source::dto::parse_hazard_source",
            dto_hazard_number(&hazard_hit_points(&actual.source_dto)?.temporary_maximum),
            &dto_hazard_number(&hazard_hit_points(&mutation.source_dto)?.temporary_maximum),
        )?,
        presence_stage(
            FinalOwnerStage::Canonical,
            "HazardHitPoints.source_metadata.temporary_maximum",
            "source::hazard_core::convert_hit_points",
            hazard_tempmax_source_metadata(&actual.canonical)?,
            &hazard_tempmax_source_metadata(&mutation.canonical)?,
        )?,
        presence_stage(
            FinalOwnerStage::PostProjection,
            "IndexBuildInput.canonical_bodies[].defenses.hit_points.source_metadata.temporary_maximum",
            "index_build_input::index_build_input",
            hazard_tempmax_source_metadata(&actual.post_projection)?,
            &hazard_tempmax_source_metadata(&mutation.post_projection)?,
        )?,
        owner_presence_stage(
            FinalOwnerStage::ArtifactHydration,
            "RetrievedRecord.body.RecordBody::Hazard.HazardRecord.defenses.hit_points.source_metadata.temporary_maximum",
            "SqliteIndexReader::load_hydrated_records_by_key grouped hazard hp.tempmax",
            &hydration_source,
            &hydration_source_mutation,
            (
                hazard_tempmax_source_metadata(&actual.hydration)?,
                Some(hazard_tempmax_source_metadata(&mutation.hydration)?),
            ),
            (
                &owner_fingerprint("baseline hydrated defenses", &actual.hydration.defenses),
                &owner_fingerprint("grouped hydrated defenses", &mutation.hydration.defenses),
            ),
        )?,
    ];
    sealed_receipt(
        identity,
        fixture.reference,
        source,
        source_mutation,
        HAZARD_TEMP_MAX_READER,
        "sealed::ActorHazardTemporaryMaximum",
        observations,
    )
}

fn capture_hazard_action_type(
    identity: SourceLeafIdentity,
    fixture: ResolvedFixture,
    ledger: &SourceLeafCoverageLedger,
) -> Result<SourceLeafReceipt, CoverageContractError> {
    let item_id = embedded_item_id(&fixture)?;
    let item_ordinal = raw_item_ordinal(&fixture.raw, item_id)?;
    let pointer = format!("/items/{item_ordinal}/system/actionType/value");
    let raw_value = fixture.raw.pointer(&pointer).cloned().ok_or_else(|| {
        error(
            CoverageFailureCode::ReaderNotObserved,
            "missing embedded hazard action type",
        )
    })?;
    validate_hazard_excerpt(&fixture, &raw_value)?;
    let action_type = raw_value.as_str().ok_or_else(|| {
        error(
            CoverageFailureCode::ReaderNotObserved,
            "embedded hazard action type is not a string",
        )
    })?;
    let mutated_action_type = if action_type == "passive" {
        "reaction"
    } else {
        "passive"
    };
    let mut mutated_raw = fixture.raw.clone();
    let target = mutated_raw.pointer_mut(&pointer).ok_or_else(|| {
        error(
            CoverageFailureCode::ReaderNotObserved,
            "validated hazard action type pointer disappeared before mutation",
        )
    })?;
    *target = Value::String(mutated_action_type.to_string());
    let actual = run_hazard_pipeline(&fixture, fixture.raw.clone(), ledger, &identity)?;
    let mutation = run_hazard_pipeline(&fixture, mutated_raw, ledger, &identity)?;
    let source = dto_hazard_string(&hazard_item(&actual.source_dto, item_id)?.action.action_type);
    let source_mutation = dto_hazard_string(
        &hazard_item(&mutation.source_dto, item_id)?
            .action
            .action_type,
    );
    let baseline_hydration_item_id = hydration_item_id(&actual, item_ordinal)?;
    let hydration_mutation_item_id = hydration_item_id(&mutation, item_ordinal)?;
    let hydration_source = dto_hazard_string(
        &hazard_item(&actual.hydration_source_dto, baseline_hydration_item_id)?
            .action
            .action_type,
    );
    let hydration_source_mutation = dto_hazard_string(
        &hazard_item(&mutation.hydration_source_dto, hydration_mutation_item_id)?
            .action
            .action_type,
    );
    let observations = vec![
        presence_stage(
            FinalOwnerStage::SourceDto,
            "HazardActionSource.action_type",
            "source::dto::parse_hazard_source",
            source.clone(),
            &source_mutation,
        )?,
        presence_stage(
            FinalOwnerStage::Canonical,
            "HazardActionCapability.action_type",
            "source::hazard_entities::convert_action",
            hazard_action_type(&actual.canonical, item_id)?,
            &hazard_action_type(&mutation.canonical, item_id)?,
        )?,
        presence_stage(
            FinalOwnerStage::PostProjection,
            "IndexBuildInput.canonical_bodies[].embedded_entities.entities[].action_type",
            "index_build_input::index_build_input",
            hazard_action_type(&actual.post_projection, item_id)?,
            &hazard_action_type(&mutation.post_projection, item_id)?,
        )?,
        owner_presence_stage(
            FinalOwnerStage::ArtifactHydration,
            "RetrievedRecord.body.RecordBody::Hazard.HazardCapability typed or unsupported field ($.system.actionType.value)",
            "SqliteIndexReader::load_hydrated_records_by_key grouped hazard action type",
            &hydration_source,
            &hydration_source_mutation,
            (
                hazard_action_type(&actual.hydration, baseline_hydration_item_id)?,
                Some(hazard_action_type(
                    &mutation.hydration,
                    hydration_mutation_item_id,
                )?),
            ),
            (
                &owner_fingerprint("baseline hydrated action", &actual.hydration),
                &owner_fingerprint("grouped hydrated action", &mutation.hydration),
            ),
        )?,
    ];
    sealed_receipt(
        identity,
        fixture.reference,
        source,
        source_mutation,
        HAZARD_ACTION_TYPE_READER,
        "sealed::EmbeddedHazardActionType",
        observations,
    )
}

fn capture_hazard_damage(
    identity: SourceLeafIdentity,
    fixture: ResolvedFixture,
    ledger: &SourceLeafCoverageLedger,
) -> Result<SourceLeafReceipt, CoverageContractError> {
    let item_id = embedded_item_id(&fixture)?;
    let item_ordinal = raw_item_ordinal(&fixture.raw, item_id)?;
    let damage_map = fixture
        .raw
        .pointer(&format!("/items/{item_ordinal}/system/damageRolls"))
        .and_then(Value::as_object)
        .ok_or_else(|| {
            error(
                CoverageFailureCode::ReaderNotObserved,
                "missing embedded hazard damageRolls map",
            )
        })?;
    let source_key = damage_map.keys().next().cloned().ok_or_else(|| {
        error(
            CoverageFailureCode::ReaderNotObserved,
            "embedded hazard damageRolls map is empty",
        )
    })?;
    let pointer = format!(
        "/items/{item_ordinal}/system/damageRolls/{}/damage",
        escape_json_pointer(&source_key)
    );
    let raw_value = fixture.raw.pointer(&pointer).cloned().ok_or_else(|| {
        error(
            CoverageFailureCode::ReaderNotObserved,
            "missing embedded hazard damage formula",
        )
    })?;
    validate_hazard_excerpt(&fixture, &raw_value)?;
    let damage = raw_value.as_str().ok_or_else(|| {
        error(
            CoverageFailureCode::ReaderNotObserved,
            "embedded hazard damage formula is not a string",
        )
    })?;
    let mutated_damage = format!("{damage}+1");
    let mut mutated_raw = fixture.raw.clone();
    let target = mutated_raw.pointer_mut(&pointer).ok_or_else(|| {
        error(
            CoverageFailureCode::ReaderNotObserved,
            "validated hazard damage formula pointer disappeared before mutation",
        )
    })?;
    *target = Value::String(mutated_damage);
    let actual = run_hazard_pipeline(&fixture, fixture.raw.clone(), ledger, &identity)?;
    let mutation = run_hazard_pipeline(&fixture, mutated_raw, ledger, &identity)?;
    let source = dto_hazard_damage(&actual.source_dto, item_id, &source_key)?;
    let source_mutation = dto_hazard_damage(&mutation.source_dto, item_id, &source_key)?;
    let baseline_hydration_item_id = hydration_item_id(&actual, item_ordinal)?;
    let hydration_mutation_item_id = hydration_item_id(&mutation, item_ordinal)?;
    let hydration_source = dto_hazard_damage(
        &actual.hydration_source_dto,
        baseline_hydration_item_id,
        &source_key,
    )?;
    let hydration_source_mutation = dto_hazard_damage(
        &mutation.hydration_source_dto,
        hydration_mutation_item_id,
        &source_key,
    )?;
    let observations = vec![
        presence_stage(
            FinalOwnerStage::SourceDto,
            "HazardDamageSource.damage",
            "source::dto::parse_hazard_source",
            source.clone(),
            &source_mutation,
        )?,
        presence_stage(
            FinalOwnerStage::Canonical,
            "HazardStrikeCapability.damage_rolls[].damage",
            "source::hazard_entities::convert_damage",
            hazard_damage(&actual.canonical, item_id, &source_key)?,
            &hazard_damage(&mutation.canonical, item_id, &source_key)?,
        )?,
        presence_stage(
            FinalOwnerStage::PostProjection,
            "IndexBuildInput.canonical_bodies[].embedded_entities.entities[].damage_rolls[].damage",
            "index_build_input::index_build_input",
            hazard_damage(&actual.post_projection, item_id, &source_key)?,
            &hazard_damage(&mutation.post_projection, item_id, &source_key)?,
        )?,
        owner_presence_stage(
            FinalOwnerStage::ArtifactHydration,
            "RetrievedRecord.body.RecordBody::Hazard.HazardStrikeCapability.damage_rolls[] ($.system.damageRolls.*.damage)",
            "SqliteIndexReader::load_hydrated_records_by_key grouped hazard damage",
            &hydration_source,
            &hydration_source_mutation,
            (
                hazard_damage(&actual.hydration, baseline_hydration_item_id, &source_key)?,
                Some(hazard_damage(
                    &mutation.hydration,
                    hydration_mutation_item_id,
                    &source_key,
                )?),
            ),
            (
                &owner_fingerprint("baseline hydrated damage", &actual.hydration),
                &owner_fingerprint("grouped hydrated damage", &mutation.hydration),
            ),
        )?,
    ];
    sealed_receipt(
        identity,
        fixture.reference,
        source,
        source_mutation,
        HAZARD_DAMAGE_READER,
        "sealed::EmbeddedHazardDamage",
        observations,
    )
}

fn capture_hazard_exact_leaf(
    identity: SourceLeafIdentity,
    fixture: ResolvedFixture,
    disposition: HazardExactDisposition,
    ledger: &SourceLeafCoverageLedger,
) -> Result<SourceLeafReceipt, CoverageContractError> {
    let occurrence = first_exact_hazard_leaf(&fixture, &identity)?;
    let excerpt = occurrence
        .value
        .clone()
        .unwrap_or_else(|| serde_json::json!({ "state": "missing" }));
    validate_hazard_excerpt(&fixture, &excerpt)?;
    let source = occurrence.receipt_value(disposition);

    let mut mutated_raw = fixture.raw.clone();
    let mutated_value = occurrence.value.as_ref().map_or_else(
        || Value::String("source-leaf-missing-mutation".to_string()),
        mutate_hazard_leaf_value,
    );
    set_json_pointer(&mut mutated_raw, &occurrence.pointer, mutated_value.clone())?;
    let mut mutation_reference = fixture.reference.clone();
    if identity.selector.role == SourceDocumentRole::Embedded
        && identity.normalized_path == "$._id"
        && let Some(mutated_id) = mutated_value.as_str()
        && let Some((parent, _)) = mutation_reference.record_key.rsplit_once("#item:")
    {
        mutation_reference.record_key = format!("{parent}#item:{mutated_id}");
    }
    let mutation_occurrence = first_exact_hazard_leaf(
        &ResolvedFixture {
            reference: mutation_reference,
            serialized: serde_json::to_vec(&mutated_raw).map_err(|message| {
                error(CoverageFailureCode::InvalidContract, message.to_string())
            })?,
            raw: mutated_raw.clone(),
        },
        &identity,
    )?;
    let source_mutation = mutation_occurrence.receipt_value(disposition);

    let actual = if disposition == HazardExactDisposition::ProvenanceOnly {
        run_hazard_pipeline_with_baseline_hydration(&fixture, fixture.raw.clone())?
    } else {
        run_hazard_pipeline(&fixture, fixture.raw.clone(), ledger, &identity)?
    };
    let mutation = if disposition == HazardExactDisposition::ProvenanceOnly {
        run_hazard_pipeline_with_baseline_hydration(&fixture, mutated_raw)
    } else {
        run_hazard_pipeline(&fixture, mutated_raw, ledger, &identity)
    };
    let classification_rejected = identity.normalized_path == "$.type" && mutation.is_err();
    let actual_owners =
        exact_hazard_owner_fingerprints(&actual, &identity, &occurrence, disposition)?;
    let actual_owner_values =
        exact_hazard_owner_values(&actual, &identity, &occurrence, disposition)?;
    let mutation_owners = mutation
        .as_ref()
        .ok()
        .map(|candidate| {
            exact_hazard_owner_fingerprints(candidate, &identity, &occurrence, disposition)
        })
        .transpose()?;
    let mutation_owner_values = mutation
        .as_ref()
        .ok()
        .map(|candidate| {
            exact_hazard_owner_values(candidate, &identity, &mutation_occurrence, disposition)
        })
        .transpose()?;
    let actual_hydration =
        exact_hazard_hydration_owner_evidence(&actual, &fixture, &identity, disposition)?;
    let mutation_hydration = if disposition == HazardExactDisposition::ProvenanceOnly {
        None
    } else {
        mutation
            .as_ref()
            .ok()
            .map(|candidate| {
                exact_hazard_hydration_owner_evidence(candidate, &fixture, &identity, disposition)
            })
            .transpose()?
    };
    if !classification_rejected {
        let mutation_owners = mutation_owners.as_ref().ok_or_else(|| {
            error(
                CoverageFailureCode::ReaderNotObserved,
                format!(
                    "exact hazard leaf {} mutation did not reach its typed owners",
                    identity.normalized_path
                ),
            )
        })?;
        let mut unchanged = [
            ("source_dto", actual_owners.source == mutation_owners.source),
            (
                "canonical",
                actual_owners.canonical == mutation_owners.canonical,
            ),
            (
                "post_projection",
                actual_owners.post_projection == mutation_owners.post_projection,
            ),
        ]
        .into_iter()
        .filter_map(|(stage, unchanged)| unchanged.then_some(stage))
        .collect::<Vec<_>>();
        if disposition != HazardExactDisposition::ProvenanceOnly
            && mutation_hydration.as_ref().is_some_and(|mutation| {
                actual_hydration.owner_fingerprint == mutation.owner_fingerprint
            })
        {
            unchanged.push("artifact_hydration");
        }
        if !unchanged.is_empty() {
            return Err(error(
                CoverageFailureCode::ReaderNotObserved,
                format!(
                    "exact hazard leaf {} did not change exact declared ingest accessors: {}",
                    identity.normalized_path,
                    unchanged.join(", "),
                ),
            ));
        }
    }

    let destinations =
        hazard_exact_leaf_destinations(&identity.selector, &identity.normalized_path, disposition);
    let reader_id = match disposition {
        HazardExactDisposition::Promoted => HAZARD_EXACT_PROMOTED_READER,
        HazardExactDisposition::TypedUnsupported => HAZARD_EXACT_UNSUPPORTED_READER,
        HazardExactDisposition::ProvenanceOnly => HAZARD_EXACT_PROVENANCE_READER,
    };
    if disposition == HazardExactDisposition::ProvenanceOnly {
        let mutation_pipeline = mutation.as_ref().map_err(|source_error| {
            error(
                CoverageFailureCode::ProvenanceNotDurable,
                format!("provenance mutation failed to normalize: {source_error}"),
            )
        })?;
        let semantic_fingerprint = hazard_semantic_fingerprint(&actual.canonical);
        let mutation_semantic_fingerprint =
            hazard_semantic_fingerprint(&mutation_pipeline.canonical);
        if semantic_fingerprint != mutation_semantic_fingerprint {
            return Err(error(
                CoverageFailureCode::ProvenanceNotDurable,
                format!(
                    "provenance-only hazard leaf {} leaked into typed presentation/mechanic output",
                    identity.normalized_path
                ),
            ));
        }
        let observation = owner_presence_stage(
            FinalOwnerStage::DurableProvenance,
            &destinations.canonical,
            "SqliteIndexReader baseline provenance owner plus source::normalize mutation owner",
            &source,
            &source_mutation,
            (
                actual_hydration.owner_value,
                mutation_owner_values
                    .as_ref()
                    .map(|owners| owners.canonical.clone()),
            ),
            (
                &actual_hydration.owner_fingerprint,
                mutation_owners
                    .as_ref()
                    .map_or("classification rejected", |owners| {
                        owners.canonical.as_str()
                    }),
            ),
        )?;
        let reader = ActualReadEvidence {
            reader_id: reader_id.to_string(),
            accessor_binding: "sealed::ActorHazardExactProvenanceLeaf".to_string(),
            purpose: SourceAccessorPurpose::ProvenanceReader,
            mutation_digest: digest_serializable(&source_mutation)?,
        };
        let semantic_output = Some(SemanticOutputObservation {
            accessor_binding: format!(
                "HazardRecord presentation/mechanic projection unchanged [{}]",
                digest_bytes(semantic_fingerprint.as_bytes())
            ),
            observed: false,
            mutation_observed: true,
        });
        let observations = vec![observation];
        let evidence_digest = evidence_digest(
            &identity,
            &fixture.reference,
            &source,
            &reader,
            &observations,
            &semantic_output,
        )?;
        return Ok(SourceLeafReceipt {
            identity,
            fixture: fixture.reference,
            source,
            reader,
            observations,
            semantic_output,
            evidence_digest,
        });
    }

    let mutation_rejection = mutation.as_ref().err();
    let observations = vec![
        owner_presence_or_rejected_stage(
            FinalOwnerStage::SourceDto,
            &destinations.source,
            "source::dto::parse_hazard_source exact typed owner",
            (&source, &source_mutation),
            (
                actual_owner_values.source,
                mutation_owner_values
                    .as_ref()
                    .map(|owners| owners.source.clone()),
            ),
            (
                &actual_owners.source,
                mutation_owners
                    .as_ref()
                    .map_or("classification rejected", |owners| owners.source.as_str()),
            ),
            mutation_rejection,
        )?,
        owner_presence_or_rejected_stage(
            FinalOwnerStage::Canonical,
            &destinations.canonical,
            "source::normalize::hazard exact canonical owner",
            (&source, &source_mutation),
            (
                actual_owner_values.canonical,
                mutation_owner_values
                    .as_ref()
                    .map(|owners| owners.canonical.clone()),
            ),
            (
                &actual_owners.canonical,
                mutation_owners
                    .as_ref()
                    .map_or("classification rejected", |owners| {
                        owners.canonical.as_str()
                    }),
            ),
            mutation_rejection,
        )?,
        owner_presence_or_rejected_stage(
            FinalOwnerStage::PostProjection,
            &destinations.post_projection,
            "index_build_input::index_build_input",
            (&source, &source_mutation),
            (
                actual_owner_values.post_projection,
                mutation_owner_values
                    .as_ref()
                    .map(|owners| owners.post_projection.clone()),
            ),
            (
                &actual_owners.post_projection,
                mutation_owners
                    .as_ref()
                    .map_or("classification rejected", |owners| {
                        owners.post_projection.as_str()
                    }),
            ),
            mutation_rejection,
        )?,
        owner_presence_or_rejected_stage(
            FinalOwnerStage::ArtifactHydration,
            &destinations.hydration,
            "SqliteIndexReader::load_hydrated_records_by_key RecordBody::Hazard",
            (
                &actual_hydration.expected_source,
                mutation_hydration
                    .as_ref()
                    .map_or(&source_mutation, |evidence| &evidence.expected_source),
            ),
            (
                actual_hydration.owner_value,
                mutation_hydration
                    .as_ref()
                    .map(|evidence| evidence.owner_value.clone()),
            ),
            (
                &actual_hydration.owner_fingerprint,
                mutation_hydration
                    .as_ref()
                    .map_or("mutation rejected before artifact hydration", |evidence| {
                        evidence.owner_fingerprint.as_str()
                    }),
            ),
            mutation_rejection,
        )?,
    ];
    sealed_receipt(
        identity,
        fixture.reference,
        source,
        source_mutation,
        reader_id,
        "sealed::ActorHazardExactLeaf",
        observations,
    )
}

#[derive(Debug, Clone)]
struct ExactHazardLeafOccurrence {
    pointer: String,
    value: Option<Value>,
    member_kind: Option<SourceMemberKind>,
    member_identity: Option<String>,
    ordinal: Option<usize>,
    collection_cardinality: Option<usize>,
    collection_authored_order: Option<usize>,
}

impl ExactHazardLeafOccurrence {
    fn receipt_value(
        &self,
        disposition: HazardExactDisposition,
    ) -> SourcePresence<SourceLeafValue> {
        match &self.value {
            Some(value) => self.receipt_value_for(value.clone(), disposition),
            None => SourcePresence::Missing,
        }
    }

    fn receipt_value_for(
        &self,
        value: Value,
        disposition: HazardExactDisposition,
    ) -> SourcePresence<SourceLeafValue> {
        if value.is_null() && disposition != HazardExactDisposition::TypedUnsupported {
            return SourcePresence::Null;
        }
        let json_type = source_json_type(&value);
        let unsupported = (disposition == HazardExactDisposition::TypedUnsupported).then(|| {
            TypedUnsupportedValue {
                value: value.clone(),
                reason: "exact hazard leaf retained as typed unsupported".to_string(),
            }
        });
        SourcePresence::Value(SourceLeafValue {
            json_type,
            value: Some(value),
            stable_digest: None,
            member_kind: self.member_kind,
            member_identity: self.member_identity.clone(),
            ordinal: self.ordinal,
            multiplicity: 1,
            unsupported,
        })
    }
}

fn first_exact_hazard_leaf(
    fixture: &ResolvedFixture,
    identity: &SourceLeafIdentity,
) -> Result<ExactHazardLeafOccurrence, CoverageContractError> {
    let normalized_path = identity.normalized_path.as_str();
    let path = normalized_path.strip_prefix("$.").ok_or_else(|| {
        error(
            CoverageFailureCode::ReaderNotObserved,
            format!("invalid exact hazard path {normalized_path}"),
        )
    })?;
    let segments = path.split('.').collect::<Vec<_>>();
    let (scope, pointer_prefix) = if identity.selector.role == SourceDocumentRole::Embedded {
        let item_id = fixture
            .reference
            .record_key
            .rsplit_once("#item:")
            .map(|(_, item_id)| item_id)
            .ok_or_else(|| {
                error(
                    CoverageFailureCode::FixtureNotSourceGrounded,
                    "embedded exact-leaf fixture has no item identity",
                )
            })?;
        let (index, item) = fixture
            .raw
            .pointer("/items")
            .and_then(Value::as_array)
            .and_then(|items| {
                items
                    .iter()
                    .enumerate()
                    .find(|(_, item)| item.get("_id").and_then(Value::as_str) == Some(item_id))
            })
            .ok_or_else(|| {
                error(
                    CoverageFailureCode::FixtureNotSourceGrounded,
                    "embedded exact-leaf item is absent",
                )
            })?;
        (item, format!("/items/{index}"))
    } else {
        (&fixture.raw, String::new())
    };
    let mut matches = Vec::new();
    collect_exact_hazard_leaves(scope, &segments, pointer_prefix.clone(), None, &mut matches);
    let (pointer, value, collection) = matches.into_iter().next().map_or_else(
        || {
            let pointer = missing_hazard_leaf_pointer(&segments, &pointer_prefix)?;
            Ok((pointer, None, None))
        },
        |matched| Ok((matched.pointer, Some(matched.value), matched.collection)),
    )?;
    let member_kind = collection.as_ref().map(|collection| collection.kind);
    let member_identity = collection.as_ref().map(|collection| match collection.kind {
        SourceMemberKind::Array => pointer.clone(),
        SourceMemberKind::Map | SourceMemberKind::Nested => collection.identity.clone(),
    });
    let ordinal = collection
        .as_ref()
        .and_then(|collection| (collection.kind == SourceMemberKind::Array).then_some(0));
    Ok(ExactHazardLeafOccurrence {
        member_kind,
        member_identity,
        ordinal,
        collection_cardinality: collection.as_ref().map(|value| value.cardinality),
        collection_authored_order: collection.as_ref().map(|value| value.authored_order),
        pointer,
        value,
    })
}

fn missing_hazard_leaf_pointer(
    segments: &[&str],
    pointer_prefix: &str,
) -> Result<String, CoverageContractError> {
    if segments.contains(&"*")
        || segments[..segments.len().saturating_sub(1)]
            .iter()
            .any(|segment| segment.contains("[]"))
    {
        return Err(error(
            CoverageFailureCode::ReaderNotObserved,
            "a missing exact-leaf fixture requires a non-collection source path",
        ));
    }
    Ok(segments
        .iter()
        .fold(pointer_prefix.to_string(), |pointer, segment| {
            if let Some(key) = segment.strip_suffix("[]") {
                format!("{pointer}/{}/0", escape_json_pointer(key))
            } else {
                format!("{pointer}/{}", escape_json_pointer(segment))
            }
        }))
}

fn set_json_pointer(
    root: &mut Value,
    pointer: &str,
    value: Value,
) -> Result<(), CoverageContractError> {
    let segments = pointer
        .strip_prefix('/')
        .ok_or_else(|| {
            error(
                CoverageFailureCode::ReaderNotObserved,
                "invalid JSON pointer",
            )
        })?
        .split('/')
        .map(|segment| segment.replace("~1", "/").replace("~0", "~"))
        .collect::<Vec<_>>();
    let (last, parents) = segments.split_last().ok_or_else(|| {
        error(
            CoverageFailureCode::ReaderNotObserved,
            "cannot mutate an empty JSON pointer",
        )
    })?;
    let mut current = root;
    for segment in parents {
        if current.is_array() {
            let index = segment.parse::<usize>().map_err(|_| {
                error(
                    CoverageFailureCode::ReaderNotObserved,
                    format!("nonnumeric array parent while mutating {pointer}"),
                )
            })?;
            current = current
                .as_array_mut()
                .and_then(|array| array.get_mut(index))
                .ok_or_else(|| {
                    error(
                        CoverageFailureCode::ReaderNotObserved,
                        format!("missing array parent while mutating {pointer}"),
                    )
                })?;
        } else {
            let Value::Object(object) = current else {
                return Err(error(
                    CoverageFailureCode::ReaderNotObserved,
                    format!("non-object parent while mutating {pointer}"),
                ));
            };
            current = object
                .entry(segment.clone())
                .or_insert_with(|| Value::Object(Default::default()));
        }
    }
    if current.is_array() {
        let index = last.parse::<usize>().map_err(|_| {
            error(
                CoverageFailureCode::ReaderNotObserved,
                format!("nonnumeric array leaf while mutating {pointer}"),
            )
        })?;
        let array = current.as_array_mut().ok_or_else(|| {
            error(
                CoverageFailureCode::ReaderNotObserved,
                format!("missing array leaf while mutating {pointer}"),
            )
        })?;
        if index > array.len() {
            return Err(error(
                CoverageFailureCode::ReaderNotObserved,
                format!("array leaf is outside {pointer}"),
            ));
        }
        if index == array.len() {
            array.push(value);
        } else {
            array[index] = value;
        }
    } else {
        let Value::Object(object) = current else {
            return Err(error(
                CoverageFailureCode::ReaderNotObserved,
                format!("non-object leaf parent while mutating {pointer}"),
            ));
        };
        object.insert(last.clone(), value);
    }
    Ok(())
}

#[derive(Debug, Clone)]
struct ExactHazardCollectionOccurrence {
    kind: SourceMemberKind,
    identity: String,
    authored_order: usize,
    cardinality: usize,
}

#[derive(Debug)]
struct ExactHazardLeafMatch {
    pointer: String,
    value: Value,
    collection: Option<ExactHazardCollectionOccurrence>,
}

fn collect_exact_hazard_leaves(
    value: &Value,
    segments: &[&str],
    pointer: String,
    collection: Option<ExactHazardCollectionOccurrence>,
    output: &mut Vec<ExactHazardLeafMatch>,
) {
    let Some((segment, remaining)) = segments.split_first() else {
        output.push(ExactHazardLeafMatch {
            pointer: if pointer.is_empty() {
                "/".to_string()
            } else {
                pointer
            },
            value: value.clone(),
            collection,
        });
        return;
    };
    let is_array = segment.ends_with("[]");
    let key = segment.strip_suffix("[]").unwrap_or(segment);
    if key == "*" {
        if let Some(object) = value.as_object() {
            let cardinality = object.len();
            for (authored_order, (member, child)) in object.iter().enumerate() {
                collect_exact_hazard_leaves(
                    child,
                    remaining,
                    format!("{pointer}/{}", escape_json_pointer(member)),
                    Some(ExactHazardCollectionOccurrence {
                        kind: SourceMemberKind::Map,
                        identity: member.clone(),
                        authored_order,
                        cardinality,
                    }),
                    output,
                );
            }
        }
        return;
    }
    let Some(child) = value.get(key) else {
        return;
    };
    let child_pointer = format!("{pointer}/{}", escape_json_pointer(key));
    if is_array {
        if let Some(array) = child.as_array() {
            let cardinality = array.len();
            for (index, member) in array.iter().enumerate() {
                collect_exact_hazard_leaves(
                    member,
                    remaining,
                    format!("{child_pointer}/{index}"),
                    Some(ExactHazardCollectionOccurrence {
                        kind: SourceMemberKind::Array,
                        identity: index.to_string(),
                        authored_order: index,
                        cardinality,
                    }),
                    output,
                );
            }
        }
    } else {
        collect_exact_hazard_leaves(child, remaining, child_pointer, collection, output);
    }
}

fn mutate_hazard_leaf_value(value: &Value) -> Value {
    match value {
        Value::Bool(value) => Value::Bool(!value),
        Value::Number(value) => value
            .as_i64()
            .map(|value| Value::from(value.saturating_add(1)))
            .unwrap_or_else(|| Value::String("source-leaf-number-mutation".to_string())),
        Value::String(value) => Value::String(format!("{value}\u{241f}source-leaf-mutation")),
        Value::Null => Value::String("source-leaf-null-mutation".to_string()),
        Value::Array(_) | Value::Object(_) => {
            Value::String("source-leaf-container-mutation".to_string())
        }
    }
}

fn source_json_type(value: &Value) -> SourceJsonType {
    match value {
        Value::Null => SourceJsonType::Null,
        Value::Bool(_) => SourceJsonType::Boolean,
        Value::Number(_) => SourceJsonType::Number,
        Value::String(_) => SourceJsonType::String,
        Value::Array(_) => SourceJsonType::Array,
        Value::Object(_) => SourceJsonType::Object,
    }
}

struct HazardExactLeafDestinations {
    source: String,
    canonical: String,
    post_projection: String,
    hydration: String,
}

fn hazard_exact_leaf_destinations(
    selector: &SourceLeafSelector,
    path: &str,
    disposition: HazardExactDisposition,
) -> HazardExactLeafDestinations {
    let embedded = selector.role == SourceDocumentRole::Embedded;
    let (source_owner, canonical_owner) =
        if (embedded && path == "$.img") || (!embedded && path == "$.items[].img") {
            ("HazardItemSource.image", "HazardEntity.image")
        } else if !embedded && path == "$.img" {
            ("HazardSource.image", "HazardRecord.provenance.image")
        } else if embedded && path.starts_with("$.system.rules[]") {
            (
                "HazardItemCommonSource.rules[]",
                "HazardRuleElement authored member",
            )
        } else if embedded && path.starts_with("$.system.damageRolls.*") {
            (
                "HazardDamageSource",
                "HazardStrikeCapability.damage_rolls[]",
            )
        } else if embedded && path.starts_with("$.system") {
            (
                "HazardItemSource typed system field",
                "HazardCapability typed or unsupported field",
            )
        } else if embedded {
            (
                "HazardItemSource identity/context field",
                "HazardEntity/HazardEntityOccurrence field",
            )
        } else if path.starts_with("$.items[].system.rules[]") {
            (
                "HazardItemCommonSource.rules[]",
                "HazardRuleElement authored member",
            )
        } else if path.starts_with("$.items[].system.damageRolls.*") {
            (
                "HazardDamageSource",
                "HazardStrikeCapability.damage_rolls[]",
            )
        } else if path.starts_with("$.items[].system") {
            (
                "HazardItemSource typed system field",
                "HazardCapability typed or unsupported field",
            )
        } else if path.starts_with("$.items[]") {
            (
                "HazardItemSource identity/context field",
                "HazardEntity/HazardEntityOccurrence field",
            )
        } else if path.starts_with("$.system.attributes") {
            (
                "HazardDetectionSource/HazardDefensesSource field",
                "HazardRecord detection/defenses field",
            )
        } else if path.starts_with("$.system.saves") {
            (
                "HazardSavesSource field",
                "HazardRecord.defenses.saves field",
            )
        } else if path.starts_with("$.system.details") {
            (
                "HazardSource details field",
                "HazardRecord lifecycle/classification/publication field",
            )
        } else if path.starts_with("$.system.traits") {
            (
                "HazardTraitsSource field",
                "HazardRecord traits/rarity/size field",
            )
        } else {
            (
                "HazardSource root field",
                "HazardRecord identity/provenance field",
            )
        };
    if let Some((canonical, post_projection, hydration)) =
        named_hazard_source_metadata_destinations(selector, path)
    {
        return HazardExactLeafDestinations {
            source: format!("{source_owner} ({path})"),
            canonical: canonical.to_string(),
            post_projection: post_projection.to_string(),
            hydration: hydration.to_string(),
        };
    }
    let canonical = if disposition == HazardExactDisposition::TypedUnsupported {
        format!("{canonical_owner}.unsupported_fields[{path}]")
    } else {
        format!("{canonical_owner} ({path})")
    };
    HazardExactLeafDestinations {
        source: format!("{source_owner} ({path})"),
        post_projection: format!("IndexBuildInput.canonical_bodies[].{canonical}"),
        hydration: format!("RetrievedRecord.body.RecordBody::Hazard.{canonical}"),
        canonical,
    }
}

fn named_hazard_source_metadata_destinations(
    selector: &SourceLeafSelector,
    path: &str,
) -> Option<(&'static str, &'static str, &'static str)> {
    let local_path = path
        .strip_prefix("$.items[]")
        .or_else(|| path.strip_prefix('$'))
        .unwrap_or(path);
    match path {
        "$.prototypeToken.name" => Some((
            "HazardRecord.provenance.token.name",
            "IndexBuildInput.canonical_bodies[].HazardRecord.provenance.token.name",
            "RetrievedRecord.body.RecordBody::Hazard.HazardRecord.provenance.token.name",
        )),
        "$.system.attributes.hasHealth" => Some((
            "HazardDefenses.source_metadata.has_health",
            "IndexBuildInput.canonical_bodies[].HazardRecord.defenses.source_metadata.has_health",
            "RetrievedRecord.body.RecordBody::Hazard.HazardRecord.defenses.source_metadata.has_health",
        )),
        "$.system.attributes.hp.tempmax" => Some((
            "HazardHitPoints.source_metadata.temporary_maximum",
            "IndexBuildInput.canonical_bodies[].defenses.hit_points.source_metadata.temporary_maximum",
            "RetrievedRecord.body.RecordBody::Hazard.HazardRecord.defenses.hit_points.source_metadata.temporary_maximum",
        )),
        "$.system.saves.*.saveDetail" => Some((
            "HazardSaves.source_metadata save-specific detail fact",
            "IndexBuildInput.canonical_bodies[].HazardRecord.defenses.saves.source_metadata",
            "RetrievedRecord.body.RecordBody::Hazard.HazardRecord.defenses.saves.source_metadata",
        )),
        _ if selector.role == SourceDocumentRole::Embedded || path.starts_with("$.items[]") => {
            match local_path {
                "._stats.compendiumSource" => Some((
                    "HazardItemCommon.lineage.compendium_source",
                    "IndexBuildInput.canonical_bodies[].HazardItemCommon.lineage.compendium_source",
                    "RetrievedRecord.body.RecordBody::Hazard.HazardItemCommon.lineage.compendium_source",
                )),
                ".system.traits.rarity" => Some((
                    "HazardItemCommon.rarity",
                    "IndexBuildInput.canonical_bodies[].HazardItemCommon.rarity",
                    "RetrievedRecord.body.RecordBody::Hazard.HazardItemCommon.rarity",
                )),
                ".system.attack.value" => Some((
                    "HazardStrikeCapability.source_metadata.attack",
                    "IndexBuildInput.canonical_bodies[].HazardStrikeCapability.source_metadata.attack",
                    "RetrievedRecord.body.RecordBody::Hazard.HazardStrikeCapability.source_metadata.attack",
                )),
                ".system.weaponType.value" => Some((
                    "HazardStrikeCapability.source_metadata.weapon_type",
                    "IndexBuildInput.canonical_bodies[].HazardStrikeCapability.source_metadata.weapon_type",
                    "RetrievedRecord.body.RecordBody::Hazard.HazardStrikeCapability.source_metadata.weapon_type",
                )),
                ".system.attackEffects.custom" => Some((
                    "HazardStrikeCapability.source_metadata.attack_effects_custom",
                    "IndexBuildInput.canonical_bodies[].HazardStrikeCapability.source_metadata.attack_effects_custom",
                    "RetrievedRecord.body.RecordBody::Hazard.HazardStrikeCapability.source_metadata.attack_effects_custom",
                )),
                _ => None,
            }
        }
        _ => None,
    }
}

#[derive(Debug)]
struct HazardExactOwnerFingerprints {
    source: String,
    canonical: String,
    post_projection: String,
}

#[derive(Debug)]
struct HazardExactOwnerValues {
    source: SourcePresence<SourceLeafValue>,
    canonical: SourcePresence<SourceLeafValue>,
    post_projection: SourcePresence<SourceLeafValue>,
}

struct HazardHydrationOwnerEvidence {
    expected_source: SourcePresence<SourceLeafValue>,
    owner_value: SourcePresence<SourceLeafValue>,
    owner_fingerprint: String,
}

fn exact_hazard_hydration_owner_evidence(
    pipeline: &HazardPipeline,
    fixture: &ResolvedFixture,
    identity: &SourceLeafIdentity,
    disposition: HazardExactDisposition,
) -> Result<HazardHydrationOwnerEvidence, CoverageContractError> {
    let occurrence = exact_hazard_hydration_occurrence(pipeline, fixture, identity)?;
    Ok(HazardHydrationOwnerEvidence {
        expected_source: exact_hazard_source_value(
            &pipeline.hydration_source_dto,
            identity,
            &occurrence,
            disposition,
        )?,
        owner_value: exact_hazard_canonical_value(
            &pipeline.hydration,
            identity,
            &occurrence,
            disposition,
        )?,
        owner_fingerprint: exact_hazard_canonical_fingerprint(
            &pipeline.hydration,
            identity,
            &occurrence,
            disposition,
        )?,
    })
}

fn exact_hazard_hydration_occurrence(
    pipeline: &HazardPipeline,
    fixture: &ResolvedFixture,
    identity: &SourceLeafIdentity,
) -> Result<ExactHazardLeafOccurrence, CoverageContractError> {
    let mut reference = fixture.reference.clone();
    if identity.selector.role == SourceDocumentRole::Embedded {
        // Resolve the authored item position from the authenticated baseline
        // before reading the grouped mutation. The compatible group may mutate
        // that item's source ID, so selecting by the baseline ID afterward
        // would inspect no owner at all.
        let baseline = first_exact_hazard_leaf(fixture, identity)?;
        let item_ordinal = pointer_index_after(&baseline.pointer, "items")?;
        let item_id = pipeline
            .hydration_raw
            .pointer(&format!("/items/{item_ordinal}/_id"))
            .and_then(Value::as_str)
            .ok_or_else(|| {
                error(
                    CoverageFailureCode::ArtifactHydrationMismatch,
                    format!(
                        "grouped hazard hydration omitted item identity at authored ordinal {item_ordinal}"
                    ),
                )
            })?;
        let parent = reference
            .record_key
            .rsplit_once("#item:")
            .map(|(parent, _)| parent)
            .ok_or_else(|| {
                error(
                    CoverageFailureCode::FixtureNotSourceGrounded,
                    "embedded exact-leaf fixture has no item identity",
                )
            })?;
        reference.record_key = format!("{parent}#item:{item_id}");
    }
    first_exact_hazard_leaf(
        &ResolvedFixture {
            reference,
            serialized: serde_json::to_vec(&pipeline.hydration_raw).map_err(|message| {
                error(CoverageFailureCode::InvalidContract, message.to_string())
            })?,
            raw: pipeline.hydration_raw.clone(),
        },
        identity,
    )
}

fn exact_hazard_owner_values(
    pipeline: &HazardPipeline,
    identity: &SourceLeafIdentity,
    occurrence: &ExactHazardLeafOccurrence,
    disposition: HazardExactDisposition,
) -> Result<HazardExactOwnerValues, CoverageContractError> {
    Ok(HazardExactOwnerValues {
        source: exact_hazard_source_value(&pipeline.source_dto, identity, occurrence, disposition)?,
        canonical: exact_hazard_canonical_value(
            &pipeline.canonical,
            identity,
            occurrence,
            disposition,
        )?,
        post_projection: exact_hazard_canonical_value(
            &pipeline.post_projection,
            identity,
            occurrence,
            disposition,
        )?,
    })
}

fn owner_leaf_value(
    occurrence: &ExactHazardLeafOccurrence,
    disposition: HazardExactDisposition,
    value: Value,
) -> SourcePresence<SourceLeafValue> {
    occurrence.receipt_value_for(value, disposition)
}

fn dto_field_owner_value<T>(
    field: &HazardSourceField<T>,
    occurrence: &ExactHazardLeafOccurrence,
    disposition: HazardExactDisposition,
    encode: impl FnOnce(&T) -> Result<Value, CoverageContractError>,
) -> Result<SourcePresence<SourceLeafValue>, CoverageContractError> {
    match field {
        SourcePresence::Missing => Ok(SourcePresence::Missing),
        SourcePresence::Null => Ok(if disposition == HazardExactDisposition::TypedUnsupported {
            owner_leaf_value(occurrence, disposition, Value::Null)
        } else {
            SourcePresence::Null
        }),
        SourcePresence::Value(DtoHazardSourceValue::Typed(value)) => {
            Ok(owner_leaf_value(occurrence, disposition, encode(value)?))
        }
        SourcePresence::Value(DtoHazardSourceValue::Unsupported(summary)) => Ok(owner_leaf_value(
            occurrence,
            disposition,
            serde_json::from_str(&summary.value).map_err(|message| {
                error(
                    CoverageFailureCode::DtoMismatch,
                    format!(
                        "invalid exact source DTO JSON at {}: {message}",
                        summary.source_path
                    ),
                )
            })?,
        )),
    }
}

fn json_owner_value(value: &impl Serialize) -> Result<Value, CoverageContractError> {
    serde_json::to_value(value).map_err(|message| {
        error(
            CoverageFailureCode::ReaderNotObserved,
            format!("failed to encode exact typed owner value: {message}"),
        )
    })
}

fn array_member_owner_value<T: Serialize>(
    values: &[T],
    occurrence: &ExactHazardLeafOccurrence,
    mismatch_code: CoverageFailureCode,
    owner: &str,
) -> Result<Value, CoverageContractError> {
    let index = last_pointer_index(&occurrence.pointer)?;
    verify_owner_collection(
        occurrence,
        SourceMemberKind::Array,
        None,
        index,
        values.len(),
        mismatch_code,
        owner,
    )?;
    values
        .get(index)
        .map(json_owner_value)
        .transpose()?
        .ok_or_else(|| {
            error(
                CoverageFailureCode::ReaderNotObserved,
                format!(
                    "exact typed owner omitted array member {index} for {}",
                    occurrence.pointer
                ),
            )
        })
}

fn dto_array_member_owner_value<T: Serialize>(
    field: &HazardSourceField<Vec<T>>,
    occurrence: &ExactHazardLeafOccurrence,
    disposition: HazardExactDisposition,
) -> Result<SourcePresence<SourceLeafValue>, CoverageContractError> {
    match field {
        SourcePresence::Missing => Ok(SourcePresence::Missing),
        SourcePresence::Null => Ok(SourcePresence::Null),
        SourcePresence::Value(DtoHazardSourceValue::Typed(values)) => {
            if occurrence.value.is_none() && values.is_empty() {
                return Ok(SourcePresence::Missing);
            }
            let index = last_pointer_index(&occurrence.pointer)?;
            verify_owner_collection(
                occurrence,
                SourceMemberKind::Array,
                None,
                index,
                values.len(),
                CoverageFailureCode::DtoMismatch,
                "exact source DTO array",
            )?;
            match values.get(index) {
                Some(value) => Ok(owner_leaf_value(
                    occurrence,
                    disposition,
                    json_owner_value(value)?,
                )),
                None => Ok(SourcePresence::Missing),
            }
        }
        SourcePresence::Value(DtoHazardSourceValue::Unsupported(summary)) => {
            Ok(owner_leaf_value(occurrence, disposition, {
                let exact: Value = serde_json::from_str(&summary.value).map_err(|message| {
                    error(
                        CoverageFailureCode::DtoMismatch,
                        format!("invalid exact source DTO array JSON: {message}"),
                    )
                })?;
                let values = exact.as_array().ok_or_else(|| {
                    error(
                        CoverageFailureCode::DtoMismatch,
                        "exact unsupported source DTO array owner is not an array",
                    )
                })?;
                let index = last_pointer_index(&occurrence.pointer)?;
                verify_owner_collection(
                    occurrence,
                    SourceMemberKind::Array,
                    None,
                    index,
                    values.len(),
                    CoverageFailureCode::DtoMismatch,
                    "exact unsupported source DTO array",
                )?;
                values.get(index).cloned().ok_or_else(|| {
                    error(
                        CoverageFailureCode::DtoMismatch,
                        format!("unsupported source DTO array omitted member {index}"),
                    )
                })?
            }))
        }
    }
}

fn verify_owner_collection(
    occurrence: &ExactHazardLeafOccurrence,
    kind: SourceMemberKind,
    identity: Option<&str>,
    authored_order: usize,
    cardinality: usize,
    mismatch_code: CoverageFailureCode,
    owner: &str,
) -> Result<(), CoverageContractError> {
    let identity_matches = match kind {
        SourceMemberKind::Array => true,
        SourceMemberKind::Map | SourceMemberKind::Nested => {
            occurrence.member_identity.as_deref() == identity
        }
    };
    if occurrence.member_kind != Some(kind)
        || occurrence.collection_authored_order != Some(authored_order)
        || occurrence.collection_cardinality != Some(cardinality)
        || !identity_matches
    {
        return Err(error(
            mismatch_code,
            format!(
                "{owner} collection meaning does not equal authenticated source collection: source_kind={:?}, source_identity={:?}, source_order={:?}, source_cardinality={:?}; owner_kind={kind:?}, owner_identity={identity:?}, owner_order={authored_order}, owner_cardinality={cardinality}",
                occurrence.member_kind,
                occurrence.member_identity,
                occurrence.collection_authored_order,
                occurrence.collection_cardinality,
            ),
        ));
    }
    Ok(())
}

fn validate_spell_excerpt(
    fixture: &ResolvedFixture,
    source: &SourcePresence<SourceLeafValue>,
) -> Result<(), CoverageContractError> {
    let actual = digest_serializable(source)?;
    if actual != fixture.reference.excerpt_digest {
        return Err(error(
            CoverageFailureCode::ReceiptProvenanceInvalid,
            format!(
                "registered spell excerpt digest mismatch: expected {}, found {actual}",
                fixture.reference.excerpt_digest
            ),
        ));
    }
    Ok(())
}

fn exact_summary_value(
    summaries: &[crate::source::dto::ValueSummary],
    pointer: &str,
) -> Result<Value, CoverageContractError> {
    let summary = summaries
        .iter()
        .find(|summary| summary.source_path == pointer)
        .ok_or_else(|| {
            error(
                CoverageFailureCode::DtoMismatch,
                format!("typed source summary omitted exact source path {pointer}"),
            )
        })?;
    serde_json::from_str(&summary.value).map_err(|message| {
        error(
            CoverageFailureCode::DtoMismatch,
            format!("invalid exact typed source summary JSON at {pointer}: {message}"),
        )
    })
}

fn exact_hazard_owner_fingerprints(
    pipeline: &HazardPipeline,
    identity: &SourceLeafIdentity,
    occurrence: &ExactHazardLeafOccurrence,
    disposition: HazardExactDisposition,
) -> Result<HazardExactOwnerFingerprints, CoverageContractError> {
    Ok(HazardExactOwnerFingerprints {
        source: exact_hazard_source_fingerprint(
            &pipeline.source_dto,
            identity,
            occurrence,
            disposition,
        )?,
        canonical: exact_hazard_canonical_fingerprint(
            &pipeline.canonical,
            identity,
            occurrence,
            disposition,
        )?,
        post_projection: exact_hazard_canonical_fingerprint(
            &pipeline.post_projection,
            identity,
            occurrence,
            disposition,
        )?,
    })
}

fn owner_fingerprint(label: &str, value: &impl std::fmt::Debug) -> String {
    format!("{label}={value:?}")
}

fn hazard_semantic_fingerprint(hazard: &HazardRecord) -> String {
    owner_fingerprint(
        "HazardRecord typed presentation/mechanics",
        &(
            atlas_record::project_hazard_facts(hazard),
            atlas_record::build_hazard_presentation_document(hazard, |_| true),
        ),
    )
}

fn dto_typed<'a, T>(
    value: &'a HazardSourceField<T>,
    owner: &str,
) -> Result<&'a T, CoverageContractError> {
    match value {
        SourcePresence::Value(DtoHazardSourceValue::Typed(value)) => Ok(value),
        _ => Err(error(
            CoverageFailureCode::DtoMismatch,
            format!("{owner} is not a typed source owner"),
        )),
    }
}

fn exact_hazard_source_value(
    versioned: &VersionedHazardSource,
    identity: &SourceLeafIdentity,
    occurrence: &ExactHazardLeafOccurrence,
    disposition: HazardExactDisposition,
) -> Result<SourcePresence<SourceLeafValue>, CoverageContractError> {
    let source = &versioned.source;
    let path = identity.normalized_path.as_str();
    if identity.selector.role == SourceDocumentRole::Embedded {
        return exact_hazard_item_source_value(
            source,
            path,
            occurrence,
            disposition,
            &identity.selector,
        );
    }
    match path {
        "$._id" => Ok(owner_leaf_value(
            occurrence,
            disposition,
            Value::String(source.id.clone()),
        )),
        "$.name" => Ok(owner_leaf_value(
            occurrence,
            disposition,
            Value::String(source.name.clone()),
        )),
        "$.type" => Ok(owner_leaf_value(
            occurrence,
            disposition,
            Value::String(source.actor_type.clone()),
        )),
        "$.img" => dto_field_owner_value(&source.image, occurrence, disposition, json_owner_value),
        "$.folder" => {
            dto_field_owner_value(&source.folder, occurrence, disposition, json_owner_value)
        }
        "$.effects" => dto_field_owner_value(&source.effects, occurrence, disposition, |values| {
            values
                .iter()
                .map(|value| serde_json::from_str(&value.value))
                .collect::<Result<Vec<Value>, _>>()
                .map(Value::Array)
                .map_err(|message| {
                    error(
                        CoverageFailureCode::DtoMismatch,
                        format!("invalid HazardSource.effects exact JSON: {message}"),
                    )
                })
        }),
        "$.system.creatureType" => dto_field_owner_value(
            &source.creature_type,
            occurrence,
            disposition,
            json_owner_value,
        ),
        "$.system.statusEffects[]" => {
            dto_array_member_owner_value(&source.status_effects, occurrence, disposition)
        }
        "$.system.details.level.value" => {
            dto_field_owner_value(&source.level, occurrence, disposition, json_owner_value)
        }
        "$.system.details.isComplex" => dto_field_owner_value(
            &source.complexity,
            occurrence,
            disposition,
            json_owner_value,
        ),
        "$.system.attributes.emitsSound" => dto_field_owner_value(
            &source.emits_sound,
            occurrence,
            disposition,
            |value| match value {
                crate::source::dto::HazardEmitsSoundSource::Boolean(value) => {
                    Ok(Value::Bool(*value))
                }
                crate::source::dto::HazardEmitsSoundSource::Named(value) => {
                    Ok(Value::String(value.clone()))
                }
            },
        ),
        "$.prototypeToken.name" => {
            let token = dto_typed(&source.prototype_token, "HazardSource.prototype_token")?;
            dto_field_owner_value(&token.name, occurrence, disposition, json_owner_value)
        }
        "$._stats.compendiumSource" => Ok(owner_leaf_value(
            occurrence,
            disposition,
            exact_summary_value(&source.unclaimed, &occurrence.pointer)?,
        )),
        _ if path.starts_with("$.system.traits.") => {
            let traits = dto_typed(&source.traits, "HazardSource.traits")?;
            match path {
                "$.system.traits.value[]" => {
                    dto_array_member_owner_value(&traits.values, occurrence, disposition)
                }
                "$.system.traits.rarity" => {
                    dto_field_owner_value(&traits.rarity, occurrence, disposition, json_owner_value)
                }
                "$.system.traits.size.value" => {
                    dto_field_owner_value(&traits.size, occurrence, disposition, json_owner_value)
                }
                _ => unknown_hazard_accessor("source DTO value", path),
            }
        }
        _ if path.starts_with("$.system.details.publication.") => {
            let publication = dto_typed(&source.publication, "HazardSource.publication")?;
            match path {
                "$.system.details.publication.title" => dto_field_owner_value(
                    &publication.title,
                    occurrence,
                    disposition,
                    json_owner_value,
                ),
                "$.system.details.publication.remaster" => dto_field_owner_value(
                    &publication.remaster,
                    occurrence,
                    disposition,
                    json_owner_value,
                ),
                "$.system.details.publication.license" => dto_field_owner_value(
                    &publication.license,
                    occurrence,
                    disposition,
                    json_owner_value,
                ),
                _ => unknown_hazard_accessor("source DTO value", path),
            }
        }
        _ if path.starts_with("$.system.details.") => {
            let lifecycle = dto_typed(&source.lifecycle, "HazardSource.lifecycle")?;
            let field = match path {
                "$.system.details.description" => &lifecycle.description,
                "$.system.details.disable" => &lifecycle.disable,
                "$.system.details.routine" => &lifecycle.routine,
                "$.system.details.reset" => &lifecycle.reset,
                _ => return unknown_hazard_accessor("source DTO value", path),
            };
            dto_field_owner_value(field, occurrence, disposition, json_owner_value)
        }
        _ if path.starts_with("$.system.attributes.stealth.") => {
            let detection = dto_typed(&source.detection, "HazardSource.detection")?;
            match path {
                "$.system.attributes.stealth.value" => dto_field_owner_value(
                    &detection.value,
                    occurrence,
                    disposition,
                    json_owner_value,
                ),
                "$.system.attributes.stealth.details" => dto_field_owner_value(
                    &detection.details,
                    occurrence,
                    disposition,
                    json_owner_value,
                ),
                _ => unknown_hazard_accessor("source DTO value", path),
            }
        }
        _ if path.starts_with("$.system.attributes.") || path.starts_with("$.system.saves.") => {
            exact_hazard_defense_source_value(source, path, occurrence, disposition)
        }
        _ if path.starts_with("$.items[]") => exact_hazard_item_source_value(
            source,
            path,
            occurrence,
            disposition,
            &identity.selector,
        ),
        _ => unknown_hazard_accessor("source DTO value", path),
    }
}

fn exact_hazard_defense_source_value(
    source: &crate::source::dto::HazardSource,
    path: &str,
    occurrence: &ExactHazardLeafOccurrence,
    disposition: HazardExactDisposition,
) -> Result<SourcePresence<SourceLeafValue>, CoverageContractError> {
    let defenses = dto_typed(&source.defenses, "HazardSource.defenses")?;
    match path {
        "$.system.attributes.ac.value" => dto_field_owner_value(
            &defenses.armor_class,
            occurrence,
            disposition,
            json_owner_value,
        ),
        "$.system.attributes.hardness" => dto_field_owner_value(
            &defenses.hardness,
            occurrence,
            disposition,
            json_owner_value,
        ),
        "$.system.attributes.hasHealth" => dto_field_owner_value(
            &defenses.has_health,
            occurrence,
            disposition,
            json_owner_value,
        ),
        _ if path.starts_with("$.system.attributes.hp.") => {
            let hit_points = dto_typed(&defenses.hit_points, "HazardDefensesSource.hit_points")?;
            match path {
                "$.system.attributes.hp.value" => dto_field_owner_value(
                    &hit_points.current,
                    occurrence,
                    disposition,
                    json_owner_value,
                ),
                "$.system.attributes.hp.max" => dto_field_owner_value(
                    &hit_points.maximum,
                    occurrence,
                    disposition,
                    json_owner_value,
                ),
                "$.system.attributes.hp.temp" => dto_field_owner_value(
                    &hit_points.temporary,
                    occurrence,
                    disposition,
                    json_owner_value,
                ),
                "$.system.attributes.hp.details" => dto_field_owner_value(
                    &hit_points.details,
                    occurrence,
                    disposition,
                    json_owner_value,
                ),
                "$.system.attributes.hp.tempmax" => dto_field_owner_value(
                    &hit_points.temporary_maximum,
                    occurrence,
                    disposition,
                    json_owner_value,
                ),
                _ => unknown_hazard_accessor("source DTO value", path),
            }
        }
        _ if path.starts_with("$.system.saves.") => {
            let saves = dto_typed(&defenses.saves, "HazardDefensesSource.saves")?;
            let kind = pointer_member_after(&occurrence.pointer, "saves")?;
            let present = [
                ("fortitude", &saves.fortitude),
                ("reflex", &saves.reflex),
                ("will", &saves.will),
            ]
            .into_iter()
            .filter(|(_, save)| {
                !matches!(save.value, SourcePresence::Missing)
                    || !matches!(save.detail, SourcePresence::Missing)
            })
            .collect::<Vec<_>>();
            let (authored_order, save) = present
                .iter()
                .enumerate()
                .find_map(|(order, (owner_kind, save))| {
                    (*owner_kind == kind).then_some((order, *save))
                })
                .ok_or_else(|| {
                    error(
                        CoverageFailureCode::DtoMismatch,
                        format!("HazardSavesSource omitted authored key {kind}"),
                    )
                })?;
            verify_owner_collection(
                occurrence,
                SourceMemberKind::Map,
                Some(&kind),
                authored_order,
                present.len(),
                CoverageFailureCode::DtoMismatch,
                "HazardSavesSource",
            )?;
            if path.ends_with(".value") {
                dto_field_owner_value(&save.value, occurrence, disposition, json_owner_value)
            } else if path.ends_with(".saveDetail") {
                dto_field_owner_value(&save.detail, occurrence, disposition, json_owner_value)
            } else {
                unknown_hazard_accessor("source DTO save value", path)
            }
        }
        _ if path.starts_with("$.system.attributes.immunities[]")
            || path.starts_with("$.system.attributes.weaknesses[]")
            || path.starts_with("$.system.attributes.resistances[]") =>
        {
            let (family, entries) = if path.contains(".immunities[]") {
                ("immunities", &defenses.immunities)
            } else if path.contains(".weaknesses[]") {
                ("weaknesses", &defenses.weaknesses)
            } else {
                ("resistances", &defenses.resistances)
            };
            let entries = dto_typed(entries, "HazardDefensesSource IWR entries")?;
            let index = pointer_index_after(&occurrence.pointer, family)?;
            let entry = entries.get(index).ok_or_else(|| {
                error(
                    CoverageFailureCode::DtoMismatch,
                    format!("missing {family}[{index}] source owner"),
                )
            })?;
            if entry.source_ordinal != index as u32 {
                return Err(error(
                    CoverageFailureCode::DtoMismatch,
                    format!("{family}[{index}] source owner has wrong authored order"),
                ));
            }
            if !path.ends_with("[]") {
                verify_owner_collection(
                    occurrence,
                    SourceMemberKind::Array,
                    None,
                    entry.source_ordinal as usize,
                    entries.len(),
                    CoverageFailureCode::DtoMismatch,
                    "HazardDefensesSource IWR entries",
                )?;
            }
            if path.ends_with(".type") {
                dto_field_owner_value(&entry.iwr_type, occurrence, disposition, json_owner_value)
            } else if path.ends_with(".value") {
                dto_field_owner_value(&entry.value, occurrence, disposition, json_owner_value)
            } else if path.ends_with(".exceptions[]") {
                dto_array_member_owner_value(&entry.exceptions, occurrence, disposition)
            } else if path.ends_with(".doubleVs[]") {
                dto_array_member_owner_value(&entry.double_vs, occurrence, disposition)
            } else {
                unknown_hazard_accessor("source DTO value", path)
            }
        }
        _ => unknown_hazard_accessor("source DTO value", path),
    }
}

fn exact_hazard_source_fingerprint(
    versioned: &VersionedHazardSource,
    identity: &SourceLeafIdentity,
    occurrence: &ExactHazardLeafOccurrence,
    disposition: HazardExactDisposition,
) -> Result<String, CoverageContractError> {
    let source = &versioned.source;
    let path = identity.normalized_path.as_str();
    if identity.selector.role == SourceDocumentRole::Embedded {
        return exact_hazard_item_source_fingerprint(
            source,
            path,
            occurrence,
            disposition,
            &identity.selector,
        );
    }
    let fingerprint = match path {
        "$._id" => owner_fingerprint("HazardSource.id", &source.id),
        "$.name" => owner_fingerprint("HazardSource.name", &source.name),
        "$.type" => owner_fingerprint("HazardSource.actor_type", &source.actor_type),
        "$.img" => owner_fingerprint("HazardSource.image", &source.image),
        "$.folder" => owner_fingerprint("HazardSource.folder", &source.folder),
        "$.effects" => owner_fingerprint("HazardSource.effects", &source.effects),
        "$.system.creatureType" => {
            owner_fingerprint("HazardSource.creature_type", &source.creature_type)
        }
        "$.system.statusEffects[]" => {
            owner_fingerprint("HazardSource.status_effects", &source.status_effects)
        }
        "$.system.details.level.value" => owner_fingerprint("HazardSource.level", &source.level),
        "$.system.details.isComplex" => {
            owner_fingerprint("HazardSource.complexity", &source.complexity)
        }
        "$.system.attributes.emitsSound" => {
            owner_fingerprint("HazardSource.emits_sound", &source.emits_sound)
        }
        "$.prototypeToken.name" => {
            let token = dto_typed(&source.prototype_token, "HazardSource.prototype_token")?;
            owner_fingerprint("HazardTokenSourceMetadata.name", &token.name)
        }
        "$._stats.compendiumSource" => exact_summary_fingerprint(
            &source.unclaimed,
            &occurrence.pointer,
            "HazardSource.unclaimed",
        )?,
        _ if path.starts_with("$.system.traits.") => {
            let traits = dto_typed(&source.traits, "HazardSource.traits")?;
            match path {
                "$.system.traits.value[]" => {
                    owner_fingerprint("HazardTraitsSource.values", &traits.values)
                }
                "$.system.traits.rarity" => {
                    owner_fingerprint("HazardTraitsSource.rarity", &traits.rarity)
                }
                "$.system.traits.size.value" => {
                    owner_fingerprint("HazardTraitsSource.size", &traits.size)
                }
                _ => return unknown_hazard_accessor("source DTO", path),
            }
        }
        _ if path.starts_with("$.system.details.publication.") => {
            let publication = dto_typed(&source.publication, "HazardSource.publication")?;
            match path {
                "$.system.details.publication.title" => {
                    owner_fingerprint("HazardPublicationSource.title", &publication.title)
                }
                "$.system.details.publication.remaster" => {
                    owner_fingerprint("HazardPublicationSource.remaster", &publication.remaster)
                }
                "$.system.details.publication.license" => {
                    owner_fingerprint("HazardPublicationSource.license", &publication.license)
                }
                _ => return unknown_hazard_accessor("source DTO", path),
            }
        }
        _ if path.starts_with("$.system.details.") => {
            let lifecycle = dto_typed(&source.lifecycle, "HazardSource.lifecycle")?;
            match path {
                "$.system.details.description" => {
                    owner_fingerprint("HazardLifecycleSource.description", &lifecycle.description)
                }
                "$.system.details.disable" => {
                    owner_fingerprint("HazardLifecycleSource.disable", &lifecycle.disable)
                }
                "$.system.details.routine" => {
                    owner_fingerprint("HazardLifecycleSource.routine", &lifecycle.routine)
                }
                "$.system.details.reset" => {
                    owner_fingerprint("HazardLifecycleSource.reset", &lifecycle.reset)
                }
                _ => return unknown_hazard_accessor("source DTO", path),
            }
        }
        _ if path.starts_with("$.system.attributes.stealth.") => {
            let detection = dto_typed(&source.detection, "HazardSource.detection")?;
            match path {
                "$.system.attributes.stealth.value" => {
                    owner_fingerprint("HazardDetectionSource.value", &detection.value)
                }
                "$.system.attributes.stealth.details" => {
                    owner_fingerprint("HazardDetectionSource.details", &detection.details)
                }
                _ => return unknown_hazard_accessor("source DTO", path),
            }
        }
        _ if path.starts_with("$.system.attributes.") || path.starts_with("$.system.saves.") => {
            exact_hazard_defense_source_fingerprint(source, path, occurrence)?
        }
        _ if path.starts_with("$.items[]")
            || identity.selector.role == SourceDocumentRole::Embedded =>
        {
            exact_hazard_item_source_fingerprint(
                source,
                path,
                occurrence,
                disposition,
                &identity.selector,
            )?
        }
        _ => return unknown_hazard_accessor("source DTO", path),
    };
    Ok(fingerprint)
}

fn exact_hazard_defense_source_fingerprint(
    source: &crate::source::dto::HazardSource,
    path: &str,
    occurrence: &ExactHazardLeafOccurrence,
) -> Result<String, CoverageContractError> {
    let defenses = dto_typed(&source.defenses, "HazardSource.defenses")?;
    let fingerprint = match path {
        "$.system.attributes.ac.value" => {
            owner_fingerprint("HazardDefensesSource.armor_class", &defenses.armor_class)
        }
        "$.system.attributes.hardness" => {
            owner_fingerprint("HazardDefensesSource.hardness", &defenses.hardness)
        }
        "$.system.attributes.hasHealth" => {
            owner_fingerprint("HazardDefensesSource.has_health", &defenses.has_health)
        }
        _ if path.starts_with("$.system.attributes.hp.") => {
            let hit_points = dto_typed(&defenses.hit_points, "HazardDefensesSource.hit_points")?;
            match path {
                "$.system.attributes.hp.value" => {
                    owner_fingerprint("HazardHitPointsSource.current", &hit_points.current)
                }
                "$.system.attributes.hp.max" => {
                    owner_fingerprint("HazardHitPointsSource.maximum", &hit_points.maximum)
                }
                "$.system.attributes.hp.temp" => {
                    owner_fingerprint("HazardHitPointsSource.temporary", &hit_points.temporary)
                }
                "$.system.attributes.hp.details" => {
                    owner_fingerprint("HazardHitPointsSource.details", &hit_points.details)
                }
                "$.system.attributes.hp.tempmax" => owner_fingerprint(
                    "HazardHitPointsSource.temporary_maximum",
                    &hit_points.temporary_maximum,
                ),
                _ => return unknown_hazard_accessor("source DTO", path),
            }
        }
        "$.system.saves.*.value" | "$.system.saves.*.saveDetail" => {
            let saves = dto_typed(&defenses.saves, "HazardDefensesSource.saves")?;
            let kind = pointer_member_after(&occurrence.pointer, "saves")?;
            let save = match kind.as_str() {
                "fortitude" => &saves.fortitude,
                "reflex" => &saves.reflex,
                "will" => &saves.will,
                _ => return unknown_hazard_accessor("source DTO save", &kind),
            };
            if path.ends_with(".value") {
                owner_fingerprint("HazardSaveSource.value", &save.value)
            } else {
                owner_fingerprint("HazardSaveSource.detail", &save.detail)
            }
        }
        _ if path.starts_with("$.system.attributes.immunities[]")
            || path.starts_with("$.system.attributes.weaknesses[]")
            || path.starts_with("$.system.attributes.resistances[]") =>
        {
            let (family, entries) = if path.contains(".immunities[]") {
                ("immunities", &defenses.immunities)
            } else if path.contains(".weaknesses[]") {
                ("weaknesses", &defenses.weaknesses)
            } else {
                ("resistances", &defenses.resistances)
            };
            let entries = dto_typed(entries, "HazardDefensesSource IWR entries")?;
            let index = pointer_index_after(&occurrence.pointer, family)?;
            let entry = entries.get(index).ok_or_else(|| {
                error(
                    CoverageFailureCode::DtoMismatch,
                    format!("missing {family}[{index}] source owner"),
                )
            })?;
            if path.ends_with(".type") {
                owner_fingerprint("HazardIwrSource.iwr_type", &entry.iwr_type)
            } else if path.ends_with(".value") {
                owner_fingerprint("HazardIwrSource.value", &entry.value)
            } else if path.ends_with(".exceptions[]") {
                owner_fingerprint("HazardIwrSource.exceptions", &entry.exceptions)
            } else if path.ends_with(".doubleVs[]") {
                owner_fingerprint("HazardIwrSource.double_vs", &entry.double_vs)
            } else {
                return unknown_hazard_accessor("source DTO", path);
            }
        }
        _ => return unknown_hazard_accessor("source DTO", path),
    };
    Ok(fingerprint)
}

fn exact_hazard_item_source_fingerprint(
    source: &crate::source::dto::HazardSource,
    path: &str,
    occurrence: &ExactHazardLeafOccurrence,
    disposition: HazardExactDisposition,
    selector: &SourceLeafSelector,
) -> Result<String, CoverageContractError> {
    let items = dto_typed(&source.items, "HazardSource.items")?;
    let index = pointer_index_after(&occurrence.pointer, "items")?;
    let item = items.get(index).ok_or_else(|| {
        error(
            CoverageFailureCode::DtoMismatch,
            format!("missing HazardItemSource at authored ordinal {index}"),
        )
    })?;
    let local_path = path
        .strip_prefix("$.items[]")
        .or_else(|| path.strip_prefix('$'))
        .unwrap_or(path);
    if disposition == HazardExactDisposition::TypedUnsupported
        && selector.type_discriminator == "consumable"
    {
        return exact_summary_fingerprint(
            &item.unclaimed,
            &occurrence.pointer,
            "HazardItemSource.unclaimed",
        );
    }
    let fingerprint = match local_path {
        "._id" => owner_fingerprint("HazardItemSource.id", &item.id),
        ".name" => owner_fingerprint("HazardItemSource.name", &item.name),
        ".type" => owner_fingerprint("HazardItemSource.item_type", &item.item_type),
        ".img" => owner_fingerprint("HazardItemSource.image", &item.image),
        ".folder" => owner_fingerprint("HazardItemSource.folder", &item.folder),
        ".sort" => owner_fingerprint("HazardItemSource.sort", &item.sort),
        ".system.description.value" => owner_fingerprint(
            "HazardItemCommonSource.description",
            &item.common.description,
        ),
        ".system.publication.title" => {
            let publication = dto_typed(
                &item.common.publication,
                "HazardItemCommonSource.publication",
            )?;
            owner_fingerprint("HazardPublicationSource.title", &publication.title)
        }
        ".system.publication.remaster" => {
            let publication = dto_typed(
                &item.common.publication,
                "HazardItemCommonSource.publication",
            )?;
            owner_fingerprint("HazardPublicationSource.remaster", &publication.remaster)
        }
        ".system.publication.license" => {
            let publication = dto_typed(
                &item.common.publication,
                "HazardItemCommonSource.publication",
            )?;
            owner_fingerprint("HazardPublicationSource.license", &publication.license)
        }
        ".system.slug" => owner_fingerprint("HazardItemCommonSource.slug", &item.common.slug),
        ".system.traits.value[]" => {
            owner_fingerprint("HazardItemCommonSource.traits", &item.common.traits)
        }
        ".system.traits.rarity" => {
            owner_fingerprint("HazardItemCommonSource.rarity", &item.common.rarity)
        }
        "._stats.compendiumSource" => {
            owner_fingerprint("HazardItemCommonSource.lineage", &item.common.lineage)
        }
        ".system.actionType.value" => {
            owner_fingerprint("HazardActionSource.action_type", &item.action.action_type)
        }
        ".system.actions.value" => {
            owner_fingerprint("HazardActionSource.actions", &item.action.actions)
        }
        ".system.category" => {
            owner_fingerprint("HazardActionSource.category", &item.action.category)
        }
        ".system.deathNote" => {
            owner_fingerprint("HazardActionSource.death_note", &item.action.death_note)
        }
        ".system.frequency.value" | ".system.frequency.max" | ".system.frequency.per" => {
            let frequency = dto_typed(&item.action.frequency, "HazardActionSource.frequency")?;
            match local_path {
                ".system.frequency.value" => {
                    owner_fingerprint("HazardFrequencySource.value", &frequency.value)
                }
                ".system.frequency.max" => {
                    owner_fingerprint("HazardFrequencySource.maximum", &frequency.maximum)
                }
                ".system.frequency.per" => {
                    owner_fingerprint("HazardFrequencySource.per", &frequency.per)
                }
                _ => return unknown_hazard_accessor("DTO fingerprint", local_path),
            }
        }
        ".system.selfEffect.uuid" | ".system.selfEffect.name" => {
            owner_fingerprint("HazardActionSource.self_effect", &item.action.self_effect)
        }
        ".system.bonus.value" => owner_fingerprint("HazardStrikeSource.bonus", &item.strike.bonus),
        ".system.attackEffects.value[]" => owner_fingerprint(
            "HazardStrikeSource.attack_effects",
            &item.strike.attack_effects,
        ),
        ".system.attack.value" => {
            owner_fingerprint("HazardStrikeSource.attack", &item.strike.attack)
        }
        ".system.weaponType.value" => {
            owner_fingerprint("HazardStrikeSource.weapon_type", &item.strike.weapon_type)
        }
        ".system.attackEffects.custom" => owner_fingerprint(
            "HazardStrikeSource.attack_effects_custom",
            &item.strike.attack_effects_custom,
        ),
        _ if local_path.starts_with(".system.damageRolls.*.") => {
            let damages = dto_typed(&item.strike.damage_rolls, "HazardStrikeSource.damage_rolls")?;
            let key = pointer_member_after(&occurrence.pointer, "damageRolls")?;
            let damage = damages
                .iter()
                .find(|damage| damage.source_key == key)
                .ok_or_else(|| {
                    error(
                        CoverageFailureCode::DtoMismatch,
                        format!("missing HazardDamageSource key {key}"),
                    )
                })?;
            if local_path.ends_with(".damage") {
                owner_fingerprint("HazardDamageSource.damage", &damage.damage)
            } else if local_path.ends_with(".damageType") {
                owner_fingerprint("HazardDamageSource.damage_type", &damage.damage_type)
            } else if local_path.ends_with(".category") {
                owner_fingerprint("HazardDamageSource.category", &damage.category)
            } else {
                return unknown_hazard_accessor("source DTO", path);
            }
        }
        _ if local_path.starts_with(".system.rules[].") => {
            let rules = dto_typed(&item.common.rules, "HazardItemCommonSource.rules")?;
            let rule_index = pointer_index_after(&occurrence.pointer, "rules")?;
            let rule = rules.get(rule_index).ok_or_else(|| {
                error(
                    CoverageFailureCode::DtoMismatch,
                    format!("missing HazardItemCommonSource.rules[{rule_index}]"),
                )
            })?;
            let exact: Value = serde_json::from_str(&rule.value).map_err(|message| {
                error(
                    CoverageFailureCode::DtoMismatch,
                    format!("invalid typed rule source JSON: {message}"),
                )
            })?;
            let member = local_path
                .rsplit('.')
                .next()
                .unwrap_or_default()
                .trim_end_matches("[]");
            let member_value = exact.get(member).ok_or_else(|| {
                error(
                    CoverageFailureCode::DtoMismatch,
                    format!("typed rule source owner omitted member {member}"),
                )
            })?;
            owner_fingerprint("HazardItemCommonSource.rules[] member", member_value)
        }
        _ => return unknown_hazard_accessor("source DTO", path),
    };
    Ok(fingerprint)
}

fn exact_hazard_item_source_value(
    source: &crate::source::dto::HazardSource,
    path: &str,
    occurrence: &ExactHazardLeafOccurrence,
    disposition: HazardExactDisposition,
    selector: &SourceLeafSelector,
) -> Result<SourcePresence<SourceLeafValue>, CoverageContractError> {
    let items = dto_typed(&source.items, "HazardSource.items")?;
    let index = pointer_index_after(&occurrence.pointer, "items")?;
    let item = items.get(index).ok_or_else(|| {
        error(
            CoverageFailureCode::DtoMismatch,
            format!("missing HazardItemSource at authored ordinal {index}"),
        )
    })?;
    if item.source_ordinal != index as u32 {
        return Err(error(
            CoverageFailureCode::DtoMismatch,
            format!("HazardItemSource[{index}] has wrong authored order"),
        ));
    }
    let local_path = path
        .strip_prefix("$.items[]")
        .or_else(|| path.strip_prefix('$'))
        .unwrap_or(path);
    if path.starts_with("$.items[]") && !local_path.contains("[]") && !local_path.contains(".*") {
        verify_owner_collection(
            occurrence,
            SourceMemberKind::Array,
            None,
            item.source_ordinal as usize,
            items.len(),
            CoverageFailureCode::DtoMismatch,
            "HazardSource.items",
        )?;
    }
    if disposition == HazardExactDisposition::TypedUnsupported
        && selector.type_discriminator == "consumable"
    {
        return Ok(owner_leaf_value(
            occurrence,
            disposition,
            exact_summary_value(&item.unclaimed, &occurrence.pointer)?,
        ));
    }
    match local_path {
        "._id" => dto_field_owner_value(&item.id, occurrence, disposition, json_owner_value),
        ".name" => dto_field_owner_value(&item.name, occurrence, disposition, json_owner_value),
        ".type" => {
            dto_field_owner_value(&item.item_type, occurrence, disposition, json_owner_value)
        }
        ".img" => dto_field_owner_value(&item.image, occurrence, disposition, json_owner_value),
        ".folder" => dto_field_owner_value(&item.folder, occurrence, disposition, json_owner_value),
        ".sort" => dto_field_owner_value(&item.sort, occurrence, disposition, json_owner_value),
        ".system.description.value" => dto_field_owner_value(
            &item.common.description,
            occurrence,
            disposition,
            json_owner_value,
        ),
        ".system.publication.title"
        | ".system.publication.remaster"
        | ".system.publication.license" => {
            let publication = dto_typed(
                &item.common.publication,
                "HazardItemCommonSource.publication",
            )?;
            let field = match local_path {
                ".system.publication.title" => &publication.title,
                ".system.publication.license" => &publication.license,
                ".system.publication.remaster" => {
                    return dto_field_owner_value(
                        &publication.remaster,
                        occurrence,
                        disposition,
                        json_owner_value,
                    );
                }
                _ => return unknown_hazard_accessor("DTO value", local_path),
            };
            dto_field_owner_value(field, occurrence, disposition, json_owner_value)
        }
        ".system.slug" => {
            dto_field_owner_value(&item.common.slug, occurrence, disposition, json_owner_value)
        }
        ".system.traits.value[]" => {
            dto_array_member_owner_value(&item.common.traits, occurrence, disposition)
        }
        ".system.traits.rarity" => dto_field_owner_value(
            &item.common.rarity,
            occurrence,
            disposition,
            json_owner_value,
        ),
        "._stats.compendiumSource" => {
            let lineage = dto_typed(&item.common.lineage, "HazardItemCommonSource.lineage")?;
            dto_field_owner_value(
                &lineage.compendium_source,
                occurrence,
                disposition,
                json_owner_value,
            )
        }
        ".system.actionType.value" => dto_field_owner_value(
            &item.action.action_type,
            occurrence,
            disposition,
            json_owner_value,
        ),
        ".system.actions.value" => dto_field_owner_value(
            &item.action.actions,
            occurrence,
            disposition,
            json_owner_value,
        ),
        ".system.category" => dto_field_owner_value(
            &item.action.category,
            occurrence,
            disposition,
            json_owner_value,
        ),
        ".system.deathNote" => dto_field_owner_value(
            &item.action.death_note,
            occurrence,
            disposition,
            json_owner_value,
        ),
        ".system.frequency.value" | ".system.frequency.max" | ".system.frequency.per" => {
            let frequency = match &item.action.frequency {
                SourcePresence::Missing | SourcePresence::Null => {
                    return Ok(SourcePresence::Missing);
                }
                SourcePresence::Value(DtoHazardSourceValue::Typed(value)) => value,
                SourcePresence::Value(DtoHazardSourceValue::Unsupported(_)) => {
                    return Err(error(
                        CoverageFailureCode::DtoMismatch,
                        "HazardActionSource.frequency has no typed nested leaf owner",
                    ));
                }
            };
            if local_path.ends_with(".value") {
                dto_field_owner_value(&frequency.value, occurrence, disposition, json_owner_value)
            } else if local_path.ends_with(".max") {
                dto_field_owner_value(
                    &frequency.maximum,
                    occurrence,
                    disposition,
                    json_owner_value,
                )
            } else {
                dto_field_owner_value(&frequency.per, occurrence, disposition, json_owner_value)
            }
        }
        ".system.selfEffect.uuid" | ".system.selfEffect.name" => {
            let self_effect = match &item.action.self_effect {
                SourcePresence::Missing | SourcePresence::Null => {
                    return Ok(SourcePresence::Missing);
                }
                SourcePresence::Value(DtoHazardSourceValue::Typed(value)) => value,
                SourcePresence::Value(DtoHazardSourceValue::Unsupported(_)) => {
                    return Err(error(
                        CoverageFailureCode::DtoMismatch,
                        "HazardActionSource.self_effect has no typed nested leaf owner",
                    ));
                }
            };
            if local_path.ends_with(".uuid") {
                dto_field_owner_value(&self_effect.uuid, occurrence, disposition, json_owner_value)
            } else {
                dto_field_owner_value(&self_effect.name, occurrence, disposition, json_owner_value)
            }
        }
        ".system.bonus.value" => dto_field_owner_value(
            &item.strike.bonus,
            occurrence,
            disposition,
            json_owner_value,
        ),
        ".system.attackEffects.value[]" => {
            dto_array_member_owner_value(&item.strike.attack_effects, occurrence, disposition)
        }
        ".system.attack.value" => dto_field_owner_value(
            &item.strike.attack,
            occurrence,
            disposition,
            json_owner_value,
        ),
        ".system.weaponType.value" => dto_field_owner_value(
            &item.strike.weapon_type,
            occurrence,
            disposition,
            json_owner_value,
        ),
        ".system.attackEffects.custom" => dto_field_owner_value(
            &item.strike.attack_effects_custom,
            occurrence,
            disposition,
            json_owner_value,
        ),
        _ if local_path.starts_with(".system.damageRolls.*.") => {
            let damages = dto_typed(&item.strike.damage_rolls, "HazardStrikeSource.damage_rolls")?;
            let key = pointer_member_after(&occurrence.pointer, "damageRolls")?;
            let damage = damages
                .iter()
                .find(|damage| damage.source_key == key)
                .ok_or_else(|| {
                    error(
                        CoverageFailureCode::DtoMismatch,
                        format!("missing HazardDamageSource key {key}"),
                    )
                })?;
            let identity_matches = damage
                .source_path
                .rsplit_once('/')
                .map(|(_, key)| key)
                .filter(|owner_key| *owner_key == key)
                .is_some();
            verify_owner_collection(
                occurrence,
                SourceMemberKind::Map,
                Some(&damage.source_key),
                damage.authored_order as usize,
                damages.len(),
                CoverageFailureCode::DtoMismatch,
                "HazardStrikeSource.damage_rolls",
            )?;
            if !identity_matches {
                return Err(error(
                    CoverageFailureCode::DtoMismatch,
                    format!("HazardDamageSource key {key} lost source identity"),
                ));
            }
            if local_path.ends_with(".damage") {
                dto_field_owner_value(&damage.damage, occurrence, disposition, json_owner_value)
            } else if local_path.ends_with(".damageType") {
                dto_field_owner_value(
                    &damage.damage_type,
                    occurrence,
                    disposition,
                    json_owner_value,
                )
            } else if local_path.ends_with(".category") {
                dto_field_owner_value(&damage.category, occurrence, disposition, json_owner_value)
            } else {
                unknown_hazard_accessor("source DTO value", path)
            }
        }
        _ if local_path.starts_with(".system.rules[].") => {
            let rules = dto_typed(&item.common.rules, "HazardItemCommonSource.rules")?;
            let rule_index = pointer_index_after(&occurrence.pointer, "rules")?;
            let rule = rules.get(rule_index).ok_or_else(|| {
                error(
                    CoverageFailureCode::DtoMismatch,
                    format!("missing HazardItemCommonSource.rules[{rule_index}]"),
                )
            })?;
            if rule.source_path != format!("/items/{index}/system/rules/{rule_index}") {
                return Err(error(
                    CoverageFailureCode::DtoMismatch,
                    format!("HazardItemCommonSource.rules[{rule_index}] lost authored identity"),
                ));
            }
            if !local_path.ends_with("[]") {
                verify_owner_collection(
                    occurrence,
                    SourceMemberKind::Array,
                    None,
                    rule_index,
                    rules.len(),
                    CoverageFailureCode::DtoMismatch,
                    "HazardItemCommonSource.rules",
                )?;
            }
            let exact: Value = serde_json::from_str(&rule.value).map_err(|message| {
                error(
                    CoverageFailureCode::DtoMismatch,
                    format!("invalid typed rule source JSON: {message}"),
                )
            })?;
            let member = local_path
                .rsplit('.')
                .next()
                .unwrap_or_default()
                .trim_end_matches("[]");
            let mut member_value = exact.get(member).cloned().ok_or_else(|| {
                error(
                    CoverageFailureCode::DtoMismatch,
                    format!("typed rule source owner omitted member {member}"),
                )
            })?;
            if local_path.ends_with("[]") {
                let member_index = last_pointer_index(&occurrence.pointer)?;
                let member_values = member_value.as_array().ok_or_else(|| {
                    error(
                        CoverageFailureCode::DtoMismatch,
                        format!("typed rule source owner {member} is not an array"),
                    )
                })?;
                verify_owner_collection(
                    occurrence,
                    SourceMemberKind::Array,
                    None,
                    member_index,
                    member_values.len(),
                    CoverageFailureCode::DtoMismatch,
                    "HazardItemCommonSource.rules nested array",
                )?;
                member_value = member_values.get(member_index).cloned().ok_or_else(|| {
                    error(
                        CoverageFailureCode::DtoMismatch,
                        format!("typed rule source owner omitted {member}[{member_index}]"),
                    )
                })?;
            }
            Ok(owner_leaf_value(occurrence, disposition, member_value))
        }
        _ => unknown_hazard_accessor("source DTO value", path),
    }
}

fn exact_summary_fingerprint(
    summaries: &[crate::source::dto::ValueSummary],
    pointer: &str,
    owner: &str,
) -> Result<String, CoverageContractError> {
    let summary = summaries
        .iter()
        .find(|summary| summary.source_path == pointer)
        .ok_or_else(|| {
            error(
                CoverageFailureCode::DtoMismatch,
                format!("{owner} omitted exact source path {pointer}"),
            )
        })?;
    Ok(owner_fingerprint(owner, summary))
}

fn canonical_fact_owner_value<T>(
    fact: &atlas_record::HazardFact<T>,
    occurrence: &ExactHazardLeafOccurrence,
    disposition: HazardExactDisposition,
    encode: impl FnOnce(&T) -> Result<Value, CoverageContractError>,
) -> Result<SourcePresence<SourceLeafValue>, CoverageContractError> {
    match &fact.value {
        FactValue::Missing => Ok(SourcePresence::Missing),
        FactValue::Null => Ok(if disposition == HazardExactDisposition::TypedUnsupported {
            owner_leaf_value(occurrence, disposition, Value::Null)
        } else {
            SourcePresence::Null
        }),
        FactValue::Value(CanonicalHazardSourceValue::Typed(value)) => {
            Ok(owner_leaf_value(occurrence, disposition, encode(value)?))
        }
        FactValue::Value(CanonicalHazardSourceValue::Unsupported(value)) => Ok(owner_leaf_value(
            occurrence,
            disposition,
            serde_json::from_str(&value.exact_json).map_err(|message| {
                error(
                    CoverageFailureCode::CanonicalMismatch,
                    format!(
                        "invalid exact canonical owner JSON at {}: {message}",
                        value.relative_source_path
                    ),
                )
            })?,
        )),
    }
}

fn canonical_rich_owner_value(
    fact: &atlas_record::HazardFact<atlas_record::RichDocument>,
    occurrence: &ExactHazardLeafOccurrence,
    disposition: HazardExactDisposition,
) -> Result<SourcePresence<SourceLeafValue>, CoverageContractError> {
    match &fact.value {
        FactValue::Missing => Ok(SourcePresence::Missing),
        FactValue::Null => Ok(SourcePresence::Null),
        FactValue::Value(CanonicalHazardSourceValue::Unsupported(value)) => Ok(owner_leaf_value(
            occurrence,
            disposition,
            serde_json::from_str(&value.exact_json).map_err(|message| {
                error(
                    CoverageFailureCode::CanonicalMismatch,
                    format!(
                        "invalid exact canonical rich owner JSON at {}: {message}",
                        value.relative_source_path
                    ),
                )
            })?,
        )),
        FactValue::Value(CanonicalHazardSourceValue::Typed(document)) => {
            let source_markup = occurrence
                .value
                .as_ref()
                .and_then(Value::as_str)
                .ok_or_else(|| {
                    error(
                        CoverageFailureCode::CanonicalMismatch,
                        "typed RichDocument owner has no authenticated string source value",
                    )
                })?;
            let expected = parse_foundry_content_with_localization(source_markup, None).document;
            if !rich_documents_source_equivalent(document, &expected) {
                return Err(error(
                    CoverageFailureCode::CanonicalMismatch,
                    format!(
                        "typed RichDocument owner at {} is not semantically equivalent to its authenticated source markup",
                        occurrence.pointer
                    ),
                ));
            }
            Ok(owner_leaf_value(
                occurrence,
                disposition,
                Value::String(source_markup.to_string()),
            ))
        }
    }
}

fn rich_documents_source_equivalent(
    actual: &atlas_record::RichDocument,
    expected: &atlas_record::RichDocument,
) -> bool {
    fn erase_derived_target(document: &mut atlas_record::RichDocument) {
        atlas_record::visit_foundry_links_mut(document, |link| {
            link.target = atlas_record::RichLinkTarget::External {
                target: link.source.authored_target.clone(),
                label: None,
            };
        });
    }

    let mut actual = actual.clone();
    let mut expected = expected.clone();
    erase_derived_target(&mut actual);
    erase_derived_target(&mut expected);
    actual == expected
}

fn canonical_array_member<T: Serialize>(
    values: &[T],
    occurrence: &ExactHazardLeafOccurrence,
) -> Result<Value, CoverageContractError> {
    array_member_owner_value(
        values,
        occurrence,
        CoverageFailureCode::CanonicalMismatch,
        "exact canonical array",
    )
}

fn canonical_array_member_fact_owner_value<T>(
    fact: &atlas_record::HazardFact<Vec<T>>,
    occurrence: &ExactHazardLeafOccurrence,
    disposition: HazardExactDisposition,
    encode: impl FnOnce(&T) -> Result<Value, CoverageContractError>,
) -> Result<SourcePresence<SourceLeafValue>, CoverageContractError> {
    match &fact.value {
        FactValue::Missing => Ok(SourcePresence::Missing),
        FactValue::Null => Ok(SourcePresence::Null),
        FactValue::Value(CanonicalHazardSourceValue::Typed(values)) => {
            if occurrence.value.is_none() && values.is_empty() {
                return Ok(SourcePresence::Missing);
            }
            let index = last_pointer_index(&occurrence.pointer)?;
            verify_owner_collection(
                occurrence,
                SourceMemberKind::Array,
                None,
                index,
                values.len(),
                CoverageFailureCode::CanonicalMismatch,
                "exact canonical typed array",
            )?;
            match values.get(index) {
                Some(value) => Ok(owner_leaf_value(occurrence, disposition, encode(value)?)),
                None => Ok(SourcePresence::Missing),
            }
        }
        FactValue::Value(CanonicalHazardSourceValue::Unsupported(value)) => {
            let exact: Value = serde_json::from_str(&value.exact_json).map_err(|message| {
                error(
                    CoverageFailureCode::CanonicalMismatch,
                    format!("invalid exact canonical array owner JSON: {message}"),
                )
            })?;
            let index = last_pointer_index(&occurrence.pointer)?;
            let values = exact.as_array().ok_or_else(|| {
                error(
                    CoverageFailureCode::CanonicalMismatch,
                    "unsupported canonical array owner is not an array",
                )
            })?;
            verify_owner_collection(
                occurrence,
                SourceMemberKind::Array,
                None,
                index,
                values.len(),
                CoverageFailureCode::CanonicalMismatch,
                "exact unsupported canonical array",
            )?;
            let member = values.get(index).cloned().ok_or_else(|| {
                error(
                    CoverageFailureCode::CanonicalMismatch,
                    format!("unsupported canonical array owner omitted member {index}"),
                )
            })?;
            Ok(owner_leaf_value(occurrence, disposition, member))
        }
    }
}

fn is_named_hazard_source_metadata_path(path: &str) -> bool {
    let local_path = path
        .strip_prefix("$.items[]")
        .or_else(|| path.strip_prefix('$'))
        .unwrap_or(path);
    matches!(
        path,
        "$.prototypeToken.name"
            | "$.system.attributes.hasHealth"
            | "$.system.attributes.hp.tempmax"
            | "$.system.saves.*.saveDetail"
    ) || matches!(
        local_path,
        "._stats.compendiumSource"
            | ".system.traits.rarity"
            | ".system.attack.value"
            | ".system.weaponType.value"
            | ".system.attackEffects.custom"
    )
}

fn exact_hazard_canonical_value(
    hazard: &HazardRecord,
    identity: &SourceLeafIdentity,
    occurrence: &ExactHazardLeafOccurrence,
    disposition: HazardExactDisposition,
) -> Result<SourcePresence<SourceLeafValue>, CoverageContractError> {
    let path = identity.normalized_path.as_str();
    verify_canonical_item_collection_if_leaf(hazard, path, occurrence)?;
    verify_canonical_save_collection_if_leaf(hazard, path, occurrence)?;
    if disposition == HazardExactDisposition::TypedUnsupported
        && !is_named_hazard_source_metadata_path(path)
    {
        return exact_hazard_unsupported_value(hazard, occurrence, disposition);
    }
    if identity.selector.role == SourceDocumentRole::Embedded {
        return exact_hazard_item_canonical_value(hazard, path, occurrence, disposition);
    }
    match path {
        "$._id" => Ok(owner_leaf_value(
            occurrence,
            disposition,
            Value::String(hazard.identity.source_id.as_str().to_string()),
        )),
        "$.name" => Ok(owner_leaf_value(
            occurrence,
            disposition,
            Value::String(hazard.identity.name.clone()),
        )),
        "$.type" => Ok(owner_leaf_value(
            occurrence,
            disposition,
            Value::String("hazard".to_string()),
        )),
        "$.img" => canonical_fact_owner_value(
            &hazard.provenance.image,
            occurrence,
            disposition,
            json_owner_value,
        ),
        "$.folder" => canonical_fact_owner_value(
            &hazard.provenance.source_folder,
            occurrence,
            disposition,
            json_owner_value,
        ),
        "$.effects" => canonical_fact_owner_value(
            &hazard.provenance.actor_effects,
            occurrence,
            disposition,
            |values| {
                values
                    .iter()
                    .map(|value| serde_json::from_str(&value.exact_json))
                    .collect::<Result<Vec<Value>, _>>()
                    .map(Value::Array)
                    .map_err(|message| {
                        error(
                            CoverageFailureCode::CanonicalMismatch,
                            format!("invalid HazardProvenance.actor_effects JSON: {message}"),
                        )
                    })
            },
        ),
        "$.system.creatureType" => canonical_fact_owner_value(
            &hazard.provenance.source_creature_type,
            occurrence,
            disposition,
            json_owner_value,
        ),
        "$.system.statusEffects[]" => canonical_array_member_fact_owner_value(
            &hazard.provenance.source_status_effects,
            occurrence,
            disposition,
            json_owner_value,
        ),
        "$.system.details.level.value" => {
            canonical_fact_owner_value(&hazard.level, occurrence, disposition, json_owner_value)
        }
        "$.system.details.isComplex" => {
            canonical_fact_owner_value(&hazard.complexity, occurrence, disposition, |value| {
                Ok(Value::Bool(
                    *value == atlas_record::HazardComplexity::Complex,
                ))
            })
        }
        "$.system.attributes.emitsSound" => {
            canonical_fact_owner_value(&hazard.emits_sound, occurrence, disposition, |value| {
                match value {
                    atlas_record::HazardEmitsSound::Boolean(value) => Ok(Value::Bool(*value)),
                    atlas_record::HazardEmitsSound::Named(value) => {
                        Ok(Value::String(value.clone()))
                    }
                }
            })
        }
        "$.prototypeToken.name" => {
            let token = hazard.provenance.token.typed().ok_or_else(|| {
                error(
                    CoverageFailureCode::CanonicalMismatch,
                    "HazardProvenance.token is not typed",
                )
            })?;
            canonical_fact_owner_value(&token.name, occurrence, disposition, json_owner_value)
        }
        _ if path.starts_with("$.system.traits.") => match path {
            "$.system.traits.value[]" => canonical_array_member_fact_owner_value(
                &hazard.traits,
                occurrence,
                disposition,
                |value| Ok(Value::String(value.as_str().to_string())),
            ),
            "$.system.traits.rarity" => {
                canonical_fact_owner_value(&hazard.rarity, occurrence, disposition, |value| {
                    Ok(Value::String(value.as_str().to_string()))
                })
            }
            "$.system.traits.size.value" => {
                canonical_fact_owner_value(&hazard.size, occurrence, disposition, |value| {
                    Ok(Value::String(value.as_source().to_string()))
                })
            }
            _ => unknown_hazard_accessor("canonical value", path),
        },
        _ if path.starts_with("$.system.details.publication.") => {
            let publication = hazard.publication.typed().ok_or_else(|| {
                error(
                    CoverageFailureCode::CanonicalMismatch,
                    "HazardRecord.publication is not typed",
                )
            })?;
            match path {
                "$.system.details.publication.title" => canonical_fact_owner_value(
                    &publication.title,
                    occurrence,
                    disposition,
                    json_owner_value,
                ),
                "$.system.details.publication.remaster" => canonical_fact_owner_value(
                    &publication.remaster,
                    occurrence,
                    disposition,
                    json_owner_value,
                ),
                "$.system.details.publication.license" => canonical_fact_owner_value(
                    &publication.license,
                    occurrence,
                    disposition,
                    |value| Ok(Value::String(value.as_str().to_string())),
                ),
                _ => unknown_hazard_accessor("canonical value", path),
            }
        }
        _ if path.starts_with("$.system.details.") => {
            let lifecycle = hazard.lifecycle.typed().ok_or_else(|| {
                error(
                    CoverageFailureCode::CanonicalMismatch,
                    "HazardRecord.lifecycle is not typed",
                )
            })?;
            let field = match path {
                "$.system.details.description" => &lifecycle.description,
                "$.system.details.disable" => &lifecycle.disable,
                "$.system.details.routine" => &lifecycle.routine,
                "$.system.details.reset" => &lifecycle.reset,
                _ => return unknown_hazard_accessor("canonical value", path),
            };
            canonical_rich_owner_value(field, occurrence, disposition)
        }
        _ if path.starts_with("$.system.attributes.stealth.") => {
            let detection = hazard.detection.typed().ok_or_else(|| {
                error(
                    CoverageFailureCode::CanonicalMismatch,
                    "HazardRecord.detection is not typed",
                )
            })?;
            match path {
                "$.system.attributes.stealth.value" => canonical_fact_owner_value(
                    &detection.stealth_modifier,
                    occurrence,
                    disposition,
                    json_owner_value,
                ),
                "$.system.attributes.stealth.details" => {
                    canonical_rich_owner_value(&detection.details, occurrence, disposition)
                }
                _ => unknown_hazard_accessor("canonical value", path),
            }
        }
        _ if path.starts_with("$.system.attributes.") || path.starts_with("$.system.saves.") => {
            exact_hazard_defense_canonical_value(hazard, path, occurrence, disposition)
        }
        _ if path.starts_with("$.items[]") => {
            exact_hazard_item_canonical_value(hazard, path, occurrence, disposition)
        }
        _ => unknown_hazard_accessor("canonical value", path),
    }
}

fn exact_hazard_canonical_fingerprint(
    hazard: &HazardRecord,
    identity: &SourceLeafIdentity,
    occurrence: &ExactHazardLeafOccurrence,
    disposition: HazardExactDisposition,
) -> Result<String, CoverageContractError> {
    let path = identity.normalized_path.as_str();
    if disposition == HazardExactDisposition::TypedUnsupported
        && !is_named_hazard_source_metadata_path(path)
    {
        return exact_hazard_unsupported_fingerprint(hazard, &occurrence.pointer);
    }
    if identity.selector.role == SourceDocumentRole::Embedded {
        return exact_hazard_item_canonical_fingerprint(hazard, path, occurrence);
    }
    let fingerprint = match path {
        "$._id" => owner_fingerprint(
            "HazardRecord.identity.source_id",
            &hazard.identity.source_id,
        ),
        "$.name" => owner_fingerprint("HazardRecord.identity.name", &hazard.identity.name),
        "$.type" => owner_fingerprint("RecordBody::Hazard", &"hazard"),
        "$.img" => owner_fingerprint("HazardRecord.provenance.image", &hazard.provenance.image),
        "$.folder" => owner_fingerprint(
            "HazardRecord.provenance.source_folder",
            &hazard.provenance.source_folder,
        ),
        "$.effects" => owner_fingerprint(
            "HazardRecord.provenance.actor_effects",
            &hazard.provenance.actor_effects,
        ),
        "$.system.creatureType" => owner_fingerprint(
            "HazardRecord.provenance.source_creature_type",
            &hazard.provenance.source_creature_type,
        ),
        "$.system.statusEffects[]" => owner_fingerprint(
            "HazardRecord.provenance.source_status_effects",
            &hazard.provenance.source_status_effects,
        ),
        "$.system.details.level.value" => owner_fingerprint("HazardRecord.level", &hazard.level),
        "$.system.details.isComplex" => {
            owner_fingerprint("HazardRecord.complexity", &hazard.complexity)
        }
        "$.system.attributes.emitsSound" => {
            owner_fingerprint("HazardRecord.emits_sound", &hazard.emits_sound)
        }
        "$.prototypeToken.name" => {
            owner_fingerprint("HazardProvenance.token", &hazard.provenance.token)
        }
        _ if path.starts_with("$.system.traits.") => match path {
            "$.system.traits.value[]" => owner_fingerprint("HazardRecord.traits", &hazard.traits),
            "$.system.traits.rarity" => owner_fingerprint("HazardRecord.rarity", &hazard.rarity),
            "$.system.traits.size.value" => owner_fingerprint("HazardRecord.size", &hazard.size),
            _ => return unknown_hazard_accessor("canonical", path),
        },
        _ if path.starts_with("$.system.details.publication.") => {
            let publication = hazard.publication.typed().ok_or_else(|| {
                error(
                    CoverageFailureCode::CanonicalMismatch,
                    "HazardRecord.publication is not typed",
                )
            })?;
            match path {
                "$.system.details.publication.title" => {
                    owner_fingerprint("HazardPublication.title", &publication.title)
                }
                "$.system.details.publication.remaster" => {
                    owner_fingerprint("HazardPublication.remaster", &publication.remaster)
                }
                "$.system.details.publication.license" => {
                    owner_fingerprint("HazardPublication.license", &publication.license)
                }
                _ => return unknown_hazard_accessor("canonical", path),
            }
        }
        _ if path.starts_with("$.system.details.") => {
            let lifecycle = hazard.lifecycle.typed().ok_or_else(|| {
                error(
                    CoverageFailureCode::CanonicalMismatch,
                    "HazardRecord.lifecycle is not typed",
                )
            })?;
            match path {
                "$.system.details.description" => {
                    owner_fingerprint("HazardLifecycle.description", &lifecycle.description)
                }
                "$.system.details.disable" => {
                    owner_fingerprint("HazardLifecycle.disable", &lifecycle.disable)
                }
                "$.system.details.routine" => {
                    owner_fingerprint("HazardLifecycle.routine", &lifecycle.routine)
                }
                "$.system.details.reset" => {
                    owner_fingerprint("HazardLifecycle.reset", &lifecycle.reset)
                }
                _ => return unknown_hazard_accessor("canonical", path),
            }
        }
        _ if path.starts_with("$.system.attributes.stealth.") => {
            let detection = hazard.detection.typed().ok_or_else(|| {
                error(
                    CoverageFailureCode::CanonicalMismatch,
                    "HazardRecord.detection is not typed",
                )
            })?;
            match path {
                "$.system.attributes.stealth.value" => owner_fingerprint(
                    "HazardDetection.stealth_modifier",
                    &detection.stealth_modifier,
                ),
                "$.system.attributes.stealth.details" => {
                    owner_fingerprint("HazardDetection.details", &detection.details)
                }
                _ => return unknown_hazard_accessor("canonical", path),
            }
        }
        _ if path.starts_with("$.system.attributes.") || path.starts_with("$.system.saves.") => {
            exact_hazard_defense_canonical_fingerprint(hazard, path, occurrence)?
        }
        _ if path.starts_with("$.items[]")
            || identity.selector.role == SourceDocumentRole::Embedded =>
        {
            exact_hazard_item_canonical_fingerprint(hazard, path, occurrence)?
        }
        _ => return unknown_hazard_accessor("canonical", path),
    };
    Ok(fingerprint)
}

fn exact_hazard_defense_canonical_fingerprint(
    hazard: &HazardRecord,
    path: &str,
    occurrence: &ExactHazardLeafOccurrence,
) -> Result<String, CoverageContractError> {
    let defenses = hazard.defenses.typed().ok_or_else(|| {
        error(
            CoverageFailureCode::CanonicalMismatch,
            "HazardRecord.defenses is not typed",
        )
    })?;
    let fingerprint = match path {
        "$.system.attributes.ac.value" => {
            owner_fingerprint("HazardDefenses.armor_class", &defenses.armor_class)
        }
        "$.system.attributes.hardness" => {
            owner_fingerprint("HazardDefenses.hardness", &defenses.hardness)
        }
        "$.system.attributes.hasHealth" => owner_fingerprint(
            "HazardDefenseSourceMetadata.has_health",
            &defenses.source_metadata.has_health,
        ),
        _ if path.starts_with("$.system.attributes.hp.") => {
            let hit_points = defenses.hit_points.typed().ok_or_else(|| {
                error(
                    CoverageFailureCode::CanonicalMismatch,
                    "HazardDefenses.hit_points is not typed",
                )
            })?;
            match path {
                "$.system.attributes.hp.value" => {
                    owner_fingerprint("HazardHitPoints.current", &hit_points.current)
                }
                "$.system.attributes.hp.max" => {
                    owner_fingerprint("HazardHitPoints.maximum", &hit_points.maximum)
                }
                "$.system.attributes.hp.temp" => {
                    owner_fingerprint("HazardHitPoints.temporary", &hit_points.temporary)
                }
                "$.system.attributes.hp.details" => {
                    owner_fingerprint("HazardHitPoints.details", &hit_points.details)
                }
                "$.system.attributes.hp.tempmax" => owner_fingerprint(
                    "HazardHitPointSourceMetadata.temporary_maximum",
                    &hit_points.source_metadata.temporary_maximum,
                ),
                _ => return unknown_hazard_accessor("canonical", path),
            }
        }
        "$.system.saves.*.value" | "$.system.saves.*.saveDetail" => {
            let saves = defenses.saves.typed().ok_or_else(|| {
                error(
                    CoverageFailureCode::CanonicalMismatch,
                    "HazardDefenses.saves is not typed",
                )
            })?;
            match (
                pointer_member_after(&occurrence.pointer, "saves")?.as_str(),
                path.ends_with(".saveDetail"),
            ) {
                ("fortitude", false) => {
                    owner_fingerprint("HazardSaves.fortitude", &saves.fortitude)
                }
                ("reflex", false) => owner_fingerprint("HazardSaves.reflex", &saves.reflex),
                ("will", false) => owner_fingerprint("HazardSaves.will", &saves.will),
                ("fortitude", true) => owner_fingerprint(
                    "HazardSaveSourceMetadata.fortitude_detail",
                    &saves.source_metadata.fortitude_detail,
                ),
                ("reflex", true) => owner_fingerprint(
                    "HazardSaveSourceMetadata.reflex_detail",
                    &saves.source_metadata.reflex_detail,
                ),
                ("will", true) => owner_fingerprint(
                    "HazardSaveSourceMetadata.will_detail",
                    &saves.source_metadata.will_detail,
                ),
                _ => return unknown_hazard_accessor("canonical save", path),
            }
        }
        _ if path.starts_with("$.system.attributes.immunities[]")
            || path.starts_with("$.system.attributes.weaknesses[]")
            || path.starts_with("$.system.attributes.resistances[]") =>
        {
            let (family, entries) = if path.contains(".immunities[]") {
                ("immunities", &defenses.immunities)
            } else if path.contains(".weaknesses[]") {
                ("weaknesses", &defenses.weaknesses)
            } else {
                ("resistances", &defenses.resistances)
            };
            let entries = entries.typed().ok_or_else(|| {
                error(
                    CoverageFailureCode::CanonicalMismatch,
                    format!("HazardDefenses.{family} is not typed"),
                )
            })?;
            let index = pointer_index_after(&occurrence.pointer, family)?;
            let entry = entries.get(index).ok_or_else(|| {
                error(
                    CoverageFailureCode::CanonicalMismatch,
                    format!("missing HazardDefenses.{family}[{index}]"),
                )
            })?;
            if path.ends_with(".type") {
                owner_fingerprint("HazardIwr.iwr_type", &entry.iwr_type)
            } else if path.ends_with(".value") {
                owner_fingerprint("HazardIwr.value", &entry.value)
            } else if path.ends_with(".exceptions[]") {
                owner_fingerprint("HazardIwr.exceptions", &entry.exceptions)
            } else if path.ends_with(".doubleVs[]") {
                owner_fingerprint("HazardIwr.double_vs", &entry.double_vs)
            } else {
                return unknown_hazard_accessor("canonical", path);
            }
        }
        _ => return unknown_hazard_accessor("canonical", path),
    };
    Ok(fingerprint)
}

fn exact_hazard_defense_canonical_value(
    hazard: &HazardRecord,
    path: &str,
    occurrence: &ExactHazardLeafOccurrence,
    disposition: HazardExactDisposition,
) -> Result<SourcePresence<SourceLeafValue>, CoverageContractError> {
    let defenses = hazard.defenses.typed().ok_or_else(|| {
        error(
            CoverageFailureCode::CanonicalMismatch,
            "HazardRecord.defenses is not typed",
        )
    })?;
    match path {
        "$.system.attributes.ac.value" => canonical_fact_owner_value(
            &defenses.armor_class,
            occurrence,
            disposition,
            json_owner_value,
        ),
        "$.system.attributes.hardness" => canonical_fact_owner_value(
            &defenses.hardness,
            occurrence,
            disposition,
            json_owner_value,
        ),
        "$.system.attributes.hasHealth" => canonical_fact_owner_value(
            &defenses.source_metadata.has_health,
            occurrence,
            disposition,
            json_owner_value,
        ),
        _ if path.starts_with("$.system.attributes.hp.") => {
            let hit_points = defenses.hit_points.typed().ok_or_else(|| {
                error(
                    CoverageFailureCode::CanonicalMismatch,
                    "HazardDefenses.hit_points is not typed",
                )
            })?;
            match path {
                "$.system.attributes.hp.value" => canonical_fact_owner_value(
                    &hit_points.current,
                    occurrence,
                    disposition,
                    json_owner_value,
                ),
                "$.system.attributes.hp.max" => canonical_fact_owner_value(
                    &hit_points.maximum,
                    occurrence,
                    disposition,
                    json_owner_value,
                ),
                "$.system.attributes.hp.temp" => canonical_fact_owner_value(
                    &hit_points.temporary,
                    occurrence,
                    disposition,
                    json_owner_value,
                ),
                "$.system.attributes.hp.details" => {
                    canonical_rich_owner_value(&hit_points.details, occurrence, disposition)
                }
                "$.system.attributes.hp.tempmax" => canonical_fact_owner_value(
                    &hit_points.source_metadata.temporary_maximum,
                    occurrence,
                    disposition,
                    json_owner_value,
                ),
                _ => unknown_hazard_accessor("canonical value", path),
            }
        }
        "$.system.saves.*.value" | "$.system.saves.*.saveDetail" => {
            let saves = defenses.saves.typed().ok_or_else(|| {
                error(
                    CoverageFailureCode::CanonicalMismatch,
                    "HazardDefenses.saves is not typed",
                )
            })?;
            match (
                pointer_member_after(&occurrence.pointer, "saves")?.as_str(),
                path.ends_with(".saveDetail"),
            ) {
                ("fortitude", false) => canonical_fact_owner_value(
                    &saves.fortitude,
                    occurrence,
                    disposition,
                    json_owner_value,
                ),
                ("reflex", false) => canonical_fact_owner_value(
                    &saves.reflex,
                    occurrence,
                    disposition,
                    json_owner_value,
                ),
                ("will", false) => canonical_fact_owner_value(
                    &saves.will,
                    occurrence,
                    disposition,
                    json_owner_value,
                ),
                ("fortitude", true) => canonical_fact_owner_value(
                    &saves.source_metadata.fortitude_detail,
                    occurrence,
                    disposition,
                    json_owner_value,
                ),
                ("reflex", true) => canonical_fact_owner_value(
                    &saves.source_metadata.reflex_detail,
                    occurrence,
                    disposition,
                    json_owner_value,
                ),
                ("will", true) => canonical_fact_owner_value(
                    &saves.source_metadata.will_detail,
                    occurrence,
                    disposition,
                    json_owner_value,
                ),
                _ => unknown_hazard_accessor("canonical save value", path),
            }
        }
        _ if path.starts_with("$.system.attributes.immunities[]")
            || path.starts_with("$.system.attributes.weaknesses[]")
            || path.starts_with("$.system.attributes.resistances[]") =>
        {
            let (family, entries) = if path.contains(".immunities[]") {
                ("immunities", &defenses.immunities)
            } else if path.contains(".weaknesses[]") {
                ("weaknesses", &defenses.weaknesses)
            } else {
                ("resistances", &defenses.resistances)
            };
            let entries = entries.typed().ok_or_else(|| {
                error(
                    CoverageFailureCode::CanonicalMismatch,
                    format!("HazardDefenses.{family} is not typed"),
                )
            })?;
            let index = pointer_index_after(&occurrence.pointer, family)?;
            let entry = entries.get(index).ok_or_else(|| {
                error(
                    CoverageFailureCode::CanonicalMismatch,
                    format!("missing HazardDefenses.{family}[{index}]"),
                )
            })?;
            if entry.authored_order != index as u32 {
                return Err(error(
                    CoverageFailureCode::CanonicalMismatch,
                    format!("HazardDefenses.{family}[{index}] lost authored order"),
                ));
            }
            if !path.ends_with("[]") {
                verify_owner_collection(
                    occurrence,
                    SourceMemberKind::Array,
                    None,
                    entry.authored_order as usize,
                    entries.len(),
                    CoverageFailureCode::CanonicalMismatch,
                    "HazardDefenses IWR entries",
                )?;
            }
            if path.ends_with(".type") {
                canonical_fact_owner_value(
                    &entry.iwr_type,
                    occurrence,
                    disposition,
                    json_owner_value,
                )
            } else if path.ends_with(".value") {
                canonical_fact_owner_value(&entry.value, occurrence, disposition, json_owner_value)
            } else if path.ends_with(".exceptions[]") {
                canonical_array_member_fact_owner_value(
                    &entry.exceptions,
                    occurrence,
                    disposition,
                    json_owner_value,
                )
            } else if path.ends_with(".doubleVs[]") {
                canonical_array_member_fact_owner_value(
                    &entry.double_vs,
                    occurrence,
                    disposition,
                    json_owner_value,
                )
            } else {
                unknown_hazard_accessor("canonical value", path)
            }
        }
        _ => unknown_hazard_accessor("canonical value", path),
    }
}

fn exact_hazard_item_canonical_fingerprint(
    hazard: &HazardRecord,
    path: &str,
    occurrence: &ExactHazardLeafOccurrence,
) -> Result<String, CoverageContractError> {
    let source_ordinal = pointer_index_after(&occurrence.pointer, "items")? as u32;
    let (entity, entity_occurrence) = hazard_entity_occurrence(hazard, source_ordinal)?;
    let local_path = path
        .strip_prefix("$.items[]")
        .or_else(|| path.strip_prefix('$'))
        .unwrap_or(path);
    let fingerprint = match local_path {
        "._id" => owner_fingerprint("HazardEntity.source_identity", &entity.source_identity),
        ".name" => owner_fingerprint("HazardEntity.label", &entity.label),
        ".type" => owner_fingerprint(
            "HazardEntity.family/capability",
            &(entity.family, &entity.capability),
        ),
        ".img" => owner_fingerprint("HazardEntity.image", &entity.image),
        ".folder" => owner_fingerprint(
            "HazardEntityOccurrence.source_folder",
            &entity_occurrence.source_folder,
        ),
        ".sort" => owner_fingerprint(
            "HazardEntityOccurrence.source_sort",
            &entity_occurrence.source_sort,
        ),
        ".system.description.value" => owner_fingerprint(
            "HazardItemCommon.description",
            &hazard_capability_common(&entity.capability).description,
        ),
        ".system.publication.title"
        | ".system.publication.remaster"
        | ".system.publication.license" => {
            let publication = hazard_capability_common(&entity.capability)
                .publication
                .typed()
                .ok_or_else(|| {
                    error(
                        CoverageFailureCode::CanonicalMismatch,
                        "HazardItemCommon.publication is not typed",
                    )
                })?;
            match local_path {
                ".system.publication.title" => {
                    owner_fingerprint("HazardPublication.title", &publication.title)
                }
                ".system.publication.remaster" => {
                    owner_fingerprint("HazardPublication.remaster", &publication.remaster)
                }
                ".system.publication.license" => {
                    owner_fingerprint("HazardPublication.license", &publication.license)
                }
                _ => return unknown_hazard_accessor("canonical fingerprint", local_path),
            }
        }
        ".system.slug" => owner_fingerprint(
            "HazardItemCommon.slug",
            &hazard_capability_common(&entity.capability).slug,
        ),
        ".system.traits.value[]" => owner_fingerprint(
            "HazardItemCommon.traits",
            &hazard_capability_common(&entity.capability).traits,
        ),
        ".system.traits.rarity" => owner_fingerprint(
            "HazardItemCommon.rarity",
            &hazard_capability_common(&entity.capability).rarity,
        ),
        "._stats.compendiumSource" => owner_fingerprint(
            "HazardItemCommon.lineage",
            &hazard_capability_common(&entity.capability).lineage,
        ),
        ".system.actionType.value"
        | ".system.actions.value"
        | ".system.category"
        | ".system.deathNote"
        | ".system.frequency.value"
        | ".system.frequency.max"
        | ".system.frequency.per"
        | ".system.selfEffect.uuid"
        | ".system.selfEffect.name" => {
            exact_hazard_action_canonical_fingerprint(&entity.capability, local_path)?
        }
        ".system.bonus.value"
        | ".system.attackEffects.value[]"
        | ".system.attack.value"
        | ".system.weaponType.value"
        | ".system.attackEffects.custom"
        | ".system.damageRolls.*.damage"
        | ".system.damageRolls.*.damageType"
        | ".system.damageRolls.*.category" => {
            exact_hazard_strike_canonical_fingerprint(&entity.capability, local_path, occurrence)?
        }
        _ if local_path.starts_with(".system.rules[].") => exact_hazard_rule_canonical_fingerprint(
            hazard_capability_common(&entity.capability),
            local_path,
            occurrence,
        )?,
        _ => return unknown_hazard_accessor("canonical", path),
    };
    Ok(fingerprint)
}

fn exact_hazard_item_canonical_value(
    hazard: &HazardRecord,
    path: &str,
    occurrence: &ExactHazardLeafOccurrence,
    disposition: HazardExactDisposition,
) -> Result<SourcePresence<SourceLeafValue>, CoverageContractError> {
    let source_ordinal = pointer_index_after(&occurrence.pointer, "items")? as u32;
    let (entity, entity_occurrence) = hazard_entity_occurrence(hazard, source_ordinal)?;
    let local_path = path
        .strip_prefix("$.items[]")
        .or_else(|| path.strip_prefix('$'))
        .unwrap_or(path);
    match local_path {
        "._id" => match &entity.source_identity {
            HazardEntitySourceIdentity::Stable { source_id } => Ok(owner_leaf_value(
                occurrence,
                disposition,
                Value::String(source_id.as_str().to_string()),
            )),
            HazardEntitySourceIdentity::Fallback { .. } => Err(error(
                CoverageFailureCode::CanonicalMismatch,
                "promoted hazard child _id has only fallback identity",
            )),
        },
        ".name" => Ok(owner_leaf_value(
            occurrence,
            disposition,
            Value::String(entity.label.clone()),
        )),
        ".type" => {
            let item_type = match &entity.capability {
                HazardCapability::Action(_) => "action",
                HazardCapability::Strike(_) => "melee",
                HazardCapability::Condition(_) => "condition",
                HazardCapability::Effect(_) => "effect",
                HazardCapability::UnsupportedChild(value) => &value.child_type,
            };
            Ok(owner_leaf_value(
                occurrence,
                disposition,
                Value::String(item_type.to_string()),
            ))
        }
        ".img" => {
            canonical_fact_owner_value(&entity.image, occurrence, disposition, json_owner_value)
        }
        ".folder" => canonical_fact_owner_value(
            &entity_occurrence.source_folder,
            occurrence,
            disposition,
            json_owner_value,
        ),
        ".sort" => canonical_fact_owner_value(
            &entity_occurrence.source_sort,
            occurrence,
            disposition,
            json_owner_value,
        ),
        ".system.description.value" => canonical_rich_owner_value(
            &hazard_capability_common(&entity.capability).description,
            occurrence,
            disposition,
        ),
        ".system.publication.title"
        | ".system.publication.remaster"
        | ".system.publication.license" => {
            let publication = hazard_capability_common(&entity.capability)
                .publication
                .typed()
                .ok_or_else(|| {
                    error(
                        CoverageFailureCode::CanonicalMismatch,
                        "HazardItemCommon.publication is not typed",
                    )
                })?;
            match local_path {
                ".system.publication.title" => canonical_fact_owner_value(
                    &publication.title,
                    occurrence,
                    disposition,
                    json_owner_value,
                ),
                ".system.publication.remaster" => canonical_fact_owner_value(
                    &publication.remaster,
                    occurrence,
                    disposition,
                    json_owner_value,
                ),
                ".system.publication.license" => canonical_fact_owner_value(
                    &publication.license,
                    occurrence,
                    disposition,
                    |value| Ok(Value::String(value.as_str().to_string())),
                ),
                _ => unknown_hazard_accessor("canonical value", local_path),
            }
        }
        ".system.slug" => canonical_fact_owner_value(
            &hazard_capability_common(&entity.capability).slug,
            occurrence,
            disposition,
            json_owner_value,
        ),
        ".system.traits.value[]" => canonical_array_member_fact_owner_value(
            &hazard_capability_common(&entity.capability).traits,
            occurrence,
            disposition,
            |value| Ok(Value::String(value.as_str().to_string())),
        ),
        ".system.traits.rarity" => canonical_fact_owner_value(
            &hazard_capability_common(&entity.capability).rarity,
            occurrence,
            disposition,
            |value| Ok(Value::String(value.as_str().to_string())),
        ),
        "._stats.compendiumSource" => {
            let lineage = hazard_capability_common(&entity.capability)
                .lineage
                .typed()
                .ok_or_else(|| {
                    error(
                        CoverageFailureCode::CanonicalMismatch,
                        "HazardItemCommon.lineage is not typed",
                    )
                })?;
            canonical_fact_owner_value(
                &lineage.compendium_source,
                occurrence,
                disposition,
                json_owner_value,
            )
        }
        ".system.actionType.value"
        | ".system.actions.value"
        | ".system.category"
        | ".system.deathNote"
        | ".system.frequency.value"
        | ".system.frequency.max"
        | ".system.frequency.per"
        | ".system.selfEffect.uuid"
        | ".system.selfEffect.name" => exact_hazard_action_canonical_value(
            &entity.capability,
            local_path,
            occurrence,
            disposition,
        ),
        ".system.bonus.value"
        | ".system.attackEffects.value[]"
        | ".system.attack.value"
        | ".system.weaponType.value"
        | ".system.attackEffects.custom"
        | ".system.damageRolls.*.damage"
        | ".system.damageRolls.*.damageType"
        | ".system.damageRolls.*.category" => exact_hazard_strike_canonical_value(
            &entity.capability,
            local_path,
            occurrence,
            disposition,
        ),
        _ if local_path.starts_with(".system.rules[].") => exact_hazard_rule_canonical_value(
            hazard_capability_common(&entity.capability),
            local_path,
            occurrence,
            disposition,
        ),
        _ => unknown_hazard_accessor("canonical value", path),
    }
}

fn exact_hazard_action_canonical_value(
    capability: &HazardCapability,
    path: &str,
    occurrence: &ExactHazardLeafOccurrence,
    disposition: HazardExactDisposition,
) -> Result<SourcePresence<SourceLeafValue>, CoverageContractError> {
    let HazardCapability::Action(action) = capability else {
        return Err(error(
            CoverageFailureCode::CanonicalMismatch,
            "exact Action leaf did not produce HazardCapability::Action",
        ));
    };
    match path {
        ".system.actionType.value" => {
            canonical_fact_owner_value(&action.action_type, occurrence, disposition, |value| {
                Ok(Value::String(
                    match value {
                        HazardActionType::Action => "action",
                        HazardActionType::Reaction => "reaction",
                        HazardActionType::Free => "free",
                        HazardActionType::Passive => "passive",
                    }
                    .to_string(),
                ))
            })
        }
        ".system.actions.value" => {
            canonical_fact_owner_value(&action.actions, occurrence, disposition, |value| {
                Ok(Value::from(value.value()))
            })
        }
        ".system.category" => {
            canonical_fact_owner_value(&action.category, occurrence, disposition, |value| {
                Ok(Value::String(
                    match value {
                        atlas_record::HazardActionCategory::Interaction => "interaction",
                        atlas_record::HazardActionCategory::Defensive => "defensive",
                        atlas_record::HazardActionCategory::Offensive => "offensive",
                        atlas_record::HazardActionCategory::Familiar => "familiar",
                    }
                    .to_string(),
                ))
            })
        }
        ".system.deathNote" => canonical_fact_owner_value(
            &action.death_note,
            occurrence,
            disposition,
            json_owner_value,
        ),
        ".system.frequency.value" | ".system.frequency.max" | ".system.frequency.per" => {
            let frequency = match &action.frequency.value {
                FactValue::Missing | FactValue::Null => return Ok(SourcePresence::Missing),
                FactValue::Value(CanonicalHazardSourceValue::Typed(value)) => value,
                FactValue::Value(CanonicalHazardSourceValue::Unsupported(_)) => {
                    return Err(error(
                        CoverageFailureCode::CanonicalMismatch,
                        "HazardActionCapability.frequency has no typed nested leaf owner",
                    ));
                }
            };
            if path.ends_with(".value") {
                canonical_fact_owner_value(
                    &frequency.value,
                    occurrence,
                    disposition,
                    json_owner_value,
                )
            } else if path.ends_with(".max") {
                canonical_fact_owner_value(
                    &frequency.maximum,
                    occurrence,
                    disposition,
                    json_owner_value,
                )
            } else {
                canonical_fact_owner_value(
                    &frequency.per,
                    occurrence,
                    disposition,
                    json_owner_value,
                )
            }
        }
        ".system.selfEffect.uuid" | ".system.selfEffect.name" => {
            let self_effect = match &action.self_effect.value {
                FactValue::Missing | FactValue::Null => return Ok(SourcePresence::Missing),
                FactValue::Value(CanonicalHazardSourceValue::Typed(value)) => value,
                FactValue::Value(CanonicalHazardSourceValue::Unsupported(_)) => {
                    return Err(error(
                        CoverageFailureCode::CanonicalMismatch,
                        "HazardActionCapability.self_effect has no typed nested leaf owner",
                    ));
                }
            };
            let fact = if path.ends_with(".uuid") {
                &self_effect.target_uuid
            } else {
                &self_effect.label
            };
            canonical_fact_owner_value(fact, occurrence, disposition, json_owner_value)
        }
        _ => unknown_hazard_accessor("canonical Action value", path),
    }
}

fn exact_hazard_strike_canonical_value(
    capability: &HazardCapability,
    path: &str,
    occurrence: &ExactHazardLeafOccurrence,
    disposition: HazardExactDisposition,
) -> Result<SourcePresence<SourceLeafValue>, CoverageContractError> {
    let HazardCapability::Strike(strike) = capability else {
        return Err(error(
            CoverageFailureCode::CanonicalMismatch,
            "exact Strike leaf did not produce HazardCapability::Strike",
        ));
    };
    match path {
        ".system.bonus.value" => {
            canonical_fact_owner_value(&strike.bonus, occurrence, disposition, json_owner_value)
        }
        ".system.attackEffects.value[]" => canonical_array_member_fact_owner_value(
            &strike.attack_effects,
            occurrence,
            disposition,
            json_owner_value,
        ),
        ".system.attack.value" => canonical_fact_owner_value(
            &strike.source_metadata.attack,
            occurrence,
            disposition,
            json_owner_value,
        ),
        ".system.weaponType.value" => canonical_fact_owner_value(
            &strike.source_metadata.weapon_type,
            occurrence,
            disposition,
            |value| {
                Ok(Value::String(
                    match value {
                        atlas_record::HazardSourceAttackMode::Melee => "melee",
                        atlas_record::HazardSourceAttackMode::Ranged => "ranged",
                    }
                    .to_string(),
                ))
            },
        ),
        ".system.attackEffects.custom" => canonical_fact_owner_value(
            &strike.source_metadata.attack_effects_custom,
            occurrence,
            disposition,
            json_owner_value,
        ),
        _ if path.starts_with(".system.damageRolls.*.") => {
            let damage_rolls = strike.damage_rolls.typed().ok_or_else(|| {
                error(
                    CoverageFailureCode::CanonicalMismatch,
                    "HazardStrikeCapability.damage_rolls is not typed",
                )
            })?;
            let key = pointer_member_after(&occurrence.pointer, "damageRolls")?;
            let damage = damage_rolls
                .iter()
                .find(|damage| damage.source_key == key)
                .ok_or_else(|| {
                    error(
                        CoverageFailureCode::CanonicalMismatch,
                        format!("missing HazardStrikeDamage key {key}"),
                    )
                })?;
            verify_owner_collection(
                occurrence,
                SourceMemberKind::Map,
                Some(&damage.source_key),
                damage.authored_order as usize,
                damage_rolls.len(),
                CoverageFailureCode::CanonicalMismatch,
                "HazardStrikeCapability.damage_rolls",
            )?;
            if path.ends_with(".damage") {
                canonical_fact_owner_value(
                    &damage.damage,
                    occurrence,
                    disposition,
                    json_owner_value,
                )
            } else if path.ends_with(".damageType") {
                canonical_fact_owner_value(
                    &damage.damage_type,
                    occurrence,
                    disposition,
                    json_owner_value,
                )
            } else if path.ends_with(".category") {
                canonical_fact_owner_value(&damage.category, occurrence, disposition, |value| {
                    Ok(Value::String(
                        match value {
                            atlas_record::HazardDamageCategory::Persistent => "persistent",
                            atlas_record::HazardDamageCategory::Precision => "precision",
                            atlas_record::HazardDamageCategory::Splash => "splash",
                        }
                        .to_string(),
                    ))
                })
            } else {
                unknown_hazard_accessor("canonical Strike value", path)
            }
        }
        _ => unknown_hazard_accessor("canonical Strike value", path),
    }
}

fn hazard_entity_occurrence(
    hazard: &HazardRecord,
    source_ordinal: u32,
) -> Result<
    (
        &atlas_record::HazardEntity,
        &atlas_record::HazardEntityOccurrence,
    ),
    CoverageContractError,
> {
    let entities = hazard.embedded_entities.typed().ok_or_else(|| {
        error(
            CoverageFailureCode::CanonicalMismatch,
            "HazardRecord.embedded_entities is not typed",
        )
    })?;
    let occurrence = entities
        .occurrences
        .iter()
        .find(|occurrence| occurrence.source_ordinal == source_ordinal)
        .ok_or_else(|| {
            error(
                CoverageFailureCode::CanonicalMismatch,
                format!("missing hazard occurrence for source ordinal {source_ordinal}"),
            )
        })?;
    let entity = entities
        .entities
        .iter()
        .find(|entity| entity.id == occurrence.entity_id)
        .ok_or_else(|| {
            error(
                CoverageFailureCode::CanonicalMismatch,
                format!("missing hazard entity for source ordinal {source_ordinal}"),
            )
        })?;
    Ok((entity, occurrence))
}

fn verify_canonical_item_collection_if_leaf(
    hazard: &HazardRecord,
    path: &str,
    occurrence: &ExactHazardLeafOccurrence,
) -> Result<(), CoverageContractError> {
    let Some(local_path) = path.strip_prefix("$.items[]") else {
        return Ok(());
    };
    if local_path.contains("[]") || local_path.contains(".*") {
        return Ok(());
    }
    let source_ordinal = pointer_index_after(&occurrence.pointer, "items")? as u32;
    let entities = hazard.embedded_entities.typed().ok_or_else(|| {
        error(
            CoverageFailureCode::CanonicalMismatch,
            "HazardRecord.embedded_entities is not typed",
        )
    })?;
    let owner_occurrence = entities
        .occurrences
        .iter()
        .find(|candidate| candidate.source_ordinal == source_ordinal)
        .ok_or_else(|| {
            error(
                CoverageFailureCode::CanonicalMismatch,
                format!("missing hazard occurrence for source ordinal {source_ordinal}"),
            )
        })?;
    verify_owner_collection(
        occurrence,
        SourceMemberKind::Array,
        None,
        owner_occurrence.source_ordinal as usize,
        entities.occurrences.len(),
        CoverageFailureCode::CanonicalMismatch,
        "HazardRecord.embedded_entities.occurrences",
    )
}

fn verify_canonical_save_collection_if_leaf(
    hazard: &HazardRecord,
    path: &str,
    occurrence: &ExactHazardLeafOccurrence,
) -> Result<(), CoverageContractError> {
    if !path.starts_with("$.system.saves.*.") {
        return Ok(());
    }
    let defenses = hazard.defenses.typed().ok_or_else(|| {
        error(
            CoverageFailureCode::CanonicalMismatch,
            "HazardRecord.defenses is not typed",
        )
    })?;
    let saves = defenses.saves.typed().ok_or_else(|| {
        error(
            CoverageFailureCode::CanonicalMismatch,
            "HazardDefenses.saves is not typed",
        )
    })?;
    let selected = pointer_member_after(&occurrence.pointer, "saves")?;
    let candidates = [
        (
            "fortitude",
            &saves.fortitude,
            &saves.source_metadata.fortitude_detail,
        ),
        (
            "reflex",
            &saves.reflex,
            &saves.source_metadata.reflex_detail,
        ),
        ("will", &saves.will, &saves.source_metadata.will_detail),
    ];
    let present = candidates
        .into_iter()
        .filter(|(_, value, detail)| {
            !matches!(value.value, FactValue::Missing)
                || !matches!(detail.value, FactValue::Missing)
        })
        .collect::<Vec<_>>();
    let authored_order = present
        .iter()
        .position(|(key, _, _)| *key == selected)
        .ok_or_else(|| {
            error(
                CoverageFailureCode::CanonicalMismatch,
                format!("HazardSaves omitted authored key {selected}"),
            )
        })?;
    verify_owner_collection(
        occurrence,
        SourceMemberKind::Map,
        Some(&selected),
        authored_order,
        present.len(),
        CoverageFailureCode::CanonicalMismatch,
        "HazardSaves",
    )
}

fn hazard_capability_common(capability: &HazardCapability) -> &atlas_record::HazardItemCommon {
    match capability {
        HazardCapability::Action(value) => &value.common,
        HazardCapability::Strike(value) => &value.common,
        HazardCapability::Condition(value) => &value.common,
        HazardCapability::Effect(value) => &value.common,
        HazardCapability::UnsupportedChild(value) => &value.common,
    }
}

fn exact_hazard_action_canonical_fingerprint(
    capability: &HazardCapability,
    path: &str,
) -> Result<String, CoverageContractError> {
    let HazardCapability::Action(action) = capability else {
        return Err(error(
            CoverageFailureCode::CanonicalMismatch,
            "exact Action leaf did not produce HazardCapability::Action",
        ));
    };
    let fingerprint = match path {
        ".system.actionType.value" => {
            owner_fingerprint("HazardActionCapability.action_type", &action.action_type)
        }
        ".system.actions.value" => {
            owner_fingerprint("HazardActionCapability.actions", &action.actions)
        }
        ".system.category" => {
            owner_fingerprint("HazardActionCapability.category", &action.category)
        }
        ".system.deathNote" => {
            owner_fingerprint("HazardActionCapability.death_note", &action.death_note)
        }
        ".system.frequency.value" | ".system.frequency.max" | ".system.frequency.per" => {
            let frequency = action.frequency.typed().ok_or_else(|| {
                error(
                    CoverageFailureCode::CanonicalMismatch,
                    "HazardActionCapability.frequency is not typed",
                )
            })?;
            match path {
                ".system.frequency.value" => {
                    owner_fingerprint("HazardFrequency.value", &frequency.value)
                }
                ".system.frequency.max" => {
                    owner_fingerprint("HazardFrequency.maximum", &frequency.maximum)
                }
                ".system.frequency.per" => owner_fingerprint("HazardFrequency.per", &frequency.per),
                _ => return unknown_hazard_accessor("canonical Action", path),
            }
        }
        ".system.selfEffect.uuid" | ".system.selfEffect.name" => {
            owner_fingerprint("HazardActionCapability.self_effect", &action.self_effect)
        }
        _ => return unknown_hazard_accessor("canonical Action", path),
    };
    Ok(fingerprint)
}

fn exact_hazard_strike_canonical_fingerprint(
    capability: &HazardCapability,
    path: &str,
    occurrence: &ExactHazardLeafOccurrence,
) -> Result<String, CoverageContractError> {
    let HazardCapability::Strike(strike) = capability else {
        return Err(error(
            CoverageFailureCode::CanonicalMismatch,
            "exact Strike leaf did not produce HazardCapability::Strike",
        ));
    };
    let fingerprint = match path {
        ".system.bonus.value" => owner_fingerprint("HazardStrikeCapability.bonus", &strike.bonus),
        ".system.attackEffects.value[]" => owner_fingerprint(
            "HazardStrikeCapability.attack_effects",
            &strike.attack_effects,
        ),
        ".system.attack.value" => owner_fingerprint(
            "HazardStrikeSourceMetadata.attack",
            &strike.source_metadata.attack,
        ),
        ".system.weaponType.value" => owner_fingerprint(
            "HazardStrikeSourceMetadata.weapon_type",
            &strike.source_metadata.weapon_type,
        ),
        ".system.attackEffects.custom" => owner_fingerprint(
            "HazardStrikeSourceMetadata.attack_effects_custom",
            &strike.source_metadata.attack_effects_custom,
        ),
        _ if path.starts_with(".system.damageRolls.*.") => {
            let damage_rolls = strike.damage_rolls.typed().ok_or_else(|| {
                error(
                    CoverageFailureCode::CanonicalMismatch,
                    "HazardStrikeCapability.damage_rolls is not typed",
                )
            })?;
            let key = pointer_member_after(&occurrence.pointer, "damageRolls")?;
            let damage = damage_rolls
                .iter()
                .find(|damage| damage.source_key == key)
                .ok_or_else(|| {
                    error(
                        CoverageFailureCode::CanonicalMismatch,
                        format!("missing HazardStrikeDamage key {key}"),
                    )
                })?;
            if path.ends_with(".damage") {
                owner_fingerprint("HazardStrikeDamage.damage", &damage.damage)
            } else if path.ends_with(".damageType") {
                owner_fingerprint("HazardStrikeDamage.damage_type", &damage.damage_type)
            } else if path.ends_with(".category") {
                owner_fingerprint("HazardStrikeDamage.category", &damage.category)
            } else {
                return unknown_hazard_accessor("canonical Strike", path);
            }
        }
        _ => return unknown_hazard_accessor("canonical Strike", path),
    };
    Ok(fingerprint)
}

fn exact_hazard_rule_canonical_fingerprint(
    common: &atlas_record::HazardItemCommon,
    path: &str,
    occurrence: &ExactHazardLeafOccurrence,
) -> Result<String, CoverageContractError> {
    let rules = common.rules.typed().ok_or_else(|| {
        error(
            CoverageFailureCode::CanonicalMismatch,
            "HazardItemCommon.rules is not typed",
        )
    })?;
    let index = pointer_index_after(&occurrence.pointer, "rules")?;
    let rule = rules.get(index).ok_or_else(|| {
        error(
            CoverageFailureCode::CanonicalMismatch,
            format!("missing HazardItemCommon.rules[{index}]"),
        )
    })?;
    let fingerprint = if path.ends_with(".key") {
        owner_fingerprint("HazardRuleElement variant", rule)
    } else {
        match rule {
            atlas_record::HazardRuleElement::Immunity(rule) => {
                if path.ends_with(".mode") {
                    owner_fingerprint("HazardImmunityRule.mode", &rule.mode)
                } else if path.ends_with(".type") || path.ends_with(".type[]") {
                    owner_fingerprint("HazardImmunityRule.immunity_types", &rule.immunity_types)
                } else {
                    return unknown_hazard_accessor("canonical Immunity rule", path);
                }
            }
            atlas_record::HazardRuleElement::ActiveEffectLike(rule) => {
                if path.ends_with(".mode") {
                    owner_fingerprint("HazardActiveEffectLikeRule.mode", &rule.mode)
                } else if path.ends_with(".path") {
                    owner_fingerprint("HazardActiveEffectLikeRule.path", &rule.path)
                } else if path.ends_with(".value") {
                    owner_fingerprint("HazardActiveEffectLikeRule.value", &rule.value)
                } else {
                    return unknown_hazard_accessor("canonical ActiveEffectLike rule", path);
                }
            }
            atlas_record::HazardRuleElement::Aura(rule) => {
                if path.ends_with(".radius") {
                    owner_fingerprint("HazardAuraRule.radius", &rule.radius)
                } else if path.ends_with(".slug") {
                    owner_fingerprint("HazardAuraRule.slug", &rule.slug)
                } else if path.ends_with(".traits[]") {
                    owner_fingerprint("HazardAuraRule.traits", &rule.traits)
                } else {
                    return unknown_hazard_accessor("canonical Aura rule", path);
                }
            }
            atlas_record::HazardRuleElement::DamageDice(rule) => {
                if path.ends_with(".critical") {
                    owner_fingerprint("HazardDamageDiceRule.critical", &rule.critical)
                } else if path.ends_with(".diceNumber") {
                    owner_fingerprint("HazardDamageDiceRule.dice_number", &rule.dice_number)
                } else if path.ends_with(".dieSize") {
                    owner_fingerprint("HazardDamageDiceRule.die_size", &rule.die_size)
                } else if path.ends_with(".damageType") {
                    owner_fingerprint("HazardDamageDiceRule.damage_type", &rule.damage_type)
                } else if path.ends_with(".selector") {
                    owner_fingerprint("HazardDamageDiceRule.selector", &rule.selector)
                } else {
                    return unknown_hazard_accessor("canonical DamageDice rule", path);
                }
            }
            atlas_record::HazardRuleElement::FlatModifier(rule) => {
                if path.ends_with(".critical") {
                    owner_fingerprint("HazardFlatModifierRule.critical", &rule.critical)
                } else if path.ends_with(".damageType") {
                    owner_fingerprint("HazardFlatModifierRule.damage_type", &rule.damage_type)
                } else if path.ends_with(".selector") {
                    owner_fingerprint("HazardFlatModifierRule.selector", &rule.selector)
                } else if path.ends_with(".value") {
                    owner_fingerprint("HazardFlatModifierRule.value", &rule.value)
                } else {
                    return unknown_hazard_accessor("canonical FlatModifier rule", path);
                }
            }
            atlas_record::HazardRuleElement::Note(rule) => {
                if path.ends_with(".outcome[]") {
                    owner_fingerprint("HazardNoteRule.outcomes", &rule.outcomes)
                } else if path.ends_with(".selector") {
                    owner_fingerprint("HazardNoteRule.selector", &rule.selector)
                } else if path.ends_with(".text") {
                    owner_fingerprint("HazardNoteRule.text", &rule.text)
                } else if path.ends_with(".title") {
                    owner_fingerprint("HazardNoteRule.title", &rule.title)
                } else if path.ends_with(".visibility") {
                    owner_fingerprint("HazardNoteRule.visibility", &rule.visibility)
                } else {
                    return unknown_hazard_accessor("canonical Note rule", path);
                }
            }
            atlas_record::HazardRuleElement::Unsupported(rule) => {
                owner_fingerprint("HazardUnsupportedRule.source", &rule.source)
            }
        }
    };
    Ok(fingerprint)
}

fn exact_hazard_rule_canonical_value(
    common: &atlas_record::HazardItemCommon,
    path: &str,
    occurrence: &ExactHazardLeafOccurrence,
    disposition: HazardExactDisposition,
) -> Result<SourcePresence<SourceLeafValue>, CoverageContractError> {
    let rules = common.rules.typed().ok_or_else(|| {
        error(
            CoverageFailureCode::CanonicalMismatch,
            "HazardItemCommon.rules is not typed",
        )
    })?;
    let index = pointer_index_after(&occurrence.pointer, "rules")?;
    let rule = rules.get(index).ok_or_else(|| {
        error(
            CoverageFailureCode::CanonicalMismatch,
            format!("missing HazardItemCommon.rules[{index}]"),
        )
    })?;
    let authored_order = match rule {
        atlas_record::HazardRuleElement::Immunity(value) => value.authored_order,
        atlas_record::HazardRuleElement::ActiveEffectLike(value) => value.authored_order,
        atlas_record::HazardRuleElement::Aura(value) => value.authored_order,
        atlas_record::HazardRuleElement::DamageDice(value) => value.authored_order,
        atlas_record::HazardRuleElement::FlatModifier(value) => value.authored_order,
        atlas_record::HazardRuleElement::Note(value) => value.authored_order,
        atlas_record::HazardRuleElement::Unsupported(value) => value.authored_order,
    };
    if authored_order != index as u32 {
        return Err(error(
            CoverageFailureCode::CanonicalMismatch,
            format!("HazardItemCommon.rules[{index}] lost authored order"),
        ));
    }
    if !path.ends_with("[]") {
        verify_owner_collection(
            occurrence,
            SourceMemberKind::Array,
            None,
            authored_order as usize,
            rules.len(),
            CoverageFailureCode::CanonicalMismatch,
            "HazardItemCommon.rules",
        )?;
    }
    if path.ends_with(".key") {
        let key = match rule {
            atlas_record::HazardRuleElement::Immunity(_) => "Immunity",
            atlas_record::HazardRuleElement::ActiveEffectLike(_) => "ActiveEffectLike",
            atlas_record::HazardRuleElement::Aura(_) => "Aura",
            atlas_record::HazardRuleElement::DamageDice(_) => "DamageDice",
            atlas_record::HazardRuleElement::FlatModifier(_) => "FlatModifier",
            atlas_record::HazardRuleElement::Note(_) => "Note",
            atlas_record::HazardRuleElement::Unsupported(value) => {
                let exact: Value =
                    serde_json::from_str(&value.source.exact_json).map_err(|message| {
                        error(
                            CoverageFailureCode::CanonicalMismatch,
                            format!("invalid unsupported rule owner JSON: {message}"),
                        )
                    })?;
                let key = exact.get("key").cloned().ok_or_else(|| {
                    error(
                        CoverageFailureCode::CanonicalMismatch,
                        "unsupported rule owner omitted its exact key",
                    )
                })?;
                return Ok(owner_leaf_value(occurrence, disposition, key));
            }
        };
        return Ok(owner_leaf_value(
            occurrence,
            disposition,
            Value::String(key.to_string()),
        ));
    }
    match rule {
        atlas_record::HazardRuleElement::Immunity(rule) => {
            if path.ends_with(".mode") {
                canonical_rule_mode_value(&rule.mode, occurrence, disposition)
            } else if path.ends_with(".type") {
                canonical_fact_owner_value(&rule.immunity_types, occurrence, disposition, |value| {
                    match value {
                        atlas_record::HazardRuleType::Single(value) => {
                            Ok(Value::String(value.clone()))
                        }
                        atlas_record::HazardRuleType::Multiple(_) => Err(error(
                            CoverageFailureCode::CanonicalMismatch,
                            "scalar Immunity type produced a multiple-value owner",
                        )),
                    }
                })
            } else if path.ends_with(".type[]") {
                canonical_fact_owner_value(&rule.immunity_types, occurrence, disposition, |value| {
                    match value {
                        atlas_record::HazardRuleType::Multiple(values) => {
                            canonical_array_member(values, occurrence)
                        }
                        atlas_record::HazardRuleType::Single(_) => Err(error(
                            CoverageFailureCode::CanonicalMismatch,
                            "array Immunity type produced a single-value owner",
                        )),
                    }
                })
            } else {
                unknown_hazard_accessor("canonical Immunity value", path)
            }
        }
        atlas_record::HazardRuleElement::ActiveEffectLike(rule) => {
            if path.ends_with(".mode") {
                canonical_rule_mode_value(&rule.mode, occurrence, disposition)
            } else if path.ends_with(".path") {
                canonical_fact_owner_value(&rule.path, occurrence, disposition, json_owner_value)
            } else if path.ends_with(".value") {
                canonical_fact_owner_value(&rule.value, occurrence, disposition, json_owner_value)
            } else {
                unknown_hazard_accessor("canonical ActiveEffectLike value", path)
            }
        }
        atlas_record::HazardRuleElement::Aura(rule) => {
            if path.ends_with(".radius") {
                canonical_fact_owner_value(&rule.radius, occurrence, disposition, json_owner_value)
            } else if path.ends_with(".slug") {
                canonical_fact_owner_value(&rule.slug, occurrence, disposition, json_owner_value)
            } else if path.ends_with(".traits[]") {
                canonical_array_member_fact_owner_value(
                    &rule.traits,
                    occurrence,
                    disposition,
                    |value| Ok(Value::String(value.as_str().to_string())),
                )
            } else {
                unknown_hazard_accessor("canonical Aura value", path)
            }
        }
        atlas_record::HazardRuleElement::DamageDice(rule) => {
            let fact_bool;
            let fact_number;
            let fact_string;
            if path.ends_with(".critical") {
                fact_bool = &rule.critical;
                canonical_fact_owner_value(fact_bool, occurrence, disposition, json_owner_value)
            } else if path.ends_with(".diceNumber") {
                fact_number = &rule.dice_number;
                canonical_fact_owner_value(fact_number, occurrence, disposition, json_owner_value)
            } else {
                fact_string = if path.ends_with(".dieSize") {
                    &rule.die_size
                } else if path.ends_with(".damageType") {
                    &rule.damage_type
                } else if path.ends_with(".selector") {
                    &rule.selector
                } else {
                    return unknown_hazard_accessor("canonical DamageDice value", path);
                };
                canonical_fact_owner_value(fact_string, occurrence, disposition, json_owner_value)
            }
        }
        atlas_record::HazardRuleElement::FlatModifier(rule) => {
            if path.ends_with(".critical") {
                canonical_fact_owner_value(
                    &rule.critical,
                    occurrence,
                    disposition,
                    json_owner_value,
                )
            } else if path.ends_with(".value") {
                canonical_fact_owner_value(&rule.value, occurrence, disposition, json_owner_value)
            } else {
                let fact = if path.ends_with(".damageType") {
                    &rule.damage_type
                } else if path.ends_with(".selector") {
                    &rule.selector
                } else {
                    return unknown_hazard_accessor("canonical FlatModifier value", path);
                };
                canonical_fact_owner_value(fact, occurrence, disposition, json_owner_value)
            }
        }
        atlas_record::HazardRuleElement::Note(rule) => {
            if path.ends_with(".outcome[]") {
                canonical_array_member_fact_owner_value(
                    &rule.outcomes,
                    occurrence,
                    disposition,
                    json_owner_value,
                )
            } else if path.ends_with(".selector") {
                canonical_fact_owner_value(
                    &rule.selector,
                    occurrence,
                    disposition,
                    json_owner_value,
                )
            } else if path.ends_with(".text") {
                canonical_rich_owner_value(&rule.text, occurrence, disposition)
            } else if path.ends_with(".title") {
                canonical_fact_owner_value(&rule.title, occurrence, disposition, json_owner_value)
            } else if path.ends_with(".visibility") {
                canonical_fact_owner_value(
                    &rule.visibility,
                    occurrence,
                    disposition,
                    json_owner_value,
                )
            } else {
                unknown_hazard_accessor("canonical Note value", path)
            }
        }
        atlas_record::HazardRuleElement::Unsupported(_) => Err(error(
            CoverageFailureCode::CanonicalMismatch,
            "promoted rule leaf produced an unsupported rule owner",
        )),
    }
}

fn canonical_rule_mode_value(
    fact: &atlas_record::HazardFact<atlas_record::HazardRuleMode>,
    occurrence: &ExactHazardLeafOccurrence,
    disposition: HazardExactDisposition,
) -> Result<SourcePresence<SourceLeafValue>, CoverageContractError> {
    canonical_fact_owner_value(fact, occurrence, disposition, |value| {
        Ok(Value::String(
            match value {
                atlas_record::HazardRuleMode::Add => "add",
                atlas_record::HazardRuleMode::Remove => "remove",
                atlas_record::HazardRuleMode::Override => "override",
            }
            .to_string(),
        ))
    })
}

fn exact_hazard_unsupported_fingerprint(
    hazard: &HazardRecord,
    pointer: &str,
) -> Result<String, CoverageContractError> {
    Ok(owner_fingerprint(
        "HazardUnsupportedFact",
        exact_hazard_unsupported_fact(hazard, pointer)?,
    ))
}

fn exact_hazard_unsupported_value(
    hazard: &HazardRecord,
    occurrence: &ExactHazardLeafOccurrence,
    disposition: HazardExactDisposition,
) -> Result<SourcePresence<SourceLeafValue>, CoverageContractError> {
    let fact = exact_hazard_unsupported_fact(hazard, &occurrence.pointer)?;
    let value = serde_json::from_str(&fact.value.exact_json).map_err(|message| {
        error(
            CoverageFailureCode::CanonicalMismatch,
            format!(
                "invalid typed unsupported owner JSON at {}: {message}",
                fact.value.relative_source_path
            ),
        )
    })?;
    Ok(owner_leaf_value(occurrence, disposition, value))
}

fn exact_hazard_unsupported_fact<'a>(
    hazard: &'a HazardRecord,
    pointer: &str,
) -> Result<&'a atlas_record::HazardUnsupportedFact, CoverageContractError> {
    let fact = hazard
        .unsupported_facts()
        .into_iter()
        .find(|fact| fact.value.relative_source_path == pointer)
        .ok_or_else(|| {
            error(
                CoverageFailureCode::CanonicalMismatch,
                format!("no typed unsupported hazard owner retains {pointer}"),
            )
        })?;
    Ok(fact)
}

fn pointer_index_after(pointer: &str, key: &str) -> Result<usize, CoverageContractError> {
    pointer_member_after(pointer, key)?
        .parse::<usize>()
        .map_err(|_| {
            error(
                CoverageFailureCode::ReaderNotObserved,
                format!("source pointer {pointer} has no numeric member after {key}"),
            )
        })
}

fn last_pointer_index(pointer: &str) -> Result<usize, CoverageContractError> {
    pointer
        .rsplit('/')
        .next()
        .and_then(|value| value.parse::<usize>().ok())
        .ok_or_else(|| {
            error(
                CoverageFailureCode::ReaderNotObserved,
                format!("source pointer {pointer} has no trailing array ordinal"),
            )
        })
}

fn pointer_member_after(pointer: &str, key: &str) -> Result<String, CoverageContractError> {
    let segments = pointer
        .trim_start_matches('/')
        .split('/')
        .map(|segment| segment.replace("~1", "/").replace("~0", "~"))
        .collect::<Vec<_>>();
    segments
        .windows(2)
        .find(|window| window[0] == key)
        .map(|window| window[1].clone())
        .ok_or_else(|| {
            error(
                CoverageFailureCode::ReaderNotObserved,
                format!("source pointer {pointer} has no member after {key}"),
            )
        })
}

fn unknown_hazard_accessor<T>(stage: &str, path: &str) -> Result<T, CoverageContractError> {
    Err(error(
        CoverageFailureCode::ReaderNotObserved,
        format!("no exact hazard {stage} accessor owns {path}"),
    ))
}

fn mapped_probe_error<T>(owner: &str) -> Result<T, CoverageContractError> {
    Err(error(
        CoverageFailureCode::ReaderNotObserved,
        format!("mapped spell probe reached the dedicated {owner}"),
    ))
}

fn spell_source_destination(probe: SpellLeafProbe) -> Result<&'static str, CoverageContractError> {
    Ok(match probe {
        SpellLeafProbe::Rank => "SpellItemSource.classification.rank",
        SpellLeafProbe::DamageFormula => "SpellItemSource.damage[*].formula",
        SpellLeafProbe::OverlaySort => "SpellItemSource.overlays[*].sort",
        SpellLeafProbe::RuleKey => "SpellItemSource.rules[].authored_key",
        SpellLeafProbe::StandaloneLocation => "SpellItemSource.location_provenance",
        SpellLeafProbe::Image => "SpellItemSource.image",
        SpellLeafProbe::PublicationLicense => "SpellItemSource.publication_license",
        SpellLeafProbe::Mapped => return mapped_probe_error("source destination"),
    })
}

fn spell_canonical_destination(
    probe: SpellLeafProbe,
) -> Result<&'static str, CoverageContractError> {
    Ok(match probe {
        SpellLeafProbe::Rank => "SpellDefinition.classification.rank",
        SpellLeafProbe::DamageFormula => "SpellDefinition.damage[*].formula",
        SpellLeafProbe::OverlaySort => "SpellDefinition.overlays[*].sort",
        SpellLeafProbe::RuleKey => "SpellDefinition.rules[].authored_key",
        SpellLeafProbe::StandaloneLocation => "SpellDefinition.provenance.standalone_location",
        SpellLeafProbe::Image => "SpellDefinition.source_context.image",
        SpellLeafProbe::PublicationLicense => "SpellDefinition.source_context.publication_license",
        SpellLeafProbe::Mapped => return mapped_probe_error("canonical destination"),
    })
}

fn spell_post_destination(probe: SpellLeafProbe) -> Result<&'static str, CoverageContractError> {
    Ok(match probe {
        SpellLeafProbe::Rank => "PostProjection::SpellDefinition.classification.rank",
        SpellLeafProbe::DamageFormula => "PostProjection::SpellDefinition.damage[*].formula",
        SpellLeafProbe::OverlaySort => "PostProjection::SpellDefinition.overlays[*].sort",
        SpellLeafProbe::RuleKey => "PostProjection::SpellDefinition.rules[].authored_key",
        SpellLeafProbe::StandaloneLocation => {
            "PostProjection::SpellDefinition.provenance.standalone_location"
        }
        SpellLeafProbe::Image => "PostProjection::SpellDefinition.source_context.image",
        SpellLeafProbe::PublicationLicense => {
            "PostProjection::SpellDefinition.source_context.publication_license"
        }
        SpellLeafProbe::Mapped => return mapped_probe_error("post-projection destination"),
    })
}

fn consumable_spell_child_source_destination(
    probe: ConsumableSpellChildLeafProbe,
) -> Result<&'static str, CoverageContractError> {
    Ok(match probe {
        ConsumableSpellChildLeafProbe::ChildId => "ConsumableSpellChildSource.spell.id",
        ConsumableSpellChildLeafProbe::StandaloneLocator => {
            "ConsumableSpellChildSource.standalone_locator"
        }
        ConsumableSpellChildLeafProbe::HeightenedRank => {
            "ConsumableSpellChildSource.spell.location.heightened_rank"
        }
        ConsumableSpellChildLeafProbe::LocationValue => {
            "ConsumableSpellChildSource.spell.location.value"
        }
        ConsumableSpellChildLeafProbe::Image => "SpellItemSource.image",
        ConsumableSpellChildLeafProbe::Slug => "SpellItemSource.slug",
        ConsumableSpellChildLeafProbe::PublicationLicense => "SpellItemSource.publication_license",
        ConsumableSpellChildLeafProbe::PublicationRemaster => {
            "SpellItemSource.publication_remaster"
        }
        ConsumableSpellChildLeafProbe::PublicationTitle => "SpellItemSource.publication_title",
        ConsumableSpellChildLeafProbe::Rarity => "SpellItemSource.rarity",
        ConsumableSpellChildLeafProbe::SortProvenance => {
            "AtlasRecord.provenance.raw_json#/system/spell"
        }
        ConsumableSpellChildLeafProbe::Mapped => {
            return mapped_probe_error("consumable spell source destination");
        }
    })
}

fn consumable_spell_child_canonical_destination(
    probe: ConsumableSpellChildLeafProbe,
) -> Result<&'static str, CoverageContractError> {
    Ok(match probe {
        ConsumableSpellChildLeafProbe::ChildId => "ConsumableSpellChild.child_id",
        ConsumableSpellChildLeafProbe::StandaloneLocator => {
            "ConsumableSpellChild.standalone_locator"
        }
        ConsumableSpellChildLeafProbe::HeightenedRank => {
            "ConsumableSpellChild.location.heightened_rank"
        }
        ConsumableSpellChildLeafProbe::LocationValue => "ConsumableSpellChild.location.value",
        ConsumableSpellChildLeafProbe::Image => "SpellDefinition.source_context.image",
        ConsumableSpellChildLeafProbe::PublicationLicense => {
            "SpellDefinition.source_context.publication_license"
        }
        ConsumableSpellChildLeafProbe::Slug => {
            "SpellDefinition.source_context.consumable_child.slug"
        }
        ConsumableSpellChildLeafProbe::PublicationRemaster => {
            "SpellDefinition.source_context.consumable_child.publication_remaster"
        }
        ConsumableSpellChildLeafProbe::PublicationTitle => {
            "SpellDefinition.source_context.consumable_child.publication_title"
        }
        ConsumableSpellChildLeafProbe::Rarity => {
            "SpellDefinition.source_context.consumable_child.rarity"
        }
        ConsumableSpellChildLeafProbe::SortProvenance => {
            "AtlasRecord.provenance.raw_json#/system/spell"
        }
        ConsumableSpellChildLeafProbe::Mapped => {
            return mapped_probe_error("consumable spell canonical destination");
        }
    })
}

fn consumable_spell_child_post_destination(
    probe: ConsumableSpellChildLeafProbe,
) -> Result<&'static str, CoverageContractError> {
    Ok(match probe {
        ConsumableSpellChildLeafProbe::ChildId => "PostProjection::ConsumableSpellChild.child_id",
        ConsumableSpellChildLeafProbe::StandaloneLocator => {
            "PostProjection::ConsumableSpellChild.standalone_locator"
        }
        ConsumableSpellChildLeafProbe::HeightenedRank => {
            "PostProjection::ConsumableSpellChild.location.heightened_rank"
        }
        ConsumableSpellChildLeafProbe::LocationValue => {
            "PostProjection::ConsumableSpellChild.location.value"
        }
        ConsumableSpellChildLeafProbe::Image => {
            "PostProjection::SpellDefinition.source_context.image"
        }
        ConsumableSpellChildLeafProbe::PublicationLicense => {
            "PostProjection::SpellDefinition.source_context.publication_license"
        }
        ConsumableSpellChildLeafProbe::Slug => {
            "PostProjection::SpellDefinition.source_context.consumable_child.slug"
        }
        ConsumableSpellChildLeafProbe::PublicationRemaster => {
            "PostProjection::SpellDefinition.source_context.consumable_child.publication_remaster"
        }
        ConsumableSpellChildLeafProbe::PublicationTitle => {
            "PostProjection::SpellDefinition.source_context.consumable_child.publication_title"
        }
        ConsumableSpellChildLeafProbe::Rarity => {
            "PostProjection::SpellDefinition.source_context.consumable_child.rarity"
        }
        ConsumableSpellChildLeafProbe::SortProvenance => {
            "AtlasRecord.provenance.raw_json#/system/spell"
        }
        ConsumableSpellChildLeafProbe::Mapped => {
            return mapped_probe_error("consumable spell post-projection destination");
        }
    })
}

fn sealed_receipt(
    identity: SourceLeafIdentity,
    fixture: FixtureReference,
    source: SourcePresence<SourceLeafValue>,
    source_mutation: SourcePresence<SourceLeafValue>,
    reader_id: &str,
    accessor_binding: &str,
    observations: Vec<StageObservation>,
) -> Result<SourceLeafReceipt, CoverageContractError> {
    if source == source_mutation {
        return Err(error(
            CoverageFailureCode::ReaderNotObserved,
            "registered adversarial source accessor did not observe its internal mutation",
        ));
    }
    let reader = ActualReadEvidence {
        reader_id: reader_id.to_string(),
        accessor_binding: accessor_binding.to_string(),
        purpose: SourceAccessorPurpose::Parser,
        mutation_digest: digest_serializable(&source_mutation)?,
    };
    let semantic_output = None;
    let evidence_digest = evidence_digest(
        &identity,
        &fixture,
        &source,
        &reader,
        &observations,
        &semantic_output,
    )?;
    Ok(SourceLeafReceipt {
        identity,
        fixture,
        source,
        reader,
        observations,
        semantic_output,
        evidence_digest,
    })
}

impl AbilitySlot {
    const fn canonical_name(self) -> &'static str {
        match self {
            Self::Strength => "strength",
            Self::Dexterity => "dexterity",
            Self::Constitution => "constitution",
            Self::Intelligence => "intelligence",
            Self::Wisdom => "wisdom",
            Self::Charisma => "charisma",
        }
    }

    const fn mod_pointer(self) -> &'static str {
        match self {
            Self::Strength => "/system/abilities/str/mod",
            Self::Dexterity => "/system/abilities/dex/mod",
            Self::Constitution => "/system/abilities/con/mod",
            Self::Intelligence => "/system/abilities/int/mod",
            Self::Wisdom => "/system/abilities/wis/mod",
            Self::Charisma => "/system/abilities/cha/mod",
        }
    }
}

#[derive(Debug, Clone)]
struct HazardPipeline {
    source_dto: VersionedHazardSource,
    canonical: HazardRecord,
    post_projection: HazardRecord,
    hydration: HazardRecord,
    hydration_source_dto: VersionedHazardSource,
    hydration_raw: Value,
}

struct HazardPipelineCore {
    source_dto: VersionedHazardSource,
    canonical: HazardRecord,
    post_projection: HazardRecord,
    input: atlas_index::IndexBuildInput,
    record_key: atlas_domain::RecordKey,
}

static HAZARD_HYDRATION_CACHE: OnceLock<Mutex<BTreeMap<String, HazardRecord>>> = OnceLock::new();
static HAZARD_HYDRATION_LEDGERS: OnceLock<
    Result<Vec<SourceLeafCoverageLedger>, CoverageContractError>,
> = OnceLock::new();
static TEMPORARY_HAZARD_ARTIFACT_SEQUENCE: AtomicU64 = AtomicU64::new(0);

fn hazard_hydration_ledgers() -> Result<&'static [SourceLeafCoverageLedger], CoverageContractError>
{
    HAZARD_HYDRATION_LEDGERS
        .get_or_init(|| {
            HAZARD_HYDRATION_LEDGER_SOURCES
                .iter()
                .map(|source| parse_source_leaf_ledger(source))
                .collect()
        })
        .as_ref()
        .map(Vec::as_slice)
        .map_err(Clone::clone)
}

fn grouped_hazard_hydration_mutation(
    fixture: &ResolvedFixture,
    target_identity: &SourceLeafIdentity,
) -> Result<Value, CoverageContractError> {
    let mut mutations = BTreeMap::<String, Value>::new();
    let isolate_rule_key = target_identity.normalized_path == "$.items[].system.rules[].key";
    let isolate_embedded_type = target_identity.selector.role == SourceDocumentRole::Embedded
        && target_identity.normalized_path == "$.type";
    let isolate_discriminator = isolate_rule_key || isolate_embedded_type;
    for ledger in hazard_hydration_ledgers()? {
        for leaf in &ledger.leaves {
            let identity = ledger.identity_for(leaf);
            let is_rule_key = identity.normalized_path == "$.items[].system.rules[].key";
            let is_embedded_type = identity.selector.role == SourceDocumentRole::Embedded
                && identity.normalized_path == "$.type";
            // These two leaves select the typed owner variant. Mutating either can
            // deliberately make sibling fields inaccessible, so each receives one
            // isolated fixture artifact while ordinary leaves remain grouped.
            if (isolate_discriminator && &identity != target_identity)
                || (!isolate_discriminator && (is_rule_key || is_embedded_type))
            {
                continue;
            }
            let registration = registration_for(&identity, leaf.reader.reader_id.as_deref())?;
            for contract in leaf.fixtures.iter().filter(|contract| {
                contract.source_path == fixture.reference.source_path
                    && contract.source_file_digest == fixture.reference.source_file_digest
            }) {
                let mut reference = fixture.reference.clone();
                reference.record_key.clone_from(&contract.record_key);
                let selected_fixture = ResolvedFixture {
                    reference,
                    serialized: serde_json::to_vec(&fixture.raw).map_err(|message| {
                        error(CoverageFailureCode::InvalidContract, message.to_string())
                    })?,
                    raw: fixture.raw.clone(),
                };
                let Some((pointer, value)) =
                    grouped_hazard_leaf_mutation(registration, &identity, &selected_fixture)?
                else {
                    continue;
                };
                if let Some(previous) = mutations.insert(pointer.clone(), value.clone())
                    && previous != value
                {
                    return Err(error(
                        CoverageFailureCode::ArtifactHydrationMismatch,
                        format!("grouped hazard hydration has conflicting mutations at {pointer}"),
                    ));
                }
            }
        }
    }
    if mutations.is_empty() {
        return Err(error(
            CoverageFailureCode::ArtifactHydrationMismatch,
            format!(
                "ledger has no compatible ArtifactHydration mutations for {}",
                fixture.reference.source_path
            ),
        ));
    }
    let mut grouped = fixture.raw.clone();
    for (pointer, value) in mutations {
        set_json_pointer(&mut grouped, &pointer, value)?;
    }
    Ok(grouped)
}

fn grouped_hazard_leaf_mutation(
    registration: RegisteredAccessor,
    identity: &SourceLeafIdentity,
    fixture: &ResolvedFixture,
) -> Result<Option<(String, Value)>, CoverageContractError> {
    match registration {
        RegisteredAccessor::ActorHazardDisable => {
            let pointer = "/system/details/disable".to_string();
            let value = fixture
                .raw
                .pointer(&pointer)
                .and_then(Value::as_str)
                .ok_or_else(|| {
                    error(
                        CoverageFailureCode::ReaderNotObserved,
                        "missing grouped hazard disable",
                    )
                })?;
            Ok(Some((
                pointer,
                Value::String(format!("{value}\u{241f}source-leaf-mutation")),
            )))
        }
        RegisteredAccessor::ActorHazardTemporaryMaximum => {
            let pointer = "/system/attributes/hp/tempmax".to_string();
            let value = fixture
                .raw
                .pointer(&pointer)
                .and_then(Value::as_i64)
                .ok_or_else(|| {
                    error(
                        CoverageFailureCode::ReaderNotObserved,
                        "missing grouped hazard hp.tempmax",
                    )
                })?;
            Ok(Some((pointer, Value::from(value + 101))))
        }
        RegisteredAccessor::EmbeddedHazardActionType => {
            let item_id = embedded_item_id(fixture)?;
            let item_ordinal = raw_item_ordinal(&fixture.raw, item_id)?;
            let pointer = format!("/items/{item_ordinal}/system/actionType/value");
            let value = fixture
                .raw
                .pointer(&pointer)
                .and_then(Value::as_str)
                .ok_or_else(|| {
                    error(
                        CoverageFailureCode::ReaderNotObserved,
                        "missing grouped embedded hazard action type",
                    )
                })?;
            Ok(Some((
                pointer,
                Value::String(if value == "passive" {
                    "reaction".to_string()
                } else {
                    "passive".to_string()
                }),
            )))
        }
        RegisteredAccessor::EmbeddedHazardDamage => {
            let item_id = embedded_item_id(fixture)?;
            let item_ordinal = raw_item_ordinal(&fixture.raw, item_id)?;
            let damage_map = fixture
                .raw
                .pointer(&format!("/items/{item_ordinal}/system/damageRolls"))
                .and_then(Value::as_object)
                .ok_or_else(|| {
                    error(
                        CoverageFailureCode::ReaderNotObserved,
                        "missing grouped embedded hazard damage map",
                    )
                })?;
            let source_key = damage_map.keys().next().ok_or_else(|| {
                error(
                    CoverageFailureCode::ReaderNotObserved,
                    "grouped embedded hazard damage map is empty",
                )
            })?;
            let pointer = format!(
                "/items/{item_ordinal}/system/damageRolls/{}/damage",
                escape_json_pointer(source_key)
            );
            let value = fixture
                .raw
                .pointer(&pointer)
                .and_then(Value::as_str)
                .ok_or_else(|| {
                    error(
                        CoverageFailureCode::ReaderNotObserved,
                        "missing grouped embedded hazard damage formula",
                    )
                })?;
            Ok(Some((pointer, Value::String(format!("{value}+1")))))
        }
        RegisteredAccessor::ActorHazardExact(HazardExactDisposition::Promoted)
        | RegisteredAccessor::ActorHazardExact(HazardExactDisposition::TypedUnsupported)
            if identity.normalized_path != "$.type"
                || identity.selector.role == SourceDocumentRole::Embedded =>
        {
            let occurrence = first_exact_hazard_leaf(fixture, identity)?;
            let value = occurrence.value.as_ref().map_or_else(
                || Value::String("source-leaf-missing-mutation".to_string()),
                mutate_hazard_leaf_value,
            );
            Ok(Some((occurrence.pointer, value)))
        }
        RegisteredAccessor::ActorHazardFolder
        | RegisteredAccessor::ActorHazardExact(HazardExactDisposition::ProvenanceOnly)
        | RegisteredAccessor::ActorHazardExact(_)
        | RegisteredAccessor::ItemActionName
        | RegisteredAccessor::ActorNpcAbilityMod(_)
        | RegisteredAccessor::ActorNpcShadowSkillBase
        | RegisteredAccessor::ItemSpell(_)
        | RegisteredAccessor::ConsumableSpellChild(_) => Ok(None),
    }
}

fn run_hazard_pipeline(
    fixture: &ResolvedFixture,
    raw: Value,
    _ledger: &SourceLeafCoverageLedger,
    target_identity: &SourceLeafIdentity,
) -> Result<HazardPipeline, CoverageContractError> {
    let baseline = raw == fixture.raw;
    let core = run_hazard_pipeline_core(fixture, raw)?;
    let hydration_raw = (!baseline)
        .then(|| grouped_hazard_hydration_mutation(fixture, target_identity))
        .transpose()?;
    let hydration_core = hydration_raw
        .as_ref()
        .map(|raw| run_hazard_pipeline_core(fixture, raw.clone()))
        .transpose()?;
    let hydration_raw = hydration_raw.as_ref().unwrap_or(&fixture.raw);
    let hydration_source_dto = hydration_core.as_ref().map_or_else(
        || core.source_dto.clone(),
        |grouped| grouped.source_dto.clone(),
    );
    let hydration = if let Some(grouped_core) = hydration_core.as_ref() {
        cached_hazard_hydration(
            fixture,
            hydration_raw,
            &grouped_core.input,
            &grouped_core.record_key,
        )?
    } else {
        cached_hazard_hydration(fixture, hydration_raw, &core.input, &core.record_key)?
    };
    Ok(HazardPipeline {
        source_dto: core.source_dto,
        canonical: core.canonical,
        post_projection: core.post_projection,
        hydration,
        hydration_source_dto,
        hydration_raw: hydration_raw.clone(),
    })
}

fn run_hazard_pipeline_with_baseline_hydration(
    fixture: &ResolvedFixture,
    raw: Value,
) -> Result<HazardPipeline, CoverageContractError> {
    let baseline = raw == fixture.raw;
    let core = run_hazard_pipeline_core(fixture, raw)?;
    let baseline_core = (!baseline)
        .then(|| run_hazard_pipeline_core(fixture, fixture.raw.clone()))
        .transpose()?;
    let hydration_core = baseline_core.as_ref().unwrap_or(&core);
    let hydration_source_dto = hydration_core.source_dto.clone();
    let hydration = cached_hazard_hydration(
        fixture,
        &fixture.raw,
        &hydration_core.input,
        &hydration_core.record_key,
    )?;
    Ok(HazardPipeline {
        source_dto: core.source_dto,
        canonical: core.canonical,
        post_projection: core.post_projection,
        hydration,
        hydration_source_dto,
        hydration_raw: fixture.raw.clone(),
    })
}

#[cfg(test)]
fn run_hazard_pipeline_with_artifact_hook(
    fixture: &ResolvedFixture,
    raw: Value,
    before_manifest: impl FnOnce(&Path) -> Result<bool, CoverageContractError>,
) -> Result<HazardPipeline, CoverageContractError> {
    let core = run_hazard_pipeline_core(fixture, raw.clone())?;
    let hydration = hydrate_hazard_through_sqlite(&core.input, &core.record_key, before_manifest)?;
    Ok(HazardPipeline {
        hydration_source_dto: core.source_dto.clone(),
        hydration_raw: raw,
        source_dto: core.source_dto,
        canonical: core.canonical,
        post_projection: core.post_projection,
        hydration,
    })
}

fn run_hazard_pipeline_core(
    fixture: &ResolvedFixture,
    raw: Value,
) -> Result<HazardPipelineCore, CoverageContractError> {
    let parent_record_key = fixture
        .reference
        .record_key
        .split_once('#')
        .map_or(fixture.reference.record_key.as_str(), |(parent, _)| parent);
    let pack = parent_record_key
        .split_once(':')
        .map(|(pack, _)| pack)
        .ok_or_else(|| {
            error(
                CoverageFailureCode::FixtureNotSourceGrounded,
                "invalid hazard record key",
            )
        })?;
    let pack_name = PackName::new(pack.to_string()).map_err(|message| {
        error(
            CoverageFailureCode::FixtureNotSourceGrounded,
            message.to_string(),
        )
    })?;
    let manifest_pack = ManifestPack {
        name: pack.strip_prefix("pf2e.").unwrap_or(pack).to_string(),
        label: "source-leaf hazard fixture".to_string(),
        document_type: "Actor".to_string(),
        path: fixture
            .reference
            .source_path
            .rsplit_once('/')
            .map_or("packs", |(parent, _)| parent)
            .to_string(),
    };
    let source_dto = parse_hazard_source(
        pinned_source_version_metadata(),
        SourceIdentity::new(
            parent_record_key.to_string(),
            fixture.reference.source_path.clone(),
        ),
        raw.clone(),
        &crate::source::dto::SerializedSourceObject::from_json(&raw).ok_or_else(|| {
            error(
                CoverageFailureCode::ReaderNotObserved,
                "hazard source root is not an object",
            )
        })?,
    )
    .map_err(|message| error(CoverageFailureCode::ReaderNotObserved, message))?;
    let loaded = normalize_record(
        &manifest_pack,
        &pack_name,
        Path::new(&fixture.reference.source_path),
        Path::new("."),
        raw,
        None,
    )
    .map_err(|message| error(CoverageFailureCode::ReaderNotObserved, message))?;
    let canonical = hazard_body(loaded.facts.canonical_body.as_ref())?.clone();
    let source_load = SourceLoad {
        manifest_path: PathBuf::from("static/system.json"),
        source_signature: PF2E_SOURCE_PINNED_SIGNATURE.to_string(),
        source_record_count: 1,
        packs: vec![LoadedPack {
            name: pack_name,
            label: manifest_pack.label,
            document_type: manifest_pack.document_type,
            declared_path: manifest_pack.path.clone(),
            resolved_path: PathBuf::from(manifest_pack.path),
            record_count: 1,
        }],
        records: vec![loaded],
        references: Vec::new(),
        aliases: Vec::new(),
        remaster_links: Vec::new(),
        pending_document_embeddings: Vec::new(),
        document_embeddings: Vec::new(),
        document_embedding_tokenization: Default::default(),
        diagnostics: IngestDiagnostics::default(),
        skipped_records: Vec::new(),
        warnings: Vec::new(),
    };
    let input = index_build_input(source_load);
    let post_projection = hazard_body(input.canonical_bodies.first())?.clone();
    let record_key = input.records[0].identity.key.clone();
    Ok(HazardPipelineCore {
        source_dto,
        canonical,
        post_projection,
        input,
        record_key,
    })
}

fn cached_hazard_hydration(
    fixture: &ResolvedFixture,
    hydration_raw: &Value,
    input: &atlas_index::IndexBuildInput,
    record_key: &atlas_domain::RecordKey,
) -> Result<HazardRecord, CoverageContractError> {
    let cache_key = hazard_hydration_cache_key(fixture, hydration_raw)?;
    let cache = HAZARD_HYDRATION_CACHE.get_or_init(|| Mutex::new(BTreeMap::new()));
    let mut cache = cache.lock().map_err(|_| {
        error(
            CoverageFailureCode::ArtifactHydrationMismatch,
            "hazard receipt hydration cache is poisoned",
        )
    })?;
    if let Some(hydration) = cache.get(&cache_key) {
        return Ok(hydration.clone());
    }
    let hydration = hydrate_hazard_through_sqlite(input, record_key, |_| Ok(false))?;
    cache.insert(cache_key, hydration.clone());
    Ok(hydration)
}

fn hazard_hydration_cache_key(
    fixture: &ResolvedFixture,
    hydration_raw: &Value,
) -> Result<String, CoverageContractError> {
    digest_serializable(&(fixture.reference.source_path.as_str(), hydration_raw))
}

fn hydrate_hazard_through_sqlite(
    input: &atlas_index::IndexBuildInput,
    record_key: &atlas_domain::RecordKey,
    before_manifest: impl FnOnce(&Path) -> Result<bool, CoverageContractError>,
) -> Result<HazardRecord, CoverageContractError> {
    let artifact = TemporaryHazardArtifact::new()?;
    let artifact_path = artifact.artifact_path();
    let receipt = atlas_index::IndexArtifactWriter::write(
        &atlas_index::SqliteIndexWriter::new(artifact_path.clone()),
        input,
        atlas_embedding::EmbeddingModelId::BgeSmallEnV15,
    )
    .map_err(|message| error(CoverageFailureCode::ArtifactHydrationMismatch, message))?;
    let writer_artifact_sha256 = receipt.artifact_sha256().to_string();
    drop(receipt);
    let artifact_changed = before_manifest(&artifact_path)?;
    let artifact_sha256 = if artifact_changed {
        format!(
            "{:x}",
            Sha256::digest(std::fs::read(&artifact_path).map_err(|message| {
                error(CoverageFailureCode::ArtifactHydrationMismatch, message)
            })?)
        )
    } else {
        writer_artifact_sha256
    };
    let generated_record_count = input
        .generated_record_count()
        .map_err(|message| error(CoverageFailureCode::ArtifactHydrationMismatch, message))?;
    let manifest = ArtifactManifest::new(ArtifactManifestInput {
        source_root: PathBuf::from("pinned-source-receipt"),
        source_signature: input.source_signature.clone(),
        source_record_count: input.source_record_count,
        artifact_record_count: input.artifact_record_count(),
        generated_record_count,
        document_embedding_count: input.document_embeddings.len(),
        embedding_model: atlas_embedding::EmbeddingModelId::BgeSmallEnV15
            .as_str()
            .to_string(),
        artifact_sha256,
        source_position: SourcePositionReport {
            git_commit: Some(PF2E_SOURCE_PINNED_COMMIT.to_string()),
            fingerprint: None,
            unavailable_reason: None,
        },
    });
    write_artifact_manifest(&artifact.manifest_path(), &manifest)
        .map_err(|message| error(CoverageFailureCode::ArtifactHydrationMismatch, message))?;
    let reader = atlas_index::SqliteIndexReader::open_read_only(&artifact_path)
        .map_err(|message| error(CoverageFailureCode::ArtifactHydrationMismatch, message))?;
    let hydrated = reader
        .load_hydrated_records_by_key(std::slice::from_ref(record_key))
        .map_err(|message| error(CoverageFailureCode::ArtifactHydrationMismatch, message))?;
    let record = hydrated.first().ok_or_else(|| {
        error(
            CoverageFailureCode::ArtifactHydrationMismatch,
            "manifest-bound hazard read returned no record",
        )
    })?;
    hazard_body(record.body.as_ref()).cloned()
}

struct TemporaryHazardArtifact {
    root: PathBuf,
}

impl TemporaryHazardArtifact {
    fn new() -> Result<Self, CoverageContractError> {
        let sequence = TEMPORARY_HAZARD_ARTIFACT_SEQUENCE.fetch_add(1, Ordering::Relaxed);
        let root = std::env::temp_dir().join(format!(
            "pf2e-atlas-hazard-receipt-{}-{sequence}",
            std::process::id()
        ));
        std::fs::create_dir(&root)
            .map_err(|message| error(CoverageFailureCode::ArtifactHydrationMismatch, message))?;
        Ok(Self { root })
    }

    fn artifact_path(&self) -> PathBuf {
        self.root.join("index.sqlite")
    }

    fn manifest_path(&self) -> PathBuf {
        self.root.join("manifest.json")
    }
}

impl Drop for TemporaryHazardArtifact {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.root);
    }
}

fn hazard_body(body: Option<&RecordBody>) -> Result<&HazardRecord, CoverageContractError> {
    body.and_then(RecordBody::hazard).ok_or_else(|| {
        error(
            CoverageFailureCode::CanonicalMismatch,
            "registered hazard pipeline did not produce a canonical hazard body",
        )
    })
}

fn hazard_lifecycle(
    source: &VersionedHazardSource,
) -> Result<&crate::source::dto::HazardLifecycleSource, CoverageContractError> {
    match &source.source.lifecycle {
        SourcePresence::Value(DtoHazardSourceValue::Typed(value)) => Ok(value),
        _ => Err(error(
            CoverageFailureCode::DtoMismatch,
            "hazard lifecycle is not a typed source object",
        )),
    }
}

fn hazard_hit_points(
    source: &VersionedHazardSource,
) -> Result<&crate::source::dto::HazardHitPointsSource, CoverageContractError> {
    let defenses = match &source.source.defenses {
        SourcePresence::Value(DtoHazardSourceValue::Typed(value)) => value,
        _ => {
            return Err(error(
                CoverageFailureCode::DtoMismatch,
                "hazard defenses are not a typed source object",
            ));
        }
    };
    match &defenses.hit_points {
        SourcePresence::Value(DtoHazardSourceValue::Typed(value)) => Ok(value),
        _ => Err(error(
            CoverageFailureCode::DtoMismatch,
            "hazard hit points are not a typed source object",
        )),
    }
}

fn hazard_item<'a>(
    source: &'a VersionedHazardSource,
    item_id: &str,
) -> Result<&'a crate::source::dto::HazardItemSource, CoverageContractError> {
    let items = match &source.source.items {
        SourcePresence::Value(DtoHazardSourceValue::Typed(items)) => items,
        _ => {
            return Err(error(
                CoverageFailureCode::DtoMismatch,
                "hazard items are not a typed source array",
            ));
        }
    };
    items
        .iter()
        .find(|item| {
            matches!(
                &item.id,
                SourcePresence::Value(DtoHazardSourceValue::Typed(value)) if value == item_id
            )
        })
        .ok_or_else(|| {
            error(
                CoverageFailureCode::DtoMismatch,
                format!("hazard item {item_id} is absent from source DTO"),
            )
        })
}

fn embedded_item_id(fixture: &ResolvedFixture) -> Result<&str, CoverageContractError> {
    fixture
        .reference
        .record_key
        .split_once("#item:")
        .map(|(_, item_id)| item_id)
        .filter(|item_id| !item_id.is_empty())
        .ok_or_else(|| {
            error(
                CoverageFailureCode::FixtureNotSourceGrounded,
                "embedded hazard receipt lacks an item identity",
            )
        })
}

fn raw_item_ordinal(raw: &Value, item_id: &str) -> Result<usize, CoverageContractError> {
    raw.pointer("/items")
        .and_then(Value::as_array)
        .and_then(|items| {
            items
                .iter()
                .position(|item| item.get("_id").and_then(Value::as_str) == Some(item_id))
        })
        .ok_or_else(|| {
            error(
                CoverageFailureCode::FixtureNotSourceGrounded,
                format!("embedded hazard item {item_id} is absent from source"),
            )
        })
}

fn hydration_item_id(
    pipeline: &HazardPipeline,
    authored_ordinal: usize,
) -> Result<&str, CoverageContractError> {
    pipeline
        .hydration_raw
        .pointer(&format!("/items/{authored_ordinal}/_id"))
        .and_then(Value::as_str)
        .ok_or_else(|| {
            error(
                CoverageFailureCode::ArtifactHydrationMismatch,
                format!(
                    "hydration source omitted embedded item identity at authored ordinal {authored_ordinal}"
                ),
            )
        })
}

fn hazard_entity<'a>(
    hazard: &'a HazardRecord,
    item_id: &str,
) -> Result<&'a atlas_record::HazardEntity, CoverageContractError> {
    let entities = hazard.embedded_entities.typed().ok_or_else(|| {
        error(
            CoverageFailureCode::CanonicalMismatch,
            "canonical hazard embedded entities are not typed",
        )
    })?;
    entities
        .entities
        .iter()
        .find(|entity| {
            matches!(
                &entity.source_identity,
                HazardEntitySourceIdentity::Stable { source_id }
                    if source_id.as_str() == item_id
            )
        })
        .ok_or_else(|| {
            error(
                CoverageFailureCode::CanonicalMismatch,
                format!("canonical hazard entity {item_id} is absent"),
            )
        })
}

fn dto_hazard_string(source: &HazardSourceField<String>) -> SourcePresence<SourceLeafValue> {
    match source {
        SourcePresence::Missing => SourcePresence::Missing,
        SourcePresence::Null => SourcePresence::Null,
        SourcePresence::Value(DtoHazardSourceValue::Typed(value)) => string_leaf(value.clone()),
        SourcePresence::Value(DtoHazardSourceValue::Unsupported(value)) => unsupported_scalar_leaf(
            serde_json::from_str(&value.value).unwrap_or(Value::Null),
            SourceJsonType::String,
            "unexpected hazard source string shape",
        ),
    }
}

fn canonical_hazard_string(
    source: &atlas_record::HazardFact<String>,
) -> SourcePresence<SourceLeafValue> {
    match &source.value {
        FactValue::Missing => SourcePresence::Missing,
        FactValue::Null => SourcePresence::Null,
        FactValue::Value(CanonicalHazardSourceValue::Typed(value)) => string_leaf(value.clone()),
        FactValue::Value(CanonicalHazardSourceValue::Unsupported(value)) => {
            unsupported_scalar_leaf(
                serde_json::from_str(&value.exact_json).unwrap_or(Value::Null),
                SourceJsonType::String,
                "unsupported canonical hazard string",
            )
        }
    }
}

fn hazard_rich_document_leaf(
    source_text: &str,
    lifecycle: &atlas_record::HazardFact<atlas_record::HazardLifecycle>,
    relative_path: &str,
) -> Result<SourcePresence<SourceLeafValue>, CoverageContractError> {
    let lifecycle = lifecycle.typed().ok_or_else(|| {
        error(
            CoverageFailureCode::CanonicalMismatch,
            "canonical hazard lifecycle is not typed",
        )
    })?;
    let document = &lifecycle.disable;
    if document.provenance.relative_source_path != relative_path
        || document.typed()
            != Some(&parse_foundry_content_with_localization(source_text, None).document)
    {
        return Err(error(
            CoverageFailureCode::CanonicalMismatch,
            "canonical hazard disable does not match the parsed authored source",
        ));
    }
    Ok(string_leaf(source_text.to_string()))
}

fn dto_hazard_number(source: &HazardSourceField<i64>) -> SourcePresence<SourceLeafValue> {
    match source {
        SourcePresence::Missing => SourcePresence::Missing,
        SourcePresence::Null => SourcePresence::Null,
        SourcePresence::Value(DtoHazardSourceValue::Typed(value)) => number_leaf(*value),
        SourcePresence::Value(DtoHazardSourceValue::Unsupported(value)) => unsupported_scalar_leaf(
            serde_json::from_str(&value.value).unwrap_or(Value::Null),
            SourceJsonType::Number,
            "unexpected hazard source number shape",
        ),
    }
}

fn hazard_tempmax_source_metadata(
    hazard: &HazardRecord,
) -> Result<SourcePresence<SourceLeafValue>, CoverageContractError> {
    let defenses = hazard.defenses.typed().ok_or_else(|| {
        error(
            CoverageFailureCode::CanonicalMismatch,
            "canonical hazard defenses are not typed",
        )
    })?;
    let hit_points = defenses.hit_points.typed().ok_or_else(|| {
        error(
            CoverageFailureCode::CanonicalMismatch,
            "canonical hazard hit points are not typed",
        )
    })?;
    Ok(match &hit_points.source_metadata.temporary_maximum.value {
        FactValue::Missing => SourcePresence::Missing,
        FactValue::Null => SourcePresence::Null,
        FactValue::Value(CanonicalHazardSourceValue::Typed(value)) => number_leaf(*value),
        FactValue::Value(CanonicalHazardSourceValue::Unsupported(value)) => {
            unsupported_scalar_leaf(
                serde_json::from_str(&value.exact_json).unwrap_or(Value::Null),
                SourceJsonType::Number,
                "unexpected canonical hazard hp.tempmax shape",
            )
        }
    })
}

fn hazard_action_type(
    hazard: &HazardRecord,
    item_id: &str,
) -> Result<SourcePresence<SourceLeafValue>, CoverageContractError> {
    let HazardCapability::Action(action) = &hazard_entity(hazard, item_id)?.capability else {
        return Err(error(
            CoverageFailureCode::CanonicalMismatch,
            "selected hazard item is not an Action capability",
        ));
    };
    match &action.action_type.value {
        FactValue::Missing => Ok(SourcePresence::Missing),
        FactValue::Null => Ok(SourcePresence::Null),
        FactValue::Value(CanonicalHazardSourceValue::Typed(value)) => Ok(string_leaf(
            match value {
                HazardActionType::Action => "action",
                HazardActionType::Reaction => "reaction",
                HazardActionType::Free => "free",
                HazardActionType::Passive => "passive",
            }
            .to_string(),
        )),
        FactValue::Value(CanonicalHazardSourceValue::Unsupported(value)) => {
            Ok(unsupported_scalar_leaf(
                serde_json::from_str(&value.exact_json).unwrap_or(Value::Null),
                SourceJsonType::String,
                "unsupported hazard action type",
            ))
        }
    }
}

fn dto_hazard_damage(
    source: &VersionedHazardSource,
    item_id: &str,
    source_key: &str,
) -> Result<SourcePresence<SourceLeafValue>, CoverageContractError> {
    let item = hazard_item(source, item_id)?;
    let damage_rolls = match &item.strike.damage_rolls {
        SourcePresence::Value(DtoHazardSourceValue::Typed(value)) => value,
        _ => {
            return Err(error(
                CoverageFailureCode::DtoMismatch,
                "hazard strike damageRolls are not typed",
            ));
        }
    };
    let damage = damage_rolls
        .iter()
        .find(|damage| damage.source_key == source_key)
        .ok_or_else(|| {
            error(
                CoverageFailureCode::DtoMismatch,
                format!("hazard damageRolls member {source_key} is absent"),
            )
        })?;
    Ok(map_string_from_hazard_field(source_key, &damage.damage))
}

fn hazard_damage(
    hazard: &HazardRecord,
    item_id: &str,
    source_key: &str,
) -> Result<SourcePresence<SourceLeafValue>, CoverageContractError> {
    let HazardCapability::Strike(strike) = &hazard_entity(hazard, item_id)?.capability else {
        return Err(error(
            CoverageFailureCode::CanonicalMismatch,
            "selected hazard item is not a Strike capability",
        ));
    };
    let rolls = strike.damage_rolls.typed().ok_or_else(|| {
        error(
            CoverageFailureCode::CanonicalMismatch,
            "canonical hazard strike damageRolls are not typed",
        )
    })?;
    let damage = rolls
        .iter()
        .find(|damage| damage.source_key == source_key)
        .ok_or_else(|| {
            error(
                CoverageFailureCode::CanonicalMismatch,
                format!("canonical hazard damageRolls member {source_key} is absent"),
            )
        })?;
    Ok(map_string_from_canonical_hazard_fact(
        source_key,
        &damage.damage,
    ))
}

fn map_string_from_hazard_field(
    key: &str,
    source: &HazardSourceField<String>,
) -> SourcePresence<SourceLeafValue> {
    match source {
        SourcePresence::Missing => SourcePresence::Missing,
        SourcePresence::Null => SourcePresence::Null,
        SourcePresence::Value(DtoHazardSourceValue::Typed(value)) => {
            hazard_map_string_leaf(key, value.clone())
        }
        SourcePresence::Value(DtoHazardSourceValue::Unsupported(value)) => unsupported_map_leaf(
            key,
            serde_json::from_str(&value.value).unwrap_or(Value::Null),
            SourceJsonType::String,
        ),
    }
}

fn map_string_from_canonical_hazard_fact(
    key: &str,
    source: &atlas_record::HazardFact<String>,
) -> SourcePresence<SourceLeafValue> {
    match &source.value {
        FactValue::Missing => SourcePresence::Missing,
        FactValue::Null => SourcePresence::Null,
        FactValue::Value(CanonicalHazardSourceValue::Typed(value)) => {
            hazard_map_string_leaf(key, value.clone())
        }
        FactValue::Value(CanonicalHazardSourceValue::Unsupported(value)) => unsupported_map_leaf(
            key,
            serde_json::from_str(&value.exact_json).unwrap_or(Value::Null),
            SourceJsonType::String,
        ),
    }
}

fn validate_hazard_excerpt(
    fixture: &ResolvedFixture,
    value: &Value,
) -> Result<(), CoverageContractError> {
    let bytes = serde_json::to_vec(value).map_err(|message| {
        error(
            CoverageFailureCode::ReceiptProvenanceInvalid,
            format!("failed to serialize hazard source excerpt: {message}"),
        )
    })?;
    if digest_bytes(&bytes) != fixture.reference.excerpt_digest {
        return Err(error(
            CoverageFailureCode::ReceiptProvenanceInvalid,
            "registered hazard excerpt does not match its declaration digest",
        ));
    }
    Ok(())
}

#[derive(Debug)]
struct NpcPipeline {
    source_dto: VersionedNpcSource,
    canonical: CreatureRecord,
    post_projection: CreatureRecord,
    hydration: CreatureRecord,
    public_surface: RecordJson,
    #[cfg(test)]
    diagnostic_source_fields: Vec<String>,
    #[cfg(test)]
    persisted_record: atlas_record::AtlasRecord,
    #[cfg(test)]
    persisted_pack: atlas_index::IndexBuildPack,
}

fn run_npc_pipeline(
    fixture: &ResolvedFixture,
    raw: Value,
) -> Result<NpcPipeline, CoverageContractError> {
    let pack = fixture
        .reference
        .record_key
        .split_once(':')
        .map(|(pack, _)| pack)
        .ok_or_else(|| {
            error(
                CoverageFailureCode::FixtureNotSourceGrounded,
                "invalid record key",
            )
        })?;
    let pack_name = PackName::new(pack.to_string()).map_err(|message| {
        error(
            CoverageFailureCode::FixtureNotSourceGrounded,
            message.to_string(),
        )
    })?;
    let manifest_pack = ManifestPack {
        name: pack.strip_prefix("pf2e.").unwrap_or(pack).to_string(),
        label: "source-leaf fixture".to_string(),
        document_type: "Actor".to_string(),
        path: fixture
            .reference
            .source_path
            .rsplit_once('/')
            .map_or("packs", |(parent, _)| parent)
            .to_string(),
    };
    let source_dto = parse_npc_source(
        pinned_source_version_metadata(),
        SourceIdentity::new(
            fixture.reference.record_key.clone(),
            fixture.reference.source_path.clone(),
        ),
        raw.clone(),
    )
    .map_err(|message| error(CoverageFailureCode::ReaderNotObserved, message))?;
    let loaded = normalize_record(
        &manifest_pack,
        &pack_name,
        Path::new(&fixture.reference.source_path),
        Path::new("."),
        raw,
        None,
    )
    .map_err(|message| error(CoverageFailureCode::ReaderNotObserved, message))?;
    let canonical = creature_body(loaded.facts.canonical_body.as_ref())?.clone();
    #[cfg(test)]
    let diagnostic_source_fields = loaded
        .facts
        .npc_core_diagnostics
        .iter()
        .map(|diagnostic| diagnostic.source_field.clone())
        .collect();
    let source_load = SourceLoad {
        manifest_path: PathBuf::from("static/system.json"),
        source_signature: PF2E_SOURCE_PINNED_SIGNATURE.to_string(),
        source_record_count: 1,
        packs: vec![LoadedPack {
            name: pack_name,
            label: manifest_pack.label,
            document_type: manifest_pack.document_type,
            declared_path: manifest_pack.path.clone(),
            resolved_path: PathBuf::from(manifest_pack.path),
            record_count: 1,
        }],
        records: vec![loaded],
        references: Vec::new(),
        aliases: Vec::new(),
        remaster_links: Vec::new(),
        pending_document_embeddings: Vec::new(),
        document_embeddings: Vec::new(),
        document_embedding_tokenization: Default::default(),
        diagnostics: IngestDiagnostics::default(),
        skipped_records: Vec::new(),
        warnings: Vec::new(),
    };
    let input = index_build_input(source_load);
    let post_projection = creature_body(input.canonical_bodies.first())?.clone();
    #[cfg(test)]
    let persisted_record = input.records.first().cloned().ok_or_else(|| {
        error(
            CoverageFailureCode::PostProjectionMismatch,
            "NPC pipeline produced no post-IndexBuildInput record",
        )
    })?;
    #[cfg(test)]
    let persisted_pack = input.packs.first().cloned().ok_or_else(|| {
        error(
            CoverageFailureCode::PostProjectionMismatch,
            "NPC pipeline produced no post-IndexBuildInput pack",
        )
    })?;
    let bodies = input
        .canonical_bodies
        .into_iter()
        .map(|body| (body.record_key().clone(), body))
        .collect::<BTreeMap<_, _>>();
    let hydrated = atlas_index::hydrate_record_parts(input.records, bodies, BTreeMap::new())
        .map_err(|message| error(CoverageFailureCode::ArtifactHydrationMismatch, message))?;
    let hydration = creature_body(hydrated[0].body.as_ref())?.clone();
    let public_surface = record_json(
        &hydrated[0],
        RecordJsonOptions {
            detail: DetailLevel::Full,
            include_source_json: true,
        },
    )
    .map_err(|message| error(CoverageFailureCode::PublicSurfaceMismatch, message))?;
    Ok(NpcPipeline {
        source_dto,
        canonical,
        post_projection,
        hydration,
        public_surface,
        #[cfg(test)]
        diagnostic_source_fields,
        #[cfg(test)]
        persisted_record,
        #[cfg(test)]
        persisted_pack,
    })
}

fn creature_body(body: Option<&RecordBody>) -> Result<&CreatureRecord, CoverageContractError> {
    match body {
        Some(RecordBody::Creature(creature)) => Ok(creature),
        Some(RecordBody::Hazard(_)) => Err(error(
            CoverageFailureCode::CanonicalMismatch,
            "NPC pipeline produced a hazard body",
        )),
        Some(RecordBody::Spell(_)) => Err(error(
            CoverageFailureCode::CanonicalMismatch,
            "NPC pipeline produced a non-creature canonical body",
        )),
        None => Err(error(
            CoverageFailureCode::CanonicalMismatch,
            "NPC pipeline produced no canonical creature body",
        )),
    }
}

fn validate_actor_excerpt(fixture: &ResolvedFixture) -> Result<(), CoverageContractError> {
    let excerpt = actor_excerpt(&fixture.raw)?;
    if digest_serializable(&excerpt)? != fixture.reference.excerpt_digest {
        return Err(error(
            CoverageFailureCode::ReceiptProvenanceInvalid,
            "registered Actor NPC excerpt does not match its declaration digest",
        ));
    }
    Ok(())
}

fn actor_excerpt(raw: &Value) -> Result<Value, CoverageContractError> {
    let required = |pointer: &str| {
        raw.pointer(pointer).cloned().ok_or_else(|| {
            error(
                CoverageFailureCode::FixtureNotSourceGrounded,
                format!("pinned Actor NPC fixture lacks {pointer}"),
            )
        })
    };
    Ok(serde_json::json!({
        "_id": required("/_id")?,
        "name": required("/name")?,
        "type": required("/type")?,
        "system": {
            "abilities": required("/system/abilities")?,
            "skills": required("/system/skills")?,
        }
    }))
}

fn ability_mod_from_raw(
    raw: &Value,
    slot: AbilitySlot,
) -> Result<SourcePresence<SourceLeafValue>, CoverageContractError> {
    match raw.pointer(slot.mod_pointer()) {
        Some(Value::Null) => Ok(SourcePresence::Null),
        Some(Value::Number(number)) => number.as_i64().map(number_leaf).ok_or_else(|| {
            error(
                CoverageFailureCode::ReaderNotObserved,
                format!("{} is not an integer", slot.mod_pointer()),
            )
        }),
        Some(other) => Err(error(
            CoverageFailureCode::ReaderNotObserved,
            format!(
                "{} has unsupported source shape {other:?}",
                slot.mod_pointer()
            ),
        )),
        None => Ok(SourcePresence::Missing),
    }
}

fn dto_ability_value(
    source: &VersionedNpcSource,
    slot: AbilitySlot,
) -> SourcePresence<SourceLeafValue> {
    let abilities = match &source.source.core.abilities {
        SourcePresence::Missing => return SourcePresence::Missing,
        SourcePresence::Null => return SourcePresence::Null,
        SourcePresence::Value(abilities) => abilities,
    };
    let ability = match slot {
        AbilitySlot::Strength => &abilities.strength,
        AbilitySlot::Dexterity => &abilities.dexterity,
        AbilitySlot::Constitution => &abilities.constitution,
        AbilitySlot::Intelligence => &abilities.intelligence,
        AbilitySlot::Wisdom => &abilities.wisdom,
        AbilitySlot::Charisma => &abilities.charisma,
    };
    match ability {
        SourcePresence::Missing => SourcePresence::Missing,
        SourcePresence::Null => SourcePresence::Null,
        SourcePresence::Value(ability) => source_number(&ability.r#mod),
    }
}

fn creature_ability_value(
    creature: &CreatureRecord,
    slot: AbilitySlot,
) -> SourcePresence<SourceLeafValue> {
    let FactValue::Value(abilities) = &creature.legacy_abilities.value else {
        return fact_presence(&creature.legacy_abilities.value);
    };
    let ability = match slot {
        AbilitySlot::Strength => &abilities.strength,
        AbilitySlot::Dexterity => &abilities.dexterity,
        AbilitySlot::Constitution => &abilities.constitution,
        AbilitySlot::Intelligence => &abilities.intelligence,
        AbilitySlot::Wisdom => &abilities.wisdom,
        AbilitySlot::Charisma => &abilities.charisma,
    };
    fact_number(ability)
}

fn unsupported_skill_key(raw: &Value) -> Result<String, CoverageContractError> {
    let skills = raw
        .pointer("/system/skills")
        .and_then(Value::as_object)
        .ok_or_else(|| {
            error(
                CoverageFailureCode::FixtureNotSourceGrounded,
                "pinned Actor NPC fixture has no skills map",
            )
        })?;
    let keys = skills
        .keys()
        .filter(|key| CreatureSkillKind::from_source_slug(key).is_none())
        .cloned()
        .collect::<Vec<_>>();
    if keys.len() != 1 {
        return Err(error(
            CoverageFailureCode::FixtureNotSourceGrounded,
            format!(
                "adversarial shadow-skill fixture requires exactly one unsupported key, found {keys:?}"
            ),
        ));
    }
    Ok(keys[0].clone())
}

fn shadow_skill_from_raw(
    raw: &Value,
    key: &str,
) -> Result<SourcePresence<SourceLeafValue>, CoverageContractError> {
    let pointer = format!("/system/skills/{}/base", escape_json_pointer(key));
    match raw.pointer(&pointer) {
        Some(Value::Null) => Ok(unsupported_map_leaf(key, Value::Null, SourceJsonType::Null)),
        Some(Value::Number(number)) => number
            .as_i64()
            .map(|value| unsupported_map_leaf(key, Value::from(value), SourceJsonType::Number))
            .ok_or_else(|| {
                error(
                    CoverageFailureCode::ReaderNotObserved,
                    format!("shadow skill {key:?} base is not an integer"),
                )
            }),
        Some(other) => Err(error(
            CoverageFailureCode::ReaderNotObserved,
            format!("shadow skill {key:?} has unsupported base {other:?}"),
        )),
        None => Err(error(
            CoverageFailureCode::ReaderNotObserved,
            format!("shadow skill {key:?} has no authored base member"),
        )),
    }
}

fn dto_shadow_skill_value(
    source: &VersionedNpcSource,
    key: &str,
) -> SourcePresence<SourceLeafValue> {
    let skills = match &source.source.core.skills {
        SourcePresence::Missing => return SourcePresence::Missing,
        SourcePresence::Null => return SourcePresence::Null,
        SourcePresence::Value(skills) => skills,
    };
    let Some(skill) = skills.get(key) else {
        return SourcePresence::Missing;
    };
    match &skill.base {
        SourcePresence::Value(value) => {
            unsupported_map_leaf(key, Value::from(*value), SourceJsonType::Number)
        }
        SourcePresence::Null => unsupported_map_leaf(key, Value::Null, SourceJsonType::Null),
        SourcePresence::Missing => SourcePresence::Missing,
    }
}

fn creature_skill_value(creature: &CreatureRecord, key: &str) -> SourcePresence<SourceLeafValue> {
    let FactValue::Value(skills) = &creature.skills.value else {
        return fact_presence(&creature.skills.value);
    };
    let Some(entry) = skills
        .iter()
        .flat_map(|skill| &skill.source_entries)
        .find(|entry| entry.authored_key == key)
    else {
        return SourcePresence::Missing;
    };
    skill_fact_leaf(key, &entry.modifier)
}

fn public_skill_value(record: &RecordJson, key: &str) -> SourcePresence<SourceLeafValue> {
    let RecordPresentationJson::Creature { skills, .. } = &record.presentation else {
        return SourcePresence::Missing;
    };
    let Some((skill, source_entry)) = skills.as_ref().and_then(|skills| {
        skills.iter().find_map(|skill| {
            skill
                .source_entries
                .iter()
                .find(|entry| entry.authored_key == key)
                .map(|entry| (skill, entry))
        })
    }) else {
        return SourcePresence::Missing;
    };
    if skill.unmodeled.is_some() {
        return match source_entry.modifier {
            atlas_record::CreatureIntegerPresenceJson::Missing => SourcePresence::Missing,
            atlas_record::CreatureIntegerPresenceJson::Null => {
                unsupported_map_leaf(key, Value::Null, SourceJsonType::Null)
            }
            atlas_record::CreatureIntegerPresenceJson::Value(value) => {
                unsupported_map_leaf(key, Value::from(value), SourceJsonType::Number)
            }
        };
    }
    match source_entry.modifier {
        atlas_record::CreatureIntegerPresenceJson::Value(value)
            if CreatureSkillKind::from_source_slug(key).is_none() =>
        {
            unsupported_map_leaf(key, Value::from(value), SourceJsonType::Number)
        }
        atlas_record::CreatureIntegerPresenceJson::Value(value) => map_number_leaf(key, value),
        atlas_record::CreatureIntegerPresenceJson::Null => SourcePresence::Null,
        atlas_record::CreatureIntegerPresenceJson::Missing => SourcePresence::Missing,
    }
}

fn skill_fact_leaf(key: &str, source: &FactValue<i64>) -> SourcePresence<SourceLeafValue> {
    let noncanonical_key = CreatureSkillKind::from_source_slug(key).is_none();
    match source {
        FactValue::Value(value) if noncanonical_key => {
            unsupported_map_leaf(key, Value::from(*value), SourceJsonType::Number)
        }
        FactValue::Null if noncanonical_key => {
            unsupported_map_leaf(key, Value::Null, SourceJsonType::Null)
        }
        FactValue::Value(value) => map_number_leaf(key, *value),
        FactValue::Null => SourcePresence::Null,
        FactValue::Missing => SourcePresence::Missing,
    }
}

fn escape_json_pointer(value: &str) -> String {
    value.replace('~', "~0").replace('/', "~1")
}

fn source_number(source: &SourcePresence<i64>) -> SourcePresence<SourceLeafValue> {
    match source {
        SourcePresence::Missing => SourcePresence::Missing,
        SourcePresence::Null => SourcePresence::Null,
        SourcePresence::Value(value) => number_leaf(*value),
    }
}

fn fact_number(source: &FactValue<i64>) -> SourcePresence<SourceLeafValue> {
    match source {
        FactValue::Missing => SourcePresence::Missing,
        FactValue::Null => SourcePresence::Null,
        FactValue::Value(value) => number_leaf(*value),
    }
}

fn fact_presence<T>(source: &FactValue<T>) -> SourcePresence<SourceLeafValue> {
    match source {
        FactValue::Missing => SourcePresence::Missing,
        FactValue::Null => SourcePresence::Null,
        FactValue::Value(_) => SourcePresence::Missing,
    }
}

fn number_leaf(value: i64) -> SourcePresence<SourceLeafValue> {
    SourcePresence::Value(SourceLeafValue {
        json_type: SourceJsonType::Number,
        value: Some(Value::from(value)),
        stable_digest: None,
        member_kind: None,
        member_identity: None,
        ordinal: None,
        multiplicity: 1,
        unsupported: None,
    })
}

fn map_number_leaf(key: &str, value: i64) -> SourcePresence<SourceLeafValue> {
    SourcePresence::Value(SourceLeafValue {
        json_type: SourceJsonType::Number,
        value: Some(Value::from(value)),
        stable_digest: None,
        member_kind: Some(SourceMemberKind::Map),
        member_identity: Some(key.to_string()),
        ordinal: None,
        multiplicity: 1,
        unsupported: None,
    })
}

fn hazard_map_string_leaf(key: &str, value: String) -> SourcePresence<SourceLeafValue> {
    SourcePresence::Value(SourceLeafValue {
        json_type: SourceJsonType::String,
        value: Some(Value::String(value)),
        stable_digest: None,
        member_kind: Some(SourceMemberKind::Map),
        member_identity: Some(key.to_string()),
        ordinal: None,
        multiplicity: 1,
        unsupported: None,
    })
}

fn unsupported_scalar_leaf(
    value: Value,
    json_type: SourceJsonType,
    reason: &str,
) -> SourcePresence<SourceLeafValue> {
    SourcePresence::Value(SourceLeafValue {
        json_type,
        value: Some(value.clone()),
        stable_digest: None,
        member_kind: None,
        member_identity: None,
        ordinal: None,
        multiplicity: 1,
        unsupported: Some(TypedUnsupportedValue {
            value,
            reason: reason.to_string(),
        }),
    })
}

fn unsupported_map_leaf(
    key: &str,
    value: Value,
    json_type: SourceJsonType,
) -> SourcePresence<SourceLeafValue> {
    SourcePresence::Value(SourceLeafValue {
        json_type,
        value: Some(value.clone()),
        stable_digest: None,
        member_kind: Some(SourceMemberKind::Map),
        member_identity: Some(key.to_string()),
        ordinal: None,
        multiplicity: 1,
        unsupported: Some(TypedUnsupportedValue {
            value,
            reason: "authored noncanonical skill-map key".to_string(),
        }),
    })
}

fn presence_stage(
    stage: FinalOwnerStage,
    destination: &str,
    accessor_binding: &str,
    value: SourcePresence<SourceLeafValue>,
    mutation: &SourcePresence<SourceLeafValue>,
) -> Result<StageObservation, CoverageContractError> {
    if &value == mutation {
        return Err(error(
            stage.mismatch_code(),
            format!(
                "{destination} did not observe the selected source mutation through {accessor_binding}"
            ),
        ));
    }
    Ok(StageObservation {
        stage,
        destination: destination.to_string(),
        accessor_binding: accessor_binding.to_string(),
        mutation_digest: digest_serializable(mutation)?,
        value,
    })
}

fn owner_presence_stage(
    stage: FinalOwnerStage,
    destination: &str,
    accessor_binding: &str,
    source: &SourcePresence<SourceLeafValue>,
    mutation: &SourcePresence<SourceLeafValue>,
    owner_values: (
        SourcePresence<SourceLeafValue>,
        Option<SourcePresence<SourceLeafValue>>,
    ),
    owner_fingerprints: (&str, &str),
) -> Result<StageObservation, CoverageContractError> {
    let (owner_value, owner_mutation_value) = owner_values;
    let (owner_fingerprint, owner_mutation_fingerprint) = owner_fingerprints;
    if source != &owner_value {
        return Err(error(
            stage.mismatch_code(),
            format!(
                "exact accessor {accessor_binding} owner meaning does not equal authenticated source meaning"
            ),
        ));
    }
    if let Some(owner_mutation_value) = &owner_mutation_value {
        if mutation != owner_mutation_value {
            return Err(error(
                stage.mismatch_code(),
                format!(
                    "exact accessor {accessor_binding} mutated owner meaning does not equal authenticated mutated source meaning: source={mutation:?}; owner={owner_mutation_value:?}"
                ),
            ));
        }
        if &owner_value == owner_mutation_value {
            return Err(error(
                CoverageFailureCode::ReaderNotObserved,
                format!("exact accessor {accessor_binding} did not observe its owner mutation"),
            ));
        }
    } else if owner_fingerprint == owner_mutation_fingerprint {
        return Err(error(
            CoverageFailureCode::ReaderNotObserved,
            format!("exact accessor {accessor_binding} did not observe its owner mutation"),
        ));
    }
    Ok(StageObservation {
        stage,
        destination: destination.to_string(),
        accessor_binding: format!("{accessor_binding} [{owner_fingerprint}]"),
        mutation_digest: digest_serializable(&(
            mutation,
            owner_mutation_value,
            owner_mutation_fingerprint,
        ))?,
        value: owner_value,
    })
}

fn owner_presence_or_rejected_stage(
    stage: FinalOwnerStage,
    destination: &str,
    accessor_binding: &str,
    source_values: (
        &SourcePresence<SourceLeafValue>,
        &SourcePresence<SourceLeafValue>,
    ),
    owner_values: (
        SourcePresence<SourceLeafValue>,
        Option<SourcePresence<SourceLeafValue>>,
    ),
    owner_fingerprints: (&str, &str),
    mutation_rejection: Option<&CoverageContractError>,
) -> Result<StageObservation, CoverageContractError> {
    let (source, mutation) = source_values;
    if owner_values.1.is_some() {
        return owner_presence_stage(
            stage,
            destination,
            accessor_binding,
            source,
            mutation,
            owner_values,
            owner_fingerprints,
        );
    }
    let rejection = mutation_rejection.ok_or_else(|| {
        error(
            CoverageFailureCode::ReaderNotObserved,
            format!("exact accessor {accessor_binding} has no mutation owner or rejection"),
        )
    })?;
    let (owner_value, _) = owner_values;
    if source != &owner_value {
        return Err(error(
            stage.mismatch_code(),
            format!(
                "exact accessor {accessor_binding} baseline owner meaning does not equal authenticated source meaning"
            ),
        ));
    }
    Ok(StageObservation {
        stage,
        destination: destination.to_string(),
        accessor_binding: format!(
            "{accessor_binding} [{}; mutation rejected before this owner stage: {:?}]",
            owner_fingerprints.0, rejection.code
        ),
        mutation_digest: digest_serializable(&(
            mutation,
            "mutation_rejected_before_owner_stage",
            rejection.code,
            rejection.message.as_str(),
        ))?,
        value: owner_value,
    })
}

#[derive(Debug, PartialEq, Eq)]
struct ItemNamePipeline {
    source_dto: String,
    canonical: String,
    post_projection: String,
    hydration: String,
    public_surface: String,
}

fn run_item_name_pipeline(
    fixture: &ResolvedFixture,
    raw: Value,
) -> Result<ItemNamePipeline, CoverageContractError> {
    let pack = fixture
        .reference
        .record_key
        .split_once(':')
        .map(|(pack, _)| pack)
        .ok_or_else(|| {
            error(
                CoverageFailureCode::FixtureNotSourceGrounded,
                "invalid record key",
            )
        })?;
    let pack_name = PackName::new(pack.to_string()).map_err(|message| {
        error(
            CoverageFailureCode::FixtureNotSourceGrounded,
            message.to_string(),
        )
    })?;
    let manifest_pack = ManifestPack {
        name: pack.strip_prefix("pf2e.").unwrap_or(pack).to_string(),
        label: "source-leaf fixture".to_string(),
        document_type: "Item".to_string(),
        path: fixture
            .reference
            .source_path
            .rsplit_once('/')
            .map_or("packs", |(parent, _)| parent)
            .to_string(),
    };
    let source_dto = parse_item_source(
        pinned_source_version_metadata(),
        SourceIdentity::new(
            fixture.reference.record_key.clone(),
            fixture.reference.source_path.clone(),
        ),
        None,
        raw.clone(),
    )
    .map_err(|message| error(CoverageFailureCode::ReaderNotObserved, message))?
    .source
    .source()
    .name
    .clone();
    let loaded = normalize_record(
        &manifest_pack,
        &pack_name,
        Path::new(&fixture.reference.source_path),
        Path::new("."),
        raw,
        None,
    )
    .map_err(|message| error(CoverageFailureCode::ReaderNotObserved, message))?;
    let canonical = loaded.record.identity.name.clone();
    let source_load = SourceLoad {
        manifest_path: PathBuf::from("static/system.json"),
        source_signature: PF2E_SOURCE_PINNED_SIGNATURE.to_string(),
        source_record_count: 1,
        packs: vec![LoadedPack {
            name: pack_name,
            label: manifest_pack.label,
            document_type: manifest_pack.document_type,
            declared_path: manifest_pack.path.clone(),
            resolved_path: PathBuf::from(manifest_pack.path),
            record_count: 1,
        }],
        records: vec![loaded],
        references: Vec::new(),
        aliases: Vec::new(),
        remaster_links: Vec::new(),
        pending_document_embeddings: Vec::new(),
        document_embeddings: Vec::new(),
        document_embedding_tokenization: Default::default(),
        diagnostics: IngestDiagnostics::default(),
        skipped_records: Vec::new(),
        warnings: Vec::new(),
    };
    let input = index_build_input(source_load);
    let post_projection = input.records[0].identity.name.clone();
    let bodies = input
        .canonical_bodies
        .into_iter()
        .map(|body| (body.record_key().clone(), body))
        .collect::<BTreeMap<_, _>>();
    let hydrated = atlas_index::hydrate_record_parts(input.records, bodies, BTreeMap::new())
        .map_err(|message| error(CoverageFailureCode::ArtifactHydrationMismatch, message))?;
    let hydration = hydrated[0].record.identity.name.clone();
    let public = record_json(
        &hydrated[0],
        RecordJsonOptions {
            detail: DetailLevel::Full,
            include_source_json: true,
        },
    )
    .map_err(|message| error(CoverageFailureCode::PublicSurfaceMismatch, message))?;
    Ok(ItemNamePipeline {
        source_dto,
        canonical,
        post_projection,
        hydration,
        public_surface: public.name.clone(),
    })
}

#[derive(Debug)]
struct ResolvedFixture {
    reference: FixtureReference,
    raw: Value,
    serialized: Vec<u8>,
}

#[derive(Debug, Deserialize)]
struct PinnedSystemManifest {
    packs: Vec<PinnedManifestPack>,
}

#[derive(Debug, Deserialize)]
struct PinnedManifestPack {
    name: String,
    path: String,
    #[serde(rename = "type")]
    document_class: String,
}

impl ResolvedFixture {
    fn load(
        contract: &FixtureContract,
        source_repository: &Path,
        expected_selector: &SourceLeafSelector,
    ) -> Result<Self, CoverageContractError> {
        if contract.provenance != FixtureProvenance::PinnedSource
            || Path::new(&contract.source_path).is_absolute()
            || contract
                .source_path
                .split('/')
                .any(|segment| segment == "..")
        {
            return Err(error(
                CoverageFailureCode::ReceiptProvenanceInvalid,
                "registered receipts require a safe pinned-source relative path",
            ));
        }
        git_success(
            source_repository,
            &[
                "cat-file",
                "-e",
                &format!("{PF2E_SOURCE_PINNED_COMMIT}^{{commit}}"),
            ],
        )?;
        let bytes = git_output(
            source_repository,
            &[
                "show",
                &format!("{PF2E_SOURCE_PINNED_COMMIT}:{}", contract.source_path),
            ],
        )?;
        if digest_bytes(&bytes) != contract.source_file_digest {
            return Err(error(
                CoverageFailureCode::ReceiptProvenanceInvalid,
                "source digest does not match the blob from the pinned Git tree",
            ));
        }
        let raw: Value = serde_json::from_slice(&bytes)
            .map_err(|message| error(CoverageFailureCode::FixtureNotSourceGrounded, message))?;
        let manifest_bytes = git_output(
            source_repository,
            &[
                "show",
                &format!("{PF2E_SOURCE_PINNED_COMMIT}:static/system.json"),
            ],
        )?;
        let manifest: PinnedSystemManifest = serde_json::from_slice(&manifest_bytes)
            .map_err(|message| error(CoverageFailureCode::ReceiptProvenanceInvalid, message))?;
        let manifest_pack = manifest
            .packs
            .iter()
            .find(|pack| {
                contract
                    .source_path
                    .strip_prefix(&format!("{}/", pack.path.trim_end_matches('/')))
                    .is_some_and(|relative| !relative.is_empty())
            })
            .ok_or_else(|| {
                error(
                    CoverageFailureCode::FixtureNotSourceGrounded,
                    "source_path is not a top-level document in a pinned manifest pack",
                )
            })?;
        let raw_discriminator = raw.get("type").and_then(Value::as_str).ok_or_else(|| {
            error(
                CoverageFailureCode::FixtureNotSourceGrounded,
                "pinned fixture has no string type discriminator",
            )
        })?;
        let source_id = raw.get("_id").and_then(Value::as_str).ok_or_else(|| {
            error(
                CoverageFailureCode::FixtureNotSourceGrounded,
                "pinned fixture has no string _id",
            )
        })?;
        let parent_record_key = format!("{}:{source_id}", manifest_pack.name);
        let (actual_selector, resolved_record_key) = match expected_selector.role {
            SourceDocumentRole::Embedded => {
                let item_id = contract
                    .record_key
                    .strip_prefix(&format!("{parent_record_key}#item:"))
                    .filter(|value| !value.is_empty())
                    .ok_or_else(|| {
                        error(
                            CoverageFailureCode::FixtureNotSourceGrounded,
                            "embedded fixture record_key must identify one parent-owned item",
                        )
                    })?;
                let item = raw
                    .pointer("/items")
                    .and_then(Value::as_array)
                    .and_then(|items| {
                        items
                            .iter()
                            .find(|item| item.get("_id").and_then(Value::as_str) == Some(item_id))
                    })
                    .ok_or_else(|| {
                        error(
                            CoverageFailureCode::FixtureNotSourceGrounded,
                            "embedded fixture item does not exist in the pinned parent",
                        )
                    })?;
                let item_type = item.get("type").and_then(Value::as_str).ok_or_else(|| {
                    error(
                        CoverageFailureCode::FixtureNotSourceGrounded,
                        "embedded fixture item has no string type discriminator",
                    )
                })?;
                (
                    SourceLeafSelector {
                        source_contract_version: PF2E_SOURCE_CONTRACT_VERSION.to_string(),
                        document_class: "Item".to_string(),
                        type_discriminator: item_type.to_string(),
                        role: SourceDocumentRole::Embedded,
                        parent_context: SourceParentContextSelector {
                            document_class: Some(manifest_pack.document_class.clone()),
                            type_discriminator: Some(raw_discriminator.to_string()),
                            relationship_path: Some("Actor.items".to_string()),
                        },
                    },
                    format!("{parent_record_key}#item:{item_id}"),
                )
            }
            SourceDocumentRole::TopLevel => (
                SourceLeafSelector {
                    source_contract_version: PF2E_SOURCE_CONTRACT_VERSION.to_string(),
                    document_class: manifest_pack.document_class.clone(),
                    type_discriminator: raw_discriminator.to_string(),
                    role: SourceDocumentRole::TopLevel,
                    parent_context: SourceParentContextSelector::root(),
                },
                parent_record_key.clone(),
            ),
            SourceDocumentRole::Child
                if expected_selector.parent_context
                    == (SourceParentContextSelector {
                        document_class: Some("Item".to_string()),
                        type_discriminator: Some("consumable".to_string()),
                        relationship_path: Some("Consumable.system.spell".to_string()),
                    }) =>
            {
                let child_discriminator = raw
                    .pointer("/system/spell/type")
                    .and_then(Value::as_str)
                    .ok_or_else(|| {
                        error(
                            CoverageFailureCode::FixtureNotSourceGrounded,
                            "pinned consumable fixture has no spell child type discriminator",
                        )
                    })?;
                (
                    SourceLeafSelector {
                        source_contract_version: PF2E_SOURCE_CONTRACT_VERSION.to_string(),
                        document_class: "Item".to_string(),
                        type_discriminator: child_discriminator.to_string(),
                        role: SourceDocumentRole::Child,
                        parent_context: SourceParentContextSelector {
                            document_class: Some(manifest_pack.document_class.clone()),
                            type_discriminator: Some(raw_discriminator.to_string()),
                            relationship_path: Some("Consumable.system.spell".to_string()),
                        },
                    },
                    parent_record_key.clone(),
                )
            }
            _ => {
                return Err(error(
                    CoverageFailureCode::FixtureNotSourceGrounded,
                    "registered receipt selector role is not supported by this source path",
                ));
            }
        };
        if &actual_selector != expected_selector {
            return Err(error(
                CoverageFailureCode::FixtureNotSourceGrounded,
                format!(
                    "pinned fixture selector {:?}/{:?}/{:?}/{:?} does not match ledger selector",
                    actual_selector.document_class,
                    actual_selector.type_discriminator,
                    actual_selector.role,
                    actual_selector.parent_context
                ),
            ));
        }
        if resolved_record_key != contract.record_key {
            return Err(error(
                CoverageFailureCode::FixtureNotSourceGrounded,
                format!(
                    "fixture record_key {} does not match {resolved_record_key}",
                    contract.record_key
                ),
            ));
        }
        Ok(Self {
            reference: FixtureReference {
                case_id: contract.case_id.clone(),
                record_key: contract.record_key.clone(),
                source_path: contract.source_path.clone(),
                source_file_digest: contract.source_file_digest.clone(),
                excerpt_digest: contract.excerpt_digest.clone(),
                provenance: contract.provenance,
                source_contract_version: PF2E_SOURCE_CONTRACT_VERSION,
                source_commit: PF2E_SOURCE_PINNED_COMMIT,
                source_signature: PF2E_SOURCE_PINNED_SIGNATURE,
                registry_sha256: PF2E_TYPE_REGISTRY_SHA256,
            },
            raw,
            serialized: bytes,
        })
    }
}

fn git_success(repository: &Path, args: &[&str]) -> Result<(), CoverageContractError> {
    let output = Command::new("git")
        .arg("-C")
        .arg(repository)
        .args(args)
        .output()
        .map_err(|message| error(CoverageFailureCode::ReceiptProvenanceInvalid, message))?;
    if output.status.success() {
        Ok(())
    } else {
        Err(error(
            CoverageFailureCode::ReceiptProvenanceInvalid,
            "source repository does not contain the accepted pinned commit",
        ))
    }
}

fn git_output(repository: &Path, args: &[&str]) -> Result<Vec<u8>, CoverageContractError> {
    let output = Command::new("git")
        .arg("-C")
        .arg(repository)
        .args(args)
        .output()
        .map_err(|message| error(CoverageFailureCode::ReceiptProvenanceInvalid, message))?;
    if output.status.success() {
        Ok(output.stdout)
    } else {
        Err(error(
            CoverageFailureCode::ReceiptProvenanceInvalid,
            String::from_utf8_lossy(&output.stderr).trim(),
        ))
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct FixtureReference {
    case_id: String,
    record_key: String,
    source_path: String,
    source_file_digest: String,
    excerpt_digest: String,
    provenance: FixtureProvenance,
    source_contract_version: &'static str,
    source_commit: &'static str,
    source_signature: &'static str,
    registry_sha256: &'static str,
}

impl FixtureReference {
    pub(crate) fn case_id(&self) -> &str {
        &self.case_id
    }
    pub(crate) fn record_key(&self) -> &str {
        &self.record_key
    }
    pub(crate) fn source_path(&self) -> &str {
        &self.source_path
    }
    pub(crate) fn excerpt_digest(&self) -> &str {
        &self.excerpt_digest
    }
    pub(crate) fn source_file_digest(&self) -> &str {
        &self.source_file_digest
    }
    pub(crate) fn provenance(&self) -> FixtureProvenance {
        self.provenance
    }
    pub(crate) fn has_authenticated_source_binding(&self) -> bool {
        self.source_contract_version == PF2E_SOURCE_CONTRACT_VERSION
            && self.source_commit == PF2E_SOURCE_PINNED_COMMIT
            && self.source_signature == PF2E_SOURCE_PINNED_SIGNATURE
            && self.registry_sha256 == PF2E_TYPE_REGISTRY_SHA256
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(deny_unknown_fields)]
struct ActualReadEvidence {
    reader_id: String,
    accessor_binding: String,
    purpose: SourceAccessorPurpose,
    mutation_digest: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct StageObservation {
    stage: FinalOwnerStage,
    destination: String,
    accessor_binding: String,
    mutation_digest: String,
    value: SourcePresence<SourceLeafValue>,
}

impl StageObservation {
    pub(crate) fn stage(&self) -> FinalOwnerStage {
        self.stage
    }
    pub(crate) fn destination(&self) -> &str {
        &self.destination
    }
    pub(crate) fn accessor_binding(&self) -> &str {
        &self.accessor_binding
    }
    pub(crate) fn mutation_digest(&self) -> &str {
        &self.mutation_digest
    }
    pub(crate) fn value(&self) -> &SourcePresence<SourceLeafValue> {
        &self.value
    }
    pub(crate) fn mutation_changed_value(&self) -> bool {
        digest_serializable(&self.value)
            .is_ok_and(|actual_digest| actual_digest != self.mutation_digest)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(deny_unknown_fields)]
struct SemanticOutputObservation {
    accessor_binding: String,
    mutation_observed: bool,
    observed: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct SourceLeafValue {
    pub(crate) json_type: SourceJsonType,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(crate) value: Option<Value>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(crate) stable_digest: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(crate) member_kind: Option<SourceMemberKind>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(crate) member_identity: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(crate) ordinal: Option<usize>,
    pub(crate) multiplicity: usize,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(crate) unsupported: Option<TypedUnsupportedValue>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
#[allow(dead_code)]
pub(crate) enum SourceJsonType {
    Boolean,
    Number,
    String,
    Null,
    Array,
    Object,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize)]
#[serde(rename_all = "snake_case")]
#[allow(dead_code)]
pub(crate) enum SourceMemberKind {
    Array,
    Map,
    Nested,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct TypedUnsupportedValue {
    pub(crate) value: Value,
    pub(crate) reason: String,
}

fn string_leaf(value: String) -> SourcePresence<SourceLeafValue> {
    SourcePresence::Value(SourceLeafValue {
        json_type: SourceJsonType::String,
        value: Some(Value::String(value)),
        stable_digest: None,
        member_kind: None,
        member_identity: None,
        ordinal: None,
        multiplicity: 1,
        unsupported: None,
    })
}

fn stage(
    stage: FinalOwnerStage,
    destination: &str,
    accessor_binding: &str,
    value: String,
    mutation: &str,
) -> Result<StageObservation, CoverageContractError> {
    Ok(StageObservation {
        stage,
        destination: destination.to_string(),
        accessor_binding: accessor_binding.to_string(),
        mutation_digest: digest_serializable(&string_leaf(mutation.to_string()))?,
        value: string_leaf(value),
    })
}

fn digest_bytes(bytes: &[u8]) -> String {
    format!("sha256:{:x}", Sha256::digest(bytes))
}
fn digest_serializable(value: &impl Serialize) -> Result<String, CoverageContractError> {
    serde_json::to_vec(value)
        .map(|bytes| digest_bytes(&bytes))
        .map_err(|source_error| {
            error(
                CoverageFailureCode::ReceiptProvenanceInvalid,
                format!("failed to serialize sealed source-leaf evidence: {source_error}"),
            )
        })
}
fn evidence_digest(
    identity: &SourceLeafIdentity,
    fixture: &FixtureReference,
    source: &SourcePresence<SourceLeafValue>,
    reader: &ActualReadEvidence,
    observations: &[StageObservation],
    semantic_output: &Option<SemanticOutputObservation>,
) -> Result<String, CoverageContractError> {
    digest_serializable(&(
        identity,
        fixture,
        source,
        reader,
        observations,
        semantic_output,
    ))
}
fn error(code: CoverageFailureCode, message: impl ToString) -> CoverageContractError {
    CoverageContractError {
        code,
        message: message.to_string(),
    }
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use atlas_record::{SpellHeightening, SpellRule};

    use super::*;
    use crate::source_coverage::{
        ExpectedSourceShape, FinalOwnerContract, FixturePrevalence, MapKeyPolicy, ReaderContract,
        SourceDocumentRole, SourceLeafContract, SourceLeafDisposition, SourceLeafKind,
        SourceLeafSelector, SourceParentContextSelector, SourcePin, SourcePrevalence,
        SurfaceContract, SurfaceDecision, SurfaceDisposition, evaluate_source_leaf_coverage,
        parse_source_leaf_ledger,
    };

    #[test]
    fn mapped_spell_probes_fail_closed_if_routed_to_dedicated_accessors() {
        let standalone = spell_source_destination(SpellLeafProbe::Mapped)
            .expect_err("mapped standalone probe must use the generic receipt");
        let child = consumable_spell_child_pointer(ConsumableSpellChildLeafProbe::Mapped)
            .expect_err("mapped child probe must use the generic receipt");

        assert_eq!(standalone.code, CoverageFailureCode::ReaderNotObserved);
        assert_eq!(child.code, CoverageFailureCode::ReaderNotObserved);
    }

    #[test]
    fn json_pointer_mutation_rejects_scalar_and_null_parents_without_replacing_them() {
        for parent in [Value::String("authored".into()), Value::Null] {
            let mut root = serde_json::json!({"system": parent});
            let original = root.clone();

            let error = set_json_pointer(&mut root, "/system/value", Value::from(7))
                .expect_err("authored non-object parent must be rejected");

            assert_eq!(error.code, CoverageFailureCode::ReaderNotObserved);
            assert_eq!(root, original);
        }

        let mut root = serde_json::json!({"system": {}});
        set_json_pointer(&mut root, "/system/value", Value::from(7))
            .expect("missing member beneath an object remains insertable");
        assert_eq!(root, serde_json::json!({"system": {"value": 7}}));
    }

    fn promoted() -> SurfaceDecision {
        SurfaceDecision {
            disposition: SurfaceDisposition::Promoted,
            rationale: "selected by this engine fixture".to_string(),
        }
    }

    fn resolved_fixture(contract: &FixtureContract, raw: Value) -> ResolvedFixture {
        ResolvedFixture {
            reference: FixtureReference {
                case_id: contract.case_id.clone(),
                record_key: contract.record_key.clone(),
                source_path: contract.source_path.clone(),
                source_file_digest: contract.source_file_digest.clone(),
                excerpt_digest: contract.excerpt_digest.clone(),
                provenance: contract.provenance,
                source_contract_version: PF2E_SOURCE_CONTRACT_VERSION,
                source_commit: PF2E_SOURCE_PINNED_COMMIT,
                source_signature: PF2E_SOURCE_PINNED_SIGNATURE,
                registry_sha256: PF2E_TYPE_REGISTRY_SHA256,
            },
            serialized: serde_json::to_vec(&raw).expect("serialize fixture"),
            raw,
        }
    }

    fn ledger() -> SourceLeafCoverageLedger {
        SourceLeafCoverageLedger {
            contract_version: "atlas-source-leaf-coverage/v1".to_string(),
            type_id: "item--action--top-level--root--root--root".to_string(),
            source_pin: SourcePin::pinned(),
            selector: SourceLeafSelector {
                source_contract_version: PF2E_SOURCE_CONTRACT_VERSION.to_string(),
                document_class: "Item".to_string(),
                type_discriminator: "action".to_string(),
                role: SourceDocumentRole::TopLevel,
                parent_context: SourceParentContextSelector::root(),
            },
            leaves: vec![SourceLeafContract {
                normalized_path: "$.name".to_string(),
                leaf_kind: SourceLeafKind::Scalar,
                expected_shapes: vec![ExpectedSourceShape::String],
                source_prevalence: SourcePrevalence {
                    source_contract_version: PF2E_SOURCE_CONTRACT_VERSION.to_string(),
                    source_commit: PF2E_SOURCE_PINNED_COMMIT.to_string(),
                    source_signature: PF2E_SOURCE_PINNED_SIGNATURE.to_string(),
                    registry_sha256: PF2E_TYPE_REGISTRY_SHA256.to_string(),
                    inventory_version: crate::PF2E_SOURCE_LEAF_PREVALENCE_VERSION.to_string(),
                    inventory_sha256: crate::PF2E_SOURCE_LEAF_PREVALENCE_SHA256.to_string(),
                    entry_id: "item-action-top-level-name@4cbdaa37".to_string(),
                    // The authenticated registry reports 1,169 top-level action Items;
                    // required `name` occurs exactly once per record at this pin.
                    record_count: 1_169,
                    occurrence_count: 1_169,
                },
                fixture_prevalence: FixturePrevalence {
                    record_count: 1,
                    occurrence_count: 1,
                },
                map_key_policy: None::<MapKeyPolicy>,
                disposition: SourceLeafDisposition::Promoted,
                reader: ReaderContract {
                    reader_id: Some(ITEM_NAME_READER.to_string()),
                    parity_case_ids: vec!["registered-item-name".to_string()],
                },
                fixtures: vec![FixtureContract {
                    case_id: "registered-item-name".to_string(),
                    record_key: "actionspf2e:c40APnn4a7bWhtcZ".to_string(),
                    source_path: "packs/actions/a-challenge-for-heroes.json".to_string(),
                    source_file_digest:
                        "sha256:b71bc2d31b2a10c232a01ef57b5943b60e4747770b7f35d8ebeb27bf4b281d43"
                            .to_string(),
                    excerpt_digest:
                        "sha256:d3c3aced9d8c6e0b39d40df1f17917eac5f850a34abf4b2a76915d0bd2c4140c"
                            .to_string(),
                    provenance: FixtureProvenance::PinnedSource,
                }],
                final_owners: vec![
                    owner(FinalOwnerStage::SourceDto, "FullItemSource.name"),
                    owner(FinalOwnerStage::Canonical, "AtlasRecord.identity.name"),
                    owner(
                        FinalOwnerStage::PostProjection,
                        "IndexBuildInput.records[].identity.name",
                    ),
                    owner(
                        FinalOwnerStage::ArtifactHydration,
                        "RetrievedRecord.record.identity.name",
                    ),
                    owner(FinalOwnerStage::PublicSurface, "RecordJson.name"),
                ],
                surfaces: SurfaceContract {
                    artifact: promoted(),
                    app: promoted(),
                    ..SurfaceContract::default()
                },
                rationale: "exercise the sealed A1 engine over a pinned source leaf".to_string(),
                owner: "atlas-ingest::source_coverage".to_string(),
                acceptance_checkpoint: "A1 engine test only".to_string(),
                future_owner: None,
                future_task: None,
                prerequisite: None,
            }],
        }
    }

    fn owner(stage: FinalOwnerStage, destination: &str) -> FinalOwnerContract {
        FinalOwnerContract {
            stage,
            destination: destination.to_string(),
        }
    }

    fn require_pinned_repository() -> PathBuf {
        let repository = if let Some(path) =
            std::env::var_os("PF2E_SOURCE_REPOSITORY").map(PathBuf::from)
        {
            path
        } else {
            let output = Command::new("git")
                .args(["worktree", "list", "--porcelain"])
                .output()
                .expect("TEST PREREQUISITE: git must locate the Atlas worktree list");
            String::from_utf8(output.stdout)
                .expect("TEST PREREQUISITE: git worktree output must be UTF-8")
                .lines()
                .filter_map(|line| line.strip_prefix("worktree "))
                .map(|root| Path::new(root).join("vendor/pf2e"))
                .find(|candidate| candidate.is_dir())
                .expect(
                    "TEST PREREQUISITE: set PF2E_SOURCE_REPOSITORY to a Git repository containing the accepted PF2E commit",
                )
        };
        let accepted_commit = format!("{PF2E_SOURCE_PINNED_COMMIT}^{{commit}}");
        let output = Command::new("git")
            .args(["-C"])
            .arg(&repository)
            .args(["cat-file", "-e", &accepted_commit])
            .output()
            .expect("TEST PREREQUISITE: git must inspect the PF2E source repository");
        assert!(
            output.status.success(),
            "TEST PREREQUISITE: PF2E_SOURCE_REPOSITORY must contain accepted commit {PF2E_SOURCE_PINNED_COMMIT}: {}",
            String::from_utf8_lossy(&output.stderr)
        );
        repository
    }

    #[test]
    fn changed_but_inverted_remaster_owner_fails_semantic_equivalence() {
        let remaster = |value| {
            SourcePresence::Value(SourceLeafValue {
                json_type: SourceJsonType::Boolean,
                value: Some(Value::Bool(value)),
                stable_digest: None,
                member_kind: None,
                member_identity: None,
                ordinal: None,
                multiplicity: 1,
                unsupported: None,
            })
        };
        let source = remaster(true);
        let source_mutation = remaster(false);
        let error = owner_presence_stage(
            FinalOwnerStage::Canonical,
            "HazardPublication.remaster",
            "sealed::HazardPublication.remaster semantic equivalence",
            &source,
            &source_mutation,
            (remaster(false), Some(remaster(true))),
            (
                "HazardPublication.remaster=false",
                "HazardPublication.remaster=true",
            ),
        )
        .expect_err("a mutation-sensitive but inverted owner must fail source parity");

        assert_eq!(error.code, CoverageFailureCode::CanonicalMismatch);
        assert!(error.message.contains("owner meaning"));
    }

    #[test]
    fn exact_array_owner_with_extra_member_fails_collection_equivalence() {
        let occurrence = ExactHazardLeafOccurrence {
            pointer: "/system/statusEffects/0".to_string(),
            value: Some(Value::String("cold".to_string())),
            member_kind: Some(SourceMemberKind::Array),
            member_identity: Some("/system/statusEffects/0".to_string()),
            ordinal: Some(0),
            collection_cardinality: Some(1),
            collection_authored_order: Some(0),
        };
        let owner = vec!["cold".to_string(), "fire".to_string()];

        let error = array_member_owner_value(
            &owner,
            &occurrence,
            CoverageFailureCode::CanonicalMismatch,
            "adversarial canonical status effects",
        )
        .expect_err("an owner with an extra member must fail source collection parity");

        assert_eq!(error.code, CoverageFailureCode::CanonicalMismatch);
        assert!(error.message.contains("owner_cardinality=2"));
        assert!(error.message.contains("source_cardinality=Some(1)"));
    }

    #[test]
    fn compatible_cross_ledger_mutations_share_one_grouped_hydration_key() {
        let ledgers = hazard_hydration_ledgers().expect("accepted hazard hydration ledgers");
        let actor_ledger = ledgers
            .iter()
            .find(|ledger| ledger.type_id == "actor--hazard--top-level--root--root--root")
            .expect("Actor hazard ledger");
        let action_ledger = ledgers
            .iter()
            .find(|ledger| ledger.type_id == "item--action--embedded--actor--hazard--actor-items")
            .expect("embedded Action hazard ledger");
        let raw: Value = serde_json::from_str(include_str!(
            "../../tests/fixtures/hazards/pinned/packs/age-of-ashes-bestiary/book-2-cult-of-cinders/dragon-pillar.json"
        ))
        .expect("Dragon Pillar fixture JSON");
        let actor_leaf = actor_ledger
            .leaves
            .iter()
            .find(|leaf| leaf.normalized_path == "$.items[].system.traits.value[]")
            .expect("Actor-owned embedded trait leaf");
        let action_leaf = action_ledger
            .leaves
            .iter()
            .find(|leaf| leaf.normalized_path == "$.system.actionType.value")
            .expect("embedded Action action-type leaf");
        let actor_contract = actor_leaf
            .fixtures
            .iter()
            .find(|fixture| fixture.source_path.ends_with("/dragon-pillar.json"))
            .expect("Dragon Pillar Actor fixture");
        let action_contract = action_leaf
            .fixtures
            .iter()
            .find(|fixture| fixture.source_path.ends_with("/dragon-pillar.json"))
            .expect("Dragon Pillar Action fixture");
        let fixture = resolved_fixture(actor_contract, raw.clone());
        let actor_identity = actor_ledger.identity_for(actor_leaf);
        let action_identity = action_ledger.identity_for(action_leaf);

        let actor_group = grouped_hazard_hydration_mutation(&fixture, &actor_identity)
            .expect("Actor hydration group");
        let action_group = grouped_hazard_hydration_mutation(&fixture, &action_identity)
            .expect("Action hydration group");
        assert_eq!(actor_group, action_group);
        assert_eq!(
            hazard_hydration_cache_key(&fixture, &actor_group).expect("Actor group cache key"),
            hazard_hydration_cache_key(&fixture, &action_group).expect("Action group cache key")
        );

        for (ledger, leaf, contract) in [
            (actor_ledger, actor_leaf, actor_contract),
            (action_ledger, action_leaf, action_contract),
        ] {
            let identity = ledger.identity_for(leaf);
            let selected_fixture = resolved_fixture(contract, raw.clone());
            let registration = registration_for(&identity, leaf.reader.reader_id.as_deref())
                .expect("registered grouped accessor");
            let (pointer, expected) =
                grouped_hazard_leaf_mutation(registration, &identity, &selected_fixture)
                    .expect("grouped mutation")
                    .expect("compatible mutation");
            assert_eq!(actor_group.pointer(&pointer), Some(&expected));
        }
    }

    #[test]
    fn wrong_persisted_hazard_value_fails_artifact_hydration_parity() {
        let ledger = parse_source_leaf_ledger(include_str!(
            "../../../../contracts/source-leaf-coverage/v1/actor-hazard.yaml"
        ))
        .expect("Actor hazard ledger");
        let leaf = ledger
            .leaves
            .iter()
            .find(|leaf| leaf.normalized_path == "$.name")
            .expect("hazard name leaf");
        let identity = ledger.identity_for(leaf);
        let fixture_contract = &leaf.fixtures[0];
        let fixture = resolved_fixture(
            fixture_contract,
            serde_json::from_str(include_str!(
                "../../tests/fixtures/hazards/pinned/packs/hazards/hidden-pit.json"
            ))
            .expect("Hidden Pit fixture JSON"),
        );
        let occurrence = first_exact_hazard_leaf(&fixture, &identity).expect("hazard name");
        let source = occurrence.receipt_value(HazardExactDisposition::Promoted);
        let pipeline = run_hazard_pipeline_with_artifact_hook(
            &fixture,
            fixture.raw.clone(),
            |artifact_path| {
                let connection = rusqlite::Connection::open(artifact_path).map_err(|message| {
                    error(CoverageFailureCode::ArtifactHydrationMismatch, message)
                })?;
                let canonical_json: String = connection
                    .query_row(
                        "SELECT canonical_json FROM canonical_hazard_records WHERE record_key=?1",
                        [fixture.reference.record_key.as_str()],
                        |row| row.get(0),
                    )
                    .map_err(|message| {
                        error(CoverageFailureCode::ArtifactHydrationMismatch, message)
                    })?;
                let mut canonical: Value =
                    serde_json::from_str(&canonical_json).map_err(|message| {
                        error(CoverageFailureCode::ArtifactHydrationMismatch, message)
                    })?;
                *canonical
                    .pointer_mut("/value/identity/name")
                    .ok_or_else(|| {
                        error(
                            CoverageFailureCode::ArtifactHydrationMismatch,
                            "persisted hazard body has no canonical identity name",
                        )
                    })? = Value::String("Wrong Persisted Hazard".to_string());
                connection
                    .execute(
                        "UPDATE canonical_hazard_records SET canonical_json=?1 WHERE record_key=?2",
                        rusqlite::params![canonical.to_string(), fixture.reference.record_key],
                    )
                    .map_err(|message| {
                        error(CoverageFailureCode::ArtifactHydrationMismatch, message)
                    })?;
                Ok(true)
            },
        )
        .expect("manifest-bound SQLite hydration of deliberately wrong persisted value");
        let values = exact_hazard_owner_values(
            &pipeline,
            &identity,
            &occurrence,
            HazardExactDisposition::Promoted,
        )
        .expect("independent hazard owner values");
        let hydration = exact_hazard_hydration_owner_evidence(
            &pipeline,
            &fixture,
            &identity,
            HazardExactDisposition::Promoted,
        )
        .expect("independent artifact hydration owner value");
        assert_eq!(values.source, source);
        assert_eq!(values.canonical, source);
        assert_eq!(values.post_projection, source);
        assert_eq!(hydration.expected_source, source);

        let error = owner_presence_stage(
            FinalOwnerStage::ArtifactHydration,
            "RetrievedRecord.body.RecordBody::Hazard.identity.name",
            "SqliteIndexReader::load_hydrated_records_by_key",
            &source,
            &source,
            (hydration.owner_value, None),
            ("wrong persisted hazard", "unreached mutation"),
        )
        .expect_err("a wrong persisted value must fail artifact-hydration parity");
        assert_eq!(error.code, CoverageFailureCode::ArtifactHydrationMismatch);
        assert!(error.message.contains("owner meaning"));
    }

    #[test]
    fn registered_receipt_executes_real_pipeline_and_does_not_equate_global_prevalence() {
        let repository = require_pinned_repository();
        let ledger = ledger();
        let receipt = capture_registered_source_leaf_receipt(&ledger, 0, 0, &repository)
            .expect("sealed registered receipt");
        let report = evaluate_source_leaf_coverage(&ledger, &[receipt]);
        assert!(report.passed, "{:#?}", report.failures);
        assert_eq!(ledger.leaves[0].source_prevalence.occurrence_count, 1_169);
        assert_eq!(ledger.leaves[0].fixture_prevalence.occurrence_count, 1);
    }

    #[test]
    fn unchanged_actual_owner_value_cannot_pass_on_a_mutation_digest_alone() {
        let unchanged = string_leaf("authored".to_string());
        let error = presence_stage(
            FinalOwnerStage::Canonical,
            "SpellDefinition.source_context.publication_license",
            "canonical typed accessor",
            unchanged.clone(),
            &unchanged,
        )
        .expect_err("an unchanged actual owner must fail even when it can be hashed");
        assert_eq!(error.code, CoverageFailureCode::CanonicalMismatch);
        assert!(error.message.contains("did not observe"));
    }

    #[test]
    fn grouped_spell_artifacts_keep_typed_siblings_while_isolating_discriminators() {
        const ACIDIC_BURST: &str = "packs/spells/1st-rank/acidic-burst.json";
        const QI_BLAST: &str = "packs/spells/focus/qi-blast.json";

        let repository = require_pinned_repository();
        let ledger = parse_source_leaf_ledger(include_str!(
            "../../../../contracts/source-leaf-coverage/v1/item-spell.yaml"
        ))
        .expect("Item spell ledger");
        let mut fixture_entries =
            registered_spell_fixture_entries(&ledger).expect("registered spell fixtures");
        fixture_entries
            .retain(|source_path, _| matches!(source_path.as_str(), ACIDIC_BURST | QI_BLAST));
        let (evidence, operations) =
            build_spell_artifact_evidence(&ledger, &repository, &fixture_entries)
                .expect("partitioned source-backed artifact evidence");
        assert_eq!(operations.source_file_count, 2);
        assert_eq!(operations.compatible_sibling_group_count, 2);
        assert_eq!(operations.isolated_discriminator_group_count, 5);
        assert_eq!(operations.sqlite_cycle_count(), 9);

        let acidic = evidence.get(ACIDIC_BURST).expect("Acidic Burst evidence");
        let acidic_siblings = acidic
            .mutations
            .get(&SpellArtifactMutationGroup::CompatibleSiblings)
            .and_then(|hydration| hydration.body.as_ref())
            .expect("hydrated Acidic Burst sibling mutation");
        let FactValue::Value(SpellSourceValue::Known(SpellHeightening::Interval(interval))) =
            &acidic_siblings.definition.heightening
        else {
            panic!("Acidic Burst sibling mutations must retain typed interval heightening");
        };
        assert_eq!(
            interval.interval,
            FactValue::Value(SpellSourceValue::Known(2)),
            "the hydrated model must contain the compatible interval mutation"
        );
        let acidic_type_group = SpellArtifactMutationGroup::IsolatedDiscriminator {
            normalized_path: "$.system.heightening.type".to_string(),
            case_id: "item-spell-top-level-system-heightening-type".to_string(),
        };
        let acidic_type = acidic
            .mutations
            .get(&acidic_type_group)
            .and_then(|hydration| hydration.body.as_ref())
            .expect("hydrated Acidic Burst discriminator mutation");
        let FactValue::Value(SpellSourceValue::Unsupported(unsupported)) =
            &acidic_type.definition.heightening
        else {
            panic!("isolated Acidic Burst type mutation must be typed unsupported");
        };
        assert!(unsupported.value.contains("source-leaf-mutation"));

        let qi_blast = evidence.get(QI_BLAST).expect("Qi Blast evidence");
        let qi_siblings = qi_blast
            .mutations
            .get(&SpellArtifactMutationGroup::CompatibleSiblings)
            .and_then(|hydration| hydration.body.as_ref())
            .expect("hydrated Qi Blast sibling mutation");
        let FactValue::Value(SpellSourceValue::Known(rules)) = &qi_siblings.definition.rules else {
            panic!("Qi Blast sibling mutations must retain typed rules");
        };
        assert!(matches!(rules[0].rule, SpellRule::RollOption(_)));
        assert!(matches!(rules[1].rule, SpellRule::DamageAlteration(_)));
        assert!(matches!(rules[2].rule, SpellRule::ItemAlteration(_)));
        assert!(matches!(rules[3].rule, SpellRule::ItemAlteration(_)));

        for ordinal in 0..4 {
            let group = SpellArtifactMutationGroup::IsolatedDiscriminator {
                normalized_path: "$.system.rules[].key".to_string(),
                case_id: format!("qi-blast-rule-{ordinal}"),
            };
            let isolated = qi_blast
                .mutations
                .get(&group)
                .and_then(|hydration| hydration.body.as_ref())
                .expect("hydrated isolated Qi Blast rule-key mutation");
            let FactValue::Value(SpellSourceValue::Known(rules)) = &isolated.definition.rules
            else {
                panic!("isolated Qi Blast key mutation must retain the typed rule collection");
            };
            assert!(matches!(rules[ordinal].rule, SpellRule::Unsupported(_)));
            assert_eq!(
                rules
                    .iter()
                    .filter(|rule| matches!(rule.rule, SpellRule::Unsupported(_)))
                    .count(),
                1,
                "each isolated artifact mutates exactly one rule discriminator"
            );
        }
    }

    #[test]
    fn source_backed_spell_description_rejects_constant_and_wrong_content_owners() {
        let repository = require_pinned_repository();
        let ledger = parse_source_leaf_ledger(include_str!(
            "../../../../contracts/source-leaf-coverage/v1/item-spell.yaml"
        ))
        .expect("Item spell ledger");
        let mut receipts = Vec::new();
        for (leaf_index, leaf) in ledger.leaves.iter().enumerate() {
            for fixture_index in 0..leaf.fixtures.len() {
                receipts.push(
                    capture_registered_source_leaf_receipt(
                        &ledger,
                        leaf_index,
                        fixture_index,
                        &repository,
                    )
                    .expect("source-backed spell receipt"),
                );
            }
        }
        let description_index = receipts
            .iter()
            .position(|receipt| receipt.identity.normalized_path == "$.system.description.value")
            .expect("description value receipt");
        let receipt = &receipts[description_index];
        let report = evaluate_source_leaf_coverage(&ledger, &receipts);
        assert!(report.passed, "{:#?}", report.failures);
        let SourcePresence::Value(source) = receipt.source() else {
            panic!("description receipt must carry typed semantic evidence");
        };
        assert!(source.value.is_none(), "raw HTML is not owner parity");
        assert!(source.stable_digest.is_some());
        assert!(
            receipt
                .observations()
                .iter()
                .all(StageObservation::mutation_changed_value),
            "every promoted stage must observe the source-backed description mutation"
        );

        let authored_target = "Compendium.pf2e.spells-srd.Item.Heal".to_string();
        let mut transformed_reference =
            RichDocument::new(vec![atlas_record::RichNode::FoundryLink {
                link: atlas_record::FoundryLink {
                    target: RichLinkTarget::Unresolved {
                        target: authored_target.clone(),
                        fallback_label: "Heal".to_string(),
                    },
                    label: None,
                    source: atlas_record::FoundryLinkSource {
                        macro_kind: atlas_record::FoundryLinkMacroKind::Uuid,
                        authored_target,
                        relation: None,
                    },
                    behavior: atlas_record::FoundryLinkBehavior::Reference,
                },
            }]);
        let unresolved = rich_document_leaf(&transformed_reference, MappedSpellStage::Source)
            .expect("unresolved authored semantics");
        visit_foundry_links_mut(&mut transformed_reference, |link| {
            link.target = RichLinkTarget::Record {
                key: atlas_domain::RecordKey::parse("spells-srd:rfZpqmj0AIIdkVIs")
                    .expect("Heal record key"),
                name: "Heal".to_string(),
            };
        });
        assert_eq!(
            rich_document_leaf(&transformed_reference, MappedSpellStage::PostProjection)
                .expect("resolved authored semantics"),
            unresolved,
            "reference resolution is an intentional RichDocument transformation, not source loss"
        );

        let mut constant_receipts = receipts.clone();
        let constant_owner = &mut constant_receipts[description_index];
        let constant = constant_owner
            .observations
            .iter_mut()
            .find(|observation| observation.stage == FinalOwnerStage::Canonical)
            .expect("canonical observation");
        constant.mutation_digest =
            digest_serializable(&constant.value).expect("constant owner digest");
        constant_owner.evidence_digest = evidence_digest(
            &constant_owner.identity,
            &constant_owner.fixture,
            &constant_owner.source,
            &constant_owner.reader,
            &constant_owner.observations,
            &constant_owner.semantic_output,
        )
        .expect("reseal constant-owner negative");
        let report = evaluate_source_leaf_coverage(&ledger, &constant_receipts);
        assert!(report.failures.iter().any(|failure| {
            failure.code == CoverageFailureCode::CanonicalMismatch
                && failure.message.contains("did not observe")
        }));

        let mut wrong_receipts = receipts;
        let wrong_owner = &mut wrong_receipts[description_index];
        let wrong = wrong_owner
            .observations
            .iter_mut()
            .find(|observation| observation.stage == FinalOwnerStage::Canonical)
            .expect("canonical observation");
        wrong.value = rich_document_leaf(
            &RichDocument::new(vec![atlas_record::RichNode::Text {
                text: "unrelated owner content".to_string(),
            }]),
            MappedSpellStage::Canonical,
        )
        .expect("wrong typed content observation");
        wrong_owner.evidence_digest = evidence_digest(
            &wrong_owner.identity,
            &wrong_owner.fixture,
            &wrong_owner.source,
            &wrong_owner.reader,
            &wrong_owner.observations,
            &wrong_owner.semantic_output,
        )
        .expect("reseal wrong-owner negative");
        let report = evaluate_source_leaf_coverage(&ledger, &wrong_receipts);
        assert!(report.failures.iter().any(|failure| {
            failure.code == CoverageFailureCode::CanonicalMismatch
                && failure.message.contains("parity mismatch in value")
        }));
    }

    #[test]
    fn missing_pin_and_wrong_digest_fail_at_distinct_authentication_steps() {
        let ledger = ledger();
        let error = capture_registered_source_leaf_receipt(&ledger, 0, 0, Path::new("."))
            .expect_err("Atlas repository is not the pinned PF2e source repository");
        assert_eq!(error.code, CoverageFailureCode::ReceiptProvenanceInvalid);

        let mut fake = ledger.clone();
        fake.leaves[0].fixtures[0].source_file_digest = format!("sha256:{}", "0".repeat(64));
        let repository = require_pinned_repository();
        let error = capture_registered_source_leaf_receipt(&fake, 0, 0, &repository)
            .expect_err("wrong digest must fail after the pinned commit resolves");
        assert_eq!(error.code, CoverageFailureCode::ReceiptProvenanceInvalid);
        assert!(error.message.contains("source digest"));
    }

    #[test]
    fn pinned_wrong_type_item_cannot_satisfy_action_selector() {
        let repository = require_pinned_repository();
        let mut wrong_type = ledger();
        let fixture = &mut wrong_type.leaves[0].fixtures[0];
        fixture.record_key = "feats-srd:j54VJmwwAQZBlS6J".to_string();
        fixture.source_path = "packs/feats/ancestry/anadi/anadi-lore.json".to_string();
        fixture.source_file_digest =
            "sha256:eb780ea7b02cf283631785e2cacd4228c8b78e8b76d08777b4eaba376f339300".to_string();
        fixture.excerpt_digest =
            "sha256:f9523b02833f49a7bc12cb863e0c1ab0519253b62cd34182a0ea8beeb7d7d287".to_string();
        let error = capture_registered_source_leaf_receipt(&wrong_type, 0, 0, &repository)
            .expect_err("pinned Item/feat fixture cannot satisfy Item/action selector");
        assert_eq!(error.code, CoverageFailureCode::FixtureNotSourceGrounded);
        assert!(error.message.contains("does not match ledger selector"));
    }

    #[test]
    fn unregistered_direct_json_shape_has_no_receipt_path() {
        let mut ledger = ledger();
        ledger.leaves[0].reader.reader_id = Some("tests::DirectJsonAccessor".to_string());
        let error = capture_registered_source_leaf_receipt(&ledger, 0, 0, Path::new("."))
            .expect_err("unregistered caller accessors are rejected before source loading");
        assert_eq!(error.code, CoverageFailureCode::ReaderNotObserved);
    }

    #[test]
    fn actor_adversaries_keep_exact_authored_null_identity_distinct_from_optional_null() {
        let repository = require_pinned_repository();
        let ledger = parse_source_leaf_ledger(include_str!(
            "../../../../contracts/source-leaf-coverage/v1/actor-npc.yaml"
        ))
        .expect("Actor NPC ledger");

        let malformed = capture_registered_source_leaf_receipt(&ledger, 6, 1, &repository)
            .expect("malformed shadow-skill receipt");
        let SourcePresence::Value(value) = malformed.source() else {
            panic!("authored malformed key with null base is populated unsupported evidence");
        };
        assert_eq!(value.member_identity.as_deref(), Some("acrobatics+13"));
        assert_eq!(value.json_type, SourceJsonType::Null);
        assert_eq!(value.value, Some(Value::Null));
        assert_eq!(
            value
                .unsupported
                .as_ref()
                .map(|unsupported| &unsupported.value),
            Some(&Value::Null)
        );
        for observation in malformed.observations() {
            assert_eq!(
                observation.value(),
                malformed.source(),
                "every selected owner retains the exact unknown authored key"
            );
        }

        let malformed_identity = ledger.identity_for(&ledger.leaves[6]);
        let malformed_fixture = ResolvedFixture::load(
            &ledger.leaves[6].fixtures[1],
            &repository,
            &malformed_identity.selector,
        )
        .expect("malformed pinned fixture");
        let malformed_pipeline =
            run_npc_pipeline(&malformed_fixture, malformed_fixture.raw.clone())
                .expect("malformed production pipeline");
        assert!(
            !malformed_pipeline
                .diagnostic_source_fields
                .iter()
                .any(|field| field == "$.system.skills.acrobatics+13")
        );

        let gray_identity = ledger.identity_for(&ledger.leaves[6]);
        let gray_fixture = ResolvedFixture::load(
            &ledger.leaves[6].fixtures[0],
            &repository,
            &gray_identity.selector,
        )
        .expect("Gray Master pinned fixture");
        let gray_pipeline = run_npc_pipeline(&gray_fixture, gray_fixture.raw.clone())
            .expect("Gray Master production pipeline");
        let gray_intimidation = gray_pipeline
            .canonical
            .skills
            .value
            .as_value()
            .expect("Gray Master skills")
            .iter()
            .find(|skill| skill.kind == CreatureSkillKind::Intimidation)
            .expect("exact intimidate alias becomes Intimidation");
        assert_eq!(gray_intimidation.modifier, FactValue::Value(38));
        assert_eq!(
            gray_intimidation
                .source_entries
                .iter()
                .map(|entry| entry.authored_key.as_str())
                .collect::<Vec<_>>(),
            ["intimidation", "intimidate"]
        );
        assert!(
            gray_pipeline
                .canonical
                .skills
                .value
                .as_value()
                .expect("Gray Master skills")
                .iter()
                .all(|skill| skill.kind != CreatureSkillKind::Unmodeled)
        );

        let known_optional_null =
            capture_registered_source_leaf_receipt(&ledger, 4, 3, &repository)
                .expect("Inkdrop explicit-null wisdom receipt");
        assert!(matches!(known_optional_null.source(), SourcePresence::Null));

        let divergent = capture_registered_source_leaf_receipt(&ledger, 0, 2, &repository)
            .expect("Karumzek divergent mod/value receipt");
        assert_eq!(divergent.source(), &number_leaf(6));
        for observation in divergent.observations() {
            assert_eq!(observation.value(), divergent.source());
        }
    }

    #[test]
    fn exact_intimidate_alias_is_indispensable_through_real_sqlite() {
        struct TemporaryArtifact(PathBuf);

        impl TemporaryArtifact {
            fn new() -> Self {
                let nonce = std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .expect("system time after Unix epoch")
                    .as_nanos();
                let root = std::env::temp_dir().join(format!(
                    "pf2e-atlas-b1-exact-alias-{}-{nonce}",
                    std::process::id()
                ));
                std::fs::create_dir_all(&root).expect("create alias-only artifact root");
                Self(root)
            }

            fn artifact(&self) -> PathBuf {
                self.0.join("index.sqlite")
            }

            fn manifest(&self) -> PathBuf {
                self.0.join("manifest.json")
            }
        }

        impl Drop for TemporaryArtifact {
            fn drop(&mut self) {
                let _ = std::fs::remove_dir_all(&self.0);
            }
        }

        fn assert_exact_alias_skill(creature: &CreatureRecord) {
            let skills = creature.skills.value.as_value().expect("creature skills");
            let intimidation = skills
                .iter()
                .find(|skill| skill.kind == CreatureSkillKind::Intimidation)
                .expect("exact intimidate alias must produce modeled Intimidation");
            assert_eq!(intimidation.modifier, FactValue::Value(38));
            assert_eq!(intimidation.source_entries.len(), 1);
            assert_eq!(intimidation.source_entries[0].authored_key, "intimidate");
            assert_eq!(
                intimidation.source_entries[0].modifier,
                FactValue::Value(38)
            );
            assert!(intimidation.unmodeled.as_value().is_none());
            assert!(
                skills
                    .iter()
                    .all(|skill| skill.kind != CreatureSkillKind::Unmodeled)
            );
        }

        let repository = require_pinned_repository();
        let ledger = parse_source_leaf_ledger(include_str!(
            "../../../../contracts/source-leaf-coverage/v1/actor-npc.yaml"
        ))
        .expect("Actor NPC ledger");
        let identity = ledger.identity_for(&ledger.leaves[6]);
        let fixture = ResolvedFixture::load(
            &ledger.leaves[6].fixtures[0],
            &repository,
            &identity.selector,
        )
        .expect("Gray Master pinned fixture");
        validate_actor_excerpt(&fixture).expect("authenticated Gray Master excerpt");
        let source = shadow_skill_from_raw(&fixture.raw, "intimidate")
            .expect("populated authored intimidate base");
        let mut alias_only = fixture.raw.clone();
        let skills = alias_only
            .pointer_mut("/system/skills")
            .and_then(Value::as_object_mut)
            .expect("Gray Master skills");
        assert_eq!(
            skills
                .get("intimidate")
                .and_then(|skill| skill.get("base"))
                .and_then(Value::as_i64),
            Some(38)
        );
        assert!(
            skills.remove("intimidation").is_some(),
            "mutation removes only the canonical sibling"
        );
        assert!(skills.contains_key("intimidate"));
        let alias_only = run_npc_pipeline(&fixture, alias_only).expect("alias-only pipeline");

        assert_eq!(
            dto_shadow_skill_value(&alias_only.source_dto, "intimidate"),
            source
        );
        assert_exact_alias_skill(&alias_only.canonical);
        assert_eq!(
            creature_skill_value(&alias_only.canonical, "intimidate"),
            source
        );
        assert_exact_alias_skill(&alias_only.post_projection);
        assert_eq!(
            creature_skill_value(&alias_only.post_projection, "intimidate"),
            source
        );

        let record_key = alias_only.persisted_record.identity.key.clone();
        let input = atlas_index::IndexBuildInput {
            source_signature: PF2E_SOURCE_PINNED_SIGNATURE.to_string(),
            source_record_count: 1,
            packs: vec![alias_only.persisted_pack],
            records: vec![alias_only.persisted_record],
            canonical_bodies: vec![RecordBody::Creature(alias_only.post_projection)],
            canonical_spell_children: Vec::new(),
            references: Vec::new(),
            aliases: Vec::new(),
            remaster_links: Vec::new(),
            pending_document_embeddings: Vec::new(),
            document_embeddings: Vec::new(),
        };
        assert!(input.pending_document_embeddings.is_empty());
        assert!(input.document_embeddings.is_empty());

        let artifact = TemporaryArtifact::new();
        let artifact_path = artifact.artifact();
        let receipt = atlas_index::IndexArtifactWriter::write(
            &atlas_index::SqliteIndexWriter::new(artifact_path.clone()),
            &input,
            atlas_embedding::EmbeddingModelId::BgeSmallEnV15,
        )
        .expect("write alias-only canonical JSON to SQLite");
        std::fs::write(
            artifact.manifest(),
            serde_json::to_vec(&serde_json::json!({
                "manifest_version": atlas_index::ARTIFACT_MANIFEST_VERSION,
                "artifact_contract_version": atlas_index::ARTIFACT_CONTRACT_VERSION,
                "schema_version": atlas_index::ARTIFACT_SCHEMA_VERSION,
                "build": { "artifact_sha256": receipt.artifact_sha256() },
            }))
            .expect("serialize alias-only artifact manifest"),
        )
        .expect("write alias-only artifact manifest");
        let reader = atlas_index::SqliteIndexReader::open_read_only(&artifact_path)
            .expect("open alias-only SQLite artifact");
        let hydrated = reader
            .load_hydrated_records_by_key(std::slice::from_ref(&record_key))
            .expect("hydrate alias-only record from SQLite");
        assert_eq!(hydrated.len(), 1);
        let sqlite_creature = creature_body(hydrated[0].body.as_ref())
            .expect("SQLite-hydrated Gray Master creature body");
        assert_exact_alias_skill(sqlite_creature);
        assert_eq!(creature_skill_value(sqlite_creature, "intimidate"), source);

        let cli = record_json(
            &hydrated[0],
            RecordJsonOptions {
                detail: DetailLevel::Full,
                include_source_json: false,
            },
        )
        .expect("project SQLite-hydrated alias-only record");
        let RecordPresentationJson::Creature {
            skills: Some(cli_skills),
            ..
        } = &cli.presentation
        else {
            panic!("alias-only CLI projection must include creature skills");
        };
        let cli_intimidation = cli_skills
            .iter()
            .find(|skill| skill.slug == "intimidation")
            .expect("exact alias must project as CLI Intimidation");
        assert_eq!(cli_intimidation.modifier, Some(38));
        assert_eq!(cli_intimidation.source_entries.len(), 1);
        assert_eq!(
            cli_intimidation.source_entries[0].authored_key,
            "intimidate"
        );
        assert_eq!(
            cli_intimidation.source_entries[0].modifier,
            atlas_record::CreatureIntegerPresenceJson::Value(38)
        );
        assert!(cli_intimidation.unmodeled.is_none());
        assert_eq!(public_skill_value(&cli, "intimidate"), source);
    }
}
