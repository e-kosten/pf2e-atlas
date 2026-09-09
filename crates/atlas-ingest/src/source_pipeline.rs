use std::path::Path;

use atlas_domain::PackName;
use tracing::{debug, info};

use crate::diagnostics::{
    DERIVED_AFFLICTION_INSTANCES_PACK_LABEL, DERIVED_AFFLICTION_INSTANCES_PACK_NAME,
    DERIVED_AFFLICTIONS_PACK_LABEL, DERIVED_AFFLICTIONS_PACK_NAME,
};
use crate::embeddings;
use crate::error::IngestError;
use crate::generated::afflictions;
use crate::generated::afflictions::{GeneratedAfflictionRelationshipKind, GeneratedAfflictionRole};
use crate::records::references::{
    build_record_reference_index, resolve_content_references, resolve_reference_edges,
};
use crate::records::{aliases, taxonomy, variants};
use crate::source::loader::load_foundry_source_records;
use crate::source::npc_entities::finalize_npc_embedded_entities;
use crate::source::owned_content::{
    finalize_consumable_owned_content, finalize_hazard_owned_content, finalize_npc_owned_content,
    finalize_spell_owned_content,
};
use crate::source::spells::finalize_consumable_spell_children;
use crate::source::{LoadedPack, SourceLoad};

pub(crate) fn load_foundry_source(
    source_root: impl AsRef<Path>,
    manifest_path: Option<&Path>,
) -> Result<SourceLoad, IngestError> {
    let source_root = source_root.as_ref();
    info!(source = %source_root.display(), "loading Foundry source records");
    let mut source = load_foundry_source_records(source_root, manifest_path)?;
    info!(
        packs = source.packs.len(),
        source_records = source.records.len(),
        skipped_records = source.skipped_records.len(),
        "loaded Foundry source records"
    );

    source_progress("source_normalize", "Building source reference index");
    info!("building reference index");
    let reference_index = build_record_reference_index(&source.records);
    finalize_npc_embedded_entities(&mut source.records, &reference_index);
    finalize_consumable_spell_children(&mut source.records, &reference_index)?;
    crate::source::consumables::finalize_standalone_consumable_spell_bindings(&mut source.records)
        .map_err(|message| IngestError::RecordNormalizationFailed {
            path: "consumable spell binding".to_string(),
            message,
        })?;
    finalize_consumable_owned_content(&mut source.records);
    crate::source::consumables::finalize_consumable_occurrences(
        &mut source.records,
        &reference_index,
    )
    .map_err(|message| IngestError::RecordNormalizationFailed {
        path: "consumable occurrence conversion".to_string(),
        message,
    })?;
    finalize_npc_owned_content(&mut source.records);
    finalize_hazard_owned_content(&mut source.records);
    finalize_spell_owned_content(&mut source.records);
    source_progress("source_normalize", "Generating derived affliction records");
    info!("generating derived affliction records");
    let generated_afflictions =
        afflictions::build_generated_afflictions(&source.records, &reference_index)
            .map_err(|error| IngestError::GeneratedAfflictionFailed(error.to_string()))?;
    let generated_references = generated_afflictions.references.clone();
    if !generated_afflictions.records.is_empty() {
        let canonical_count = generated_afflictions
            .records
            .iter()
            .filter(|loaded| {
                loaded.facts.generated_affliction_role == Some(GeneratedAfflictionRole::Canonical)
            })
            .count();
        let instance_count = generated_afflictions
            .records
            .iter()
            .filter(|loaded| {
                loaded.facts.generated_affliction_role
                    == Some(GeneratedAfflictionRole::SourceInstance)
            })
            .count();
        debug_assert_eq!(
            generated_afflictions.records.len(),
            canonical_count + instance_count,
            "every generated affliction record has an explicit role"
        );
        debug_assert!(
            generated_afflictions
                .relationships
                .iter()
                .all(|relationship| {
                    relationship.from != relationship.to
                        && matches!(
                    relationship.kind,
                    GeneratedAfflictionRelationshipKind::HostHasSourceInstance
                        | GeneratedAfflictionRelationshipKind::SourceInstanceOfCanonical
                        | GeneratedAfflictionRelationshipKind::CanonicalDerivedFromHostOccurrence
                )
                })
        );
        debug_assert_eq!(
            generated_afflictions.relationships.len(),
            instance_count * 3,
            "each source instance has exactly three typed relationships"
        );
        for kind in [
            GeneratedAfflictionRelationshipKind::HostHasSourceInstance,
            GeneratedAfflictionRelationshipKind::SourceInstanceOfCanonical,
            GeneratedAfflictionRelationshipKind::CanonicalDerivedFromHostOccurrence,
        ] {
            debug_assert_eq!(
                generated_afflictions
                    .relationships
                    .iter()
                    .filter(|relationship| relationship.kind == kind)
                    .count(),
                instance_count,
                "each source instance has one relationship of every required kind"
            );
        }
        source.diagnostics.generated_affliction_canonical_records = canonical_count;
        source.diagnostics.generated_affliction_instance_records = instance_count;
        source.diagnostics.generated_affliction_reference_edges =
            generated_afflictions.relationships.len();
        source.packs.push(LoadedPack {
            name: PackName::new(DERIVED_AFFLICTIONS_PACK_NAME.to_string()).map_err(|error| {
                IngestError::ManifestParseFailed(format!(
                    "invalid derived affliction pack: {error}"
                ))
            })?,
            label: DERIVED_AFFLICTIONS_PACK_LABEL.to_string(),
            document_type: "Item".to_string(),
            declared_path: "derived://afflictions".to_string(),
            resolved_path: source_root.join("derived-afflictions"),
            record_count: canonical_count,
        });
        source.packs.push(LoadedPack {
            name: PackName::new(DERIVED_AFFLICTION_INSTANCES_PACK_NAME.to_string()).map_err(
                |error| {
                    IngestError::ManifestParseFailed(format!(
                        "invalid derived affliction instance pack: {error}"
                    ))
                },
            )?,
            label: DERIVED_AFFLICTION_INSTANCES_PACK_LABEL.to_string(),
            document_type: "Item".to_string(),
            declared_path: "derived://affliction-instances".to_string(),
            resolved_path: source_root.join("derived-affliction-instances"),
            record_count: instance_count,
        });
        source.records.extend(generated_afflictions.records);
        info!(
            canonical = canonical_count,
            instances = instance_count,
            reference_edges = generated_afflictions.references.len(),
            "generated derived affliction records"
        );
    }

    source_progress("source_normalize", "Assigning taxonomy families");
    info!(
        records = source.records.len(),
        "assigning taxonomy families"
    );
    let reference_index = build_record_reference_index(&source.records);
    resolve_content_references(&mut source.records, &reference_index);
    taxonomy::assign_inferred_taxonomy_groups(
        &mut source.records,
        &source.packs,
        &reference_index,
        &mut source.diagnostics,
    );
    source_progress("source_normalize", "Assigning variant groups");
    info!(records = source.records.len(), "assigning variant groups");
    variants::assign_variant_groups(
        &mut source.records,
        &reference_index,
        &mut source.diagnostics,
    );
    source_progress("source_normalize", "Resolving reference edges");
    info!("resolving reference edges");
    source.references = resolve_reference_edges(&source.records);
    source.references.extend(generated_references);
    source.references.sort_by(|left, right| {
        (
            left.from_record_key.to_string(),
            left.to_record_key.to_string(),
            left.reference_text.as_str(),
            left.source_kind.as_str(),
        )
            .cmp(&(
                right.from_record_key.to_string(),
                right.to_record_key.to_string(),
                right.reference_text.as_str(),
                right.source_kind.as_str(),
            ))
    });
    source.references.dedup_by(|left, right| {
        left.from_record_key == right.from_record_key
            && left.to_record_key == right.to_record_key
            && left.reference_text == right.reference_text
            && left.source_kind == right.source_kind
    });
    info!(
        reference_edges = source.references.len(),
        "resolved reference edges"
    );
    source_progress("source_normalize", "Resolving aliases and remaster links");
    info!("resolving aliases and remaster links");
    source.aliases =
        aliases::resolve_record_aliases(&source.records, &reference_index, source_root);
    source.remaster_links =
        aliases::resolve_remaster_links(&source.records, &reference_index, source_root);
    info!(
        aliases = source.aliases.len(),
        remaster_links = source.remaster_links.len(),
        "resolved aliases and remaster links"
    );
    source_progress("source_normalize", "Preparing document embedding inputs");
    info!("preparing document embedding inputs");
    source.pending_document_embeddings = embeddings::build_pending_document_embeddings(
        &source.records,
        &source.aliases,
        &source.remaster_links,
    );
    let embedding_unit_summary =
        embeddings::summarize_pending_document_embeddings(&source.pending_document_embeddings);
    debug!(
        total_units = embedding_unit_summary.total_units,
        parent_units = embedding_unit_summary.parent_units,
        child_units = embedding_unit_summary.child_units,
        records_with_child_units = embedding_unit_summary.records_with_child_units,
        records_over_20_child_units = embedding_unit_summary.records_over_20_child_units,
        records_over_50_child_units = embedding_unit_summary.records_over_50_child_units,
        records_over_100_child_units = embedding_unit_summary.records_over_100_child_units,
        max_child_units_per_record = embedding_unit_summary.max_child_units_per_record,
        "document embedding unit fanout diagnostics"
    );
    info!(
        pending_document_embeddings = source.pending_document_embeddings.len(),
        "prepared document embedding inputs"
    );

    Ok(source)
}

fn source_progress(phase: &'static str, message: &'static str) {
    info!(target: "atlas_progress", phase, "{message}");
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;
    use std::path::Path;

    use atlas_record::{
        ConsumableEntityTarget, ConsumableSourceValue, ConsumableSpellReuse, FactValue,
        FoundryRecordType, HazardCapability, HazardUnsupportedField, RecordBody, SpellHeightening,
        SpellRule, SpellSourceValue, SpellStandaloneTarget, UnsupportedSourceShape,
    };
    use serde_json::Value;
    use sha2::{Digest, Sha256};

    use super::load_foundry_source;
    use crate::source::dto::{SourcePresence, SpellDocumentSource};

    const FIXTURE_ROOT: &str = "tests/fixtures/foundry-source/spell-source-contract";
    const LOSSLESS_FIXTURE_ROOT: &str = "tests/fixtures/foundry-source/lossless-source-contract";

    #[test]
    fn pinned_consumable_family_census_and_spell_reuse_are_complete() {
        let Some(root) = std::env::var_os("PF2E_SOURCE_ROOT") else {
            return;
        };
        let source = load_foundry_source(Path::new(&root), None)
            .expect("accepted pinned PF2E source should load");

        let standalone = source
            .records
            .iter()
            .filter(|loaded| matches!(loaded.facts.canonical_body, Some(RecordBody::Consumable(_))))
            .count();
        let standalone_spell_children = source
            .records
            .iter()
            .filter(|loaded| matches!(loaded.facts.canonical_body, Some(RecordBody::Consumable(_))))
            .map(|loaded| loaded.facts.canonical_spell_children.len())
            .sum::<usize>();
        assert_eq!(standalone, 1_670);
        assert_eq!(standalone_spell_children, 10);

        let dragon_breath = source
            .records
            .iter()
            .find(|loaded| loaded.record.identity.key.id().as_str() == "LKwI47eeJC0Y4hGu")
            .and_then(|loaded| loaded.facts.canonical_body.as_ref())
            .and_then(RecordBody::as_consumable)
            .expect("Silver Dragon's Breath Potion has a canonical consumable body");
        let damage = dragon_breath
            .definition
            .damage
            .as_value()
            .and_then(ConsumableSourceValue::known)
            .expect("structured consumable damage");
        assert_eq!(
            damage
                .formula
                .as_value()
                .and_then(ConsumableSourceValue::known)
                .map(String::as_str),
            Some("4d6")
        );
        assert_eq!(
            damage
                .category
                .as_value()
                .and_then(ConsumableSourceValue::known)
                .map(String::as_str),
            Some("damage")
        );
        assert_eq!(
            damage
                .damage_type
                .as_value()
                .and_then(ConsumableSourceValue::known)
                .map(String::as_str),
            Some("cold")
        );

        let consumable = |id: &str| {
            source
                .records
                .iter()
                .find(|loaded| loaded.record.identity.key.id().as_str() == id)
                .and_then(|loaded| loaded.facts.canonical_body.as_ref())
                .and_then(RecordBody::as_consumable)
                .unwrap_or_else(|| panic!("canonical consumable {id}"))
        };
        assert_eq!(
            consumable("E7BcwZy8nTpTLYf1")
                .definition
                .base_item
                .as_value()
                .and_then(ConsumableSourceValue::known)
                .map(String::as_str),
            Some("alchemical-bomb")
        );
        assert_eq!(
            consumable("oQr0x30JoLrWxQdA")
                .definition
                .price
                .as_value()
                .and_then(ConsumableSourceValue::known)
                .and_then(|price| price.per.as_value())
                .and_then(ConsumableSourceValue::known),
            Some(&10)
        );
        assert_eq!(
            consumable("XWO5pMQcYNybqWzf")
                .definition
                .material
                .as_value()
                .and_then(ConsumableSourceValue::known)
                .and_then(|material| material.effects.as_value())
                .and_then(ConsumableSourceValue::known)
                .map(Vec::as_slice),
            Some(&["silver".to_string()][..])
        );
        assert_eq!(
            consumable("UHrLqWCnFEUspSQj")
                .definition
                .price
                .as_value()
                .and_then(ConsumableSourceValue::known)
                .and_then(|price| price.denominations.as_value())
                .and_then(ConsumableSourceValue::known)
                .map(Vec::as_slice),
            Some(&[][..])
        );
        assert_eq!(
            consumable("rKknk8odXDBpON5l")
                .definition
                .rules
                .as_value()
                .and_then(ConsumableSourceValue::known)
                .map(Vec::len),
            Some(1)
        );

        let mut npc = (0_usize, 0_usize, 0_usize);
        let mut character = (0_usize, 0_usize, 0_usize);
        let mut hazard = (0_usize, 0_usize, 0_usize);
        let mut spell_reused = 0_usize;
        let mut spell_mismatched = 0_usize;
        let mut spell_mismatch_reasons = BTreeMap::<&'static str, usize>::new();
        let mut parent_owned = (0_usize, 0_usize, 0_usize);
        let mut copied_content = 0_usize;
        let mut local_content = 0_usize;
        let mut gm_content = 0_usize;
        let consumable_targets = source
            .records
            .iter()
            .filter_map(|loaded| {
                let Some(RecordBody::Consumable(consumable)) = &loaded.facts.canonical_body else {
                    return None;
                };
                Some((consumable.identity.record_key.clone(), consumable))
            })
            .collect::<std::collections::BTreeMap<_, _>>();
        for loaded in &source.records {
            let occurrences = &loaded.facts.consumable_occurrences;
            if occurrences.occurrences.is_empty() {
                continue;
            }
            let resolved = occurrences
                .entities
                .iter()
                .filter(|entity| matches!(entity.target, ConsumableEntityTarget::Resolved { .. }))
                .count();
            let spell_bearing = occurrences
                .occurrences
                .iter()
                .filter(|occurrence| {
                    !matches!(occurrence.spell_reuse, ConsumableSpellReuse::NotPresent)
                })
                .count();
            for occurrence in &occurrences.occurrences {
                for document in &occurrence.authored_content.documents {
                    assert!(matches!(
                        document.owner,
                        atlas_record::ContentOwner::ConsumableOccurrence(ref id)
                            if id == &occurrence.id
                    ));
                    if document.source_kind
                        == atlas_record::ContentSourceKind::EmbeddedGmDescription
                    {
                        gm_content += 1;
                    }
                    match &document.duplicate_status {
                        atlas_record::DuplicateContentStatus::CopiedFromConsumableTarget {
                            target_record_key,
                            target_content_key,
                            target_content_hash,
                        } => {
                            copied_content += 1;
                            let target = consumable_targets
                                .get(target_record_key)
                                .expect("copied content target must be canonical");
                            let target_document = target
                                .content
                                .documents
                                .iter()
                                .find(|candidate| &candidate.id.content_key == target_content_key)
                                .expect("copied target content key must exist");
                            assert_eq!(target_document.content_hash.as_str(), target_content_hash);
                            assert_eq!(target_document.document, document.document);
                        }
                        atlas_record::DuplicateContentStatus::Unique => local_content += 1,
                        atlas_record::DuplicateContentStatus::CopiedFromCanonicalTarget {
                            ..
                        } => {
                            panic!("H5 content must use its typed target-copy identity")
                        }
                    }
                }
                let entity = occurrences
                    .entities
                    .iter()
                    .find(|entity| entity.id == occurrence.entity_id)
                    .expect("every occurrence has one entity");
                assert_eq!(occurrence.owner_record_key, loaded.record.identity.key);
                assert!(occurrence.source_path.contains("#items["));
                if matches!(entity.target, ConsumableEntityTarget::ParentOwned { .. }) {
                    match &loaded.record.foundry.record_type {
                        FoundryRecordType::Npc => parent_owned.0 += 1,
                        FoundryRecordType::Character => parent_owned.1 += 1,
                        FoundryRecordType::Hazard => parent_owned.2 += 1,
                        other => panic!("unexpected parent-owned consumable owner {other:?}"),
                    }
                }
                match &occurrence.spell_reuse {
                    ConsumableSpellReuse::NotPresent => {}
                    ConsumableSpellReuse::Reused { .. } => spell_reused += 1,
                    ConsumableSpellReuse::Mismatch {
                        reason,
                        local_evidence,
                    } => {
                        if let atlas_record::ConsumableLocalSpellEvidence::Child(child) =
                            local_evidence
                        {
                            assert_eq!(child.parent_record_key, loaded.record.identity.key);
                        }
                        spell_mismatched += 1;
                        *spell_mismatch_reasons
                            .entry(consumable_spell_mismatch_reason(*reason))
                            .or_default() += 1;
                    }
                }
            }
            let entry = match &loaded.record.foundry.record_type {
                FoundryRecordType::Npc => &mut npc,
                FoundryRecordType::Character => &mut character,
                FoundryRecordType::Hazard => &mut hazard,
                ref other => panic!("unexpected consumable occurrence owner {other:?}"),
            };
            entry.0 += occurrences.occurrences.len();
            entry.1 += resolved;
            entry.2 += spell_bearing;
        }

        assert_eq!(npc, (1_328, 1_259, 135));
        assert_eq!(character, (496, 489, 45));
        assert_eq!(hazard, (1, 0, 0));
        assert_eq!(parent_owned, (69, 7, 1));
        assert_eq!(npc.0 + character.0 + hazard.0, 1_825);
        assert_eq!(npc.2 + character.2 + hazard.2, 180);
        assert_eq!(spell_reused + spell_mismatched, 180);
        assert_eq!(spell_reused, 0);
        assert_eq!(
            spell_mismatch_reasons,
            BTreeMap::from([("target_without_child", 180)]),
            "each pinned embedded Spell snapshot must retain its exact H2 comparison disposition"
        );
        assert!(copied_content > 0);
        assert!(local_content > 0);
        assert_eq!(gm_content, 2);
        assert_false_door_consumable(&source);
        assert_consumable_observed_leaf_inventory(&source);
    }

    fn assert_false_door_consumable(source: &crate::source::SourceLoad) {
        let loaded = source
            .records
            .iter()
            .find(|loaded| {
                loaded.record.identity.key.to_string()
                    == "agents-of-edgewatch-bestiary:u1cuwAE3xzhYW4Mi"
            })
            .expect("pinned False Door Trap");
        let [occurrence] = loaded.facts.consumable_occurrences.occurrences.as_slice() else {
            panic!("False Door must retain exactly one consumable occurrence")
        };
        let entity = loaded
            .facts
            .consumable_occurrences
            .entities
            .iter()
            .find(|entity| entity.id == occurrence.entity_id)
            .expect("False Door consumable entity");
        let atlas_record::ConsumableEntityTarget::ParentOwned {
            definition,
            resolution,
            content_identity,
        } = &entity.target
        else {
            panic!("False Door consumable remains parent-owned")
        };

        assert!(matches!(
            resolution,
            atlas_record::ConsumableTargetResolution::NoLocator
        ));
        assert!(matches!(content_identity, FactValue::Value(_)));
        assert_eq!(
            occurrence.id.as_str(),
            "consumable-3458624b33476e66305231676b59504a"
        );
        assert_eq!(
            occurrence
                .source_id
                .as_value()
                .and_then(ConsumableSourceValue::known)
                .map(|value| value.as_str()),
            Some("4XbK3Gnf0R1gkYPJ")
        );
        assert_eq!(occurrence.authored_order, 1);
        assert_eq!(occurrence.contextual_name, "Purple Worm Venom");
        assert!(matches!(
            occurrence.locator,
            atlas_record::ConsumableLocatorState::Missing
        ));
        assert_eq!(
            occurrence
                .state
                .quantity
                .as_value()
                .and_then(ConsumableSourceValue::known),
            Some(&1)
        );
        assert_eq!(
            occurrence
                .state
                .current_uses
                .as_value()
                .and_then(ConsumableSourceValue::known),
            Some(&1)
        );
        assert_eq!(
            occurrence
                .state
                .current_hp
                .as_value()
                .and_then(ConsumableSourceValue::known),
            Some(&0)
        );
        assert!(matches!(occurrence.state.container_id, FactValue::Null));
        let equipped = occurrence
            .state
            .equipped
            .as_value()
            .and_then(ConsumableSourceValue::known)
            .expect("False Door equipped state");
        assert_eq!(
            equipped
                .carry_type
                .as_value()
                .and_then(ConsumableSourceValue::known)
                .map(String::as_str),
            Some("worn")
        );
        assert!(matches!(equipped.hands_held, FactValue::Missing));
        assert!(matches!(equipped.in_slot, FactValue::Missing));

        assert_eq!(
            definition
                .level
                .as_value()
                .and_then(ConsumableSourceValue::known),
            Some(&13)
        );
        assert_eq!(
            definition
                .category
                .as_value()
                .and_then(ConsumableSourceValue::known)
                .map(String::as_str),
            Some("potion")
        );
        assert_eq!(
            definition
                .bulk
                .as_value()
                .and_then(ConsumableSourceValue::known)
                .map(|value| value.as_str()),
            Some("0.1")
        );
        assert_eq!(
            definition
                .usage
                .as_value()
                .and_then(ConsumableSourceValue::known)
                .map(String::as_str),
            Some("held-in-one-hand")
        );
        assert_eq!(
            definition
                .maximum_uses
                .as_value()
                .and_then(ConsumableSourceValue::known),
            Some(&1)
        );
        assert_eq!(
            definition
                .maximum_hp
                .as_value()
                .and_then(ConsumableSourceValue::known),
            Some(&0)
        );
        assert_eq!(
            definition
                .hardness
                .as_value()
                .and_then(ConsumableSourceValue::known),
            Some(&0)
        );
        assert!(matches!(definition.damage, FactValue::Null));
        assert!(matches!(definition.base_item, FactValue::Null));
        assert_eq!(
            definition
                .traits
                .as_value()
                .and_then(ConsumableSourceValue::known)
                .map(Vec::as_slice),
            Some(&["alchemical", "consumable", "injury", "poison"].map(str::to_string)[..])
        );
        let publication = definition
            .publication
            .as_value()
            .and_then(ConsumableSourceValue::known)
            .expect("False Door publication");
        assert_eq!(
            publication
                .title
                .as_value()
                .and_then(ConsumableSourceValue::known)
                .map(String::as_str),
            Some("")
        );
        assert_eq!(
            publication
                .remaster
                .as_value()
                .and_then(ConsumableSourceValue::known),
            Some(&false)
        );
        let [document] = occurrence.authored_content.documents.as_slice() else {
            panic!("False Door consumable description")
        };
        assert!(matches!(
            document.owner,
            atlas_record::ContentOwner::ConsumableOccurrence(ref id) if id == &occurrence.id
        ));
        assert_eq!(
            document.provenance.nested_source_id.as_deref(),
            Some("4XbK3Gnf0R1gkYPJ")
        );
        let rendered = atlas_record::render_plain_text(&document.document);
        assert!(rendered.contains("DC 32 Fortitude"));
        assert!(rendered.contains("Maximum Duration"));
        assert!(rendered.contains("Enfeebled 2"));
        assert!(
            atlas_record::iter_foundry_links(&document.document)
                .any(|link| link.source.authored_target.ends_with(".Item.Enfeebled"))
        );
    }

    fn consumable_spell_mismatch_reason(
        reason: atlas_record::ConsumableSpellMismatchReason,
    ) -> &'static str {
        match reason {
            atlas_record::ConsumableSpellMismatchReason::UnresolvedParent => "unresolved_parent",
            atlas_record::ConsumableSpellMismatchReason::TargetWithoutChild => {
                "target_without_child"
            }
            atlas_record::ConsumableSpellMismatchReason::LocalChildMissing => "local_child_missing",
            atlas_record::ConsumableSpellMismatchReason::LocalChildMalformed => {
                "local_child_malformed"
            }
            atlas_record::ConsumableSpellMismatchReason::ChildIdentity => "child_identity",
            atlas_record::ConsumableSpellMismatchReason::SourceContext => "source_context",
            atlas_record::ConsumableSpellMismatchReason::Definition => "definition",
            atlas_record::ConsumableSpellMismatchReason::ContentOrReferences => {
                "content_or_references"
            }
            atlas_record::ConsumableSpellMismatchReason::OverlayOrFormOrder => {
                "overlay_or_form_order"
            }
        }
    }

    #[derive(Default)]
    struct LeafInventoryEntry {
        occurrences: usize,
        shapes: BTreeMap<&'static str, usize>,
    }

    fn assert_consumable_observed_leaf_inventory(source: &crate::source::SourceLoad) {
        let mut inventories = BTreeMap::<&'static str, BTreeMap<String, LeafInventoryEntry>>::new();
        for loaded in &source.records {
            let Some(raw) = loaded.record.provenance.raw_json.as_deref() else {
                continue;
            };
            let root: Value = serde_json::from_str(raw).expect("pinned raw source is JSON");
            if matches!(loaded.facts.canonical_body, Some(RecordBody::Consumable(_))) {
                collect_consumable_leaves(&root, "$", inventories.entry("standalone").or_default());
                continue;
            }
            let context = match &loaded.record.foundry.record_type {
                FoundryRecordType::Npc => "npc",
                FoundryRecordType::Character => "character",
                FoundryRecordType::Hazard => "hazard",
                _ => continue,
            };
            let Some(items) = root.get("items").and_then(Value::as_array) else {
                continue;
            };
            for item in items
                .iter()
                .filter(|item| item.get("type").and_then(Value::as_str) == Some("consumable"))
            {
                collect_consumable_leaves(item, "$", inventories.entry(context).or_default());
            }
        }

        for (context, expected_paths, expected_occurrences, expected_digest) in [
            (
                "standalone",
                70,
                51_307,
                "8f82f0ea0586dcf290e89eb3229b251bf35e24c3c196b03b5f9f563ea62777d5",
            ),
            (
                "npc",
                67,
                46_381,
                "a0cba2d6467de8ab69522fabc6d87fcf5a1ce06c4fddc305686c398415be17b1",
            ),
            (
                "character",
                52,
                17_035,
                "2713bec6771bc125319db021b8ffe3d8b756e0bf53d428c4229e0ddf3d3b546c",
            ),
            (
                "hazard",
                31,
                34,
                "e2b910a99e27c08f94e2a6c12baf3093f941fa28db146a78c2af47a3346125d9",
            ),
        ] {
            let inventory = inventories
                .get(context)
                .unwrap_or_else(|| panic!("missing {context} consumable inventory"));
            assert_eq!(inventory.len(), expected_paths, "{context} leaf paths");
            assert_eq!(
                inventory
                    .values()
                    .map(|entry| entry.occurrences)
                    .sum::<usize>(),
                expected_occurrences,
                "{context} leaf occurrences"
            );
            let mut canonical = String::new();
            for (path, entry) in inventory {
                canonical.push_str(path);
                canonical.push('\t');
                canonical.push_str(&entry.occurrences.to_string());
                canonical.push('\t');
                canonical.push_str(
                    &serde_json::to_string(&entry.shapes).expect("leaf shapes serialize"),
                );
                canonical.push('\n');
            }
            assert_eq!(
                format!("{:x}", Sha256::digest(canonical.as_bytes())),
                expected_digest,
                "{context} reverse observed-leaf inventory"
            );
        }
    }

    fn collect_consumable_leaves(
        value: &Value,
        path: &str,
        inventory: &mut BTreeMap<String, LeafInventoryEntry>,
    ) {
        if path == "$.system.spell" {
            return;
        }
        match value {
            Value::Object(values) => {
                for (key, value) in values {
                    let segment = if path == "$.system.price.value" {
                        "*"
                    } else {
                        key
                    };
                    collect_consumable_leaves(value, &format!("{path}.{segment}"), inventory);
                }
            }
            Value::Array(values) => {
                for value in values {
                    collect_consumable_leaves(value, &format!("{path}[]"), inventory);
                }
            }
            Value::Null | Value::Bool(_) | Value::Number(_) | Value::String(_) => {
                let shape = match value {
                    Value::Null => "null",
                    Value::Bool(_) => "boolean",
                    Value::Number(_) => "number",
                    Value::String(_) => "string",
                    Value::Array(_) | Value::Object(_) => return,
                };
                let entry = inventory.entry(path.to_string()).or_default();
                entry.occurrences += 1;
                *entry.shapes.entry(shape).or_default() += 1;
            }
        }
    }

    #[test]
    fn production_loader_preserves_order_duplicates_and_selected_spell_shapes() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR")).join(FIXTURE_ROOT);
        let source = load_foundry_source(&root, None).expect("portable spell fixture source");
        let mut record_ids = source
            .records
            .iter()
            .map(|loaded| loaded.record.identity.key.id().as_str())
            .collect::<Vec<_>>();
        record_ids.sort_unstable();
        assert_eq!(
            record_ids,
            [
                "5gophZ4AOKW4VW27",
                "HmKajQS0DP23bipp",
                "NacrNSvfODxpZena",
                "Popa5umI3H33levx",
                "eOtQtVRLeGH39dNx",
                "nonlexicalSpellMaps",
                "oo7YcRC2gcez81PV",
                "rfZpqmj0AIIdkVIs",
                "x9RIFhquazom4p02",
            ]
        );

        let spell = |id: &str| {
            source
                .records
                .iter()
                .find(|loaded| loaded.record.identity.key.id().as_str() == id)
                .and_then(|loaded| loaded.facts.canonical_body.as_ref())
                .and_then(RecordBody::as_spell)
                .unwrap_or_else(|| panic!("canonical spell {id}"))
        };

        let ordered = spell("nonlexicalSpellMaps");
        let damage = ordered
            .definition
            .damage
            .as_value()
            .and_then(SpellSourceValue::as_known)
            .expect("ordered damage");
        assert_eq!(
            damage
                .iter()
                .map(|member| (member.key.as_str(), member.authored_order))
                .collect::<Vec<_>>(),
            [("zeta-damage", 0), ("alpha-damage", 1), ("zeta-damage", 2)]
        );
        let heightening = ordered
            .definition
            .heightening
            .as_value()
            .and_then(SpellSourceValue::as_known)
            .expect("fixed heightening");
        let SpellHeightening::Fixed(layers) = heightening else {
            panic!("fixed heightening");
        };
        assert_eq!(
            layers
                .iter()
                .map(|layer| (layer.key.as_str(), layer.authored_order))
                .collect::<Vec<_>>(),
            [("8", 0), ("5", 1)]
        );
        let overlays = ordered
            .definition
            .overlays
            .as_value()
            .and_then(SpellSourceValue::as_known)
            .expect("ordered overlays");
        assert_eq!(
            overlays
                .iter()
                .map(|overlay| (overlay.key.as_str(), overlay.authored_order))
                .collect::<Vec<_>>(),
            [("zeta-overlay", 0), ("alpha-overlay", 1)]
        );

        let heal = spell("rfZpqmj0AIIdkVIs");
        assert_eq!(
            heal.definition
                .source_context
                .image
                .as_value()
                .and_then(SpellSourceValue::as_known)
                .map(String::as_str),
            Some("icons/magic/life/cross-worn-green.webp")
        );
        assert_eq!(
            heal.definition
                .source_context
                .publication_license
                .as_value()
                .and_then(SpellSourceValue::as_known)
                .map(|value| value.as_str()),
            Some("ORC")
        );

        let rime = spell("Popa5umI3H33levx");
        let fixed = rime
            .definition
            .heightening
            .as_value()
            .and_then(SpellSourceValue::as_known)
            .expect("Rime fixed heightening");
        let SpellHeightening::Fixed(layers) = fixed else {
            panic!("Rime fixed heightening");
        };
        assert_eq!(
            layers
                .iter()
                .map(|layer| layer.key.as_str())
                .collect::<Vec<_>>(),
            ["5", "8"]
        );
        let rank_five_area = layers[0]
            .patch
            .targeting
            .as_value()
            .and_then(SpellSourceValue::as_known)
            .and_then(|targeting| targeting.area.as_value())
            .and_then(SpellSourceValue::as_known)
            .expect("Rime rank-five area");
        assert_eq!(
            rank_five_area
                .legacy_area_type
                .as_value()
                .and_then(SpellSourceValue::as_known)
                .map(|legacy| legacy.value.as_str()),
            Some("burst")
        );

        let deity = spell("x9RIFhquazom4p02");
        let defense = deity
            .definition
            .defense
            .as_value()
            .and_then(SpellSourceValue::as_known)
            .expect("Deity's Strike dual defense");
        assert!(matches!(defense.passive, FactValue::Value(_)));
        assert!(matches!(defense.save, FactValue::Value(_)));

        let planar = spell("HmKajQS0DP23bipp");
        let ritual = planar
            .definition
            .ritual
            .as_value()
            .and_then(SpellSourceValue::as_known)
            .expect("explicit ritual root");
        assert!(matches!(ritual.primary_check, FactValue::Value(_)));
        assert!(matches!(ritual.secondary_casters, FactValue::Value(_)));
        assert!(matches!(ritual.secondary_checks, FactValue::Value(_)));

        let qi_rules = spell("oo7YcRC2gcez81PV")
            .definition
            .rules
            .as_value()
            .and_then(SpellSourceValue::as_known)
            .expect("Qi Blast rules");
        assert!(matches!(qi_rules[0].rule, SpellRule::RollOption(_)));
        assert!(matches!(qi_rules[1].rule, SpellRule::DamageAlteration(_)));
        assert!(matches!(qi_rules[2].rule, SpellRule::ItemAlteration(_)));
        assert!(matches!(qi_rules[3].rule, SpellRule::ItemAlteration(_)));
        assert_eq!(
            qi_rules
                .iter()
                .map(|rule| rule.authored_order)
                .collect::<Vec<_>>(),
            [0, 1, 2, 3]
        );
        let phase_rules = spell("5gophZ4AOKW4VW27")
            .definition
            .rules
            .as_value()
            .and_then(SpellSourceValue::as_known)
            .expect("Phase Bolt rules");
        assert!(matches!(phase_rules[0].rule, SpellRule::EphemeralEffect(_)));

        let wand = source
            .records
            .iter()
            .find(|loaded| loaded.record.identity.key.id().as_str() == "eOtQtVRLeGH39dNx")
            .expect("Arboreal Wand");
        let child = wand
            .facts
            .canonical_spell_children
            .first()
            .expect("canonical consumable spell child");
        let Some(RecordBody::Consumable(consumable)) = wand.facts.canonical_body.as_ref() else {
            panic!("Arboreal Wand must have a canonical consumable body");
        };
        assert_eq!(consumable.identity.source_id.as_str(), "eOtQtVRLeGH39dNx");
        assert_eq!(
            consumable
                .definition
                .spell_child_id
                .as_value()
                .map(|value| value.as_str()),
            Some("7w37duycMs4YOBeu")
        );
        assert_eq!(
            consumable
                .definition
                .maximum_uses
                .as_value()
                .and_then(ConsumableSourceValue::known),
            Some(&1)
        );
        assert_eq!(
            consumable
                .source_state
                .current_uses
                .as_value()
                .and_then(ConsumableSourceValue::known),
            Some(&1)
        );
        assert!(matches!(
            consumable.source_state.equipped,
            FactValue::Missing
        ));
        assert_eq!(consumable.content.documents.len(), 1);
        assert_eq!(child.child_id.as_str(), "7w37duycMs4YOBeu");
        assert!(matches!(
            &child.standalone_target,
            FactValue::Value(SpellStandaloneTarget::Resolved(key))
                if key.to_string() == "spells-srd:rfZpqmj0AIIdkVIs"
        ));
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
        assert_eq!(
            child
                .definition
                .source_context
                .image
                .as_value()
                .and_then(SpellSourceValue::as_known)
                .map(String::as_str),
            Some("icons/magic/life/cross-worn-green.webp")
        );
        assert_eq!(
            child
                .definition
                .source_context
                .publication_license
                .as_value()
                .and_then(SpellSourceValue::as_known)
                .map(|value| value.as_str()),
            Some("ORC")
        );
        let child_source = child
            .definition
            .source_context
            .consumable_child
            .as_value()
            .expect("local child source context");
        assert_eq!(
            child_source
                .slug
                .as_value()
                .and_then(SpellSourceValue::as_known)
                .map(String::as_str),
            Some("heal")
        );
        assert_eq!(
            child_source
                .publication_title
                .as_value()
                .and_then(SpellSourceValue::as_known)
                .map(String::as_str),
            Some("Pathfinder Player Core")
        );
        assert_eq!(
            child_source
                .publication_remaster
                .as_value()
                .and_then(SpellSourceValue::as_known),
            Some(&true)
        );
        assert_eq!(
            child_source
                .rarity
                .as_value()
                .and_then(SpellSourceValue::as_known)
                .map(|rarity| rarity.as_str()),
            Some("common")
        );
        let raw: serde_json::Value = serde_json::from_str(
            wand.record
                .provenance
                .raw_json
                .as_deref()
                .expect("exact parent raw source"),
        )
        .expect("parent raw source is JSON");
        assert_eq!(
            raw.pointer("/system/spell/img")
                .and_then(serde_json::Value::as_str),
            Some("icons/magic/life/cross-worn-green.webp")
        );
        assert_eq!(
            raw.pointer("/system/spell/sort")
                .and_then(serde_json::Value::as_i64),
            Some(133_500_000)
        );
        assert_eq!(
            raw.pointer("/system/spell/system/slug")
                .and_then(serde_json::Value::as_str),
            Some("heal")
        );
        assert_eq!(
            raw.pointer("/system/spell/system/publication/title")
                .and_then(serde_json::Value::as_str),
            Some("Pathfinder Player Core")
        );
        assert_eq!(
            raw.pointer("/system/spell/system/publication/license")
                .and_then(serde_json::Value::as_str),
            Some("ORC")
        );
        assert_eq!(
            raw.pointer("/system/spell/system/publication/remaster")
                .and_then(serde_json::Value::as_bool),
            Some(true)
        );
        assert_eq!(
            raw.pointer("/system/spell/system/traits/rarity")
                .and_then(serde_json::Value::as_str),
            Some("common")
        );
        assert_eq!(
            raw.pointer("/system/uses/max")
                .and_then(serde_json::Value::as_i64),
            Some(1)
        );
    }

    #[test]
    fn production_loader_delivers_lossless_duplicate_values_to_family_owners() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR")).join(LOSSLESS_FIXTURE_ROOT);
        let source = load_foundry_source(&root, None).expect("lossless fixture source");
        assert_eq!(source.records.len(), 2);

        let spell_loaded = source
            .records
            .iter()
            .find(|loaded| loaded.record.identity.key.id().as_str() == "losslessSpell")
            .expect("lossless spell source record");
        let SpellDocumentSource::Standalone(spell_dto) = spell_loaded
            .facts
            .spell_source
            .as_ref()
            .expect("lossless spell DTO")
        else {
            panic!("standalone spell DTO")
        };
        let spell_dto_damage = spell_dto
            .damage
            .as_value()
            .and_then(SpellSourceValue::as_known)
            .expect("spell DTO damage entries");
        assert_eq!(
            spell_dto_damage
                .iter()
                .map(|entry| {
                    (
                        entry.key.as_str(),
                        entry.authored_order,
                        entry
                            .value
                            .formula
                            .as_value()
                            .and_then(SpellSourceValue::as_known)
                            .map(String::as_str),
                    )
                })
                .collect::<Vec<_>>(),
            [
                ("zeta-damage", 0, Some("1d8")),
                ("alpha-damage", 1, Some("1d4")),
                ("zeta-damage", 2, Some("2d8")),
            ]
        );
        let spell = spell_loaded
            .facts
            .canonical_body
            .as_ref()
            .and_then(RecordBody::as_spell)
            .expect("canonical lossless spell");
        let spell_damage = spell
            .definition
            .damage
            .as_value()
            .and_then(SpellSourceValue::as_known)
            .expect("spell damage entries");
        assert_eq!(
            spell_damage
                .iter()
                .map(|entry| {
                    (
                        entry.key.as_str(),
                        entry.authored_order,
                        entry
                            .value
                            .formula
                            .as_value()
                            .and_then(SpellSourceValue::as_known)
                            .map(String::as_str),
                    )
                })
                .collect::<Vec<_>>(),
            [
                ("zeta-damage", 0, Some("1d8")),
                ("alpha-damage", 1, Some("1d4")),
                ("zeta-damage", 2, Some("2d8")),
            ]
        );
        assert_eq!(
            spell_damage[0]
                .value
                .kinds
                .as_value()
                .and_then(SpellSourceValue::as_known)
                .map(Vec::as_slice),
            Some(&["damage".to_string(), "damage".to_string()][..])
        );
        let spell_dto_duration = spell_dto
            .duration
            .as_value()
            .and_then(SpellSourceValue::as_known)
            .expect("spell DTO duration");
        let FactValue::Value(SpellSourceValue::Unsupported(spell_dto_duration_value)) =
            &spell_dto_duration.value
        else {
            panic!("spell DTO duplicate scalar must be exact unsupported")
        };
        let spell_duration = spell
            .definition
            .duration
            .as_value()
            .and_then(SpellSourceValue::as_known)
            .expect("canonical spell duration");
        let FactValue::Value(SpellSourceValue::Unsupported(spell_duration_value)) =
            &spell_duration.value
        else {
            panic!("canonical duplicate scalar must be exact unsupported")
        };
        let expected_duplicate_duration = r#"[{"key":"value","authored_order":0,"value":"1 round"},{"key":"value","authored_order":1,"value":"2 rounds"}]"#;
        assert_eq!(
            spell_dto_duration_value.shape,
            UnsupportedSourceShape::Array
        );
        assert_eq!(spell_duration_value.shape, UnsupportedSourceShape::Array);
        assert_eq!(spell_dto_duration_value.value, expected_duplicate_duration);
        assert_eq!(spell_duration_value.value, expected_duplicate_duration);
        let FactValue::Value(SpellSourceValue::Unsupported(spell_dto_description)) =
            &spell_dto.description_markup
        else {
            panic!("spell DTO duplicate description value must be exact unsupported")
        };
        let expected_duplicate_description = r#"[{"key":"value","authored_order":0,"value":"First public description"},{"key":"value","authored_order":1,"value":"Second public description"}]"#;
        assert_eq!(spell_dto_description.value, expected_duplicate_description);
        let canonical_description = spell
            .definition
            .unsupported_notes
            .iter()
            .find(|note| note.source_path == "system.description")
            .expect("canonical duplicate description evidence");
        assert_eq!(
            canonical_description.value.value,
            r#"[{"key":"description","authored_order":0,"value":{"gm":"GM context","value":"First public description","value":"Second public description"}}]"#
        );
        let spell_unknown = spell
            .definition
            .unsupported_notes
            .iter()
            .find(|fact| fact.source_path == "system.mystery")
            .expect("exact spell unknown");
        assert_eq!(
            spell_unknown.value.value,
            r#"{"enabled":false,"empty":{},"values":[0,null]}"#
        );

        let hazard_loaded = source
            .records
            .iter()
            .find(|loaded| loaded.record.identity.key.id().as_str() == "losslessHazard")
            .expect("lossless hazard source record");
        let hazard_dto = hazard_loaded
            .facts
            .hazard_source
            .as_ref()
            .expect("lossless hazard DTO");
        let SourcePresence::Value(crate::source::dto::HazardSourceValue::Typed(hazard_items)) =
            &hazard_dto.source.items
        else {
            panic!("hazard DTO items")
        };
        let SourcePresence::Value(crate::source::dto::HazardSourceValue::Typed(hazard_dto_damage)) =
            &hazard_items[0].strike.damage_rolls
        else {
            panic!("hazard DTO damage")
        };
        assert_eq!(
            hazard_dto_damage
                .iter()
                .map(|entry| {
                    (
                        entry.source_key.as_str(),
                        entry.authored_order,
                        match &entry.damage {
                            SourcePresence::Value(
                                crate::source::dto::HazardSourceValue::Typed(value),
                            ) => Some(value.as_str()),
                            _ => None,
                        },
                    )
                })
                .collect::<Vec<_>>(),
            [
                ("zeta-damage", 0, Some("1d6")),
                ("alpha-damage", 1, Some("1d4")),
                ("zeta-damage", 2, Some("2d6")),
            ]
        );
        let hazard_body = hazard_loaded
            .facts
            .canonical_body
            .as_ref()
            .expect("canonical lossless hazard");
        let RecordBody::Hazard(hazard) = hazard_body else {
            panic!("lossless hazard body dispatch")
        };
        let strike = match &hazard
            .embedded_entities
            .typed()
            .expect("hazard entities")
            .entities[0]
            .capability
        {
            HazardCapability::Strike(strike) => strike,
            other => panic!("expected strike, observed {other:?}"),
        };
        assert_eq!(
            strike
                .damage_rolls
                .typed()
                .expect("hazard damage entries")
                .iter()
                .map(|entry| {
                    (
                        entry.source_key.as_str(),
                        entry.authored_order,
                        entry.damage.typed().map(String::as_str),
                    )
                })
                .collect::<Vec<_>>(),
            [
                ("zeta-damage", 0, Some("1d6")),
                ("alpha-damage", 1, Some("1d4")),
                ("zeta-damage", 2, Some("2d6")),
            ]
        );
        let hazard_unknown = strike
            .unsupported_fields
            .iter()
            .find(|fact| {
                matches!(
                    &fact.field,
                    HazardUnsupportedField::StrikeUnexpected(path)
                        if path == "/items/0/system/damageRolls/zeta-damage/mystery/enabled"
                )
            })
            .expect("exact hazard unknown");
        assert_eq!(hazard_unknown.value.exact_json, "false");
    }
}
