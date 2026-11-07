//! The approved, normalized reference subset bundled with Atlas.
use crate::IngestError;
use atlas_domain::{DefinitionKey, DefinitionNamespace};
use serde::Deserialize;
use sha2::{Digest, Sha256};
use std::collections::BTreeMap;

pub const REFERENCE_PREPARATION_RECIPE: &str = "reference-definitions/v6";

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
    publication: String,
    language_page: u32,
    weapon_group_page: u32,
    base_grant: String,
    language_speakers: Vec<(String, Vec<String>)>,
    weapon_specializations: Vec<(String, String)>,
    cup_language: Option<String>,
}
pub(crate) enum SupplementBody {
    Speakers(Vec<String>),
    Specialization(String),
}
pub(crate) struct SupplementEntry {
    pub(crate) page: u32,
    pub(crate) grants: Vec<String>,
    pub(crate) body: SupplementBody,
}
pub(crate) struct ReferenceSupplementCapture {
    pub(crate) entries: BTreeMap<DefinitionKey, SupplementEntry>,
    pub(crate) publication: String,
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
            || manifest.recipe != "pc1-reference-subset/v2"
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
        let wildsong = atlas_foundry_model::generated::reference_definitions()
            .iter()
            .find(|entry| entry.namespace == "language" && entry.identifier == "wildsong")
            .and_then(|entry| entry.record_key);
        let cup_language = payload
            .language_speakers
            .iter()
            .any(|(identifier, _)| identifier == "shadowtongue")
            .then_some("shadowtongue");
        if payload.schema != 2
            || payload.publication != "Pathfinder Player Core"
            || payload.language_page == 0
            || payload.weapon_group_page == 0
            || payload.base_grant != "ORC"
            || payload.language_speakers.len() != manifest.selection.language_contexts
            || payload.weapon_specializations.len() != manifest.selection.weapon_groups
            || payload.cup_language.as_deref() != cup_language
            || wildsong != Some(manifest.selection.wildsong_existing_record.as_str())
        {
            return Err(invalid(
                "unsupported or incomplete reference subset selection",
            ));
        }

        let mut entries = BTreeMap::new();
        let mut insert = |namespace, identifier: String, entry| {
            let key = DefinitionKey::new(namespace, identifier)
                .map_err(|error| invalid(error.to_string()))?;
            if entries.insert(key, entry).is_some() {
                return Err(invalid("duplicate selected reference entry"));
            }
            Ok(())
        };
        for (identifier, speakers) in payload.language_speakers {
            if identifier == "wildsong"
                || speakers.is_empty()
                || speakers
                    .iter()
                    .any(|speaker| speaker.is_empty() || speaker.contains("{@"))
            {
                return Err(invalid("invalid selected language speakers"));
            }
            let grants = if Some(identifier.as_str()) == cup_language {
                vec![payload.base_grant.clone(), "CUP".into()]
            } else {
                vec![payload.base_grant.clone()]
            };
            insert(
                DefinitionNamespace::Language,
                identifier,
                SupplementEntry {
                    page: payload.language_page,
                    grants,
                    body: SupplementBody::Speakers(speakers),
                },
            )?;
        }
        for (identifier, markup) in payload.weapon_specializations {
            if !["crossbow", "dart", "knife", "pick"].contains(&identifier.as_str())
                || markup.trim().is_empty()
            {
                return Err(invalid("invalid selected weapon specialization"));
            }
            insert(
                DefinitionNamespace::WeaponGroup,
                identifier,
                SupplementEntry {
                    page: payload.weapon_group_page,
                    grants: vec![payload.base_grant.clone()],
                    body: SupplementBody::Specialization(markup),
                },
            )?;
        }
        Ok(Self {
            entries,
            publication: payload.publication,
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
        assert!(
            matches!(&chthonian.body, SupplementBody::Speakers(speakers) if speakers == &["demons"])
        );
        assert_eq!(chthonian.page, 89);
        assert!(
            !captured
                .entries
                .contains_key(&"language:wildsong".parse().unwrap())
        );
        let shadowtongue = &captured.entries[&"language:shadowtongue".parse().unwrap()];
        assert!(
            matches!(&shadowtongue.body, SupplementBody::Speakers(speakers) if speakers == &["Nidalese", "Netherworld creatures"])
        );
        assert!(shadowtongue.grants.contains(&"CUP".into()));
        let crossbow = &captured.entries[&"weapon_group:crossbow".parse().unwrap()];
        let SupplementBody::Specialization(prose) = &crossbow.body else {
            panic!("crossbow specialization");
        };
        assert!(prose.contains("1d8 persistent bleed damage"));
        assert!(prose.contains("item bonus to attack rolls"));
        assert!(!prose.contains("{@"));
        assert_eq!(captured.entries.len(), 28);
        assert_eq!(
            captured
                .entries
                .keys()
                .filter(|key| key.namespace() == DefinitionNamespace::Language)
                .count(),
            24
        );
    }
}
