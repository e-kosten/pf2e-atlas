use std::fs;
use std::path::{Path, PathBuf};

use atlas_domain::PackName;
use serde_json::Value;
use sha2::{Digest, Sha256};

use crate::model::{ManifestPack, ParsedManifest};
use crate::normalize::normalize_record;
use crate::references::{build_record_reference_index, resolve_reference_edges};
use crate::{
    DERIVED_AFFLICTION_INSTANCES_PACK_LABEL, DERIVED_AFFLICTION_INSTANCES_PACK_NAME,
    DERIVED_AFFLICTIONS_PACK_LABEL, DERIVED_AFFLICTIONS_PACK_NAME, IngestDiagnostics, IngestError,
    LoadedPack, LoadedRecord, SkippedRecord, SourceLoad, aliases, generated_afflictions, variants,
};

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

pub(crate) fn parse_manifest(path: &Path) -> Result<ParsedManifest, IngestError> {
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

pub(crate) fn compute_source_signature(
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

pub(crate) fn hash_field(hasher: &mut Sha256, value: &str) {
    hasher.update(value.len().to_string().as_bytes());
    hasher.update(b":");
    hasher.update(value.as_bytes());
    hasher.update(b"\n");
}

pub(crate) fn sha256_hex(bytes: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(bytes);
    hex_lower(hasher.finalize())
}

pub(crate) fn hex_lower(bytes: impl AsRef<[u8]>) -> String {
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

pub(crate) fn collect_json_files(root: &Path, paths: &mut Vec<PathBuf>) -> Result<(), IngestError> {
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

pub(crate) fn read_json_record(path: &Path) -> Result<Value, IngestError> {
    let serialized = fs::read_to_string(path)
        .map_err(|error| IngestError::RecordParseFailed(error.to_string()))?;
    serde_json::from_str(&serialized)
        .map_err(|error| IngestError::RecordParseFailed(format!("{}: {error}", path.display())))
}
