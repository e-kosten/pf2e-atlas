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

mod aliases;
mod generated_afflictions;
mod metrics;
pub mod report;
mod schema;
mod variants;

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
    pub manifest_path: PathBuf,
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

pub fn build_artifact(options: BuildArtifactOptions) -> Result<BuildArtifactReport, IngestError> {
    let source = load_foundry_source(&options.source_root, options.manifest_path.as_deref())?;
    if source.records.is_empty() {
        return Err(IngestError::NoRecordsLoaded);
    }
    write_artifact(&options.output_path, &source)?;
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
    let generated_afflictions =
        generated_afflictions::build_generated_afflictions(&records, &reference_index);
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
    variants::assign_taxonomy_families(&mut records, &packs, &reference_index, &mut diagnostics);
    variants::assign_variant_groups(&mut records, &reference_index, &mut diagnostics);
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
    let aliases = aliases::resolve_record_aliases(&records, &reference_index, source_root);
    let remaster_links = aliases::resolve_remaster_links(&records, &reference_index, source_root);

    Ok(SourceLoad {
        manifest_path,
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
    let metrics = metrics::extract_metrics(&raw, &manifest_pack.document_type, &record_type);
    let actor_data =
        (manifest_pack.document_type == "Actor").then(|| metrics::extract_actor_side_data(&raw));
    let item_data = (manifest_pack.document_type == "Item").then(|| {
        metrics::extract_item_side_data(
            &raw,
            system_category.clone(),
            system_base_item.clone(),
            system_group.clone(),
            system_usage.clone(),
            price_cp,
        )
    });
    let spell_data = (manifest_pack.document_type == "Item" && record_type == "spell")
        .then(|| metrics::extract_spell_side_data(&raw, &traits));
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
        .map(|value| metrics::slugify_metric_segment(&value).replace('_', " "))
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

pub fn write_artifact(path: &Path, source: &SourceLoad) -> Result<(), IngestError> {
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
    schema::create_artifact_schema(&transaction)?;
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
