use atlas_domain::RecordKey;
use atlas_record::{
    ConsumableSpellChild, ConsumableSpellSourceContext, ContentId, ContentOrigin, ContentOwner,
    ContentProvenance, ContentRole, ContentSourceKind, DuplicateContentStatus, FactValue,
    OwnedRichContent, OwnedRichContentDocument, RecordBody, SpellDefinition, SpellIdentity,
    SpellProvenance, SpellRecord, SpellSourceContext, SpellSourceId, SpellSourceValue,
    SpellStandaloneTarget, UnsupportedSourceReason, UnsupportedSourceShape, UnsupportedSourceValue,
};

use crate::error::IngestError;
use crate::records::{LoadedSourceRecord, RecordReferenceIndex, SourceContentFact};

use super::dto::{ConsumableSpellChildSource, SpellDocumentSource, SpellItemSource};
use super::owned_content::source_diagnostics;

pub(crate) fn convert_standalone_spell(
    record_key: RecordKey,
    source_path: &str,
    source: SpellItemSource,
) -> Result<RecordBody, String> {
    let source_id = SpellSourceId::new(source.id.clone())
        .map_err(|_| format!("invalid spell source ID {:?}", source.id))?;
    let name = source.name.clone();
    Ok(RecordBody::Spell(SpellRecord {
        identity: SpellIdentity {
            record_key,
            source_id,
            name,
        },
        definition: definition(source, source_path, true),
    }))
}

pub(crate) fn finalize_consumable_spell_children(
    records: &mut [LoadedSourceRecord],
    index: &RecordReferenceIndex,
) -> Result<(), IngestError> {
    for loaded in records {
        let Some(SpellDocumentSource::ConsumableChild(source)) = loaded.facts.spell_source.clone()
        else {
            continue;
        };
        let child_content = loaded
            .facts
            .source_facts
            .content_sources
            .iter()
            .find(|content| {
                content.source_kind == ContentSourceKind::EmbeddedSpellDescription
                    && content.nested_source_id.as_deref() == Some(source.spell.id.as_str())
            })
            .cloned();
        let child = convert_consumable_spell_child(
            loaded.record.identity.key.clone(),
            &loaded.record.provenance.source_path,
            source,
            index,
            child_content,
        )
        .map_err(|message| IngestError::RecordNormalizationFailed {
            path: loaded.record.provenance.source_path.clone(),
            message,
        })?;
        loaded.facts.canonical_spell_children.push(child);
    }
    Ok(())
}

fn convert_consumable_spell_child(
    parent_record_key: RecordKey,
    parent_source_path: &str,
    source: ConsumableSpellChildSource,
    index: &RecordReferenceIndex,
    child_content: Option<SourceContentFact>,
) -> Result<ConsumableSpellChild, String> {
    let ConsumableSpellChildSource {
        authored_order,
        spell,
        standalone_locator,
    } = source;
    let child_id = atlas_record::SpellChildId::new(spell.id.clone())
        .map_err(|_| format!("invalid consumable spell child ID {:?}", spell.id))?;
    let standalone_target = resolve_standalone_target(&standalone_locator, index);
    let duplicate_status = match &standalone_target {
        FactValue::Value(SpellStandaloneTarget::Resolved(target_record_key)) => {
            DuplicateContentStatus::CopiedFromCanonicalTarget {
                target_record_key: target_record_key.clone(),
            }
        }
        _ => DuplicateContentStatus::Unique,
    };
    let location = spell.location.clone();
    let name = FactValue::Value(SpellSourceValue::Known(spell.name.clone()));
    let mut definition = definition(spell, &format!("{parent_source_path}#system.spell"), false);
    if let Some(source) = child_content {
        let content_key = atlas_record::ContentKey::new(source.content_key.clone())
            .map_err(|_| "invalid consumable spell content key".to_string())?;
        let diagnostics = source_diagnostics(&source);
        definition
            .content
            .documents
            .push(OwnedRichContentDocument::new(
                ContentId::new(parent_record_key.clone(), content_key),
                source.identity_stability,
                ContentOwner::Record(parent_record_key.clone()),
                ContentRole::EmbeddedCapability,
                ContentOrigin::RecordField {
                    source_kind: source.source_kind,
                    relative_source_path: source.relative_source_path.clone(),
                },
                source.source_kind.default_visibility(),
                ContentProvenance {
                    source_record_key: parent_record_key.clone(),
                    relative_source_path: parent_source_path.to_string(),
                    field_or_pointer_family: source.relative_source_path.clone(),
                    nested_source_id: source.nested_source_id.clone(),
                    authored_ordinal_or_range: source.authored_ordinal_or_range.clone(),
                    authored_label: source.label.clone(),
                },
                source.source_kind,
                source.authored_order,
                source.label,
                source.document,
                duplicate_status,
                diagnostics,
            ));
    }
    Ok(ConsumableSpellChild {
        parent_record_key,
        child_id,
        name,
        authored_order,
        location,
        standalone_locator,
        standalone_target,
        definition,
    })
}

fn definition(source: SpellItemSource, source_path: &str, standalone: bool) -> SpellDefinition {
    let consumable_child = if standalone {
        FactValue::Missing
    } else {
        FactValue::Value(ConsumableSpellSourceContext {
            slug: source.slug,
            publication_title: source.publication_title,
            publication_remaster: source.publication_remaster,
            rarity: source.rarity,
        })
    };
    SpellDefinition {
        source_context: SpellSourceContext {
            image: source.image,
            publication_license: source.publication_license,
            consumable_child,
        },
        classification: source.classification,
        casting: source.casting,
        targeting: source.targeting,
        defense: source.defense,
        damage: source.damage,
        duration: source.duration,
        heightening: source.heightening,
        overlays: source.overlays,
        ritual: source.ritual,
        rules: source.rules,
        content: OwnedRichContent::default(),
        unsupported_notes: source.unsupported_notes,
        provenance: SpellProvenance {
            source_path: source_path.to_string(),
            source_contract_version: super::dto::PF2E_SOURCE_CONTRACT_VERSION.to_string(),
            source_system_version: super::dto::PF2E_SOURCE_PINNED_SYSTEM_VERSION.to_string(),
            source_upstream_commit: super::dto::PF2E_SOURCE_PINNED_COMMIT.to_string(),
            standalone_location: if standalone {
                source.location_provenance
            } else {
                FactValue::Missing
            },
        },
    }
}

fn resolve_standalone_target(
    locator: &atlas_record::SpellFact<atlas_record::StableSourceLocator>,
    index: &RecordReferenceIndex,
) -> FactValue<SpellStandaloneTarget> {
    match locator {
        FactValue::Missing => FactValue::Missing,
        FactValue::Null => FactValue::Null,
        FactValue::Value(SpellSourceValue::Unsupported(value)) => {
            FactValue::Value(SpellStandaloneTarget::Unresolved(value.clone()))
        }
        FactValue::Value(SpellSourceValue::Known(locator)) => {
            match strict_pack_id(locator.as_str(), index) {
                Some(record_key) => FactValue::Value(SpellStandaloneTarget::Resolved(record_key)),
                None => {
                    FactValue::Value(SpellStandaloneTarget::Unresolved(UnsupportedSourceValue {
                        shape: UnsupportedSourceShape::String,
                        value: locator.as_str().to_string(),
                        reason: UnsupportedSourceReason::SourceFieldDrift,
                    }))
                }
            }
        }
    }
}

fn strict_pack_id(locator: &str, index: &RecordReferenceIndex) -> Option<RecordKey> {
    let parts = locator.split('.').collect::<Vec<_>>();
    let (pack, id) = match parts.as_slice() {
        ["Compendium", "pf2e", pack, "Item", id] => (*pack, *id),
        ["pf2e", pack, id] => (*pack, *id),
        _ => return None,
    };
    index
        .by_pack_id
        .get(&(pack.to_string(), id.to_string()))
        .cloned()
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use super::*;
    use crate::records::references::{build_record_reference_index, resolve_content_references};
    use crate::source::ManifestPack;
    use crate::source::normalize::normalize_record_from_source_bytes;
    use crate::source::owned_content::finalize_spell_owned_content;
    use atlas_domain::PackName;
    use atlas_record::{
        DuplicateContentStatus, FactValue, RecordBody, SpellFormContext, SpellFormField,
        SpellFormId, SpellResolvedField, SpellSourceValue, SpellStandaloneTarget,
        SpellUnsupportedSourceField, iter_foundry_links, render_plain_text,
    };
    use serde_json::Value;

    #[test]
    fn arboreal_wand_child_keeps_local_context_and_authenticates_heal_target() {
        let heal = normalize(
            "spells-srd",
            "packs/spells/1st-rank/heal.json",
            br#"{"_id":"rfZpqmj0AIIdkVIs","name":"Heal","type":"spell","system":{"area":null,"cost":{"value":""},"counteraction":false,"damage":{"0":{"applyMod":false,"category":null,"formula":"1d8","kinds":["damage","healing"],"materials":[],"type":"vitality"}},"defense":{"save":{"basic":true,"statistic":"fortitude"}},"description":{"value":"<p>Canonical Heal wording.</p>"},"duration":{"sustained":false,"value":""},"heightening":{"damage":{"0":"1d8"},"interval":1,"type":"interval"},"level":{"value":1},"range":{"value":"varies"},"requirements":"","rules":[],"target":{"value":"1 willing living creature or 1 undead"},"time":{"value":"1 to 3"},"traits":{"rarity":"common","traditions":["divine","primal"],"value":["healing","manipulate","vitality"]}}}"#,
        );
        let wand = normalize(
            "equipment-srd",
            "packs/equipment/arboreal-wand-rank-4.json",
            br#"{"_id":"eOtQtVRLeGH39dNx","name":"Arboreal Wand (Rank 4)","type":"consumable","system":{"description":{"value":"<p>You cast @UUID[Compendium.pf2e.spells-srd.Item.Heal] at the indicated rank.</p>"},"spell":{"_id":"7w37duycMs4YOBeu","flags":{"core":{"sourceId":"Compendium.pf2e.spells-srd.Item.rfZpqmj0AIIdkVIs"}},"name":"Heal","system":{"area":null,"cost":{"value":""},"counteraction":false,"damage":{"0":{"applyMod":false,"category":null,"formula":"1d8","kinds":["damage","healing"],"materials":[],"type":"vitality"}},"defense":{"save":{"basic":true,"statistic":"fortitude"}},"description":{"value":"<p>Embedded Heal wording with @Template[emanation|distance:30].</p>"},"duration":{"sustained":false,"value":""},"heightening":{"damage":{"0":"1d8"},"interval":1,"type":"interval"},"level":{"value":1},"location":{"heightenedLevel":4,"value":null},"range":{"value":"varies"},"requirements":"","rules":[],"target":{"value":"1 willing living creature or 1 undead"},"time":{"value":"1 to 3"},"traits":{"rarity":"common","traditions":["divine","primal"],"value":["healing","manipulate","vitality"]}},"type":"spell"},"traits":{"rarity":"common","value":["consumable","magical","wand"]},"uses":{"autoDestroy":false,"max":1,"value":1}}}"#,
        );
        let mut records = vec![heal, wand];
        let index = build_record_reference_index(&records);
        finalize_consumable_spell_children(&mut records, &index).expect("child conversion");
        finalize_spell_owned_content(&mut records);
        resolve_content_references(&mut records, &index);

        let canonical_heal = records[0]
            .facts
            .canonical_body
            .as_ref()
            .and_then(RecordBody::as_spell)
            .expect("canonical Heal");
        assert_eq!(canonical_heal.definition.content.documents.len(), 1);
        assert_eq!(records[0].record.content.documents.len(), 1);
        assert_eq!(
            canonical_heal.definition.content.documents[0]
                .id
                .content_key
                .as_str(),
            "description"
        );

        let child = &records[1].facts.canonical_spell_children[0];
        assert_eq!(
            child.parent_record_key.to_string(),
            "equipment-srd:eOtQtVRLeGH39dNx"
        );
        assert_eq!(child.child_id.as_str(), "7w37duycMs4YOBeu");
        assert_eq!(child.authored_order, 0);
        let location = child
            .location
            .as_value()
            .and_then(SpellSourceValue::as_known)
            .expect("authored child location");
        assert_eq!(location.value, FactValue::Null);
        assert_eq!(
            location
                .heightened_rank
                .as_value()
                .and_then(SpellSourceValue::as_known),
            Some(&4)
        );
        assert!(matches!(
            &child.standalone_target,
            FactValue::Value(SpellStandaloneTarget::Resolved(key))
                if key.to_string() == "spells-srd:rfZpqmj0AIIdkVIs"
        ));
        assert_eq!(child.definition.content.documents.len(), 1);
        let content = &child.definition.content.documents[0];
        assert!(matches!(
            &content.duplicate_status,
            DuplicateContentStatus::CopiedFromCanonicalTarget { target_record_key }
                if target_record_key.to_string() == "spells-srd:rfZpqmj0AIIdkVIs"
        ));
        assert_eq!(
            content.provenance.nested_source_id.as_deref(),
            Some("7w37duycMs4YOBeu")
        );
        assert_eq!(
            render_plain_text(&content.document),
            "Embedded Heal wording with 30-foot emanation."
        );

        assert_eq!(records[1].record.content.documents.len(), 1);
        assert_eq!(
            records[1].record.content.documents[0].source_kind,
            ContentSourceKind::Description
        );
        let parent_link = iter_foundry_links(&records[1].record.content.documents[0].document)
            .next()
            .expect("separate parent description reference");
        assert_eq!(
            parent_link
                .target
                .record_key()
                .map(ToString::to_string)
                .as_deref(),
            Some("spells-srd:rfZpqmj0AIIdkVIs")
        );
        let raw_parent: Value = serde_json::from_str(
            records[1]
                .record
                .provenance
                .raw_json
                .as_deref()
                .expect("exact parent source provenance"),
        )
        .expect("parent provenance JSON");
        assert_eq!(
            raw_parent.pointer("/system/uses/value"),
            Some(&Value::from(1))
        );
        assert!(
            child
                .definition
                .unsupported_notes
                .iter()
                .all(|note| { !note.source_path.starts_with("system.uses") })
        );
    }

    #[test]
    fn malformed_or_missing_consumable_locator_remains_local_and_unresolved() {
        let heal = normalize(
            "spells-srd",
            "packs/spells/1st-rank/heal.json",
            br#"{"_id":"rfZpqmj0AIIdkVIs","name":"Heal","type":"spell","system":{"level":{"value":1},"rules":[],"traits":{"rarity":"common","traditions":["divine"],"value":["healing"]}}}"#,
        );
        let malformed = normalize(
            "equipment-srd",
            "packs/equipment/local-spell.json",
            br#"{"_id":"local-parent","name":"Local Spell Item","type":"consumable","system":{"spell":{"_id":"local-child","flags":{"core":{"sourceId":"Heal"}},"name":"Local Spell","system":{"level":{"value":2},"location":{"heightenedLevel":2,"value":null},"rules":[],"traits":{"rarity":"common","traditions":[],"value":[]}},"type":"spell"}}}"#,
        );
        let mut records = vec![heal, malformed];
        let index = build_record_reference_index(&records);
        finalize_consumable_spell_children(&mut records, &index).expect("local child retained");
        let child = &records[1].facts.canonical_spell_children[0];
        assert!(matches!(
            child.standalone_target,
            FactValue::Value(SpellStandaloneTarget::Unresolved(_))
        ));
        assert_eq!(child.child_id.as_str(), "local-child");
    }

    #[test]
    fn area_details_and_unknown_member_evidence_survive_and_fail_closed() {
        let loaded = normalize(
            "spells-srd",
            "packs/spells/drift.json",
            br#"{"_id":"area-drift","name":"Area Drift","type":"spell","system":{"area":{"type":"burst","value":20,"details":"ground only","future":null},"counteraction":false,"duration":{"sustained":false,"value":""},"level":{"value":1},"rules":[],"traits":{"rarity":"common","traditions":["arcane"],"value":["concentrate"]}}}"#,
        );
        let spell = loaded
            .facts
            .canonical_body
            .as_ref()
            .and_then(RecordBody::as_spell)
            .expect("canonical spell");
        let targeting = spell
            .definition
            .targeting
            .as_value()
            .and_then(SpellSourceValue::as_known)
            .expect("targeting");
        let area = targeting
            .area
            .as_value()
            .and_then(SpellSourceValue::as_known)
            .expect("area");
        assert_eq!(
            area.details
                .as_value()
                .and_then(SpellSourceValue::as_known)
                .map(String::as_str),
            Some("ground only")
        );
        let unknown = area
            .unsupported_notes
            .iter()
            .find(|note| note.authored_key == "future")
            .expect("unknown area member evidence");
        assert_eq!(unknown.field, SpellUnsupportedSourceField::AreaMember);
        assert_eq!(unknown.source_path, "system.area.future");
        assert_eq!(unknown.authored_order, Some(3));
        assert_eq!(unknown.value.value, "null");
        let form = spell
            .resolve_form(
                SpellFormId::base(&spell.identity.record_key),
                SpellFormContext {
                    cast_rank: 1,
                    overlay_id: None,
                },
            )
            .expect("base form with localized unsupported field");
        assert!(matches!(
            form.targeting,
            SpellResolvedField::Unavailable(ref unavailable)
                if unavailable.field == SpellFormField::Targeting
        ));
        assert!(matches!(
            form.classification,
            SpellResolvedField::Available(_)
        ));
    }

    #[test]
    #[ignore = "requires an explicitly pinned PF2E_SOURCE_ROOT checkout"]
    fn pinned_selected_spell_and_ritual_files_cross_the_ordered_source_boundary() {
        let root = std::env::var_os("PF2E_SOURCE_ROOT")
            .expect("set PF2E_SOURCE_ROOT to the accepted pinned PF2E checkout");
        let selected = [
            (
                "packs/spells/1st-rank/heal.json",
                "rfZpqmj0AIIdkVIs",
                "Heal",
            ),
            (
                "packs/spells/2nd-rank/rime-slick.json",
                "Popa5umI3H33levx",
                "Rime Slick",
            ),
            (
                "packs/spells/7th-rank/deitys-strike.json",
                "x9RIFhquazom4p02",
                "Deity's Strike",
            ),
            (
                "packs/spells/ritual/planar-displacement.json",
                "HmKajQS0DP23bipp",
                "Planar Displacement",
            ),
            (
                "packs/spells/focus/qi-blast.json",
                "oo7YcRC2gcez81PV",
                "Qi Blast",
            ),
            (
                "packs/spells/cantrip/phase-bolt.json",
                "5gophZ4AOKW4VW27",
                "Phase Bolt",
            ),
        ];
        let mut records =
            selected
                .iter()
                .map(|(relative, id, name)| {
                    let source = std::fs::read(Path::new(&root).join(relative))
                        .expect("selected pinned spell source");
                    let loaded = normalize("spells-srd", relative, &source);
                    let spell = loaded
                        .facts
                        .canonical_body
                        .as_ref()
                        .and_then(RecordBody::as_spell)
                        .expect("canonical selected spell");
                    assert_eq!(spell.identity.source_id.as_str(), *id);
                    assert_eq!(spell.identity.name, *name);
                    assert!(
                        spell.definition.unsupported_notes.iter().all(|note| {
                            note.field != SpellUnsupportedSourceField::SystemMember
                        })
                    );
                    loaded
                })
                .collect::<Vec<_>>();
        let wand_path = "packs/equipment/arboreal-wand-rank-4.json";
        let wand_source =
            std::fs::read(Path::new(&root).join(wand_path)).expect("pinned Arboreal Wand source");
        records.push(normalize("equipment-srd", wand_path, &wand_source));
        let index = build_record_reference_index(&records);
        finalize_consumable_spell_children(&mut records, &index).expect("pinned child conversion");
        finalize_spell_owned_content(&mut records);
        resolve_content_references(&mut records, &index);

        let heal = records[0]
            .facts
            .canonical_body
            .as_ref()
            .and_then(RecordBody::as_spell)
            .expect("Heal");
        let overlays = heal
            .definition
            .ordered_overlays()
            .expect("Heal overlay order");
        assert_eq!(
            overlays
                .iter()
                .map(|overlay| overlay.overlay_id.as_str())
                .collect::<Vec<_>>(),
            vec![
                "7qdtetowq348s9oc",
                "37gy7l19tik74o4s",
                "lfxcoz2d3f8j2zq1",
                "7vbvdrv2cl87sqta",
            ]
        );
        for overlay in overlays {
            heal.resolve_form(
                SpellFormId::overlay(&heal.identity.record_key, &overlay.overlay_id),
                SpellFormContext {
                    cast_rank: 1,
                    overlay_id: Some(overlay.overlay_id.clone()),
                },
            )
            .expect("pinned Heal overlay resolves from authored map identity");
        }
        let child = &records.last().expect("wand").facts.canonical_spell_children[0];
        assert!(matches!(
            &child.standalone_target,
            FactValue::Value(SpellStandaloneTarget::Resolved(key))
                if key.to_string() == "spells-srd:rfZpqmj0AIIdkVIs"
        ));
    }

    fn normalize(pack: &str, source_path: &str, serialized: &[u8]) -> LoadedSourceRecord {
        normalize_record_from_source_bytes(
            &ManifestPack {
                name: pack.to_string(),
                label: pack.to_string(),
                document_type: "Item".to_string(),
                path: format!("packs/{pack}"),
            },
            &PackName::new(pack.to_string()).expect("pack"),
            Path::new(source_path),
            Path::new("."),
            serialized,
            None,
        )
        .expect("source normalizes")
    }
}
