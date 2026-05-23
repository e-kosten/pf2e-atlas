use std::collections::BTreeSet;
use std::path::Path;

use crate::{IndexBuildInput, IndexBuildPack};
use arrow_schema::DataType;
use atlas_artifact::metadata::artifact_metadata_keys;
use atlas_embedding::{EMBEDDING_UNIT_POLICY_VERSION, EmbeddingModelSpec};
use atlas_record::{
    ContentDocument, MetricRow, NormalizedRecord, build_record_fts_projection,
    metrics as metric_definitions,
};

use crate::IndexWriteError;
use crate::ladybug::writer::embeddings::LadybugEmbedding;
use crate::ladybug::writer::evidence::evidence_units;
use crate::ladybug::writer::facts::{
    alias_key, filter_value_key, metric_key_id, metric_value_type, namespace_prefix,
    publication_key, record_filter_values,
};
use crate::ladybug::writer::parquet::{
    arrow_bools, arrow_field, arrow_fixed_f32_list_field, arrow_fixed_f32_lists, arrow_ints,
    arrow_optional_bools, arrow_optional_floats, arrow_optional_ints, arrow_optional_strings,
    arrow_strings, write_parquet,
};
use crate::writer_visibility::RetrievalVisibility;

pub(crate) fn write_pack_parquet(
    staging_path: &Path,
    packs: &[IndexBuildPack<'_>],
) -> Result<(), IndexWriteError> {
    write_parquet(
        &staging_path.join("pack.parquet"),
        vec![
            arrow_field("pack_name", DataType::Utf8, false),
            arrow_field("pack_label", DataType::Utf8, false),
            arrow_field("document_type", DataType::Utf8, false),
            arrow_field("source_path", DataType::Utf8, false),
        ],
        vec![
            arrow_strings(packs.iter().map(|pack| pack.name.to_string())),
            arrow_strings(packs.iter().map(|pack| pack.label.to_string())),
            arrow_strings(packs.iter().map(|pack| pack.document_type.to_string())),
            arrow_strings(packs.iter().map(|pack| pack.declared_path.to_string())),
        ],
    )
}

pub(crate) fn write_graph_node_parquet(
    staging_path: &Path,
    input: &IndexBuildInput<'_>,
    embeddings: &[LadybugEmbedding],
    embedding_spec: EmbeddingModelSpec,
) -> Result<(), IndexWriteError> {
    let retrieval_visibility = RetrievalVisibility::from_remaster_links(input.remaster_links);
    let records = input.records.clone();

    write_artifact_metadata_parquet(staging_path, embedding_spec)?;

    write_parquet(
        &staging_path.join("record.parquet"),
        vec![
            arrow_field("record_key", DataType::Utf8, false),
            arrow_field("id", DataType::Utf8, false),
            arrow_field("name", DataType::Utf8, false),
            arrow_field("normalized_name", DataType::Utf8, false),
            arrow_field("record_family", DataType::Utf8, false),
            arrow_field("foundry_document_type", DataType::Utf8, false),
            arrow_field("foundry_record_type", DataType::Utf8, false),
            arrow_field("level", DataType::Int64, true),
            arrow_field("rarity", DataType::Utf8, true),
            arrow_field("traits_json", DataType::Utf8, false),
            arrow_field("prerequisites_json", DataType::Utf8, false),
            arrow_field("system_category", DataType::Utf8, true),
            arrow_field("system_group", DataType::Utf8, true),
            arrow_field("system_base_item", DataType::Utf8, true),
            arrow_field("system_usage", DataType::Utf8, true),
            arrow_field("system_price_json", DataType::Utf8, true),
            arrow_field("system_actions_value", DataType::Int64, true),
            arrow_field("system_time_value", DataType::Utf8, true),
            arrow_field("system_duration_value", DataType::Utf8, true),
            arrow_field("price_cp", DataType::Int64, true),
            arrow_field("activation_time_kind", DataType::Utf8, true),
            arrow_field("activation_time_actions", DataType::Int64, true),
            arrow_field("activation_time_duration_value", DataType::Int64, true),
            arrow_field("activation_time_duration_unit", DataType::Utf8, true),
            arrow_field("activation_time_text", DataType::Utf8, true),
            arrow_field("duration_kind", DataType::Utf8, true),
            arrow_field("duration_value", DataType::Int64, true),
            arrow_field("duration_unit", DataType::Utf8, true),
            arrow_field("duration_text", DataType::Utf8, true),
            arrow_field("publication_title", DataType::Utf8, true),
            arrow_field("publication_family", DataType::Utf8, false),
            arrow_field("publication_remaster", DataType::Boolean, false),
            arrow_field("description_json", DataType::Utf8, true),
            arrow_field("blurb_json", DataType::Utf8, true),
            arrow_field("folder_id", DataType::Utf8, true),
            arrow_field("taxonomy_families_json", DataType::Utf8, false),
            arrow_field("variant_group_key", DataType::Utf8, true),
            arrow_field("variant_base_name", DataType::Utf8, true),
            arrow_field("variant_label", DataType::Utf8, true),
            arrow_field("variant_axes_json", DataType::Utf8, false),
            arrow_field("variant_confidence", DataType::Float64, true),
            arrow_field("variant_source", DataType::Utf8, false),
            arrow_field("is_default_visible", DataType::Boolean, false),
            arrow_field("source_path", DataType::Utf8, false),
            arrow_field("raw_json", DataType::Utf8, false),
            arrow_field("actor_size", DataType::Utf8, true),
            arrow_field("actor_languages_json", DataType::Utf8, true),
            arrow_field("actor_speed_types_json", DataType::Utf8, true),
            arrow_field("actor_senses_json", DataType::Utf8, true),
            arrow_field("actor_immunities_json", DataType::Utf8, true),
            arrow_field("actor_resistances_json", DataType::Utf8, true),
            arrow_field("actor_weaknesses_json", DataType::Utf8, true),
            arrow_field("actor_disable_text", DataType::Utf8, true),
            arrow_field("actor_disable_skills_json", DataType::Utf8, true),
            arrow_field("actor_is_complex", DataType::Boolean, true),
            arrow_field("item_bulk_value", DataType::Float64, true),
            arrow_field("item_hands_requirement", DataType::Utf8, true),
            arrow_field("item_damage_types_json", DataType::Utf8, true),
            arrow_field("spell_traditions_json", DataType::Utf8, true),
            arrow_field("spell_kinds_json", DataType::Utf8, true),
            arrow_field("spell_range_text", DataType::Utf8, true),
            arrow_field("spell_range_value", DataType::Float64, true),
            arrow_field("spell_target_text", DataType::Utf8, true),
            arrow_field("spell_area_type", DataType::Utf8, true),
            arrow_field("spell_area_value", DataType::Float64, true),
            arrow_field("spell_save_type", DataType::Utf8, true),
            arrow_field("spell_sustained", DataType::Boolean, true),
            arrow_field("spell_basic_save", DataType::Boolean, true),
            arrow_field("spell_damage_types_json", DataType::Utf8, true),
        ],
        vec![
            arrow_strings(records.iter().map(|record| record.key.to_string())),
            arrow_strings(records.iter().map(|record| record.id.to_string())),
            arrow_strings(records.iter().map(|record| record.name.clone())),
            arrow_strings(records.iter().map(|record| record.normalized_name.clone())),
            arrow_strings(
                records
                    .iter()
                    .map(|record| record.record_family.as_str().to_string()),
            ),
            arrow_strings(
                records
                    .iter()
                    .map(|record| record.foundry_document_type.as_str().to_string()),
            ),
            arrow_strings(
                records
                    .iter()
                    .map(|record| record.foundry_record_type.as_str().to_string()),
            ),
            arrow_optional_ints(records.iter().map(|record| record.level)),
            arrow_optional_strings(records.iter().map(|record| record.rarity.clone())),
            arrow_strings(
                records
                    .iter()
                    .map(|record| json_string_array(&record.traits)),
            ),
            arrow_strings(
                records
                    .iter()
                    .map(|record| json_string_array(&record.prerequisites)),
            ),
            arrow_optional_strings(records.iter().map(|record| record.system_category.clone())),
            arrow_optional_strings(records.iter().map(|record| record.system_group.clone())),
            arrow_optional_strings(records.iter().map(|record| record.system_base_item.clone())),
            arrow_optional_strings(records.iter().map(|record| record.system_usage.clone())),
            arrow_optional_strings(
                records
                    .iter()
                    .map(|record| record.system_price_json.clone()),
            ),
            arrow_optional_ints(records.iter().map(|record| record.system_actions_value)),
            arrow_optional_strings(
                records
                    .iter()
                    .map(|record| record.system_time_value.clone()),
            ),
            arrow_optional_strings(
                records
                    .iter()
                    .map(|record| record.system_duration_value.clone()),
            ),
            arrow_optional_ints(records.iter().map(|record| record.price_cp)),
            arrow_optional_strings(records.iter().map(|record| {
                record
                    .activation_time
                    .as_ref()
                    .map(|value| value.kind.as_str().to_string())
            })),
            arrow_optional_ints(records.iter().map(|record| {
                record
                    .activation_time
                    .as_ref()
                    .and_then(|value| value.actions)
            })),
            arrow_optional_ints(records.iter().map(|record| {
                record
                    .activation_time
                    .as_ref()
                    .and_then(|value| value.duration_value)
            })),
            arrow_optional_strings(records.iter().map(|record| {
                record
                    .activation_time
                    .as_ref()
                    .and_then(|value| value.duration_unit.map(|unit| unit.as_str().to_string()))
            })),
            arrow_optional_strings(records.iter().map(|record| {
                record
                    .activation_time
                    .as_ref()
                    .map(|value| value.text.clone())
            })),
            arrow_optional_strings(records.iter().map(|record| {
                record
                    .duration
                    .as_ref()
                    .map(|value| value.kind.as_str().to_string())
            })),
            arrow_optional_ints(records.iter().map(|record| {
                record
                    .duration
                    .as_ref()
                    .and_then(|value| value.duration_value)
            })),
            arrow_optional_strings(records.iter().map(|record| {
                record
                    .duration
                    .as_ref()
                    .and_then(|value| value.duration_unit.map(|unit| unit.as_str().to_string()))
            })),
            arrow_optional_strings(
                records
                    .iter()
                    .map(|record| record.duration.as_ref().map(|value| value.text.clone())),
            ),
            arrow_optional_strings(
                records
                    .iter()
                    .map(|record| record.publication_title.clone()),
            ),
            arrow_strings(
                records
                    .iter()
                    .map(|record| record.publication_family.as_str().to_string()),
            ),
            arrow_bools(records.iter().map(|record| record.publication_remaster)),
            arrow_optional_strings(
                records
                    .iter()
                    .map(|record| optional_content_json(&record.description)),
            ),
            arrow_optional_strings(
                records
                    .iter()
                    .map(|record| optional_content_json(&record.blurb)),
            ),
            arrow_optional_strings(records.iter().map(|record| record.folder_id.clone())),
            arrow_strings(
                records
                    .iter()
                    .map(|record| json_string_array(&record.taxonomy_families)),
            ),
            arrow_optional_strings(
                records
                    .iter()
                    .map(|record| record.variant_group_key.clone()),
            ),
            arrow_optional_strings(
                records
                    .iter()
                    .map(|record| record.variant_base_name.clone()),
            ),
            arrow_optional_strings(records.iter().map(|record| record.variant_label.clone())),
            arrow_strings(
                records
                    .iter()
                    .map(|record| json_string_array(&record.variant_axes)),
            ),
            arrow_optional_floats(records.iter().map(|record| record.variant_confidence)),
            arrow_strings(records.iter().map(|record| record.variant_source.clone())),
            arrow_bools(
                records
                    .iter()
                    .map(|record| retrieval_visibility.is_default_visible(record)),
            ),
            arrow_strings(records.iter().map(|record| record.source_path.clone())),
            arrow_strings(records.iter().map(|record| record.raw_json.clone())),
            arrow_optional_strings(records.iter().map(|record| {
                record
                    .actor_data
                    .as_ref()
                    .and_then(|value| value.size.clone())
            })),
            arrow_optional_strings(records.iter().map(|record| {
                record
                    .actor_data
                    .as_ref()
                    .map(|value| json_string_array(&value.languages))
            })),
            arrow_optional_strings(records.iter().map(|record| {
                record
                    .actor_data
                    .as_ref()
                    .map(|value| json_string_array(&value.speed_types))
            })),
            arrow_optional_strings(records.iter().map(|record| {
                record
                    .actor_data
                    .as_ref()
                    .map(|value| json_string_array(&value.senses))
            })),
            arrow_optional_strings(records.iter().map(|record| {
                record
                    .actor_data
                    .as_ref()
                    .map(|value| json_string_array(&value.immunities))
            })),
            arrow_optional_strings(records.iter().map(|record| {
                record
                    .actor_data
                    .as_ref()
                    .map(|value| json_string_array(&value.resistances))
            })),
            arrow_optional_strings(records.iter().map(|record| {
                record
                    .actor_data
                    .as_ref()
                    .map(|value| json_string_array(&value.weaknesses))
            })),
            arrow_optional_strings(records.iter().map(|record| {
                record
                    .actor_data
                    .as_ref()
                    .and_then(|value| value.disable_text.clone())
            })),
            arrow_optional_strings(records.iter().map(|record| {
                record
                    .actor_data
                    .as_ref()
                    .map(|value| json_string_array(&value.disable_skills))
            })),
            arrow_optional_bools(
                records
                    .iter()
                    .map(|record| record.actor_data.as_ref().map(|value| value.is_complex)),
            ),
            arrow_optional_floats(
                records
                    .iter()
                    .map(|record| record.item_data.as_ref().and_then(|value| value.bulk_value)),
            ),
            arrow_optional_strings(records.iter().map(|record| {
                record
                    .item_data
                    .as_ref()
                    .and_then(|value| value.hands_requirement.clone())
            })),
            arrow_optional_strings(records.iter().map(|record| {
                record
                    .item_data
                    .as_ref()
                    .map(|value| json_string_array(&value.damage_types))
            })),
            arrow_optional_strings(records.iter().map(|record| {
                record
                    .spell_data
                    .as_ref()
                    .map(|value| json_string_array(&value.traditions))
            })),
            arrow_optional_strings(records.iter().map(|record| {
                record
                    .spell_data
                    .as_ref()
                    .map(|value| json_string_array(&value.spell_kinds))
            })),
            arrow_optional_strings(records.iter().map(|record| {
                record
                    .spell_data
                    .as_ref()
                    .and_then(|value| value.range_text.clone())
            })),
            arrow_optional_floats(records.iter().map(|record| {
                record
                    .spell_data
                    .as_ref()
                    .and_then(|value| value.range_value)
            })),
            arrow_optional_strings(records.iter().map(|record| {
                record
                    .spell_data
                    .as_ref()
                    .and_then(|value| value.target_text.clone())
            })),
            arrow_optional_strings(records.iter().map(|record| {
                record
                    .spell_data
                    .as_ref()
                    .and_then(|value| value.area_type.clone())
            })),
            arrow_optional_floats(records.iter().map(|record| {
                record
                    .spell_data
                    .as_ref()
                    .and_then(|value| value.area_value)
            })),
            arrow_optional_strings(records.iter().map(|record| {
                record
                    .spell_data
                    .as_ref()
                    .and_then(|value| value.save_type.clone())
            })),
            arrow_optional_bools(
                records
                    .iter()
                    .map(|record| record.spell_data.as_ref().map(|value| value.sustained)),
            ),
            arrow_optional_bools(
                records
                    .iter()
                    .map(|record| record.spell_data.as_ref().map(|value| value.basic_save)),
            ),
            arrow_optional_strings(records.iter().map(|record| {
                record
                    .spell_data
                    .as_ref()
                    .map(|value| json_string_array(&value.damage_types))
            })),
        ],
    )?;

    let mut search_doc_keys = Vec::new();
    let mut search_record_keys = Vec::new();
    let mut titles = Vec::new();
    let mut aliases = Vec::new();
    let mut traits = Vec::new();
    let mut taxonomy_terms = Vec::new();
    let mut constraint_terms = Vec::new();
    let mut mechanic_terms = Vec::new();
    let mut source_terms = Vec::new();
    let mut metric_terms = Vec::new();
    let mut headings = Vec::new();
    let mut bodies = Vec::new();
    let mut facts = Vec::new();
    let mut reference_terms = Vec::new();
    let mut embedded_content = Vec::new();
    for record in &records {
        if !retrieval_visibility.is_default_visible(record) {
            continue;
        }
        let record_aliases = input
            .aliases
            .iter()
            .filter(|alias| alias.canonical_record_key == record.key)
            .map(|alias| alias.alias_text.clone())
            .collect::<Vec<_>>();
        let fts = build_record_fts_projection(record, &record_aliases);
        search_doc_keys.push(format!("{}#fts", record.key));
        search_record_keys.push(record.key.to_string());
        titles.push(fts.title);
        aliases.push(fts.aliases);
        traits.push(fts.traits);
        taxonomy_terms.push(fts.taxonomy_terms);
        constraint_terms.push(fts.constraint_terms);
        mechanic_terms.push(fts.mechanic_terms);
        source_terms.push(fts.source_terms);
        metric_terms.push(fts.metric_terms);
        headings.push(fts.headings);
        bodies.push(fts.body);
        facts.push(fts.facts);
        reference_terms.push(fts.references);
        embedded_content.push(fts.embedded_content);
    }
    write_parquet(
        &staging_path.join("search_document.parquet"),
        vec![
            arrow_field("search_doc_key", DataType::Utf8, false),
            arrow_field("record_key", DataType::Utf8, false),
            arrow_field("title", DataType::Utf8, false),
            arrow_field("aliases", DataType::Utf8, false),
            arrow_field("traits", DataType::Utf8, false),
            arrow_field("taxonomy_terms", DataType::Utf8, false),
            arrow_field("constraint_terms", DataType::Utf8, false),
            arrow_field("mechanic_terms", DataType::Utf8, false),
            arrow_field("source_terms", DataType::Utf8, false),
            arrow_field("metric_terms", DataType::Utf8, false),
            arrow_field("headings", DataType::Utf8, false),
            arrow_field("body", DataType::Utf8, false),
            arrow_field("facts", DataType::Utf8, false),
            arrow_field("reference_terms", DataType::Utf8, false),
            arrow_field("embedded_content", DataType::Utf8, false),
        ],
        vec![
            arrow_strings(search_doc_keys),
            arrow_strings(search_record_keys),
            arrow_strings(titles),
            arrow_strings(aliases),
            arrow_strings(traits),
            arrow_strings(taxonomy_terms),
            arrow_strings(constraint_terms),
            arrow_strings(mechanic_terms),
            arrow_strings(source_terms),
            arrow_strings(metric_terms),
            arrow_strings(headings),
            arrow_strings(bodies),
            arrow_strings(facts),
            arrow_strings(reference_terms),
            arrow_strings(embedded_content),
        ],
    )?;

    write_embedding_unit_parquet(staging_path, embeddings)?;
    write_content_unit_parquet(staging_path, &records)?;
    write_evidence_unit_parquet(staging_path, &records)?;
    write_trait_parquet(staging_path, &records)?;
    write_filter_value_parquet(staging_path, &records)?;
    write_publication_parquet(staging_path, &records)?;
    write_alias_parquet(staging_path, input)?;
    write_metric_parquet(staging_path, &records)?;
    write_variant_group_parquet(staging_path, &records)?;
    Ok(())
}

fn write_artifact_metadata_parquet(
    staging_path: &Path,
    spec: EmbeddingModelSpec,
) -> Result<(), IndexWriteError> {
    let metadata = [
        (
            artifact_metadata_keys::EMBEDDING_PROVIDER_FAMILY,
            spec.provider_family.to_string(),
        ),
        (
            artifact_metadata_keys::EMBEDDING_MODEL_ID,
            spec.model_id.to_string(),
        ),
        (
            artifact_metadata_keys::EMBEDDING_MODEL_REVISION,
            spec.model_revision.to_string(),
        ),
        (
            artifact_metadata_keys::EMBEDDING_TOKENIZER_ID,
            spec.tokenizer_id.to_string(),
        ),
        (
            artifact_metadata_keys::EMBEDDING_POOLING,
            spec.pooling.as_str().to_string(),
        ),
        (
            artifact_metadata_keys::EMBEDDING_NORMALIZATION,
            spec.normalization.as_str().to_string(),
        ),
        (
            artifact_metadata_keys::EMBEDDING_DIMENSIONS,
            spec.dimensions.to_string(),
        ),
        (
            artifact_metadata_keys::EMBEDDING_DTYPE,
            spec.dtype.as_str().to_string(),
        ),
        (
            artifact_metadata_keys::EMBEDDING_DISTANCE_METRIC,
            spec.distance_metric.as_str().to_string(),
        ),
        (
            artifact_metadata_keys::EMBEDDING_DOCUMENT_PREFIX,
            spec.document_prefix.to_string(),
        ),
        (
            artifact_metadata_keys::EMBEDDING_QUERY_PREFIX,
            spec.query_prefix.to_string(),
        ),
        (
            artifact_metadata_keys::EMBEDDING_UNIT_POLICY_VERSION,
            EMBEDDING_UNIT_POLICY_VERSION.to_string(),
        ),
    ];
    write_parquet(
        &staging_path.join("artifact_metadata.parquet"),
        vec![
            arrow_field("key", DataType::Utf8, false),
            arrow_field("value", DataType::Utf8, false),
        ],
        vec![
            arrow_strings(metadata.iter().map(|(key, _)| (*key).to_string())),
            arrow_strings(metadata.iter().map(|(_, value)| value.clone())),
        ],
    )
}

fn write_embedding_unit_parquet(
    staging_path: &Path,
    embeddings: &[LadybugEmbedding],
) -> Result<(), IndexWriteError> {
    let dimensions = embeddings
        .first()
        .map_or(384, |embedding| embedding.dimensions);
    write_parquet(
        &staging_path.join("embedding_unit.parquet"),
        vec![
            arrow_field("embedding_unit_key", DataType::Utf8, false),
            arrow_field("record_key", DataType::Utf8, false),
            arrow_field("unit_kind", DataType::Utf8, false),
            arrow_field("label", DataType::Utf8, true),
            arrow_field("ordinal", DataType::Int64, false),
            arrow_field("semantic_input_hash", DataType::Utf8, false),
            arrow_field("dimensions", DataType::Int64, false),
            arrow_fixed_f32_list_field("embedding", dimensions),
        ],
        vec![
            arrow_strings(
                embeddings
                    .iter()
                    .map(|embedding| embedding.embedding_unit_key.clone()),
            ),
            arrow_strings(
                embeddings
                    .iter()
                    .map(|embedding| embedding.record_key.clone()),
            ),
            arrow_strings(
                embeddings
                    .iter()
                    .map(|embedding| embedding.unit_kind.clone()),
            ),
            arrow_optional_strings(embeddings.iter().map(|embedding| embedding.label.clone())),
            arrow_ints(embeddings.iter().map(|embedding| embedding.ordinal)),
            arrow_strings(
                embeddings
                    .iter()
                    .map(|embedding| embedding.semantic_input_hash.clone()),
            ),
            arrow_ints(
                embeddings
                    .iter()
                    .map(|embedding| embedding.dimensions as i64),
            ),
            arrow_fixed_f32_lists(
                embeddings
                    .iter()
                    .map(|embedding| embedding.vector.as_slice()),
                dimensions,
            )?,
        ],
    )
}

fn write_content_unit_parquet(
    staging_path: &Path,
    records: &[&NormalizedRecord],
) -> Result<(), IndexWriteError> {
    let mut content_unit_keys = Vec::new();
    let mut record_keys = Vec::new();
    let mut ordinals = Vec::new();
    let mut source_kinds = Vec::new();
    let mut visibilities = Vec::new();
    let mut contributes_to_search = Vec::new();
    let mut contributes_to_references = Vec::new();
    let mut labels = Vec::new();
    let mut content_jsons = Vec::new();
    for record in records {
        for (ordinal, supplemental) in record.supplemental_content.iter().enumerate() {
            content_unit_keys.push(format!("{}#content#{ordinal}", record.key));
            record_keys.push(record.key.to_string());
            ordinals.push(ordinal as i64);
            source_kinds.push(supplemental.source_kind.as_str().to_string());
            visibilities.push(supplemental.visibility.as_str().to_string());
            contributes_to_search.push(supplemental.contributes_to_search);
            contributes_to_references.push(supplemental.contributes_to_references);
            labels.push(supplemental.label.clone());
            content_jsons.push(
                serde_json::to_string(&supplemental.document)
                    .map_err(|error| IndexWriteError::WriteFailed(error.to_string()))?,
            );
        }
    }
    write_parquet(
        &staging_path.join("content_unit.parquet"),
        vec![
            arrow_field("content_unit_key", DataType::Utf8, false),
            arrow_field("record_key", DataType::Utf8, false),
            arrow_field("ordinal", DataType::Int64, false),
            arrow_field("source_kind", DataType::Utf8, false),
            arrow_field("visibility", DataType::Utf8, false),
            arrow_field("contributes_to_search", DataType::Boolean, false),
            arrow_field("contributes_to_references", DataType::Boolean, false),
            arrow_field("label", DataType::Utf8, true),
            arrow_field("content_json", DataType::Utf8, false),
        ],
        vec![
            arrow_strings(content_unit_keys),
            arrow_strings(record_keys),
            arrow_ints(ordinals),
            arrow_strings(source_kinds),
            arrow_strings(visibilities),
            arrow_bools(contributes_to_search),
            arrow_bools(contributes_to_references),
            arrow_optional_strings(labels),
            arrow_strings(content_jsons),
        ],
    )
}

fn write_evidence_unit_parquet(
    staging_path: &Path,
    records: &[&NormalizedRecord],
) -> Result<(), IndexWriteError> {
    let evidence_units = evidence_units(records);
    write_parquet(
        &staging_path.join("evidence_unit.parquet"),
        vec![
            arrow_field("evidence_unit_key", DataType::Utf8, false),
            arrow_field("record_key", DataType::Utf8, false),
            arrow_field("source_content_unit_key", DataType::Utf8, true),
            arrow_field("source_kind", DataType::Utf8, false),
            arrow_field("visibility", DataType::Utf8, false),
            arrow_field("unit_kind", DataType::Utf8, false),
            arrow_field("label", DataType::Utf8, true),
            arrow_field("ordinal", DataType::Int64, false),
            arrow_field("search_text", DataType::Utf8, false),
        ],
        vec![
            arrow_strings(
                evidence_units
                    .iter()
                    .map(|unit| unit.evidence_unit_key.clone()),
            ),
            arrow_strings(evidence_units.iter().map(|unit| unit.record_key.clone())),
            arrow_optional_strings(
                evidence_units
                    .iter()
                    .map(|unit| unit.source_content_unit_key.clone()),
            ),
            arrow_strings(
                evidence_units
                    .iter()
                    .map(|unit| unit.source_kind.as_str().to_string()),
            ),
            arrow_strings(
                evidence_units
                    .iter()
                    .map(|unit| unit.visibility.as_str().to_string()),
            ),
            arrow_strings(evidence_units.iter().map(|unit| unit.unit_kind.clone())),
            arrow_optional_strings(evidence_units.iter().map(|unit| unit.label.clone())),
            arrow_ints(evidence_units.iter().map(|unit| unit.ordinal)),
            arrow_strings(evidence_units.iter().map(|unit| unit.search_text.clone())),
        ],
    )
}

fn write_trait_parquet(
    staging_path: &Path,
    records: &[&NormalizedRecord],
) -> Result<(), IndexWriteError> {
    let names = records
        .iter()
        .flat_map(|record| record.traits.iter().cloned())
        .collect::<BTreeSet<_>>();
    write_parquet(
        &staging_path.join("trait.parquet"),
        vec![arrow_field("name", DataType::Utf8, false)],
        vec![arrow_strings(names)],
    )
}

fn write_filter_value_parquet(
    staging_path: &Path,
    records: &[&NormalizedRecord],
) -> Result<(), IndexWriteError> {
    let mut values = BTreeSet::new();
    for record in records {
        values.extend(record_filter_values(record));
    }
    write_parquet(
        &staging_path.join("filter_value.parquet"),
        vec![
            arrow_field("filter_value_key", DataType::Utf8, false),
            arrow_field("field", DataType::Utf8, false),
            arrow_field("value", DataType::Utf8, false),
        ],
        vec![
            arrow_strings(
                values
                    .iter()
                    .map(|(field, value)| filter_value_key(field, value)),
            ),
            arrow_strings(values.iter().map(|(field, _)| field.to_string())),
            arrow_strings(values.into_iter().map(|(_, value)| value)),
        ],
    )
}

fn write_publication_parquet(
    staging_path: &Path,
    records: &[&NormalizedRecord],
) -> Result<(), IndexWriteError> {
    let mut publications = BTreeSet::new();
    let mut keys = Vec::new();
    let mut titles = Vec::new();
    let mut families = Vec::new();
    let mut remasters = Vec::new();
    for record in records {
        if let Some(title) = &record.publication_title {
            let key = publication_key(title);
            if publications.insert(key.clone()) {
                keys.push(key);
                titles.push(title.clone());
                families.push(record.publication_family.as_str().to_string());
                remasters.push(record.publication_remaster);
            }
        }
    }
    write_parquet(
        &staging_path.join("publication.parquet"),
        vec![
            arrow_field("publication_key", DataType::Utf8, false),
            arrow_field("title", DataType::Utf8, false),
            arrow_field("family", DataType::Utf8, false),
            arrow_field("remaster", DataType::Boolean, false),
        ],
        vec![
            arrow_strings(keys),
            arrow_strings(titles),
            arrow_strings(families),
            arrow_bools(remasters),
        ],
    )
}

fn write_alias_parquet(
    staging_path: &Path,
    input: &IndexBuildInput<'_>,
) -> Result<(), IndexWriteError> {
    write_parquet(
        &staging_path.join("alias.parquet"),
        vec![
            arrow_field("alias_key", DataType::Utf8, false),
            arrow_field("alias_text", DataType::Utf8, false),
            arrow_field("normalized_alias", DataType::Utf8, false),
            arrow_field("source_kind", DataType::Utf8, false),
            arrow_field("source_ref", DataType::Utf8, false),
        ],
        vec![
            arrow_strings(input.aliases.iter().map(alias_key)),
            arrow_strings(input.aliases.iter().map(|alias| alias.alias_text.clone())),
            arrow_strings(
                input
                    .aliases
                    .iter()
                    .map(|alias| alias.normalized_alias.clone()),
            ),
            arrow_strings(
                input
                    .aliases
                    .iter()
                    .map(|alias| alias.source.as_str().to_string()),
            ),
            arrow_strings(input.aliases.iter().map(|alias| alias.source_ref.clone())),
        ],
    )
}

fn write_metric_parquet(
    staging_path: &Path,
    records: &[&NormalizedRecord],
) -> Result<(), IndexWriteError> {
    let mut metrics = BTreeSet::new();
    let mut ids = Vec::new();
    let mut domains = Vec::new();
    let mut keys = Vec::new();
    let mut value_types = Vec::new();
    let mut namespace_prefixes = Vec::new();
    let mut labels = Vec::new();
    let mut short_labels = Vec::new();
    let mut groups = Vec::new();
    let mut known = Vec::new();
    for record in records {
        for metric in &record.metrics {
            let id = metric_key_id(metric.domain.as_str(), &metric.key);
            if metrics.insert(id.clone()) {
                let metadata = metric_node_metadata(metric);
                ids.push(id);
                domains.push(metric.domain.as_str().to_string());
                keys.push(metric.key.clone());
                value_types.push(metric_value_type(&metric.value).to_string());
                namespace_prefixes.push(metadata.namespace_prefix);
                labels.push(metadata.label);
                short_labels.push(metadata.short_label);
                groups.push(metadata.group);
                known.push(metadata.known);
            }
        }
    }
    write_parquet(
        &staging_path.join("metric.parquet"),
        vec![
            arrow_field("metric_key_id", DataType::Utf8, false),
            arrow_field("metric_domain", DataType::Utf8, false),
            arrow_field("metric_key", DataType::Utf8, false),
            arrow_field("value_type", DataType::Utf8, false),
            arrow_field("namespace_prefix", DataType::Utf8, false),
            arrow_field("label", DataType::Utf8, true),
            arrow_field("short_label", DataType::Utf8, true),
            arrow_field("group_name", DataType::Utf8, true),
            arrow_field("known", DataType::Boolean, false),
        ],
        vec![
            arrow_strings(ids),
            arrow_strings(domains),
            arrow_strings(keys),
            arrow_strings(value_types),
            arrow_strings(namespace_prefixes),
            arrow_optional_strings(labels),
            arrow_optional_strings(short_labels),
            arrow_optional_strings(groups),
            arrow_bools(known),
        ],
    )
}

struct MetricNodeMetadata {
    namespace_prefix: String,
    label: Option<String>,
    short_label: Option<String>,
    group: Option<String>,
    known: bool,
}

fn metric_node_metadata(metric: &MetricRow) -> MetricNodeMetadata {
    let matched = metric_definitions::definition_for(metric.domain, &metric.key);
    let display = metric_definitions::label_for_row(metric);
    MetricNodeMetadata {
        namespace_prefix: matched
            .as_ref()
            .map(|matched| matched.definition.namespace().to_string())
            .unwrap_or_else(|| namespace_prefix(&metric.key)),
        label: display.known.then_some(display.label),
        short_label: display.short_label,
        group: matched.map(|matched| matched.definition.group().as_str().to_string()),
        known: display.known,
    }
}

fn write_variant_group_parquet(
    staging_path: &Path,
    records: &[&NormalizedRecord],
) -> Result<(), IndexWriteError> {
    let mut variants = BTreeSet::new();
    let mut keys = Vec::new();
    let mut base_names = Vec::new();
    let mut sources = Vec::new();
    let mut confidences = Vec::new();
    for record in records {
        if let Some(key) = &record.variant_group_key
            && variants.insert(key.clone())
        {
            keys.push(key.clone());
            base_names.push(record.variant_base_name.clone());
            sources.push(record.variant_source.as_str().to_string());
            confidences.push(record.variant_confidence);
        }
    }
    write_parquet(
        &staging_path.join("variant_group.parquet"),
        vec![
            arrow_field("variant_group_key", DataType::Utf8, false),
            arrow_field("base_name", DataType::Utf8, true),
            arrow_field("source", DataType::Utf8, false),
            arrow_field("confidence", DataType::Float64, true),
        ],
        vec![
            arrow_strings(keys),
            arrow_optional_strings(base_names),
            arrow_strings(sources),
            arrow_optional_floats(confidences),
        ],
    )
}

fn json_string_array(values: &[String]) -> String {
    serde_json::to_string(values).unwrap_or_else(|_| "[]".to_string())
}

fn optional_content_json(document: &Option<ContentDocument>) -> Option<String> {
    document
        .as_ref()
        .and_then(|document| serde_json::to_string(document).ok())
}
