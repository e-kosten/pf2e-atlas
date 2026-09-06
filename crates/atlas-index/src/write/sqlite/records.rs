use atlas_record::{
    AtlasRecord, ContentDiagnostic, ContentOrigin, ContentProvenance, DuplicateContentStatus,
    FoundryDocumentMechanics, ProductRetrievalPolicy, RecordBody, build_search_fts_projection,
    project_creature_facts, project_hazard_facts,
};
use diesel::SqliteConnection;
use diesel::prelude::*;
use sha2::{Digest, Sha256};

use super::labels::{
    metric_domain_label, metric_value_parts, publication_family_label, rarity_label,
    time_kind_label, time_unit_label,
};
use super::models::{
    ActorRecordRow, ItemRecordRow, RecordContentRow, RecordMetricRow, RecordRow, RecordTraitRow,
    RecordsFtsRow,
};
use crate::IndexWriteError;
use atlas_record::{RecordAlias, RemasterLink};

pub(super) fn write_records(
    connection: &mut SqliteConnection,
    records: &[AtlasRecord],
    aliases: &[RecordAlias],
    remaster_links: &[RemasterLink],
    canonical_bodies: &[RecordBody],
    canonical_record_keys: &std::collections::BTreeSet<String>,
) -> Result<(), IndexWriteError> {
    let retrieval_policy = ProductRetrievalPolicy::from_remaster_links(remaster_links);
    let canonical_bodies_by_key = canonical_bodies_by_key(records, canonical_bodies)?;
    let mut record_rows = Vec::new();
    let mut content_rows = Vec::new();
    let mut trait_rows = Vec::new();
    let mut actor_rows = Vec::new();
    let mut item_rows = Vec::new();
    let mut metric_rows = Vec::new();
    let mut fts_rows = Vec::new();
    for record in records {
        let record_key = record.identity.key.to_string();
        let projected_metrics;
        let persisted_metrics = if matches!(
            record.classification.kind,
            atlas_domain::RecordKind::Creature | atlas_domain::RecordKind::Hazard
        ) {
            let expected_foundry_type = match record.classification.kind {
                atlas_domain::RecordKind::Creature => atlas_record::FoundryRecordType::Npc,
                atlas_domain::RecordKind::Hazard => atlas_record::FoundryRecordType::Hazard,
                _ => unreachable!("guarded canonical family"),
            };
            if record.foundry.record_type != expected_foundry_type {
                return Err(IndexWriteError::WriteFailed(format!(
                    "{} record `{}` does not have the expected Foundry body kind",
                    record.classification.kind.as_str(),
                    record.identity.key
                )));
            }
            if !matches!(record.mechanics.document, FoundryDocumentMechanics::None) {
                return Err(IndexWriteError::WriteFailed(format!(
                    "canonical record `{}` retains forbidden generic document mechanics",
                    record.identity.key
                )));
            }
            if !record.mechanics.metrics.is_empty() {
                return Err(IndexWriteError::WriteFailed(format!(
                    "canonical record `{}` retains forbidden generic metrics",
                    record.identity.key
                )));
            }
            let body = canonical_bodies_by_key
                .get(&record_key)
                .copied()
                .ok_or_else(|| {
                    IndexWriteError::WriteFailed(format!(
                        "canonical record `{}` is missing its required canonical body",
                        record.identity.key
                    ))
                })?;
            projected_metrics = match (record.classification.kind, body) {
                (atlas_domain::RecordKind::Creature, RecordBody::Creature(creature)) => {
                    project_creature_facts(creature).metrics
                }
                (atlas_domain::RecordKind::Hazard, RecordBody::Hazard(hazard)) => {
                    project_hazard_facts(hazard).metrics
                }
                (expected, actual) => {
                    let actual = match actual {
                        RecordBody::Creature(_) => "creature",
                        RecordBody::Hazard(_) => "hazard",
                        RecordBody::Spell(_) => "spell",
                    };
                    return Err(IndexWriteError::WriteFailed(format!(
                        "{} record `{}` has an unexpected {actual} body",
                        expected.as_str(),
                        record.identity.key
                    )));
                }
            };
            projected_metrics.as_slice()
        } else if record.classification.kind == atlas_domain::RecordKind::Spell {
            if record.foundry.record_type != atlas_record::FoundryRecordType::Spell {
                return Err(IndexWriteError::WriteFailed(format!(
                    "spell record `{}` does not have the spell body kind",
                    record.identity.key
                )));
            }
            let body = canonical_bodies_by_key
                .get(&record_key)
                .copied()
                .ok_or_else(|| {
                    IndexWriteError::WriteFailed(format!(
                        "spell record `{}` is missing its required canonical body",
                        record.identity.key
                    ))
                })?;
            let Some(spell) = body.as_spell() else {
                return Err(IndexWriteError::WriteFailed(format!(
                    "spell record `{}` has a non-spell canonical body",
                    record.identity.key
                )));
            };
            if spell.identity.name != record.identity.name
                || spell.identity.source_id.as_str() != record.identity.id().as_str()
            {
                return Err(IndexWriteError::WriteFailed(format!(
                    "spell body identity for `{}` does not match its generic record owner",
                    record.identity.key
                )));
            }
            if !matches!(record.mechanics.document, FoundryDocumentMechanics::None) {
                return Err(IndexWriteError::WriteFailed(format!(
                    "canonical spell `{}` retains forbidden generic document mechanics",
                    record.identity.key
                )));
            }
            if !record.mechanics.metrics.is_empty() {
                return Err(IndexWriteError::WriteFailed(format!(
                    "canonical spell `{}` retains forbidden generic metrics",
                    record.identity.key
                )));
            }
            record.mechanics.metrics.as_slice()
        } else {
            if matches!(
                record.foundry.record_type,
                atlas_record::FoundryRecordType::Npc
                    | atlas_record::FoundryRecordType::Hazard
                    | atlas_record::FoundryRecordType::Spell
            ) {
                return Err(IndexWriteError::WriteFailed(format!(
                    "record `{}` has a canonical body kind that disagrees with its record kind",
                    record.identity.key
                )));
            }
            if canonical_bodies_by_key.contains_key(&record_key) {
                return Err(IndexWriteError::WriteFailed(format!(
                    "noncanonical record `{}` has an unexpected canonical body",
                    record.identity.key
                )));
            }
            record.mechanics.metrics.as_slice()
        };
        let retrieval = retrieval_policy.decision(record);
        let record_role = retrieval.role.as_str();
        let retrieval_disposition = retrieval.disposition.as_str();
        let retrieval_rationale = retrieval.rationale.as_str();
        let is_default_visible = retrieval.disposition.is_ordinary();
        let traits_json = serde_json::to_string(&record.classification.traits)
            .map_err(|error| IndexWriteError::WriteFailed(error.to_string()))?;
        let prerequisites_json = serde_json::to_string(&record.requirements.prerequisites)
            .map_err(|error| IndexWriteError::WriteFailed(error.to_string()))?;
        let taxonomy_families_json = json_array(&record.classification.taxonomy.inferred_groups)?;
        let variant = record.variant.as_ref();
        let variant_axes_json = json_array(
            variant
                .map(|membership| membership.axes.as_slice())
                .unwrap_or_default(),
        )?;
        let activation_time = record.timing.activation_time();
        let duration = record.timing.duration_time();
        let system_actions_value = record.timing.activation_actions_value();
        let system_time_value = record.timing.activation_time_value().map(str::to_string);
        let system_duration_value = record.timing.duration_value_text().map(str::to_string);
        let item_mechanics = record.mechanics.item();
        let metric_order_sha256 =
            crate::read::records::children::metric_order_digest(persisted_metrics)
                .map_err(IndexWriteError::WriteFailed)?;
        let visibility_state = if record.visibility.visible_by_default() {
            "visible"
        } else {
            "hidden"
        };
        let visibility_reason = match record.visibility.reason() {
            atlas_record::RecordVisibilityReason::SourceRecord => "source_record",
            atlas_record::RecordVisibilityReason::GeneratedCanonical => "generated_canonical",
            atlas_record::RecordVisibilityReason::GeneratedInstance => "generated_instance",
        };
        record_rows.push(RecordRow {
            record_key: record.identity.key.to_string(),
            id: record.identity.id().as_str().to_string(),
            name: record.identity.name.clone(),
            normalized_name: record.identity.normalized_name(),
            record_kind: record.classification.kind.as_str().to_string(),
            pack_name: record.identity.pack().as_str().to_string(),
            pack_label: record.foundry.pack_label.clone(),
            foundry_document_type: record.foundry.document_type.as_str().to_string(),
            foundry_record_type: record.foundry.record_type.as_str().to_string(),
            level: record.classification.level,
            rarity: record
                .classification
                .rarity
                .map(rarity_label)
                .map(str::to_string),
            traits_json,
            prerequisites_json,
            system_category: item_mechanics.and_then(|item| item.category.clone()),
            system_group: item_mechanics.and_then(|item| item.group.clone()),
            system_base_item: item_mechanics.and_then(|item| item.base_item.clone()),
            system_usage: item_mechanics.and_then(|item| item.usage.clone()),
            system_price_json: item_mechanics.and_then(|item| item.price_json.clone()),
            system_actions_value,
            system_time_value,
            system_duration_value,
            price_cp: item_mechanics.and_then(|item| item.price_cp),
            activation_time_kind: activation_time
                .map(|time| time_kind_label(time.kind).to_string()),
            activation_time_actions: activation_time.and_then(|time| time.actions),
            activation_time_duration_value: activation_time.and_then(|time| time.duration_value),
            activation_time_duration_unit: activation_time
                .and_then(|time| time.duration_unit.map(time_unit_label))
                .map(str::to_string),
            activation_time_text: activation_time.map(|time| time.text.clone()),
            duration_kind: duration.map(|time| time_kind_label(time.kind).to_string()),
            duration_value: duration.and_then(|time| time.duration_value),
            duration_unit: duration
                .and_then(|time| time.duration_unit.map(time_unit_label))
                .map(str::to_string),
            duration_text: duration.map(|time| time.text.clone()),
            publication_title: record.publication.title.clone(),
            publication_remaster: record.publication.remaster,
            publication_family: publication_family_label(record.publication.category).to_string(),
            folder_id: record.foundry.folder_id.clone(),
            taxonomy_families_json,
            variant_group_key: variant.map(|membership| membership.group_key.clone()),
            variant_base_name: variant.map(|membership| membership.base_name.clone()),
            variant_label: variant.and_then(|membership| membership.label.clone()),
            variant_axes_json,
            variant_confidence: variant.and_then(|membership| membership.confidence),
            variant_source: variant
                .map(|membership| membership.source.as_str())
                .unwrap_or("none")
                .to_string(),
            source_path: record.provenance.source_path.clone(),
            is_default_visible,
            visibility_state: visibility_state.to_string(),
            visibility_reason: visibility_reason.to_string(),
            metric_count: to_i64(persisted_metrics.len(), "records.metric_count")?,
            metric_order_sha256,
            raw_json: record.provenance.raw_json.clone().unwrap_or_default(),
            record_role: record_role.to_string(),
            retrieval_disposition: retrieval_disposition.to_string(),
            retrieval_rationale: retrieval_rationale.to_string(),
        });
        if !canonical_record_keys.contains(&record.identity.key.to_string()) {
            let mut content_inputs = Vec::new();
            for (ordinal, content) in record.content.documents.iter().enumerate() {
                let content_json = serde_json::to_string(&content.document)
                    .map_err(|error| IndexWriteError::WriteFailed(error.to_string()))?;
                content_inputs.push((ordinal, content, content_json));
            }
            let content_keys = allocated_content_keys(&content_inputs);
            for ((ordinal, content, content_json), content_key) in
                content_inputs.into_iter().zip(content_keys)
            {
                content_rows.push(RecordContentRow {
                    record_key: record.identity.key.to_string(),
                    content_key,
                    authored_order: to_i64(ordinal, "record_content.authored_order")?,
                    identity_stability: "unstable_authored_ordinal".to_string(),
                    owner_kind: "record".to_string(),
                    owner_record_key: Some(record.identity.key.to_string()),
                    owner_entity_id: None,
                    owner_occurrence_id: None,
                    owner_occurrence_authored_order: None,
                    owner_hazard_entity_id: None,
                    owner_hazard_occurrence_id: None,
                    owner_hazard_occurrence_authored_order: None,
                    role: legacy_content_role(content.source_kind).to_string(),
                    origin_json: crate::artifact::canonical_json::encode(
                        &ContentOrigin::RecordField {
                            source_kind: content.source_kind,
                            relative_source_path: content.source_kind.as_str().to_string(),
                        },
                    )
                    .map_err(IndexWriteError::WriteFailed)?,
                    source_kind: content.source_kind.as_str().to_string(),
                    visibility: content.visibility().as_str().to_string(),
                    provenance_json: crate::artifact::canonical_json::encode(&ContentProvenance {
                        source_record_key: record.identity.key.clone(),
                        relative_source_path: content.source_kind.as_str().to_string(),
                        field_or_pointer_family: content.source_kind.as_str().to_string(),
                        nested_source_id: None,
                        authored_ordinal_or_range: Some(ordinal.to_string()),
                        authored_label: content.label.clone(),
                    })
                    .map_err(IndexWriteError::WriteFailed)?,
                    contributes_to_search: content.contributes_to_search(),
                    contributes_to_references: content.contributes_to_reference_occurrences(),
                    label: content.label.clone(),
                    content_json,
                    content_hash: atlas_record::ContentHash::for_document(&content.document)
                        .as_str()
                        .to_string(),
                    duplicate_status_json: crate::artifact::canonical_json::encode(
                        &DuplicateContentStatus::Unique,
                    )
                    .map_err(IndexWriteError::WriteFailed)?,
                    diagnostics_json: crate::artifact::canonical_json::encode(&Vec::<
                        ContentDiagnostic,
                    >::new(
                    ))
                    .map_err(IndexWriteError::WriteFailed)?,
                });
            }
        }
        for trait_value in &record.classification.traits {
            trait_rows.push(RecordTraitRow {
                record_key: record.identity.key.to_string(),
                trait_value: trait_value.clone(),
            });
        }
        if !canonical_record_keys.contains(&record_key)
            && let Some(actor_data) = record.mechanics.actor()
        {
            actor_rows.push(ActorRecordRow {
                record_key: record.identity.key.to_string(),
                size: actor_data.size.clone(),
                languages_json: json_array(&actor_data.languages)?,
                speed_types_json: json_array(&actor_data.speed_types)?,
                senses_json: json_array(&actor_data.senses)?,
                immunities_json: json_array(&actor_data.immunities)?,
                resistances_json: json_array(&actor_data.resistances)?,
                weaknesses_json: json_array(&actor_data.weaknesses)?,
                disable_text: actor_data.disable_text.clone(),
                disable_skills_json: json_array(&actor_data.disable_skills)?,
                is_complex: actor_data.is_complex,
            });
        }
        if !canonical_record_keys.contains(&record_key)
            && let Some(item_data) = record.mechanics.item()
        {
            item_rows.push(ItemRecordRow {
                record_key: record.identity.key.to_string(),
                system_category: item_data.category.clone(),
                system_base_item: item_data.base_item.clone(),
                system_group: item_data.group.clone(),
                system_usage: item_data.usage.clone(),
                system_price_json: item_data.price_json.clone(),
                price_cp: item_data.price_cp,
                bulk_value: item_data.bulk_value,
                hands_requirement: item_data.hands_requirement.clone(),
                damage_types_json: json_array(&item_data.damage_types)?,
            });
        }
        for (ordinal, metric) in persisted_metrics.iter().enumerate() {
            let (value_type, number_value, text_value, bool_value) =
                metric_value_parts(&metric.value);
            metric_rows.push(RecordMetricRow {
                record_key: record.identity.key.to_string(),
                ordinal: to_i64(ordinal, "record_metrics.ordinal")?,
                metric_domain: metric_domain_label(metric.domain).to_string(),
                metric_key: metric.key.clone(),
                value_type: value_type.to_string(),
                number_value,
                text_value: text_value.map(str::to_string),
                bool_value: bool_value.map(|value| value != 0),
            });
        }
        if is_default_visible {
            let record_aliases = aliases
                .iter()
                .filter(|alias| alias.canonical_record_key == record.identity.key)
                .map(|alias| alias.alias_text.clone())
                .collect::<Vec<_>>();
            let fts = build_search_fts_projection(
                record,
                &record_aliases,
                canonical_bodies_by_key
                    .get(&record.identity.key.to_string())
                    .copied(),
            );
            fts_rows.push(RecordsFtsRow {
                record_key: record.identity.key.to_string(),
                title: Some(fts.title),
                aliases: Some(fts.aliases),
                traits: Some(fts.traits),
                taxonomy_terms: Some(fts.taxonomy_terms),
                constraint_terms: Some(fts.constraint_terms),
                mechanic_terms: Some(fts.mechanic_terms),
                source_terms: Some(fts.source_terms),
                metric_terms: Some(fts.metric_terms),
                headings: Some(fts.headings),
                body: Some(fts.body),
                facts: Some(fts.facts),
                reference_terms: Some(fts.references),
                embedded_content: Some(fts.embedded_content),
            });
        }
    }
    for rows in record_rows.chunks(super::INSERT_BATCH_ROWS) {
        diesel::insert_into(crate::schema::records::table)
            .values(rows)
            .execute(connection)
            .map_err(|error| IndexWriteError::WriteFailed(error.to_string()))?;
    }
    for rows in content_rows.chunks(super::INSERT_BATCH_ROWS) {
        diesel::insert_into(crate::schema::record_content::table)
            .values(rows)
            .execute(connection)
            .map_err(|error| IndexWriteError::WriteFailed(error.to_string()))?;
    }
    for rows in trait_rows.chunks(super::INSERT_BATCH_ROWS) {
        diesel::insert_into(crate::schema::record_traits::table)
            .values(rows)
            .execute(connection)
            .map_err(|error| IndexWriteError::WriteFailed(error.to_string()))?;
    }
    for rows in actor_rows.chunks(super::INSERT_BATCH_ROWS) {
        diesel::insert_into(crate::schema::actor_records::table)
            .values(rows)
            .execute(connection)
            .map_err(|error| IndexWriteError::WriteFailed(error.to_string()))?;
    }
    for rows in item_rows.chunks(super::INSERT_BATCH_ROWS) {
        diesel::insert_into(crate::schema::item_records::table)
            .values(rows)
            .execute(connection)
            .map_err(|error| IndexWriteError::WriteFailed(error.to_string()))?;
    }
    for rows in metric_rows.chunks(super::INSERT_BATCH_ROWS) {
        diesel::insert_into(crate::schema::record_metrics::table)
            .values(rows)
            .execute(connection)
            .map_err(|error| IndexWriteError::WriteFailed(error.to_string()))?;
    }
    for rows in fts_rows.chunks(super::INSERT_BATCH_ROWS) {
        diesel::insert_into(crate::schema::records_fts::table)
            .values(rows)
            .execute(connection)
            .map_err(|error| IndexWriteError::WriteFailed(error.to_string()))?;
    }
    Ok(())
}

fn canonical_bodies_by_key<'a>(
    records: &[AtlasRecord],
    canonical_bodies: &'a [RecordBody],
) -> Result<std::collections::BTreeMap<String, &'a RecordBody>, IndexWriteError> {
    let record_keys = records
        .iter()
        .map(|record| record.identity.key.to_string())
        .collect::<std::collections::BTreeSet<_>>();
    let mut bodies = std::collections::BTreeMap::new();
    for body in canonical_bodies {
        let key = body.record_key().to_string();
        if !record_keys.contains(&key) {
            return Err(IndexWriteError::WriteFailed(format!(
                "canonical body `{key}` has no matching record"
            )));
        }
        if bodies.insert(key.clone(), body).is_some() {
            return Err(IndexWriteError::WriteFailed(format!(
                "record `{key}` has multiple canonical bodies"
            )));
        }
    }
    Ok(bodies)
}

fn legacy_content_role(source_kind: atlas_record::ContentSourceKind) -> &'static str {
    use atlas_record::ContentSourceKind;
    match source_kind {
        ContentSourceKind::Description => "primary_description",
        ContentSourceKind::Blurb => "summary",
        ContentSourceKind::EmbeddedItemDescription
        | ContentSourceKind::EmbeddedSpellDescription => "embedded_capability",
        ContentSourceKind::GeneratedAffliction => "generated_narrative",
        _ => "supplemental_rules",
    }
}

pub(crate) fn allocated_content_keys(
    inputs: &[(usize, &atlas_record::RecordContentDocument, String)],
) -> Vec<String> {
    let bases = inputs
        .iter()
        .map(|(_, content, content_json)| {
            let mut hasher = Sha256::new();
            hasher.update(content.source_kind.as_str().as_bytes());
            hasher.update(b"\0");
            if let Some(label) = &content.label {
                hasher.update(label.as_bytes());
            }
            hasher.update(b"\0");
            hasher.update(content_json.as_bytes());
            let digest = hasher.finalize();
            let short_hash = digest[..8]
                .iter()
                .map(|byte| format!("{byte:02x}"))
                .collect::<String>();
            format!("{}:{short_hash}", content.source_kind.as_str())
        })
        .collect::<Vec<_>>();
    let mut base_counts = std::collections::BTreeMap::<String, usize>::new();
    for base in &bases {
        *base_counts.entry(base.clone()).or_insert(0) += 1;
    }
    let mut seen = std::collections::BTreeMap::<String, usize>::new();
    bases
        .into_iter()
        .map(|base| {
            if base_counts.get(&base).copied().unwrap_or(0) <= 1 {
                return base;
            }
            let ordinal = seen.entry(base.clone()).or_insert(0);
            let key = format!("{base}:{ordinal}");
            *ordinal += 1;
            key
        })
        .collect()
}

fn json_array(values: &[String]) -> Result<String, IndexWriteError> {
    serde_json::to_string(values).map_err(|error| IndexWriteError::WriteFailed(error.to_string()))
}

fn to_i64(value: usize, field: &'static str) -> Result<i64, IndexWriteError> {
    i64::try_from(value)
        .map_err(|_| IndexWriteError::WriteFailed(format!("{field} does not fit in i64")))
}
