use std::collections::BTreeMap;
use std::path::Path;
use std::time::Instant;

use crate::{
    DiscoveryError, FilterValueRequest, FilteredRecordKeyPage, FilteredRecordSort,
    FtsColumnWeights, FtsQuery, FtsSearchHit, GraphProductIndex, GraphReferenceEdge,
    RecordLoadOptions, RecordResolutionMatchKind, RecordResolutionResult, ReferenceEdgeDirection,
    RemasterLinks, SearchCandidateRecord, SearchError, SearchIndex, VariantGroup, VectorSearchHit,
};
use atlas_domain::{
    FilterFieldDiscovery, FilterValueDiscovery, MetricDomain, MetricValueType, RecordKey,
    SearchFilterNode,
};
use atlas_record::{
    ContentDocument, ContentSourceKind, ContentVisibility, MetricRow, MetricValue, PersistedRecord,
    PersistedRecordSet, SupplementalContentDocument,
};
use lbug::{Connection, Database, SystemConfig};
use thiserror::Error;

mod discovery;
mod filter;
mod graph;
mod row;
mod search;
use self::filter::compile_scope;
use self::row::{
    alias_from_row, bool_at, float_at, optional_string_at, query_rows, query_rows_traced,
    record_from_row, record_key_at, string_at,
};

pub struct LadybugIndexReader {
    connection: Connection<'static>,
}

#[derive(Debug, Error)]
pub enum LadybugIndexReaderError {
    #[error("LadybugDB query failed: {0}")]
    Query(String),
    #[error("LadybugDB row had invalid data: {0}")]
    InvalidData(String),
    #[error("LadybugDB search shape is not supported yet: {0}")]
    Unsupported(String),
}

impl LadybugIndexReader {
    pub fn open(path: impl AsRef<Path>) -> Result<Self, LadybugIndexReaderError> {
        let database = Box::leak(Box::new(
            Database::new(path.as_ref(), SystemConfig::default())
                .map_err(|error| LadybugIndexReaderError::Query(error.to_string()))?,
        ));
        let connection = Connection::new(database)
            .map_err(|error| LadybugIndexReaderError::Query(error.to_string()))?;
        let index = Self { connection };
        index.prepare_search_indexes();
        Ok(index)
    }

    fn prepare_search_indexes(&self) {
        load_extension(&self.connection, "FTS");
        load_extension(&self.connection, "VECTOR");
    }

    pub(crate) fn connection(&self) -> &Connection<'static> {
        &self.connection
    }
}

fn load_extension(connection: &Connection<'_>, extension: &str) {
    let load_query = format!("LOAD EXTENSION {extension};");
    if connection.query(&load_query).is_ok() {
        return;
    }
    let install_query = format!("INSTALL {extension};");
    if connection.query(&install_query).is_ok() {
        let _ = connection.query(&load_query);
    }
}

struct RecordKeyMatchQuery<'a> {
    query: &'a str,
    normalized_query: &'a str,
    match_kind: RecordResolutionMatchKind,
    field: &'a str,
    value: &'a str,
    require_no_variant_label: bool,
}

impl SearchIndex for LadybugIndexReader {
    fn load_records_by_key(&self, keys: &[RecordKey]) -> Result<Vec<PersistedRecord>, SearchError> {
        self.load_records_by_key_impl(keys).map_err(search_error)
    }

    fn load_records_by_key_with_options(
        &self,
        keys: &[RecordKey],
        options: RecordLoadOptions,
    ) -> Result<Vec<PersistedRecord>, SearchError> {
        self.load_records_by_key_with_options_impl(keys, options)
            .map_err(search_error)
    }

    fn load_search_candidate_records(
        &self,
        keys: &[RecordKey],
    ) -> Result<Vec<SearchCandidateRecord>, SearchError> {
        self.load_search_candidate_records_impl(keys)
            .map_err(search_error)
    }

    fn load_record_set(&self) -> Result<PersistedRecordSet, SearchError> {
        self.load_record_set_impl().map_err(search_error)
    }

    fn resolve_record_matches(
        &self,
        query: &str,
        normalized_query: &str,
        filter: Option<&SearchFilterNode>,
    ) -> Result<Option<Vec<RecordResolutionResult>>, SearchError> {
        self.resolve_record_matches_with_options(
            query,
            normalized_query,
            filter,
            RecordLoadOptions::include_raw_json(),
        )
    }

    fn resolve_record_matches_with_options(
        &self,
        query: &str,
        normalized_query: &str,
        filter: Option<&SearchFilterNode>,
        options: RecordLoadOptions,
    ) -> Result<Option<Vec<RecordResolutionResult>>, SearchError> {
        self.resolve_record_matches_with_options_impl(query, normalized_query, filter, options)
            .map(Some)
            .map_err(search_error)
    }

    fn resolve_metric_filters(
        &self,
        filter: Option<&SearchFilterNode>,
    ) -> Result<Option<SearchFilterNode>, SearchError> {
        Ok(filter.cloned())
    }

    fn list_filtered_record_keys(
        &self,
        filter: Option<&SearchFilterNode>,
        sort: FilteredRecordSort,
        limit: u32,
        offset: u32,
    ) -> Result<FilteredRecordKeyPage, SearchError> {
        self.list_filtered_record_keys_impl(filter, sort, limit, offset)
            .map_err(search_error)
    }

    fn list_filter_fields(
        &self,
        filter: Option<&SearchFilterNode>,
        filter_json: Option<serde_json::Value>,
        _force_dynamic: bool,
    ) -> Result<FilterFieldDiscovery, DiscoveryError> {
        self.list_filter_fields_impl(filter, filter_json)
    }

    fn list_filter_values(
        &self,
        filter: Option<&SearchFilterNode>,
        request: FilterValueRequest,
    ) -> Result<FilterValueDiscovery, DiscoveryError> {
        self.list_filter_values_impl(filter, request)
    }

    fn query_fts_index(
        &self,
        fts_query: &FtsQuery,
        filter: Option<&SearchFilterNode>,
        limit: u32,
        _weights: FtsColumnWeights,
    ) -> Result<Vec<FtsSearchHit>, SearchError> {
        self.query_fts_index_impl(fts_query, filter, limit)
            .map_err(search_error)
    }

    fn query_fts_candidate_record_keys(
        &self,
        fts_query: &FtsQuery,
        candidate_keys: &[RecordKey],
    ) -> Result<Vec<RecordKey>, SearchError> {
        self.query_fts_candidate_record_keys_impl(fts_query, candidate_keys)
            .map_err(search_error)
    }

    fn query_vector_index(
        &self,
        query_vector: &[f32],
        filter: Option<&SearchFilterNode>,
        limit: u32,
        include_child_units: bool,
    ) -> Result<Vec<VectorSearchHit>, SearchError> {
        self.query_vector_index_impl(query_vector, filter, limit, include_child_units)
            .map_err(search_error)
    }
}

impl GraphProductIndex for LadybugIndexReader {
    fn reference_edges_for_seed(
        &self,
        seed: &RecordKey,
        direction: ReferenceEdgeDirection,
    ) -> Result<Vec<GraphReferenceEdge>, SearchError> {
        self.reference_edges_for_seed_impl(seed, direction)
            .map_err(search_error)
    }

    fn variant_group_for_record(
        &self,
        _seed: &RecordKey,
    ) -> Result<Option<VariantGroup>, SearchError> {
        Err(SearchError::UnsupportedRetrievalPattern(
            "Ladybug graph variants",
        ))
    }

    fn variant_groups_by_base_name(
        &self,
        _normalized_base_name: &str,
    ) -> Result<Vec<VariantGroup>, SearchError> {
        Err(SearchError::UnsupportedRetrievalPattern(
            "Ladybug graph variant base names",
        ))
    }

    fn remaster_links_for_record(
        &self,
        _seed: &RecordKey,
    ) -> Result<Option<RemasterLinks>, SearchError> {
        Err(SearchError::UnsupportedRetrievalPattern(
            "Ladybug graph remaster links",
        ))
    }
}

impl LadybugIndexReader {
    fn load_records_by_key_impl(
        &self,
        keys: &[RecordKey],
    ) -> Result<Vec<PersistedRecord>, LadybugIndexReaderError> {
        self.load_records_by_key_with_options_impl(keys, RecordLoadOptions::include_raw_json())
    }

    fn load_records_by_key_with_options_impl(
        &self,
        keys: &[RecordKey],
        options: RecordLoadOptions,
    ) -> Result<Vec<PersistedRecord>, LadybugIndexReaderError> {
        if keys.is_empty() {
            return Ok(Vec::new());
        }
        let total_started_at = Instant::now();
        let raw_json_projection = if options.include_raw_json {
            "record.raw_json"
        } else {
            "''"
        };
        let record_match = if keys.len() == 1 {
            format!(
                "MATCH (record:Record {{record_key: {}}})-[:FROM_PACK]->(pack:Pack)",
                string_literal(&keys[0].to_string())
            )
        } else {
            format!(
                "MATCH (record:Record)-[:FROM_PACK]->(pack:Pack)
                 WHERE record.record_key IN {}",
                list_literal(keys.iter().map(ToString::to_string))
            )
        };
        let sql = format!(
            "{record_match}
             RETURN record.record_key, record.id, record.name, record.normalized_name,
                    record.record_family, pack.pack_name, pack.pack_label,
                    record.foundry_document_type, record.foundry_record_type, record.level,
                    record.rarity, record.traits_json, record.prerequisites_json,
                    record.system_category, record.system_group, record.system_base_item,
                    record.system_usage, record.system_price_json, record.system_actions_value,
                    record.system_time_value, record.system_duration_value, record.price_cp,
                    record.activation_time_kind, record.activation_time_actions,
                    record.activation_time_duration_value, record.activation_time_duration_unit,
                    record.activation_time_text, record.duration_kind, record.duration_value,
                    record.duration_unit, record.duration_text, record.publication_title,
                    record.publication_family, record.publication_remaster, record.description_json,
                    record.blurb_json, record.folder_id, record.taxonomy_families_json,
                    record.variant_group_key, record.variant_base_name, record.variant_label,
                    record.variant_axes_json, record.variant_confidence, record.variant_source,
                    record.is_default_visible, record.source_path, {raw_json_projection},
                    record.actor_size, record.actor_languages_json, record.actor_speed_types_json,
                    record.actor_senses_json, record.actor_immunities_json,
                    record.actor_resistances_json, record.actor_weaknesses_json,
                    record.actor_disable_text, record.actor_disable_skills_json,
                    record.actor_is_complex, record.item_bulk_value,
                    record.item_hands_requirement, record.item_damage_types_json,
                    record.spell_traditions_json, record.spell_kinds_json,
                    record.spell_range_text, record.spell_range_value, record.spell_target_text,
                    record.spell_area_type, record.spell_area_value, record.spell_save_type,
                    record.spell_sustained, record.spell_basic_save, record.spell_damage_types_json
             ORDER BY record.record_key;"
        );
        let rows = query_rows_traced(&self.connection, &sql, "ladybug_load_records_query_nodes")?;
        let started_at = Instant::now();
        let mut records = rows
            .iter()
            .map(|row| record_from_row(row))
            .collect::<Result<Vec<_>, _>>()?;
        trace_ladybug_phase("ladybug_load_records_decode_nodes", started_at);
        let started_at = Instant::now();
        let metrics = self.metrics_for_keys(keys)?;
        trace_ladybug_phase("ladybug_load_records_query_metrics", started_at);
        let started_at = Instant::now();
        let supplemental_content = self.supplemental_content_for_keys(keys)?;
        trace_ladybug_phase("ladybug_load_records_query_content", started_at);
        let started_at = Instant::now();
        for record in &mut records {
            record.metrics = metrics.get(&record.key).cloned().unwrap_or_default();
            record.supplemental_content = supplemental_content
                .get(&record.key)
                .cloned()
                .unwrap_or_default();
        }
        trace_ladybug_phase("ladybug_load_records_assemble", started_at);
        trace_ladybug_phase("ladybug_load_records_total", total_started_at);
        Ok(records)
    }

    fn load_search_candidate_records_impl(
        &self,
        keys: &[RecordKey],
    ) -> Result<Vec<SearchCandidateRecord>, LadybugIndexReaderError> {
        if keys.is_empty() {
            return Ok(Vec::new());
        }
        let sql = format!(
            "MATCH (record:Record)
             WHERE record.record_key IN {}
             RETURN record.record_key, record.name, record.traits_json,
                    record.taxonomy_families_json, record.prerequisites_json,
                    record.system_category, record.system_group
             ORDER BY record.record_key;",
            list_literal(keys.iter().map(ToString::to_string))
        );
        let rows = query_rows_traced(
            &self.connection,
            &sql,
            "ladybug_load_search_candidates_query",
        )?;
        let started_at = Instant::now();
        let records = rows
            .iter()
            .map(|row| {
                Ok(SearchCandidateRecord {
                    key: record_key_at(row, 0)?,
                    name: string_at(row, 1)?,
                    traits: json_string_array("record.traits_json", &string_at(row, 2)?)?,
                    taxonomy_families: json_string_array(
                        "record.taxonomy_families_json",
                        &string_at(row, 3)?,
                    )?,
                    prerequisites: json_string_array(
                        "record.prerequisites_json",
                        &string_at(row, 4)?,
                    )?,
                    system_category: optional_string_at(row, 5)?,
                    system_group: optional_string_at(row, 6)?,
                })
            })
            .collect();
        trace_ladybug_phase("ladybug_load_search_candidates_decode", started_at);
        records
    }

    fn load_record_set_impl(&self) -> Result<PersistedRecordSet, LadybugIndexReaderError> {
        let key_rows = query_rows(
            &self.connection,
            "MATCH (record:Record)
             RETURN record.record_key
             ORDER BY record.record_key;",
        )?;
        let keys = key_rows
            .iter()
            .map(|row| record_key_at(row, 0))
            .collect::<Result<Vec<_>, _>>()?;
        let records = self.load_records_by_key_impl(&keys)?;
        let alias_rows = query_rows(
            &self.connection,
            "MATCH (record:Record)-[:HAS_ALIAS]->(alias:Alias)
             RETURN record.record_key, alias.alias_text, alias.normalized_alias,
                    alias.source_kind, alias.source_ref
             ORDER BY record.record_key, alias.normalized_alias;",
        )?;
        let aliases = alias_rows
            .iter()
            .map(|row| alias_from_row(row))
            .collect::<Result<Vec<_>, _>>()?;
        let reference_edges = self.reference_edges_impl()?;
        let remaster_links = self.remaster_links_impl()?;
        Ok(PersistedRecordSet {
            records,
            aliases,
            reference_edges,
            remaster_links,
        })
    }

    fn resolve_record_matches_with_options_impl(
        &self,
        query: &str,
        normalized_query: &str,
        filter: Option<&SearchFilterNode>,
        options: RecordLoadOptions,
    ) -> Result<Vec<RecordResolutionResult>, LadybugIndexReaderError> {
        let exact = self.resolve_record_key_matches(
            RecordKeyMatchQuery {
                query,
                normalized_query,
                match_kind: RecordResolutionMatchKind::Name,
                field: "record.name",
                value: query,
                require_no_variant_label: false,
            },
            filter,
            options,
        )?;
        if !exact.is_empty() {
            return Ok(exact);
        }

        let normalized = self.resolve_record_key_matches(
            RecordKeyMatchQuery {
                query,
                normalized_query,
                match_kind: RecordResolutionMatchKind::NormalizedName,
                field: "record.normalized_name",
                value: normalized_query,
                require_no_variant_label: true,
            },
            filter,
            options,
        )?;
        if !normalized.is_empty() {
            return Ok(normalized);
        }

        let alias = self.resolve_alias_matches(query, normalized_query, filter, options)?;
        if !alias.is_empty() {
            return Ok(alias);
        }

        self.resolve_record_key_matches(
            RecordKeyMatchQuery {
                query,
                normalized_query,
                match_kind: RecordResolutionMatchKind::VariantName,
                field: "record.normalized_name",
                value: normalized_query,
                require_no_variant_label: false,
            },
            filter,
            options,
        )
    }

    fn resolve_record_key_matches(
        &self,
        key_query: RecordKeyMatchQuery<'_>,
        filter: Option<&SearchFilterNode>,
        options: RecordLoadOptions,
    ) -> Result<Vec<RecordResolutionResult>, LadybugIndexReaderError> {
        let RecordKeyMatchQuery {
            query,
            normalized_query,
            match_kind,
            field,
            value,
            require_no_variant_label,
        } = key_query;
        let scope = compile_scope(filter)?;
        let variant_predicate = match match_kind {
            RecordResolutionMatchKind::NormalizedName if require_no_variant_label => {
                " AND record.variant_label IS NULL"
            }
            RecordResolutionMatchKind::VariantName => " AND record.variant_label IS NOT NULL",
            _ => "",
        };
        let sql = format!(
            "{} AND {field} = {}{variant_predicate}
             RETURN DISTINCT record.record_key
             ORDER BY record.record_key;",
            scope.match_with_where("record"),
            string_literal(value)
        );
        let keys = query_rows(&self.connection, &sql)?
            .iter()
            .map(|row| record_key_at(row, 0))
            .collect::<Result<Vec<_>, _>>()?;
        let mut records = self.load_records_by_key_with_options_impl(&keys, options)?;
        records.sort_by(|left, right| left.key.cmp(&right.key));
        Ok(records
            .into_iter()
            .map(|record| RecordResolutionResult {
                query: query.to_string(),
                normalized_query: normalized_query.to_string(),
                match_kind,
                matched_text: match match_kind {
                    RecordResolutionMatchKind::Name => record.name.clone(),
                    _ => record.normalized_name.clone(),
                },
                alias_source: None,
                alias_source_ref: None,
                record,
            })
            .collect())
    }

    fn resolve_alias_matches(
        &self,
        query: &str,
        normalized_query: &str,
        filter: Option<&SearchFilterNode>,
        options: RecordLoadOptions,
    ) -> Result<Vec<RecordResolutionResult>, LadybugIndexReaderError> {
        let scope = compile_scope(filter)?;
        let sql = format!(
            "MATCH (record:Record)-[:HAS_ALIAS]->(alias:Alias)
             {}
             {}
             AND alias.normalized_alias = {}
             RETURN DISTINCT record.record_key, alias.alias_text, alias.source_kind, alias.source_ref
             ORDER BY record.record_key, alias.alias_text;",
            scope.optional_match_suffix("record"),
            scope.where_clause("record"),
            string_literal(normalized_query)
        );
        let alias_rows = query_rows(&self.connection, &sql)?;
        let keys = alias_rows
            .iter()
            .map(|row| record_key_at(row, 0))
            .collect::<Result<Vec<_>, _>>()?;
        let records_by_key = self
            .load_records_by_key_with_options_impl(&keys, options)?
            .into_iter()
            .map(|record| (record.key.clone(), record))
            .collect::<BTreeMap<_, _>>();
        let mut matches = Vec::new();
        for row in alias_rows {
            let record_key = record_key_at(&row, 0)?;
            if let Some(record) = records_by_key.get(&record_key) {
                matches.push(RecordResolutionResult {
                    query: query.to_string(),
                    normalized_query: normalized_query.to_string(),
                    match_kind: RecordResolutionMatchKind::Alias,
                    matched_text: string_at(&row, 1)?,
                    alias_source: Some(string_at(&row, 2)?),
                    alias_source_ref: Some(string_at(&row, 3)?),
                    record: record.clone(),
                });
            }
        }
        Ok(matches)
    }

    fn metrics_for_keys(
        &self,
        keys: &[RecordKey],
    ) -> Result<BTreeMap<RecordKey, Vec<MetricRow>>, LadybugIndexReaderError> {
        let sql = format!(
            "MATCH (record:Record)-[metric_rel:HAS_METRIC]->(metric:Metric)
             WHERE record.record_key IN {}
             RETURN record.record_key, metric.metric_domain, metric.metric_key,
                    metric.value_type, metric_rel.number_value, metric_rel.text_value,
                    metric_rel.bool_value
             ORDER BY record.record_key, metric.metric_key;",
            list_literal(keys.iter().map(ToString::to_string))
        );
        let mut metrics = BTreeMap::<RecordKey, Vec<MetricRow>>::new();
        let rows = query_rows_traced(&self.connection, &sql, "ladybug_load_metrics_query")?;
        let started_at = Instant::now();
        for row in rows {
            let value_type =
                MetricValueType::from_canonical(&string_at(&row, 3)?).ok_or_else(|| {
                    LadybugIndexReaderError::InvalidData("invalid metric type".to_string())
                })?;
            let value = match value_type {
                MetricValueType::Number => MetricValue::Number(float_at(&row, 4)?),
                MetricValueType::Text => MetricValue::Text(string_at(&row, 5)?),
                MetricValueType::Boolean => MetricValue::Boolean(bool_at(&row, 6)?),
            };
            let domain = MetricDomain::from_canonical(&string_at(&row, 1)?).ok_or_else(|| {
                LadybugIndexReaderError::InvalidData("invalid metric domain".to_string())
            })?;
            metrics
                .entry(record_key_at(&row, 0)?)
                .or_default()
                .push(MetricRow {
                    domain,
                    key: string_at(&row, 2)?,
                    value,
                });
        }
        trace_ladybug_phase("ladybug_load_metrics_decode", started_at);
        Ok(metrics)
    }

    fn supplemental_content_for_keys(
        &self,
        keys: &[RecordKey],
    ) -> Result<BTreeMap<RecordKey, Vec<SupplementalContentDocument>>, LadybugIndexReaderError>
    {
        let sql = format!(
            "MATCH (record:Record)-[:HAS_CONTENT_UNIT]->(content:ContentUnit)
             WHERE record.record_key IN {}
             RETURN record.record_key, content.source_kind, content.visibility,
                    content.contributes_to_search, content.contributes_to_references,
                    content.label, content.content_json
             ORDER BY record.record_key, content.ordinal;",
            list_literal(keys.iter().map(ToString::to_string))
        );
        let mut content = BTreeMap::<RecordKey, Vec<SupplementalContentDocument>>::new();
        let rows = query_rows_traced(&self.connection, &sql, "ladybug_load_content_query")?;
        let started_at = Instant::now();
        for row in rows {
            let source_kind = ContentSourceKind::from_canonical(&string_at(&row, 1)?)
                .unwrap_or(ContentSourceKind::Description);
            let visibility = ContentVisibility::from_canonical(&string_at(&row, 2)?)
                .unwrap_or(ContentVisibility::Public);
            let document =
                serde_json::from_str::<ContentDocument>(&string_at(&row, 6)?).map_err(invalid)?;
            content
                .entry(record_key_at(&row, 0)?)
                .or_default()
                .push(SupplementalContentDocument {
                    source_kind,
                    visibility,
                    contributes_to_search: bool_at(&row, 3)?,
                    contributes_to_references: bool_at(&row, 4)?,
                    label: optional_string_at(&row, 5)?,
                    document,
                });
        }
        trace_ladybug_phase("ladybug_load_content_decode", started_at);
        Ok(content)
    }
}

pub(crate) fn string_literal(value: &str) -> String {
    format!("'{}'", value.replace('\\', "\\\\").replace('\'', "\\'"))
}

fn list_literal(values: impl IntoIterator<Item = String>) -> String {
    format!(
        "[{}]",
        values
            .into_iter()
            .map(|value| string_literal(&value))
            .collect::<Vec<_>>()
            .join(", ")
    )
}

fn vector_literal(values: &[f32]) -> String {
    format!(
        "[{}]",
        values
            .iter()
            .map(|value| value.to_string())
            .collect::<Vec<_>>()
            .join(", ")
    )
}

fn stable_hash(value: &str) -> u64 {
    let mut hash = 0xcbf29ce484222325_u64;
    for byte in value.as_bytes() {
        hash ^= u64::from(*byte);
        hash = hash.wrapping_mul(0x100000001b3);
    }
    hash
}

pub(crate) fn trace_ladybug_phase(phase: &str, started_at: Instant) {
    if std::env::var_os("ATLAS_SEARCH_TRACE").is_some() {
        eprintln!(
            "atlas-search trace: {phase} completed in {}ms",
            started_at.elapsed().as_millis()
        );
    }
}

pub(crate) fn unsupported<T>(feature: &str) -> Result<T, LadybugIndexReaderError> {
    Err(LadybugIndexReaderError::Unsupported(format!(
        "{feature} is not implemented in the Ladybug read spike yet"
    )))
}

pub(crate) fn invalid(error: impl std::fmt::Display) -> LadybugIndexReaderError {
    LadybugIndexReaderError::InvalidData(error.to_string())
}

fn json_string_array(context: &str, value: &str) -> Result<Vec<String>, LadybugIndexReaderError> {
    serde_json::from_str(value)
        .map_err(|error| LadybugIndexReaderError::InvalidData(format!("{context}: {error}")))
}

fn search_error(error: LadybugIndexReaderError) -> SearchError {
    match error {
        LadybugIndexReaderError::Unsupported(message) => SearchError::InvalidSearchOptions(message),
        LadybugIndexReaderError::InvalidData(message) | LadybugIndexReaderError::Query(message) => {
            SearchError::Embedding(message)
        }
    }
}
