#![deny(unsafe_code)]

use std::fs;
use std::path::{Path, PathBuf};

use atlas_domain::{
    ARTIFACT_CONTRACT_VERSION, ARTIFACT_METADATA_TABLE, ARTIFACT_SCHEMA_VERSION, Category,
    EXPECTED_CONTENT_HASH_ALGORITHM, EXPECTED_EMBEDDING_DIMENSIONS,
    EXPECTED_EMBEDDING_DISTANCE_METRIC, EXPECTED_EMBEDDING_DOCUMENT_PREFIX,
    EXPECTED_EMBEDDING_DTYPE, EXPECTED_EMBEDDING_MODEL_ID, EXPECTED_EMBEDDING_MODEL_REVISION,
    EXPECTED_EMBEDDING_NORMALIZATION, EXPECTED_EMBEDDING_POOLING,
    EXPECTED_EMBEDDING_PROVIDER_FAMILY, EXPECTED_EMBEDDING_QUERY_PREFIX,
    EXPECTED_EMBEDDING_TOKENIZER_ID, EXPECTED_FTS_TOKENIZER, EXPECTED_SOURCE_KIND, PackName,
    RecordId, RecordKey, SourceCategory, Subcategory, TextStatus, artifact_metadata_keys,
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

#[derive(Debug, Clone, PartialEq, Eq)]
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

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LoadedRecord {
    pub key: RecordKey,
    pub id: RecordId,
    pub name: String,
    pub normalized_name: String,
    pub category: Category,
    pub subcategory: Option<Subcategory>,
    pub pack_name: PackName,
    pub pack_label: String,
    pub document_type: String,
    pub record_type: String,
    pub traits: Vec<String>,
    pub publication_title: Option<String>,
    pub publication_remaster: bool,
    pub description_text: Option<String>,
    pub has_description: bool,
    pub source_category: SourceCategory,
    pub source_path: String,
    pub text_status: TextStatus,
    pub search_text: String,
    pub raw_json: String,
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
    let (category, subcategory) = classify_record(&manifest_pack.document_type, &record_type);
    let traits = extract_traits(&raw);
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
    let search_text = create_search_text(&name, description_text.as_deref(), &traits);
    let raw_json = serde_json::to_string(&raw).map_err(|error| {
        normalization_error(path, &format!("raw JSON serialization failed: {error}"))
    })?;

    Ok(LoadedRecord {
        key,
        id,
        name,
        normalized_name,
        category,
        subcategory,
        pack_name: pack_name.clone(),
        pack_label: manifest_pack.label.clone(),
        document_type: manifest_pack.document_type.clone(),
        record_type,
        traits,
        publication_title,
        publication_remaster,
        description_text: description_text.clone(),
        has_description: description_text.is_some(),
        source_category: SourceCategory::Unknown,
        source_path,
        text_status,
        search_text,
        raw_json,
    })
}

fn normalization_error(path: &Path, message: &str) -> IngestError {
    IngestError::RecordNormalizationFailed {
        path: path.display().to_string(),
        message: message.to_string(),
    }
}

fn classify_record(document_type: &str, record_type: &str) -> (Category, Option<Subcategory>) {
    match (document_type, record_type) {
        ("Actor", "hazard") => (Category::Hazard, Some(Subcategory::Trap)),
        ("Actor", "familiar") => (Category::Creature, Some(Subcategory::Familiar)),
        ("Actor", _) => (Category::Creature, None),
        ("Item", "action") => (Category::Rule, Some(Subcategory::Action)),
        ("Item", "condition") => (Category::Rule, Some(Subcategory::Condition)),
        ("Item", "effect") => (Category::Rule, Some(Subcategory::Effect)),
        ("Item", "spell") => (Category::Spell, None),
        ("Item", "feat") => (Category::Feat, None),
        ("Item", "ancestry") => (Category::CharacterCreation, Some(Subcategory::Ancestry)),
        ("Item", "background") => (Category::CharacterCreation, Some(Subcategory::Background)),
        ("Item", "class") => (Category::CharacterCreation, Some(Subcategory::Class)),
        ("Item", "heritage") => (Category::CharacterCreation, Some(Subcategory::Heritage)),
        ("Item", "weapon") => (Category::Equipment, Some(Subcategory::Weapon)),
        ("Item", "armor") => (Category::Equipment, Some(Subcategory::Armor)),
        ("Item", "shield") => (Category::Equipment, Some(Subcategory::Shield)),
        ("Item", "consumable") => (Category::Equipment, Some(Subcategory::Consumable)),
        ("JournalEntry", _) | ("JournalEntryPage", _) => {
            (Category::Lore, Some(Subcategory::Journal))
        }
        _ => (Category::Lore, None),
    }
}

fn string_field(raw: &Value, key: &str) -> Option<String> {
    raw.get(key)?.as_str().map(str::to_string)
}

fn pointer_string(raw: &Value, pointer: &str) -> Option<String> {
    raw.pointer(pointer)?.as_str().map(str::to_string)
}

fn pointer_bool(raw: &Value, pointer: &str) -> Option<bool> {
    raw.pointer(pointer)?.as_bool()
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
              category TEXT NOT NULL,
              subcategory TEXT,
              pack_name TEXT NOT NULL,
              pack_label TEXT NOT NULL,
              document_type TEXT NOT NULL,
              record_type TEXT NOT NULL,
              level INTEGER,
              rarity TEXT,
              traits_json TEXT NOT NULL,
              derived_tags_json TEXT NOT NULL,
              publication_title TEXT,
              publication_remaster INTEGER NOT NULL,
              description_text TEXT,
              blurb_text TEXT,
              has_description INTEGER NOT NULL,
              description_snippet TEXT,
              source_category TEXT NOT NULL,
              folder_id TEXT,
              families_json TEXT NOT NULL,
              variant_family_key TEXT,
              variant_base_name TEXT,
              variant_label TEXT,
              variant_axes_json TEXT NOT NULL,
              variant_confidence REAL,
              variant_source TEXT NOT NULL,
              source_path TEXT NOT NULL,
              is_unique INTEGER NOT NULL,
              is_search_canonical INTEGER NOT NULL,
              search_text TEXT NOT NULL,
              raw_json TEXT NOT NULL
            );

            CREATE VIRTUAL TABLE records_fts USING fts5(
              record_key UNINDEXED,
              name,
              search_text
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
              record_key, id, name, normalized_name, category, subcategory, pack_name, pack_label, document_type, record_type,
              level, rarity, traits_json, derived_tags_json, publication_title, publication_remaster, description_text, blurb_text,
              has_description, description_snippet, source_category, folder_id, families_json, variant_family_key, variant_base_name,
              variant_label, variant_axes_json, variant_confidence, variant_source, source_path, is_unique, is_search_canonical,
              search_text, raw_json
            ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, ?16, ?17, ?18, ?19, ?20, ?21, ?22, ?23, ?24, ?25, ?26, ?27, ?28, ?29, ?30, ?31, ?32, ?33, ?34)",
        )
        .map_err(|error| IngestError::ArtifactWriteFailed(error.to_string()))?;
    let mut insert_fts = connection
        .prepare("INSERT INTO records_fts (record_key, name, search_text) VALUES (?1, ?2, ?3)")
        .map_err(|error| IngestError::ArtifactWriteFailed(error.to_string()))?;

    for record in records {
        let traits_json = serde_json::to_string(&record.traits)
            .map_err(|error| IngestError::ArtifactWriteFailed(error.to_string()))?;
        insert_record
            .execute(params![
                record.key.to_string(),
                record.id.as_str(),
                record.name.as_str(),
                record.normalized_name.as_str(),
                record.category.as_str(),
                record.subcategory.map(Subcategory::as_str),
                record.pack_name.as_str(),
                record.pack_label.as_str(),
                record.document_type.as_str(),
                record.record_type.as_str(),
                Option::<i64>::None,
                Option::<String>::None,
                traits_json,
                "[]",
                record.publication_title.as_deref(),
                i64::from(record.publication_remaster),
                record.description_text.as_deref(),
                Option::<String>::None,
                i64::from(record.has_description),
                record.description_text.as_deref(),
                source_category_label(record.source_category),
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
                record.search_text.as_str(),
                record.raw_json.as_str(),
            ])
            .map_err(|error| IngestError::ArtifactWriteFailed(error.to_string()))?;
        insert_fts
            .execute((
                record.key.to_string(),
                record.name.as_str(),
                record.search_text.as_str(),
            ))
            .map_err(|error| IngestError::ArtifactWriteFailed(error.to_string()))?;
    }
    Ok(())
}

fn source_category_label(source_category: SourceCategory) -> &'static str {
    match source_category {
        SourceCategory::Core => "core",
        SourceCategory::Rules => "rules",
        SourceCategory::Adventure => "adventure",
        SourceCategory::Unknown => "unknown",
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
