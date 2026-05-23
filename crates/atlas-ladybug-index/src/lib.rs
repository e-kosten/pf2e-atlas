#![deny(unsafe_code)]

use std::collections::BTreeMap;
use std::path::Path;

use atlas_discovery::{all_discovery_field_definitions, metric_filter_field_info};
use atlas_domain::{
    BooleanFieldCounts, FilterDiscoveryExecution, FilterFieldDiscovery, FilterFieldInfo,
    FilterFieldStats, FilterSample, FilterSampleExample, FilterValueCount, FilterValueDiscovery,
    FilterValuePayload, FilterValuePolicy, MetadataBooleanField, MetadataBooleanMatch,
    MetadataEnumStringField, MetadataNumberField, MetadataNumberMatch, MetadataPredicate,
    MetadataSetField, MetadataSetMatch, MetadataStringMatch, MetadataTextMatch,
    MetadataTextStringField, MetricDomain, MetricKeyDiscovery, MetricValuePayload, MetricValueType,
    NumericFieldStats, PackName, PublicationFamily, RecordFamily, RecordId, RecordKey,
    RemasterLinkSource, ScalarValue, SearchFilterNode, TimeKind, TimeUnit,
};
use atlas_index::{
    DiscoveryError, DiscoveryValueSort, FilterValueRequest, FilteredRecordKeyPage,
    FilteredRecordSort, FtsColumnWeights, FtsQuery, FtsSearchHit, GraphReferenceEdge,
    ReferenceEdgeDirection, VectorSearchHit,
};
use atlas_record::{
    ActorSideData, AliasSource, ContentDocument, ContentSourceKind, ContentVisibility,
    ItemSideData, MetricRow, MetricValue, NormalizedTime, PersistedRecord, PersistedRecordSet,
    RecordAlias, ReferenceEdge, RemasterLink, SpellSideData, SupplementalContentDocument,
};
use atlas_search::{RecordResolutionMatchKind, RecordResolutionResult, SearchError, SearchIndex};
use lbug::{Connection, Database, SystemConfig, Value};
use thiserror::Error;

pub struct LadybugIndex {
    connection: Connection<'static>,
}

#[derive(Debug, Error)]
pub enum LadybugIndexError {
    #[error("LadybugDB query failed: {0}")]
    Query(String),
    #[error("LadybugDB row had invalid data: {0}")]
    InvalidData(String),
    #[error("LadybugDB search shape is not supported yet: {0}")]
    Unsupported(String),
}

impl LadybugIndex {
    pub fn open(path: impl AsRef<Path>) -> Result<Self, LadybugIndexError> {
        let database = Box::leak(Box::new(
            Database::new(path.as_ref(), SystemConfig::default())
                .map_err(|error| LadybugIndexError::Query(error.to_string()))?,
        ));
        let connection = Connection::new(database)
            .map_err(|error| LadybugIndexError::Query(error.to_string()))?;
        let index = Self { connection };
        index.prepare_search_indexes();
        Ok(index)
    }

    fn prepare_search_indexes(&self) {
        load_extension(&self.connection, "FTS");
        load_extension(&self.connection, "VECTOR");
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

impl SearchIndex for LadybugIndex {
    fn load_records_by_key(&self, keys: &[RecordKey]) -> Result<Vec<PersistedRecord>, SearchError> {
        self.load_records_by_key_impl(keys).map_err(search_error)
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
        self.resolve_record_matches_impl(query, normalized_query, filter)
            .map(Some)
            .map_err(search_error)
    }

    fn reference_edges_for_seed(
        &self,
        seed: &RecordKey,
        direction: ReferenceEdgeDirection,
    ) -> Result<Vec<GraphReferenceEdge>, SearchError> {
        self.reference_edges_for_seed_impl(seed, direction)
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

impl LadybugIndex {
    fn load_records_by_key_impl(
        &self,
        keys: &[RecordKey],
    ) -> Result<Vec<PersistedRecord>, LadybugIndexError> {
        if keys.is_empty() {
            return Ok(Vec::new());
        }
        let sql = format!(
            "MATCH (record:Record)-[:FROM_PACK]->(pack:Pack)
             WHERE record.record_key IN {}
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
                    record.is_default_visible, record.source_path, record.raw_json,
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
             ORDER BY record.record_key;",
            list_literal(keys.iter().map(ToString::to_string))
        );
        let rows = query_rows(&self.connection, &sql)?;
        let mut records = rows
            .iter()
            .map(|row| record_from_row(row))
            .collect::<Result<Vec<_>, _>>()?;
        let traits = self.traits_for_keys(keys)?;
        let metrics = self.metrics_for_keys(keys)?;
        let supplemental_content = self.supplemental_content_for_keys(keys)?;
        for record in &mut records {
            if let Some(values) = traits.get(&record.key) {
                record.traits = values.clone();
            }
            record.metrics = metrics.get(&record.key).cloned().unwrap_or_default();
            record.supplemental_content = supplemental_content
                .get(&record.key)
                .cloned()
                .unwrap_or_default();
        }
        Ok(records)
    }

    fn load_record_set_impl(&self) -> Result<PersistedRecordSet, LadybugIndexError> {
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

    fn resolve_record_matches_impl(
        &self,
        query: &str,
        normalized_query: &str,
        filter: Option<&SearchFilterNode>,
    ) -> Result<Vec<RecordResolutionResult>, LadybugIndexError> {
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
        )?;
        if !normalized.is_empty() {
            return Ok(normalized);
        }

        let alias = self.resolve_alias_matches(query, normalized_query, filter)?;
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
        )
    }

    fn resolve_record_key_matches(
        &self,
        key_query: RecordKeyMatchQuery<'_>,
        filter: Option<&SearchFilterNode>,
    ) -> Result<Vec<RecordResolutionResult>, LadybugIndexError> {
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
        let mut records = self.load_records_by_key_impl(&keys)?;
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
    ) -> Result<Vec<RecordResolutionResult>, LadybugIndexError> {
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
            .load_records_by_key_impl(&keys)?
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

    fn reference_edges_for_seed_impl(
        &self,
        seed: &RecordKey,
        direction: ReferenceEdgeDirection,
    ) -> Result<Vec<GraphReferenceEdge>, LadybugIndexError> {
        let (pattern, from_index, to_index) = match direction {
            ReferenceEdgeDirection::Outgoing => (
                format!(
                    "MATCH (from:Record {{record_key: {}}})-[edge:REFERENCES]->(to:Record)",
                    string_literal(&seed.to_string())
                ),
                0,
                1,
            ),
            ReferenceEdgeDirection::Backlink => (
                format!(
                    "MATCH (from:Record)-[edge:REFERENCES]->(to:Record {{record_key: {}}})",
                    string_literal(&seed.to_string())
                ),
                0,
                1,
            ),
        };
        let sql = format!(
            "{pattern}
             WHERE edge.visibility = 'public'
             RETURN from.record_key, to.record_key, edge.display_text, edge.reference_text,
                    edge.source_kind, edge.visibility;"
        );
        query_rows(&self.connection, &sql)?
            .iter()
            .map(|row| graph_edge_from_row(row, from_index, to_index))
            .collect()
    }

    fn reference_edges_impl(&self) -> Result<Vec<ReferenceEdge>, LadybugIndexError> {
        query_rows(
            &self.connection,
            "MATCH (from:Record)-[edge:REFERENCES]->(to:Record)
             RETURN from.record_key, to.record_key, edge.display_text, edge.reference_text,
                    edge.source_kind, edge.visibility
             ORDER BY from.record_key, to.record_key, edge.reference_text;",
        )?
        .iter()
        .map(|row| reference_edge_from_row(row))
        .collect()
    }

    fn remaster_links_impl(&self) -> Result<Vec<RemasterLink>, LadybugIndexError> {
        query_rows(
            &self.connection,
            "MATCH (legacy:Record)-[edge:REMASTERED_BY]->(remaster:Record)
             RETURN remaster.record_key, legacy.record_key, edge.source_kind, edge.source_ref
             ORDER BY legacy.record_key, remaster.record_key;",
        )?
        .iter()
        .map(|row| remaster_link_from_row(row))
        .collect()
    }

    fn list_filter_fields_impl(
        &self,
        filter: Option<&SearchFilterNode>,
        filter_json: Option<serde_json::Value>,
    ) -> Result<FilterFieldDiscovery, DiscoveryError> {
        let matching_record_count = self.count_matching_records_discovery(filter)?;
        let fields = all_discovery_field_definitions()
            .iter()
            .filter(|definition| ladybug_projection(definition.field).is_some())
            .map(|definition| self.field_applies_discovery(*definition, filter))
            .collect::<Result<Vec<_>, _>>()?
            .into_iter()
            .flatten()
            .collect::<Vec<_>>();
        let mut fields = fields;
        if self.metric_key_count_discovery(filter)? > 0 {
            fields.push(metric_filter_field_info(false));
        }
        Ok(FilterFieldDiscovery {
            filter: filter_json,
            execution: FilterDiscoveryExecution::Dynamic,
            matching_record_count,
            fields,
        })
    }

    fn list_filter_values_impl(
        &self,
        filter: Option<&SearchFilterNode>,
        request: FilterValueRequest,
    ) -> Result<FilterValueDiscovery, DiscoveryError> {
        if request.field == "metric" {
            return self.metric_values_discovery(filter, request);
        }
        let definition = atlas_discovery::discovery_field_definition(&request.field)
            .ok_or_else(|| unknown_discovery_field(&request.field))?;
        let projection = ladybug_projection(definition.field).ok_or_else(|| {
            DiscoveryError::InvalidOption(format!(
                "field `{}` is not implemented for LadybugDB discovery yet",
                definition.field
            ))
        })?;
        if request.sort.is_some() && definition.value_policy != FilterValuePolicy::Enumerable {
            return Err(DiscoveryError::InvalidOption(
                "--sort applies only to enumerable value fields".to_string(),
            ));
        }
        if request.sample_limit.is_some() && definition.value_policy != FilterValuePolicy::Sample {
            return Err(DiscoveryError::InvalidOption(
                "--sample-limit applies only to sampled text fields".to_string(),
            ));
        }
        let matching_record_count = self.count_matching_records_discovery(filter)?;
        if matching_record_count > 0
            && !self.projection_applies_discovery(projection.clone(), filter)?
        {
            return Err(DiscoveryError::FieldNotApplicable(format!(
                "field `{}` is not applicable in the current filter space",
                request.field
            )));
        }
        let payload = match definition.value_policy {
            FilterValuePolicy::Enumerable => {
                let sort = request.sort.unwrap_or(definition.default_sort);
                let values = self.enumerable_values_discovery(projection.clone(), filter, sort)?;
                let null_count = self.null_count_discovery(projection, filter)?;
                FilterValuePayload::Enumerable {
                    values,
                    null_count,
                    sort,
                }
            }
            FilterValuePolicy::Sample => {
                let sample_limit = request.sample_limit.unwrap_or(20);
                if sample_limit == 0 || sample_limit > 100 {
                    return Err(DiscoveryError::InvalidOption(
                        "--sample-limit must be between 1 and 100".to_string(),
                    ));
                }
                let values = self.enumerable_values_discovery(
                    projection.clone(),
                    filter,
                    DiscoveryValueSort::Count,
                )?;
                let null_count = self.null_count_discovery(projection, filter)?;
                let mut field_stats = filter_field_stats(&values);
                field_stats.null_count = null_count;
                let examples = values
                    .iter()
                    .take(sample_limit)
                    .map(|value| FilterSampleExample {
                        text: value.value.clone(),
                        count: value.count,
                        truncated: false,
                    })
                    .collect::<Vec<_>>();
                FilterValuePayload::Sample {
                    sample: FilterSample {
                        selection: "top_repeated_then_deterministic".to_string(),
                        sample_limit,
                        distinct_count: field_stats.distinct_count,
                        omitted_distinct_count: field_stats
                            .distinct_count
                            .saturating_sub(examples.len() as u64),
                        examples,
                    },
                    field_stats,
                    null_count,
                }
            }
            FilterValuePolicy::NumericStats => FilterValuePayload::NumericStats {
                stats: self.numeric_stats_discovery(projection, filter)?,
            },
            FilterValuePolicy::BooleanCounts => FilterValuePayload::BooleanCounts {
                counts: self.boolean_counts_discovery(projection, filter)?,
            },
            _ => {
                return Err(DiscoveryError::InvalidOption(format!(
                    "field `{}` is not a metadata value field",
                    definition.field
                )));
            }
        };
        Ok(FilterValueDiscovery {
            field: definition.field.to_string(),
            filter: request.filter_json,
            execution: FilterDiscoveryExecution::Dynamic,
            matching_record_count,
            payload,
        })
    }

    fn list_filtered_record_keys_impl(
        &self,
        filter: Option<&SearchFilterNode>,
        sort: FilteredRecordSort,
        limit: u32,
        offset: u32,
    ) -> Result<FilteredRecordKeyPage, LadybugIndexError> {
        let scope = compile_scope(filter)?;
        let sort_clause = sort_clause(sort)?;
        let total_sql = format!(
            "{} RETURN count(DISTINCT record);",
            scope.match_with_where("record")
        );
        let total = query_rows(&self.connection, &total_sql)?
            .first()
            .and_then(|row| int_at(row, 0).ok())
            .unwrap_or(0) as u64;
        let sql = format!(
            "{} RETURN DISTINCT record.record_key, record.name, record.level, record.source_path
             {sort_clause}
             SKIP {offset}
             LIMIT {limit};",
            scope.match_with_where("record")
        );
        let record_keys = query_rows(&self.connection, &sql)?
            .iter()
            .map(|row| record_key_at(row, 0))
            .collect::<Result<Vec<_>, _>>()?;
        Ok(FilteredRecordKeyPage { record_keys, total })
    }

    fn count_matching_records_discovery(
        &self,
        filter: Option<&SearchFilterNode>,
    ) -> Result<u64, DiscoveryError> {
        let scope = compile_scope(filter).map_err(discovery_error)?;
        let sql = format!(
            "{} RETURN count(DISTINCT record);",
            scope.match_with_where("record")
        );
        query_rows(&self.connection, &sql)
            .map_err(discovery_error)?
            .first()
            .map(|row| u64_at(row, 0).map_err(discovery_error))
            .transpose()
            .map(|value| value.unwrap_or(0))
    }

    fn field_applies_discovery(
        &self,
        definition: atlas_discovery::DiscoveryFieldDefinition,
        filter: Option<&SearchFilterNode>,
    ) -> Result<Option<FilterFieldInfo>, DiscoveryError> {
        let Some(projection) = ladybug_projection(definition.field) else {
            return Ok(None);
        };
        if self.projection_applies_discovery(projection, filter)? {
            Ok(Some(definition.info(false)))
        } else {
            Ok(None)
        }
    }

    fn projection_applies_discovery(
        &self,
        projection: LadybugValueProjection,
        filter: Option<&SearchFilterNode>,
    ) -> Result<bool, DiscoveryError> {
        let scope = compile_scope(filter).map_err(discovery_error)?;
        let sql = format!(
            "{} {} {}
             WITH {} AS value
             WHERE value IS NOT NULL{}
             RETURN count(value) > 0;",
            scope.match_with_where("record"),
            projection.match_clause,
            projection.extra_where_clause(),
            projection.value_expr,
            projection.non_empty_string_predicate("value")
        );
        query_rows(&self.connection, &sql)
            .map_err(discovery_error)?
            .first()
            .map(|row| bool_at(row, 0).map_err(discovery_error))
            .transpose()
            .map(|value| value.unwrap_or(false))
    }

    fn enumerable_values_discovery(
        &self,
        projection: LadybugValueProjection,
        filter: Option<&SearchFilterNode>,
        sort: DiscoveryValueSort,
    ) -> Result<Vec<FilterValueCount>, DiscoveryError> {
        let scope = compile_scope(filter).map_err(discovery_error)?;
        let order = match sort {
            DiscoveryValueSort::Alpha | DiscoveryValueSort::Canonical => "value ASC",
            DiscoveryValueSort::Count => "catalog_count DESC, value ASC",
        };
        let sql = format!(
            "{} {} {}
             WITH {} AS value, record
             WHERE value IS NOT NULL{}
             RETURN value, count(DISTINCT record) AS catalog_count
             ORDER BY {order};",
            scope.match_with_where("record"),
            projection.match_clause,
            projection.extra_where_clause(),
            projection.value_expr,
            projection.non_empty_string_predicate("value")
        );
        query_rows(&self.connection, &sql)
            .map_err(discovery_error)?
            .iter()
            .map(|row| {
                Ok(FilterValueCount {
                    value: value_to_discovery_string(row, 0)?,
                    count: u64_at(row, 1)?,
                })
            })
            .collect::<Result<Vec<_>, _>>()
            .map_err(discovery_error)
    }

    fn null_count_discovery(
        &self,
        projection: LadybugValueProjection,
        filter: Option<&SearchFilterNode>,
    ) -> Result<u64, DiscoveryError> {
        let total = self.count_matching_records_discovery(filter)?;
        let scope = compile_scope(filter).map_err(discovery_error)?;
        let sql = format!(
            "{} {} {}
             WITH record, {} AS value
             WHERE value IS NOT NULL{}
             RETURN count(DISTINCT record);",
            scope.match_with_where("record"),
            projection.match_clause,
            projection.extra_where_clause(),
            projection.value_expr,
            projection.non_empty_string_predicate("value")
        );
        let with_value = query_rows(&self.connection, &sql)
            .map_err(discovery_error)?
            .first()
            .map(|row| u64_at(row, 0).map_err(discovery_error))
            .transpose()?
            .unwrap_or(0);
        Ok(total.saturating_sub(with_value))
    }

    fn numeric_stats_discovery(
        &self,
        projection: LadybugValueProjection,
        filter: Option<&SearchFilterNode>,
    ) -> Result<NumericFieldStats, DiscoveryError> {
        let scope = compile_scope(filter).map_err(discovery_error)?;
        let sql = format!(
            "{} {} {}
             WITH {} AS value
             WHERE value IS NOT NULL
             RETURN value
             ORDER BY value ASC;",
            scope.match_with_where("record"),
            projection.match_clause,
            projection.extra_where_clause(),
            projection.value_expr,
        );
        let values = query_rows(&self.connection, &sql)
            .map_err(discovery_error)?
            .iter()
            .map(|row| float_at(row, 0))
            .collect::<Result<Vec<_>, _>>()
            .map_err(discovery_error)?;
        Ok(numeric_stats_from_values(
            &values,
            self.count_matching_records_discovery(filter)?,
        ))
    }

    fn boolean_counts_discovery(
        &self,
        projection: LadybugValueProjection,
        filter: Option<&SearchFilterNode>,
    ) -> Result<BooleanFieldCounts, DiscoveryError> {
        let scope = compile_scope(filter).map_err(discovery_error)?;
        let count_value = |expected: bool| -> Result<u64, DiscoveryError> {
            let sql = format!(
                "{} {} {}
                 WITH record, {} AS value
                 WHERE value = {expected}
                 RETURN count(DISTINCT record);",
                scope.match_with_where("record"),
                projection.match_clause,
                projection.extra_where_clause(),
                projection.value_expr
            );
            query_rows(&self.connection, &sql)
                .map_err(discovery_error)?
                .first()
                .map(|row| u64_at(row, 0).map_err(discovery_error))
                .transpose()
                .map(|value| value.unwrap_or(0))
        };
        let true_count = count_value(true)?;
        let false_count = count_value(false)?;
        Ok(BooleanFieldCounts {
            r#true: true_count,
            r#false: false_count,
            null: self
                .count_matching_records_discovery(filter)?
                .saturating_sub(true_count + false_count),
        })
    }

    fn metric_key_count_discovery(
        &self,
        filter: Option<&SearchFilterNode>,
    ) -> Result<u64, DiscoveryError> {
        let scope = compile_scope(filter).map_err(discovery_error)?;
        let sql = format!(
            "{} MATCH (record)-[:HAS_METRIC]->(metric:Metric)
             RETURN count(DISTINCT metric.metric_key);",
            scope.match_with_where("record")
        );
        query_rows(&self.connection, &sql)
            .map_err(discovery_error)?
            .first()
            .map(|row| u64_at(row, 0).map_err(discovery_error))
            .transpose()
            .map(|value| value.unwrap_or(0))
    }

    fn metric_values_discovery(
        &self,
        filter: Option<&SearchFilterNode>,
        request: FilterValueRequest,
    ) -> Result<FilterValueDiscovery, DiscoveryError> {
        let matching_record_count = self.count_matching_records_discovery(filter)?;
        let payload = if let Some(metric_key) = request.metric.as_deref() {
            let metric = resolve_ladybug_metric_from_candidates(
                self.metric_keys_discovery(
                    filter,
                    Some(metric_key),
                    request.metric_prefix.as_deref(),
                    request.metric_query.as_deref(),
                    request.metric_domain.as_deref(),
                    false,
                )?,
                metric_key,
            )?;
            let values = self.metric_value_payload_discovery(filter, &metric)?;
            FilterValuePayload::MetricValues {
                metric: Box::new(metric),
                values,
            }
        } else {
            FilterValuePayload::MetricKeys {
                metrics: self.metric_keys_discovery(
                    filter,
                    request.metric_label.as_deref(),
                    request.metric_prefix.as_deref(),
                    request.metric_query.as_deref(),
                    request.metric_domain.as_deref(),
                    false,
                )?,
            }
        };
        Ok(FilterValueDiscovery {
            field: "metric".to_string(),
            filter: request.filter_json,
            execution: FilterDiscoveryExecution::Dynamic,
            matching_record_count,
            payload,
        })
    }

    #[allow(clippy::too_many_arguments)]
    fn metric_keys_discovery(
        &self,
        filter: Option<&SearchFilterNode>,
        exact_metric_or_label: Option<&str>,
        prefix: Option<&str>,
        query: Option<&str>,
        domain: Option<&str>,
        exact_metric_only: bool,
    ) -> Result<Vec<MetricKeyDiscovery>, DiscoveryError> {
        let scope = compile_scope(filter).map_err(discovery_error)?;
        let mut predicates = Vec::new();
        if let Some(value) = exact_metric_or_label {
            let literal = string_literal(value);
            if exact_metric_only {
                predicates.push(format!("metric.metric_key = {literal}"));
            } else {
                predicates.push(format!(
                    "(metric.metric_key = {literal} OR metric.label = {literal} OR metric.short_label = {literal})"
                ));
            }
        }
        if let Some(prefix) = prefix {
            predicates.push(format!(
                "metric.metric_key STARTS WITH {}",
                string_literal(prefix)
            ));
        }
        if let Some(query) = query {
            let literal = string_literal(&query.to_ascii_lowercase());
            predicates.push(format!(
                "(lower(metric.metric_key) CONTAINS {literal} OR lower(metric.label) CONTAINS {literal} OR lower(metric.short_label) CONTAINS {literal})"
            ));
        }
        if let Some(domain) = domain {
            predicates.push(format!("metric.metric_domain = {}", string_literal(domain)));
        }
        let where_clause = if predicates.is_empty() {
            String::new()
        } else {
            format!("WHERE {}", predicates.join(" AND "))
        };
        let sql = format!(
            "{} MATCH (record)-[:HAS_METRIC]->(metric:Metric)
             {where_clause}
             RETURN metric.metric_domain, record.record_family, metric.metric_key,
                    metric.value_type, metric.namespace_prefix, metric.label,
                    metric.short_label, metric.group_name, metric.known,
                    count(DISTINCT record)
             ORDER BY metric.metric_key;",
            scope.match_with_where("record")
        );
        query_rows(&self.connection, &sql)
            .map_err(discovery_error)?
            .iter()
            .map(|row| {
                let metric_key = string_at(row, 2)?;
                Ok(MetricKeyDiscovery {
                    metric_domain: string_at(row, 0)?,
                    record_family: string_at(row, 1)?,
                    namespace_prefix: string_at(row, 4)?,
                    metric_key,
                    label: optional_string_at(row, 5)?,
                    short_label: optional_string_at(row, 6)?,
                    group: optional_string_at(row, 7)?,
                    known: bool_at(row, 8)?,
                    value_type: string_at(row, 3)?,
                    count: u64_at(row, 9)?,
                    numeric_stats: None,
                })
            })
            .collect::<Result<Vec<_>, _>>()
            .map_err(discovery_error)
    }

    fn metric_value_payload_discovery(
        &self,
        filter: Option<&SearchFilterNode>,
        metric: &MetricKeyDiscovery,
    ) -> Result<MetricValuePayload, DiscoveryError> {
        let projection = LadybugValueProjection {
            match_clause: "MATCH (record)-[metric_rel:HAS_METRIC]->(metric:Metric)".to_string(),
            value_expr: match metric.value_type.as_str() {
                "number" => "metric_rel.number_value",
                "boolean" => "metric_rel.bool_value",
                _ => "metric_rel.text_value",
            }
            .to_string(),
            value_kind: match metric.value_type.as_str() {
                "number" => ProjectionValueKind::Number,
                "boolean" => ProjectionValueKind::Boolean,
                _ => ProjectionValueKind::String,
            },
            extra_predicate: Some(format!(
                "metric.metric_key = {}",
                string_literal(&metric.metric_key)
            )),
        };
        match metric.value_type.as_str() {
            "number" => Ok(MetricValuePayload::NumericStats {
                stats: self.numeric_stats_discovery(projection, filter)?,
            }),
            "boolean" => Ok(MetricValuePayload::BooleanCounts {
                counts: self.boolean_counts_discovery(projection, filter)?,
            }),
            _ => Ok(MetricValuePayload::TextValues {
                values: self.enumerable_values_discovery(
                    projection,
                    filter,
                    DiscoveryValueSort::Count,
                )?,
            }),
        }
    }

    fn query_fts_index_impl(
        &self,
        fts_query: &FtsQuery,
        filter: Option<&SearchFilterNode>,
        limit: u32,
    ) -> Result<Vec<FtsSearchHit>, LadybugIndexError> {
        if limit == 0 {
            return Ok(Vec::new());
        }
        let scope = compile_scope(filter)?;
        let sql = format!(
            "CALL QUERY_FTS_INDEX('SearchDocument', 'search_document_fts', {}, top := {})
             WITH node AS doc, score
             MATCH (record:Record)-[:HAS_SEARCH_DOCUMENT]->(doc)
             {}
             {}
             RETURN record.record_key, score
             ORDER BY score DESC
             LIMIT {};",
            string_literal(&fts_query.as_match_query()),
            limit,
            scope.optional_match_suffix("record"),
            scope.where_clause("record"),
            limit
        );
        query_rows(&self.connection, &sql)?
            .iter()
            .map(|row| {
                Ok(FtsSearchHit {
                    record_key: record_key_at(row, 0)?,
                    rank: float_at(row, 1)?,
                })
            })
            .collect()
    }

    fn query_fts_candidate_record_keys_impl(
        &self,
        fts_query: &FtsQuery,
        candidate_keys: &[RecordKey],
    ) -> Result<Vec<RecordKey>, LadybugIndexError> {
        if candidate_keys.is_empty() {
            return Ok(Vec::new());
        }
        let sql = format!(
            "CALL QUERY_FTS_INDEX('SearchDocument', 'search_document_fts', {}, top := {})
             WITH node AS doc, score
             MATCH (record:Record)-[:HAS_SEARCH_DOCUMENT]->(doc)
             WHERE record.record_key IN {}
             RETURN DISTINCT record.record_key;",
            string_literal(&fts_query.as_match_query()),
            candidate_keys.len(),
            list_literal(candidate_keys.iter().map(ToString::to_string))
        );
        query_rows(&self.connection, &sql)?
            .iter()
            .map(|row| record_key_at(row, 0))
            .collect()
    }

    fn query_vector_index_impl(
        &self,
        query_vector: &[f32],
        filter: Option<&SearchFilterNode>,
        limit: u32,
        include_child_units: bool,
    ) -> Result<Vec<VectorSearchHit>, LadybugIndexError> {
        if limit == 0 {
            return Ok(Vec::new());
        }
        let scope = compile_scope(filter)?;
        let graph_name = if filter.is_some() {
            let projection = format!(
                "{} MATCH (record:Record)-[:HAS_EMBEDDING_UNIT]->(embedding:EmbeddingUnit)
                 {} RETURN embedding",
                scope.optional_match_prefix(),
                scope.where_clause("record")
            );
            let name = format!("eligible_embeddings_{}", stable_hash(&projection));
            let _ = self.connection.query(&format!(
                "CALL PROJECT_GRAPH_CYPHER({}, {});",
                string_literal(&name),
                string_literal(&projection)
            ));
            name
        } else {
            "EmbeddingUnit".to_string()
        };
        let unit_filter = if include_child_units {
            String::new()
        } else {
            "AND embedding.unit_kind = 'parent'".to_string()
        };
        let sql = format!(
            "CALL QUERY_VECTOR_INDEX({}, 'embedding_hnsw', CAST({}, 'FLOAT[{}]'), {}, efs := 50)
             WITH node AS embedding, distance
             MATCH (record:Record)-[:HAS_EMBEDDING_UNIT]->(embedding)
             WHERE record.is_default_visible {unit_filter}
             RETURN record.record_key, embedding.embedding_unit_key, embedding.unit_kind,
                    embedding.label, distance
             ORDER BY distance
             LIMIT {};",
            string_literal(&graph_name),
            vector_literal(query_vector),
            query_vector.len(),
            limit,
            limit
        );
        query_rows(&self.connection, &sql)?
            .iter()
            .map(|row| vector_hit_from_row(row))
            .collect()
    }

    fn traits_for_keys(
        &self,
        keys: &[RecordKey],
    ) -> Result<BTreeMap<RecordKey, Vec<String>>, LadybugIndexError> {
        let sql = format!(
            "MATCH (record:Record)-[:HAS_TRAIT]->(trait:Trait)
             WHERE record.record_key IN {}
             RETURN record.record_key, trait.name
             ORDER BY record.record_key, trait.name;",
            list_literal(keys.iter().map(ToString::to_string))
        );
        let mut traits = BTreeMap::<RecordKey, Vec<String>>::new();
        for row in query_rows(&self.connection, &sql)? {
            traits
                .entry(record_key_at(&row, 0)?)
                .or_default()
                .push(string_at(&row, 1)?);
        }
        Ok(traits)
    }

    fn metrics_for_keys(
        &self,
        keys: &[RecordKey],
    ) -> Result<BTreeMap<RecordKey, Vec<MetricRow>>, LadybugIndexError> {
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
        for row in query_rows(&self.connection, &sql)? {
            let value_type = MetricValueType::from_canonical(&string_at(&row, 3)?)
                .ok_or_else(|| LadybugIndexError::InvalidData("invalid metric type".to_string()))?;
            let value = match value_type {
                MetricValueType::Number => MetricValue::Number(float_at(&row, 4)?),
                MetricValueType::Text => MetricValue::Text(string_at(&row, 5)?),
                MetricValueType::Boolean => MetricValue::Boolean(bool_at(&row, 6)?),
            };
            let domain = MetricDomain::from_canonical(&string_at(&row, 1)?).ok_or_else(|| {
                LadybugIndexError::InvalidData("invalid metric domain".to_string())
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
        Ok(metrics)
    }

    fn supplemental_content_for_keys(
        &self,
        keys: &[RecordKey],
    ) -> Result<BTreeMap<RecordKey, Vec<SupplementalContentDocument>>, LadybugIndexError> {
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
        for row in query_rows(&self.connection, &sql)? {
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
        Ok(content)
    }
}

struct CompiledScope {
    matches: Vec<String>,
    predicates: Vec<String>,
}

#[derive(Debug, Clone)]
struct LadybugValueProjection {
    match_clause: String,
    value_expr: String,
    value_kind: ProjectionValueKind,
    extra_predicate: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ProjectionValueKind {
    String,
    Number,
    Boolean,
}

impl LadybugValueProjection {
    fn scalar(property: &'static str, value_kind: ProjectionValueKind) -> Self {
        Self {
            match_clause: String::new(),
            value_expr: format!("record.{property}"),
            value_kind,
            extra_predicate: None,
        }
    }

    fn relationship(
        match_clause: &'static str,
        value_expr: &'static str,
        value_kind: ProjectionValueKind,
    ) -> Self {
        Self {
            match_clause: match_clause.to_string(),
            value_expr: value_expr.to_string(),
            value_kind,
            extra_predicate: None,
        }
    }

    fn relationship_with_predicate(
        match_clause: String,
        value_expr: &'static str,
        value_kind: ProjectionValueKind,
        extra_predicate: String,
    ) -> Self {
        Self {
            match_clause,
            value_expr: value_expr.to_string(),
            value_kind,
            extra_predicate: Some(extra_predicate),
        }
    }

    fn non_empty_string_predicate(&self, _alias: &str) -> &'static str {
        if self.value_kind == ProjectionValueKind::String {
            " AND value <> ''"
        } else {
            ""
        }
    }

    fn extra_where_clause(&self) -> String {
        self.extra_predicate
            .as_ref()
            .map(|predicate| format!("WHERE {predicate}"))
            .unwrap_or_default()
    }
}

impl CompiledScope {
    fn match_with_where(&self, record_alias: &str) -> String {
        format!(
            "MATCH ({record_alias}:Record) {} {}",
            self.optional_match_suffix(record_alias),
            self.where_clause(record_alias)
        )
    }

    fn optional_match_prefix(&self) -> String {
        self.matches
            .iter()
            .map(|pattern| format!("MATCH {pattern}"))
            .collect::<Vec<_>>()
            .join(" ")
    }

    fn optional_match_suffix(&self, record_alias: &str) -> String {
        let matches = self
            .matches
            .iter()
            .map(|pattern| pattern.replace("(record)", &format!("({record_alias})")))
            .collect::<Vec<_>>()
            .join(" ");
        if matches.is_empty() {
            String::new()
        } else {
            format!("MATCH {matches}")
        }
    }

    fn where_clause(&self, record_alias: &str) -> String {
        let mut predicates = vec![format!("{record_alias}.is_default_visible")];
        predicates.extend(
            self.predicates
                .iter()
                .map(|predicate| predicate.replace("record.", &format!("{record_alias}."))),
        );
        format!("WHERE {}", predicates.join(" AND "))
    }
}

fn ladybug_projection(field: &str) -> Option<LadybugValueProjection> {
    use ProjectionValueKind::{Boolean, Number, String};
    Some(match field {
        "record_family" => LadybugValueProjection::scalar("record_family", String),
        "pack_name" => LadybugValueProjection::relationship(
            "MATCH (record)-[:FROM_PACK]->(value_pack:Pack)",
            "value_pack.pack_name",
            String,
        ),
        "pack_label" => LadybugValueProjection::relationship(
            "MATCH (record)-[:FROM_PACK]->(value_pack:Pack)",
            "value_pack.pack_label",
            String,
        ),
        "foundry_record_type" => LadybugValueProjection::scalar("foundry_record_type", String),
        "publication_title" => LadybugValueProjection::scalar("publication_title", String),
        "publication_family" => LadybugValueProjection::scalar("publication_family", String),
        "publication_remaster" => LadybugValueProjection::scalar("publication_remaster", Boolean),
        "rarity" => LadybugValueProjection::scalar("rarity", String),
        "level" => LadybugValueProjection::scalar("level", Number),
        "action_cost" => LadybugValueProjection::scalar("activation_time_actions", Number),
        "traits" => LadybugValueProjection::relationship(
            "MATCH (record)-[:HAS_TRAIT]->(value_trait:Trait)",
            "value_trait.name",
            String,
        ),
        "taxonomy_families" => filter_value_projection("taxonomy_families"),
        "traditions" => filter_value_projection("traditions"),
        "spell_kinds" => filter_value_projection("spell_kinds"),
        "damage_types" => filter_value_projection("damage_types"),
        "languages" => filter_value_projection("languages"),
        "speed_types" => filter_value_projection("speed_types"),
        "senses" => filter_value_projection("senses"),
        "immunities" => filter_value_projection("immunities"),
        "resistances" => filter_value_projection("resistances"),
        "weaknesses" => filter_value_projection("weaknesses"),
        "disable_skills" => filter_value_projection("disable_skills"),
        "variant_axes" => filter_value_projection("variant_axes"),
        "size" => LadybugValueProjection::scalar("actor_size", String),
        "usage" => LadybugValueProjection::scalar("system_usage", String),
        "item_group" => LadybugValueProjection::scalar("system_group", String),
        "item_category" => LadybugValueProjection::scalar("system_category", String),
        "base_item" => LadybugValueProjection::scalar("system_base_item", String),
        "hands" => LadybugValueProjection::scalar("item_hands_requirement", String),
        "save_type" => LadybugValueProjection::scalar("spell_save_type", String),
        "area_type" => LadybugValueProjection::scalar("spell_area_type", String),
        "duration_unit" => LadybugValueProjection::scalar("duration_unit", String),
        "sustained" => LadybugValueProjection::scalar("spell_sustained", Boolean),
        "basic_save" => LadybugValueProjection::scalar("spell_basic_save", Boolean),
        "is_complex" => LadybugValueProjection::scalar("actor_is_complex", Boolean),
        "price_cp" => LadybugValueProjection::scalar("price_cp", Number),
        "bulk_value" => LadybugValueProjection::scalar("item_bulk_value", Number),
        "range_value" => LadybugValueProjection::scalar("spell_range_value", Number),
        "area_value" => LadybugValueProjection::scalar("spell_area_value", Number),
        "variant_group_key" => LadybugValueProjection::scalar("variant_group_key", String),
        "range_text" => LadybugValueProjection::scalar("spell_range_text", String),
        "duration_text" => LadybugValueProjection::scalar("duration_text", String),
        "target_text" => LadybugValueProjection::scalar("spell_target_text", String),
        "disable_text" => LadybugValueProjection::scalar("actor_disable_text", String),
        "variant_base_name" => LadybugValueProjection::scalar("variant_base_name", String),
        "variant_label" => LadybugValueProjection::scalar("variant_label", String),
        _ => return None,
    })
}

fn filter_value_projection(field: &'static str) -> LadybugValueProjection {
    LadybugValueProjection::relationship_with_predicate(
        "MATCH (record)-[:HAS_FILTER_VALUE]->(value_filter:FilterValue)".to_string(),
        "value_filter.value",
        ProjectionValueKind::String,
        format!("value_filter.field = {}", string_literal(field)),
    )
}

fn compile_scope(filter: Option<&SearchFilterNode>) -> Result<CompiledScope, LadybugIndexError> {
    let mut matches = Vec::new();
    let mut predicates = Vec::new();
    if let Some(filter) = filter {
        predicates.push(compile_filter(filter, &mut matches)?);
    }
    Ok(CompiledScope {
        matches,
        predicates,
    })
}

fn compile_filter(
    filter: &SearchFilterNode,
    matches: &mut Vec<String>,
) -> Result<String, LadybugIndexError> {
    match filter {
        SearchFilterNode::RecordFamily { value } => Ok(format!(
            "record.record_family = {}",
            string_literal(value.as_str())
        )),
        SearchFilterNode::LinksTo { target } => {
            matches.push(format!(
                "(record)-[:REFERENCES]->(:Record {{record_key: {}}})",
                string_literal(&target.to_string())
            ));
            Ok("true".to_string())
        }
        SearchFilterNode::LinkedFrom { source } => {
            matches.push(format!(
                "(:Record {{record_key: {}}})-[:REFERENCES]->(record)",
                string_literal(&source.to_string())
            ));
            Ok("true".to_string())
        }
        SearchFilterNode::MetadataPredicate { predicate } => compile_metadata(predicate, matches),
        SearchFilterNode::Metric { metric, r#match } => {
            let metric_alias = format!("metric_{}", matches.len());
            matches.push(format!(
                "(record)-[{metric_alias}_rel:HAS_METRIC]->({metric_alias}:Metric {{metric_key: {}}})",
                string_literal(metric)
            ));
            Ok(match r#match {
                atlas_domain::MetricMatch::Eq {
                    value: ScalarValue::Number(value),
                } => {
                    format!("{metric_alias}_rel.number_value = {value}")
                }
                atlas_domain::MetricMatch::NotEq {
                    value: ScalarValue::Number(value),
                } => {
                    format!("{metric_alias}_rel.number_value <> {value}")
                }
                atlas_domain::MetricMatch::Eq { .. } | atlas_domain::MetricMatch::NotEq { .. } => {
                    return unsupported("non-numeric metric equality");
                }
                atlas_domain::MetricMatch::Gt { value } => {
                    format!("{metric_alias}_rel.number_value > {value}")
                }
                atlas_domain::MetricMatch::Gte { value } => {
                    format!("{metric_alias}_rel.number_value >= {value}")
                }
                atlas_domain::MetricMatch::Lt { value } => {
                    format!("{metric_alias}_rel.number_value < {value}")
                }
                atlas_domain::MetricMatch::Lte { value } => {
                    format!("{metric_alias}_rel.number_value <= {value}")
                }
            })
        }
        SearchFilterNode::MetricCompare { .. } => unsupported("metric comparison filters"),
        SearchFilterNode::AnyOf { children } => compile_set_include_any(children, matches)
            .or_else(|| Some(boolean_filter(children, " OR ", matches)))
            .expect("Some"),
        SearchFilterNode::AllOf { children } => boolean_filter(children, " AND ", matches),
        SearchFilterNode::Not { child } => Ok(format!("NOT ({})", compile_filter(child, matches)?)),
    }
}

fn boolean_filter(
    children: &[SearchFilterNode],
    joiner: &str,
    matches: &mut Vec<String>,
) -> Result<String, LadybugIndexError> {
    if children.is_empty() {
        return Ok(if joiner.contains("AND") {
            "true"
        } else {
            "false"
        }
        .to_string());
    }
    let parts = children
        .iter()
        .map(|child| compile_filter(child, matches).map(|value| format!("({value})")))
        .collect::<Result<Vec<_>, _>>()?;
    Ok(parts.join(joiner))
}

fn compile_set_include_any(
    children: &[SearchFilterNode],
    matches: &mut Vec<String>,
) -> Option<Result<String, LadybugIndexError>> {
    let mut values = Vec::new();
    let mut field = None;
    for child in children {
        let SearchFilterNode::MetadataPredicate {
            predicate:
                MetadataPredicate::Set {
                    field: child_field,
                    r#match: MetadataSetMatch::Includes { value },
                },
        } = child
        else {
            return None;
        };
        if field.is_some_and(|field| field != *child_field) {
            return None;
        }
        field = Some(*child_field);
        values.push(value.clone());
    }
    let field = field?;
    let alias = format!("set_any_{}", matches.len());
    let value_list = values
        .iter()
        .map(|value| string_literal(value))
        .collect::<Vec<_>>()
        .join(", ");
    match field {
        MetadataSetField::Traits => {
            matches.push(format!("(record)-[:HAS_TRAIT]->({alias}:Trait)"));
            Some(Ok(format!("{alias}.name IN [{value_list}]")))
        }
        field => match filter_value_field_name(field) {
            Some(field_name) => {
                matches.push(format!(
                    "(record)-[:HAS_FILTER_VALUE]->({alias}:FilterValue)"
                ));
                Some(Ok(format!(
                    "{alias}.field = {} AND {alias}.value IN [{value_list}]",
                    string_literal(field_name)
                )))
            }
            None => Some(unsupported("this metadata set filter")),
        },
    }
}

fn compile_metadata(
    predicate: &MetadataPredicate,
    matches: &mut Vec<String>,
) -> Result<String, LadybugIndexError> {
    match predicate {
        MetadataPredicate::Set { field, r#match } => compile_set_metadata(*field, r#match, matches),
        MetadataPredicate::EnumString { field, r#match } => {
            compile_enum_string_metadata(*field, r#match, matches)
        }
        MetadataPredicate::Text { field, r#match } => compile_text_metadata(*field, r#match),
        MetadataPredicate::Number { field, r#match } => {
            let Some(column) = number_field_column(*field) else {
                return unsupported("this metadata number filter");
            };
            number_match(column, *r#match)
        }
        MetadataPredicate::Boolean { field, r#match } => {
            let Some(column) = boolean_field_column(*field) else {
                return unsupported("this metadata boolean filter");
            };
            boolean_match(column, *r#match)
        }
    }
}

fn compile_set_metadata(
    field: MetadataSetField,
    r#match: &MetadataSetMatch,
    matches: &mut Vec<String>,
) -> Result<String, LadybugIndexError> {
    match (field, r#match) {
        (MetadataSetField::Traits, MetadataSetMatch::Includes { value }) => {
            matches.push(format!(
                "(record)-[:HAS_TRAIT]->(:Trait {{name: {}}})",
                string_literal(value)
            ));
            Ok("true".to_string())
        }
        (MetadataSetField::Traits, MetadataSetMatch::IsNotNull) => {
            matches.push("(record)-[:HAS_TRAIT]->(:Trait)".to_string());
            Ok("true".to_string())
        }
        (MetadataSetField::Traits, MetadataSetMatch::IsNull) => unsupported("null trait filters"),
        (field, MetadataSetMatch::Includes { value }) => {
            let Some(field_name) = filter_value_field_name(field) else {
                return unsupported("this metadata set filter");
            };
            matches.push(format!(
                "(record)-[:HAS_FILTER_VALUE]->(:FilterValue {{filter_value_key: {}}})",
                string_literal(&filter_value_key(field_name, value))
            ));
            Ok("true".to_string())
        }
        (field, MetadataSetMatch::IsNotNull) => {
            let Some(field_name) = filter_value_field_name(field) else {
                return unsupported("this metadata set filter");
            };
            let alias = format!("set_exists_{}", matches.len());
            matches.push(format!(
                "(record)-[:HAS_FILTER_VALUE]->({alias}:FilterValue)"
            ));
            Ok(format!("{alias}.field = {}", string_literal(field_name)))
        }
        (_, MetadataSetMatch::IsNull) => unsupported("null set filters"),
    }
}

fn compile_enum_string_metadata(
    field: MetadataEnumStringField,
    r#match: &MetadataStringMatch,
    matches: &mut Vec<String>,
) -> Result<String, LadybugIndexError> {
    match field {
        MetadataEnumStringField::PackName => compile_pack_metadata("pack_name", r#match, matches),
        MetadataEnumStringField::PackLabel => compile_pack_metadata("pack_label", r#match, matches),
        field => {
            let Some(column) = enum_string_field_column(field) else {
                return unsupported("this metadata enum filter");
            };
            string_match(column, r#match)
        }
    }
}

fn compile_pack_metadata(
    column: &'static str,
    r#match: &MetadataStringMatch,
    matches: &mut Vec<String>,
) -> Result<String, LadybugIndexError> {
    let alias = format!("pack_{}", matches.len());
    matches.push(format!("(record)-[:FROM_PACK]->({alias}:Pack)"));
    string_match(&format!("{alias}.{column}"), r#match)
}

fn compile_text_metadata(
    field: MetadataTextStringField,
    r#match: &MetadataTextMatch,
) -> Result<String, LadybugIndexError> {
    let Some(column) = text_field_column(field) else {
        return unsupported("this metadata text filter");
    };
    text_match(column, r#match)
}

fn string_match(field: &str, r#match: &MetadataStringMatch) -> Result<String, LadybugIndexError> {
    match r#match {
        MetadataStringMatch::Eq { value } => Ok(format!("{field} = {}", string_literal(value))),
        MetadataStringMatch::NotEq { value } => Ok(format!("{field} <> {}", string_literal(value))),
        MetadataStringMatch::IsNull => Ok(format!("{field} IS NULL")),
        MetadataStringMatch::IsNotNull => Ok(format!("{field} IS NOT NULL")),
    }
}

fn text_match(field: &str, r#match: &MetadataTextMatch) -> Result<String, LadybugIndexError> {
    match r#match {
        MetadataTextMatch::Eq { value } => Ok(format!("{field} = {}", string_literal(value))),
        MetadataTextMatch::NotEq { value } => Ok(format!("{field} <> {}", string_literal(value))),
        MetadataTextMatch::Contains { value } => {
            Ok(format!("{field} CONTAINS {}", string_literal(value)))
        }
        MetadataTextMatch::NotContains { value } => {
            Ok(format!("NOT ({field} CONTAINS {})", string_literal(value)))
        }
        MetadataTextMatch::IsNull => Ok(format!("{field} IS NULL")),
        MetadataTextMatch::IsNotNull => Ok(format!("{field} IS NOT NULL")),
    }
}

fn filter_value_field_name(field: MetadataSetField) -> Option<&'static str> {
    Some(match field {
        MetadataSetField::Traits | MetadataSetField::DerivedTags => return None,
        MetadataSetField::TaxonomyFamilies => "taxonomy_families",
        MetadataSetField::Traditions => "traditions",
        MetadataSetField::SpellKinds => "spell_kinds",
        MetadataSetField::DamageTypes => "damage_types",
        MetadataSetField::Languages => "languages",
        MetadataSetField::SpeedTypes => "speed_types",
        MetadataSetField::Senses => "senses",
        MetadataSetField::Immunities => "immunities",
        MetadataSetField::Resistances => "resistances",
        MetadataSetField::Weaknesses => "weaknesses",
        MetadataSetField::DisableSkills => "disable_skills",
        MetadataSetField::VariantAxes => "variant_axes",
    })
}

fn filter_value_key(field: &str, value: &str) -> String {
    format!("{field}:{value}")
}

fn enum_string_field_column(field: MetadataEnumStringField) -> Option<&'static str> {
    Some(match field {
        MetadataEnumStringField::PackName | MetadataEnumStringField::PackLabel => return None,
        MetadataEnumStringField::PublicationFamily => "record.publication_family",
        MetadataEnumStringField::Size => "record.actor_size",
        MetadataEnumStringField::Usage => "record.system_usage",
        MetadataEnumStringField::SystemGroup => "record.system_group",
        MetadataEnumStringField::FoundryRecordType => "record.foundry_record_type",
        MetadataEnumStringField::BaseItem => "record.system_base_item",
        MetadataEnumStringField::Hands => "record.item_hands_requirement",
        MetadataEnumStringField::SaveType => "record.spell_save_type",
        MetadataEnumStringField::AreaType => "record.spell_area_type",
        MetadataEnumStringField::DurationUnit => "record.duration_unit",
        MetadataEnumStringField::Rarity => "record.rarity",
        MetadataEnumStringField::VariantGroupKey => "record.variant_group_key",
    })
}

fn text_field_column(field: MetadataTextStringField) -> Option<&'static str> {
    Some(match field {
        MetadataTextStringField::PublicationTitle => "record.publication_title",
        MetadataTextStringField::RangeText => "record.spell_range_text",
        MetadataTextStringField::DurationText => "record.duration_text",
        MetadataTextStringField::TargetText => "record.spell_target_text",
        MetadataTextStringField::DisableText => "record.actor_disable_text",
        MetadataTextStringField::VariantBaseName => "record.variant_base_name",
        MetadataTextStringField::VariantLabel => "record.variant_label",
    })
}

fn number_field_column(field: MetadataNumberField) -> Option<&'static str> {
    Some(match field {
        MetadataNumberField::Level => "record.level",
        MetadataNumberField::PriceCp => "record.price_cp",
        MetadataNumberField::BulkValue => "record.item_bulk_value",
        MetadataNumberField::ActionCost => "record.activation_time_actions",
        MetadataNumberField::Hands => return None,
        MetadataNumberField::RangeValue => "record.spell_range_value",
        MetadataNumberField::AreaValue => "record.spell_area_value",
    })
}

fn boolean_field_column(field: MetadataBooleanField) -> Option<&'static str> {
    Some(match field {
        MetadataBooleanField::PublicationRemaster => "record.publication_remaster",
        MetadataBooleanField::Sustained => "record.spell_sustained",
        MetadataBooleanField::BasicSave => "record.spell_basic_save",
        MetadataBooleanField::IsComplex => "record.actor_is_complex",
    })
}

fn number_match(field: &str, r#match: MetadataNumberMatch) -> Result<String, LadybugIndexError> {
    Ok(match r#match {
        MetadataNumberMatch::Eq { value } => format!("{field} = {value}"),
        MetadataNumberMatch::Gt { value } => format!("{field} > {value}"),
        MetadataNumberMatch::Gte { value } => format!("{field} >= {value}"),
        MetadataNumberMatch::Lt { value } => format!("{field} < {value}"),
        MetadataNumberMatch::Lte { value } => format!("{field} <= {value}"),
        MetadataNumberMatch::Between { min, max } => {
            format!("{field} >= {min} AND {field} <= {max}")
        }
        MetadataNumberMatch::IsNull => format!("{field} IS NULL"),
        MetadataNumberMatch::IsNotNull => format!("{field} IS NOT NULL"),
    })
}

fn boolean_match(field: &str, r#match: MetadataBooleanMatch) -> Result<String, LadybugIndexError> {
    Ok(match r#match {
        MetadataBooleanMatch::Eq { value } => format!("{field} = {value}"),
        MetadataBooleanMatch::IsNull => format!("{field} IS NULL"),
        MetadataBooleanMatch::IsNotNull => format!("{field} IS NOT NULL"),
    })
}

fn sort_clause(sort: FilteredRecordSort) -> Result<&'static str, LadybugIndexError> {
    match sort {
        FilteredRecordSort::RecordKey => Ok("ORDER BY record.record_key"),
        FilteredRecordSort::Alphabetical => Ok("ORDER BY record.name, record.record_key"),
        FilteredRecordSort::LevelAsc => Ok("ORDER BY record.level, record.name, record.record_key"),
        FilteredRecordSort::LevelDesc => {
            Ok("ORDER BY record.level DESC, record.name, record.record_key")
        }
        FilteredRecordSort::Random { .. } => unsupported("random sorting"),
        FilteredRecordSort::PriceAsc | FilteredRecordSort::PriceDesc => {
            unsupported("price sorting")
        }
    }
}

fn record_from_row(row: &[Value]) -> Result<PersistedRecord, LadybugIndexError> {
    let key = record_key_at(row, 0)?;
    let id = RecordId::new(string_at(row, 1)?).map_err(invalid)?;
    let record_family = string_at(row, 4)?
        .parse::<RecordFamily>()
        .map_err(invalid)?;
    let pack_name = PackName::new(string_at(row, 5)?).map_err(invalid)?;
    let publication_family = PublicationFamily::from_canonical(&string_at(row, 32)?)
        .unwrap_or(PublicationFamily::Unknown);
    let actor_data = optional_actor_data(row)?;
    let item_data = optional_item_data(row)?;
    let spell_data = optional_spell_data(row)?;
    Ok(PersistedRecord {
        key,
        id,
        name: string_at(row, 2)?,
        normalized_name: string_at(row, 3)?,
        record_family,
        pack_name,
        pack_label: string_at(row, 6)?,
        foundry_document_type: string_at(row, 7)?,
        foundry_record_type: string_at(row, 8)?,
        level: optional_i64_at(row, 9)?,
        rarity: optional_string_at(row, 10)?,
        traits: json_string_array_at(row, 11)?,
        prerequisites: json_string_array_at(row, 12)?,
        system_category: optional_string_at(row, 13)?,
        system_group: optional_string_at(row, 14)?,
        system_base_item: optional_string_at(row, 15)?,
        system_usage: optional_string_at(row, 16)?,
        system_price_json: optional_string_at(row, 17)?,
        system_actions_value: optional_i64_at(row, 18)?,
        system_time_value: optional_string_at(row, 19)?,
        system_duration_value: optional_string_at(row, 20)?,
        price_cp: optional_i64_at(row, 21)?,
        activation_time: normalized_time_at(row, 22, 23, 24, 25, 26)?,
        duration: normalized_time_at(row, 27, usize::MAX, 28, 29, 30)?,
        metrics: Vec::new(),
        actor_data,
        item_data,
        spell_data,
        publication_title: optional_string_at(row, 31)?,
        publication_remaster: bool_at(row, 33)?,
        description: optional_content_document_at(row, 34)?,
        blurb: optional_content_document_at(row, 35)?,
        supplemental_content: Vec::new(),
        publication_family,
        folder_id: optional_string_at(row, 36)?,
        taxonomy_families: json_string_array_at(row, 37)?,
        variant_group_key: optional_string_at(row, 38)?,
        variant_base_name: optional_string_at(row, 39)?,
        variant_label: optional_string_at(row, 40)?,
        variant_axes: json_string_array_at(row, 41)?,
        variant_confidence: optional_float_at(row, 42)?,
        variant_source: string_at(row, 43)?,
        is_default_visible: bool_at(row, 44)?,
        source_path: string_at(row, 45)?,
        raw_json: string_at(row, 46)?,
    })
}

fn alias_from_row(row: &[Value]) -> Result<RecordAlias, LadybugIndexError> {
    Ok(RecordAlias {
        canonical_record_key: record_key_at(row, 0)?,
        alias_text: string_at(row, 1)?,
        normalized_alias: string_at(row, 2)?,
        source: AliasSource::from_canonical(&string_at(row, 3)?)
            .unwrap_or(AliasSource::CompendiumSource),
        source_ref: string_at(row, 4)?,
    })
}

fn graph_edge_from_row(
    row: &[Value],
    from_index: usize,
    to_index: usize,
) -> Result<GraphReferenceEdge, LadybugIndexError> {
    let source_kind = ContentSourceKind::from_canonical(&string_at(row, 4)?)
        .unwrap_or(ContentSourceKind::Description);
    let visibility =
        ContentVisibility::from_canonical(&string_at(row, 5)?).unwrap_or(ContentVisibility::Public);
    Ok(GraphReferenceEdge {
        from_record_key: record_key_at(row, from_index)?,
        to_record_key: record_key_at(row, to_index)?,
        display_text: optional_string_at(row, 2)?,
        reference_text: string_at(row, 3)?,
        source_kind,
        visibility,
    })
}

fn reference_edge_from_row(row: &[Value]) -> Result<ReferenceEdge, LadybugIndexError> {
    Ok(ReferenceEdge {
        from_record_key: record_key_at(row, 0)?,
        to_record_key: record_key_at(row, 1)?,
        display_text: optional_string_at(row, 2)?,
        reference_text: string_at(row, 3)?,
        source_kind: ContentSourceKind::from_canonical(&string_at(row, 4)?)
            .unwrap_or(ContentSourceKind::Description),
        visibility: ContentVisibility::from_canonical(&string_at(row, 5)?)
            .unwrap_or(ContentVisibility::Public),
    })
}

fn remaster_link_from_row(row: &[Value]) -> Result<RemasterLink, LadybugIndexError> {
    Ok(RemasterLink {
        remaster_record_key: record_key_at(row, 0)?,
        legacy_record_key: record_key_at(row, 1)?,
        source: RemasterLinkSource::from_canonical(&string_at(row, 2)?)
            .unwrap_or(RemasterLinkSource::Migration),
        source_ref: string_at(row, 3)?,
    })
}

fn vector_hit_from_row(row: &[Value]) -> Result<VectorSearchHit, LadybugIndexError> {
    Ok(VectorSearchHit {
        record_key: string_at(row, 0)?,
        embedding_unit_key: string_at(row, 1)?,
        unit_kind: string_at(row, 2)?,
        label: optional_string_at(row, 3)?,
        distance: float_at(row, 4)?,
    })
}

fn query_rows(
    connection: &Connection<'_>,
    sql: &str,
) -> Result<Vec<Vec<Value>>, LadybugIndexError> {
    let mut result = connection
        .query(sql)
        .map_err(|error| LadybugIndexError::Query(format!("{error}; query: {sql}")))?;
    let mut rows = Vec::new();
    for row in &mut result {
        rows.push(row.to_vec());
    }
    Ok(rows)
}

fn record_key_at(row: &[Value], index: usize) -> Result<RecordKey, LadybugIndexError> {
    RecordKey::parse(&string_at(row, index)?).map_err(invalid)
}

fn string_at(row: &[Value], index: usize) -> Result<String, LadybugIndexError> {
    match row.get(index) {
        Some(Value::String(value)) => Ok(value.clone()),
        Some(value) => Err(LadybugIndexError::InvalidData(format!(
            "expected string at column {index}, got {value}"
        ))),
        None => Err(LadybugIndexError::InvalidData(format!(
            "missing column {index}"
        ))),
    }
}

fn optional_string_at(row: &[Value], index: usize) -> Result<Option<String>, LadybugIndexError> {
    match row.get(index) {
        Some(Value::Null(_)) => Ok(None),
        Some(Value::String(value)) => Ok(Some(value.clone())),
        Some(value) => Err(LadybugIndexError::InvalidData(format!(
            "expected optional string at column {index}, got {value}"
        ))),
        None => Err(LadybugIndexError::InvalidData(format!(
            "missing column {index}"
        ))),
    }
}

fn int_at(row: &[Value], index: usize) -> Result<i64, LadybugIndexError> {
    match row.get(index) {
        Some(Value::Int64(value)) => Ok(*value),
        Some(Value::Int32(value)) => Ok(i64::from(*value)),
        Some(value) => Err(LadybugIndexError::InvalidData(format!(
            "expected integer at column {index}, got {value}"
        ))),
        None => Err(LadybugIndexError::InvalidData(format!(
            "missing column {index}"
        ))),
    }
}

fn u64_at(row: &[Value], index: usize) -> Result<u64, LadybugIndexError> {
    let value = int_at(row, index)?;
    u64::try_from(value).map_err(|_| {
        LadybugIndexError::InvalidData(format!(
            "expected non-negative integer at column {index}, got {value}"
        ))
    })
}

fn optional_i64_at(row: &[Value], index: usize) -> Result<Option<i64>, LadybugIndexError> {
    if index == usize::MAX {
        return Ok(None);
    }
    match row.get(index) {
        Some(Value::Null(_)) => Ok(None),
        Some(_) => int_at(row, index).map(Some),
        None => Err(LadybugIndexError::InvalidData(format!(
            "missing column {index}"
        ))),
    }
}

fn optional_bool_at(row: &[Value], index: usize) -> Result<Option<bool>, LadybugIndexError> {
    match row.get(index) {
        Some(Value::Null(_)) => Ok(None),
        Some(_) => bool_at(row, index).map(Some),
        None => Err(LadybugIndexError::InvalidData(format!(
            "missing column {index}"
        ))),
    }
}

fn bool_at(row: &[Value], index: usize) -> Result<bool, LadybugIndexError> {
    match row.get(index) {
        Some(Value::Bool(value)) => Ok(*value),
        Some(value) => Err(LadybugIndexError::InvalidData(format!(
            "expected bool at column {index}, got {value}"
        ))),
        None => Err(LadybugIndexError::InvalidData(format!(
            "missing column {index}"
        ))),
    }
}

fn float_at(row: &[Value], index: usize) -> Result<f64, LadybugIndexError> {
    match row.get(index) {
        Some(Value::Double(value)) => Ok(*value),
        Some(Value::Float(value)) => Ok(f64::from(*value)),
        Some(Value::Int64(value)) => Ok(*value as f64),
        Some(Value::Int32(value)) => Ok(f64::from(*value)),
        Some(value) => Err(LadybugIndexError::InvalidData(format!(
            "expected float at column {index}, got {value}"
        ))),
        None => Err(LadybugIndexError::InvalidData(format!(
            "missing column {index}"
        ))),
    }
}

fn value_to_discovery_string(row: &[Value], index: usize) -> Result<String, LadybugIndexError> {
    match row.get(index) {
        Some(Value::String(value)) => Ok(value.clone()),
        Some(Value::Bool(value)) => Ok(value.to_string()),
        Some(Value::Int64(value)) => Ok(value.to_string()),
        Some(Value::Int32(value)) => Ok(value.to_string()),
        Some(Value::Double(value)) => Ok(value.to_string()),
        Some(Value::Float(value)) => Ok(value.to_string()),
        Some(value) => Err(LadybugIndexError::InvalidData(format!(
            "expected scalar discovery value at column {index}, got {value}"
        ))),
        None => Err(LadybugIndexError::InvalidData(format!(
            "missing column {index}"
        ))),
    }
}

fn discovery_error(error: LadybugIndexError) -> DiscoveryError {
    DiscoveryError::QueryFailed(error.to_string())
}

fn unknown_discovery_field(field: &str) -> DiscoveryError {
    let suggestion = match field {
        "packs" | "pack" => " Did you mean `pack_name` or `pack_label`?",
        "sources" | "source" => " Did you mean `publication_title`?",
        "actorMetrics" | "itemMetrics" | "actor_metrics" | "item_metrics" => {
            " Did you mean `metric`?"
        }
        _ => "",
    };
    DiscoveryError::InvalidField(format!(
        "unknown filter field `{field}`.{suggestion} Run `atlas filters fields` to discover supported fields."
    ))
}

fn filter_field_stats(values: &[FilterValueCount]) -> FilterFieldStats {
    let value_count = values.iter().map(|value| value.count).sum::<u64>();
    let distinct_count = values.len() as u64;
    let singleton_count = values.iter().filter(|value| value.count == 1).count() as u64;
    FilterFieldStats {
        value_count,
        null_count: 0,
        distinct_count,
        singleton_count,
        singleton_ratio: if distinct_count == 0 {
            0.0
        } else {
            singleton_count as f64 / distinct_count as f64
        },
        observation_singleton_ratio: if value_count == 0 {
            0.0
        } else {
            singleton_count as f64 / value_count as f64
        },
    }
}

fn numeric_stats_from_values(values: &[f64], matching_record_count: u64) -> NumericFieldStats {
    let count = values.len() as u64;
    NumericFieldStats {
        count,
        null_count: matching_record_count.saturating_sub(count),
        min: values.first().copied(),
        p05: percentile(values, 0.05),
        p25: percentile(values, 0.25),
        p50: percentile(values, 0.50),
        mean: (!values.is_empty()).then(|| values.iter().sum::<f64>() / values.len() as f64),
        p75: percentile(values, 0.75),
        p95: percentile(values, 0.95),
        max: values.last().copied(),
    }
}

fn percentile(values: &[f64], percentile: f64) -> Option<f64> {
    if values.is_empty() {
        return None;
    }
    let rank = ((values.len() as f64) * percentile).ceil() as usize;
    let index = rank.saturating_sub(1).min(values.len() - 1);
    values.get(index).copied()
}

fn resolve_ladybug_metric_from_candidates(
    metrics: Vec<MetricKeyDiscovery>,
    value: &str,
) -> Result<MetricKeyDiscovery, DiscoveryError> {
    let key_matches = metrics
        .iter()
        .filter(|metric| metric.metric_key == value)
        .cloned()
        .collect::<Vec<_>>();
    match key_matches.as_slice() {
        [metric] => return Ok(metric.clone()),
        [] => {}
        _ => {
            return Err(DiscoveryError::AmbiguousMetric(format!(
                "metric key `{value}` is ambiguous; candidates: {}",
                ladybug_metric_candidates(&key_matches)
            )));
        }
    }

    let normalized = normalize_ladybug_metric_label(value);
    let label_matches = metrics
        .into_iter()
        .filter(|metric| {
            metric.known
                && (ladybug_metric_label_matches(metric.label.as_deref(), &normalized)
                    || ladybug_metric_label_matches(metric.short_label.as_deref(), &normalized))
        })
        .collect::<Vec<_>>();
    match label_matches.as_slice() {
        [metric] => Ok(metric.clone()),
        [] => Err(DiscoveryError::InvalidOption(format!(
            "metric `{value}` did not match a metric key, exact known label, or exact known short label"
        ))),
        _ => Err(DiscoveryError::AmbiguousMetric(format!(
            "metric label `{value}` is ambiguous; candidates: {}",
            ladybug_metric_candidates(&label_matches)
        ))),
    }
}

fn ladybug_metric_candidates(metrics: &[MetricKeyDiscovery]) -> String {
    metrics
        .iter()
        .map(|metric| {
            format!(
                "{} ({}, {}, {})",
                metric.metric_key, metric.metric_domain, metric.record_family, metric.value_type
            )
        })
        .collect::<Vec<_>>()
        .join(", ")
}

fn ladybug_metric_label_matches(value: Option<&str>, normalized: &str) -> bool {
    value
        .map(normalize_ladybug_metric_label)
        .is_some_and(|label| label == normalized)
}

fn normalize_ladybug_metric_label(value: &str) -> String {
    value
        .chars()
        .flat_map(char::to_lowercase)
        .filter(|character| character.is_ascii_alphanumeric())
        .collect()
}

fn optional_float_at(row: &[Value], index: usize) -> Result<Option<f64>, LadybugIndexError> {
    match row.get(index) {
        Some(Value::Null(_)) => Ok(None),
        Some(_) => float_at(row, index).map(Some),
        None => Err(LadybugIndexError::InvalidData(format!(
            "missing column {index}"
        ))),
    }
}

fn json_string_array_at(row: &[Value], index: usize) -> Result<Vec<String>, LadybugIndexError> {
    let json = string_at(row, index)?;
    serde_json::from_str::<Vec<String>>(&json).map_err(invalid)
}

fn optional_content_document_at(
    row: &[Value],
    index: usize,
) -> Result<Option<ContentDocument>, LadybugIndexError> {
    optional_string_at(row, index)?
        .map(|json| serde_json::from_str::<ContentDocument>(&json).map_err(invalid))
        .transpose()
}

fn normalized_time_at(
    row: &[Value],
    kind_index: usize,
    actions_index: usize,
    duration_value_index: usize,
    duration_unit_index: usize,
    text_index: usize,
) -> Result<Option<NormalizedTime>, LadybugIndexError> {
    let Some(kind) = optional_string_at(row, kind_index)? else {
        return Ok(None);
    };
    let Some(text) = optional_string_at(row, text_index)? else {
        return Ok(None);
    };
    Ok(Some(NormalizedTime {
        kind: TimeKind::from_canonical(&kind).unwrap_or(TimeKind::Other),
        actions: optional_i64_at(row, actions_index)?,
        duration_value: optional_i64_at(row, duration_value_index)?,
        duration_unit: optional_string_at(row, duration_unit_index)?
            .and_then(|value| TimeUnit::from_canonical(&value)),
        text,
    }))
}

fn optional_actor_data(row: &[Value]) -> Result<Option<ActorSideData>, LadybugIndexError> {
    if optional_string_at(row, 47)?.is_none()
        && optional_string_at(row, 48)?.is_none()
        && optional_string_at(row, 54)?.is_none()
        && optional_bool_at(row, 56)?.is_none()
    {
        return Ok(None);
    }
    Ok(Some(ActorSideData {
        size: optional_string_at(row, 47)?,
        languages: optional_json_string_array_at(row, 48)?,
        speed_types: optional_json_string_array_at(row, 49)?,
        senses: optional_json_string_array_at(row, 50)?,
        immunities: optional_json_string_array_at(row, 51)?,
        resistances: optional_json_string_array_at(row, 52)?,
        weaknesses: optional_json_string_array_at(row, 53)?,
        disable_text: optional_string_at(row, 54)?,
        disable_skills: optional_json_string_array_at(row, 55)?,
        is_complex: optional_bool_at(row, 56)?.unwrap_or(false),
    }))
}

fn optional_item_data(row: &[Value]) -> Result<Option<ItemSideData>, LadybugIndexError> {
    if optional_float_at(row, 57)?.is_none()
        && optional_string_at(row, 58)?.is_none()
        && optional_string_at(row, 59)?.is_none()
    {
        return Ok(None);
    }
    Ok(Some(ItemSideData {
        system_category: optional_string_at(row, 13)?,
        system_base_item: optional_string_at(row, 15)?,
        system_group: optional_string_at(row, 14)?,
        system_usage: optional_string_at(row, 16)?,
        price_cp: optional_i64_at(row, 21)?,
        bulk_value: optional_float_at(row, 57)?,
        hands_requirement: optional_string_at(row, 58)?,
        damage_types: optional_json_string_array_at(row, 59)?,
    }))
}

fn optional_spell_data(row: &[Value]) -> Result<Option<SpellSideData>, LadybugIndexError> {
    if optional_string_at(row, 60)?.is_none()
        && optional_string_at(row, 61)?.is_none()
        && optional_string_at(row, 62)?.is_none()
    {
        return Ok(None);
    }
    Ok(Some(SpellSideData {
        traditions: optional_json_string_array_at(row, 60)?,
        spell_kinds: optional_json_string_array_at(row, 61)?,
        range_text: optional_string_at(row, 62)?,
        range_value: optional_float_at(row, 63)?,
        target_text: optional_string_at(row, 64)?,
        area_type: optional_string_at(row, 65)?,
        area_value: optional_float_at(row, 66)?,
        save_type: optional_string_at(row, 67)?,
        sustained: optional_bool_at(row, 68)?.unwrap_or(false),
        basic_save: optional_bool_at(row, 69)?.unwrap_or(false),
        damage_types: optional_json_string_array_at(row, 70)?,
    }))
}

fn optional_json_string_array_at(
    row: &[Value],
    index: usize,
) -> Result<Vec<String>, LadybugIndexError> {
    optional_string_at(row, index)?
        .map(|json| serde_json::from_str::<Vec<String>>(&json).map_err(invalid))
        .transpose()
        .map(|value| value.unwrap_or_default())
}

fn string_literal(value: &str) -> String {
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

fn unsupported<T>(feature: &str) -> Result<T, LadybugIndexError> {
    Err(LadybugIndexError::Unsupported(format!(
        "{feature} is not implemented in the Ladybug read spike yet"
    )))
}

fn invalid(error: impl std::fmt::Display) -> LadybugIndexError {
    LadybugIndexError::InvalidData(error.to_string())
}

fn search_error(error: LadybugIndexError) -> SearchError {
    match error {
        LadybugIndexError::Unsupported(message) => SearchError::InvalidSearchOptions(message),
        LadybugIndexError::InvalidData(message) | LadybugIndexError::Query(message) => {
            SearchError::Embedding(message)
        }
    }
}
