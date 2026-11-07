//! The approved, normalized reference subset bundled with Atlas.
use crate::IngestError;
use atlas_domain::{DefinitionKey, DefinitionNamespace};
use serde::Deserialize;
use sha2::{Digest, Sha256};
use std::collections::BTreeMap;

pub const REFERENCE_PREPARATION_RECIPE: &str = "reference-definitions/v5";

const MANIFEST: &[u8] = include_bytes!("../reference-data/manifest.json");
const PAYLOAD: &[u8] = include_bytes!("../reference-data/payload.json");

const BUNDLED_FILES: &[(&str, &[u8])] = &[
    ("payload.json", PAYLOAD),
    ("notices.json", crate::reference_notices::BUNDLED_NOTICES),
    (
        "orc-license.pdf",
        include_bytes!("../reference-data/orc-license.pdf"),
    ),
    (
        "community-use-policy.pdf",
        include_bytes!("../reference-data/community-use-policy.pdf"),
    ),
];

fn hash(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}
fn invalid(message: impl Into<String>) -> IngestError {
    IngestError::SourceSelectionFailed(message.into())
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct Manifest {
    schema: u32,
    recipe: String,
    permission_recipe: String,
    source_repository: String,
    source_commit: String,
    source_files: BTreeMap<String, SourceFile>,
    source_package: String,
    content_locale: String,
    english_fallback: bool,
    mapping_recipe: String,
    files: BTreeMap<String, String>,
    selection: Selection,
}
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct SourceFile {
    sha256: String,
    url: String,
}
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct Selection {
    language_contexts: usize,
    weapon_groups: usize,
    wildsong_existing_record: String,
}
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct Payload {
    schema: u32,
    entries: Vec<SupplementEntry>,
}
#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct SupplementEntry {
    pub(crate) namespace: DefinitionNamespace,
    pub(crate) identifier: String,
    pub(crate) source_field: String,
    pub(crate) publication: String,
    pub(crate) page: u32,
    pub(crate) content_locale: String,
    pub(crate) grants: Vec<String>,
    pub(crate) speakers: Vec<String>,
    pub(crate) markup: Option<String>,
}
pub(crate) struct ReferenceSupplementCapture {
    pub(crate) entries: BTreeMap<DefinitionKey, SupplementEntry>,
    pub(crate) source_commit: String,
    pub(crate) identity: String,
    source_files: BTreeMap<String, SourceFile>,
}
impl ReferenceSupplementCapture {
    /// Canonical provenance identifies the immutable raw file. The normalized
    /// selected field has its own prepared markup hash and manifest checksum.
    pub(crate) fn raw_source(&self, namespace: DefinitionNamespace) -> Option<(&str, &str)> {
        let path = match namespace {
            DefinitionNamespace::Language => "data/languages.json",
            DefinitionNamespace::WeaponGroup => "data/groups.json",
            _ => return None,
        };
        self.source_files
            .get_key_value(path)
            .map(|(path, source)| (path.as_str(), source.sha256.as_str()))
    }
    pub(crate) fn capture(locale: &str) -> Result<Self, IngestError> {
        let manifest: Manifest = serde_json::from_slice(MANIFEST)
            .map_err(|error| invalid(format!("reference manifest: {error}")))?;
        if manifest.schema != 1
            || manifest.recipe != "pc1-reference-subset/v1"
            || manifest.permission_recipe != "pc1-functional-orc-cup-descriptive/v1"
            || manifest.mapping_recipe != "pf2e-reference-tokens/v1"
            || manifest.source_package != "pf2e"
            || manifest.content_locale != "en"
            || !manifest.english_fallback
        {
            return Err(invalid(
                "unsupported reference manifest recipe or locale policy",
            ));
        }
        if locale.is_empty()
            || manifest.source_repository != "https://github.com/Pf2eToolsOrg/Pf2eTools"
            || manifest.source_commit.len() != 40
            || !manifest
                .source_commit
                .bytes()
                .all(|b| b.is_ascii_hexdigit())
        {
            return Err(invalid("invalid reference source provenance"));
        }
        for (name, source) in &manifest.source_files {
            if !["data/languages.json", "data/groups.json"].contains(&name.as_str())
                || source.sha256.len() != 64
                || !source.sha256.bytes().all(|b| b.is_ascii_hexdigit())
                || source.url
                    != format!(
                        "https://raw.githubusercontent.com/Pf2eToolsOrg/Pf2eTools/{}/{name}",
                        manifest.source_commit
                    )
            {
                return Err(invalid("invalid reference source-file identity"));
            }
        }
        if manifest.source_files.len() != 2 || manifest.files.len() != BUNDLED_FILES.len() {
            return Err(invalid("incomplete reference source/file manifest"));
        }
        for (name, bytes) in BUNDLED_FILES {
            if manifest.files.get(*name) != Some(&hash(bytes)) {
                return Err(invalid(format!("reference file checksum mismatch: {name}")));
            }
        }
        let payload: Payload = serde_json::from_slice(PAYLOAD)
            .map_err(|error| invalid(format!("reference payload: {error}")))?;
        if payload.schema != 1
            || manifest.selection.language_contexts != 24
            || manifest.selection.weapon_groups != 4
            || manifest.selection.wildsong_existing_record != "classfeatures:RiAGlnnp4S21BAG3"
        {
            return Err(invalid("unsupported reference subset selection"));
        }
        let mut entries = BTreeMap::new();
        for entry in payload.entries {
            let key = DefinitionKey::new(entry.namespace, &entry.identifier)
                .map_err(|error| invalid(error.to_string()))?;
            let valid = match entry.namespace {
                DefinitionNamespace::Language => {
                    entry.source_field == "typicalSpeakers"
                        && !entry.speakers.is_empty()
                        && entry.markup.is_none()
                        && entry.identifier != "wildsong"
                }
                DefinitionNamespace::WeaponGroup => {
                    entry.source_field == "specialization"
                        && entry.speakers.is_empty()
                        && entry
                            .markup
                            .as_ref()
                            .is_some_and(|markup| !markup.trim().is_empty())
                        && ["crossbow", "dart", "knife", "pick"]
                            .contains(&entry.identifier.as_str())
                }
                _ => false,
            };
            if !valid
                || entry.publication != "Pathfinder Player Core"
                || entry.content_locale != "en"
                || entry.grants.is_empty()
                || entry
                    .grants
                    .iter()
                    .any(|grant| !["ORC", "CUP"].contains(&grant.as_str()))
                || entry
                    .speakers
                    .iter()
                    .any(|speaker| speaker.is_empty() || speaker.contains("{@"))
                || entries.insert(key, entry).is_some()
            {
                return Err(invalid("invalid or conflicting selected reference entry"));
            }
        }
        if entries.len() != 28
            || entries
                .keys()
                .filter(|key| key.namespace() == DefinitionNamespace::Language)
                .count()
                != 24
        {
            return Err(invalid("incomplete selected reference subset"));
        }
        Ok(Self {
            entries,
            source_commit: manifest.source_commit,
            identity: hash(MANIFEST),
            source_files: manifest.source_files,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn approved_subset_preserves_context_and_prose_without_a_second_wildsong_body() {
        let captured = ReferenceSupplementCapture::capture("en").unwrap();
        let chthonian = &captured.entries[&"language:chthonian".parse().unwrap()];
        assert_eq!(chthonian.speakers, ["Demons"]);
        assert_eq!(chthonian.content_locale, "en");
        assert!(chthonian.markup.is_none());
        assert!(
            !captured
                .entries
                .contains_key(&"language:wildsong".parse().unwrap())
        );
        let shadowtongue = &captured.entries[&"language:shadowtongue".parse().unwrap()];
        assert_eq!(shadowtongue.speakers, ["Nidalese", "Netherworld creatures"]);
        assert!(shadowtongue.grants.contains(&"CUP".into()));
        let crossbow = &captured.entries[&"weapon_group:crossbow".parse().unwrap()];
        let prose = crossbow.markup.as_deref().unwrap();
        assert!(prose.contains("1d8 persistent bleed damage"));
        assert!(prose.contains("item bonus to attack rolls"));
        assert!(!prose.contains("{@"));
        assert_eq!(captured.entries.len(), 28);
    }
}
