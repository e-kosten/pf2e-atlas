use std::path::Path;

use atlas_domain::{PackName, Rarity, RecordId, RecordKey};
use atlas_record::{
    ActivationTimeSourceField, AtlasRecord, ContentSourceKind, DurationTimeSourceField,
    FoundryDocumentMechanics, FoundryDocumentType, FoundryRecordInfo, FoundryRecordType,
    RecordActivationTiming, RecordClassification, RecordContent, RecordContentDocument,
    RecordDurationTiming, RecordIdentity, RecordMechanics, RecordProvenance, RecordPublication,
    RecordRequirements, RecordTaxonomy, RecordTiming, RecordVisibility,
};
use serde_json::Value;

mod content;
mod content_diagnostics;
mod content_sources;
#[cfg(test)]
mod content_tests;
mod embedded_items;
mod journal_pages;
mod json;
mod kind;
mod publication;
#[cfg(test)]
mod source_facts_tests;
mod system;
mod text;
mod time;

use content_sources::extract_content_sources;
use embedded_items::{attach_embedded_content_refs, extract_embedded_item_facts};
use journal_pages::extract_journal_page_facts;

#[cfg(test)]
pub(crate) use content::parse_foundry_content;
pub(crate) use content::{LocalizationResolver, parse_foundry_content_with_localization};
pub(crate) use content_diagnostics::{ContentParseDiagnostics, DroppedContentMacro};
pub(crate) use json::{
    normalized_pointer_string, pointer_bool, pointer_i64, pointer_string, string_array_at_pointer,
    string_field, typed_collection,
};
pub(crate) use kind::classify_record;
pub(crate) use publication::publication_family;
pub(crate) use system::{
    extract_damage_types, extract_disable_skills, extract_prerequisites, extract_sense_types,
    extract_speed_types, extract_traits, normalize_price_cp, parse_bulk_value,
    parse_hands_requirement,
};
pub(crate) use text::normalize_text;
pub(crate) use time::{normalize_activation_time, normalize_time_text};

use crate::error::IngestError;
use crate::records::metrics;
use crate::records::{LoadedSourceRecord, SourceConstructionFacts, SourceRecordFacts};
use crate::source::ManifestPack;
#[cfg(test)]
use crate::source::dto::parse_serialized_source_object;
use crate::source::dto::{
    LegacyDuplicateDisposition, SerializedSourceMember, SerializedSourceObject,
    SerializedSourceValue, SourceIdentity, SpellDocumentSource, parse_hazard_source,
    parse_npc_source_from_serialized, parse_spell_document_source, pinned_source_version_metadata,
};
use crate::source::hazard_core::{HazardCoreConversion, convert_hazard_core};
use crate::source::mechanics;
use crate::source::npc_core::{NpcCoreConversion, convert_npc_core};
use crate::source::npc_entities::collect_npc_embedded_candidates;

pub(crate) fn normalize_record(
    manifest_pack: &ManifestPack,
    pack_name: &PackName,
    path: &Path,
    source_root: &Path,
    raw: Value,
    localization: Option<&dyn LocalizationResolver>,
) -> Result<LoadedSourceRecord, IngestError> {
    let source = SerializedSourceObject::from_json(&raw)
        .ok_or_else(|| normalization_error(path, "source document root must be an object"))?;
    normalize_record_from_source(
        manifest_pack,
        pack_name,
        path,
        source_root,
        source,
        localization,
    )
}

pub(crate) fn normalize_record_from_source(
    manifest_pack: &ManifestPack,
    pack_name: &PackName,
    path: &Path,
    source_root: &Path,
    source: SerializedSourceObject,
    localization: Option<&dyn LocalizationResolver>,
) -> Result<LoadedSourceRecord, IngestError> {
    let id = source_string(&source, "_id", path)?
        .ok_or_else(|| normalization_error(path, "missing _id"))?;
    let name = source_string(&source, "name", path)?
        .ok_or_else(|| normalization_error(path, "missing name"))?;
    let record_type = source_string(&source, "type", path)?
        .unwrap_or_else(|| manifest_pack.document_type.clone());
    let id = RecordId::new(id)
        .map_err(|error| normalization_error(path, &format!("invalid _id: {error}")))?;
    let key = RecordKey::new(pack_name.clone(), id.clone());
    let source_path = path
        .strip_prefix(source_root)
        .unwrap_or(path)
        .to_string_lossy()
        .to_string();
    let record_kind =
        classify_record(&manifest_pack.document_type, &record_type).ok_or_else(|| {
            normalization_error(
                path,
                &format!(
                    "unsupported Foundry record taxonomy: {}|{}",
                    manifest_pack.document_type, record_type
                ),
            )
        })?;
    let spell_source = (manifest_pack.document_type == "Item"
        && matches!(record_type.as_str(), "spell" | "consumable"))
    .then(|| {
        parse_spell_document_source(
            &source,
            &SourceIdentity::new(key.to_string(), source_path.clone()),
        )
        .map_err(|error| normalization_error(path, &error.to_string()))
    })
    .transpose()?
    .flatten();
    let hazard = manifest_pack.document_type == "Actor" && record_type == "hazard";
    let hazard_melee_damage_paths = if hazard {
        hazard_melee_damage_rolls_paths(&source)
    } else {
        Vec::new()
    };
    let mut duplicate_policy = |object_path: &str, key: &str| {
        if spell_source
            .as_ref()
            .is_some_and(|spell| spell_duplicate_is_retained(spell, object_path, key))
            || hazard_melee_damage_paths
                .iter()
                .any(|path| path == object_path)
        {
            LegacyDuplicateDisposition::OmitAfterFamilyRetention
        } else {
            LegacyDuplicateDisposition::Reject
        }
    };
    let raw = source
        .to_legacy_json(&mut duplicate_policy)
        .map_err(|error| normalization_error(path, &error.to_string()))?;
    let npc_source = if manifest_pack.document_type == "Actor" && record_type == "npc" {
        Some(
            parse_npc_source_from_serialized(
                pinned_source_version_metadata(),
                SourceIdentity::new(key.to_string(), source_path.clone()),
                raw.clone(),
                &source,
            )
            .map_err(|error| normalization_error(path, &error.to_string()))?,
        )
    } else {
        None
    };
    let hazard_source = if manifest_pack.document_type == "Actor" && record_type == "hazard" {
        Some(
            parse_hazard_source(
                pinned_source_version_metadata(),
                SourceIdentity::new(key.to_string(), source_path.clone()),
                raw.clone(),
                &source,
            )
            .map_err(|error| normalization_error(path, &error.to_string()))?,
        )
    } else {
        None
    };
    let npc_embedded_candidates = npc_source.as_ref().map(collect_npc_embedded_candidates);
    let npc_conversion = npc_source
        .as_ref()
        .map(|source| convert_npc_core(key.clone(), &source_path, source))
        .transpose()
        .map_err(|error| normalization_error(path, &error.to_string()))?;
    let hazard_conversion = hazard_source
        .as_ref()
        .map(|source| convert_hazard_core(key.clone(), &source_path, source, localization))
        .transpose()
        .map_err(|error| normalization_error(path, &error.to_string()))?;
    let canonical_creature = match npc_conversion.as_ref().map(|conversion| &conversion.body) {
        None => None,
        Some(atlas_record::RecordBody::Creature(creature)) => Some(creature),
        Some(atlas_record::RecordBody::Hazard(_) | atlas_record::RecordBody::Spell(_)) => {
            return Err(normalization_error(
                path,
                "NPC conversion did not produce a creature body",
            ));
        }
    };
    let canonical_hazard =
        hazard_conversion
            .as_ref()
            .and_then(|conversion| match &conversion.body {
                atlas_record::RecordBody::Hazard(hazard) => Some(hazard),
                atlas_record::RecordBody::Creature(_) | atlas_record::RecordBody::Spell(_) => None,
            });
    let level = if let Some(creature) = canonical_creature {
        creature.level.value.as_value().copied()
    } else if let Some(hazard) = canonical_hazard {
        hazard.level.typed().copied()
    } else {
        pointer_i64(&raw, "/system/level/value").or_else(|| {
            (manifest_pack.document_type == "Actor")
                .then(|| pointer_i64(&raw, "/system/details/level/value"))
                .flatten()
        })
    };
    let rarity = if let Some(creature) = canonical_creature {
        creature.rarity.value.as_value().cloned()
    } else if let Some(hazard) = canonical_hazard {
        hazard.rarity.typed().copied()
    } else {
        normalized_pointer_string(&raw, "/system/traits/rarity")
            .map(|value| {
                Rarity::from_canonical(&value).ok_or_else(|| {
                    normalization_error(
                        path,
                        &format!("unsupported rarity `{value}` at /system/traits/rarity"),
                    )
                })
            })
            .transpose()?
    };
    let traits = if let Some(creature) = canonical_creature {
        creature
            .traits
            .value
            .as_value()
            .map(|traits| {
                traits
                    .iter()
                    .map(|value| value.as_str().to_string())
                    .collect()
            })
            .unwrap_or_default()
    } else if let Some(hazard) = canonical_hazard {
        hazard
            .traits
            .typed()
            .map(|traits| {
                traits
                    .iter()
                    .map(|value| value.as_str().to_string())
                    .collect()
            })
            .unwrap_or_default()
    } else {
        extract_traits(&raw)
    };
    let prerequisites = extract_prerequisites(&raw);
    let system_category = normalized_pointer_string(&raw, "/system/category");
    let system_group = normalized_pointer_string(&raw, "/system/group");
    let system_base_item = normalized_pointer_string(&raw, "/system/baseItem");
    let system_usage = normalized_pointer_string(&raw, "/system/usage/value");
    let system_price_json = raw
        .pointer("/system/price/value")
        .map(serde_json::to_string)
        .transpose()
        .map_err(|error| normalization_error(path, &format!("price JSON failed: {error}")))?;
    let system_actions_value = pointer_i64(&raw, "/system/actions/value");
    let system_time_value = normalized_pointer_string(&raw, "/system/time/value");
    let system_duration_value = normalized_pointer_string(&raw, "/system/duration/value");
    let price_cp = normalize_price_cp(raw.pointer("/system/price/value"));
    let activation_time =
        normalize_activation_time(system_actions_value, system_time_value.as_deref());
    let duration = system_duration_value
        .as_deref()
        .and_then(normalize_time_text);
    let metrics = if hazard_source.is_some() || record_kind == atlas_domain::RecordKind::Spell {
        Vec::new()
    } else {
        metrics::extract_metrics(&raw, &manifest_pack.document_type, &record_type)
            .map_err(|message| normalization_error(path, &message))?
    };
    let actor_data =
        (manifest_pack.document_type == "Actor" && record_type != "npc" && record_type != "hazard")
            .then(|| mechanics::extract_actor_mechanics(&raw, localization));
    let item_data = (manifest_pack.document_type == "Item"
        && record_kind != atlas_domain::RecordKind::Spell)
        .then(|| {
            mechanics::extract_item_mechanics(
                &raw,
                system_category.clone(),
                system_base_item.clone(),
                system_group.clone(),
                system_usage.clone(),
                system_price_json.clone(),
                price_cp,
            )
        });
    let publication_title = if let Some(creature) = canonical_creature {
        creature
            .publication
            .value
            .as_value()
            .and_then(|publication| publication.title.as_value().cloned())
    } else if let Some(hazard) = canonical_hazard {
        hazard
            .publication
            .typed()
            .and_then(|publication| publication.title.typed().cloned())
    } else {
        pointer_string(&raw, "/system/publication/title")
            .or_else(|| pointer_string(&raw, "/system/details/publication/title"))
    };
    let publication_remaster = if let Some(creature) = canonical_creature {
        creature
            .publication
            .value
            .as_value()
            .and_then(|publication| publication.remaster.as_value().copied())
            .unwrap_or(false)
    } else if let Some(hazard) = canonical_hazard {
        hazard
            .publication
            .typed()
            .and_then(|publication| publication.remaster.typed().copied())
            .unwrap_or(false)
    } else {
        pointer_bool(&raw, "/system/publication/remaster")
            .or_else(|| pointer_bool(&raw, "/system/details/publication/remaster"))
            .unwrap_or(false)
    };
    let hazard_identities = hazard_conversion
        .as_ref()
        .map(|conversion| conversion.embedded_identities.as_slice());
    let content_sources = extract_content_sources(&raw, localization, hazard_identities);
    let mut source_facts = SourceRecordFacts {
        slug: normalized_pointer_string(&raw, "/system/slug"),
        compendium_source: normalized_pointer_string(&raw, "/_stats/compendiumSource"),
        ..SourceRecordFacts::default()
    };
    for (local_key, document) in &content_sources.supplemental_content {
        if let Some(local_key) = local_key {
            source_facts
                .source_content
                .insert(local_key.clone(), document.clone());
        }
    }
    source_facts.content_sources = content_sources.owned_content.clone();
    let mut supplemental_content = content_sources
        .supplemental_content
        .into_iter()
        .map(|(_, document)| document)
        .filter(|document| {
            npc_conversion.is_none()
                || !matches!(
                    document.source_kind,
                    ContentSourceKind::EmbeddedItemDescription
                        | ContentSourceKind::EmbeddedGmDescription
                        | ContentSourceKind::EmbeddedSpellDescription
                )
        })
        .filter(|document| {
            !matches!(
                spell_source.as_ref(),
                Some(SpellDocumentSource::ConsumableChild(_))
            ) || document.source_kind != ContentSourceKind::EmbeddedSpellDescription
        })
        .collect::<Vec<_>>();
    source_facts.embedded_items = extract_embedded_item_facts(&raw, &key, hazard_identities);
    attach_embedded_content_refs(
        &mut source_facts.embedded_items,
        &source_facts.source_content,
    );
    let (journal_pages, skipped_journal_pages, journal_diagnostics) =
        extract_journal_page_facts(&raw, &key, localization);
    source_facts.journal_pages = journal_pages;
    source_facts.skipped_journal_pages = skipped_journal_pages;
    let folder_id = pointer_string(&raw, "/folder");
    let raw_json = serde_json::to_string(&raw).map_err(|error| {
        normalization_error(path, &format!("raw JSON serialization failed: {error}"))
    })?;
    let publication_family = publication_family(pack_name.as_str(), publication_title.as_deref());
    let activation = activation_time.map(|time| {
        let source_field = if system_actions_value.is_some() {
            ActivationTimeSourceField::ActionsValue
        } else {
            ActivationTimeSourceField::TimeValue
        };
        RecordActivationTiming { time, source_field }
    });
    let duration = duration.map(|time| RecordDurationTiming {
        time,
        source_field: DurationTimeSourceField::DurationValue,
    });
    let mut content_documents = Vec::new();
    if let Some(document) = content_sources.description {
        content_documents.push(RecordContentDocument {
            source_kind: ContentSourceKind::Description,
            label: None,
            document,
        });
    }
    if let Some(document) = content_sources.blurb {
        content_documents.push(RecordContentDocument {
            source_kind: ContentSourceKind::Blurb,
            label: None,
            document,
        });
    }
    content_documents.extend(std::mem::take(&mut supplemental_content));
    let document_mechanics = if let Some(actor) = actor_data {
        FoundryDocumentMechanics::Actor(actor)
    } else if let Some(item) = item_data {
        FoundryDocumentMechanics::Item(item)
    } else {
        FoundryDocumentMechanics::None
    };
    let record = AtlasRecord {
        identity: RecordIdentity { key, name },
        classification: RecordClassification {
            kind: record_kind,
            level,
            rarity,
            traits,
            taxonomy: RecordTaxonomy::default(),
        },
        foundry: FoundryRecordInfo {
            pack_label: manifest_pack.label.clone(),
            document_type: FoundryDocumentType::from_foundry(&manifest_pack.document_type),
            record_type: FoundryRecordType::from_foundry(&record_type),
            folder_id,
        },
        provenance: RecordProvenance {
            source_path,
            raw_json: Some(raw_json),
        },
        publication: RecordPublication {
            title: publication_title,
            remaster: publication_remaster,
            category: publication_family,
        },
        requirements: RecordRequirements { prerequisites },
        timing: RecordTiming {
            activation,
            duration,
        },
        mechanics: RecordMechanics {
            metrics,
            document: document_mechanics,
        },
        content: RecordContent {
            documents: content_documents,
        },
        variant: None,
        visibility: RecordVisibility::default(),
    };
    let (mut canonical_body, npc_core_diagnostics) =
        if let Some(NpcCoreConversion {
            body, diagnostics, ..
        }) = npc_conversion
        {
            (Some(body), diagnostics)
        } else if let Some(HazardCoreConversion {
            body,
            embedded_identities: _,
        }) = hazard_conversion
        {
            (Some(body), Vec::new())
        } else {
            (None, Vec::new())
        };
    if let Some(SpellDocumentSource::Standalone(source)) = spell_source.clone() {
        canonical_body = Some(
            super::spells::convert_standalone_spell(
                record.identity.key.clone(),
                &record.provenance.source_path,
                source,
            )
            .map_err(|message| normalization_error(path, &message))?,
        );
    }
    let facts = SourceConstructionFacts {
        content_parse_diagnostics: content_sources
            .diagnostics
            .into_iter()
            .chain(journal_diagnostics)
            .collect(),
        source_facts,
        npc_source,
        hazard_source,
        spell_source,
        canonical_body,
        canonical_spell_children: Vec::new(),
        npc_core_diagnostics,
        npc_embedded_candidates,
        npc_embedded_diagnostics: Vec::new(),
        generated_affliction_role: None,
    };

    Ok(LoadedSourceRecord::new(record, facts))
}

#[cfg(test)]
pub(crate) fn normalize_record_from_source_bytes(
    manifest_pack: &ManifestPack,
    pack_name: &PackName,
    path: &Path,
    source_root: &Path,
    source_bytes: &[u8],
    localization: Option<&dyn LocalizationResolver>,
) -> Result<LoadedSourceRecord, IngestError> {
    let source = parse_serialized_source_object(source_bytes)
        .map_err(|error| normalization_error(path, &error))?;
    normalize_record_from_source(
        manifest_pack,
        pack_name,
        path,
        source_root,
        source,
        localization,
    )
}

fn source_string(
    source: &SerializedSourceObject,
    key: &str,
    path: &Path,
) -> Result<Option<String>, IngestError> {
    match source.member(key) {
        SerializedSourceMember::Missing | SerializedSourceMember::Null => Ok(None),
        SerializedSourceMember::Value(value) => {
            value.string().map(str::to_string).map(Some).ok_or_else(|| {
                normalization_error(path, &format!("source member `{key}` must be a string"))
            })
        }
        SerializedSourceMember::Duplicate(values) => Err(normalization_error(
            path,
            &format!(
                "duplicate source identity/discriminator member `{key}`: [{}]",
                values
                    .iter()
                    .map(|value| value.compact_json())
                    .collect::<Vec<_>>()
                    .join(",")
            ),
        )),
    }
}

fn spell_duplicate_is_retained(spell: &SpellDocumentSource, object_path: &str, key: &str) -> bool {
    if key == "gm"
        && matches!(
            object_path,
            "/system/description" | "/system/spell/system/description"
        )
    {
        return false;
    }
    match spell {
        SpellDocumentSource::Standalone(_) => {
            object_path.is_empty() && key == "img" || within_pointer_subtree(object_path, "/system")
        }
        SpellDocumentSource::ConsumableChild(_) => {
            object_path == "/system/spell" && key == "img"
                || within_pointer_subtree(object_path, "/system/spell/system")
        }
    }
}

fn within_pointer_subtree(path: &str, root: &str) -> bool {
    path == root
        || path
            .strip_prefix(root)
            .is_some_and(|suffix| suffix.starts_with('/'))
}

fn hazard_melee_damage_rolls_paths(source: &SerializedSourceObject) -> Vec<String> {
    let SerializedSourceMember::Value(SerializedSourceValue::Array(items)) = source.member("items")
    else {
        return Vec::new();
    };
    items
        .iter()
        .enumerate()
        .filter_map(|(source_ordinal, item)| {
            let item = item.object()?;
            matches!(
                item.member("type"),
                SerializedSourceMember::Value(value) if value.string() == Some("melee")
            )
            .then(|| format!("/items/{source_ordinal}/system/damageRolls"))
        })
        .collect()
}

pub(crate) fn normalization_error(path: &Path, message: &str) -> IngestError {
    IngestError::RecordNormalizationFailed {
        path: path.display().to_string(),
        message: message.to_string(),
    }
}

#[cfg(test)]
mod lossless_source_tests {
    use super::*;

    fn item_pack() -> ManifestPack {
        ManifestPack {
            name: "spells-srd".to_string(),
            label: "Spells".to_string(),
            document_type: "Item".to_string(),
            path: "packs/spells".to_string(),
        }
    }

    fn actor_pack() -> ManifestPack {
        ManifestPack {
            name: "hazards".to_string(),
            label: "Hazards".to_string(),
            document_type: "Actor".to_string(),
            path: "packs/hazards".to_string(),
        }
    }

    fn normalization_failure(pack: &ManifestPack, source: &[u8]) -> String {
        normalize_record_from_source_bytes(
            pack,
            &PackName::new(pack.name.clone()).expect("pack"),
            Path::new("packs/duplicate.json"),
            Path::new("."),
            source,
            None,
        )
        .expect_err("unowned duplicate must fail")
        .to_string()
    }

    #[test]
    fn duplicate_root_dispatch_discriminator_fails_before_legacy_projection() {
        let source_root = Path::new(".");
        let path = Path::new("packs/spells/duplicate-type.json");
        let error = normalize_record_from_source_bytes(
            &item_pack(),
            &PackName::new("spells-srd").expect("pack"),
            path,
            source_root,
            br#"{"_id":"duplicateType","name":"Duplicate Type","type":"spell","type":"consumable","system":{}}"#,
            None,
        )
        .expect_err("duplicate root type must fail");
        assert!(
            error
                .to_string()
                .contains("duplicate source identity/discriminator member `type`")
        );
    }

    #[test]
    fn standalone_spell_rejects_unowned_root_duplicate() {
        let error = normalization_failure(
            &item_pack(),
            br#"{"_id":"s","name":"S","type":"spell","folder":"first","folder":"second","system":{"rules":[]}}"#,
        );
        assert!(error.contains("duplicate source member at /folder"));
    }

    #[test]
    fn consumable_spell_rejects_unowned_child_envelope_duplicate() {
        let error = normalization_failure(
            &item_pack(),
            br#"{"_id":"c","name":"C","type":"consumable","system":{"spell":{"_id":"s","name":"S","type":"spell","sort":1,"sort":2,"system":{"rules":[]}}}}"#,
        );
        assert!(error.contains("duplicate source member at /system/spell/sort"));
    }

    #[test]
    fn consumable_spell_rejects_spell_prefix_neighbor_duplicate() {
        let error = normalization_failure(
            &item_pack(),
            br#"{"_id":"c","name":"C","type":"consumable","system":{"spell":{"_id":"s","name":"S","type":"spell","system":{"rules":[]}},"spellbook":{"value":1,"value":2}}}"#,
        );
        assert!(error.contains("duplicate source member at /system/spellbook/value"));
    }

    #[test]
    fn hazard_rejects_non_melee_damage_map_duplicate() {
        let error = normalization_failure(
            &actor_pack(),
            br#"{"_id":"h","name":"H","type":"hazard","items":[{"_id":"a","name":"A","type":"action","system":{"damageRolls":{"same":{"damage":"1d6"},"same":{"damage":"2d6"}}}}],"system":{}}"#,
        );
        assert!(error.contains("duplicate source member at /items/0/system/damageRolls/same"));
    }

    #[test]
    fn spell_rejects_duplicate_gm_content_owned_by_legacy_projection() {
        for (source, path) in [
            (
                &br#"{"_id":"s","name":"S","type":"spell","system":{"description":{"gm":"first","gm":"second","value":"public"},"rules":[]}}"#[..],
                "/system/description/gm",
            ),
            (
                &br#"{"_id":"c","name":"C","type":"consumable","system":{"spell":{"_id":"s","name":"S","type":"spell","system":{"description":{"gm":"first","gm":"second","value":"public"},"rules":[]}}}}"#[..],
                "/system/spell/system/description/gm",
            ),
        ] {
            let error = normalization_failure(&item_pack(), source);
            assert!(error.contains(&format!("duplicate source member at {path}")));
        }
    }
}
