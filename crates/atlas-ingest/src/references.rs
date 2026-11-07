//! Selected reference catalog preparation; records retain their existing content authority.
use crate::{
    CapturedBuildInputs, IngestError, REFERENCE_PREPARATION_RECIPE,
    source::localization::LocalizationCatalog,
};
use atlas_domain::{DefinitionKey, DefinitionNamespace, RecordKey};
use atlas_foundry_model::generated::{
    FoundryReferenceDefinition, reference_definitions, trait_definitions,
};
use atlas_index::{
    BuildFrameStream, OwnedAbilityCatalog, PublicationSpool, ReferenceAvailability,
    ReferenceBacking, ReferenceEntry, ReferenceGrantScope, ReferenceGrantUse, ReferencePermission,
    ReferencePermissionDisposition, ReferenceProvenance, ReferencePublication,
    ReferenceRecordMapping, ReferenceSourceLocation,
};
use atlas_record::{
    source_content::{LocalizationResolver, escape_content_text, prepare_reference_definition},
    source_record::{
        RecordLocalizationOccurrence, SourceBackedRecord, SourceNodeView, SourceReferenceIndex,
        owned_action_localizations, root_description_localizations,
    },
};
use serde::Deserialize;
use std::{
    collections::{BTreeMap, BTreeSet},
    path::Path,
};

fn invalid(error: impl ToString) -> IngestError {
    IngestError::SourceSelectionFailed(error.to_string())
}
fn index(error: atlas_index::IndexError) -> IngestError {
    IngestError::ArtifactWriteFailed(error.to_string())
}

fn grant_uses(grants: &[String]) -> Vec<ReferenceGrantUse> {
    grants
        .iter()
        .map(|license| ReferenceGrantUse {
            license: license.clone(),
            scope: if license == "CUP" {
                ReferenceGrantScope::DescriptiveReferences
            } else {
                ReferenceGrantScope::FunctionalContent
            },
        })
        .collect()
}

#[derive(Deserialize)]
struct SourcePolicies {
    schema: u32,
    recipe: String,
    rows: Vec<SourcePolicy>,
}
#[derive(Deserialize)]
struct SourcePolicy {
    keys: Vec<DefinitionKey>,
    #[serde(flatten)]
    attribution: SourceAttribution,
}
#[derive(Clone, Deserialize)]
struct SourceAttribution {
    publications: Vec<ReferencePublication>,
    grants: Vec<String>,
    notices: Vec<String>,
}
struct MappingRecord {
    family: Option<String>,
    kind: String,
    name: Option<String>,
    root_localizations: BTreeSet<String>,
}

pub(crate) struct ReferenceBuild<'a> {
    inputs: &'a CapturedBuildInputs,
    catalog: &'a LocalizationCatalog,
    applicable: bool,
    preparation: Option<atlas_index::ReferencePreparation>,
    audience: atlas_record::source_content::ContentAudience,
    declared_item_packs: BTreeSet<String>,
    records: BTreeMap<RecordKey, MappingRecord>,
    expected_records: BTreeSet<RecordKey>,
    occurrences: BuildFrameStream<Vec<RecordLocalizationOccurrence>>,
    policies: BTreeMap<DefinitionKey, SourceAttribution>,
}

impl<'a> ReferenceBuild<'a> {
    pub(crate) fn new(
        inputs: &'a CapturedBuildInputs,
        catalog: &'a LocalizationCatalog,
        scratch: &Path,
        context: &atlas_index::SourceArtifactBuildContext,
    ) -> Result<Self, IngestError> {
        let manifest = &inputs.source().manifest().manifest;
        let applicable = manifest
            .metadata
            .get("id")
            .and_then(serde_json::Value::as_str)
            == Some("pf2e");
        let policies: SourcePolicies =
            serde_json::from_slice(crate::reference_notices::SOURCE_POLICY).map_err(invalid)?;
        if policies.schema != 2 || policies.recipe != "foundry-reference-source-policy/v2" {
            return Err(invalid("unsupported reference source-policy recipe"));
        }
        let mut selected_policies = BTreeMap::new();
        for SourcePolicy { keys, attribution } in policies.rows {
            for key in keys {
                if selected_policies.insert(key, attribution.clone()).is_some() {
                    return Err(invalid("duplicate reference source-policy key"));
                }
            }
        }
        Ok(Self {
            inputs,
            catalog,
            applicable,
            audience: context.audience,
            preparation: context.reference_preparation.clone(),
            declared_item_packs: manifest
                .packs
                .iter()
                .filter(|pack| pack.document_type == "Item")
                .map(|pack| pack.name.clone())
                .collect(),
            records: BTreeMap::new(),
            expected_records: reference_definitions()
                .iter()
                .filter_map(|entry| entry.record_key)
                .map(str::parse::<RecordKey>)
                .collect::<Result<BTreeSet<_>, _>>()
                .map_err(invalid)?,
            occurrences: BuildFrameStream::new_in(scratch).map_err(index)?,
            policies: selected_policies,
        })
    }

    /// Retain compact mapping evidence and neutral authored occurrences while the
    /// checked record is already in hand. Never replay all rich records later.
    pub(crate) fn observe_record(
        &mut self,
        record: &SourceBackedRecord,
    ) -> Result<(), IngestError> {
        if self.expected_records.contains(record.key()) {
            let node = SourceNodeView::from(record.source());
            self.records.insert(
                record.key().clone(),
                MappingRecord {
                    family: node.source_type().value().map(str::to_owned),
                    kind: node.document_kind().into(),
                    name: node.name().value().cloned(),
                    root_localizations: root_description_localizations(record, self.audience)
                        .collect(),
                },
            );
        }
        if self.pack_applies(record.key().pack().as_str()) {
            let occurrences = owned_action_localizations(record, self.audience);
            if !occurrences.is_empty() {
                self.occurrences.push(&occurrences).map_err(index)?;
            }
        }
        Ok(())
    }

    fn pack_applies(&self, pack: &str) -> bool {
        atlas_index::reference_pack_applies(self.preparation.as_ref(), "pf2e", pack)
    }

    pub(crate) fn publish_catalog(
        &mut self,
        spool: &mut PublicationSpool,
        resolver: &SourceReferenceIndex,
    ) -> Result<(), IngestError> {
        let notices = &self.inputs.reference_notices().notices;
        for notice in notices.values() {
            spool.reference_notices.push(notice).map_err(index)?;
        }
        let mut abilities = OwnedAbilityCatalog::default();
        for authored in reference_definitions()
            .iter()
            .filter(|entry| !entry.excluded)
        {
            let entry = self.entry(authored, resolver, notices)?;
            abilities.add_entry(&entry);
            spool.reference_entries.push(&entry).map_err(index)?;
        }
        for authored in trait_definitions() {
            let key = atlas_domain::TraitKey::new(authored.identifier).map_err(invalid)?;
            abilities.add_trait(key.into(), authored.label_key, authored.description_key);
        }
        self.occurrences
            .visit(&mut |occurrences| {
                for evidence in abilities.project(occurrences) {
                    spool.owned_ability_evidence.push(&evidence)?;
                }
                Ok(())
            })
            .map_err(index)
    }

    fn entry(
        &self,
        authored: &FoundryReferenceDefinition,
        resolver: &SourceReferenceIndex,
        notices: &BTreeMap<String, atlas_index::ReferenceNotice>,
    ) -> Result<ReferenceEntry, IngestError> {
        let key: DefinitionKey = format!("{}:{}", authored.namespace, authored.identifier)
            .parse()
            .map_err(invalid)?;
        let supplemental = self
            .applicable
            .then(|| self.inputs.reference_supplement().entries.get(&key))
            .flatten();
        let target = authored
            .record_key
            .map(str::parse::<RecordKey>)
            .transpose()
            .map_err(invalid)?;
        let record = target.as_ref().and_then(|key| self.records.get(key));
        let label = record
            .and_then(|record| record.name.clone())
            .or_else(|| {
                authored
                    .label_key
                    .and_then(|key| self.catalog.localized_value(key))
                    .map(str::to_owned)
            })
            .or_else(|| authored.display_label.map(str::to_owned))
            .unwrap_or_else(|| authored.identifier.to_owned());
        let mut entry = ReferenceEntry {
            key: key.clone(),
            label,
            backing: ReferenceBacking::None,
            availability: ReferenceAvailability::LabelOnly,
            provenance: ReferenceProvenance {
                source_package: if self.applicable {
                    "pf2e"
                } else {
                    self.inputs
                        .source()
                        .manifest()
                        .manifest
                        .metadata
                        .get("id")
                        .and_then(serde_json::Value::as_str)
                        .unwrap_or("unknown")
                }
                .into(),
                source_revision: self.inputs.source_revision().map(str::to_owned),
                source_repository: None,
                source_file: Some(authored.source_file.into()),
                source_sha256: None,
                label_key: authored.label_key.map(str::to_owned),
                description_key: authored.description_key.map(str::to_owned),
                catalogs: vec![authored.namespace.into()],
                source_locations: vec![ReferenceSourceLocation {
                    file: authored.source_file.into(),
                    line: authored.source_line,
                    column: 1,
                }],
                indexing_locale: self.catalog.locale.clone(),
                content_locale: None,
                publications: vec![],
                record_mapping: target.as_ref().map(|target| ReferenceRecordMapping {
                    target: target.clone(),
                    package: "pf2e".into(),
                    expected_document_kind: "Item".into(),
                    expected_item_type: authored.expected_family.map(str::to_owned),
                    required_localization_key: authored
                        .required_localization_key
                        .map(str::to_owned),
                }),
                recipe: REFERENCE_PREPARATION_RECIPE.into(),
            },
            permission: ReferencePermission {
                disposition: ReferencePermissionDisposition::ApprovedContent,
                body_grants: vec![],
                label_grants: if self.applicable && key.namespace() == DefinitionNamespace::Language
                {
                    grant_uses(&["CUP".into()])
                } else {
                    vec![]
                },
                notices: if self.applicable && key.namespace() == DefinitionNamespace::Language {
                    vec!["community-use".into()]
                } else {
                    vec![]
                },
            },
        };
        if !self.applicable {
            entry.availability = ReferenceAvailability::SourceInapplicable;
            return Ok(entry);
        }
        if let Some(target) = target {
            entry.permission.disposition = ReferencePermissionDisposition::ExistingRecordPolicy;
            entry.availability = match record {
                _ if !self.pack_applies(target.pack().as_str()) => {
                    ReferenceAvailability::SourceInapplicable
                }
                None => ReferenceAvailability::MissingTarget,
                Some(record)
                    if !self.declared_item_packs.contains(target.pack().as_str())
                        || record.kind != "Item"
                        || authored
                            .expected_family
                            .is_some_and(|family| record.family.as_deref() != Some(family)) =>
                {
                    ReferenceAvailability::WrongFamily
                }
                Some(record)
                    if authored
                        .required_localization_key
                        .is_some_and(|key| !record.root_localizations.contains(key)) =>
                {
                    ReferenceAvailability::ConflictingEvidence
                }
                Some(_) => {
                    entry.backing = ReferenceBacking::Record { key: target };
                    ReferenceAvailability::Available
                }
            };
            return Ok(entry);
        }
        let policy = self.policies.get(&key);
        let markup = if let Some(supplemental) = supplemental {
            entry.provenance.source_revision =
                Some(self.inputs.reference_supplement().source_commit.clone());
            entry.provenance.source_repository =
                Some("https://github.com/Pf2eToolsOrg/Pf2eTools".into());
            if let Some((file, hash)) = self
                .inputs
                .reference_supplement()
                .raw_source(key.namespace())
            {
                entry.provenance.source_file = Some(file.into());
                entry.provenance.source_sha256 = Some(hash.into());
            }
            entry.provenance.publications = vec![ReferencePublication {
                title: Some(supplemental.publication.clone()),
                page: Some(supplemental.page),
                license: Some("ORC".into()),
            }];
            entry.permission.body_grants = grant_uses(&supplemental.grants);
            entry.permission.notices.extend([
                "orc-license".into(),
                "player-core".into(),
                "pf2etools-formatting".into(),
            ]);
            if key.namespace() == DefinitionNamespace::Language {
                entry.permission.notices.push("community-use".into());
            }
            supplemental.markup.clone().or_else(|| {
                if key.namespace() == DefinitionNamespace::Language {
                    language_speaker_markup(&supplemental.speakers)
                } else {
                    None
                }
            })
        } else if let Some(description_key) = authored.description_key {
            let Some(markup) = self
                .catalog
                .english_value(description_key)
                .filter(|markup| !markup.trim().is_empty())
            else {
                entry.availability = ReferenceAvailability::MissingLocalizedBody;
                return Ok(entry);
            };
            let Some(policy) = policy else {
                entry.availability = ReferenceAvailability::PermissionUnresolved;
                entry.permission.disposition =
                    ReferencePermissionDisposition::MetadataOnlyUnresolved;
                return Ok(entry);
            };
            if let Some((file, hash)) = self.catalog.english_source(description_key) {
                entry.provenance.source_file = Some(file.into());
                entry.provenance.source_sha256 = Some(hash.into());
            }
            entry.provenance.publications = policy.publications.clone();
            entry.permission.body_grants = grant_uses(&policy.grants);
            entry
                .permission
                .notices
                .extend(policy.notices.iter().cloned());
            Some(markup.to_owned())
        } else {
            None
        };
        entry.permission.notices.sort();
        entry.permission.notices.dedup();
        if let Some(markup) = markup {
            if entry
                .permission
                .notices
                .iter()
                .any(|id| !notices.contains_key(id))
            {
                entry.availability = ReferenceAvailability::PermissionUnresolved;
                entry.permission.disposition =
                    ReferencePermissionDisposition::MetadataOnlyUnresolved;
                entry.permission.body_grants.clear();
                entry
                    .permission
                    .notices
                    .retain(|id| notices.contains_key(id));
                return Ok(entry);
            }
            let resolve = |kind: &atlas_record::source_content::ContentReferenceKind,
                           target: &str| {
                resolver.resolve_global_content_reference(kind, target)
            };
            let english = crate::source::localization::EnglishLocalization(self.catalog);
            let content = prepare_reference_definition(
                key,
                &markup,
                self.audience,
                Some(&english),
                Some(&resolve),
            )
            .map_err(invalid)?;
            entry.backing = ReferenceBacking::Prepared {
                authored_markup: markup,
                content,
            };
            entry.provenance.content_locale = Some("en".into());
            entry.availability = ReferenceAvailability::Available;
        }
        Ok(entry)
    }
}

fn language_speaker_markup(speakers: &[String]) -> Option<String> {
    let mut speakers = speakers.to_vec();
    let first = speakers.first_mut()?;
    // The captured table capitalizes its first cell. Only reviewed common-noun
    // phrases change case; proper names and new source wording stay verbatim.
    if matches!(
        first.as_str(),
        "Evil fey"
            | "Demons"
            | "Humans"
            | "Devils"
            | "Dragons"
            | "Dwarves"
            | "Elves"
            | "Angels"
            | "Fey"
            | "Gnomes"
            | "Goblins"
            | "Halflings"
            | "Giants"
            | "The hyena-like kholos"
            | "Wood elemental creatures"
            | "Ghouls"
            | "Orcs"
            | "Earth elemental creatures"
            | "Fire elemental creatures"
            | "Subterranean civilizations"
            | "Air elemental creatures"
            | "Metal elemental creatures"
            | "Aquatic creatures"
    ) {
        first.make_ascii_lowercase();
    }
    let (last, leading) = speakers.split_last()?;
    let text = if leading.is_empty() {
        last.clone()
    } else {
        format!("{} and {last}", leading.join(", "))
    };
    Some(format!(
        "<p><strong>Spoken by</strong> {}</p>",
        escape_content_text(&text)
    ))
}

#[cfg(test)]
mod tests;
