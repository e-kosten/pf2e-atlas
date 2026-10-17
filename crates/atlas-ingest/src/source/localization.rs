//! Explicit indexing locale with pinned source-English fallback.
use crate::IngestError;
use atlas_record::source_content::LocalizationResolver;
use serde_json::Value;
use sha2::{Digest, Sha256};
use std::{collections::BTreeMap, fs, path::Path};
#[derive(Debug)]
pub(crate) struct LocalizationCatalog {
    entries: BTreeMap<String, String>,
    pub(crate) locale: String,
    pub(crate) locale_sha256: String,
    pub(crate) english_sha256: String,
}
impl LocalizationCatalog {
    pub(crate) fn load(source_root: &Path, locale: &str) -> Result<Self, IngestError> {
        if locale.is_empty()
            || !locale
                .bytes()
                .all(|byte| byte.is_ascii_alphanumeric() || byte == b'-')
        {
            return Err(IngestError::LocalizationFailed(
                "locale must be an explicit catalog identifier".into(),
            ));
        }
        let directory = source_root.join("static/lang");
        let (mut entries, english_sha256) = load_catalog(&directory.join("en.json"))?;
        let locale_sha256 = if locale == "en" {
            english_sha256.clone()
        } else {
            let (selected, hash) = load_catalog(&directory.join(format!("{locale}.json")))?;
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
/// Persist only labels reached by an admitted product identity, using pinned key
/// evidence and the same selected-locale/English resolver as content preparation.
pub(crate) fn used_trait_labels(
    source: &crate::EnrichedFoundrySource,
    catalog: &LocalizationCatalog,
) -> Result<BTreeMap<String, String>, IngestError> {
    use atlas_record::source_record::{SourceQueryView, select_record_search};
    let mut labels = BTreeMap::new();
    for pack in &source.packs {
        for document in &pack.documents {
            let crate::EnrichedDocumentOutcome::Addressed { record, .. } = &document.outcome else {
                continue;
            };
            // Identities use typed selection alone; empty prepared content avoids
            // rebuilding or parsing any rich prose for this vocabulary pass.
            let identities = select_record_search(
                record,
                &[],
                crate::build::default_audience(),
                &pack.metadata.label,
                &BTreeMap::new(),
            )
            .map_err(|e| IngestError::SourceSelectionFailed(e.to_string()))?
            .identities;
            for identity in identities {
                let Some(node) = record.node_at(&identity.owners) else {
                    continue;
                };
                let query = SourceQueryView {
                    source: node,
                    pack_id: record.key().pack().as_str(),
                    pack_label: &pack.metadata.label,
                };
                if let Some(traits) = query.traits().value() {
                    for identifier in traits {
                        if let Some(key) =
                            atlas_foundry_model::generated::trait_label_key(identifier)
                            && let Some(label) = catalog
                                .localized_value(key)
                                .filter(|s| !s.trim().is_empty())
                        {
                            labels.insert(identifier.clone(), label.to_owned());
                        }
                    }
                }
            }
        }
    }
    Ok(labels)
}
fn load_catalog(path: &Path) -> Result<(BTreeMap<String, String>, String), IngestError> {
    let bytes = fs::read(path)
        .map_err(|e| IngestError::LocalizationFailed(format!("{}: {e}", path.display())))?;
    let value: Value = serde_json::from_slice(&bytes)
        .map_err(|e| IngestError::LocalizationFailed(format!("{}: {e}", path.display())))?;
    if !value.is_object() {
        return Err(IngestError::LocalizationFailed(format!(
            "{}: catalog must be an object",
            path.display()
        )));
    }
    let mut entries = BTreeMap::new();
    flatten_entries(None, &value, &mut entries);
    Ok((entries, format!("{:x}", Sha256::digest(bytes))))
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
