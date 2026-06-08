use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

use serde_json::Value;
use sha2::{Digest, Sha256};

use crate::error::IngestError;
use crate::source::normalize::LocalizationResolver;

#[derive(Debug, Clone, Default)]
pub(crate) struct LocalizationCatalog {
    entries: BTreeMap<String, String>,
    source_files: Vec<LocalizationSourceFile>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct LocalizationSourceFile {
    pub(crate) path: PathBuf,
    pub(crate) content_hash: String,
}

impl LocalizationCatalog {
    pub(crate) fn load(source_root: &Path) -> Result<Self, IngestError> {
        let path = source_root.join("static/lang/en.json");
        if !path.is_file() {
            return Ok(Self::default());
        }

        let serialized = fs::read_to_string(&path)
            .map_err(|error| IngestError::ManifestParseFailed(error.to_string()))?;
        let content_hash = sha256_hex(serialized.as_bytes());
        let value = serde_json::from_str::<Value>(&serialized).map_err(|error| {
            IngestError::ManifestParseFailed(format!("{}: {error}", path.display()))
        })?;

        let mut entries = BTreeMap::new();
        flatten_entries(None, &value, &mut entries);

        Ok(Self {
            entries,
            source_files: vec![LocalizationSourceFile { path, content_hash }],
        })
    }

    #[cfg(test)]
    pub(crate) fn from_entries(entries: impl IntoIterator<Item = (String, String)>) -> Self {
        Self {
            entries: entries.into_iter().collect(),
            source_files: Vec::new(),
        }
    }

    pub(crate) fn source_files(&self) -> &[LocalizationSourceFile] {
        &self.source_files
    }
}

impl LocalizationResolver for LocalizationCatalog {
    fn localized_value(&self, key: &str) -> Option<&str> {
        self.entries.get(key).map(String::as_str)
    }
}

fn flatten_entries(prefix: Option<&str>, value: &Value, entries: &mut BTreeMap<String, String>) {
    match value {
        Value::String(text) => {
            if let Some(key) = prefix {
                entries.insert(key.to_string(), text.clone());
            }
        }
        Value::Object(object) => {
            for (key, value) in object {
                let next_key = prefix
                    .map(|prefix| format!("{prefix}.{key}"))
                    .unwrap_or_else(|| key.clone());
                flatten_entries(Some(&next_key), value, entries);
            }
        }
        Value::Null | Value::Bool(_) | Value::Number(_) | Value::Array(_) => {}
    }
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
        .collect::<String>()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn flattens_nested_string_entries_to_dotted_keys() {
        let value = serde_json::json!({
            "PF2E": {
                "TraitFire": "Fire",
                "Nested": {
                    "IgnoredNumber": 1,
                    "Text": "Nested text"
                }
            }
        });
        let mut entries = BTreeMap::new();

        flatten_entries(None, &value, &mut entries);

        assert_eq!(entries.get("PF2E.TraitFire"), Some(&"Fire".to_string()));
        assert_eq!(
            entries.get("PF2E.Nested.Text"),
            Some(&"Nested text".to_string())
        );
        assert!(!entries.contains_key("PF2E.Nested.IgnoredNumber"));
    }
}
