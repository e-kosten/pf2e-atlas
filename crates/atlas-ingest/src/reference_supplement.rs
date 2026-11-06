//! One approved, normalized reference subset; acquisition/cache policy stays in runtime.
use crate::IngestError;
use atlas_domain::{DefinitionKey, DefinitionNamespace};
use atlas_index::ReferenceNotice;
use serde::Deserialize;
use sha2::{Digest, Sha256};
use std::{collections::BTreeMap, fs, path::PathBuf};

pub const REFERENCE_PREPARATION_RECIPE: &str = "reference-definitions/v2";

#[derive(Debug, Clone, Default)]
pub enum ReferenceSupplementSelection {
    /// Verified embedded bytes are available offline and need no network access.
    #[default]
    Bundled,
    /// Explicit removal is an input revision, never a missing-file fallback.
    Absent {
        revision: String,
    },
    Directory {
        path: PathBuf,
    },
}

pub struct BundledReferenceFile {
    pub name: &'static str,
    pub bytes: &'static [u8],
}

static BUNDLED: &[BundledReferenceFile] = &[
    BundledReferenceFile {
        name: "manifest.json",
        bytes: include_bytes!("../reference-data/manifest.json"),
    },
    BundledReferenceFile {
        name: "payload.json",
        bytes: include_bytes!("../reference-data/payload.json"),
    },
    BundledReferenceFile {
        name: "notices.json",
        bytes: include_bytes!("../reference-data/notices.json"),
    },
    BundledReferenceFile {
        name: "orc-license.pdf",
        bytes: include_bytes!("../reference-data/orc-license.pdf"),
    },
    BundledReferenceFile {
        name: "community-use-policy.pdf",
        bytes: include_bytes!("../reference-data/community-use-policy.pdf"),
    },
];

pub fn bundled_reference_supplement() -> &'static [BundledReferenceFile] {
    BUNDLED
}
pub fn bundled_reference_supplement_id() -> String {
    hash(BUNDLED[0].bytes)
}
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
    system_version: String,
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
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct Notices {
    schema: u32,
    notices: Vec<Notice>,
}
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct Notice {
    id: String,
    text: String,
    source_url: Option<String>,
}

/// Small assets are consumed into owned bytes at capture; originals are never rescanned.
pub(crate) struct ReferenceSupplementCapture {
    pub(crate) entries: BTreeMap<DefinitionKey, SupplementEntry>,
    pub(crate) notices: Vec<ReferenceNotice>,
    pub(crate) source_commit: Option<String>,
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
    pub(crate) fn capture(
        selection: &ReferenceSupplementSelection,
        package: &str,
        version: Option<&str>,
        locale: &str,
    ) -> Result<Self, IngestError> {
        if let ReferenceSupplementSelection::Absent { revision } = selection {
            if revision.trim().is_empty() {
                return Err(invalid(
                    "reference supplement absence needs an explicit revision",
                ));
            }
            return Ok(Self {
                entries: BTreeMap::new(),
                notices: Vec::new(),
                source_commit: None,
                identity: hash(format!("absent:{revision}").as_bytes()),
                source_files: BTreeMap::new(),
            });
        }
        let read = |name: &str| -> Result<Vec<u8>, IngestError> {
            match selection {
                ReferenceSupplementSelection::Bundled => BUNDLED
                    .iter()
                    .find(|file| file.name == name)
                    .map(|file| file.bytes.to_vec())
                    .ok_or_else(|| invalid("unrecognized bundled reference file")),
                ReferenceSupplementSelection::Directory { path } => fs::read(path.join(name))
                    .map_err(|error| {
                        invalid(format!(
                            "required reference file {name} unavailable: {error}"
                        ))
                    }),
                ReferenceSupplementSelection::Absent { .. } => {
                    Err(invalid("absent reference supplement has no files"))
                }
            }
        };
        let manifest_bytes = read("manifest.json")?;
        let manifest: Manifest = serde_json::from_slice(&manifest_bytes)
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
        if package == "pf2e" && version != Some(manifest.system_version.as_str()) {
            return Err(invalid(
                "reference supplement does not support the selected PF2e version",
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
        if manifest.source_files.len() != 2 || manifest.files.len() != BUNDLED.len() - 1 {
            return Err(invalid("incomplete reference source/file manifest"));
        }
        let mut owned = BTreeMap::new();
        for file in BUNDLED.iter().filter(|file| file.name != "manifest.json") {
            let bytes = read(file.name)?;
            if manifest.files.get(file.name) != Some(&hash(&bytes)) {
                return Err(invalid(format!(
                    "reference file checksum mismatch: {}",
                    file.name
                )));
            }
            owned.insert(file.name, bytes);
        }
        let payload: Payload = serde_json::from_slice(&owned["payload.json"])
            .map_err(|error| invalid(format!("reference payload: {error}")))?;
        let notices: Notices = serde_json::from_slice(&owned["notices.json"])
            .map_err(|error| invalid(format!("reference notices: {error}")))?;
        if payload.schema != 1
            || notices.schema != 1
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
        let mut ids = std::collections::BTreeSet::new();
        let mut notices = notices
            .notices
            .into_iter()
            .map(|notice| {
                if notice.text.trim().is_empty() || !ids.insert(notice.id.clone()) {
                    return Err(invalid("empty or duplicate reference notice"));
                }
                Ok(ReferenceNotice {
                    id: notice.id,
                    sha256: hash(notice.text.as_bytes()),
                    text: notice.text,
                    source_url: notice.source_url,
                })
            })
            .collect::<Result<Vec<_>, _>>()?;
        if [
            "orc-license",
            "player-core",
            "community-use",
            "pf2etools-formatting",
        ]
        .iter()
        .any(|id| !ids.contains(*id))
        {
            return Err(invalid("reference subset lacks required notices"));
        }
        // Retain the compact extraction/mapping recipe, raw-file identities and
        // normalized payload identity for source-free attribution/export.
        if ids.contains("reference-subset-manifest") {
            return Err(invalid("reserved reference provenance notice identity"));
        }
        notices.push(ReferenceNotice {
            id: "reference-subset-manifest".into(),
            sha256: hash(&manifest_bytes),
            text: String::from_utf8(manifest_bytes.clone())
                .map_err(|error| invalid(error.to_string()))?,
            source_url: None,
        });
        Ok(Self {
            entries,
            notices,
            source_commit: Some(manifest.source_commit),
            identity: hash(&manifest_bytes),
            source_files: manifest.source_files,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn directory() -> tempfile::TempDir {
        let directory = tempfile::tempdir().unwrap();
        for file in bundled_reference_supplement() {
            fs::write(directory.path().join(file.name), file.bytes).unwrap();
        }
        directory
    }

    fn capture(
        selection: &ReferenceSupplementSelection,
    ) -> Result<ReferenceSupplementCapture, IngestError> {
        ReferenceSupplementCapture::capture(selection, "pf2e", Some("8.6.1"), "en")
    }

    #[test]
    fn approved_subset_preserves_context_and_prose_without_a_second_wildsong_body() {
        let captured = capture(&ReferenceSupplementSelection::Bundled).unwrap();
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
        assert!(
            captured
                .notices
                .iter()
                .any(|notice| notice.id == "player-core" && notice.text.contains("Carlos Cabrera"))
        );
    }

    #[test]
    fn embedded_and_cached_assets_have_one_path_independent_owned_identity() {
        let first = directory();
        let second = directory();
        let selection = ReferenceSupplementSelection::Directory {
            path: first.path().into(),
        };
        let captured = capture(&selection).unwrap();
        assert_eq!(
            captured.identity,
            capture(&ReferenceSupplementSelection::Bundled)
                .unwrap()
                .identity
        );
        assert_eq!(
            captured.identity,
            capture(&ReferenceSupplementSelection::Directory {
                path: second.path().into()
            })
            .unwrap()
            .identity
        );
        // Capture consumed the small asset. Later mutations belong to the next attempt.
        fs::write(first.path().join("payload.json"), b"corrupt").unwrap();
        assert_eq!(
            captured.entries[&"language:chthonian".parse().unwrap()].speakers,
            ["Demons"]
        );
        assert!(capture(&selection).is_err());
    }

    #[test]
    fn selected_missing_corrupt_and_incompatible_assets_do_not_become_absent() {
        let directory = directory();
        let selection = ReferenceSupplementSelection::Directory {
            path: directory.path().into(),
        };
        fs::remove_file(directory.path().join("notices.json")).unwrap();
        assert!(capture(&selection).is_err());
        assert!(
            ReferenceSupplementCapture::capture(
                &ReferenceSupplementSelection::Bundled,
                "pf2e",
                Some("0.0.0"),
                "en"
            )
            .is_err()
        );
        assert!(
            ReferenceSupplementCapture::capture(
                &ReferenceSupplementSelection::Bundled,
                "custom-module",
                Some("0.0.0"),
                "en"
            )
            .is_ok()
        );
        let absent = capture(&ReferenceSupplementSelection::Absent {
            revision: "disabled/v2".into(),
        })
        .unwrap();
        assert!(absent.entries.is_empty());
        assert_ne!(
            absent.identity,
            capture(&ReferenceSupplementSelection::Bundled)
                .unwrap()
                .identity
        );
        assert!(
            capture(&ReferenceSupplementSelection::Absent {
                revision: "".into()
            })
            .is_err()
        );
    }

    #[test]
    fn approved_notice_only_change_changes_the_input_identity() {
        let directory = directory();
        let selection = ReferenceSupplementSelection::Directory {
            path: directory.path().into(),
        };
        let before = capture(&selection).unwrap();
        let mut notices: serde_json::Value =
            serde_json::from_slice(&fs::read(directory.path().join("notices.json")).unwrap())
                .unwrap();
        let row = notices["notices"]
            .as_array_mut()
            .unwrap()
            .iter_mut()
            .find(|row| row["id"] == "pf2etools-formatting")
            .unwrap();
        row["text"] = format!(
            "{}\nUpdated contributor credit.",
            row["text"].as_str().unwrap()
        )
        .into();
        let bytes = serde_json::to_vec(&notices).unwrap();
        fs::write(directory.path().join("notices.json"), &bytes).unwrap();
        let mut manifest: serde_json::Value =
            serde_json::from_slice(&fs::read(directory.path().join("manifest.json")).unwrap())
                .unwrap();
        manifest["files"]["notices.json"] = hash(&bytes).into();
        fs::write(
            directory.path().join("manifest.json"),
            serde_json::to_vec(&manifest).unwrap(),
        )
        .unwrap();
        let after = capture(&selection).unwrap();
        assert_ne!(before.identity, after.identity);
        assert_eq!(before.entries.len(), after.entries.len());
        let provenance = |capture: &ReferenceSupplementCapture| -> serde_json::Value {
            serde_json::from_str(
                &capture
                    .notices
                    .iter()
                    .find(|notice| notice.id == "reference-subset-manifest")
                    .unwrap()
                    .text,
            )
            .unwrap()
        };
        assert_eq!(
            provenance(&before)["files"]["payload.json"],
            provenance(&after)["files"]["payload.json"]
        );
        assert_ne!(
            before.notices.last().unwrap().sha256,
            after.notices.last().unwrap().sha256
        );
    }
}
