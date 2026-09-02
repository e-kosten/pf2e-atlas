use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::process::Command;

use atlas_domain::{DetailLevel, PackName};
use atlas_record::{
    CreatureRecord, CreatureSkillKind, FactValue, RecordBody, RecordJson, RecordJsonOptions,
    RecordPresentationJson, record_json,
};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use sha2::{Digest, Sha256};

use crate::SourcePresence;
use crate::diagnostics::IngestDiagnostics;
use crate::index_build_input::index_build_input;
use crate::source::dto::{
    PF2E_SOURCE_CONTRACT_VERSION, PF2E_SOURCE_PINNED_COMMIT, PF2E_SOURCE_PINNED_SIGNATURE,
    SourceIdentity, VersionedNpcSource, parse_item_source, parse_npc_source,
    pinned_source_version_metadata,
};
use crate::source::normalize::normalize_record;
use crate::source::{LoadedPack, ManifestPack, SourceLoad};

use super::{
    CoverageContractError, CoverageFailureCode, FinalOwnerStage, FixtureContract,
    FixtureProvenance, PF2E_TYPE_REGISTRY_SHA256, SourceDocumentRole, SourceLeafCoverageLedger,
    SourceLeafIdentity, SourceLeafSelector, SourceParentContextSelector, lint_source_leaf_ledger,
};

const ITEM_NAME_READER: &str = "source::dto::FullItemSource::name";
const NPC_ABILITY_MOD_READER: &str = "source::dto::NpcLegacyAbilitySource::mod";
const NPC_SKILLS_READER: &str = "source::dto::NpcCoreSource::skills";

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
        self.evidence_digest
            == evidence_digest(
                &self.identity,
                &self.fixture,
                &self.source,
                &self.reader,
                &self.observations,
                &self.semantic_output,
            )
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
    )
}

#[derive(Debug, Clone, Copy)]
enum RegisteredAccessor {
    ItemActionName,
    ActorNpcAbilityMod(AbilitySlot),
    ActorNpcShadowSkillBase,
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
        && ability_slot(&identity.normalized_path).is_some()
    {
        Ok(RegisteredAccessor::ActorNpcAbilityMod(
            ability_slot(&identity.normalized_path).expect("checked above"),
        ))
    } else if identity.type_id == "actor--npc--top-level--root--root--root"
        && identity.selector.document_class == "Actor"
        && identity.selector.type_discriminator == "npc"
        && identity.normalized_path == "$.system.skills.*.base"
        && reader_id == Some(NPC_SKILLS_READER)
    {
        Ok(RegisteredAccessor::ActorNpcShadowSkillBase)
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

impl RegisteredAccessor {
    fn capture(
        self,
        identity: SourceLeafIdentity,
        fixture: ResolvedFixture,
    ) -> Result<SourceLeafReceipt, CoverageContractError> {
        match self {
            Self::ItemActionName => capture_item_name(identity, fixture),
            Self::ActorNpcAbilityMod(slot) => capture_actor_npc_ability(identity, fixture, slot),
            Self::ActorNpcShadowSkillBase => capture_actor_npc_shadow_skill(identity, fixture),
        }
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
    let excerpt = serde_json::to_vec(&Value::String(source_name.to_string()))
        .expect("JSON string serialization cannot fail");
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
        ),
        stage(
            FinalOwnerStage::Canonical,
            "AtlasRecord.identity.name",
            "source::normalize::normalize_record",
            actual.canonical,
            &mutation.canonical,
        ),
        stage(
            FinalOwnerStage::PostProjection,
            "IndexBuildInput.records[].identity.name",
            "index_build_input::index_build_input",
            actual.post_projection,
            &mutation.post_projection,
        ),
        stage(
            FinalOwnerStage::ArtifactHydration,
            "RetrievedRecord.record.identity.name",
            "atlas_index::hydrate_record_parts",
            actual.hydration,
            &mutation.hydration,
        ),
        stage(
            FinalOwnerStage::PublicSurface,
            "RecordJson.name",
            "atlas_record::record_json",
            actual.public_surface,
            &mutation.public_surface,
        ),
    ];
    let reader = ActualReadEvidence {
        reader_id: ITEM_NAME_READER.to_string(),
        accessor_binding: "sealed::ItemActionName".to_string(),
        purpose: SourceAccessorPurpose::Parser,
        mutation_digest: digest_serializable(&string_leaf(mutation.source_dto)),
    };
    let semantic_output = None;
    let evidence_digest = evidence_digest(
        &identity,
        &fixture.reference,
        &source,
        &reader,
        &observations,
        &semantic_output,
    );
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
    *mutated_raw
        .pointer_mut(slot.mod_pointer())
        .expect("the source leaf was checked above") = mutation_value;
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
        ),
        presence_stage(
            FinalOwnerStage::Canonical,
            &canonical_destination,
            "source::npc_core::convert_npc_core",
            creature_ability_value(&actual.canonical, slot),
            &creature_ability_value(&mutation.canonical, slot),
        ),
        presence_stage(
            FinalOwnerStage::PostProjection,
            &post_destination,
            "index_build_input::index_build_input",
            creature_ability_value(&actual.post_projection, slot),
            &creature_ability_value(&mutation.post_projection, slot),
        ),
        presence_stage(
            FinalOwnerStage::ArtifactHydration,
            &hydration_destination,
            "atlas_index::hydrate_record_parts",
            creature_ability_value(&actual.hydration, slot),
            &creature_ability_value(&mutation.hydration, slot),
        ),
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
    *mutated_raw
        .pointer_mut(&base_pointer)
        .expect("the shadow-skill base was checked above") = mutation_value;
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
        ),
        presence_stage(
            FinalOwnerStage::Canonical,
            "CreatureRecord.skills[*].source_entries[*].modifier",
            "source::npc_core::convert_npc_core",
            creature_skill_value(&actual.canonical, &skill_key),
            &creature_skill_value(&mutation.canonical, &skill_key),
        ),
        presence_stage(
            FinalOwnerStage::PostProjection,
            "IndexBuildInput.canonical_bodies[].skills[*].source_entries[*].modifier",
            "index_build_input::index_build_input",
            creature_skill_value(&actual.post_projection, &skill_key),
            &creature_skill_value(&mutation.post_projection, &skill_key),
        ),
        presence_stage(
            FinalOwnerStage::ArtifactHydration,
            "RetrievedRecord.body.skills[*].source_entries[*].modifier",
            "atlas_index::hydrate_record_parts",
            creature_skill_value(&actual.hydration, &skill_key),
            &creature_skill_value(&mutation.hydration, &skill_key),
        ),
        presence_stage(
            FinalOwnerStage::PublicSurface,
            "RecordPresentationJson.creature.skills[*].source_entries[*].modifier",
            "atlas_record::record_json",
            public_skill_value(&actual.public_surface, &skill_key),
            &public_skill_value(&mutation.public_surface, &skill_key),
        ),
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
        mutation_digest: digest_serializable(&source_mutation),
    };
    let semantic_output = None;
    let evidence_digest = evidence_digest(
        &identity,
        &fixture,
        &source,
        &reader,
        &observations,
        &semantic_output,
    );
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

#[derive(Debug)]
struct NpcPipeline {
    source_dto: VersionedNpcSource,
    canonical: CreatureRecord,
    post_projection: CreatureRecord,
    hydration: CreatureRecord,
    public_surface: RecordJson,
    #[cfg(test)]
    diagnostic_source_fields: Vec<String>,
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
    let bodies = input
        .canonical_bodies
        .into_iter()
        .map(|body| {
            let RecordBody::Creature(creature) = &body;
            (creature.identity.record_key.clone(), body)
        })
        .collect::<BTreeMap<_, _>>();
    let hydrated = atlas_index::hydrate_record_parts(input.records, bodies)
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
    })
}

fn creature_body(body: Option<&RecordBody>) -> Result<&CreatureRecord, CoverageContractError> {
    match body {
        Some(RecordBody::Creature(creature)) => Ok(creature),
        None => Err(error(
            CoverageFailureCode::CanonicalMismatch,
            "NPC pipeline produced no canonical creature body",
        )),
    }
}

fn validate_actor_excerpt(fixture: &ResolvedFixture) -> Result<(), CoverageContractError> {
    let excerpt = actor_excerpt(&fixture.raw)?;
    if digest_serializable(&excerpt) != fixture.reference.excerpt_digest {
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
    let SourcePresence::Value(abilities) = &source.source.core.abilities else {
        return match &source.source.core.abilities {
            SourcePresence::Missing => SourcePresence::Missing,
            SourcePresence::Null => SourcePresence::Null,
            SourcePresence::Value(_) => unreachable!(),
        };
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
    let SourcePresence::Value(skills) = &source.source.core.skills else {
        return match &source.source.core.skills {
            SourcePresence::Missing => SourcePresence::Missing,
            SourcePresence::Null => SourcePresence::Null,
            SourcePresence::Value(_) => unreachable!(),
        };
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
    let Some(skill) = skills.iter().find(|skill| {
        skill
            .source_entries
            .iter()
            .any(|entry| entry.authored_key == key)
    }) else {
        return SourcePresence::Missing;
    };
    let entry = skill
        .source_entries
        .iter()
        .find(|entry| entry.authored_key == key)
        .expect("skill source entry was selected above");
    skill_fact_leaf(key, &entry.modifier)
}

fn public_skill_value(record: &RecordJson, key: &str) -> SourcePresence<SourceLeafValue> {
    let RecordPresentationJson::Creature { skills, .. } = &record.presentation else {
        return SourcePresence::Missing;
    };
    let Some(skill) = skills.as_ref().and_then(|skills| {
        skills.iter().find(|skill| {
            skill
                .source_entries
                .iter()
                .any(|entry| entry.authored_key == key)
        })
    }) else {
        return SourcePresence::Missing;
    };
    let source_entry = skill
        .source_entries
        .iter()
        .find(|entry| entry.authored_key == key)
        .expect("public skill source entry was selected above");
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
) -> StageObservation {
    StageObservation {
        stage,
        destination: destination.to_string(),
        accessor_binding: accessor_binding.to_string(),
        mutation_digest: digest_serializable(mutation),
        value,
    }
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
        .map(|body| {
            let RecordBody::Creature(creature) = &body;
            (creature.identity.record_key.clone(), body)
        })
        .collect::<BTreeMap<_, _>>();
    let hydrated = atlas_index::hydrate_record_parts(input.records, bodies)
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
        let actual_selector = SourceLeafSelector {
            source_contract_version: PF2E_SOURCE_CONTRACT_VERSION.to_string(),
            document_class: manifest_pack.document_class.clone(),
            type_discriminator: raw_discriminator.to_string(),
            role: SourceDocumentRole::TopLevel,
            parent_context: SourceParentContextSelector::root(),
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
        let source_id = raw.get("_id").and_then(Value::as_str).ok_or_else(|| {
            error(
                CoverageFailureCode::FixtureNotSourceGrounded,
                "pinned fixture has no string _id",
            )
        })?;
        let resolved_record_key = format!("{}:{source_id}", manifest_pack.name);
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
) -> StageObservation {
    StageObservation {
        stage,
        destination: destination.to_string(),
        accessor_binding: accessor_binding.to_string(),
        mutation_digest: digest_serializable(&string_leaf(mutation.to_string())),
        value: string_leaf(value),
    }
}

fn digest_bytes(bytes: &[u8]) -> String {
    format!("sha256:{:x}", Sha256::digest(bytes))
}
fn digest_serializable(value: &impl Serialize) -> String {
    digest_bytes(&serde_json::to_vec(value).expect("source-leaf evidence is serializable"))
}
fn evidence_digest(
    identity: &SourceLeafIdentity,
    fixture: &FixtureReference,
    source: &SourcePresence<SourceLeafValue>,
    reader: &ActualReadEvidence,
    observations: &[StageObservation],
    semantic_output: &Option<SemanticOutputObservation>,
) -> String {
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

    use super::*;
    use crate::source_coverage::{
        ExpectedSourceShape, FinalOwnerContract, FixturePrevalence, MapKeyPolicy, ReaderContract,
        SourceDocumentRole, SourceLeafContract, SourceLeafDisposition, SourceLeafKind,
        SourceLeafSelector, SourceParentContextSelector, SourcePin, SourcePrevalence,
        SurfaceContract, SurfaceDecision, SurfaceDisposition, evaluate_source_leaf_coverage,
        parse_source_leaf_ledger,
    };

    fn promoted() -> SurfaceDecision {
        SurfaceDecision {
            disposition: SurfaceDisposition::Promoted,
            rationale: "selected by this engine fixture".to_string(),
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
    fn removing_exact_intimidate_alias_fails_canonical_hydrated_and_public_parity() {
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
        let skill_key = unsupported_skill_key(&fixture.raw).expect("exact raw alias");
        assert_eq!(skill_key, "intimidate");
        let source = shadow_skill_from_raw(&fixture.raw, &skill_key).expect("authored alias base");
        let mut alias_removed = fixture.raw.clone();
        assert!(
            alias_removed
                .pointer_mut("/system/skills")
                .and_then(Value::as_object_mut)
                .expect("Gray Master skills")
                .remove("intimidate")
                .is_some()
        );
        let source_mutation = SourcePresence::Missing;
        assert!(matches!(source_mutation, SourcePresence::Missing));
        let removed = run_npc_pipeline(&fixture, alias_removed).expect("alias-removed pipeline");
        let observations = vec![
            presence_stage(
                FinalOwnerStage::SourceDto,
                "NpcCoreSource.skills[*].base",
                "source::dto::parse_npc_source",
                dto_shadow_skill_value(&removed.source_dto, &skill_key),
                &source_mutation,
            ),
            presence_stage(
                FinalOwnerStage::Canonical,
                "CreatureRecord.skills[*].source_entries[*].modifier",
                "source::npc_core::convert_npc_core",
                creature_skill_value(&removed.canonical, &skill_key),
                &source_mutation,
            ),
            presence_stage(
                FinalOwnerStage::PostProjection,
                "IndexBuildInput.canonical_bodies[].skills[*].source_entries[*].modifier",
                "index_build_input::index_build_input",
                creature_skill_value(&removed.post_projection, &skill_key),
                &source_mutation,
            ),
            presence_stage(
                FinalOwnerStage::ArtifactHydration,
                "RetrievedRecord.body.skills[*].source_entries[*].modifier",
                "atlas_index::hydrate_record_parts",
                creature_skill_value(&removed.hydration, &skill_key),
                &source_mutation,
            ),
            presence_stage(
                FinalOwnerStage::PublicSurface,
                "RecordPresentationJson.creature.skills[*].source_entries[*].modifier",
                "atlas_record::record_json",
                public_skill_value(&removed.public_surface, &skill_key),
                &source_mutation,
            ),
        ];
        let broken_gray = sealed_receipt(
            identity,
            fixture.reference,
            source,
            source_mutation,
            NPC_SKILLS_READER,
            "sealed::ActorNpcExactAliasRemovalNegative",
            observations,
        )
        .expect("sealed alias-removal negative receipt");

        let mut receipts = Vec::new();
        for (leaf_index, leaf) in ledger.leaves.iter().enumerate() {
            for fixture_index in 0..leaf.fixtures.len() {
                if leaf_index == 6 && fixture_index == 0 {
                    receipts.push(broken_gray.clone());
                } else {
                    receipts.push(
                        capture_registered_source_leaf_receipt(
                            &ledger,
                            leaf_index,
                            fixture_index,
                            &repository,
                        )
                        .expect("authenticated sibling receipt"),
                    );
                }
            }
        }
        let report = evaluate_source_leaf_coverage(&ledger, &receipts);
        assert!(!report.passed);
        let codes = report
            .failures
            .iter()
            .map(|failure| failure.code)
            .collect::<std::collections::BTreeSet<_>>();
        assert_eq!(
            codes,
            [
                CoverageFailureCode::DtoMismatch,
                CoverageFailureCode::CanonicalMismatch,
                CoverageFailureCode::PostProjectionMismatch,
                CoverageFailureCode::ArtifactHydrationMismatch,
                CoverageFailureCode::PublicSurfaceMismatch,
            ]
            .into_iter()
            .collect()
        );
    }
}
