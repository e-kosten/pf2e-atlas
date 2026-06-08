use std::fs;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::{Duration, Instant};

use atlas_domain::PackName;
use rayon::prelude::*;
use serde_json::Value;
use sha2::{Digest, Sha256};
use tracing::info;

use crate::diagnostics::IngestDiagnostics;
use crate::error::IngestError;
use crate::source::localization::{LocalizationCatalog, LocalizationSourceFile};
use crate::source::model::SkippedRecord;
use crate::source::normalize::{ContentParseDiagnostics, DroppedContentMacro, normalize_record};
use crate::source::{LoadedPack, ManifestPack, ParsedManifest, SourceLoad};

const DROPPED_INLINE_MACRO_EXAMPLE_LIMIT: usize = 5;

#[derive(Debug)]
struct SourceSignatureRecord {
    source_path: String,
    content_hash: String,
}

#[derive(Debug, Clone, Copy, Default)]
struct SourceLoadTiming {
    discover_duration: Duration,
    read_duration: Duration,
    hash_duration: Duration,
    parse_duration: Duration,
    normalize_duration: Duration,
    signature_duration: Duration,
}

#[derive(Debug)]
struct LoadedSourceFile {
    record: crate::records::LoadedSourceRecord,
    source_signature: SourceSignatureRecord,
}

#[derive(Debug)]
struct ProcessedSourceFile {
    path: PathBuf,
    timing: SourceLoadTiming,
    result: Result<LoadedSourceFile, IngestError>,
}

pub(crate) fn load_foundry_source_records(
    source_root: impl AsRef<Path>,
    manifest_path: Option<&Path>,
) -> Result<SourceLoad, IngestError> {
    let load_started_at = Instant::now();
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
    let localization = LocalizationCatalog::load(source_root)?;
    let mut packs = Vec::new();
    let mut records = Vec::new();
    let mut source_signature_records = Vec::new();
    let mut skipped_records = Vec::new();
    let mut warnings = Vec::new();
    let mut diagnostics = IngestDiagnostics::default();
    let manifest_packs = parsed_manifest.manifest.packs;
    let manifest_pack_count = manifest_packs.len();
    let mut timing = SourceLoadTiming::default();

    for (pack_index, manifest_pack) in manifest_packs.into_iter().enumerate() {
        let pack_number = pack_index + 1;
        let resolved_path = resolve_pack_path(source_root, &manifest_pack);
        if !resolved_path.is_dir() {
            info!(
                pack = manifest_pack.name.as_str(),
                current = pack_number,
                total = manifest_pack_count,
                path = %resolved_path.display(),
                "skipping unreadable source pack"
            );
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
        info!(target: "atlas_progress",
            phase = "source_packs",
            current = pack_number as u64,
            total = manifest_pack_count as u64,
            "Scanning source pack: {pack}",
            pack = manifest_pack.name.as_str()
        );
        let discover_started_at = Instant::now();
        let paths = json_files(&resolved_path)?;
        timing.discover_duration += discover_started_at.elapsed();
        info!(target: "atlas_progress",
            phase = "source_packs",
            current = pack_number as u64,
            total = manifest_pack_count as u64,
            "Loading source pack: {pack} (0/{files})",
            pack = manifest_pack.name.as_str(),
            files = paths.len()
        );
        let record_start = records.len();

        let record_progress_interval = record_progress_interval(paths.len());
        let processed_count = AtomicUsize::new(0);
        let processed_files = paths
            .par_iter()
            .map(|path| {
                let processed_file = process_source_file(
                    source_root,
                    &manifest_pack,
                    &pack_name,
                    path,
                    &localization,
                );
                let processed = processed_count.fetch_add(1, Ordering::Relaxed) + 1;
                if processed == paths.len() || processed.is_multiple_of(record_progress_interval) {
                    info!(target: "atlas_progress",
                        phase = "source_packs",
                        current = pack_number as u64,
                        total = manifest_pack_count as u64,
                        "Loading source pack: {pack} ({processed}/{files})",
                        pack = manifest_pack.name.as_str(),
                        files = paths.len()
                    );
                }
                processed_file
            })
            .collect::<Vec<_>>();

        for processed_file in processed_files {
            add_timing(&mut timing, processed_file.timing);
            match processed_file.result {
                Ok(loaded) => {
                    collect_content_parse_diagnostics(
                        &loaded.record.facts.content_parse_diagnostics,
                        &mut diagnostics,
                    );
                    source_signature_records.push(loaded.source_signature);
                    records.push(loaded.record);
                }
                Err(error) => {
                    skipped_records.push(SkippedRecord {
                        path: processed_file.path,
                        reason: error.to_string(),
                    });
                    warnings.push(error.to_string());
                }
            }
        }

        let record_count = records.len() - record_start;
        info!(target: "atlas_progress",
            phase = "source_packs",
            current = pack_number as u64,
            total = manifest_pack_count as u64,
            "Finished source pack: {pack} ({record_count} records)",
            pack = manifest_pack.name.as_str()
        );
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

    let signature_started_at = Instant::now();
    let source_signature = compute_source_signature(
        source_root,
        &manifest_path,
        &parsed_manifest.content_hash,
        localization.source_files(),
        &packs,
        &source_signature_records,
        &skipped_records,
    );
    timing.signature_duration = signature_started_at.elapsed();
    let source_record_count = records.len();
    info!(
        load_wall_ms = load_started_at.elapsed().as_millis(),
        discover_ms = timing.discover_duration.as_millis(),
        read_ms = timing.read_duration.as_millis(),
        hash_ms = timing.hash_duration.as_millis(),
        parse_ms = timing.parse_duration.as_millis(),
        normalize_ms = timing.normalize_duration.as_millis(),
        signature_ms = timing.signature_duration.as_millis(),
        "source load timing"
    );

    Ok(SourceLoad {
        manifest_path,
        source_signature,
        source_record_count,
        packs,
        records,
        references: Vec::new(),
        aliases: Vec::new(),
        remaster_links: Vec::new(),
        pending_document_embeddings: Vec::new(),
        document_embeddings: Vec::new(),
        document_embedding_tokenization: Default::default(),
        diagnostics,
        skipped_records,
        warnings,
    })
}

fn process_source_file(
    source_root: &Path,
    manifest_pack: &ManifestPack,
    pack_name: &PackName,
    path: &Path,
    localization: &LocalizationCatalog,
) -> ProcessedSourceFile {
    let mut timing = SourceLoadTiming::default();
    let result = read_json_record(path, &mut timing).and_then(|raw_record| {
        let normalize_started_at = Instant::now();
        let normalized_record = normalize_record(
            manifest_pack,
            pack_name,
            path,
            source_root,
            raw_record.value,
            Some(localization),
        );
        timing.normalize_duration += normalize_started_at.elapsed();
        normalized_record.map(|record| LoadedSourceFile {
            record,
            source_signature: SourceSignatureRecord {
                source_path: relative_source_path(source_root, path),
                content_hash: raw_record.content_hash,
            },
        })
    });
    ProcessedSourceFile {
        path: path.to_path_buf(),
        timing,
        result,
    }
}

fn add_timing(total: &mut SourceLoadTiming, timing: SourceLoadTiming) {
    total.discover_duration += timing.discover_duration;
    total.read_duration += timing.read_duration;
    total.hash_duration += timing.hash_duration;
    total.parse_duration += timing.parse_duration;
    total.normalize_duration += timing.normalize_duration;
    total.signature_duration += timing.signature_duration;
}

fn collect_content_parse_diagnostics(
    parse_diagnostics: &[ContentParseDiagnostics],
    diagnostics: &mut IngestDiagnostics,
) {
    for diagnostic in parse_diagnostics {
        for dropped in &diagnostic.dropped_macros {
            record_dropped_inline_macro(dropped.clone(), diagnostics);
        }
    }
}

fn record_dropped_inline_macro(dropped: DroppedContentMacro, diagnostics: &mut IngestDiagnostics) {
    let entry = diagnostics
        .dropped_inline_macros
        .entry(dropped.name)
        .or_default();
    entry.count += 1;
    if entry.examples.len() < DROPPED_INLINE_MACRO_EXAMPLE_LIMIT
        && !entry.examples.iter().any(|example| example == &dropped.raw)
    {
        entry.examples.push(dropped.raw);
    }
}

pub(crate) fn resolve_pack_path(source_root: &Path, manifest_pack: &ManifestPack) -> PathBuf {
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

pub(crate) fn default_manifest_path(source_root: &Path) -> PathBuf {
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
    localization_sources: &[LocalizationSourceFile],
    packs: &[LoadedPack],
    records: &[SourceSignatureRecord],
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

    for source in localization_sources {
        hash_field(&mut hasher, "localization");
        hash_field(
            &mut hasher,
            &relative_source_path(source_root, &source.path),
        );
        hash_field(&mut hasher, &source.content_hash);
    }

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
    sorted_records.sort_by(|left, right| left.source_path.cmp(&right.source_path));
    for record in sorted_records {
        hash_field(&mut hasher, "record");
        hash_field(&mut hasher, &record.source_path);
        hash_field(&mut hasher, &record.content_hash);
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

pub(crate) fn relative_source_path(source_root: &Path, path: &Path) -> String {
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

pub(crate) fn json_files(root: &Path) -> Result<Vec<PathBuf>, IngestError> {
    let mut paths = Vec::new();
    collect_json_files(root, &mut paths)?;
    paths.sort();
    Ok(paths)
}

fn record_progress_interval(record_count: usize) -> usize {
    (record_count / 10).clamp(25, 500)
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

struct RawJsonRecord {
    value: Value,
    content_hash: String,
}

fn read_json_record(
    path: &Path,
    timing: &mut SourceLoadTiming,
) -> Result<RawJsonRecord, IngestError> {
    let read_started_at = Instant::now();
    let serialized = fs::read_to_string(path)
        .map_err(|error| IngestError::RecordParseFailed(error.to_string()))?;
    timing.read_duration += read_started_at.elapsed();

    let hash_started_at = Instant::now();
    let content_hash = sha256_hex(serialized.as_bytes());
    timing.hash_duration += hash_started_at.elapsed();

    let parse_started_at = Instant::now();
    let value = serde_json::from_str(&serialized)
        .map_err(|error| IngestError::RecordParseFailed(format!("{}: {error}", path.display())))?;
    timing.parse_duration += parse_started_at.elapsed();
    Ok(RawJsonRecord {
        value,
        content_hash,
    })
}
