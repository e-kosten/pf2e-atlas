use crate::{
    DiscoveryError, DiscoveryValueSort, FilterValueRequest, FilteredRecordKeyPage,
    FilteredRecordSort,
};
use atlas_discovery::{all_discovery_field_definitions, metric_filter_field_info};
use atlas_domain::{
    BooleanFieldCounts, FilterDiscoveryExecution, FilterFieldDiscovery, FilterFieldInfo,
    FilterSample, FilterSampleExample, FilterValueCount, FilterValueDiscovery, FilterValuePayload,
    FilterValuePolicy, MetricKeyDiscovery, MetricValuePayload, NumericFieldStats, SearchFilterNode,
};

use super::filter::{
    LadybugValueProjection, ProjectionValueKind, compile_scope, ladybug_projection, sort_clause,
};
use super::row::{
    bool_at, discovery_error, filter_field_stats, float_at, int_at, numeric_stats_from_values,
    optional_string_at, query_rows, record_key_at, resolve_ladybug_metric_from_candidates,
    string_at, u64_at, unknown_discovery_field, value_to_discovery_string,
};
use super::{LadybugIndexReader, LadybugIndexReaderError, string_literal};

impl LadybugIndexReader {
    pub(crate) fn list_filter_fields_impl(
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

    pub(crate) fn list_filter_values_impl(
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

    pub(crate) fn list_filtered_record_keys_impl(
        &self,
        filter: Option<&SearchFilterNode>,
        sort: FilteredRecordSort,
        limit: u32,
        offset: u32,
    ) -> Result<FilteredRecordKeyPage, LadybugIndexReaderError> {
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
}
