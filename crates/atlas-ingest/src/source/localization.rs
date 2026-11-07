//! Explicit indexing locale with pinned source-English fallback.
use crate::IngestError;
use atlas_record::source_content::LocalizationResolver;
use serde_json::Value;
use sha2::{Digest, Sha256};
use std::{
    collections::{BTreeMap, BTreeSet},
    fs,
    path::{Component, Path, PathBuf},
};
#[derive(Debug)]
pub(crate) struct LocalizationCatalog {
    entries: BTreeMap<String, String>,
    selected_keys: BTreeSet<String>,
    english_entries: BTreeMap<String, String>,
    english_origins: BTreeMap<String, (String, String)>,
    pub(crate) locale: String,
    pub(crate) locale_sha256: String,
    pub(crate) english_sha256: String,
}
impl LocalizationCatalog {
    pub(crate) fn english_source(&self, key: &str) -> Option<(&str, &str)> {
        self.english_origins
            .get(key)
            .map(|(file, hash)| (file.as_str(), hash.as_str()))
    }
    pub(crate) fn english_value(&self, key: &str) -> Option<&str> {
        self.english_entries.get(key).map(String::as_str)
    }
    pub(crate) fn content_locale(&self, key: &str) -> Option<&str> {
        self.entries.contains_key(key).then(|| {
            if self.selected_keys.contains(key) {
                self.locale.as_str()
            } else {
                "en"
            }
        })
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
        capture: Option<&crate::source_fingerprint::SourceCapture>,
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
        let mut entries = english.entries;
        let english_sha256 = english.hash;
        let english_origins = english.origins;
        let english_entries = entries.clone();
        let mut selected_keys = BTreeSet::new();
        let locale_sha256 = if locale == "en" {
            english_sha256.clone()
        } else {
            let selected = load_catalogs(
                source_root,
                &[source_root.join(format!("static/lang/{locale}.json"))],
                capture,
                &mut owned_catalog_bytes,
            )?;
            selected_keys.extend(selected.entries.keys().cloned());
            entries.extend(selected.entries);
            selected.hash
        };
        Ok(Self {
            entries,
            selected_keys,
            english_entries,
            english_origins,
            locale: locale.into(),
            locale_sha256,
            english_sha256,
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
        self.entries.get(key).map(String::as_str)
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

struct LoadedCatalogs {
    entries: BTreeMap<String, String>,
    hash: String,
    origins: BTreeMap<String, (String, String)>,
}

fn load_catalogs(
    source_root: &Path,
    paths: &[PathBuf],
    capture: Option<&crate::source_fingerprint::SourceCapture>,
    owned: &mut BTreeMap<PathBuf, Vec<u8>>,
) -> Result<LoadedCatalogs, IngestError> {
    let mut hasher = Sha256::new();
    let mut entries = BTreeMap::new();
    let mut origins = BTreeMap::new();
    for path in paths {
        if !owned.contains_key(path) {
            let bytes = match capture {
                Some(c) => c.read(path)?,
                None => fs::read(path).map_err(|e| {
                    IngestError::LocalizationFailed(format!("{}: {e}", path.display()))
                })?,
            };
            owned.insert(path.clone(), bytes);
        }
        let bytes = owned.get(path).ok_or_else(|| {
            IngestError::LocalizationFailed("owned catalog bytes are absent".into())
        })?;
        let value: Value = serde_json::from_slice(bytes)
            .map_err(|e| IngestError::LocalizationFailed(format!("{}: {e}", path.display())))?;
        if !value.is_object() {
            return Err(IngestError::LocalizationFailed(format!(
                "{}: catalog must be an object",
                path.display()
            )));
        }
        let relative = super::discovery::relative_source_path(source_root, path);
        for input in [relative.as_bytes(), bytes.as_slice()] {
            hasher.update((input.len() as u64).to_le_bytes());
            hasher.update(input);
        }
        let mut selected = BTreeMap::new();
        flatten_entries(None, &value, &mut selected);
        let digest = format!("{:x}", Sha256::digest(bytes));
        origins.extend(
            selected
                .keys()
                .map(|key| (key.clone(), (relative.clone(), digest.clone()))),
        );
        entries.extend(selected);
    }
    Ok(LoadedCatalogs {
        entries,
        hash: format!("{:x}", hasher.finalize()),
        origins,
    })
}
fn flatten_entries(prefix: Option<&str>, value: &Value, entries: &mut BTreeMap<String, String>) {
    match value {
        Value::String(text) => {
            if let Some(key) = prefix {
                entries.insert(key.into(), text.clone());
            }
        }
        Value::Object(object) => {
            for (key, value) in object {
                let next = prefix
                    .map(|prefix| format!("{prefix}.{key}"))
                    .unwrap_or_else(|| key.clone());
                flatten_entries(Some(&next), value, entries);
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
