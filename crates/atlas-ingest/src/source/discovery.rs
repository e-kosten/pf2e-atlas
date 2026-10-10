//! Manifest and pack-file discovery shared by source consumers.
use crate::error::IngestError;
use crate::source::{ManifestPack, ParsedManifest};
use sha2::{Digest, Sha256};
use std::fs;
use std::path::{Path, PathBuf};

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
    let content_hash = format!("{:x}", Sha256::digest(serialized.as_bytes()));
    let manifest = serde_json::from_str(&serialized)
        .map_err(|error| IngestError::ManifestParseFailed(error.to_string()))?;
    Ok(ParsedManifest {
        manifest,
        content_hash,
    })
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
