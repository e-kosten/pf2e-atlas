use std::collections::BTreeSet;
use std::fs;
use std::path::Path;
use std::time::Instant;

use arrow_schema::DataType;
use atlas_artifact::storage::decode_f32_vector_blob;
use atlas_record::{
    ContentBlock, ContentDocument, ContentReference, ContentReferenceLocator, MetricRow,
    NormalizedRecord, build_record_fts_projection, iter_content_references,
    metrics as metric_definitions, render_plain_text,
};
use lbug::{Connection, Database, SystemConfig};
use rusqlite::OpenFlags;
use tracing::info;

use crate::error::IngestError;
use crate::ladybug::evidence::evidence_units;
use crate::ladybug::output::{
    LadybugOutput, ladybug_progress, ladybug_progress_message, ladybug_write_error,
};
use crate::ladybug::parquet::{
    arrow_bools, arrow_field, arrow_fixed_f32_list_field, arrow_fixed_f32_lists, arrow_ints,
    arrow_optional_bools, arrow_optional_floats, arrow_optional_ints, arrow_optional_strings,
    arrow_strings, copy_from_parquet, recreate_dir, write_parquet,
};
use crate::ladybug::schema::{create_schema, create_search_indexes};
use crate::progress::elapsed_display;
use crate::records::MetricValue;
use crate::records::visibility::RetrievalVisibility;
use crate::source::{LoadedPack, SourceLoad};

#[derive(Debug, Clone)]
struct LadybugEmbedding {
    embedding_unit_key: String,
    record_key: String,
    unit_kind: String,
    label: Option<String>,
    ordinal: i64,
    semantic_input_hash: String,
    dimensions: usize,
    vector: Vec<f32>,
}

pub(crate) fn write_artifact(path: &Path, source: &SourceLoad) -> Result<(), IngestError> {
    ladybug_progress("ladybug_write", "Preparing LadybugDB output");
    let started_at = Instant::now();
    let output = LadybugOutput::prepare(path)?;
    ladybug_progress_message(
        "ladybug_write",
        format_args!("Collecting LadybugDB embedding units"),
    );
    let embeddings = ladybug_embeddings(source)?;
    ladybug_progress_message(
        "ladybug_write",
        format_args!(
            "Collected {} LadybugDB embedding units in {}",
            embeddings.len(),
            elapsed_display(started_at)
        ),
    );
    let dimensions = embeddings
        .first()
        .map_or(384, |embedding| embedding.dimensions);
    let database =
        Database::new(output.temp_path(), SystemConfig::default()).map_err(ladybug_write_error)?;
    let connection = Connection::new(&database).map_err(ladybug_write_error)?;

    ladybug_progress("ladybug_write", "Creating LadybugDB graph schema");
    create_schema(&connection, dimensions)?;
    ladybug_progress(
        "ladybug_write",
        "Writing LadybugDB graph Parquet staging files",
    );
    write_parquet_staging_and_copy(&connection, output.staging_path(), source, &embeddings)?;
    if std::env::var_os("ATLAS_LADYBUG_CREATE_SEARCH_INDEXES").is_some() {
        ladybug_progress("ladybug_write", "Creating LadybugDB search indexes");
        create_search_indexes(&connection, !embeddings.is_empty())?;
    }
    connection
        .query("CHECKPOINT;")
        .map_err(ladybug_write_error)?;
    ladybug_progress_message(
        "ladybug_write",
        format_args!(
            "Checkpointed LadybugDB output in {}",
            elapsed_display(started_at)
        ),
    );

    drop(connection);
    drop(database);
    output.commit()
}

fn write_parquet_staging_and_copy(
    connection: &Connection<'_>,
    staging_path: &Path,
    source: &SourceLoad,
    embeddings: &[LadybugEmbedding],
) -> Result<(), IngestError> {
    let started_at = Instant::now();
    recreate_dir(staging_path)?;
    write_parquet_staging_files(staging_path, source, embeddings)?;
    ladybug_progress_message(
        "ladybug_write",
        format_args!(
            "Wrote LadybugDB Parquet staging files in {}",
            elapsed_display(started_at)
        ),
    );

    for (table, file_name) in [
        ("Pack", "pack.parquet"),
        ("Record", "record.parquet"),
        ("SearchDocument", "search_document.parquet"),
        ("EmbeddingUnit", "embedding_unit.parquet"),
        ("ContentUnit", "content_unit.parquet"),
        ("EvidenceUnit", "evidence_unit.parquet"),
        ("Trait", "trait.parquet"),
        ("FilterValue", "filter_value.parquet"),
        ("Publication", "publication.parquet"),
        ("Alias", "alias.parquet"),
        ("Metric", "metric.parquet"),
        ("VariantGroup", "variant_group.parquet"),
        ("FROM_PACK", "from_pack.parquet"),
        ("PUBLISHED_IN", "published_in.parquet"),
        ("HAS_SEARCH_DOCUMENT", "has_search_document.parquet"),
        ("HAS_CONTENT_UNIT", "has_content_unit.parquet"),
        ("HAS_EVIDENCE_UNIT", "has_evidence_unit.parquet"),
        ("CONTENT_REFERENCES", "content_references.parquet"),
        ("EVIDENCE_REFERENCES", "evidence_references.parquet"),
        ("HAS_EVIDENCE_EMBEDDING", "has_evidence_embedding.parquet"),
        ("HAS_TRAIT", "has_trait.parquet"),
        ("HAS_FILTER_VALUE", "has_filter_value.parquet"),
        ("HAS_ALIAS", "has_alias.parquet"),
        ("HAS_METRIC", "has_metric.parquet"),
        ("HAS_EMBEDDING_UNIT", "has_embedding_unit.parquet"),
        ("REFERENCES", "references.parquet"),
        ("REMASTERED_BY", "remastered_by.parquet"),
        ("IN_VARIANT_GROUP", "in_variant_group.parquet"),
    ] {
        copy_from_parquet(connection, table, &staging_path.join(file_name))?;
    }

    fs::remove_dir_all(staging_path)
        .map_err(|error| IngestError::ArtifactWriteFailed(error.to_string()))?;
    ladybug_progress_message(
        "ladybug_write",
        format_args!(
            "Copied LadybugDB Parquet staging files in {}",
            elapsed_display(started_at)
        ),
    );
    Ok(())
}

fn write_parquet_staging_files(
    staging_path: &Path,
    source: &SourceLoad,
    embeddings: &[LadybugEmbedding],
) -> Result<(), IngestError> {
    write_pack_parquet(staging_path, &source.packs)?;
    write_graph_node_parquet(staging_path, source, embeddings)?;
    write_graph_relationship_parquet(staging_path, source, embeddings)?;
    Ok(())
}

fn write_pack_parquet(staging_path: &Path, packs: &[LoadedPack]) -> Result<(), IngestError> {
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
            arrow_strings(packs.iter().map(|pack| pack.label.clone())),
            arrow_strings(packs.iter().map(|pack| pack.document_type.clone())),
            arrow_strings(packs.iter().map(|pack| pack.declared_path.clone())),
        ],
    )
}

fn write_graph_node_parquet(
    staging_path: &Path,
    source: &SourceLoad,
    embeddings: &[LadybugEmbedding],
) -> Result<(), IngestError> {
    let retrieval_visibility = RetrievalVisibility::from_remaster_links(&source.remaster_links);
    let records = source
        .records
        .iter()
        .map(|loaded| &loaded.record)
        .collect::<Vec<_>>();

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
        let record_aliases = source
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
    write_alias_parquet(staging_path, source)?;
    write_metric_parquet(staging_path, &records)?;
    write_variant_group_parquet(staging_path, &records)?;
    Ok(())
}

fn write_embedding_unit_parquet(
    staging_path: &Path,
    embeddings: &[LadybugEmbedding],
) -> Result<(), IngestError> {
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
) -> Result<(), IngestError> {
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
                    .map_err(|error| IngestError::ArtifactWriteFailed(error.to_string()))?,
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
) -> Result<(), IngestError> {
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
) -> Result<(), IngestError> {
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
) -> Result<(), IngestError> {
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
) -> Result<(), IngestError> {
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

fn write_alias_parquet(staging_path: &Path, source: &SourceLoad) -> Result<(), IngestError> {
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
            arrow_strings(source.aliases.iter().map(alias_key)),
            arrow_strings(source.aliases.iter().map(|alias| alias.alias_text.clone())),
            arrow_strings(
                source
                    .aliases
                    .iter()
                    .map(|alias| alias.normalized_alias.clone()),
            ),
            arrow_strings(
                source
                    .aliases
                    .iter()
                    .map(|alias| alias.source.as_str().to_string()),
            ),
            arrow_strings(source.aliases.iter().map(|alias| alias.source_ref.clone())),
        ],
    )
}

fn write_metric_parquet(
    staging_path: &Path,
    records: &[&NormalizedRecord],
) -> Result<(), IngestError> {
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
) -> Result<(), IngestError> {
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

fn write_graph_relationship_parquet(
    staging_path: &Path,
    source: &SourceLoad,
    embeddings: &[LadybugEmbedding],
) -> Result<(), IngestError> {
    let retrieval_visibility = RetrievalVisibility::from_remaster_links(&source.remaster_links);
    let records = source
        .records
        .iter()
        .map(|loaded| &loaded.record)
        .collect::<Vec<_>>();

    write_parquet(
        &staging_path.join("from_pack.parquet"),
        vec![
            arrow_field("from", DataType::Utf8, false),
            arrow_field("to", DataType::Utf8, false),
        ],
        vec![
            arrow_strings(records.iter().map(|record| record.key.to_string())),
            arrow_strings(records.iter().map(|record| record.pack_name.to_string())),
        ],
    )?;

    let publication_records = records
        .iter()
        .filter(|record| record.publication_title.is_some())
        .copied()
        .collect::<Vec<_>>();
    write_parquet(
        &staging_path.join("published_in.parquet"),
        vec![
            arrow_field("from", DataType::Utf8, false),
            arrow_field("to", DataType::Utf8, false),
        ],
        vec![
            arrow_strings(
                publication_records
                    .iter()
                    .map(|record| record.key.to_string()),
            ),
            arrow_strings(publication_records.iter().map(|record| {
                publication_key(
                    record
                        .publication_title
                        .as_ref()
                        .expect("publication_records only includes publication titles"),
                )
            })),
        ],
    )?;

    let visible_records = records
        .iter()
        .filter(|record| retrieval_visibility.is_default_visible(record))
        .copied()
        .collect::<Vec<_>>();
    write_parquet(
        &staging_path.join("has_search_document.parquet"),
        vec![
            arrow_field("from", DataType::Utf8, false),
            arrow_field("to", DataType::Utf8, false),
        ],
        vec![
            arrow_strings(visible_records.iter().map(|record| record.key.to_string())),
            arrow_strings(
                visible_records
                    .iter()
                    .map(|record| format!("{}#fts", record.key)),
            ),
        ],
    )?;

    let mut content_from = Vec::new();
    let mut content_to = Vec::new();
    for record in &records {
        for ordinal in 0..record.supplemental_content.len() {
            content_from.push(record.key.to_string());
            content_to.push(format!("{}#content#{ordinal}", record.key));
        }
    }
    write_parquet(
        &staging_path.join("has_content_unit.parquet"),
        vec![
            arrow_field("from", DataType::Utf8, false),
            arrow_field("to", DataType::Utf8, false),
        ],
        vec![arrow_strings(content_from), arrow_strings(content_to)],
    )?;

    let evidence_units = evidence_units(&records);
    write_parquet(
        &staging_path.join("has_evidence_unit.parquet"),
        vec![
            arrow_field("from", DataType::Utf8, false),
            arrow_field("to", DataType::Utf8, false),
        ],
        vec![
            arrow_strings(evidence_units.iter().map(|unit| unit.record_key.clone())),
            arrow_strings(
                evidence_units
                    .iter()
                    .map(|unit| unit.evidence_unit_key.clone()),
            ),
        ],
    )?;

    let mut content_reference_from = Vec::new();
    let mut content_reference_to = Vec::new();
    let mut content_reference_display_texts = Vec::new();
    let mut content_reference_texts = Vec::new();
    let mut content_reference_source_kinds = Vec::new();
    let mut content_reference_visibilities = Vec::new();
    let mut content_reference_seen = BTreeSet::new();
    for record in &records {
        for (ordinal, supplemental) in record.supplemental_content.iter().enumerate() {
            if !supplemental.contributes_to_references {
                continue;
            }
            let content_unit_key = format!("{}#content#{ordinal}", record.key);
            for reference in iter_content_references(&supplemental.document) {
                let Some(to_record_key) = &reference.resolved_key else {
                    continue;
                };
                let reference_text = content_reference_text(reference);
                let dedupe_key = (
                    content_unit_key.clone(),
                    to_record_key.to_string(),
                    reference_text.clone(),
                    supplemental.source_kind.as_str().to_string(),
                );
                if content_reference_seen.insert(dedupe_key) {
                    content_reference_from.push(content_unit_key.clone());
                    content_reference_to.push(to_record_key.to_string());
                    content_reference_display_texts.push(content_reference_display_text(reference));
                    content_reference_texts.push(reference_text);
                    content_reference_source_kinds
                        .push(supplemental.source_kind.as_str().to_string());
                    content_reference_visibilities
                        .push(supplemental.visibility.as_str().to_string());
                }
            }
        }
    }
    write_parquet(
        &staging_path.join("content_references.parquet"),
        vec![
            arrow_field("from", DataType::Utf8, false),
            arrow_field("to", DataType::Utf8, false),
            arrow_field("display_text", DataType::Utf8, true),
            arrow_field("reference_text", DataType::Utf8, false),
            arrow_field("source_kind", DataType::Utf8, false),
            arrow_field("visibility", DataType::Utf8, false),
        ],
        vec![
            arrow_strings(content_reference_from),
            arrow_strings(content_reference_to),
            arrow_optional_strings(content_reference_display_texts),
            arrow_strings(content_reference_texts),
            arrow_strings(content_reference_source_kinds),
            arrow_strings(content_reference_visibilities),
        ],
    )?;

    let mut evidence_reference_from = Vec::new();
    let mut evidence_reference_to = Vec::new();
    let mut evidence_reference_display_texts = Vec::new();
    let mut evidence_reference_texts = Vec::new();
    let mut evidence_reference_source_kinds = Vec::new();
    let mut evidence_reference_visibilities = Vec::new();
    let mut evidence_reference_seen = BTreeSet::new();
    for unit in &evidence_units {
        for reference in iter_content_references(&unit.document) {
            let Some(to_record_key) = &reference.resolved_key else {
                continue;
            };
            let reference_text = content_reference_text(reference);
            let dedupe_key = (
                unit.evidence_unit_key.clone(),
                to_record_key.to_string(),
                reference_text.clone(),
                unit.source_kind.as_str().to_string(),
            );
            if evidence_reference_seen.insert(dedupe_key) {
                evidence_reference_from.push(unit.evidence_unit_key.clone());
                evidence_reference_to.push(to_record_key.to_string());
                evidence_reference_display_texts.push(content_reference_display_text(reference));
                evidence_reference_texts.push(reference_text);
                evidence_reference_source_kinds.push(unit.source_kind.as_str().to_string());
                evidence_reference_visibilities.push(unit.visibility.as_str().to_string());
            }
        }
    }
    write_parquet(
        &staging_path.join("evidence_references.parquet"),
        vec![
            arrow_field("from", DataType::Utf8, false),
            arrow_field("to", DataType::Utf8, false),
            arrow_field("display_text", DataType::Utf8, true),
            arrow_field("reference_text", DataType::Utf8, false),
            arrow_field("source_kind", DataType::Utf8, false),
            arrow_field("visibility", DataType::Utf8, false),
        ],
        vec![
            arrow_strings(evidence_reference_from),
            arrow_strings(evidence_reference_to),
            arrow_optional_strings(evidence_reference_display_texts),
            arrow_strings(evidence_reference_texts),
            arrow_strings(evidence_reference_source_kinds),
            arrow_strings(evidence_reference_visibilities),
        ],
    )?;

    let embedding_unit_keys = embeddings
        .iter()
        .map(|embedding| embedding.embedding_unit_key.as_str())
        .collect::<BTreeSet<_>>();
    let evidence_embedding_units = evidence_units
        .iter()
        .filter(|unit| embedding_unit_keys.contains(unit.evidence_unit_key.as_str()))
        .collect::<Vec<_>>();
    write_parquet(
        &staging_path.join("has_evidence_embedding.parquet"),
        vec![
            arrow_field("from", DataType::Utf8, false),
            arrow_field("to", DataType::Utf8, false),
        ],
        vec![
            arrow_strings(
                evidence_embedding_units
                    .iter()
                    .map(|unit| unit.evidence_unit_key.clone()),
            ),
            arrow_strings(
                evidence_embedding_units
                    .iter()
                    .map(|unit| unit.evidence_unit_key.clone()),
            ),
        ],
    )?;

    let mut trait_from = Vec::new();
    let mut trait_to = Vec::new();
    for record in &records {
        for trait_name in &record.traits {
            trait_from.push(record.key.to_string());
            trait_to.push(trait_name.clone());
        }
    }
    write_parquet(
        &staging_path.join("has_trait.parquet"),
        vec![
            arrow_field("from", DataType::Utf8, false),
            arrow_field("to", DataType::Utf8, false),
        ],
        vec![arrow_strings(trait_from), arrow_strings(trait_to)],
    )?;

    let mut filter_value_from = Vec::new();
    let mut filter_value_to = Vec::new();
    for record in &records {
        for (field, value) in record_filter_values(record) {
            filter_value_from.push(record.key.to_string());
            filter_value_to.push(filter_value_key(field, &value));
        }
    }
    write_parquet(
        &staging_path.join("has_filter_value.parquet"),
        vec![
            arrow_field("from", DataType::Utf8, false),
            arrow_field("to", DataType::Utf8, false),
        ],
        vec![
            arrow_strings(filter_value_from),
            arrow_strings(filter_value_to),
        ],
    )?;

    write_parquet(
        &staging_path.join("has_alias.parquet"),
        vec![
            arrow_field("from", DataType::Utf8, false),
            arrow_field("to", DataType::Utf8, false),
        ],
        vec![
            arrow_strings(
                source
                    .aliases
                    .iter()
                    .map(|alias| alias.canonical_record_key.to_string()),
            ),
            arrow_strings(source.aliases.iter().map(alias_key)),
        ],
    )?;

    let mut metric_from = Vec::new();
    let mut metric_to = Vec::new();
    let mut metric_number_values = Vec::new();
    let mut metric_text_values = Vec::new();
    let mut metric_bool_values = Vec::new();
    for record in &records {
        for metric in &record.metrics {
            metric_from.push(record.key.to_string());
            metric_to.push(metric_key_id(metric.domain.as_str(), &metric.key));
            match &metric.value {
                MetricValue::Number(value) => {
                    metric_number_values.push(Some(*value));
                    metric_text_values.push(None);
                    metric_bool_values.push(None);
                }
                MetricValue::Text(value) => {
                    metric_number_values.push(None);
                    metric_text_values.push(Some(value.clone()));
                    metric_bool_values.push(None);
                }
                MetricValue::Boolean(value) => {
                    metric_number_values.push(None);
                    metric_text_values.push(None);
                    metric_bool_values.push(Some(*value));
                }
            }
        }
    }
    write_parquet(
        &staging_path.join("has_metric.parquet"),
        vec![
            arrow_field("from", DataType::Utf8, false),
            arrow_field("to", DataType::Utf8, false),
            arrow_field("number_value", DataType::Float64, true),
            arrow_field("text_value", DataType::Utf8, true),
            arrow_field("bool_value", DataType::Boolean, true),
        ],
        vec![
            arrow_strings(metric_from),
            arrow_strings(metric_to),
            arrow_optional_floats(metric_number_values),
            arrow_optional_strings(metric_text_values),
            arrow_optional_bools(metric_bool_values),
        ],
    )?;

    write_parquet(
        &staging_path.join("has_embedding_unit.parquet"),
        vec![
            arrow_field("from", DataType::Utf8, false),
            arrow_field("to", DataType::Utf8, false),
        ],
        vec![
            arrow_strings(
                embeddings
                    .iter()
                    .map(|embedding| embedding.record_key.clone()),
            ),
            arrow_strings(
                embeddings
                    .iter()
                    .map(|embedding| embedding.embedding_unit_key.clone()),
            ),
        ],
    )?;

    write_parquet(
        &staging_path.join("references.parquet"),
        vec![
            arrow_field("from", DataType::Utf8, false),
            arrow_field("to", DataType::Utf8, false),
            arrow_field("display_text", DataType::Utf8, true),
            arrow_field("reference_text", DataType::Utf8, false),
            arrow_field("source_kind", DataType::Utf8, false),
            arrow_field("visibility", DataType::Utf8, false),
        ],
        vec![
            arrow_strings(
                source
                    .references
                    .iter()
                    .map(|reference| reference.from_record_key.to_string()),
            ),
            arrow_strings(
                source
                    .references
                    .iter()
                    .map(|reference| reference.to_record_key.to_string()),
            ),
            arrow_optional_strings(
                source
                    .references
                    .iter()
                    .map(|reference| reference.display_text.clone()),
            ),
            arrow_strings(
                source
                    .references
                    .iter()
                    .map(|reference| reference.reference_text.clone()),
            ),
            arrow_strings(
                source
                    .references
                    .iter()
                    .map(|reference| reference.source_kind.as_str().to_string()),
            ),
            arrow_strings(
                source
                    .references
                    .iter()
                    .map(|reference| reference.visibility.as_str().to_string()),
            ),
        ],
    )?;

    write_parquet(
        &staging_path.join("remastered_by.parquet"),
        vec![
            arrow_field("from", DataType::Utf8, false),
            arrow_field("to", DataType::Utf8, false),
            arrow_field("source_kind", DataType::Utf8, false),
            arrow_field("source_ref", DataType::Utf8, false),
        ],
        vec![
            arrow_strings(
                source
                    .remaster_links
                    .iter()
                    .map(|link| link.legacy_record_key.to_string()),
            ),
            arrow_strings(
                source
                    .remaster_links
                    .iter()
                    .map(|link| link.remaster_record_key.to_string()),
            ),
            arrow_strings(
                source
                    .remaster_links
                    .iter()
                    .map(|link| link.source.as_str().to_string()),
            ),
            arrow_strings(
                source
                    .remaster_links
                    .iter()
                    .map(|link| link.source_ref.clone()),
            ),
        ],
    )?;

    let mut variant_from = Vec::new();
    let mut variant_to = Vec::new();
    let mut variant_labels = Vec::new();
    let mut variant_axes_jsons = Vec::new();
    for record in &records {
        if let Some(key) = &record.variant_group_key {
            variant_from.push(record.key.to_string());
            variant_to.push(key.clone());
            variant_labels.push(record.variant_label.clone());
            variant_axes_jsons.push(
                serde_json::to_string(&record.variant_axes)
                    .map_err(|error| IngestError::ArtifactWriteFailed(error.to_string()))?,
            );
        }
    }
    write_parquet(
        &staging_path.join("in_variant_group.parquet"),
        vec![
            arrow_field("from", DataType::Utf8, false),
            arrow_field("to", DataType::Utf8, false),
            arrow_field("variant_label", DataType::Utf8, true),
            arrow_field("variant_axes_json", DataType::Utf8, false),
        ],
        vec![
            arrow_strings(variant_from),
            arrow_strings(variant_to),
            arrow_optional_strings(variant_labels),
            arrow_strings(variant_axes_jsons),
        ],
    )?;

    Ok(())
}

fn json_string_array(values: &[String]) -> String {
    serde_json::to_string(values).unwrap_or_else(|_| "[]".to_string())
}

fn record_filter_values(record: &NormalizedRecord) -> BTreeSet<(&'static str, String)> {
    let mut values = BTreeSet::new();
    values.extend(
        record
            .taxonomy_families
            .iter()
            .cloned()
            .map(|value| ("taxonomy_families", value)),
    );
    values.extend(
        record
            .variant_axes
            .iter()
            .cloned()
            .map(|value| ("variant_axes", value)),
    );
    if let Some(actor) = &record.actor_data {
        values.extend(
            actor
                .languages
                .iter()
                .cloned()
                .map(|value| ("languages", value)),
        );
        values.extend(
            actor
                .speed_types
                .iter()
                .cloned()
                .map(|value| ("speed_types", value)),
        );
        values.extend(actor.senses.iter().cloned().map(|value| ("senses", value)));
        values.extend(
            actor
                .immunities
                .iter()
                .cloned()
                .map(|value| ("immunities", value)),
        );
        values.extend(
            actor
                .resistances
                .iter()
                .cloned()
                .map(|value| ("resistances", value)),
        );
        values.extend(
            actor
                .weaknesses
                .iter()
                .cloned()
                .map(|value| ("weaknesses", value)),
        );
        values.extend(
            actor
                .disable_skills
                .iter()
                .cloned()
                .map(|value| ("disable_skills", value)),
        );
    }
    if let Some(item) = &record.item_data {
        values.extend(
            item.damage_types
                .iter()
                .cloned()
                .map(|value| ("damage_types", value)),
        );
    }
    if let Some(spell) = &record.spell_data {
        values.extend(
            spell
                .traditions
                .iter()
                .cloned()
                .map(|value| ("traditions", value)),
        );
        values.extend(
            spell
                .spell_kinds
                .iter()
                .cloned()
                .map(|value| ("spell_kinds", value)),
        );
        values.extend(
            spell
                .damage_types
                .iter()
                .cloned()
                .map(|value| ("damage_types", value)),
        );
    }
    values
}

fn filter_value_key(field: &str, value: &str) -> String {
    format!("{field}:{value}")
}

fn optional_content_json(document: &Option<ContentDocument>) -> Option<String> {
    document
        .as_ref()
        .and_then(|document| serde_json::to_string(document).ok())
}

fn ladybug_embeddings(source: &SourceLoad) -> Result<Vec<LadybugEmbedding>, IngestError> {
    if !source.document_embeddings.is_empty() {
        return Ok(source
            .document_embeddings
            .iter()
            .map(|embedding| LadybugEmbedding {
                embedding_unit_key: embedding.embedding_unit_key.clone(),
                record_key: embedding.record_key.clone(),
                unit_kind: embedding.unit_kind.as_str().to_string(),
                label: embedding.label.clone(),
                ordinal: embedding.ordinal as i64,
                semantic_input_hash: embedding.input_hash.clone(),
                dimensions: embedding.dimensions,
                vector: embedding.vector.clone(),
            })
            .collect());
    }

    let Some(path) = std::env::var_os("ATLAS_LADYBUG_LEGACY_EMBEDDINGS_SQLITE") else {
        return Ok(Vec::new());
    };
    load_legacy_sqlite_embeddings(Path::new(&path), source)
}

fn load_legacy_sqlite_embeddings(
    path: &Path,
    source: &SourceLoad,
) -> Result<Vec<LadybugEmbedding>, IngestError> {
    let source_record_keys = source
        .records
        .iter()
        .map(|loaded| loaded.record.key.to_string())
        .collect::<BTreeSet<_>>();
    let connection = rusqlite::Connection::open_with_flags(path, OpenFlags::SQLITE_OPEN_READ_ONLY)
        .map_err(|error| IngestError::ArtifactWriteFailed(error.to_string()))?;
    let mut select = connection
        .prepare(
            "SELECT record_key, dimensions, semantic_input_hash, vector_blob
             FROM embeddings
             ORDER BY record_key",
        )
        .map_err(|error| IngestError::ArtifactWriteFailed(error.to_string()))?;
    let rows = select
        .query_map([], |row| {
            Ok((
                row.get::<_, String>(0)?,
                row.get::<_, i64>(1)?,
                row.get::<_, String>(2)?,
                row.get::<_, Vec<u8>>(3)?,
            ))
        })
        .map_err(|error| IngestError::ArtifactWriteFailed(error.to_string()))?;
    let mut embeddings = Vec::new();
    for row in rows {
        let (record_key, dimensions, semantic_input_hash, vector_blob) =
            row.map_err(|error| IngestError::ArtifactWriteFailed(error.to_string()))?;
        if !source_record_keys.contains(&record_key) {
            continue;
        }
        let dimensions = usize::try_from(dimensions).map_err(|_| {
            IngestError::ArtifactWriteFailed(format!(
                "legacy embedding `{record_key}` has invalid dimensions"
            ))
        })?;
        let vector = decode_f32_vector_blob(&vector_blob)
            .map_err(|error| IngestError::ArtifactWriteFailed(error.to_string()))?;
        if vector.len() != dimensions {
            return Err(IngestError::ArtifactWriteFailed(format!(
                "legacy embedding `{record_key}` vector has {} dimensions; expected {dimensions}",
                vector.len()
            )));
        }
        embeddings.push(LadybugEmbedding {
            embedding_unit_key: format!("{record_key}#legacy-parent"),
            record_key,
            unit_kind: "legacy_parent".to_string(),
            label: None,
            ordinal: 0,
            semantic_input_hash,
            dimensions,
            vector,
        });
    }
    info!(
        legacy_sqlite_embeddings = embeddings.len(),
        source = %path.display(),
        "loaded legacy SQLite embeddings for Ladybug spike"
    );
    Ok(embeddings)
}

fn metric_value_type(value: &MetricValue) -> &'static str {
    match value {
        MetricValue::Number(_) => "number",
        MetricValue::Text(_) => "text",
        MetricValue::Boolean(_) => "boolean",
    }
}

fn alias_key(alias: &atlas_record::RecordAlias) -> String {
    format!(
        "{}#alias#{}#{}",
        alias.canonical_record_key, alias.normalized_alias, alias.source_ref
    )
}

fn metric_key_id(domain: &str, key: &str) -> String {
    format!("{domain}:{key}")
}

fn namespace_prefix(metric_key: &str) -> String {
    metric_key
        .rsplit_once('.')
        .map(|(prefix, _)| format!("{prefix}."))
        .unwrap_or_default()
}

fn content_reference_text(reference: &ContentReference) -> String {
    match &reference.locator {
        ContentReferenceLocator::FoundryUuid { raw_target }
        | ContentReferenceLocator::Compendium { raw_target } => raw_target.clone(),
        ContentReferenceLocator::PackAndLocator { pack_name, locator } => {
            format!("{pack_name}:{locator}")
        }
        ContentReferenceLocator::Unknown { raw } => raw.clone(),
    }
}

fn content_reference_display_text(reference: &ContentReference) -> Option<String> {
    reference
        .label
        .as_ref()
        .map(|label| {
            render_plain_text(&ContentDocument::new(vec![ContentBlock::Paragraph {
                content: label.clone(),
            }]))
        })
        .filter(|label| !label.trim().is_empty())
}

fn publication_key(title: &str) -> String {
    title.trim().to_lowercase()
}
