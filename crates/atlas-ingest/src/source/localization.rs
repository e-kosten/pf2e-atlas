//! Explicit indexing locale with pinned source-English fallback.
use crate::IngestError;
use atlas_record::source_content::LocalizationResolver;
use serde_json::Value;
use sha2::{Digest, Sha256};
use std::{
    collections::BTreeMap,
    fs,
    path::{Component, Path, PathBuf},
};
#[derive(Debug)]
pub(crate) struct LocalizationCatalog {
    entries: BTreeMap<String, String>,
    pub(crate) locale: String,
    pub(crate) locale_sha256: String,
    pub(crate) english_sha256: String,
}
impl LocalizationCatalog {
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
        let manifest_path = capture
            .map(|capture| capture.manifest_path.clone())
            .or_else(|| manifest.map(Path::to_owned))
            .unwrap_or_else(|| super::discovery::default_manifest_path(source_root));
        let manifest_bytes = match capture {
            Some(capture) => capture.read(&manifest_path)?,
            None => fs::read(&manifest_path)
                .map_err(|e| IngestError::LocalizationFailed(e.to_string()))?,
        };
        let parsed = super::discovery::parse_manifest_bytes(&manifest_bytes)?;
        let mut english_paths = parsed
            .manifest
            .languages
            .iter()
            .filter(|language| language.lang == "en")
            .map(|language| resolve_catalog_path(source_root, &language.path))
            .collect::<Result<Vec<_>, _>>()?;
        // Minimal source exports without language declarations retain their
        // explicit static/lang/en.json catalog convention.
        if english_paths.is_empty() {
            english_paths.push(source_root.join("static/lang/en.json"));
        }
        let (mut entries, english_sha256) = load_catalogs(source_root, &english_paths, capture)?;
        let locale_sha256 = if locale == "en" {
            english_sha256.clone()
        } else {
            let (selected, hash) = load_catalogs(
                source_root,
                &[source_root.join(format!("static/lang/{locale}.json"))],
                capture,
            )?;
            entries.extend(selected);
            hash
        };
        Ok(Self {
            entries,
            locale: locale.into(),
            locale_sha256,
            english_sha256,
        })
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

fn load_catalogs(
    source_root: &Path,
    paths: &[PathBuf],
    capture: Option<&crate::source_fingerprint::SourceCapture>,
) -> Result<(BTreeMap<String, String>, String), IngestError> {
    let mut hasher = Sha256::new();
    let mut entries = BTreeMap::new();
    for path in paths {
        let bytes = match capture {
            Some(c) => c.read(path)?,
            None => fs::read(path)
                .map_err(|e| IngestError::LocalizationFailed(format!("{}: {e}", path.display())))?,
        };
        let value: Value = serde_json::from_slice(&bytes)
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
        flatten_entries(None, &value, &mut entries);
    }
    Ok((entries, format!("{:x}", hasher.finalize())))
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
