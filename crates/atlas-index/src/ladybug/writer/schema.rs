use lbug::Connection;

use crate::IndexWriteError;
use crate::ladybug::writer::output::{
    ladybug_progress_message, ladybug_search_index_progress, ladybug_write_error,
};

pub(crate) fn create_schema(
    connection: &Connection<'_>,
    dimensions: usize,
) -> Result<(), IndexWriteError> {
    for statement in [
        "CREATE NODE TABLE Record(record_key STRING, id STRING, name STRING, normalized_name STRING, record_family STRING, foundry_document_type STRING, foundry_record_type STRING, level INT64, rarity STRING, traits_json STRING, prerequisites_json STRING, system_category STRING, system_group STRING, system_base_item STRING, system_usage STRING, system_price_json STRING, system_actions_value INT64, system_time_value STRING, system_duration_value STRING, price_cp INT64, activation_time_kind STRING, activation_time_actions INT64, activation_time_duration_value INT64, activation_time_duration_unit STRING, activation_time_text STRING, duration_kind STRING, duration_value INT64, duration_unit STRING, duration_text STRING, publication_title STRING, publication_family STRING, publication_remaster BOOL, description_json STRING, blurb_json STRING, folder_id STRING, taxonomy_families_json STRING, variant_group_key STRING, variant_base_name STRING, variant_label STRING, variant_axes_json STRING, variant_confidence DOUBLE, variant_source STRING, is_default_visible BOOL, source_path STRING, raw_json STRING, actor_size STRING, actor_languages_json STRING, actor_speed_types_json STRING, actor_senses_json STRING, actor_immunities_json STRING, actor_resistances_json STRING, actor_weaknesses_json STRING, actor_disable_text STRING, actor_disable_skills_json STRING, actor_is_complex BOOL, item_bulk_value DOUBLE, item_hands_requirement STRING, item_damage_types_json STRING, spell_traditions_json STRING, spell_kinds_json STRING, spell_range_text STRING, spell_range_value DOUBLE, spell_target_text STRING, spell_area_type STRING, spell_area_value DOUBLE, spell_save_type STRING, spell_sustained BOOL, spell_basic_save BOOL, spell_damage_types_json STRING, PRIMARY KEY(record_key));".to_string(),
        "CREATE NODE TABLE ArtifactMetadata(key STRING, value STRING, PRIMARY KEY(key));".to_string(),
        "CREATE NODE TABLE SearchDocument(search_doc_key STRING, record_key STRING, title STRING, aliases STRING, traits STRING, precision_terms STRING, taxonomy_terms STRING, constraint_terms STRING, mechanic_terms STRING, source_terms STRING, metric_terms STRING, headings STRING, body STRING, facts STRING, reference_terms STRING, embedded_content STRING, PRIMARY KEY(search_doc_key));".to_string(),
        format!("CREATE NODE TABLE EmbeddingUnit(embedding_unit_key STRING, record_key STRING, unit_kind STRING, label STRING, ordinal INT64, semantic_input_hash STRING, dimensions INT64, embedding FLOAT[{dimensions}], PRIMARY KEY(embedding_unit_key));"),
        "CREATE NODE TABLE ContentUnit(content_unit_key STRING, record_key STRING, ordinal INT64, source_kind STRING, visibility STRING, contributes_to_search BOOL, contributes_to_references BOOL, label STRING, content_json STRING, PRIMARY KEY(content_unit_key));".to_string(),
        "CREATE NODE TABLE EvidenceUnit(evidence_unit_key STRING, record_key STRING, source_content_unit_key STRING, source_kind STRING, visibility STRING, unit_kind STRING, label STRING, ordinal INT64, search_text STRING, PRIMARY KEY(evidence_unit_key));".to_string(),
        "CREATE NODE TABLE Trait(name STRING, PRIMARY KEY(name));".to_string(),
        "CREATE NODE TABLE FilterValue(filter_value_key STRING, field STRING, value STRING, PRIMARY KEY(filter_value_key));".to_string(),
        "CREATE NODE TABLE Publication(publication_key STRING, title STRING, family STRING, remaster BOOL, PRIMARY KEY(publication_key));".to_string(),
        "CREATE NODE TABLE Pack(pack_name STRING, pack_label STRING, document_type STRING, source_path STRING, PRIMARY KEY(pack_name));".to_string(),
        "CREATE NODE TABLE Alias(alias_key STRING, alias_text STRING, normalized_alias STRING, source_kind STRING, source_ref STRING, PRIMARY KEY(alias_key));".to_string(),
        "CREATE NODE TABLE Metric(metric_key_id STRING, metric_domain STRING, metric_key STRING, value_type STRING, namespace_prefix STRING, label STRING, short_label STRING, group_name STRING, known BOOL, PRIMARY KEY(metric_key_id));".to_string(),
        "CREATE NODE TABLE VariantGroup(variant_group_key STRING, base_name STRING, source STRING, confidence DOUBLE, PRIMARY KEY(variant_group_key));".to_string(),
        "CREATE NODE TABLE Concept(concept_key STRING, name STRING, kind STRING, source STRING, confidence DOUBLE, derived BOOL, PRIMARY KEY(concept_key));".to_string(),
        "CREATE REL TABLE HAS_SEARCH_DOCUMENT(FROM Record TO SearchDocument);".to_string(),
        "CREATE REL TABLE HAS_EMBEDDING_UNIT(FROM Record TO EmbeddingUnit);".to_string(),
        "CREATE REL TABLE HAS_CONTENT_UNIT(FROM Record TO ContentUnit);".to_string(),
        "CREATE REL TABLE HAS_EVIDENCE_UNIT(FROM Record TO EvidenceUnit);".to_string(),
        "CREATE REL TABLE CONTENT_REFERENCES(FROM ContentUnit TO Record, display_text STRING, reference_text STRING, source_kind STRING, visibility STRING);".to_string(),
        "CREATE REL TABLE EVIDENCE_REFERENCES(FROM EvidenceUnit TO Record, display_text STRING, reference_text STRING, source_kind STRING, visibility STRING);".to_string(),
        "CREATE REL TABLE HAS_EVIDENCE_EMBEDDING(FROM EvidenceUnit TO EmbeddingUnit);".to_string(),
        "CREATE REL TABLE HAS_TRAIT(FROM Record TO Trait);".to_string(),
        "CREATE REL TABLE HAS_FILTER_VALUE(FROM Record TO FilterValue);".to_string(),
        "CREATE REL TABLE PUBLISHED_IN(FROM Record TO Publication);".to_string(),
        "CREATE REL TABLE FROM_PACK(FROM Record TO Pack);".to_string(),
        "CREATE REL TABLE HAS_ALIAS(FROM Record TO Alias);".to_string(),
        "CREATE REL TABLE HAS_METRIC(FROM Record TO Metric, number_value DOUBLE, text_value STRING, bool_value BOOL);".to_string(),
        "CREATE REL TABLE REFERENCES(FROM Record TO Record, display_text STRING, reference_text STRING, source_kind STRING, visibility STRING);".to_string(),
        "CREATE REL TABLE REMASTERED_BY(FROM Record TO Record, source_kind STRING, source_ref STRING);".to_string(),
        "CREATE REL TABLE IN_VARIANT_GROUP(FROM Record TO VariantGroup, variant_label STRING, variant_axes_json STRING);".to_string(),
        "CREATE REL TABLE APPLIES_CONCEPT(FROM Record TO Concept);".to_string(),
        "CREATE REL TABLE MENTIONS_CONCEPT(FROM Record TO Concept);".to_string(),
    ] {
        connection.query(&statement).map_err(ladybug_write_error)?;
    }
    Ok(())
}

pub(crate) fn create_search_indexes(
    connection: &Connection<'_>,
    create_vector_index: bool,
) -> Result<(), IndexWriteError> {
    let total_steps = if create_vector_index { 6 } else { 4 };
    let mut step = 1;
    ladybug_search_index_progress(step, total_steps, "Loading LadybugDB FTS extension");
    load_extension(connection, "FTS")?;
    step += 1;
    if create_vector_index {
        ladybug_search_index_progress(step, total_steps, "Loading LadybugDB VECTOR extension");
        load_extension(connection, "VECTOR")?;
        step += 1;
    }
    ladybug_search_index_progress(
        step,
        total_steps,
        "Creating LadybugDB SearchDocument title/alias FTS index",
    );
    connection
        .query(
            "CALL CREATE_FTS_INDEX(
                'SearchDocument',
                'search_document_title_alias_fts',
                ['title', 'aliases'],
                stemmer := 'porter'
            );",
        )
        .map_err(ladybug_write_error)?;
    step += 1;
    ladybug_search_index_progress(
        step,
        total_steps,
        "Creating LadybugDB SearchDocument facet FTS index",
    );
    connection
        .query(
            "CALL CREATE_FTS_INDEX(
                'SearchDocument',
                'search_document_facet_fts',
                ['traits', 'precision_terms'],
                stemmer := 'porter'
            );",
        )
        .map_err(ladybug_write_error)?;
    step += 1;
    ladybug_search_index_progress(
        step,
        total_steps,
        "Creating LadybugDB EvidenceUnit FTS index",
    );
    connection
        .query(
            "CALL CREATE_FTS_INDEX(
                'EvidenceUnit',
                'evidence_unit_fts',
                ['label', 'search_text'],
                stemmer := 'porter'
            );",
        )
        .map_err(ladybug_write_error)?;
    if create_vector_index {
        step += 1;
        ladybug_search_index_progress(
            step,
            total_steps,
            "Creating LadybugDB EmbeddingUnit vector index",
        );
        connection
            .query("CALL CREATE_VECTOR_INDEX('EmbeddingUnit', 'embedding_hnsw', 'embedding', metric := 'cosine');")
            .map_err(ladybug_write_error)?;
    }
    ladybug_progress_message("ladybug_search_indexes", "Created LadybugDB search indexes");
    Ok(())
}

fn load_extension(connection: &Connection<'_>, extension: &str) -> Result<(), IndexWriteError> {
    let load_query = format!("LOAD EXTENSION {extension};");
    if connection.query(&load_query).is_ok() {
        return Ok(());
    }
    connection
        .query(&format!("INSTALL {extension};"))
        .map_err(ladybug_write_error)?;
    connection.query(&load_query).map_err(ladybug_write_error)?;
    Ok(())
}
