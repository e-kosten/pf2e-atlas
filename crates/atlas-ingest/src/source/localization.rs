//! Explicit indexing locale with pinned source-English fallback.
use crate::{
    IngestError,
    source_fingerprint::{SourceCapture, VerifiedSourceBytes},
};
use atlas_record::source_content::LocalizationResolver;
use serde_json::Value;
use sha2::{Digest, Sha256};
use std::{
    collections::{BTreeMap, btree_map::Entry},
    fs,
    path::{Component, Path, PathBuf},
};
#[derive(Debug)]
pub(crate) struct LocalizationCatalog {
    english: LoadedCatalogs,
    selected: Option<LoadedCatalogs>,
    pub(crate) locale: String,
    pub(crate) locale_sha256: String,
    pub(crate) english_sha256: String,
}
impl LocalizationCatalog {
    pub(crate) fn english_source(&self, key: &str) -> Option<(&str, &str)> {
        let entry = self.english.entries.get(key)?;
        let (file, hash) = &self.english.origins[entry.origin];
        Some((file, hash))
    }
    pub(crate) fn english_value(&self, key: &str) -> Option<&str> {
        self.english.value(key)
    }
    pub(crate) fn content_locale(&self, key: &str) -> Option<&str> {
        self.selected
            .as_ref()
            .and_then(|catalog| catalog.value(key))
            .map(|_| self.locale.as_str())
            .or_else(|| self.english_value(key).map(|_| "en"))
    }
    pub(crate) fn load(
        source_root: &Path,
        locale: &str,
        manifest: Option<&Path>,
    ) -> Result<Self, IngestError> {
        Self::load_captured(source_root, locale, manifest, None)
    }
    pub(crate) fn load_captured(
        source_root: &Path,
        locale: &str,
        manifest: Option<&Path>,
        capture: Option<&SourceCapture>,
    ) -> Result<Self, IngestError> {
        if locale.is_empty()
            || !locale
                .bytes()
                .all(|byte| byte.is_ascii_alphanumeric() || byte == b'-')
        {
            return Err(IngestError::LocalizationFailed(
                "locale must be an explicit catalog identifier".into(),
            ));
        }
        let english_paths = if let Some(capture) = capture {
            capture.english_catalog_paths().to_vec()
        } else {
            let manifest_path = manifest
                .map(Path::to_owned)
                .unwrap_or_else(|| super::discovery::default_manifest_path(source_root));
            let manifest_bytes = fs::read(&manifest_path)
                .map_err(|e| IngestError::LocalizationFailed(e.to_string()))?;
            let parsed = super::discovery::parse_manifest_bytes(&manifest_bytes)?;
            let mut paths = parsed
                .manifest
                .languages
                .iter()
                .filter(|language| language.lang == "en")
                .map(|language| resolve_catalog_path(source_root, &language.path))
                .collect::<Result<Vec<_>, _>>()?;
            if paths.is_empty() {
                paths.push(source_root.join("static/lang/en.json"));
            }
            paths
        };
        let mut owned_catalog_bytes = BTreeMap::new();
        let english = load_catalogs(
            source_root,
            &english_paths,
            capture,
            &mut owned_catalog_bytes,
        )?;
        let selected = if locale == "en" {
            None
        } else {
            Some(load_catalogs(
                source_root,
                &[source_root.join(format!("static/lang/{locale}.json"))],
                capture,
                &mut owned_catalog_bytes,
            )?)
        };
        Ok(Self {
            locale: locale.into(),
            locale_sha256: selected.as_ref().unwrap_or(&english).hash.clone(),
            english_sha256: english.hash.clone(),
            english,
            selected,
        })
    }
}
pub(crate) struct EnglishLocalization<'a>(pub(crate) &'a LocalizationCatalog);
impl LocalizationResolver for EnglishLocalization<'_> {
    fn localized_value(&self, key: &str) -> Option<&str> {
        self.0.english_value(key)
    }
}

impl LocalizationResolver for LocalizationCatalog {
    fn localized_value(&self, key: &str) -> Option<&str> {
        self.selected
            .as_ref()
            .and_then(|catalog| catalog.value(key))
            .or_else(|| self.english_value(key))
    }
}
pub(crate) fn resolve_catalog_path(
    source_root: &Path,
    declared: &str,
) -> Result<PathBuf, IngestError> {
    let relative = Path::new(declared);
    if relative.as_os_str().is_empty()
        || relative
            .components()
            .any(|part| !matches!(part, Component::Normal(_)))
    {
        return Err(IngestError::LocalizationFailed(format!(
            "catalog path must be relative to its package: {declared}"
        )));
    }
    let direct = source_root.join(relative);
    Ok(if direct.is_file() {
        direct
    } else {
        source_root.join("static").join(relative)
    })
}

#[derive(Debug)]
struct LocalizedEntry {
    text: String,
    origin: usize,
}

#[derive(Debug)]
struct LoadedCatalogs {
    entries: BTreeMap<String, LocalizedEntry>,
    hash: String,
    origins: Vec<(String, String)>,
}
impl LoadedCatalogs {
    fn value(&self, key: &str) -> Option<&str> {
        self.entries.get(key).map(|entry| entry.text.as_str())
    }
}

fn load_catalogs(
    source_root: &Path,
    paths: &[PathBuf],
    capture: Option<&SourceCapture>,
    owned: &mut BTreeMap<PathBuf, VerifiedSourceBytes>,
) -> Result<LoadedCatalogs, IngestError> {
    let mut hasher = Sha256::new();
    let mut entries = BTreeMap::new();
    let mut origins = Vec::new();
    for path in paths {
        let source = match owned.entry(path.clone()) {
            Entry::Occupied(entry) => entry.into_mut(),
            Entry::Vacant(entry) => entry.insert(match capture {
                Some(capture) => capture.read_verified(path)?,
                None => {
                    let bytes = fs::read(path).map_err(|e| {
                        IngestError::LocalizationFailed(format!("{}: {e}", path.display()))
                    })?;
                    VerifiedSourceBytes {
                        sha256: format!("{:x}", Sha256::digest(&bytes)),
                        bytes,
                    }
                }
            }),
        };
        let value: Value = serde_json::from_slice(&source.bytes)
            .map_err(|e| IngestError::LocalizationFailed(format!("{}: {e}", path.display())))?;
        if !value.is_object() {
            return Err(IngestError::LocalizationFailed(format!(
                "{}: catalog must be an object",
                path.display()
            )));
        }
        let relative = super::discovery::relative_source_path(source_root, path);
        for input in [relative.as_bytes(), source.bytes.as_slice()] {
            hasher.update((input.len() as u64).to_le_bytes());
            hasher.update(input);
        }
        flatten_entries(None, value, origins.len(), &mut entries);
        origins.push((relative, source.sha256.clone()));
    }
    Ok(LoadedCatalogs {
        entries,
        hash: format!("{:x}", hasher.finalize()),
        origins,
    })
}
fn flatten_entries(
    prefix: Option<&str>,
    value: Value,
    origin: usize,
    entries: &mut BTreeMap<String, LocalizedEntry>,
) {
    match value {
        Value::String(text) => {
            if let Some(key) = prefix {
                entries.insert(key.into(), LocalizedEntry { text, origin });
            }
        }
        Value::Object(object) => {
            for (key, value) in object {
                let next = prefix
                    .map(|prefix| format!("{prefix}.{key}"))
                    .unwrap_or_else(|| key.clone());
                flatten_entries(Some(&next), value, origin, entries);
            }
        }
        _ => {}
    }
}

#[cfg(test)]
mod tests;

pub(crate) fn record_trait_labels(
    record: &atlas_record::source_record::SourceBackedRecord,
    catalog: &LocalizationCatalog,
) -> BTreeMap<String, String> {
    atlas_record::source_record::searchable_identity_traits(
        record,
        crate::build::default_audience(),
    )
    .into_iter()
    .filter_map(|id| {
        let label = atlas_foundry_model::generated::trait_definition(&id)
            .and_then(|d| d.label_key)
            .and_then(|key| catalog.localized_value(key))
            .filter(|s| !s.trim().is_empty())?;
        Some((id, label.to_owned()))
    })
    .collect()
}
