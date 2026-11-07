//! Small explicit glossary data for browser and HTTP acceptance, never runtime fallback.
use atlas_domain::{DefinitionKey, DefinitionNamespace};
use atlas_index::{
    IndexBuildInput, IndexError, ReferenceAvailability, ReferenceBacking, ReferenceCoverage,
    ReferenceEntry, ReferenceFilterBinding, ReferenceGrantScope, ReferenceGrantUse,
    ReferenceNotice, ReferencePermission, ReferencePermissionDisposition, ReferencePreparation,
    ReferenceProvenance, ReferencePublication, ReferenceRecordMapping,
};
use atlas_record::source_content::{
    ContentReferenceKind, ContentReferenceTarget, ResolvedContentReference,
};
use atlas_search::test_support::{hash, record};

pub const DARKVISION: &str = "bestiary-ability-glossary-srd:qCCLZhnp2HhP3Ex6";
pub fn darkvision_record() -> atlas_index::SourceArtifactRecordInput {
    record(
        "bestiary-ability-glossary-srd",
        "Item",
        serde_json::json!({
            "_id":"qCCLZhnp2HhP3Ex6", "type":"action", "name":"Darkvision",
            "system":{"actionType":{"value":"passive"},"description":{"value":"<p>Fixture darkvision reference content.</p>"}}
        }),
    )
}
pub struct ReferenceFixture {
    pub entries: Vec<ReferenceEntry>,
    pub notices: Vec<ReferenceNotice>,
    pub occurrences: Vec<atlas_index::DefinitionReferenceInput>,
}
fn metadata(key: DefinitionKey, label: &str, field: &str, value: &str) -> ReferenceEntry {
    ReferenceEntry {
        key,
        label: label.into(),
        availability: ReferenceAvailability::LabelOnly,
        backing: ReferenceBacking::None,
        provenance: ReferenceProvenance {
            source_package: "pf2e".into(),
            source_repository: None,
            source_revision: None,
            source_file: Some("fixtures/reference-catalog.json".into()),
            source_sha256: Some(hash(b"Atlas synthetic reference fixture/v1")),
            label_key: None,
            description_key: None,
            catalogs: vec!["fixture".into()],
            source_locations: vec![],
            indexing_locale: "en".into(),
            content_locale: Some("en".into()),
            publications: vec![],
            record_mapping: None,
            recipe: "reference-fixture/v1".into(),
        },
        permission: ReferencePermission {
            disposition: ReferencePermissionDisposition::ApprovedContent,
            body_grants: vec![],
            label_grants: vec![],
            notices: vec![],
        },
        filter_bindings: vec![ReferenceFilterBinding {
            field: field.into(),
            value: value.into(),
        }],
    }
}
pub fn attach(input: &mut IndexBuildInput) -> Result<ReferenceFixture, IndexError> {
    let mut entries = vec![];
    for (namespace, token, label, field) in [
        (
            DefinitionNamespace::Language,
            "common",
            "Common",
            "languages",
        ),
        (
            DefinitionNamespace::Language,
            "chthonian",
            "Chthonian",
            "languages",
        ),
        (
            DefinitionNamespace::Language,
            "draconic",
            "Draconic",
            "languages",
        ),
        (
            DefinitionNamespace::Sense,
            "darkvision",
            "Darkvision",
            "senses",
        ),
        (
            DefinitionNamespace::WeaponGroup,
            "sword",
            "Sword",
            "weapon.group",
        ),
        (
            DefinitionNamespace::ArmorGroup,
            "plate",
            "Plate",
            "armor.group",
        ),
        (DefinitionNamespace::Size, "med", "Medium", "body.size"),
        (DefinitionNamespace::Size, "lg", "Large", "body.size"),
        (
            DefinitionNamespace::Rarity,
            "uncommon",
            "Uncommon",
            "rarity",
        ),
        (
            DefinitionNamespace::Tradition,
            "arcane",
            "Arcane",
            "spell.traditions",
        ),
        (
            DefinitionNamespace::DamageType,
            "fire",
            "Fire",
            "weapon.damage.type",
        ),
    ] {
        let key = DefinitionKey::new(namespace, token)
            .map_err(|error| IndexError::Invalid(error.to_string()))?;
        entries.push(metadata(key, label, field, token));
    }
    for (key, localization) in [
        ("language:chthonian", "Fixture.Language.Chthonian"),
        ("sense:darkvision", "Fixture.Sense.Darkvision"),
    ] {
        entries
            .iter_mut()
            .find(|entry| entry.key.to_string() == key)
            .ok_or_else(|| IndexError::Invalid("fixture localization target is absent".into()))?
            .provenance
            .label_key = Some(localization.into());
    }
    let body = "<p>Typically spoken by demons. This sentence is synthetic test content. @UUID[Compendium.pf2e.bestiary-ability-glossary-srd.Item.qCCLZhnp2HhP3Ex6]{Darkvision}</p>";
    let resolve = |kind: &ContentReferenceKind, target: &str| {
        (*kind == ContentReferenceKind::Uuid
            && target == "Compendium.pf2e.bestiary-ability-glossary-srd.Item.qCCLZhnp2HhP3Ex6")
            .then(|| ResolvedContentReference {
                target: ContentReferenceTarget::Record {
                    key: DARKVISION.parse().unwrap(),
                },
                display_name: Some("Darkvision".into()),
                item_family: Some("action".into()),
            })
    };
    let chthonian = entries
        .iter_mut()
        .find(|entry| entry.key.to_string() == "language:chthonian")
        .ok_or_else(|| IndexError::Invalid("fixture language is absent".into()))?;
    chthonian.backing = ReferenceBacking::Prepared {
        authored_markup: body.into(),
        content: atlas_record::source_content::prepare_reference_definition(
            chthonian.key.clone(),
            body,
            input.context.audience,
            None,
            Some(&resolve),
        )
        .map_err(|error| IndexError::Invalid(error.to_string()))?,
    };
    chthonian.availability = ReferenceAvailability::Available;
    chthonian.provenance.source_sha256 = Some(hash(body.as_bytes()));
    chthonian
        .provenance
        .publications
        .push(ReferencePublication {
            title: Some("Fixture rulebook".into()),
            page: Some(89),
            license: Some("CC0-1.0".into()),
        });
    chthonian.permission.body_grants.push(ReferenceGrantUse {
        license: "CC0-1.0".into(),
        scope: ReferenceGrantScope::FunctionalContent,
    });
    chthonian.permission.notices.push("fixture-content".into());
    let sense = entries
        .iter_mut()
        .find(|entry| entry.key.to_string() == "sense:darkvision")
        .ok_or_else(|| IndexError::Invalid("fixture sense is absent".into()))?;
    let target = DARKVISION
        .parse::<atlas_domain::RecordKey>()
        .map_err(|error| IndexError::Invalid(error.to_string()))?;
    sense.backing = ReferenceBacking::Record {
        key: target.clone(),
    };
    sense.availability = ReferenceAvailability::Available;
    sense.permission.disposition = ReferencePermissionDisposition::ExistingRecordPolicy;
    sense.provenance.record_mapping = Some(ReferenceRecordMapping {
        target,
        package: "pf2e".into(),
        expected_document_kind: "Item".into(),
        expected_item_type: Some("action".into()),
        required_localization_key: None,
    });
    let notice_text =
        "Synthetic test content created for the Atlas browser and HTTP fixtures; CC0-1.0.";
    let notices = vec![ReferenceNotice {
        id: "fixture-content".into(),
        sha256: hash(notice_text.as_bytes()),
        text: notice_text.into(),
        source_url: None,
    }];
    let keys = input
        .traits
        .iter()
        .map(|entry| DefinitionKey::from(entry.key.clone()))
        .chain(entries.iter().map(|entry| entry.key.clone()));
    let (expected_entry_count, entry_keys_sha256) =
        atlas_index::reference_inventory_identity(keys)?;
    input.context.reference_preparation = Some(ReferencePreparation {
        source_package: Some("pf2e".into()),
        pack_systems: input
            .packs
            .iter()
            .map(|pack| (pack.pack_id.clone(), Some("pf2e".into())))
            .collect(),
        recipe: "reference-fixture/v1".into(),
        input_hash: hash(&serde_json::to_vec(&(&entries, &notices))?),
        coverage: ReferenceCoverage::Complete,
        expected_entry_count,
        entry_keys_sha256,
    });
    let mut localizations = atlas_record::source_content::DefinitionLocalizationMap::new();
    let targets = entries
        .iter()
        .map(|entry| {
            if let Some(key) = &entry.provenance.label_key {
                localizations
                    .entry(key.clone())
                    .or_default()
                    .insert(entry.key.clone());
            }
            let target = match &entry.backing {
                ReferenceBacking::Record { key } => {
                    atlas_index::DefinitionReferenceTarget::Record {
                        key: key.clone(),
                        owners: vec![],
                    }
                }
                _ => atlas_index::DefinitionReferenceTarget::Definition {
                    key: entry.key.clone(),
                },
            };
            (entry.key.clone(), target)
        })
        .collect::<std::collections::BTreeMap<_, _>>();
    let occurrences = input
        .records
        .iter()
        .flat_map(|source| {
            atlas_record::source_record::definition_record_localization_references(
                &source.record,
                input.context.audience,
                &localizations,
            )
        })
        .map(|occurrence| atlas_index::DefinitionReferenceInput {
            source: atlas_index::DefinitionReferenceSource::Record {
                key: occurrence.locator.record,
                owners: occurrence.locator.owners,
                field: Some(occurrence.locator.field),
            },
            target: targets[&occurrence.key].clone(),
            occurrence_path: occurrence.path,
            evidence: occurrence.localization_key,
            label: None,
            visible: occurrence.visible,
        })
        .collect();
    Ok(ReferenceFixture {
        entries,
        notices,
        occurrences,
    })
}
