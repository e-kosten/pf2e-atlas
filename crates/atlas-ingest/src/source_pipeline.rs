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
    finalize_hazard_owned_content, finalize_npc_owned_content, finalize_spell_owned_content,
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
    use std::path::Path;

    use atlas_record::{
        FactValue, HazardCapability, HazardUnsupportedField, RecordBody, SpellHeightening,
        SpellRule, SpellSourceValue, SpellStandaloneTarget, UnsupportedSourceShape,
    };

    use super::load_foundry_source;
    use crate::source::dto::{SourcePresence, SpellDocumentSource};

    const FIXTURE_ROOT: &str = "tests/fixtures/foundry-source/spell-source-contract";
    const LOSSLESS_FIXTURE_ROOT: &str = "tests/fixtures/foundry-source/lossless-source-contract";

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
