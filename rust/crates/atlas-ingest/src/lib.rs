#![deny(unsafe_code)]

use std::collections::BTreeSet;
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
    MetricValueType, PackName, PublicationFamily, RecordFamily, RecordId, RecordKey, TextStatus,
    TimeKind, TimeUnit, artifact_metadata_keys,
};
use rusqlite::{Connection, OptionalExtension, params};
use serde::Deserialize;
use serde_json::Value;
use thiserror::Error;

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
    pub skipped_records: Vec<SkippedRecord>,
    pub warnings: Vec<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct SourceLoad {
    pub packs: Vec<LoadedPack>,
    pub records: Vec<LoadedRecord>,
    pub skipped_records: Vec<SkippedRecord>,
    pub warnings: Vec<String>,
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
    pub publication_family: PublicationFamily,
    pub source_path: String,
    pub text_status: TextStatus,
    pub search_text_projection: String,
    pub raw_json: String,
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
    let manifest = parse_manifest(&manifest_path)?;
    let mut packs = Vec::new();
    let mut records = Vec::new();
    let mut skipped_records = Vec::new();
    let mut warnings = Vec::new();

    for manifest_pack in manifest.packs {
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

    Ok(SourceLoad {
        packs,
        records,
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

fn parse_manifest(path: &Path) -> Result<Manifest, IngestError> {
    let serialized = fs::read_to_string(path)
        .map_err(|error| IngestError::SourceUnavailable(error.to_string()))?;
    serde_json::from_str(&serialized)
        .map_err(|error| IngestError::ManifestParseFailed(error.to_string()))
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
    let publication_title = pointer_string(&raw, "/system/publication/title");
    let publication_remaster = pointer_bool(&raw, "/system/publication/remaster").unwrap_or(false);
    let description_text =
        pointer_string(&raw, "/system/description/value").map(|value| strip_markup(&value));
    let description_text = description_text.filter(|value| !value.trim().is_empty());
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
    let raw_json = serde_json::to_string(&raw).map_err(|error| {
        normalization_error(path, &format!("raw JSON serialization failed: {error}"))
    })?;

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
        publication_family: PublicationFamily::Unknown,
        source_path,
        text_status,
        search_text_projection,
        raw_json,
    })
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
    write_artifact_metadata(&transaction, source.records.len())?;
    write_packs(&transaction, &source.packs)?;
    write_records(&transaction, &source.records)?;
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
              publication_remaster INTEGER NOT NULL,
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
              is_unique INTEGER NOT NULL,
              is_search_canonical INTEGER NOT NULL,
              search_text_projection TEXT NOT NULL,
              raw_json TEXT NOT NULL
            );

            CREATE TABLE record_traits (
              record_key TEXT NOT NULL,
              trait TEXT NOT NULL,
              PRIMARY KEY (record_key, trait),
              FOREIGN KEY (record_key) REFERENCES records(record_key) ON DELETE CASCADE
            );

            CREATE TABLE record_metrics (
              record_key TEXT NOT NULL,
              metric_domain TEXT NOT NULL CHECK (metric_domain IN ('actor', 'item')),
              metric_key TEXT NOT NULL,
              value_type TEXT NOT NULL CHECK (value_type IN ('number', 'text', 'boolean')),
              number_value REAL,
              text_value TEXT,
              bool_value INTEGER,
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
              is_complex INTEGER NOT NULL,
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
              sustained INTEGER NOT NULL,
              basic_save INTEGER NOT NULL,
              damage_types_json TEXT NOT NULL,
              FOREIGN KEY (record_key) REFERENCES records(record_key) ON DELETE CASCADE
            );

            CREATE VIRTUAL TABLE records_fts USING fts5(
              record_key UNINDEXED,
              name,
              search_text_projection
            );
            ",
        )
        .map_err(|error| IngestError::ArtifactWriteFailed(error.to_string()))
}

fn write_artifact_metadata(
    connection: &Connection,
    record_count: usize,
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
            "foundry-pf2e:minimal-rust-ingest".to_string(),
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

fn write_records(connection: &Connection, records: &[LoadedRecord]) -> Result<(), IngestError> {
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
              variant_label, variant_axes_json, variant_confidence, variant_source, source_path, is_unique, is_search_canonical,
              search_text_projection, raw_json
            ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, ?16, ?17, ?18, ?19, ?20, ?21, ?22, ?23, ?24, ?25, ?26, ?27, ?28, ?29, ?30, ?31, ?32, ?33, ?34, ?35, ?36, ?37, ?38, ?39, ?40, ?41, ?42, ?43, ?44, ?45, ?46, ?47, ?48, ?49)",
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
        let traits_json = serde_json::to_string(&record.traits)
            .map_err(|error| IngestError::ArtifactWriteFailed(error.to_string()))?;
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
                Option::<i64>::None,
                Option::<String>::None,
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
                Option::<String>::None,
                record.description_text.as_deref(),
                publication_family_label(record.publication_family),
                Option::<String>::None,
                "[]",
                Option::<String>::None,
                Option::<String>::None,
                Option::<String>::None,
                "[]",
                Option::<f64>::None,
                "none",
                record.source_path.as_str(),
                0_i64,
                1_i64,
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
        insert_fts
            .execute((
                record.key.to_string(),
                record.name.as_str(),
                record.search_text_projection.as_str(),
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
            WHERE r.is_search_canonical = 1
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
            WHERE r.is_search_canonical = 1
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
