#![deny(unsafe_code)]

use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};

use atlas_domain::{
    ARTIFACT_CONTRACT_VERSION, ARTIFACT_METADATA_TABLE, ARTIFACT_SCHEMA_VERSION,
    EXPECTED_CONTENT_HASH_ALGORITHM, EXPECTED_EMBEDDING_DIMENSIONS,
    EXPECTED_EMBEDDING_DISTANCE_METRIC, EXPECTED_EMBEDDING_DOCUMENT_PREFIX,
    EXPECTED_EMBEDDING_DTYPE, EXPECTED_EMBEDDING_MODEL_ID, EXPECTED_EMBEDDING_MODEL_REVISION,
    EXPECTED_EMBEDDING_NORMALIZATION, EXPECTED_EMBEDDING_POOLING,
    EXPECTED_EMBEDDING_PROVIDER_FAMILY, EXPECTED_EMBEDDING_QUERY_PREFIX,
    EXPECTED_EMBEDDING_TOKENIZER_ID, EXPECTED_FTS_TOKENIZER, EXPECTED_SOURCE_KIND, MetricDomain,
    MetricValueType, PackName, PublicationFamily, RecordFamily, RecordId, RecordKey,
    RemasterLinkSource, TextStatus, TimeKind, TimeUnit, artifact_metadata_keys,
};
use rusqlite::{Connection, OptionalExtension, params};
use serde::Deserialize;
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use thiserror::Error;

const DERIVED_AFFLICTIONS_PACK_NAME: &str = "derived-afflictions";
const DERIVED_AFFLICTIONS_PACK_LABEL: &str = "Derived Afflictions";
const DERIVED_AFFLICTION_INSTANCES_PACK_NAME: &str = "derived-affliction-instances";
const DERIVED_AFFLICTION_INSTANCES_PACK_LABEL: &str = "Derived Affliction Instances";

#[derive(Debug, Error)]
pub enum IngestError {
    #[error("source root is unavailable: {0}")]
    SourceUnavailable(String),
    #[error("source manifest failed to parse: {0}")]
    ManifestParseFailed(String),
    #[error("source record failed to parse: {0}")]
    RecordParseFailed(String),
    #[error("record normalization failed for {path}: {message}")]
    RecordNormalizationFailed { path: String, message: String },
    #[error("source contains no loadable Foundry records")]
    NoRecordsLoaded,
    #[error("artifact write failed: {0}")]
    ArtifactWriteFailed(String),
}

#[derive(Debug, Clone)]
pub struct BuildArtifactOptions {
    pub source_root: PathBuf,
    pub output_path: PathBuf,
    pub manifest_path: Option<PathBuf>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BuildArtifactReport {
    pub output_path: PathBuf,
    pub pack_count: usize,
    pub record_count: usize,
    pub source_signature: String,
    pub diagnostics: IngestDiagnostics,
    pub skipped_records: Vec<SkippedRecord>,
    pub warnings: Vec<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct SourceLoad {
    pub source_signature: String,
    pub packs: Vec<LoadedPack>,
    pub records: Vec<LoadedRecord>,
    pub references: Vec<ReferenceEdge>,
    pub aliases: Vec<RecordAlias>,
    pub remaster_links: Vec<RemasterLink>,
    pub diagnostics: IngestDiagnostics,
    pub skipped_records: Vec<SkippedRecord>,
    pub warnings: Vec<String>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct IngestDiagnostics {
    pub taxonomy_folder_records: usize,
    pub taxonomy_glossary_records: usize,
    pub variant_parenthetical_records: usize,
    pub variant_suffix_records: usize,
    pub variant_creature_blurb_records: usize,
    pub variant_creature_suffix_records: usize,
    pub variant_exact_base_records: usize,
    pub generated_affliction_canonical_records: usize,
    pub generated_affliction_instance_records: usize,
    pub generated_affliction_reference_edges: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SkippedRecord {
    pub path: PathBuf,
    pub reason: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LoadedPack {
    pub name: PackName,
    pub label: String,
    pub document_type: String,
    pub declared_path: String,
    pub resolved_path: PathBuf,
    pub record_count: usize,
}

#[derive(Debug, Clone, PartialEq)]
pub struct LoadedRecord {
    pub key: RecordKey,
    pub id: RecordId,
    pub name: String,
    pub normalized_name: String,
    pub record_family: RecordFamily,
    pub pack_name: PackName,
    pub pack_label: String,
    pub foundry_document_type: String,
    pub foundry_record_type: String,
    pub level: Option<i64>,
    pub rarity: Option<String>,
    pub traits: Vec<String>,
    pub system_category: Option<String>,
    pub system_group: Option<String>,
    pub system_base_item: Option<String>,
    pub system_usage: Option<String>,
    pub system_price_json: Option<String>,
    pub system_actions_value: Option<i64>,
    pub system_time_value: Option<String>,
    pub system_duration_value: Option<String>,
    pub price_cp: Option<i64>,
    pub activation_time: Option<NormalizedTime>,
    pub duration: Option<NormalizedTime>,
    pub metrics: Vec<MetricRow>,
    pub actor_data: Option<ActorSideData>,
    pub item_data: Option<ItemSideData>,
    pub spell_data: Option<SpellSideData>,
    pub publication_title: Option<String>,
    pub publication_remaster: bool,
    pub description_text: Option<String>,
    pub blurb_text: Option<String>,
    pub publication_family: PublicationFamily,
    pub folder_id: Option<String>,
    pub taxonomy_families: Vec<String>,
    pub variant_group_key: Option<String>,
    pub variant_base_name: Option<String>,
    pub variant_label: Option<String>,
    pub variant_axes: Vec<String>,
    pub variant_confidence: Option<f64>,
    pub variant_source: String,
    pub source_path: String,
    pub text_status: TextStatus,
    pub is_default_visible: bool,
    pub search_text_projection: String,
    pub reference_candidates: Vec<ReferenceCandidate>,
    pub raw_json: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReferenceCandidate {
    pub raw_target: String,
    pub display_text: Option<String>,
    pub reference_text: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReferenceEdge {
    pub from_record_key: RecordKey,
    pub to_record_key: RecordKey,
    pub display_text: Option<String>,
    pub reference_text: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum AliasSource {
    RemasterJournal,
    Migration,
    CompendiumSource,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RecordAlias {
    pub canonical_record_key: RecordKey,
    pub alias_text: String,
    pub normalized_alias: String,
    pub source: AliasSource,
    pub source_ref: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RemasterLink {
    pub remaster_record_key: RecordKey,
    pub legacy_record_key: RecordKey,
    pub source: RemasterLinkSource,
    pub source_ref: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ActorSideData {
    pub size: Option<String>,
    pub languages: Vec<String>,
    pub speed_types: Vec<String>,
    pub senses: Vec<String>,
    pub immunities: Vec<String>,
    pub resistances: Vec<String>,
    pub weaknesses: Vec<String>,
    pub disable_text: Option<String>,
    pub disable_skills: Vec<String>,
    pub is_complex: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ItemSideData {
    pub system_category: Option<String>,
    pub system_base_item: Option<String>,
    pub system_group: Option<String>,
    pub system_usage: Option<String>,
    pub price_cp: Option<i64>,
    pub bulk_value: Option<f64>,
    pub hands_requirement: Option<String>,
    pub damage_types: Vec<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct SpellSideData {
    pub traditions: Vec<String>,
    pub spell_kinds: Vec<String>,
    pub range_text: Option<String>,
    pub range_value: Option<f64>,
    pub target_text: Option<String>,
    pub area_type: Option<String>,
    pub area_value: Option<f64>,
    pub save_type: Option<String>,
    pub sustained: bool,
    pub basic_save: bool,
    pub damage_types: Vec<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct MetricRow {
    pub domain: MetricDomain,
    pub key: String,
    pub value: MetricValue,
}

#[derive(Debug, Clone, PartialEq)]
pub enum MetricValue {
    Number(f64),
    Text(String),
    Boolean(bool),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NormalizedTime {
    pub kind: TimeKind,
    pub actions: Option<i64>,
    pub duration_value: Option<i64>,
    pub duration_unit: Option<TimeUnit>,
    pub text: String,
}

#[derive(Debug, Clone, Default)]
struct RecordReferenceIndex {
    by_key: BTreeMap<String, LoadedRecord>,
    by_pack_id: BTreeMap<(String, String), RecordKey>,
    by_pack_name: BTreeMap<(String, String), Vec<RecordKey>>,
    by_name: BTreeMap<String, Vec<RecordKey>>,
}

#[derive(Debug, Clone)]
struct VariantCandidate {
    base_name: String,
    label: Option<String>,
    axes: Vec<String>,
    source: &'static str,
    diagnostic_source: VariantDiagnosticSource,
    confidence: f64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
enum AfflictionFamily {
    Curse,
    Disease,
    Poison,
}

#[derive(Debug, Clone)]
struct AfflictionOccurrence {
    host_record: LoadedRecord,
    source_record: Option<LoadedRecord>,
    source_raw: Option<Value>,
    child_raw: Value,
    family: AfflictionFamily,
    name: String,
    slug: Option<String>,
    traits: Vec<String>,
    linked_names: Vec<String>,
    source_path: String,
    occurrence_ref: String,
    candidate_keys: Vec<String>,
}

#[derive(Debug, Clone)]
struct GeneratedAfflictionBuild {
    records: Vec<LoadedRecord>,
    references: Vec<ReferenceEdge>,
}

struct DerivedAfflictionRecordInput {
    key: RecordKey,
    id: String,
    name: String,
    record_type: &'static str,
    family: AfflictionFamily,
    traits: Vec<String>,
    description_text: Option<String>,
    blurb_text: Option<String>,
    level: Option<i64>,
    rarity: Option<String>,
    publication_title: Option<String>,
    publication_remaster: bool,
    publication_family: PublicationFamily,
    source_path: String,
    is_default_visible: bool,
    search_text_projection: String,
    raw: Value,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum VariantDiagnosticSource {
    Parenthetical,
    Suffix,
    CreatureBlurb,
    CreatureSuffix,
    ExactBase,
}

#[derive(Debug, Deserialize)]
struct FolderDefinition {
    #[serde(rename = "_id")]
    id: Option<String>,
    name: Option<String>,
    folder: Option<String>,
}

#[derive(Debug, Deserialize)]
struct Manifest {
    #[serde(default)]
    packs: Vec<ManifestPack>,
}

#[derive(Debug, Deserialize)]
struct ManifestPack {
    name: String,
    label: String,
    #[serde(rename = "type")]
    document_type: String,
    path: String,
}

#[derive(Debug)]
struct ParsedManifest {
    manifest: Manifest,
    content_hash: String,
}

pub fn build_minimal_artifact(
    options: BuildArtifactOptions,
) -> Result<BuildArtifactReport, IngestError> {
    let source = load_foundry_source(&options.source_root, options.manifest_path.as_deref())?;
    if source.records.is_empty() {
        return Err(IngestError::NoRecordsLoaded);
    }
    write_minimal_artifact(&options.output_path, &source)?;
    Ok(BuildArtifactReport {
        output_path: options.output_path,
        pack_count: source.packs.len(),
        record_count: source.records.len(),
        source_signature: source.source_signature,
        diagnostics: source.diagnostics,
        skipped_records: source.skipped_records,
        warnings: source.warnings,
    })
}

pub fn load_foundry_source(
    source_root: impl AsRef<Path>,
    manifest_path: Option<&Path>,
) -> Result<SourceLoad, IngestError> {
    let source_root = source_root.as_ref();
    if !source_root.is_dir() {
        return Err(IngestError::SourceUnavailable(format!(
            "{} is not a readable directory",
            source_root.display()
        )));
    }

    let manifest_path = manifest_path
        .map(Path::to_path_buf)
        .unwrap_or_else(|| default_manifest_path(source_root));
    let parsed_manifest = parse_manifest(&manifest_path)?;
    let mut packs = Vec::new();
    let mut records = Vec::new();
    let mut skipped_records = Vec::new();
    let mut warnings = Vec::new();
    let mut diagnostics = IngestDiagnostics::default();

    for manifest_pack in parsed_manifest.manifest.packs {
        let resolved_path = resolve_pack_path(source_root, &manifest_pack);
        if !resolved_path.is_dir() {
            warnings.push(format!(
                "Skipping pack {}: {} is not a readable directory.",
                manifest_pack.name,
                resolved_path.display()
            ));
            continue;
        }

        let pack_name = PackName::new(manifest_pack.name.clone()).map_err(|error| {
            IngestError::ManifestParseFailed(format!("invalid pack name: {error}"))
        })?;
        let paths = json_files(&resolved_path)?;
        let record_start = records.len();

        for path in paths {
            let raw = match read_json_record(&path) {
                Ok(raw) => raw,
                Err(error) => {
                    skipped_records.push(SkippedRecord {
                        path: path.clone(),
                        reason: error.to_string(),
                    });
                    warnings.push(error.to_string());
                    continue;
                }
            };
            match normalize_record(&manifest_pack, &pack_name, &path, source_root, raw) {
                Ok(record) => records.push(record),
                Err(error) => {
                    skipped_records.push(SkippedRecord {
                        path: path.clone(),
                        reason: error.to_string(),
                    });
                    warnings.push(error.to_string());
                }
            }
        }

        let record_count = records.len() - record_start;
        if record_count > 0 {
            packs.push(LoadedPack {
                name: pack_name,
                label: manifest_pack.label,
                document_type: manifest_pack.document_type,
                declared_path: manifest_pack.path,
                resolved_path,
                record_count,
            });
        }
    }

    let source_signature = compute_source_signature(
        source_root,
        &manifest_path,
        &parsed_manifest.content_hash,
        &packs,
        &records,
        &skipped_records,
    );

    let reference_index = build_record_reference_index(&records);
    let generated_afflictions = build_generated_afflictions(&records, &reference_index);
    let generated_references = generated_afflictions.references.clone();
    if !generated_afflictions.records.is_empty() {
        let canonical_count = generated_afflictions
            .records
            .iter()
            .filter(|record| record.is_default_visible)
            .count();
        let instance_count = generated_afflictions.records.len() - canonical_count;
        diagnostics.generated_affliction_canonical_records = canonical_count;
        diagnostics.generated_affliction_instance_records = instance_count;
        diagnostics.generated_affliction_reference_edges = generated_afflictions.references.len();
        packs.push(LoadedPack {
            name: PackName::new(DERIVED_AFFLICTIONS_PACK_NAME.to_string()).map_err(|error| {
                IngestError::ManifestParseFailed(format!(
                    "invalid derived affliction pack: {error}"
                ))
            })?,
            label: DERIVED_AFFLICTIONS_PACK_LABEL.to_string(),
            document_type: "Item".to_string(),
            declared_path: "derived://afflictions".to_string(),
            resolved_path: source_root.join("derived-afflictions"),
            record_count: canonical_count,
        });
        packs.push(LoadedPack {
            name: PackName::new(DERIVED_AFFLICTION_INSTANCES_PACK_NAME.to_string()).map_err(
                |error| {
                    IngestError::ManifestParseFailed(format!(
                        "invalid derived affliction instance pack: {error}"
                    ))
                },
            )?,
            label: DERIVED_AFFLICTION_INSTANCES_PACK_LABEL.to_string(),
            document_type: "Item".to_string(),
            declared_path: "derived://affliction-instances".to_string(),
            resolved_path: source_root.join("derived-affliction-instances"),
            record_count: instance_count,
        });
        records.extend(generated_afflictions.records);
    }

    let reference_index = build_record_reference_index(&records);
    assign_taxonomy_families(&mut records, &packs, &reference_index, &mut diagnostics);
    assign_variant_groups(&mut records, &reference_index, &mut diagnostics);
    let mut references = resolve_reference_edges(&records, &reference_index);
    references.extend(generated_references);
    references.sort_by(|left, right| {
        (
            left.from_record_key.to_string(),
            left.to_record_key.to_string(),
            left.reference_text.as_str(),
        )
            .cmp(&(
                right.from_record_key.to_string(),
                right.to_record_key.to_string(),
                right.reference_text.as_str(),
            ))
    });
    references.dedup_by(|left, right| {
        left.from_record_key == right.from_record_key
            && left.to_record_key == right.to_record_key
            && left.reference_text == right.reference_text
    });
    let aliases = resolve_record_aliases(&records, &reference_index, source_root);
    let remaster_links = resolve_remaster_links(&records, &reference_index, source_root);

    Ok(SourceLoad {
        source_signature,
        packs,
        records,
        references,
        aliases,
        remaster_links,
        diagnostics,
        skipped_records,
        warnings,
    })
}

fn resolve_pack_path(source_root: &Path, manifest_pack: &ManifestPack) -> PathBuf {
    let declared_path = manifest_pack.path.trim_start_matches('/');
    let direct = source_root.join(declared_path);
    if direct.is_dir() {
        return direct;
    }

    if let Some(pack_directory) = Path::new(declared_path).file_name() {
        let namespaced = source_root.join("packs").join("pf2e").join(pack_directory);
        if namespaced.is_dir() {
            return namespaced;
        }
    }

    direct
}

fn default_manifest_path(source_root: &Path) -> PathBuf {
    for relative_path in ["system.pf2e.json", "static/system.json", "module.json"] {
        let candidate = source_root.join(relative_path);
        if candidate.is_file() {
            return candidate;
        }
    }
    source_root.join("module.json")
}

fn parse_manifest(path: &Path) -> Result<ParsedManifest, IngestError> {
    let serialized = fs::read_to_string(path)
        .map_err(|error| IngestError::SourceUnavailable(error.to_string()))?;
    let content_hash = sha256_hex(serialized.as_bytes());
    let manifest = serde_json::from_str(&serialized)
        .map_err(|error| IngestError::ManifestParseFailed(error.to_string()))?;
    Ok(ParsedManifest {
        manifest,
        content_hash,
    })
}

fn compute_source_signature(
    source_root: &Path,
    manifest_path: &Path,
    manifest_content_hash: &str,
    packs: &[LoadedPack],
    records: &[LoadedRecord],
    skipped_records: &[SkippedRecord],
) -> String {
    let mut hasher = Sha256::new();
    hash_field(&mut hasher, "atlas-source-signature-v1");
    hash_field(&mut hasher, "manifest");
    hash_field(
        &mut hasher,
        &relative_source_path(source_root, manifest_path),
    );
    hash_field(&mut hasher, manifest_content_hash);

    let mut sorted_packs = packs.iter().collect::<Vec<_>>();
    sorted_packs.sort_by(|left, right| left.name.as_str().cmp(right.name.as_str()));
    for pack in sorted_packs {
        hash_field(&mut hasher, "pack");
        hash_field(&mut hasher, pack.name.as_str());
        hash_field(&mut hasher, &pack.label);
        hash_field(&mut hasher, &pack.document_type);
        hash_field(&mut hasher, &pack.declared_path);
        hash_field(&mut hasher, &pack.record_count.to_string());
    }

    let mut sorted_records = records.iter().collect::<Vec<_>>();
    sorted_records.sort_by(|left, right| {
        (
            left.source_path.as_str(),
            left.pack_name.as_str(),
            left.id.as_str(),
            left.key.to_string(),
        )
            .cmp(&(
                right.source_path.as_str(),
                right.pack_name.as_str(),
                right.id.as_str(),
                right.key.to_string(),
            ))
    });
    for record in sorted_records {
        hash_field(&mut hasher, "record");
        hash_field(&mut hasher, &record.source_path);
        hash_field(&mut hasher, &record.key.to_string());
        hash_field(&mut hasher, &record.name);
        hash_field(&mut hasher, &record.foundry_document_type);
        hash_field(&mut hasher, &record.foundry_record_type);
        hash_field(&mut hasher, &sha256_hex(record.raw_json.as_bytes()));
    }

    let mut sorted_skipped_records = skipped_records.iter().collect::<Vec<_>>();
    sorted_skipped_records.sort_by(|left, right| {
        (
            relative_source_path(source_root, &left.path),
            left.reason.as_str(),
        )
            .cmp(&(
                relative_source_path(source_root, &right.path),
                right.reason.as_str(),
            ))
    });
    for skipped_record in sorted_skipped_records {
        hash_field(&mut hasher, "skipped");
        hash_field(
            &mut hasher,
            &relative_source_path(source_root, &skipped_record.path),
        );
        hash_field(&mut hasher, &skipped_record.reason);
    }

    format!("foundry-pf2e:sha256:{}", hex_lower(hasher.finalize()))
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

fn relative_source_path(source_root: &Path, path: &Path) -> String {
    let relative_path = path.strip_prefix(source_root).ok();
    let fallback_path = if path.is_absolute() {
        path.file_name().map(Path::new).unwrap_or(path)
    } else {
        path
    };
    relative_path
        .unwrap_or(fallback_path)
        .components()
        .map(|component| component.as_os_str().to_string_lossy())
        .collect::<Vec<_>>()
        .join("/")
}

fn json_files(root: &Path) -> Result<Vec<PathBuf>, IngestError> {
    let mut paths = Vec::new();
    collect_json_files(root, &mut paths)?;
    paths.sort();
    Ok(paths)
}

fn collect_json_files(root: &Path, paths: &mut Vec<PathBuf>) -> Result<(), IngestError> {
    for entry in
        fs::read_dir(root).map_err(|error| IngestError::SourceUnavailable(error.to_string()))?
    {
        let entry = entry.map_err(|error| IngestError::SourceUnavailable(error.to_string()))?;
        let path = entry.path();
        if path.is_dir() {
            collect_json_files(&path, paths)?;
        } else if path
            .extension()
            .is_some_and(|extension| extension == "json")
            && path
                .file_name()
                .is_none_or(|file_name| file_name != "_folders.json")
        {
            paths.push(path);
        }
    }
    Ok(())
}

fn read_json_record(path: &Path) -> Result<Value, IngestError> {
    let serialized = fs::read_to_string(path)
        .map_err(|error| IngestError::RecordParseFailed(error.to_string()))?;
    serde_json::from_str(&serialized)
        .map_err(|error| IngestError::RecordParseFailed(format!("{}: {error}", path.display())))
}

fn normalize_record(
    manifest_pack: &ManifestPack,
    pack_name: &PackName,
    path: &Path,
    source_root: &Path,
    raw: Value,
) -> Result<LoadedRecord, IngestError> {
    let id = string_field(&raw, "_id").ok_or_else(|| normalization_error(path, "missing _id"))?;
    let name =
        string_field(&raw, "name").ok_or_else(|| normalization_error(path, "missing name"))?;
    let record_type =
        string_field(&raw, "type").unwrap_or_else(|| manifest_pack.document_type.clone());
    let id = RecordId::new(id)
        .map_err(|error| normalization_error(path, &format!("invalid _id: {error}")))?;
    let key = RecordKey::new(pack_name.clone(), id.clone());
    let normalized_name = normalize_text(&name);
    let record_family =
        classify_record(&manifest_pack.document_type, &record_type).ok_or_else(|| {
            normalization_error(
                path,
                &format!(
                    "unsupported Foundry record taxonomy: {}|{}",
                    manifest_pack.document_type, record_type
                ),
            )
        })?;
    let level = pointer_i64(&raw, "/system/level/value");
    let rarity = normalized_pointer_string(&raw, "/system/traits/rarity");
    let traits = extract_traits(&raw);
    let system_category = normalized_pointer_string(&raw, "/system/category");
    let system_group = normalized_pointer_string(&raw, "/system/group");
    let system_base_item = normalized_pointer_string(&raw, "/system/baseItem");
    let system_usage = normalized_pointer_string(&raw, "/system/usage/value");
    let system_price_json = raw
        .pointer("/system/price/value")
        .map(serde_json::to_string)
        .transpose()
        .map_err(|error| normalization_error(path, &format!("price JSON failed: {error}")))?;
    let system_actions_value = pointer_i64(&raw, "/system/actions/value");
    let system_time_value = normalized_pointer_string(&raw, "/system/time/value");
    let system_duration_value = normalized_pointer_string(&raw, "/system/duration/value");
    let price_cp = normalize_price_cp(raw.pointer("/system/price/value"));
    let activation_time =
        normalize_activation_time(system_actions_value, system_time_value.as_deref());
    let duration = system_duration_value
        .as_deref()
        .and_then(normalize_time_text);
    let metrics = extract_metrics(&raw, &manifest_pack.document_type, &record_type);
    let actor_data =
        (manifest_pack.document_type == "Actor").then(|| extract_actor_side_data(&raw));
    let item_data = (manifest_pack.document_type == "Item").then(|| {
        extract_item_side_data(
            &raw,
            system_category.clone(),
            system_base_item.clone(),
            system_group.clone(),
            system_usage.clone(),
            price_cp,
        )
    });
    let spell_data = (manifest_pack.document_type == "Item" && record_type == "spell")
        .then(|| extract_spell_side_data(&raw, &traits));
    let publication_title = pointer_string(&raw, "/system/publication/title")
        .or_else(|| pointer_string(&raw, "/system/details/publication/title"));
    let publication_remaster = pointer_bool(&raw, "/system/publication/remaster")
        .or_else(|| pointer_bool(&raw, "/system/details/publication/remaster"))
        .unwrap_or(false);
    let description_text =
        pointer_string(&raw, "/system/description/value").map(|value| strip_markup(&value));
    let description_text = description_text.filter(|value| !value.trim().is_empty());
    let blurb_text =
        pointer_string(&raw, "/system/details/blurb").map(|value| strip_markup(&value));
    let blurb_text = blurb_text.filter(|value| !value.trim().is_empty());
    let folder_id = pointer_string(&raw, "/folder");
    let text_status = if description_text.is_some() {
        TextStatus::Resolved
    } else {
        TextStatus::Missing
    };
    let source_path = path
        .strip_prefix(source_root)
        .unwrap_or(path)
        .to_string_lossy()
        .to_string();
    let search_text_projection = create_search_text(&name, description_text.as_deref(), &traits);
    let reference_candidates = extract_reference_candidates(&raw);
    let raw_json = serde_json::to_string(&raw).map_err(|error| {
        normalization_error(path, &format!("raw JSON serialization failed: {error}"))
    })?;
    let publication_family = publication_family(pack_name.as_str(), publication_title.as_deref());

    Ok(LoadedRecord {
        key,
        id,
        name,
        normalized_name,
        record_family,
        pack_name: pack_name.clone(),
        pack_label: manifest_pack.label.clone(),
        foundry_document_type: manifest_pack.document_type.clone(),
        foundry_record_type: record_type,
        level,
        rarity,
        traits,
        system_category,
        system_group,
        system_base_item,
        system_usage,
        system_price_json,
        system_actions_value,
        system_time_value,
        system_duration_value,
        price_cp,
        activation_time,
        duration,
        metrics,
        actor_data,
        item_data,
        spell_data,
        publication_title,
        publication_remaster,
        description_text: description_text.clone(),
        blurb_text,
        publication_family,
        folder_id,
        taxonomy_families: Vec::new(),
        variant_group_key: None,
        variant_base_name: None,
        variant_label: None,
        variant_axes: Vec::new(),
        variant_confidence: None,
        variant_source: "none".to_string(),
        source_path,
        text_status,
        is_default_visible: true,
        search_text_projection,
        reference_candidates,
        raw_json,
    })
}

fn publication_family(pack_name: &str, publication_title: Option<&str>) -> PublicationFamily {
    if is_core_publication(publication_title) {
        return PublicationFamily::Core;
    }
    if is_adventure_publication(publication_title) || is_adventure_pack(pack_name) {
        return PublicationFamily::Adventure;
    }
    if publication_title.is_some_and(|title| !title.trim().is_empty()) {
        return PublicationFamily::Rules;
    }
    PublicationFamily::Unknown
}

fn is_core_publication(publication_title: Option<&str>) -> bool {
    matches!(
        normalize_text(publication_title.unwrap_or_default()).as_str(),
        "pathfinder player core"
            | "player core"
            | "pathfinder player core 2"
            | "player core 2"
            | "pathfinder gm core"
            | "gm core"
            | "pathfinder monster core"
            | "monster core"
            | "pathfinder monster core 2"
            | "monster core 2"
            | "pathfinder beginner box"
    )
}

fn is_adventure_publication(publication_title: Option<&str>) -> bool {
    let normalized = normalize_text(publication_title.unwrap_or_default());
    !normalized.is_empty()
        && (normalized.contains("adventure path")
            || normalized.contains("pathfinder society")
            || normalized.contains("quest")
            || normalized.contains("one shot")
            || normalized.contains("special")
            || normalized.starts_with("pathfinder adventure ")
            || is_pathfinder_numbered_adventure(&normalized))
}

fn is_pathfinder_numbered_adventure(value: &str) -> bool {
    let mut parts = value.split_whitespace();
    matches!(parts.next(), Some("pathfinder"))
        && parts.next().is_some_and(|part| part.parse::<u16>().is_ok())
}

fn is_adventure_pack(pack_name: &str) -> bool {
    let normalized = normalize_text(pack_name);
    normalized.starts_with("pfs ")
        || normalized.contains("one shot")
        || normalized.contains("quest")
}

fn normalization_error(path: &Path, message: &str) -> IngestError {
    IngestError::RecordNormalizationFailed {
        path: path.display().to_string(),
        message: message.to_string(),
    }
}

fn classify_record(document_type: &str, record_type: &str) -> Option<RecordFamily> {
    match (document_type, record_type) {
        ("Actor", "npc") => Some(RecordFamily::Creature),
        ("Actor", "character") => Some(RecordFamily::Character),
        ("Actor", "familiar") => Some(RecordFamily::Companion),
        ("Actor", "army") => Some(RecordFamily::Army),
        ("Actor", "hazard") => Some(RecordFamily::Hazard),
        ("Actor", "vehicle") => Some(RecordFamily::Vehicle),
        (
            "Item",
            "ammo" | "armor" | "backpack" | "consumable" | "equipment" | "kit" | "shield"
            | "treasure" | "weapon",
        ) => Some(RecordFamily::Equipment),
        ("Item", "feat") => Some(RecordFamily::Feat),
        ("Item", "spell") => Some(RecordFamily::Spell),
        ("Item", "affliction" | "affliction-instance") => Some(RecordFamily::Affliction),
        ("Item", "action" | "condition" | "effect") => Some(RecordFamily::Rule),
        ("Item", "ancestry" | "background" | "class" | "heritage") => {
            Some(RecordFamily::CharacterOption)
        }
        ("Item", "deity") | ("JournalEntry", _) | ("JournalEntryPage", _) => {
            Some(RecordFamily::Lore)
        }
        ("Macro", "script") | ("RollTable", _) => Some(RecordFamily::Tooling),
        ("Item", "campaignFeature") => Some(RecordFamily::CampaignFeature),
        _ => None,
    }
}

fn build_record_reference_index(records: &[LoadedRecord]) -> RecordReferenceIndex {
    let mut index = RecordReferenceIndex::default();
    for record in records {
        index.by_key.insert(record.key.to_string(), record.clone());
        index.by_pack_id.insert(
            (
                record.pack_name.as_str().to_string(),
                record.id.as_str().to_string(),
            ),
            record.key.clone(),
        );
        index.by_pack_id.insert(
            (
                normalize_text(record.pack_name.as_str()),
                normalize_text(record.id.as_str()),
            ),
            record.key.clone(),
        );
        index
            .by_pack_name
            .entry((
                record.pack_name.as_str().to_string(),
                record.normalized_name.clone(),
            ))
            .or_default()
            .push(record.key.clone());
        index
            .by_name
            .entry(record.normalized_name.clone())
            .or_default()
            .push(record.key.clone());
    }
    index
}

fn build_generated_afflictions(
    records: &[LoadedRecord],
    index: &RecordReferenceIndex,
) -> GeneratedAfflictionBuild {
    let occurrences = collect_affliction_occurrences(records, index);
    if occurrences.is_empty() {
        return GeneratedAfflictionBuild {
            records: Vec::new(),
            references: Vec::new(),
        };
    }

    let mut occurrences_by_family = BTreeMap::<AfflictionFamily, Vec<AfflictionOccurrence>>::new();
    for occurrence in occurrences {
        occurrences_by_family
            .entry(occurrence.family)
            .or_default()
            .push(occurrence);
    }

    let mut generated_records = Vec::new();
    let mut generated_references = Vec::new();
    for (family, family_occurrences) in occurrences_by_family {
        for cluster in cluster_affliction_occurrences(family_occurrences) {
            let candidate = choose_affliction_authoritative_candidate(&cluster);
            let identity_key = choose_affliction_canonical_identity_key(
                &cluster
                    .iter()
                    .flat_map(|occurrence| occurrence.candidate_keys.clone())
                    .collect::<Vec<_>>(),
            );
            let representative = candidate.0;
            let authoritative_record = candidate.1;
            let authoritative_raw = candidate.2;
            let all_traits = sorted_unique(
                cluster
                    .iter()
                    .flat_map(|occurrence| occurrence.traits.clone())
                    .collect(),
            );
            let all_linked_names = sorted_unique(
                cluster
                    .iter()
                    .flat_map(|occurrence| occurrence.linked_names.clone())
                    .collect(),
            );
            let canonical_id = hash_text(&identity_key);
            let canonical_key = RecordKey::new(
                PackName::new(DERIVED_AFFLICTIONS_PACK_NAME.to_string()).expect("static pack name"),
                RecordId::new(canonical_id.clone()).expect("hash id is valid"),
            );
            let canonical_description_text =
                authoritative_record.and_then(|record| record.description_text.clone());
            let canonical_description_markup =
                authoritative_raw.and_then(record_description_markup);
            let representative_instance_key = cluster.first().map(|occurrence| {
                let instance_id = hash_text(&format!(
                    "{}:{}:{}",
                    identity_key, occurrence.host_record.key, occurrence.occurrence_ref
                ));
                format!("{DERIVED_AFFLICTION_INSTANCES_PACK_NAME}:{instance_id}")
            });
            let canonical_raw = json!({
                "_id": canonical_id,
                "name": representative.name,
                "type": "affliction",
                "system": {
                    "category": affliction_family_label(family),
                    "traits": {
                        "rarity": "common",
                        "value": all_traits,
                    },
                    "description": {
                        "value": canonical_description_markup.unwrap_or_default(),
                    },
                },
                "_derived": {
                    "kind": "canonicalAffliction",
                    "normalizationKey": identity_key,
                    "representativeInstanceRecordKey": representative_instance_key,
                    "linkedNames": all_linked_names,
                    "aliasNormalizationKeys": sorted_unique(
                        cluster.iter().flat_map(|occurrence| occurrence.candidate_keys.clone()).collect()
                    ),
                    "groupKey": affliction_family_label(family),
                }
            });
            let canonical_record = derived_affliction_record(DerivedAfflictionRecordInput {
                key: canonical_key.clone(),
                id: canonical_id,
                name: representative.name.clone(),
                record_type: "affliction",
                family,
                traits: all_traits.clone(),
                description_text: canonical_description_text,
                blurb_text: authoritative_record.and_then(|record| record.blurb_text.clone()),
                level: authoritative_record
                    .and_then(|record| record.level)
                    .or(representative.host_record.level),
                rarity: authoritative_record
                    .and_then(|record| record.rarity.clone())
                    .or_else(|| representative.host_record.rarity.clone()),
                publication_title: authoritative_record
                    .and_then(|record| record.publication_title.clone())
                    .or_else(|| representative.host_record.publication_title.clone()),
                publication_remaster: authoritative_record
                    .map(|record| record.publication_remaster)
                    .unwrap_or(representative.host_record.publication_remaster),
                publication_family: authoritative_record
                    .map(|record| record.publication_family)
                    .unwrap_or(representative.host_record.publication_family),
                source_path: format!("derived://afflictions/{}", canonical_key.id()),
                is_default_visible: true,
                search_text_projection: build_affliction_canonical_search_text(
                    &representative.name,
                    family,
                    &all_traits,
                    representative.slug.as_deref(),
                    &all_linked_names,
                ),
                raw: canonical_raw,
            });
            generated_records.push(canonical_record.clone());

            for occurrence in &cluster {
                let instance_id = hash_text(&format!(
                    "{}:{}:{}",
                    identity_key, occurrence.host_record.key, occurrence.occurrence_ref
                ));
                let instance_key = RecordKey::new(
                    PackName::new(DERIVED_AFFLICTION_INSTANCES_PACK_NAME.to_string())
                        .expect("static pack name"),
                    RecordId::new(instance_id.clone()).expect("hash id is valid"),
                );
                let instance_description_text = record_description_text(&occurrence.child_raw);
                let instance_raw = build_affliction_instance_raw(
                    &instance_id,
                    occurrence,
                    &canonical_key.to_string(),
                    &identity_key,
                );
                let instance_record = derived_affliction_record(DerivedAfflictionRecordInput {
                    key: instance_key.clone(),
                    id: instance_id,
                    name: occurrence.name.clone(),
                    record_type: "affliction-instance",
                    family: occurrence.family,
                    traits: occurrence.traits.clone(),
                    description_text: instance_description_text,
                    blurb_text: None,
                    level: occurrence
                        .source_record
                        .as_ref()
                        .and_then(|record| record.level)
                        .or(occurrence.host_record.level),
                    rarity: occurrence
                        .source_record
                        .as_ref()
                        .and_then(|record| record.rarity.clone())
                        .or_else(|| occurrence.host_record.rarity.clone()),
                    publication_title: occurrence
                        .source_record
                        .as_ref()
                        .and_then(|record| record.publication_title.clone())
                        .or_else(|| occurrence.host_record.publication_title.clone()),
                    publication_remaster: occurrence
                        .source_record
                        .as_ref()
                        .map(|record| record.publication_remaster)
                        .unwrap_or(occurrence.host_record.publication_remaster),
                    publication_family: occurrence.host_record.publication_family,
                    source_path: occurrence.source_path.clone(),
                    is_default_visible: false,
                    search_text_projection: build_affliction_occurrence_search_text(
                        &occurrence.name,
                        occurrence.family,
                        &occurrence.traits,
                        &occurrence.linked_names,
                    ),
                    raw: instance_raw,
                });

                generated_references.push(ReferenceEdge {
                    from_record_key: occurrence.host_record.key.clone(),
                    to_record_key: instance_key.clone(),
                    display_text: Some(occurrence.name.clone()),
                    reference_text: format!("derived-affliction-instance:{instance_key}"),
                });
                generated_references.push(ReferenceEdge {
                    from_record_key: instance_key.clone(),
                    to_record_key: canonical_key.clone(),
                    display_text: Some(occurrence.name.clone()),
                    reference_text: format!("derived-affliction-canonical:{canonical_key}"),
                });
                generated_references.push(ReferenceEdge {
                    from_record_key: canonical_key.clone(),
                    to_record_key: occurrence.host_record.key.clone(),
                    display_text: Some(occurrence.host_record.name.clone()),
                    reference_text: format!(
                        "derived-affliction-host:{}:{instance_key}",
                        occurrence.host_record.key
                    ),
                });
                generated_records.push(instance_record);
            }
        }
    }

    generated_records.sort_by_key(|record| record.key.to_string());
    generated_references.sort_by(|left, right| {
        (
            left.from_record_key.to_string(),
            left.to_record_key.to_string(),
            left.reference_text.as_str(),
        )
            .cmp(&(
                right.from_record_key.to_string(),
                right.to_record_key.to_string(),
                right.reference_text.as_str(),
            ))
    });
    GeneratedAfflictionBuild {
        records: generated_records,
        references: generated_references,
    }
}

fn collect_affliction_occurrences(
    records: &[LoadedRecord],
    index: &RecordReferenceIndex,
) -> Vec<AfflictionOccurrence> {
    let mut occurrences = Vec::new();
    for record in records {
        let Ok(raw) = serde_json::from_str::<Value>(&record.raw_json) else {
            continue;
        };
        if let Some(occurrence) = collect_top_level_affliction_occurrence(record, &raw) {
            occurrences.push(occurrence);
        }
        occurrences.extend(collect_embedded_affliction_occurrences(record, &raw, index));
    }
    occurrences
}

fn collect_top_level_affliction_occurrence(
    record: &LoadedRecord,
    raw: &Value,
) -> Option<AfflictionOccurrence> {
    if record.foundry_record_type == "affliction" {
        return None;
    }
    if !can_generate_affliction_from_raw_type(raw) {
        return None;
    }
    let family = detect_affliction_family(raw)?;
    if !has_affliction_shape(raw) {
        return None;
    }
    let slug = record_slug(raw);
    let compendium_source = compendium_source(raw);
    Some(AfflictionOccurrence {
        host_record: record.clone(),
        source_record: Some(record.clone()),
        source_raw: Some(raw.clone()),
        child_raw: raw.clone(),
        family,
        name: record.name.clone(),
        slug: slug.clone(),
        traits: sorted_unique(
            [
                vec![affliction_family_label(family).to_string()],
                extract_traits(raw),
            ]
            .concat(),
        ),
        linked_names: extract_linked_names_from_markup(record_description_markup(raw).as_deref()),
        source_path: format!("{}#self", record.source_path),
        occurrence_ref: "self".to_string(),
        candidate_keys: build_affliction_occurrence_candidate_keys(
            family,
            &record.name,
            slug.as_deref(),
            compendium_source.as_deref(),
            Some(&record.key.to_string()),
        ),
    })
}

fn collect_embedded_affliction_occurrences(
    record: &LoadedRecord,
    raw: &Value,
    index: &RecordReferenceIndex,
) -> Vec<AfflictionOccurrence> {
    let Some(items) = raw.get("items").and_then(Value::as_array) else {
        return Vec::new();
    };
    let mut occurrences = Vec::new();
    for (item_index, item) in items.iter().enumerate() {
        let family = match detect_affliction_family(item) {
            Some(family)
                if can_generate_affliction_from_raw_type(item) && has_affliction_shape(item) =>
            {
                family
            }
            _ => continue,
        };
        let Some(name) = string_field(item, "name").map(|value| value.trim().to_string()) else {
            continue;
        };
        if name.is_empty() {
            continue;
        }
        let slug = record_slug(item);
        let compendium_source = compendium_source(item);
        let source_record = compendium_source
            .as_deref()
            .and_then(parse_compendium_source)
            .and_then(|(pack_name, id)| {
                index
                    .by_pack_id
                    .get(&(pack_name, id))
                    .and_then(|record_key| record_by_key(index, record_key))
                    .cloned()
            });
        let source_raw = source_record
            .as_ref()
            .and_then(|source_record| serde_json::from_str::<Value>(&source_record.raw_json).ok());
        let child_id = string_field(item, "_id").unwrap_or_else(|| format!("item-{item_index}"));
        occurrences.push(AfflictionOccurrence {
            host_record: record.clone(),
            source_record: source_record.clone(),
            source_raw,
            child_raw: item.clone(),
            family,
            name: name.clone(),
            slug: slug.clone(),
            traits: sorted_unique(
                [
                    vec![affliction_family_label(family).to_string()],
                    extract_traits(item),
                ]
                .concat(),
            ),
            linked_names: extract_linked_names_from_markup(
                record_description_markup(item).as_deref(),
            ),
            source_path: format!("{}#item:{child_id}", record.source_path),
            occurrence_ref: child_id,
            candidate_keys: build_affliction_occurrence_candidate_keys(
                family,
                &name,
                slug.as_deref(),
                compendium_source.as_deref(),
                source_record
                    .as_ref()
                    .map(|record| record.key.to_string())
                    .as_deref(),
            ),
        });
    }
    occurrences
}

fn can_generate_affliction_from_raw_type(raw: &Value) -> bool {
    matches!(
        string_field(raw, "type").as_deref(),
        Some("action" | "consumable" | "spell")
    )
}

fn cluster_affliction_occurrences(
    occurrences: Vec<AfflictionOccurrence>,
) -> Vec<Vec<AfflictionOccurrence>> {
    if occurrences.len() <= 1 {
        return if occurrences.is_empty() {
            Vec::new()
        } else {
            vec![occurrences]
        };
    }

    let mut parent = (0..occurrences.len()).collect::<Vec<_>>();
    for left in 0..occurrences.len() {
        for right in (left + 1)..occurrences.len() {
            if occurrences[left]
                .candidate_keys
                .iter()
                .any(|key| occurrences[right].candidate_keys.contains(key))
            {
                union_parent(&mut parent, left, right);
            }
        }
    }
    let mut clusters = BTreeMap::<usize, Vec<AfflictionOccurrence>>::new();
    for (index, occurrence) in occurrences.into_iter().enumerate() {
        let root = find_parent(&mut parent, index);
        clusters.entry(root).or_default().push(occurrence);
    }
    clusters.into_values().collect()
}

fn find_parent(parent: &mut [usize], index: usize) -> usize {
    if parent[index] != index {
        let root = find_parent(parent, parent[index]);
        parent[index] = root;
    }
    parent[index]
}

fn union_parent(parent: &mut [usize], left: usize, right: usize) {
    let left_root = find_parent(parent, left);
    let right_root = find_parent(parent, right);
    if left_root != right_root {
        parent[right_root] = left_root;
    }
}

fn choose_affliction_authoritative_candidate(
    occurrences: &[AfflictionOccurrence],
) -> (&AfflictionOccurrence, Option<&LoadedRecord>, Option<&Value>) {
    let representative = occurrences
        .iter()
        .min_by(|left, right| {
            right
                .source_record
                .is_some()
                .cmp(&left.source_record.is_some())
                .then_with(|| {
                    left.host_record
                        .key
                        .to_string()
                        .cmp(&right.host_record.key.to_string())
                })
                .then_with(|| left.occurrence_ref.cmp(&right.occurrence_ref))
        })
        .expect("non-empty cluster");
    (
        representative,
        representative.source_record.as_ref(),
        representative.source_raw.as_ref(),
    )
}

fn choose_affliction_canonical_identity_key(candidate_keys: &[String]) -> String {
    let mut keys = sorted_unique(candidate_keys.to_vec());
    keys.sort_by(|left, right| {
        affliction_identity_key_rank(left)
            .cmp(&affliction_identity_key_rank(right))
            .then_with(|| left.cmp(right))
    });
    keys.into_iter().next().unwrap_or_default()
}

fn affliction_identity_key_rank(value: &str) -> u8 {
    if value.starts_with("record:") {
        0
    } else if value.starts_with("compendium:") {
        1
    } else if value.starts_with("slug:") {
        2
    } else {
        3
    }
}

fn derived_affliction_record(input: DerivedAfflictionRecordInput) -> LoadedRecord {
    let id = RecordId::new(input.id).expect("derived id is valid");
    let text_status = if input.description_text.is_some() {
        TextStatus::Resolved
    } else {
        TextStatus::Missing
    };
    let raw_json = serde_json::to_string(&input.raw).expect("derived raw JSON serializes");
    LoadedRecord {
        key: input.key,
        id,
        name: input.name.clone(),
        normalized_name: normalize_text(&input.name),
        record_family: RecordFamily::Affliction,
        pack_name: PackName::new(
            if input.is_default_visible {
                DERIVED_AFFLICTIONS_PACK_NAME
            } else {
                DERIVED_AFFLICTION_INSTANCES_PACK_NAME
            }
            .to_string(),
        )
        .expect("static pack name"),
        pack_label: if input.is_default_visible {
            DERIVED_AFFLICTIONS_PACK_LABEL
        } else {
            DERIVED_AFFLICTION_INSTANCES_PACK_LABEL
        }
        .to_string(),
        foundry_document_type: "Item".to_string(),
        foundry_record_type: input.record_type.to_string(),
        level: input.level,
        rarity: input.rarity,
        traits: input.traits,
        system_category: Some(affliction_family_label(input.family).to_string()),
        system_group: None,
        system_base_item: None,
        system_usage: None,
        system_price_json: None,
        system_actions_value: None,
        system_time_value: None,
        system_duration_value: None,
        price_cp: None,
        activation_time: None,
        duration: None,
        metrics: Vec::new(),
        actor_data: None,
        item_data: None,
        spell_data: None,
        publication_title: input.publication_title,
        publication_remaster: input.publication_remaster,
        description_text: input.description_text,
        blurb_text: input.blurb_text,
        publication_family: input.publication_family,
        folder_id: None,
        taxonomy_families: Vec::new(),
        variant_group_key: None,
        variant_base_name: None,
        variant_label: None,
        variant_axes: Vec::new(),
        variant_confidence: None,
        variant_source: "none".to_string(),
        source_path: input.source_path,
        text_status,
        is_default_visible: input.is_default_visible,
        search_text_projection: input.search_text_projection,
        reference_candidates: Vec::new(),
        raw_json,
    }
}

fn build_affliction_instance_raw(
    instance_id: &str,
    occurrence: &AfflictionOccurrence,
    canonical_record_key: &str,
    normalization_key: &str,
) -> Value {
    let mut raw = occurrence.child_raw.clone();
    if let Value::Object(object) = &mut raw {
        object.insert("_id".to_string(), Value::String(instance_id.to_string()));
        object.insert(
            "_derived".to_string(),
            json!({
                "kind": "afflictionInstance",
                "hostRecordKey": occurrence.host_record.key.to_string(),
                "sourceRecordKey": occurrence.source_record.as_ref().map(|record| record.key.to_string()),
                "canonicalRecordKey": canonical_record_key,
                "normalizationKey": normalization_key,
                "occurrenceRef": occurrence.occurrence_ref,
                "aliasNormalizationKeys": occurrence.candidate_keys,
            }),
        );
    }
    raw
}

fn build_affliction_occurrence_candidate_keys(
    family: AfflictionFamily,
    name: &str,
    slug: Option<&str>,
    compendium_source: Option<&str>,
    source_record_key: Option<&str>,
) -> Vec<String> {
    sorted_unique(
        [
            source_record_key.map(|value| format!("record:{value}")),
            compendium_source.map(|value| format!("compendium:{}", normalize_text(value))),
            slug.map(|value| {
                format!(
                    "slug:{}:{}",
                    affliction_family_label(family),
                    normalize_text(value)
                )
            }),
            Some(format!(
                "name:{}:{}",
                affliction_family_label(family),
                normalize_text(name)
            )),
        ]
        .into_iter()
        .flatten()
        .collect(),
    )
}

fn build_affliction_occurrence_search_text(
    name: &str,
    family: AfflictionFamily,
    traits: &[String],
    linked_names: &[String],
) -> String {
    sorted_unique(
        [
            vec![
                name.to_string(),
                affliction_family_label(family).to_string(),
            ],
            traits.to_vec(),
            linked_names.to_vec(),
        ]
        .concat(),
    )
    .join("\n")
}

fn build_affliction_canonical_search_text(
    name: &str,
    family: AfflictionFamily,
    traits: &[String],
    slug: Option<&str>,
    linked_names: &[String],
) -> String {
    let slug_alias = slug.map(|value| value.replace(['-', '_'], " "));
    sorted_unique(
        [
            vec![
                name.to_string(),
                affliction_family_label(family).to_string(),
            ],
            slug_alias.into_iter().collect(),
            traits.to_vec(),
            linked_names.to_vec(),
        ]
        .concat(),
    )
    .join("\n")
}

fn detect_affliction_family(raw: &Value) -> Option<AfflictionFamily> {
    let traits = extract_traits(raw);
    let system_category =
        normalized_pointer_string(raw, "/system/category").map(|value| normalize_text(&value));
    if traits.iter().any(|trait_value| trait_value == "disease")
        || system_category.as_deref() == Some("disease")
    {
        return Some(AfflictionFamily::Disease);
    }
    if traits.iter().any(|trait_value| trait_value == "poison")
        || system_category.as_deref() == Some("poison")
    {
        return Some(AfflictionFamily::Poison);
    }
    if traits.iter().any(|trait_value| trait_value == "curse")
        || system_category.as_deref() == Some("curse")
    {
        return Some(AfflictionFamily::Curse);
    }
    None
}

fn affliction_family_label(family: AfflictionFamily) -> &'static str {
    match family {
        AfflictionFamily::Curse => "curse",
        AfflictionFamily::Disease => "disease",
        AfflictionFamily::Poison => "poison",
    }
}

fn has_affliction_shape(raw: &Value) -> bool {
    let Some(description) = record_description_text(raw) else {
        return false;
    };
    let normalized = normalize_text(&description);
    normalized.contains("saving throw") && normalized.contains("stage 1")
}

fn record_description_markup(raw: &Value) -> Option<String> {
    [
        "/system/description/value",
        "/system/details/description",
        "/system/details/publicNotes",
        "/system/details/blurb",
    ]
    .into_iter()
    .find_map(|pointer| pointer_string(raw, pointer))
    .filter(|value| !value.trim().is_empty())
}

fn record_description_text(raw: &Value) -> Option<String> {
    record_description_markup(raw)
        .map(|value| strip_markup(&value))
        .filter(|value| !value.trim().is_empty())
}

fn record_slug(raw: &Value) -> Option<String> {
    normalized_pointer_string(raw, "/system/slug")
}

fn compendium_source(raw: &Value) -> Option<String> {
    normalized_pointer_string(raw, "/_stats/compendiumSource")
}

fn parse_compendium_source(value: &str) -> Option<(String, String)> {
    let parts = value.split('.').collect::<Vec<_>>();
    if parts.len() >= 5 && parts.first() == Some(&"Compendium") && parts.get(1) == Some(&"pf2e") {
        return Some((normalize_text(parts.get(2)?), normalize_text(parts.last()?)));
    }
    None
}

fn extract_linked_names_from_markup(markup: Option<&str>) -> Vec<String> {
    let Some(markup) = markup else {
        return Vec::new();
    };
    sorted_unique(
        extract_reference_candidates_from_text(markup)
            .into_iter()
            .filter_map(|candidate| {
                candidate
                    .display_text
                    .or_else(|| fallback_linked_name(&candidate.raw_target))
            })
            .collect(),
    )
}

fn fallback_linked_name(locator: &str) -> Option<String> {
    let tail = locator.split('.').next_back()?.replace(['-', '_'], " ");
    let trimmed = tail.trim();
    (!trimmed.is_empty()).then(|| trimmed.to_string())
}

fn hash_text(value: &str) -> String {
    let mut hash: u32 = 2_166_136_261;
    for byte in value.bytes() {
        hash ^= u32::from(byte);
        hash = hash.wrapping_mul(16_777_619);
    }
    format!("{hash:x}")
}

fn assign_taxonomy_families(
    records: &mut [LoadedRecord],
    packs: &[LoadedPack],
    index: &RecordReferenceIndex,
    diagnostics: &mut IngestDiagnostics,
) {
    let folder_families = load_folder_family_maps(packs);
    let glossary_families = build_glossary_family_map(records);

    for record in records {
        let mut families = BTreeSet::new();
        let mut assigned_from_folder = false;
        let mut assigned_from_glossary = false;
        if let Some(folder_id) = &record.folder_id
            && let Some(folder_family) =
                folder_families.get(&(record.pack_name.as_str().to_string(), folder_id.clone()))
            && should_keep_folder_family(record.pack_name.as_str(), folder_family)
        {
            families.insert(folder_family.clone());
            assigned_from_folder = true;
        }

        for candidate in &record.reference_candidates {
            let Some((pack_name, locator)) = reference_pack_and_locator(&candidate.raw_target)
            else {
                continue;
            };
            if pack_name != "bestiary-family-ability-glossary" {
                continue;
            }
            let Some(record_key) = resolve_record_key(Some(&pack_name), &locator, index) else {
                continue;
            };
            if let Some(family) = glossary_families.get(&record_key.to_string()) {
                families.insert(family.clone());
                assigned_from_glossary = true;
            }
        }

        record.taxonomy_families = families.into_iter().collect();
        diagnostics.taxonomy_folder_records += usize::from(assigned_from_folder);
        diagnostics.taxonomy_glossary_records += usize::from(assigned_from_glossary);
    }
}

fn load_folder_family_maps(packs: &[LoadedPack]) -> BTreeMap<(String, String), String> {
    let mut families = BTreeMap::new();
    for pack in packs {
        let path = pack.resolved_path.join("_folders.json");
        let Ok(serialized) = fs::read_to_string(path) else {
            continue;
        };
        let Ok(raw_folders) = serde_json::from_str::<Vec<FolderDefinition>>(&serialized) else {
            continue;
        };
        let folders_by_id = raw_folders
            .into_iter()
            .filter_map(|folder| folder.id.clone().map(|id| (id, folder)))
            .collect::<BTreeMap<_, _>>();
        for folder_id in folders_by_id.keys() {
            if let Some(family) = resolve_folder_family(folder_id, &folders_by_id) {
                families.insert((pack.name.as_str().to_string(), folder_id.clone()), family);
            }
        }
    }
    families
}

fn resolve_folder_family(
    folder_id: &str,
    folders_by_id: &BTreeMap<String, FolderDefinition>,
) -> Option<String> {
    let mut visited = BTreeSet::new();
    let mut current_id = Some(folder_id.to_string());
    let mut current = None;
    while let Some(id) = current_id {
        if !visited.insert(id.clone()) {
            return None;
        }
        let folder = folders_by_id.get(&id)?;
        current = Some(folder);
        if let Some(parent_id) = folder
            .folder
            .as_ref()
            .filter(|value| !value.trim().is_empty())
        {
            current_id = Some(parent_id.clone());
        } else {
            return normalize_family_name(folder.name.as_deref().unwrap_or_default());
        }
    }
    current.and_then(|folder| normalize_family_name(folder.name.as_deref().unwrap_or_default()))
}

fn should_keep_folder_family(pack_name: &str, family: &str) -> bool {
    const NPC_CORE_FAMILY_ALLOWLIST: &[&str] = &[
        "ancestry-npcs",
        "artisan",
        "courtier",
        "criminal",
        "devotee",
        "downtrodden",
        "engineer",
        "explorer",
        "healer",
        "laborer",
        "martial-artist",
        "maverick",
        "mercenary",
        "military",
        "mystic",
        "official",
        "performer",
        "primalist",
        "scholar",
        "seafarer",
        "villain",
    ];
    pack_name == "pathfinder-npc-core" && NPC_CORE_FAMILY_ALLOWLIST.contains(&family)
}

fn build_glossary_family_map(records: &[LoadedRecord]) -> BTreeMap<String, String> {
    let mut families = BTreeMap::new();
    for record in records {
        if record.pack_name.as_str() != "bestiary-family-ability-glossary" {
            continue;
        }
        if let Some(family) = derive_glossary_family_from_source_path(&record.source_path) {
            families.insert(record.key.to_string(), family);
        }
    }
    families
}

fn derive_glossary_family_from_source_path(source_path: &str) -> Option<String> {
    let mut segments = source_path.split('/').filter(|segment| !segment.is_empty());
    while let Some(segment) = segments.next() {
        if segment == "bestiary-family-ability-glossary" {
            return segments.next().and_then(normalize_family_name);
        }
    }
    None
}

fn normalize_family_name(value: &str) -> Option<String> {
    let family = normalize_text(value).replace(' ', "-");
    (!family.is_empty()).then_some(family)
}

fn assign_variant_groups(
    records: &mut [LoadedRecord],
    index: &RecordReferenceIndex,
    diagnostics: &mut IngestDiagnostics,
) {
    let mut candidates_by_group = BTreeMap::<String, Vec<(usize, VariantCandidate)>>::new();
    let mut base_names_by_group = BTreeMap::<String, String>::new();
    let known_creature_base_names = known_creature_variant_base_names(records);
    for (index_in_records, record) in records.iter().enumerate() {
        let Some(candidate) = variant_candidate(record, index, &known_creature_base_names) else {
            continue;
        };
        let group_key = variant_group_key(record, &candidate.base_name);
        base_names_by_group.insert(group_key.clone(), candidate.base_name.clone());
        candidates_by_group
            .entry(group_key)
            .or_default()
            .push((index_in_records, candidate));
    }

    let mut assigned_indices = BTreeSet::new();
    for (group_key, mut members) in candidates_by_group {
        let Some(base_name) = base_names_by_group.get(&group_key) else {
            continue;
        };
        if let Some(base_index) = exact_base_index(records, &group_key, base_name)
            && !members
                .iter()
                .any(|(member_index, _candidate)| *member_index == base_index)
        {
            members.push((
                base_index,
                VariantCandidate {
                    base_name: base_name.clone(),
                    label: None,
                    axes: Vec::new(),
                    source: "composite",
                    diagnostic_source: VariantDiagnosticSource::ExactBase,
                    confidence: 0.62,
                },
            ));
        }
        members.sort_by_key(|(member_index, _candidate)| *member_index);
        members.dedup_by_key(|(member_index, _candidate)| *member_index);
        if members.len() < 2
            || !members.iter().any(|(_index, candidate)| {
                candidate
                    .label
                    .as_deref()
                    .is_some_and(|label| !label.is_empty())
            })
        {
            continue;
        }
        if members
            .iter()
            .any(|(member_index, _candidate)| assigned_indices.contains(member_index))
        {
            continue;
        }

        let axes = sorted_unique(
            members
                .iter()
                .flat_map(|(_index, candidate)| candidate.axes.clone())
                .collect(),
        );
        let source = if members
            .iter()
            .any(|(_index, candidate)| candidate.source == "composite")
        {
            "composite"
        } else {
            members[0].1.source
        };
        let confidence = members
            .iter()
            .map(|(_index, candidate)| candidate.confidence)
            .fold(0.0_f64, f64::max);

        for (member_index, candidate) in members {
            match candidate.diagnostic_source {
                VariantDiagnosticSource::Parenthetical => {
                    diagnostics.variant_parenthetical_records += 1;
                }
                VariantDiagnosticSource::Suffix => {
                    diagnostics.variant_suffix_records += 1;
                }
                VariantDiagnosticSource::CreatureBlurb => {
                    diagnostics.variant_creature_blurb_records += 1;
                }
                VariantDiagnosticSource::CreatureSuffix => {
                    diagnostics.variant_creature_suffix_records += 1;
                }
                VariantDiagnosticSource::ExactBase => {
                    diagnostics.variant_exact_base_records += 1;
                }
            }
            let record = &mut records[member_index];
            record.variant_group_key = Some(group_key.clone());
            record.variant_base_name = Some(base_name.clone());
            record.variant_label = candidate.label;
            record.variant_axes = axes.clone();
            record.variant_confidence = Some(confidence);
            record.variant_source = source.to_string();
            assigned_indices.insert(member_index);
        }
    }
}

fn variant_candidate(
    record: &LoadedRecord,
    index: &RecordReferenceIndex,
    known_creature_base_names: &BTreeSet<String>,
) -> Option<VariantCandidate> {
    match record.record_family {
        RecordFamily::Creature => {
            parse_creature_variant_candidate(record, index, known_creature_base_names)
                .or_else(|| parse_parenthetical_variant_candidate(&record.name))
        }
        RecordFamily::Equipment | RecordFamily::Spell => {
            parse_parenthetical_variant_candidate(&record.name)
                .or_else(|| parse_trailing_suffix_variant_candidate(&record.name))
        }
        _ => None,
    }
}

fn known_creature_variant_base_names(records: &[LoadedRecord]) -> BTreeSet<String> {
    records
        .iter()
        .filter(|record| record.record_family == RecordFamily::Creature)
        .filter_map(|record| parse_parenthetical_variant_candidate(&record.name))
        .map(|candidate| normalize_text(&candidate.base_name))
        .filter(|base_name| !base_name.is_empty())
        .collect()
}

fn parse_parenthetical_variant_candidate(name: &str) -> Option<VariantCandidate> {
    let mut remainder = name.trim().to_string();
    let mut labels = Vec::new();
    while let Some((base, label)) = split_trailing_parenthetical(&remainder) {
        if base.is_empty() || label.is_empty() {
            break;
        }
        remainder = base;
        labels.insert(0, label);
    }
    if remainder.is_empty() || labels.is_empty() {
        return None;
    }
    Some(VariantCandidate {
        base_name: remainder,
        label: Some(labels.join(", ")),
        axes: infer_variant_axes(&labels),
        source: "namePattern",
        diagnostic_source: VariantDiagnosticSource::Parenthetical,
        confidence: 0.6,
    })
}

fn split_trailing_parenthetical(value: &str) -> Option<(String, String)> {
    let value = value.trim();
    if !value.ends_with(')') {
        return None;
    }
    let open = value.rfind(" (")?;
    let base = value[..open].trim().to_string();
    let label = value[open + 2..value.len() - 1].trim().to_string();
    Some((base, label))
}

fn parse_trailing_suffix_variant_candidate(name: &str) -> Option<VariantCandidate> {
    let words = name.split_whitespace().collect::<Vec<_>>();
    let suffix = words.last()?;
    let normalized_suffix = normalize_text(suffix);
    let label = if is_grade_label(&normalized_suffix) {
        title_case_words(&normalized_suffix)
    } else if is_rank_label(&normalized_suffix) {
        normalize_rank_label(&normalized_suffix)
    } else {
        return None;
    };
    let base_name = words[..words.len() - 1].join(" ").trim().to_string();
    if base_name.is_empty() {
        return None;
    }
    let axes = infer_variant_axes(std::slice::from_ref(&label));
    Some(VariantCandidate {
        base_name,
        label: Some(label),
        axes,
        source: "namePattern",
        diagnostic_source: VariantDiagnosticSource::Suffix,
        confidence: 0.74,
    })
}

fn parse_creature_variant_candidate(
    record: &LoadedRecord,
    index: &RecordReferenceIndex,
    known_creature_base_names: &BTreeSet<String>,
) -> Option<VariantCandidate> {
    parse_creature_blurb_variant_candidate(record, index, known_creature_base_names)
        .or_else(|| parse_creature_suffix_variant_candidate(record, index))
}

fn parse_creature_blurb_variant_candidate(
    record: &LoadedRecord,
    index: &RecordReferenceIndex,
    known_creature_base_names: &BTreeSet<String>,
) -> Option<VariantCandidate> {
    let blurb = record.blurb_text.as_ref()?;
    let tokens = normalize_text(blurb)
        .split_whitespace()
        .map(ToOwned::to_owned)
        .collect::<Vec<_>>();
    if tokens.len() < 2 || tokens.len() > 6 {
        return None;
    }
    let mut label_tokens = Vec::new();
    let mut cursor = 0;
    while let Some(token) = tokens.get(cursor) {
        if is_dragon_age_label(token) || is_specialization_label(token) || is_gender_label(token) {
            label_tokens.push(token.clone());
            cursor += 1;
        } else {
            break;
        }
    }
    if label_tokens.is_empty() {
        return None;
    }
    let base_tokens = tokens[cursor..].to_vec();
    if base_tokens.is_empty() || base_tokens.len() > 3 {
        return None;
    }
    for base_name in creature_base_name_candidates(&base_tokens) {
        let base_record = exact_creature_base_record(index, &base_name);
        if base_record.is_none() && !known_creature_base_names.contains(&normalize_text(&base_name))
        {
            continue;
        }
        if is_gender_only(&label_tokens)
            && base_record.is_some_and(|record| {
                record
                    .traits
                    .iter()
                    .any(|trait_value| trait_value == "humanoid")
            })
        {
            continue;
        }
        let cleaned_labels = label_tokens
            .iter()
            .map(|token| title_case_words(token))
            .collect::<Vec<_>>();
        let label = choose_creature_variant_label(record, &base_name, &cleaned_labels);
        return Some(VariantCandidate {
            base_name,
            label,
            axes: infer_variant_axes(&cleaned_labels),
            source: "composite",
            diagnostic_source: VariantDiagnosticSource::CreatureBlurb,
            confidence: 0.86,
        });
    }
    None
}

fn parse_creature_suffix_variant_candidate(
    record: &LoadedRecord,
    index: &RecordReferenceIndex,
) -> Option<VariantCandidate> {
    const ALLOWLIST: &[(&str, &str)] = &[
        ("ghost", "ghost"),
        ("ghoul", "ghoul"),
        ("wight", "wight"),
        ("wraith", "wraith"),
    ];
    let normalized_name = normalize_text(&record.name);
    for (base, required_trait) in ALLOWLIST {
        if normalized_name == *base || !normalized_name.ends_with(&format!(" {base}")) {
            continue;
        }
        if !record
            .traits
            .iter()
            .any(|trait_value| trait_value == required_trait)
        {
            continue;
        }
        let base_name = title_case_words(base);
        let Some(base_record) = exact_creature_base_record(index, &base_name) else {
            continue;
        };
        if !base_record
            .traits
            .iter()
            .any(|trait_value| trait_value == required_trait)
        {
            continue;
        }
        return Some(VariantCandidate {
            base_name,
            label: Some(record.name.clone()),
            axes: vec!["other".to_string()],
            source: "namePattern",
            diagnostic_source: VariantDiagnosticSource::CreatureSuffix,
            confidence: 0.68,
        });
    }
    None
}

fn exact_creature_base_record<'a>(
    index: &'a RecordReferenceIndex,
    base_name: &str,
) -> Option<&'a LoadedRecord> {
    let matches = index.by_name.get(&normalize_text(base_name))?;
    matches
        .iter()
        .filter_map(|key| record_by_key(index, key))
        .find(|record| record.record_family == RecordFamily::Creature)
}

fn exact_base_index(records: &[LoadedRecord], group_key: &str, base_name: &str) -> Option<usize> {
    records.iter().position(|record| {
        record.name == base_name && variant_group_key(record, base_name) == group_key
    })
}

fn variant_group_key(record: &LoadedRecord, base_name: &str) -> String {
    variant_group_key_for_parts(record.record_family, record.pack_name.as_str(), base_name)
}

fn variant_group_key_for_parts(
    record_family: RecordFamily,
    pack_name: &str,
    base_name: &str,
) -> String {
    if record_family == RecordFamily::Creature {
        format!("creature:family:{}", slugify_hyphen(base_name))
    } else {
        format!(
            "{}:{}:{}",
            record_family.as_str(),
            pack_name,
            slugify_hyphen(base_name)
        )
    }
}

fn creature_base_name_candidates(tokens: &[String]) -> Vec<String> {
    let mut candidates = Vec::new();
    let add = |values: &[String], candidates: &mut Vec<String>| {
        let candidate = title_case_words(&values.join(" "));
        if !candidate.is_empty() && !candidates.contains(&candidate) {
            candidates.push(candidate);
        }
    };
    add(tokens, &mut candidates);
    if let Some(last) = tokens.last()
        && let Some(singular) = singularize_creature_token(last)
    {
        let mut singular_tokens = tokens.to_vec();
        if let Some(last_token) = singular_tokens.last_mut() {
            *last_token = singular;
        }
        add(&singular_tokens, &mut candidates);
    }
    candidates
}

fn singularize_creature_token(token: &str) -> Option<String> {
    if token.len() <= 3 {
        return None;
    }
    if let Some(stem) = token.strip_suffix("ies") {
        return Some(format!("{stem}y"));
    }
    for suffix in ["xes", "ches", "shes", "sses", "zes"] {
        if token.ends_with(suffix) {
            return Some(token[..token.len() - 2].to_string());
        }
    }
    if token.ends_with('s') && !token.ends_with("ss") {
        return Some(token[..token.len() - 1].to_string());
    }
    None
}

fn choose_creature_variant_label(
    record: &LoadedRecord,
    base_name: &str,
    labels: &[String],
) -> Option<String> {
    let explicit = labels.join(", ");
    let normalized_name = normalize_text(&record.name);
    let normalized_base_name = normalize_text(base_name);
    if normalized_name.is_empty() || normalized_name == normalized_base_name {
        return (!explicit.is_empty()).then_some(explicit);
    }
    let generic_only = labels.iter().all(|label| {
        is_specialization_label(&normalize_text(label)) || is_gender_label(&normalize_text(label))
    });
    if generic_only {
        return Some(record.name.clone());
    }
    if normalized_name.contains(&normalized_base_name) && !explicit.is_empty() {
        return Some(explicit);
    }
    Some(record.name.clone())
}

fn infer_variant_axes(labels: &[String]) -> Vec<String> {
    let axes = labels
        .iter()
        .flat_map(|label| {
            let normalized = normalize_text(label);
            if is_rank_label(&normalized) {
                vec!["rank".to_string()]
            } else if is_grade_label(&normalized) {
                vec!["grade".to_string()]
            } else if is_damage_type_label(&normalized) {
                vec!["damageType".to_string()]
            } else if is_dragon_age_label(&normalized) {
                vec!["dragonAge".to_string()]
            } else if is_specialization_label(&normalized) {
                vec!["specialization".to_string()]
            } else if is_gender_label(&normalized) {
                Vec::new()
            } else {
                vec!["other".to_string()]
            }
        })
        .collect::<Vec<_>>();
    let axes = sorted_unique(axes);
    if axes.is_empty() {
        vec!["other".to_string()]
    } else {
        axes
    }
}

fn sorted_unique(mut values: Vec<String>) -> Vec<String> {
    values.sort();
    values.dedup();
    values
}

fn is_grade_label(value: &str) -> bool {
    matches!(
        value,
        "minor" | "lesser" | "moderate" | "greater" | "major" | "true"
    )
}

fn is_damage_type_label(value: &str) -> bool {
    matches!(
        value,
        "acid" | "cold" | "electricity" | "fire" | "poison" | "sonic" | "void" | "vitality"
    )
}

fn is_dragon_age_label(value: &str) -> bool {
    matches!(
        value,
        "wyrmling" | "hatchling" | "young" | "juvenile" | "adult" | "ancient" | "greatwyrm"
    )
}

fn is_specialization_label(value: &str) -> bool {
    matches!(value, "spellcaster" | "elite" | "weak" | "variant")
}

fn is_gender_label(value: &str) -> bool {
    matches!(value, "male" | "female")
}

fn is_gender_only(labels: &[String]) -> bool {
    !labels.is_empty() && labels.iter().all(|label| is_gender_label(label))
}

fn is_rank_label(value: &str) -> bool {
    let normalized = value.replace('-', " ");
    let mut parts = normalized.split_whitespace();
    let Some(ordinal) = parts.next() else {
        return false;
    };
    let Some(kind) = parts.next() else {
        return false;
    };
    is_ordinal(ordinal) && matches!(kind, "rank" | "level")
}

fn is_ordinal(value: &str) -> bool {
    ["st", "nd", "rd", "th"].iter().any(|suffix| {
        value
            .strip_suffix(suffix)
            .is_some_and(|prefix| prefix.parse::<u8>().is_ok())
    })
}

fn normalize_rank_label(value: &str) -> String {
    value
        .replace('-', " ")
        .split_whitespace()
        .enumerate()
        .map(|(index, part)| {
            if index == 0 {
                part.to_string()
            } else {
                title_case_words(part)
            }
        })
        .collect::<Vec<_>>()
        .join(" ")
}

fn title_case_words(value: &str) -> String {
    value
        .split_whitespace()
        .map(|word| {
            let mut chars = word.chars();
            let Some(first) = chars.next() else {
                return String::new();
            };
            format!("{}{}", first.to_uppercase(), chars.as_str().to_lowercase())
        })
        .collect::<Vec<_>>()
        .join(" ")
}

fn slugify_hyphen(value: &str) -> String {
    let mut output = String::new();
    let mut last_was_separator = false;
    for character in normalize_text(value).chars() {
        if character.is_ascii_alphanumeric() {
            output.push(character);
            last_was_separator = false;
        } else if !last_was_separator && !output.is_empty() {
            output.push('-');
            last_was_separator = true;
        }
    }
    while output.ends_with('-') {
        output.pop();
    }
    output
}

fn resolve_reference_edges(
    records: &[LoadedRecord],
    index: &RecordReferenceIndex,
) -> Vec<ReferenceEdge> {
    let mut seen = BTreeSet::new();
    let mut references = Vec::new();
    for record in records {
        for candidate in &record.reference_candidates {
            let Some((pack_name, locator)) = reference_pack_and_locator(&candidate.raw_target)
            else {
                continue;
            };
            let to_record_key = index
                .by_pack_id
                .get(&(pack_name.clone(), locator.clone()))
                .or_else(|| {
                    index
                        .by_pack_name
                        .get(&(pack_name, normalize_text(&locator)))
                        .and_then(|matches| {
                            if matches.len() == 1 {
                                matches.first()
                            } else {
                                None
                            }
                        })
                });
            let Some(to_record_key) = to_record_key else {
                continue;
            };
            let dedupe_key = (
                record.key.to_string(),
                to_record_key.to_string(),
                candidate.reference_text.clone(),
            );
            if seen.insert(dedupe_key) {
                references.push(ReferenceEdge {
                    from_record_key: record.key.clone(),
                    to_record_key: to_record_key.clone(),
                    display_text: candidate.display_text.clone(),
                    reference_text: candidate.reference_text.clone(),
                });
            }
        }
    }

    references.sort_by(|left, right| {
        (
            left.from_record_key.to_string(),
            left.to_record_key.to_string(),
            left.reference_text.as_str(),
        )
            .cmp(&(
                right.from_record_key.to_string(),
                right.to_record_key.to_string(),
                right.reference_text.as_str(),
            ))
    });
    references
}

fn reference_pack_and_locator(raw_target: &str) -> Option<(String, String)> {
    let parts = raw_target.split('.').collect::<Vec<_>>();
    if parts.len() >= 5 && parts.first() == Some(&"Compendium") && parts.get(1) == Some(&"pf2e") {
        return Some((parts.get(2)?.to_string(), parts.last()?.to_string()));
    }
    if parts.len() >= 3 && parts.first() == Some(&"pf2e") {
        return Some((parts.get(1)?.to_string(), parts.last()?.to_string()));
    }
    None
}

fn resolve_record_key(
    pack_name: Option<&str>,
    locator_or_name: &str,
    index: &RecordReferenceIndex,
) -> Option<RecordKey> {
    let normalized = normalize_text(locator_or_name);
    if normalized.is_empty() {
        return None;
    }

    if let Some(pack_name) = pack_name {
        if let Some(record_key) = index
            .by_pack_id
            .get(&(pack_name.to_string(), locator_or_name.to_string()))
        {
            return Some(record_key.clone());
        }

        let matches = index
            .by_pack_name
            .get(&(pack_name.to_string(), normalized.clone()))?;
        return (matches.len() == 1).then(|| matches[0].clone());
    }

    let matches = index.by_name.get(&normalized)?;
    (matches.len() == 1).then(|| matches[0].clone())
}

fn record_by_key<'a>(
    index: &'a RecordReferenceIndex,
    record_key: &RecordKey,
) -> Option<&'a LoadedRecord> {
    index.by_key.get(&record_key.to_string())
}

fn resolve_record_aliases(
    records: &[LoadedRecord],
    index: &RecordReferenceIndex,
    source_root: &Path,
) -> Vec<RecordAlias> {
    let mut aliases = Vec::new();
    for record in records {
        if record.foundry_document_type == "JournalEntry"
            && record.normalized_name == "remaster changes"
        {
            aliases.extend(extract_remaster_journal_aliases(record, index));
        }
        aliases.extend(extract_compendium_source_aliases(record, index));
    }

    aliases.extend(extract_migration_aliases(source_root, index));
    dedupe_record_aliases(aliases)
}

fn extract_remaster_journal_aliases(
    record: &LoadedRecord,
    index: &RecordReferenceIndex,
) -> Vec<RecordAlias> {
    let Ok(raw) = serde_json::from_str::<Value>(&record.raw_json) else {
        return Vec::new();
    };
    let Some(pages) = raw.pointer("/pages").and_then(Value::as_array) else {
        return Vec::new();
    };

    let mut aliases = Vec::new();
    for page in pages {
        let page_name = pointer_string(page, "/name").unwrap_or_else(|| "journal-page".to_string());
        let Some(content) = pointer_string(page, "/text/content") else {
            continue;
        };

        if page_name == "Remaster Changes" {
            for list_item in html_elements(&content, "li") {
                let targets = resolve_journal_targets(&list_item, index);
                if targets.len() != 1 {
                    continue;
                }
                let plain = html_text(&list_item);
                let old_segment = split_remaster_intro_alias_segment(&plain);
                for alias_text in split_alias_list_text(&old_segment) {
                    add_record_alias(
                        &mut aliases,
                        &targets[0],
                        &alias_text,
                        AliasSource::RemasterJournal,
                        &format!("journal:{page_name}"),
                        index,
                    );
                }
            }
        }

        for row in html_elements(&content, "tr") {
            let cells = html_cells(&row);
            if cells.len() < 2 {
                continue;
            }
            let status_cell = if cells.len() >= 4 {
                &cells[2]
            } else {
                "Renamed"
            };
            let status = normalize_text(&html_text(status_cell));
            if !matches!(status.as_str(), "renamed" | "merged" | "replaced") {
                continue;
            }

            let old_cell = &cells[0];
            let new_cell = cells.last().expect("row should have at least two cells");
            let targets = resolve_journal_targets(new_cell, index);
            if targets.is_empty() {
                continue;
            }

            if targets.len() == 1 {
                let Some(old_name) = resolve_alias_source_name(old_cell, index) else {
                    continue;
                };
                add_record_alias(
                    &mut aliases,
                    &targets[0],
                    &old_name,
                    AliasSource::RemasterJournal,
                    &format!("journal:{page_name}"),
                    index,
                );
                continue;
            }

            let old_text = html_text(old_cell);
            let Some(grouped_aliases) = expand_grouped_alias_text(&old_text, targets.len()) else {
                continue;
            };
            for (alias_text, target) in grouped_aliases.iter().zip(targets.iter()) {
                add_record_alias(
                    &mut aliases,
                    target,
                    alias_text,
                    AliasSource::RemasterJournal,
                    &format!("journal:{page_name}"),
                    index,
                );
            }
        }
    }

    aliases
}

fn extract_migration_aliases(source_root: &Path, index: &RecordReferenceIndex) -> Vec<RecordAlias> {
    let mut aliases = Vec::new();
    for (legacy_name, remaster_name) in migration_rename_pairs_from_root(source_root) {
        let Some(remaster_record_key) = resolve_record_key(None, &remaster_name, index) else {
            continue;
        };
        add_record_alias(
            &mut aliases,
            &remaster_record_key,
            &legacy_name,
            AliasSource::Migration,
            "src/module/migration/migrations",
            index,
        );
    }
    aliases
}

fn extract_compendium_source_aliases(
    record: &LoadedRecord,
    index: &RecordReferenceIndex,
) -> Vec<RecordAlias> {
    let Ok(raw) = serde_json::from_str::<Value>(&record.raw_json) else {
        return Vec::new();
    };
    let Some(items) = raw.pointer("/items").and_then(Value::as_array) else {
        return Vec::new();
    };

    let mut aliases = Vec::new();
    for item in items {
        let Some(alias_text) = pointer_string(item, "/name") else {
            continue;
        };
        let Some(compendium_source) = pointer_string(item, "/_stats/compendiumSource") else {
            continue;
        };
        let Some((pack_name, locator)) = reference_pack_and_locator(&compendium_source) else {
            continue;
        };
        let Some(target_record_key) = resolve_record_key(Some(&pack_name), &locator, index) else {
            continue;
        };
        let Some(target_record) = record_by_key(index, &target_record_key) else {
            continue;
        };
        let embedded_remaster = pointer_bool(item, "/system/publication/remaster").unwrap_or(false);
        if embedded_remaster
            || !target_record.publication_remaster
            || should_ignore_compendium_alias(&alias_text, &target_record.name)
        {
            continue;
        }

        add_record_alias(
            &mut aliases,
            &target_record_key,
            &alias_text,
            AliasSource::CompendiumSource,
            &record.key.to_string(),
            index,
        );
    }
    aliases
}

fn add_record_alias(
    aliases: &mut Vec<RecordAlias>,
    canonical_record_key: &RecordKey,
    alias_text: &str,
    source: AliasSource,
    source_ref: &str,
    index: &RecordReferenceIndex,
) {
    let normalized_alias = normalize_text(alias_text);
    if normalized_alias.is_empty() {
        return;
    }
    let Some(canonical_record) = record_by_key(index, canonical_record_key) else {
        return;
    };
    if normalized_alias == canonical_record.normalized_name {
        return;
    }

    aliases.push(RecordAlias {
        canonical_record_key: canonical_record_key.clone(),
        alias_text: alias_text.trim().to_string(),
        normalized_alias,
        source,
        source_ref: source_ref.to_string(),
    });
}

fn should_ignore_compendium_alias(alias_text: &str, target_name: &str) -> bool {
    let normalized_alias = normalize_text(alias_text);
    let normalized_target = normalize_text(target_name);
    if normalized_alias.is_empty() || normalized_alias == normalized_target {
        return true;
    }
    if alias_text
        .chars()
        .any(|character| character.is_ascii_digit())
    {
        return true;
    }
    if contains_any_word(
        &normalized_alias,
        &[
            "feet",
            "foot",
            "mile",
            "miles",
            "precise",
            "imprecise",
            "status",
            "circumstance",
        ],
    ) {
        return true;
    }
    if alias_text.contains('(')
        || alias_text.contains(')')
        || target_name.trim_start().starts_with('(')
    {
        return true;
    }
    false
}

fn contains_any_word(value: &str, words: &[&str]) -> bool {
    value
        .split(|character: char| !character.is_ascii_alphanumeric())
        .any(|word| words.contains(&word))
}

fn dedupe_record_aliases(aliases: Vec<RecordAlias>) -> Vec<RecordAlias> {
    let mut seen = BTreeSet::new();
    let mut deduped = Vec::new();
    for alias in aliases {
        let key = (
            alias.canonical_record_key.to_string(),
            alias.normalized_alias.clone(),
            alias_source_label(alias.source),
            alias.source_ref.clone(),
        );
        if seen.insert(key) {
            deduped.push(alias);
        }
    }
    deduped.sort_by(|left, right| {
        (
            left.canonical_record_key.to_string(),
            left.normalized_alias.as_str(),
            alias_source_label(left.source),
            left.source_ref.as_str(),
        )
            .cmp(&(
                right.canonical_record_key.to_string(),
                right.normalized_alias.as_str(),
                alias_source_label(right.source),
                right.source_ref.as_str(),
            ))
    });
    deduped
}

fn resolve_remaster_links(
    records: &[LoadedRecord],
    index: &RecordReferenceIndex,
    source_root: &Path,
) -> Vec<RemasterLink> {
    let mut links = Vec::new();
    for record in records {
        if record.foundry_document_type != "JournalEntry"
            || record.normalized_name != "remaster changes"
        {
            continue;
        }

        links.extend(extract_remaster_journal_links(record, index));
    }

    links.extend(extract_migration_remaster_links(source_root, index));
    dedupe_remaster_links(links)
}

fn extract_remaster_journal_links(
    record: &LoadedRecord,
    index: &RecordReferenceIndex,
) -> Vec<RemasterLink> {
    let Ok(raw) = serde_json::from_str::<Value>(&record.raw_json) else {
        return Vec::new();
    };
    let Some(pages) = raw.pointer("/pages").and_then(Value::as_array) else {
        return Vec::new();
    };

    let mut links = Vec::new();
    for page in pages {
        let page_name = pointer_string(page, "/name").unwrap_or_else(|| "journal-page".to_string());
        let Some(content) = pointer_string(page, "/text/content") else {
            continue;
        };

        if page_name == "Remaster Changes" {
            for list_item in html_elements(&content, "li") {
                let targets = resolve_journal_targets(&list_item, index);
                if targets.len() != 1 {
                    continue;
                }
                let plain = html_text(&list_item);
                let old_segment = split_remaster_intro_alias_segment(&plain);
                for alias_text in split_alias_list_text(&old_segment) {
                    add_remaster_link(
                        &mut links,
                        &targets[0],
                        &alias_text,
                        RemasterLinkSource::RemasterJournal,
                        &format!("journal:{page_name}"),
                        index,
                    );
                }
            }
        }

        for row in html_elements(&content, "tr") {
            let cells = html_cells(&row);
            if cells.len() < 2 {
                continue;
            }
            let status_cell = if cells.len() >= 4 {
                &cells[2]
            } else {
                "Renamed"
            };
            let status = normalize_text(&html_text(status_cell));
            if !matches!(status.as_str(), "renamed" | "merged" | "replaced") {
                continue;
            }

            let old_cell = &cells[0];
            let new_cell = cells.last().expect("row should have at least two cells");
            let targets = resolve_journal_targets(new_cell, index);
            if targets.is_empty() {
                continue;
            }

            if targets.len() == 1 {
                let Some(old_name) = resolve_alias_source_name(old_cell, index) else {
                    continue;
                };
                add_remaster_link(
                    &mut links,
                    &targets[0],
                    &old_name,
                    RemasterLinkSource::RemasterJournal,
                    &format!("journal:{page_name}"),
                    index,
                );
                continue;
            }

            let old_text = html_text(old_cell);
            let Some(grouped_aliases) = expand_grouped_alias_text(&old_text, targets.len()) else {
                continue;
            };
            for (alias_text, target) in grouped_aliases.iter().zip(targets.iter()) {
                add_remaster_link(
                    &mut links,
                    target,
                    alias_text,
                    RemasterLinkSource::RemasterJournal,
                    &format!("journal:{page_name}"),
                    index,
                );
            }
        }
    }

    links
}

fn extract_migration_remaster_links(
    source_root: &Path,
    index: &RecordReferenceIndex,
) -> Vec<RemasterLink> {
    let mut links = Vec::new();
    for (legacy_name, remaster_name) in migration_rename_pairs_from_root(source_root) {
        let Some(remaster_record_key) = resolve_record_key(None, &remaster_name, index) else {
            continue;
        };
        add_remaster_link(
            &mut links,
            &remaster_record_key,
            &legacy_name,
            RemasterLinkSource::Migration,
            "src/module/migration/migrations",
            index,
        );
    }

    links
}

fn migration_rename_pairs_from_root(source_root: &Path) -> Vec<(String, String)> {
    let migration_root = source_root.join("src/module/migration/migrations");
    let Ok(entries) = fs::read_dir(migration_root) else {
        return Vec::new();
    };

    let mut paths = entries
        .filter_map(Result::ok)
        .map(|entry| entry.path())
        .filter(|path| path.extension().is_some_and(|extension| extension == "ts"))
        .collect::<Vec<_>>();
    paths.sort();

    let mut pairs = Vec::new();
    for path in paths {
        let Ok(source) = fs::read_to_string(path) else {
            continue;
        };
        pairs.extend(migration_rename_pairs(&source));
    }
    pairs
}

fn add_remaster_link(
    links: &mut Vec<RemasterLink>,
    remaster_record_key: &RecordKey,
    legacy_name: &str,
    source: RemasterLinkSource,
    source_ref: &str,
    index: &RecordReferenceIndex,
) {
    let Some(legacy_record_key) = resolve_record_key(None, legacy_name, index) else {
        return;
    };
    if legacy_record_key == *remaster_record_key {
        return;
    }

    let Some(remaster_record) = record_by_key(index, remaster_record_key) else {
        return;
    };
    let Some(legacy_record) = record_by_key(index, &legacy_record_key) else {
        return;
    };
    if !remaster_record.publication_remaster || legacy_record.publication_remaster {
        return;
    }

    links.push(RemasterLink {
        remaster_record_key: remaster_record_key.clone(),
        legacy_record_key,
        source,
        source_ref: source_ref.to_string(),
    });
}

fn dedupe_remaster_links(links: Vec<RemasterLink>) -> Vec<RemasterLink> {
    let mut seen = BTreeSet::new();
    let mut deduped = Vec::new();
    for link in links {
        let key = (
            link.remaster_record_key.to_string(),
            link.legacy_record_key.to_string(),
            remaster_link_source_label(link.source),
            link.source_ref.clone(),
        );
        if seen.insert(key) {
            deduped.push(link);
        }
    }
    deduped.sort_by(|left, right| {
        (
            left.remaster_record_key.to_string(),
            left.legacy_record_key.to_string(),
            remaster_link_source_label(left.source),
            left.source_ref.as_str(),
        )
            .cmp(&(
                right.remaster_record_key.to_string(),
                right.legacy_record_key.to_string(),
                remaster_link_source_label(right.source),
                right.source_ref.as_str(),
            ))
    });
    deduped
}

fn resolve_journal_targets(cell_html: &str, index: &RecordReferenceIndex) -> Vec<RecordKey> {
    let candidates = extract_reference_candidates_from_text(cell_html);
    if candidates.is_empty() {
        let plain = html_text(cell_html);
        return resolve_record_key(None, &plain, index)
            .into_iter()
            .collect();
    }

    let mut targets = Vec::new();
    for candidate in candidates {
        let Some((pack_name, locator)) = reference_pack_and_locator(&candidate.raw_target) else {
            continue;
        };
        let Some(record_key) = resolve_record_key(Some(&pack_name), &locator, index) else {
            continue;
        };
        if record_by_key(index, &record_key)
            .is_some_and(|record| record.foundry_document_type != "JournalEntry")
        {
            targets.push(record_key);
        }
    }
    targets
}

fn resolve_alias_source_name(cell_html: &str, index: &RecordReferenceIndex) -> Option<String> {
    let direct_text = html_text(cell_html);
    if !direct_text.is_empty() && !cell_html.contains("@UUID[") {
        return Some(direct_text);
    }

    let candidate = extract_reference_candidates_from_text(cell_html)
        .into_iter()
        .next()?;
    let (pack_name, locator) = reference_pack_and_locator(&candidate.raw_target)?;
    let record_key = resolve_record_key(Some(&pack_name), &locator, index)?;
    record_by_key(index, &record_key).map(|record| record.name.clone())
}

fn split_remaster_intro_alias_segment(plain_text: &str) -> String {
    for delimiter in [
        " are merged into ",
        " is merged into ",
        " are now ",
        " is now ",
    ] {
        if let Some((segment, _)) = plain_text.split_once(delimiter) {
            return segment.trim().to_string();
        }
    }
    plain_text.trim().to_string()
}

fn split_alias_list_text(value: &str) -> Vec<String> {
    value
        .replace(" and ", ", ")
        .split(',')
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(str::to_string)
        .collect()
}

fn expand_grouped_alias_text(alias_text: &str, expected_count: usize) -> Option<Vec<String>> {
    let open = alias_text.rfind('(')?;
    let close = alias_text.rfind(')')?;
    if close <= open {
        return None;
    }
    let base_name = alias_text[..open].trim();
    let variants = split_alias_list_text(&alias_text[open + 1..close]);
    if base_name.is_empty() || variants.len() != expected_count {
        return None;
    }
    Some(
        variants
            .into_iter()
            .map(|variant| format!("{base_name} ({variant})"))
            .collect(),
    )
}

fn migration_rename_pairs(source: &str) -> Vec<(String, String)> {
    let mut pairs = Vec::new();
    let mut rest = source;
    while let Some(start) = rest.find("Rename all uses and mentions of \"") {
        rest = &rest[start + "Rename all uses and mentions of \"".len()..];
        let Some(old_end) = rest.find('"') else {
            break;
        };
        let legacy_name = rest[..old_end].trim().to_string();
        rest = &rest[old_end + 1..];
        let Some(to_start) = rest.find(" to \"") else {
            continue;
        };
        rest = &rest[to_start + " to \"".len()..];
        let Some(new_end) = rest.find('"') else {
            break;
        };
        let remaster_name = rest[..new_end].trim().to_string();
        rest = &rest[new_end + 1..];
        if !legacy_name.is_empty() && !remaster_name.is_empty() {
            pairs.push((legacy_name, remaster_name));
        }
    }
    pairs
}

fn html_cells(row_html: &str) -> Vec<String> {
    let mut cells = html_elements(row_html, "td");
    cells.extend(html_elements(row_html, "th"));
    cells
}

fn html_elements(html: &str, tag_name: &str) -> Vec<String> {
    let lower = html.to_lowercase();
    let open_prefix = format!("<{tag_name}");
    let close_tag = format!("</{tag_name}>");
    let mut elements = Vec::new();
    let mut offset = 0;
    while let Some(open_relative) = lower[offset..].find(&open_prefix) {
        let open = offset + open_relative;
        let Some(open_end_relative) = lower[open..].find('>') else {
            break;
        };
        let content_start = open + open_end_relative + 1;
        let Some(close_relative) = lower[content_start..].find(&close_tag) else {
            break;
        };
        let close = content_start + close_relative;
        elements.push(html[content_start..close].to_string());
        offset = close + close_tag.len();
    }
    elements
}

fn html_text(value: &str) -> String {
    let mut output = String::new();
    let mut in_tag = false;
    let mut chars = value.chars().peekable();
    while let Some(character) = chars.next() {
        match character {
            '<' => in_tag = true,
            '>' => {
                in_tag = false;
                output.push(' ');
            }
            '@' if !in_tag && chars.peek().is_some_and(|next| *next == 'U') => {
                for next in chars.by_ref() {
                    if next == ']' {
                        break;
                    }
                }
                if chars.peek().is_some_and(|next| *next == '{') {
                    let _ = chars.next();
                    for display in chars.by_ref() {
                        if display == '}' {
                            break;
                        }
                        output.push(display);
                    }
                }
                output.push(' ');
            }
            _ if !in_tag => output.push(character),
            _ => {}
        }
    }
    output.split_whitespace().collect::<Vec<_>>().join(" ")
}

fn extract_reference_candidates(raw: &Value) -> Vec<ReferenceCandidate> {
    let mut candidates = Vec::new();
    collect_reference_candidates(raw, &mut candidates);
    candidates
}

fn collect_reference_candidates(raw: &Value, candidates: &mut Vec<ReferenceCandidate>) {
    match raw {
        Value::Array(values) => {
            for value in values {
                collect_reference_candidates(value, candidates);
            }
        }
        Value::Object(values) => {
            for value in values.values() {
                collect_reference_candidates(value, candidates);
            }
        }
        Value::String(value) => candidates.extend(extract_reference_candidates_from_text(value)),
        _ => {}
    }
}

fn extract_reference_candidates_from_text(text: &str) -> Vec<ReferenceCandidate> {
    let mut candidates = Vec::new();
    let mut offset = 0;

    while offset < text.len() {
        let Some((start, prefix)) = next_reference_prefix(text, offset) else {
            break;
        };
        let target_start = start + prefix.len();
        let Some(close_relative) = text[target_start..].find(']') else {
            break;
        };
        let close = target_start + close_relative;
        let raw_target = text[target_start..close].to_string();
        let mut end = close + 1;
        let mut display_text = None;

        if text[end..].starts_with('{')
            && let Some(display_close_relative) = text[end + 1..].find('}')
        {
            let display_close = end + 1 + display_close_relative;
            let display = strip_markup(&text[end + 1..display_close]);
            if !display.is_empty() {
                display_text = Some(display);
            }
            end = display_close + 1;
        }

        candidates.push(ReferenceCandidate {
            raw_target,
            display_text,
            reference_text: text[start..end].to_string(),
        });
        offset = end;
    }

    candidates
}

fn next_reference_prefix(text: &str, offset: usize) -> Option<(usize, &'static str)> {
    ["@UUID[", "@Compendium["]
        .into_iter()
        .filter_map(|prefix| {
            text[offset..]
                .find(prefix)
                .map(|position| (offset + position, prefix))
        })
        .min_by_key(|(position, _)| *position)
}

fn string_field(raw: &Value, key: &str) -> Option<String> {
    raw.get(key)?.as_str().map(str::to_string)
}

fn pointer_string(raw: &Value, pointer: &str) -> Option<String> {
    raw.pointer(pointer)?.as_str().map(str::to_string)
}

fn normalized_pointer_string(raw: &Value, pointer: &str) -> Option<String> {
    pointer_string(raw, pointer).and_then(|value| {
        let normalized = value.trim();
        if normalized.is_empty() {
            None
        } else {
            Some(normalized.to_string())
        }
    })
}

fn pointer_bool(raw: &Value, pointer: &str) -> Option<bool> {
    raw.pointer(pointer)?.as_bool()
}

fn pointer_i64(raw: &Value, pointer: &str) -> Option<i64> {
    raw.pointer(pointer)?.as_i64()
}

fn normalize_price_cp(value: Option<&Value>) -> Option<i64> {
    let object = value?.as_object()?;
    let platinum = object.get("pp").and_then(Value::as_i64).unwrap_or(0);
    let gold = object.get("gp").and_then(Value::as_i64).unwrap_or(0);
    let silver = object.get("sp").and_then(Value::as_i64).unwrap_or(0);
    let copper = object.get("cp").and_then(Value::as_i64).unwrap_or(0);
    let total = platinum * 1000 + gold * 100 + silver * 10 + copper;
    (total > 0).then_some(total)
}

fn normalize_activation_time(
    system_actions_value: Option<i64>,
    system_time_value: Option<&str>,
) -> Option<NormalizedTime> {
    if let Some(actions) = system_actions_value {
        return Some(NormalizedTime {
            kind: TimeKind::Actions,
            actions: Some(actions),
            duration_value: None,
            duration_unit: None,
            text: actions.to_string(),
        });
    }
    system_time_value.and_then(normalize_time_text)
}

fn normalize_time_text(value: &str) -> Option<NormalizedTime> {
    let text = value.trim();
    if text.is_empty() {
        return None;
    }

    let normalized = normalize_text(text);
    if let Ok(actions) = normalized.parse::<i64>() {
        return Some(NormalizedTime {
            kind: TimeKind::Actions,
            actions: Some(actions),
            duration_value: None,
            duration_unit: None,
            text: text.to_string(),
        });
    }

    match normalized.as_str() {
        "free" | "free action" => {
            return Some(time_with_kind(TimeKind::Free, text));
        }
        "reaction" => {
            return Some(time_with_kind(TimeKind::Reaction, text));
        }
        _ => {}
    }

    if let Some((duration_value, duration_unit)) = parse_duration_unit(&normalized) {
        return Some(NormalizedTime {
            kind: TimeKind::Duration,
            actions: None,
            duration_value: Some(duration_value),
            duration_unit: Some(duration_unit),
            text: text.to_string(),
        });
    }

    let kind = if normalized.contains(" to ")
        || normalized.contains(" or ")
        || normalized.contains("varies")
        || normalized.contains("variable")
    {
        TimeKind::Variable
    } else {
        TimeKind::Other
    };
    Some(time_with_kind(kind, text))
}

fn time_with_kind(kind: TimeKind, text: &str) -> NormalizedTime {
    NormalizedTime {
        kind,
        actions: None,
        duration_value: None,
        duration_unit: None,
        text: text.to_string(),
    }
}

fn parse_duration_unit(value: &str) -> Option<(i64, TimeUnit)> {
    let mut parts = value.split_whitespace();
    let amount = parts.next()?.parse::<i64>().ok()?;
    let unit = canonical_time_unit(parts.next()?)?;
    if parts.next().is_some() {
        return None;
    }
    Some((amount, unit))
}

fn canonical_time_unit(value: &str) -> Option<TimeUnit> {
    match value.trim_end_matches('s') {
        "round" => Some(TimeUnit::Round),
        "minute" => Some(TimeUnit::Minute),
        "hour" => Some(TimeUnit::Hour),
        "day" => Some(TimeUnit::Day),
        "week" => Some(TimeUnit::Week),
        "month" => Some(TimeUnit::Month),
        "year" => Some(TimeUnit::Year),
        _ => None,
    }
}

fn string_array_at_pointer(raw: &Value, pointer: &str) -> Vec<String> {
    let mut values = raw
        .pointer(pointer)
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .filter_map(Value::as_str)
        .map(normalize_text)
        .filter(|value| !value.is_empty())
        .collect::<Vec<_>>();
    values.sort();
    values.dedup();
    values
}

fn extract_speed_types(raw: &Value) -> Vec<String> {
    let mut values = vec!["land".to_string()];
    if let Some(other_speeds) = raw
        .pointer("/system/attributes/speed/otherSpeeds")
        .and_then(Value::as_array)
    {
        values.extend(
            other_speeds
                .iter()
                .filter_map(|speed| normalized_pointer_string(speed, "/type")),
        );
    }
    values.sort();
    values.dedup();
    values
}

fn extract_sense_types(raw: &Value) -> Vec<String> {
    let mut values = raw
        .pointer("/system/perception/senses")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .filter_map(|sense| normalized_pointer_string(sense, "/type"))
        .map(|value| slugify_metric_segment(&value).replace('_', " "))
        .filter(|value| !value.is_empty())
        .collect::<Vec<_>>();
    values.sort();
    values.dedup();
    values
}

fn typed_collection(raw: &Value, pointer: &str) -> Vec<String> {
    let mut values = raw
        .pointer(pointer)
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .filter_map(|entry| normalized_pointer_string(entry, "/type"))
        .collect::<Vec<_>>();
    values.sort();
    values.dedup();
    values
}

fn parse_bulk_value(value: &Value) -> Option<f64> {
    match value {
        Value::Number(number) => number.as_f64(),
        Value::String(text) if text == "L" => Some(0.1),
        Value::String(text) => text.trim().parse::<f64>().ok(),
        _ => None,
    }
}

fn parse_hands_requirement(usage: &str) -> Option<String> {
    if usage.contains("held-in-two-hands") {
        Some("two_hands".to_string())
    } else if usage.contains("held-in-one-plus-hands") {
        Some("one_plus_hands".to_string())
    } else if usage.contains("held-in-one-hand") {
        Some("one_hand".to_string())
    } else {
        None
    }
}

fn extract_damage_types(raw: &Value) -> Vec<String> {
    let mut values = Vec::new();
    if let Some(value) = normalized_pointer_string(raw, "/system/damage/damageType") {
        values.push(value);
    }
    if let Some(entries) = raw
        .pointer("/system/damageRolls")
        .and_then(Value::as_object)
    {
        values.extend(
            entries
                .values()
                .filter_map(|entry| normalized_pointer_string(entry, "/damageType")),
        );
    }
    if let Some(entries) = raw.pointer("/system/damage").and_then(Value::as_object) {
        values.extend(
            entries
                .values()
                .filter_map(|entry| normalized_pointer_string(entry, "/type")),
        );
    }
    values.sort();
    values.dedup();
    values
}

fn extract_disable_skills(raw: &Value) -> Vec<String> {
    let Some(markup) = pointer_string(raw, "/system/details/disable") else {
        return Vec::new();
    };
    let mut skills = Vec::new();
    for skill in [
        "acrobatics",
        "arcana",
        "athletics",
        "crafting",
        "deception",
        "diplomacy",
        "intimidation",
        "medicine",
        "nature",
        "occultism",
        "performance",
        "religion",
        "society",
        "stealth",
        "survival",
        "thievery",
    ] {
        if markup.to_lowercase().contains(skill) {
            skills.push(skill.to_string());
        }
    }
    skills.sort();
    skills.dedup();
    skills
}

fn extract_metrics(raw: &Value, document_type: &str, record_type: &str) -> Vec<MetricRow> {
    let metrics = match document_type {
        "Actor" => extract_actor_metrics(raw),
        "Item" => extract_item_metrics(raw, record_type),
        _ => Vec::new(),
    };
    dedupe_metrics(metrics)
}

fn dedupe_metrics(metrics: Vec<MetricRow>) -> Vec<MetricRow> {
    let mut seen = BTreeSet::new();
    let mut deduped = Vec::new();
    for metric in metrics.into_iter().rev() {
        if seen.insert((metric.domain, metric.key.clone())) {
            deduped.push(metric);
        }
    }
    deduped.reverse();
    deduped
}

fn extract_actor_side_data(raw: &Value) -> ActorSideData {
    let disable_text =
        pointer_string(raw, "/system/details/disable").map(|value| strip_markup(&value));
    ActorSideData {
        size: normalized_pointer_string(raw, "/system/traits/size/value"),
        languages: string_array_at_pointer(raw, "/system/details/languages/value"),
        speed_types: extract_speed_types(raw),
        senses: extract_sense_types(raw),
        immunities: typed_collection(raw, "/system/attributes/immunities"),
        resistances: typed_collection(raw, "/system/attributes/resistances"),
        weaknesses: typed_collection(raw, "/system/attributes/weaknesses"),
        disable_text,
        disable_skills: extract_disable_skills(raw),
        is_complex: pointer_bool(raw, "/system/details/isComplex").unwrap_or(false),
    }
}

fn extract_item_side_data(
    raw: &Value,
    system_category: Option<String>,
    system_base_item: Option<String>,
    system_group: Option<String>,
    system_usage: Option<String>,
    price_cp: Option<i64>,
) -> ItemSideData {
    ItemSideData {
        system_category,
        system_base_item,
        system_group,
        system_usage: system_usage.clone(),
        price_cp,
        bulk_value: raw.pointer("/system/bulk/value").and_then(parse_bulk_value),
        hands_requirement: system_usage.as_deref().and_then(parse_hands_requirement),
        damage_types: extract_damage_types(raw),
    }
}

fn extract_spell_side_data(raw: &Value, traits: &[String]) -> SpellSideData {
    SpellSideData {
        traditions: string_array_at_pointer(raw, "/system/traits/traditions"),
        spell_kinds: ["focus", "ritual", "cantrip"]
            .into_iter()
            .filter(|kind| traits.iter().any(|value| value == kind))
            .map(str::to_string)
            .collect(),
        range_text: normalized_pointer_string(raw, "/system/range/value"),
        range_value: first_number_like_at_paths(
            raw,
            &[
                "/system/range/value",
                "/system/range/increment",
                "/system/area/value",
            ],
        ),
        target_text: pointer_string(raw, "/system/target/value").map(|value| strip_markup(&value)),
        area_type: normalized_pointer_string(raw, "/system/area/type"),
        area_value: number_like_at_pointer(raw, "/system/area/value"),
        save_type: normalized_pointer_string(raw, "/system/defense/save/statistic"),
        sustained: pointer_bool(raw, "/system/duration/sustained").unwrap_or(false),
        basic_save: pointer_bool(raw, "/system/defense/save/basic").unwrap_or(false),
        damage_types: extract_damage_types(raw),
    }
}

fn extract_actor_metrics(raw: &Value) -> Vec<MetricRow> {
    let mut metrics = Vec::new();

    for ability_key in ["str", "dex", "con", "int", "wis", "cha"] {
        add_metric_number(
            &mut metrics,
            MetricDomain::Actor,
            &format!("ability.{ability_key}.mod"),
            first_number_at_paths(
                raw,
                &[
                    &format!("/system/abilities/{ability_key}/mod"),
                    &format!("/system/abilities/{ability_key}/modifier"),
                ],
            ),
        );
    }

    add_metric_number(
        &mut metrics,
        MetricDomain::Actor,
        "perception.mod",
        first_number_at_paths(
            raw,
            &[
                "/system/perception/mod",
                "/system/perception/modifier",
                "/system/perception/value",
            ],
        ),
    );
    add_metric_number(
        &mut metrics,
        MetricDomain::Actor,
        "ac.value",
        number_at_pointer(raw, "/system/attributes/ac/value"),
    );
    add_metric_number(
        &mut metrics,
        MetricDomain::Actor,
        "hardness.value",
        number_at_pointer(raw, "/system/attributes/hardness"),
    );

    for (metric_key, pointer) in [
        ("hp.value", "/system/attributes/hp/value"),
        ("hp.max", "/system/attributes/hp/max"),
        ("hp.bt", "/system/attributes/hp/brokenThreshold"),
        ("hp.bt", "/system/attributes/hp/broken"),
        ("hp.bt", "/system/attributes/hp/bt"),
    ] {
        if metrics.iter().any(|metric| metric.key == metric_key) {
            continue;
        }
        add_metric_number(
            &mut metrics,
            MetricDomain::Actor,
            metric_key,
            number_at_pointer(raw, pointer),
        );
    }

    let save_values = extract_save_metrics(raw, &mut metrics);
    add_best_worst_save_metrics(&mut metrics, &save_values);
    extract_skill_metrics(raw, &mut metrics);
    extract_speed_metrics(raw, &mut metrics);
    extract_sense_metrics(raw, &mut metrics);
    extract_stealth_metrics(raw, &mut metrics);
    metrics
}

fn extract_save_metrics(raw: &Value, metrics: &mut Vec<MetricRow>) -> Vec<(&'static str, f64)> {
    let mut save_values = Vec::new();
    let Some(saves) = raw.pointer("/system/saves").and_then(Value::as_object) else {
        return save_values;
    };

    for (raw_key, value) in saves {
        let Some(save_key) = normalize_save_key(raw_key) else {
            continue;
        };
        let save_value =
            first_number_at_paths(value, &["/mod", "/modifier", "/value", "/totalModifier"]);
        if let Some(number) = save_value {
            add_metric_number(
                metrics,
                MetricDomain::Actor,
                &format!("save.{save_key}.mod"),
                Some(number),
            );
            save_values.push((save_key, number));
        }
    }

    save_values
}

fn add_best_worst_save_metrics(metrics: &mut Vec<MetricRow>, save_values: &[(&'static str, f64)]) {
    let Some((best_save, _)) = save_values
        .iter()
        .max_by(|left, right| left.1.total_cmp(&right.1))
    else {
        return;
    };
    let Some((worst_save, _)) = save_values
        .iter()
        .min_by(|left, right| left.1.total_cmp(&right.1))
    else {
        return;
    };

    metrics.push(MetricRow {
        domain: MetricDomain::Actor,
        key: "save.best".to_string(),
        value: MetricValue::Text((*best_save).to_string()),
    });
    metrics.push(MetricRow {
        domain: MetricDomain::Actor,
        key: "save.worst".to_string(),
        value: MetricValue::Text((*worst_save).to_string()),
    });
}

fn extract_skill_metrics(raw: &Value, metrics: &mut Vec<MetricRow>) {
    let Some(skills) = raw.pointer("/system/skills").and_then(Value::as_object) else {
        return;
    };

    for (raw_key, value) in skills {
        let skill_key = slugify_metric_segment(raw_key);
        if skill_key.is_empty() {
            continue;
        }
        add_metric_number(
            metrics,
            MetricDomain::Actor,
            &format!("skill.{skill_key}.mod"),
            first_number_at_paths(value, &["/mod", "/modifier", "/value"]),
        );
        if let Some(rank) = number_at_pointer(value, "/rank") {
            add_metric_number(
                metrics,
                MetricDomain::Actor,
                &format!("skill.{skill_key}.rank"),
                Some(rank),
            );
            metrics.push(MetricRow {
                domain: MetricDomain::Actor,
                key: format!("skill.{skill_key}.proficient"),
                value: MetricValue::Boolean(rank >= 1.0),
            });
        }
    }
}

fn extract_speed_metrics(raw: &Value, metrics: &mut Vec<MetricRow>) {
    add_metric_number(
        metrics,
        MetricDomain::Actor,
        "speed.land.value",
        number_like_at_pointer(raw, "/system/attributes/speed/value"),
    );

    let Some(other_speeds) = raw
        .pointer("/system/attributes/speed/otherSpeeds")
        .and_then(Value::as_array)
    else {
        return;
    };

    for speed in other_speeds {
        let speed_type = pointer_string(speed, "/type")
            .map(|value| slugify_metric_segment(&value))
            .unwrap_or_default();
        if speed_type.is_empty() {
            continue;
        }
        add_metric_number(
            metrics,
            MetricDomain::Actor,
            &format!("speed.{speed_type}.value"),
            number_like_at_pointer(speed, "/value"),
        );
    }
}

fn extract_sense_metrics(raw: &Value, metrics: &mut Vec<MetricRow>) {
    let Some(senses) = raw
        .pointer("/system/perception/senses")
        .and_then(Value::as_array)
    else {
        return;
    };

    for sense in senses {
        let sense_type = pointer_string(sense, "/type")
            .map(|value| slugify_metric_segment(&value))
            .unwrap_or_default();
        if sense_type.is_empty() {
            continue;
        }
        add_metric_number(
            metrics,
            MetricDomain::Actor,
            &format!("sense.{sense_type}.range"),
            number_like_at_pointer(sense, "/range"),
        );
    }
}

fn extract_stealth_metrics(raw: &Value, metrics: &mut Vec<MetricRow>) {
    let stealth_mod = first_number_at_paths(
        raw,
        &[
            "/system/attributes/stealth/value",
            "/system/attributes/stealth/mod",
            "/system/attributes/stealth/modifier",
        ],
    );
    add_metric_number(metrics, MetricDomain::Actor, "stealth.mod", stealth_mod);
    add_metric_number(
        metrics,
        MetricDomain::Actor,
        "stealth.dc",
        number_at_pointer(raw, "/system/attributes/stealth/dc")
            .or_else(|| stealth_mod.map(|value| value + 10.0)),
    );
}

fn extract_item_metrics(raw: &Value, record_type: &str) -> Vec<MetricRow> {
    let mut metrics = Vec::new();
    match slugify_metric_segment(record_type).as_str() {
        "weapon" => {
            add_metric_number(
                &mut metrics,
                MetricDomain::Item,
                "weapon.range_increment",
                first_number_like_at_paths(
                    raw,
                    &[
                        "/system/range/increment",
                        "/system/range/value",
                        "/system/range",
                    ],
                ),
            );
            add_metric_number(
                &mut metrics,
                MetricDomain::Item,
                "weapon.reload",
                first_number_like_at_paths(raw, &["/system/reload/value", "/system/reload"]),
            );
            add_metric_number(
                &mut metrics,
                MetricDomain::Item,
                "weapon.damage_dice",
                number_at_pointer(raw, "/system/damage/dice"),
            );
            add_metric_number(
                &mut metrics,
                MetricDomain::Item,
                "weapon.damage_die_faces",
                damage_die_faces(raw.pointer("/system/damage/die")),
            );
        }
        "armor" => {
            for (metric_key, pointer) in [
                ("armor.ac_bonus", "/system/acBonus"),
                ("armor.dex_cap", "/system/dexCap"),
                ("armor.strength", "/system/strength"),
                ("armor.check_penalty", "/system/checkPenalty"),
                ("armor.speed_penalty", "/system/speedPenalty"),
            ] {
                add_metric_number(
                    &mut metrics,
                    MetricDomain::Item,
                    metric_key,
                    number_at_pointer(raw, pointer),
                );
            }
        }
        "shield" => {
            for (metric_key, pointer) in [
                ("shield.ac_bonus", "/system/acBonus"),
                ("shield.hardness", "/system/hardness"),
                ("shield.hp", "/system/hp/value"),
                ("shield.hp", "/system/hp/max"),
                ("shield.bt", "/system/hp/brokenThreshold"),
                ("shield.bt", "/system/hp/broken"),
                ("shield.bt", "/system/hp/bt"),
            ] {
                if metrics.iter().any(|metric| metric.key == metric_key) {
                    continue;
                }
                add_metric_number(
                    &mut metrics,
                    MetricDomain::Item,
                    metric_key,
                    number_at_pointer(raw, pointer),
                );
            }
        }
        _ => {}
    }
    metrics
}

fn add_metric_number(
    metrics: &mut Vec<MetricRow>,
    domain: MetricDomain,
    key: &str,
    value: Option<f64>,
) {
    let Some(value) = value.filter(|value| value.is_finite()) else {
        return;
    };
    metrics.push(MetricRow {
        domain,
        key: key.to_string(),
        value: MetricValue::Number(value),
    });
}

fn first_number_at_paths(raw: &Value, pointers: &[&str]) -> Option<f64> {
    pointers
        .iter()
        .find_map(|pointer| number_at_pointer(raw, pointer))
}

fn first_number_like_at_paths(raw: &Value, pointers: &[&str]) -> Option<f64> {
    pointers
        .iter()
        .find_map(|pointer| number_like_at_pointer(raw, pointer))
}

fn number_at_pointer(raw: &Value, pointer: &str) -> Option<f64> {
    raw.pointer(pointer).and_then(value_as_f64)
}

fn number_like_at_pointer(raw: &Value, pointer: &str) -> Option<f64> {
    raw.pointer(pointer).and_then(parse_numeric_like_value)
}

fn value_as_f64(value: &Value) -> Option<f64> {
    match value {
        Value::Number(number) => number.as_f64(),
        Value::String(text) => text.trim().parse::<f64>().ok(),
        _ => None,
    }
}

fn parse_numeric_like_value(value: &Value) -> Option<f64> {
    if let Some(number) = value_as_f64(value) {
        return Some(number);
    }
    let Value::String(text) = value else {
        return None;
    };
    let mut buffer = String::new();
    for character in text.chars() {
        if character.is_ascii_digit() || character == '.' || character == '-' {
            buffer.push(character);
        } else if !buffer.is_empty() {
            break;
        }
    }
    buffer.parse::<f64>().ok()
}

fn damage_die_faces(value: Option<&Value>) -> Option<f64> {
    match value? {
        Value::Number(number) => number.as_f64(),
        Value::String(text) => text
            .trim()
            .strip_prefix('d')
            .or_else(|| text.trim().strip_prefix('D'))
            .and_then(|faces| faces.parse::<f64>().ok()),
        _ => None,
    }
}

fn normalize_save_key(value: &str) -> Option<&'static str> {
    match slugify_metric_segment(value).as_str() {
        "fort" | "fortitude" => Some("fort"),
        "ref" | "reflex" => Some("ref"),
        "will" => Some("will"),
        _ => None,
    }
}

fn slugify_metric_segment(value: &str) -> String {
    let mut output = String::new();
    let mut last_was_separator = false;
    for character in value.trim().to_lowercase().chars() {
        if character.is_ascii_alphanumeric() {
            output.push(character);
            last_was_separator = false;
        } else if !last_was_separator && !output.is_empty() {
            output.push('_');
            last_was_separator = true;
        }
    }
    while output.ends_with('_') {
        output.pop();
    }
    output
}

fn extract_traits(raw: &Value) -> Vec<String> {
    let mut traits = raw
        .pointer("/system/traits/value")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .filter_map(Value::as_str)
        .map(normalize_text)
        .filter(|value| !value.is_empty())
        .collect::<Vec<_>>();
    traits.sort();
    traits.dedup();
    traits
}

fn normalize_text(value: &str) -> String {
    value
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
        .to_lowercase()
}

fn strip_markup(value: &str) -> String {
    let mut output = String::new();
    let mut in_tag = false;
    let mut chars = value.chars().peekable();
    while let Some(character) = chars.next() {
        match character {
            '<' => in_tag = true,
            '>' => {
                in_tag = false;
                output.push(' ');
            }
            '@' if chars.peek().is_some_and(|next| *next == 'U') => output.push(' '),
            _ if !in_tag => output.push(character),
            _ => {}
        }
    }
    output.split_whitespace().collect::<Vec<_>>().join(" ")
}

fn create_search_text(name: &str, description: Option<&str>, traits: &[String]) -> String {
    [Some(name), description, Some(&traits.join(" "))]
        .into_iter()
        .flatten()
        .filter(|value| !value.trim().is_empty())
        .collect::<Vec<_>>()
        .join("\n")
}

pub fn write_minimal_artifact(path: &Path, source: &SourceLoad) -> Result<(), IngestError> {
    let _ = fs::remove_file(path);
    if let Some(parent) = path.parent()
        && !parent.as_os_str().is_empty()
    {
        fs::create_dir_all(parent)
            .map_err(|error| IngestError::ArtifactWriteFailed(error.to_string()))?;
    }

    let mut connection = Connection::open(path)
        .map_err(|error| IngestError::ArtifactWriteFailed(error.to_string()))?;
    let transaction = connection
        .transaction()
        .map_err(|error| IngestError::ArtifactWriteFailed(error.to_string()))?;
    create_minimal_schema(&transaction)?;
    write_artifact_metadata(&transaction, source.records.len(), &source.source_signature)?;
    write_packs(&transaction, &source.packs)?;
    write_records(&transaction, &source.records, &source.remaster_links)?;
    write_reference_edges(&transaction, &source.references)?;
    write_record_aliases(&transaction, &source.aliases)?;
    write_remaster_links(&transaction, &source.remaster_links)?;
    write_metric_catalogs(&transaction)?;
    transaction
        .commit()
        .map_err(|error| IngestError::ArtifactWriteFailed(error.to_string()))
}

fn create_minimal_schema(connection: &Connection) -> Result<(), IngestError> {
    connection
        .execute_batch(
            "
            PRAGMA foreign_keys = ON;

            CREATE TABLE artifact_metadata (
              key TEXT PRIMARY KEY,
              value TEXT NOT NULL
            );

            CREATE TABLE packs (
              name TEXT PRIMARY KEY,
              label TEXT NOT NULL,
              document_type TEXT NOT NULL,
              declared_path TEXT NOT NULL,
              resolved_path TEXT NOT NULL,
              record_count INTEGER NOT NULL
            );

            CREATE TABLE records (
              record_key TEXT PRIMARY KEY,
              id TEXT NOT NULL,
              name TEXT NOT NULL,
              normalized_name TEXT NOT NULL,
              record_family TEXT NOT NULL,
              pack_name TEXT NOT NULL,
              pack_label TEXT NOT NULL,
              foundry_document_type TEXT NOT NULL,
              foundry_record_type TEXT NOT NULL,
              level INTEGER,
              rarity TEXT,
              traits_json TEXT NOT NULL,
              system_category TEXT,
              system_group TEXT,
              system_base_item TEXT,
              system_usage TEXT,
              system_price_json TEXT,
              system_actions_value INTEGER,
              system_time_value TEXT,
              system_duration_value TEXT,
              price_cp INTEGER,
              activation_time_kind TEXT,
              activation_time_actions INTEGER,
              activation_time_duration_value INTEGER,
              activation_time_duration_unit TEXT,
              activation_time_text TEXT,
              duration_kind TEXT,
              duration_value INTEGER,
              duration_unit TEXT,
              duration_text TEXT,
              publication_title TEXT,
              publication_remaster INTEGER NOT NULL CHECK (publication_remaster IN (0, 1)),
              description_text TEXT,
              blurb_text TEXT,
              description_snippet TEXT,
              publication_family TEXT NOT NULL,
              folder_id TEXT,
              taxonomy_families_json TEXT NOT NULL,
              variant_group_key TEXT,
              variant_base_name TEXT,
              variant_label TEXT,
              variant_axes_json TEXT NOT NULL,
              variant_confidence REAL,
              variant_source TEXT NOT NULL,
              source_path TEXT NOT NULL,
              is_default_visible INTEGER NOT NULL CHECK (is_default_visible IN (0, 1)),
              search_text_projection TEXT NOT NULL,
              raw_json TEXT NOT NULL
            );

            CREATE TABLE record_traits (
              record_key TEXT NOT NULL,
              trait TEXT NOT NULL,
              PRIMARY KEY (record_key, trait),
              FOREIGN KEY (record_key) REFERENCES records(record_key) ON DELETE CASCADE
            );

            CREATE TABLE reference_edges (
              from_record_key TEXT NOT NULL,
              to_record_key TEXT NOT NULL,
              display_text TEXT,
              reference_text TEXT NOT NULL,
              PRIMARY KEY (from_record_key, to_record_key, reference_text),
              FOREIGN KEY (from_record_key) REFERENCES records(record_key) ON DELETE CASCADE,
              FOREIGN KEY (to_record_key) REFERENCES records(record_key) ON DELETE CASCADE
            );

            CREATE TABLE record_aliases (
              canonical_record_key TEXT NOT NULL,
              alias_text TEXT NOT NULL,
              normalized_alias TEXT NOT NULL,
              source_kind TEXT NOT NULL CHECK (source_kind IN ('remaster_journal', 'migration', 'compendium_source')),
              source_ref TEXT NOT NULL,
              PRIMARY KEY (canonical_record_key, normalized_alias, source_kind, source_ref),
              FOREIGN KEY (canonical_record_key) REFERENCES records(record_key) ON DELETE CASCADE
            );

            CREATE TABLE remaster_links (
              remaster_record_key TEXT NOT NULL,
              legacy_record_key TEXT NOT NULL,
              source_kind TEXT NOT NULL CHECK (source_kind IN ('remaster_journal', 'migration')),
              source_ref TEXT NOT NULL,
              PRIMARY KEY (remaster_record_key, legacy_record_key, source_kind, source_ref),
              FOREIGN KEY (remaster_record_key) REFERENCES records(record_key) ON DELETE CASCADE,
              FOREIGN KEY (legacy_record_key) REFERENCES records(record_key) ON DELETE CASCADE
            );

            CREATE TABLE record_metrics (
              record_key TEXT NOT NULL,
              metric_domain TEXT NOT NULL CHECK (metric_domain IN ('actor', 'item')),
              metric_key TEXT NOT NULL,
              value_type TEXT NOT NULL CHECK (value_type IN ('number', 'text', 'boolean')),
              number_value REAL,
              text_value TEXT,
              bool_value INTEGER CHECK (bool_value IN (0, 1)),
              PRIMARY KEY (record_key, metric_domain, metric_key),
              FOREIGN KEY (record_key) REFERENCES records(record_key) ON DELETE CASCADE
            );

            CREATE TABLE metric_key_catalog (
              metric_domain TEXT NOT NULL CHECK (metric_domain IN ('actor', 'item')),
              record_family TEXT NOT NULL,
              namespace_prefix TEXT NOT NULL,
              metric_key TEXT NOT NULL,
              value_type TEXT NOT NULL CHECK (value_type IN ('number', 'text', 'boolean')),
              catalog_count INTEGER NOT NULL,
              numeric_min REAL,
              numeric_max REAL,
              PRIMARY KEY (metric_domain, record_family, metric_key)
            );

            CREATE TABLE metric_value_catalog (
              metric_domain TEXT NOT NULL CHECK (metric_domain IN ('actor', 'item')),
              record_family TEXT NOT NULL,
              metric_key TEXT NOT NULL,
              value TEXT NOT NULL,
              catalog_count INTEGER NOT NULL,
              PRIMARY KEY (metric_domain, record_family, metric_key, value)
            );

            CREATE TABLE actor_records (
              record_key TEXT PRIMARY KEY,
              size TEXT,
              languages_json TEXT NOT NULL,
              speed_types_json TEXT NOT NULL,
              senses_json TEXT NOT NULL,
              immunities_json TEXT NOT NULL,
              resistances_json TEXT NOT NULL,
              weaknesses_json TEXT NOT NULL,
              disable_text TEXT,
              disable_skills_json TEXT NOT NULL,
              is_complex INTEGER NOT NULL CHECK (is_complex IN (0, 1)),
              FOREIGN KEY (record_key) REFERENCES records(record_key) ON DELETE CASCADE
            );

            CREATE TABLE item_records (
              record_key TEXT PRIMARY KEY,
              system_category TEXT,
              system_base_item TEXT,
              system_group TEXT,
              system_usage TEXT,
              price_cp INTEGER,
              bulk_value REAL,
              hands_requirement TEXT,
              damage_types_json TEXT NOT NULL,
              FOREIGN KEY (record_key) REFERENCES records(record_key) ON DELETE CASCADE
            );

            CREATE TABLE spell_records (
              record_key TEXT PRIMARY KEY,
              traditions_json TEXT NOT NULL,
              spell_kinds_json TEXT NOT NULL,
              range_text TEXT,
              range_value REAL,
              target_text TEXT,
              area_type TEXT,
              area_value REAL,
              save_type TEXT,
              sustained INTEGER NOT NULL CHECK (sustained IN (0, 1)),
              basic_save INTEGER NOT NULL CHECK (basic_save IN (0, 1)),
              damage_types_json TEXT NOT NULL,
              FOREIGN KEY (record_key) REFERENCES records(record_key) ON DELETE CASCADE
            );

            CREATE VIRTUAL TABLE records_fts USING fts5(
              record_key UNINDEXED,
              name,
              search_text_projection
            );

            CREATE INDEX records_pack_name_idx ON records(pack_name);
            CREATE INDEX records_default_visible_idx ON records(is_default_visible);
            CREATE INDEX reference_edges_from_idx ON reference_edges(from_record_key);
            CREATE INDEX reference_edges_to_idx ON reference_edges(to_record_key);
            CREATE INDEX record_aliases_canonical_idx ON record_aliases(canonical_record_key);
            CREATE INDEX record_aliases_normalized_alias_idx ON record_aliases(normalized_alias);
            CREATE INDEX remaster_links_remaster_idx ON remaster_links(remaster_record_key);
            CREATE INDEX remaster_links_legacy_idx ON remaster_links(legacy_record_key);
            CREATE INDEX record_metrics_record_idx ON record_metrics(record_key);
            CREATE INDEX record_metrics_catalog_source_idx ON record_metrics(metric_domain, metric_key, value_type);
            CREATE INDEX metric_key_catalog_coverage_idx ON metric_key_catalog(metric_domain, record_family, metric_key);
            CREATE INDEX metric_value_catalog_coverage_idx ON metric_value_catalog(metric_domain, record_family, metric_key, value);
            ",
        )
        .map_err(|error| IngestError::ArtifactWriteFailed(error.to_string()))
}

fn write_artifact_metadata(
    connection: &Connection,
    record_count: usize,
    source_signature: &str,
) -> Result<(), IngestError> {
    let metadata = [
        (
            artifact_metadata_keys::ARTIFACT_CONTRACT_VERSION,
            ARTIFACT_CONTRACT_VERSION.to_string(),
        ),
        (
            artifact_metadata_keys::SCHEMA_VERSION,
            ARTIFACT_SCHEMA_VERSION.to_string(),
        ),
        (
            artifact_metadata_keys::SOURCE_KIND,
            EXPECTED_SOURCE_KIND.to_string(),
        ),
        (
            artifact_metadata_keys::SOURCE_SIGNATURE,
            source_signature.to_string(),
        ),
        (
            artifact_metadata_keys::SOURCE_RECORD_COUNT,
            record_count.to_string(),
        ),
        (
            artifact_metadata_keys::CONTENT_HASH_ALGORITHM,
            EXPECTED_CONTENT_HASH_ALGORITHM.to_string(),
        ),
        (
            artifact_metadata_keys::EMBEDDING_PROVIDER_FAMILY,
            EXPECTED_EMBEDDING_PROVIDER_FAMILY.to_string(),
        ),
        (
            artifact_metadata_keys::EMBEDDING_MODEL_ID,
            EXPECTED_EMBEDDING_MODEL_ID.to_string(),
        ),
        (
            artifact_metadata_keys::EMBEDDING_MODEL_REVISION,
            EXPECTED_EMBEDDING_MODEL_REVISION.to_string(),
        ),
        (
            artifact_metadata_keys::EMBEDDING_TOKENIZER_ID,
            EXPECTED_EMBEDDING_TOKENIZER_ID.to_string(),
        ),
        (
            artifact_metadata_keys::EMBEDDING_POOLING,
            EXPECTED_EMBEDDING_POOLING.to_string(),
        ),
        (
            artifact_metadata_keys::EMBEDDING_NORMALIZATION,
            EXPECTED_EMBEDDING_NORMALIZATION.to_string(),
        ),
        (
            artifact_metadata_keys::EMBEDDING_DIMENSIONS,
            EXPECTED_EMBEDDING_DIMENSIONS.to_string(),
        ),
        (
            artifact_metadata_keys::EMBEDDING_DTYPE,
            EXPECTED_EMBEDDING_DTYPE.to_string(),
        ),
        (
            artifact_metadata_keys::EMBEDDING_DISTANCE_METRIC,
            EXPECTED_EMBEDDING_DISTANCE_METRIC.to_string(),
        ),
        (
            artifact_metadata_keys::EMBEDDING_DOCUMENT_PREFIX,
            EXPECTED_EMBEDDING_DOCUMENT_PREFIX.to_string(),
        ),
        (
            artifact_metadata_keys::EMBEDDING_QUERY_PREFIX,
            EXPECTED_EMBEDDING_QUERY_PREFIX.to_string(),
        ),
        (
            artifact_metadata_keys::FTS_TOKENIZER,
            EXPECTED_FTS_TOKENIZER.to_string(),
        ),
        (
            artifact_metadata_keys::ADJACENT_MANIFEST_PATH,
            "manifest.json".to_string(),
        ),
    ];

    for (key, value) in metadata {
        connection
            .execute(
                &format!("INSERT INTO {ARTIFACT_METADATA_TABLE} (key, value) VALUES (?1, ?2)"),
                (key, value),
            )
            .map_err(|error| IngestError::ArtifactWriteFailed(error.to_string()))?;
    }
    Ok(())
}

fn write_packs(connection: &Connection, packs: &[LoadedPack]) -> Result<(), IngestError> {
    let mut statement = connection
        .prepare(
            "INSERT INTO packs (name, label, document_type, declared_path, resolved_path, record_count)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
        )
        .map_err(|error| IngestError::ArtifactWriteFailed(error.to_string()))?;
    for pack in packs {
        statement
            .execute((
                pack.name.as_str(),
                pack.label.as_str(),
                pack.document_type.as_str(),
                pack.declared_path.as_str(),
                pack.resolved_path.display().to_string(),
                pack.record_count,
            ))
            .map_err(|error| IngestError::ArtifactWriteFailed(error.to_string()))?;
    }
    Ok(())
}

fn write_records(
    connection: &Connection,
    records: &[LoadedRecord],
    remaster_links: &[RemasterLink],
) -> Result<(), IngestError> {
    let hidden_record_keys = remaster_links
        .iter()
        .map(|link| link.legacy_record_key.to_string())
        .collect::<BTreeSet<_>>();
    let mut insert_record = connection
        .prepare(
            "INSERT INTO records (
              record_key, id, name, normalized_name, record_family, pack_name, pack_label, foundry_document_type, foundry_record_type,
              level, rarity, traits_json, system_category, system_group, system_base_item, system_usage, system_price_json,
              system_actions_value, system_time_value, system_duration_value, price_cp,
              activation_time_kind, activation_time_actions, activation_time_duration_value, activation_time_duration_unit, activation_time_text,
              duration_kind, duration_value, duration_unit, duration_text,
              publication_title, publication_remaster, description_text, blurb_text,
              description_snippet, publication_family, folder_id, taxonomy_families_json, variant_group_key, variant_base_name,
              variant_label, variant_axes_json, variant_confidence, variant_source, source_path, is_default_visible,
              search_text_projection, raw_json
            ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, ?16, ?17, ?18, ?19, ?20, ?21, ?22, ?23, ?24, ?25, ?26, ?27, ?28, ?29, ?30, ?31, ?32, ?33, ?34, ?35, ?36, ?37, ?38, ?39, ?40, ?41, ?42, ?43, ?44, ?45, ?46, ?47, ?48)",
        )
        .map_err(|error| IngestError::ArtifactWriteFailed(error.to_string()))?;
    let mut insert_trait = connection
        .prepare("INSERT INTO record_traits (record_key, trait) VALUES (?1, ?2)")
        .map_err(|error| IngestError::ArtifactWriteFailed(error.to_string()))?;
    let mut insert_metric = connection
        .prepare(
            "INSERT INTO record_metrics (
              record_key, metric_domain, metric_key, value_type, number_value, text_value, bool_value
            ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
        )
        .map_err(|error| IngestError::ArtifactWriteFailed(error.to_string()))?;
    let mut insert_actor = connection
        .prepare(
            "INSERT INTO actor_records (
              record_key, size, languages_json, speed_types_json, senses_json, immunities_json,
              resistances_json, weaknesses_json, disable_text, disable_skills_json, is_complex
            ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)",
        )
        .map_err(|error| IngestError::ArtifactWriteFailed(error.to_string()))?;
    let mut insert_item = connection
        .prepare(
            "INSERT INTO item_records (
              record_key, system_category, system_base_item, system_group, system_usage, price_cp,
              bulk_value, hands_requirement, damage_types_json
            ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
        )
        .map_err(|error| IngestError::ArtifactWriteFailed(error.to_string()))?;
    let mut insert_spell = connection
        .prepare(
            "INSERT INTO spell_records (
              record_key, traditions_json, spell_kinds_json, range_text, range_value, target_text,
              area_type, area_value, save_type, sustained, basic_save, damage_types_json
            ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12)",
        )
        .map_err(|error| IngestError::ArtifactWriteFailed(error.to_string()))?;
    let mut insert_fts = connection
        .prepare("INSERT INTO records_fts (record_key, name, search_text_projection) VALUES (?1, ?2, ?3)")
        .map_err(|error| IngestError::ArtifactWriteFailed(error.to_string()))?;

    for record in records {
        let is_default_visible =
            record.is_default_visible && !hidden_record_keys.contains(&record.key.to_string());
        let traits_json = serde_json::to_string(&record.traits)
            .map_err(|error| IngestError::ArtifactWriteFailed(error.to_string()))?;
        let taxonomy_families_json = json_array(&record.taxonomy_families)?;
        let variant_axes_json = json_array(&record.variant_axes)?;
        let activation_time = record.activation_time.as_ref();
        let duration = record.duration.as_ref();
        insert_record
            .execute(params![
                record.key.to_string(),
                record.id.as_str(),
                record.name.as_str(),
                record.normalized_name.as_str(),
                record.record_family.as_str(),
                record.pack_name.as_str(),
                record.pack_label.as_str(),
                record.foundry_document_type.as_str(),
                record.foundry_record_type.as_str(),
                record.level,
                record.rarity.as_deref(),
                traits_json,
                record.system_category.as_deref(),
                record.system_group.as_deref(),
                record.system_base_item.as_deref(),
                record.system_usage.as_deref(),
                record.system_price_json.as_deref(),
                record.system_actions_value,
                record.system_time_value.as_deref(),
                record.system_duration_value.as_deref(),
                record.price_cp,
                activation_time.map(|time| time_kind_label(time.kind)),
                activation_time.and_then(|time| time.actions),
                activation_time.and_then(|time| time.duration_value),
                activation_time.and_then(|time| time.duration_unit.map(time_unit_label)),
                activation_time.map(|time| time.text.as_str()),
                duration.map(|time| time_kind_label(time.kind)),
                duration.and_then(|time| time.duration_value),
                duration.and_then(|time| time.duration_unit.map(time_unit_label)),
                duration.map(|time| time.text.as_str()),
                record.publication_title.as_deref(),
                i64::from(record.publication_remaster),
                record.description_text.as_deref(),
                record.blurb_text.as_deref(),
                record.description_text.as_deref(),
                publication_family_label(record.publication_family),
                record.folder_id.as_deref(),
                taxonomy_families_json,
                record.variant_group_key.as_deref(),
                record.variant_base_name.as_deref(),
                record.variant_label.as_deref(),
                variant_axes_json,
                record.variant_confidence,
                record.variant_source.as_str(),
                record.source_path.as_str(),
                i64::from(is_default_visible),
                record.search_text_projection.as_str(),
                record.raw_json.as_str(),
            ])
            .map_err(|error| IngestError::ArtifactWriteFailed(error.to_string()))?;
        for trait_value in &record.traits {
            insert_trait
                .execute((record.key.to_string(), trait_value.as_str()))
                .map_err(|error| IngestError::ArtifactWriteFailed(error.to_string()))?;
        }
        if let Some(actor_data) = &record.actor_data {
            insert_actor
                .execute(params![
                    record.key.to_string(),
                    actor_data.size.as_deref(),
                    json_array(&actor_data.languages)?,
                    json_array(&actor_data.speed_types)?,
                    json_array(&actor_data.senses)?,
                    json_array(&actor_data.immunities)?,
                    json_array(&actor_data.resistances)?,
                    json_array(&actor_data.weaknesses)?,
                    actor_data.disable_text.as_deref(),
                    json_array(&actor_data.disable_skills)?,
                    i64::from(actor_data.is_complex),
                ])
                .map_err(|error| IngestError::ArtifactWriteFailed(error.to_string()))?;
        }
        if let Some(item_data) = &record.item_data {
            insert_item
                .execute(params![
                    record.key.to_string(),
                    item_data.system_category.as_deref(),
                    item_data.system_base_item.as_deref(),
                    item_data.system_group.as_deref(),
                    item_data.system_usage.as_deref(),
                    item_data.price_cp,
                    item_data.bulk_value,
                    item_data.hands_requirement.as_deref(),
                    json_array(&item_data.damage_types)?,
                ])
                .map_err(|error| IngestError::ArtifactWriteFailed(error.to_string()))?;
        }
        if let Some(spell_data) = &record.spell_data {
            insert_spell
                .execute(params![
                    record.key.to_string(),
                    json_array(&spell_data.traditions)?,
                    json_array(&spell_data.spell_kinds)?,
                    spell_data.range_text.as_deref(),
                    spell_data.range_value,
                    spell_data.target_text.as_deref(),
                    spell_data.area_type.as_deref(),
                    spell_data.area_value,
                    spell_data.save_type.as_deref(),
                    i64::from(spell_data.sustained),
                    i64::from(spell_data.basic_save),
                    json_array(&spell_data.damage_types)?,
                ])
                .map_err(|error| IngestError::ArtifactWriteFailed(error.to_string()))?;
        }
        for metric in &record.metrics {
            let (value_type, number_value, text_value, bool_value) =
                metric_value_parts(&metric.value);
            insert_metric
                .execute(params![
                    record.key.to_string(),
                    metric_domain_label(metric.domain),
                    metric.key.as_str(),
                    value_type,
                    number_value,
                    text_value,
                    bool_value,
                ])
                .map_err(|error| IngestError::ArtifactWriteFailed(error.to_string()))?;
        }
        if is_default_visible {
            insert_fts
                .execute((
                    record.key.to_string(),
                    record.name.as_str(),
                    record.search_text_projection.as_str(),
                ))
                .map_err(|error| IngestError::ArtifactWriteFailed(error.to_string()))?;
        }
    }
    Ok(())
}

fn write_reference_edges(
    connection: &Connection,
    references: &[ReferenceEdge],
) -> Result<(), IngestError> {
    let mut insert_reference = connection
        .prepare(
            "INSERT OR IGNORE INTO reference_edges (
              from_record_key, to_record_key, display_text, reference_text
            ) VALUES (?1, ?2, ?3, ?4)",
        )
        .map_err(|error| IngestError::ArtifactWriteFailed(error.to_string()))?;

    for reference in references {
        insert_reference
            .execute((
                reference.from_record_key.to_string(),
                reference.to_record_key.to_string(),
                reference.display_text.as_deref(),
                reference.reference_text.as_str(),
            ))
            .map_err(|error| IngestError::ArtifactWriteFailed(error.to_string()))?;
    }
    Ok(())
}

fn write_record_aliases(
    connection: &Connection,
    aliases: &[RecordAlias],
) -> Result<(), IngestError> {
    let mut insert_alias = connection
        .prepare(
            "INSERT OR IGNORE INTO record_aliases (
              canonical_record_key, alias_text, normalized_alias, source_kind, source_ref
            ) VALUES (?1, ?2, ?3, ?4, ?5)",
        )
        .map_err(|error| IngestError::ArtifactWriteFailed(error.to_string()))?;

    for alias in aliases {
        insert_alias
            .execute((
                alias.canonical_record_key.to_string(),
                alias.alias_text.as_str(),
                alias.normalized_alias.as_str(),
                alias_source_label(alias.source),
                alias.source_ref.as_str(),
            ))
            .map_err(|error| IngestError::ArtifactWriteFailed(error.to_string()))?;
    }
    Ok(())
}

fn write_remaster_links(
    connection: &Connection,
    remaster_links: &[RemasterLink],
) -> Result<(), IngestError> {
    let mut insert_link = connection
        .prepare(
            "INSERT OR IGNORE INTO remaster_links (
              remaster_record_key, legacy_record_key, source_kind, source_ref
            ) VALUES (?1, ?2, ?3, ?4)",
        )
        .map_err(|error| IngestError::ArtifactWriteFailed(error.to_string()))?;

    for link in remaster_links {
        insert_link
            .execute((
                link.remaster_record_key.to_string(),
                link.legacy_record_key.to_string(),
                remaster_link_source_label(link.source),
                link.source_ref.as_str(),
            ))
            .map_err(|error| IngestError::ArtifactWriteFailed(error.to_string()))?;
    }
    Ok(())
}

fn metric_value_parts(
    value: &MetricValue,
) -> (&'static str, Option<f64>, Option<&str>, Option<i64>) {
    match value {
        MetricValue::Number(number) => (
            metric_value_type_label(MetricValueType::Number),
            Some(*number),
            None,
            None,
        ),
        MetricValue::Text(text) => (
            metric_value_type_label(MetricValueType::Text),
            None,
            Some(text.as_str()),
            None,
        ),
        MetricValue::Boolean(boolean) => (
            metric_value_type_label(MetricValueType::Boolean),
            None,
            None,
            Some(i64::from(*boolean)),
        ),
    }
}

fn json_array(values: &[String]) -> Result<String, IngestError> {
    serde_json::to_string(values)
        .map_err(|error| IngestError::ArtifactWriteFailed(error.to_string()))
}

fn metric_domain_label(domain: MetricDomain) -> &'static str {
    match domain {
        MetricDomain::Actor => "actor",
        MetricDomain::Item => "item",
    }
}

fn metric_value_type_label(value_type: MetricValueType) -> &'static str {
    match value_type {
        MetricValueType::Number => "number",
        MetricValueType::Text => "text",
        MetricValueType::Boolean => "boolean",
    }
}

fn alias_source_label(source: AliasSource) -> &'static str {
    match source {
        AliasSource::RemasterJournal => "remaster_journal",
        AliasSource::Migration => "migration",
        AliasSource::CompendiumSource => "compendium_source",
    }
}

fn remaster_link_source_label(source: RemasterLinkSource) -> &'static str {
    match source {
        RemasterLinkSource::RemasterJournal => "remaster_journal",
        RemasterLinkSource::Migration => "migration",
    }
}

fn write_metric_catalogs(connection: &Connection) -> Result<(), IngestError> {
    connection
        .execute_batch(
            "
            INSERT INTO metric_key_catalog (
              metric_domain,
              record_family,
              namespace_prefix,
              metric_key,
              value_type,
              catalog_count,
              numeric_min,
              numeric_max
            )
            SELECT
              rm.metric_domain,
              r.record_family,
              CASE
                WHEN instr(rm.metric_key, '.') > 0 THEN substr(rm.metric_key, 1, instr(rm.metric_key, '.'))
                ELSE ''
              END AS namespace_prefix,
              rm.metric_key,
              rm.value_type,
              COUNT(*) AS catalog_count,
              CASE WHEN rm.value_type = 'number' THEN MIN(rm.number_value) ELSE NULL END AS numeric_min,
              CASE WHEN rm.value_type = 'number' THEN MAX(rm.number_value) ELSE NULL END AS numeric_max
            FROM record_metrics rm
            JOIN records r ON r.record_key = rm.record_key
            WHERE r.is_default_visible = 1
            GROUP BY rm.metric_domain, r.record_family, namespace_prefix, rm.metric_key, rm.value_type;

            INSERT INTO metric_value_catalog (
              metric_domain,
              record_family,
              metric_key,
              value,
              catalog_count
            )
            SELECT
              rm.metric_domain,
              r.record_family,
              rm.metric_key,
              CASE
                WHEN rm.value_type = 'text' THEN rm.text_value
                WHEN rm.value_type = 'boolean' THEN CAST(rm.bool_value AS TEXT)
                ELSE NULL
              END AS value,
              COUNT(*) AS catalog_count
            FROM record_metrics rm
            JOIN records r ON r.record_key = rm.record_key
            WHERE r.is_default_visible = 1
              AND rm.value_type IN ('text', 'boolean')
              AND value IS NOT NULL
            GROUP BY rm.metric_domain, r.record_family, rm.metric_key, value;
            ",
        )
        .map_err(|error| IngestError::ArtifactWriteFailed(error.to_string()))
}

fn time_kind_label(kind: TimeKind) -> &'static str {
    match kind {
        TimeKind::Actions => "actions",
        TimeKind::Free => "free",
        TimeKind::Reaction => "reaction",
        TimeKind::Duration => "duration",
        TimeKind::Variable => "variable",
        TimeKind::Other => "other",
    }
}

fn time_unit_label(unit: TimeUnit) -> &'static str {
    match unit {
        TimeUnit::Round => "round",
        TimeUnit::Minute => "minute",
        TimeUnit::Hour => "hour",
        TimeUnit::Day => "day",
        TimeUnit::Week => "week",
        TimeUnit::Month => "month",
        TimeUnit::Year => "year",
    }
}

fn publication_family_label(publication_family: PublicationFamily) -> &'static str {
    match publication_family {
        PublicationFamily::Core => "core",
        PublicationFamily::Rules => "rules",
        PublicationFamily::Adventure => "adventure",
        PublicationFamily::Unknown => "unknown",
    }
}

pub fn read_artifact_counts(path: &Path) -> Result<(usize, usize), IngestError> {
    let connection = Connection::open(path)
        .map_err(|error| IngestError::ArtifactWriteFailed(error.to_string()))?;
    let pack_count = connection
        .query_row("SELECT COUNT(*) FROM packs", [], |row| {
            row.get::<_, usize>(0)
        })
        .optional()
        .map_err(|error| IngestError::ArtifactWriteFailed(error.to_string()))?
        .unwrap_or_default();
    let record_count = connection
        .query_row("SELECT COUNT(*) FROM records", [], |row| {
            row.get::<_, usize>(0)
        })
        .optional()
        .map_err(|error| IngestError::ArtifactWriteFailed(error.to_string()))?
        .unwrap_or_default();
    Ok((pack_count, record_count))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn classifies_complete_foundry_document_type_taxonomy() {
        let cases = [
            ("Actor", "npc", RecordFamily::Creature),
            ("Actor", "character", RecordFamily::Character),
            ("Actor", "familiar", RecordFamily::Companion),
            ("Actor", "army", RecordFamily::Army),
            ("Actor", "hazard", RecordFamily::Hazard),
            ("Actor", "vehicle", RecordFamily::Vehicle),
            ("Item", "ammo", RecordFamily::Equipment),
            ("Item", "armor", RecordFamily::Equipment),
            ("Item", "backpack", RecordFamily::Equipment),
            ("Item", "consumable", RecordFamily::Equipment),
            ("Item", "equipment", RecordFamily::Equipment),
            ("Item", "kit", RecordFamily::Equipment),
            ("Item", "shield", RecordFamily::Equipment),
            ("Item", "treasure", RecordFamily::Equipment),
            ("Item", "weapon", RecordFamily::Equipment),
            ("Item", "feat", RecordFamily::Feat),
            ("Item", "spell", RecordFamily::Spell),
            ("Item", "action", RecordFamily::Rule),
            ("Item", "condition", RecordFamily::Rule),
            ("Item", "effect", RecordFamily::Rule),
            ("Item", "ancestry", RecordFamily::CharacterOption),
            ("Item", "background", RecordFamily::CharacterOption),
            ("Item", "class", RecordFamily::CharacterOption),
            ("Item", "heritage", RecordFamily::CharacterOption),
            ("Item", "deity", RecordFamily::Lore),
            ("JournalEntry", "JournalEntry", RecordFamily::Lore),
            ("Macro", "script", RecordFamily::Tooling),
            ("RollTable", "RollTable", RecordFamily::Tooling),
            ("Item", "campaignFeature", RecordFamily::CampaignFeature),
            ("Item", "affliction", RecordFamily::Affliction),
            ("Item", "affliction-instance", RecordFamily::Affliction),
        ];

        for (document_type, record_type, expected) in cases {
            assert_eq!(
                classify_record(document_type, record_type),
                Some(expected),
                "{document_type}|{record_type}"
            );
        }
    }

    #[test]
    fn leaves_unknown_foundry_taxonomy_for_skip_reporting() {
        assert_eq!(classify_record("Actor", "mystery"), None);
        assert_eq!(classify_record("Item", "mystery"), None);
    }
}
