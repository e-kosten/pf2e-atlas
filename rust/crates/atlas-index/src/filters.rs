use atlas_artifact::schema::{
    Column, Table, actor_records, item_records, record_metrics, record_traits, records,
    reference_edges, spell_records,
};
use atlas_domain::metadata::{
    BooleanOperator, CollectionOperator, MetadataBooleanField, MetadataEnumStringField,
    MetadataNumberField, MetadataPredicate, MetadataSetField, MetadataTextStringField,
    NumberOperator, StringOperator, TextOperator,
};
use atlas_domain::{
    MetricOperator, NullableNumericMatch, NullableStringMatch, NumericMatch, ScalarValue,
    SearchFilterNode,
};
use rusqlite::types::Value;
use thiserror::Error;

#[derive(Debug, Clone, PartialEq)]
pub struct EligibleRecordsQuery {
    pub sql: String,
    pub parameters: Vec<Value>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct FilteredRecordKeysQuery {
    pub sql: String,
    pub parameters: Vec<Value>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FilteredRecordSort {
    RecordKeyAsc,
    NameAsc,
    LevelAsc,
    LevelDesc,
}

#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum FilterCompileError {
    #[error("filter `{filter}` is not supported by the SQL keyset compiler")]
    Unsupported { filter: String },
    #[error("filter `{filter}` is missing required value `{value}`")]
    MissingValue { filter: String, value: String },
}

pub fn compile_eligible_records_query(
    filter: Option<&SearchFilterNode>,
) -> Result<EligibleRecordsQuery, FilterCompileError> {
    let mut compiler = FilterCompiler::default();
    let base = format!(
        "SELECT {record_key} FROM {records_table} {records_alias} WHERE {default_visible} = 1",
        record_key = record_column(records::columns::RECORD_KEY),
        records_table = records::TABLE.name(),
        records_alias = RECORDS_ALIAS,
        default_visible = record_column(records::columns::IS_DEFAULT_VISIBLE),
    );
    let sql = match filter {
        Some(filter) => format!("{base} AND ({})", compiler.compile_node(filter)?),
        None => base,
    };

    Ok(EligibleRecordsQuery {
        sql,
        parameters: compiler.parameters,
    })
}

pub fn compile_filtered_record_keys_query(
    filter: Option<&SearchFilterNode>,
    sort: FilteredRecordSort,
    limit: Option<u32>,
    offset: Option<u32>,
) -> Result<FilteredRecordKeysQuery, FilterCompileError> {
    let eligible = compile_eligible_records_query(filter)?;
    Ok(eligible.into_record_keys_query(sort, limit, offset))
}

impl EligibleRecordsQuery {
    pub fn into_record_keys_query(
        self,
        sort: FilteredRecordSort,
        limit: Option<u32>,
        offset: Option<u32>,
    ) -> FilteredRecordKeysQuery {
        let mut parameters = self.parameters;
        let mut sql = format!(
            "WITH eligible(record_key) AS ({})
             SELECT {}
             FROM eligible e
             JOIN {} {} ON {} = e.record_key
             ORDER BY {}",
            self.sql,
            record_column(records::columns::RECORD_KEY),
            records::TABLE.name(),
            RECORDS_ALIAS,
            record_column(records::columns::RECORD_KEY),
            sort.sql()
        );

        match (limit, offset) {
            (Some(limit), Some(offset)) => {
                let limit_placeholder = push_integer_parameter(&mut parameters, limit);
                let offset_placeholder = push_integer_parameter(&mut parameters, offset);
                sql.push_str(&format!(
                    " LIMIT {limit_placeholder} OFFSET {offset_placeholder}"
                ));
            }
            (Some(limit), None) => {
                let limit_placeholder = push_integer_parameter(&mut parameters, limit);
                sql.push_str(&format!(" LIMIT {limit_placeholder}"));
            }
            (None, Some(offset)) => {
                let offset_placeholder = push_integer_parameter(&mut parameters, offset);
                sql.push_str(&format!(" LIMIT -1 OFFSET {offset_placeholder}"));
            }
            (None, None) => {}
        }

        FilteredRecordKeysQuery { sql, parameters }
    }
}

impl FilteredRecordSort {
    fn sql(self) -> String {
        match self {
            Self::RecordKeyAsc => format!("{} ASC", record_column(records::columns::RECORD_KEY)),
            Self::NameAsc => format!(
                "{} ASC, {} ASC",
                record_column(records::columns::NORMALIZED_NAME),
                record_column(records::columns::RECORD_KEY)
            ),
            Self::LevelAsc => format!(
                "{} IS NULL ASC, {} ASC, {} ASC, {} ASC",
                record_column(records::columns::LEVEL),
                record_column(records::columns::LEVEL),
                record_column(records::columns::NORMALIZED_NAME),
                record_column(records::columns::RECORD_KEY)
            ),
            Self::LevelDesc => format!(
                "{} IS NULL ASC, {} DESC, {} ASC, {} ASC",
                record_column(records::columns::LEVEL),
                record_column(records::columns::LEVEL),
                record_column(records::columns::NORMALIZED_NAME),
                record_column(records::columns::RECORD_KEY)
            ),
        }
    }
}

#[derive(Default)]
struct FilterCompiler {
    parameters: Vec<Value>,
}

impl FilterCompiler {
    fn compile_node(&mut self, filter: &SearchFilterNode) -> Result<String, FilterCompileError> {
        match filter {
            SearchFilterNode::Pack { value } => Ok(format!(
                "{} = {}",
                record_column(records::columns::PACK_NAME),
                self.text(value)
            )),
            SearchFilterNode::RecordFamily { value } => Ok(format!(
                "{} = {}",
                record_column(records::columns::RECORD_FAMILY),
                self.text(value.as_str())
            )),
            SearchFilterNode::Level { r#match } => {
                self.numeric_match(&record_column(records::columns::LEVEL), *r#match)
            }
            SearchFilterNode::Price { r#match } => {
                self.numeric_match(&record_column(records::columns::PRICE_CP), *r#match)
            }
            SearchFilterNode::Rarity { r#match } => {
                self.nullable_string_match(&record_column(records::columns::RARITY), r#match)
            }
            SearchFilterNode::ActionCost { r#match } => self.nullable_numeric_match(
                &record_column(records::columns::ACTIVATION_TIME_ACTIONS),
                *r#match,
            ),
            SearchFilterNode::LinksTo { target } => Ok(format!(
                "EXISTS (SELECT 1 FROM {table} {alias} WHERE {from_key} = {record_key} AND {to_key} = {})",
                self.text(&target.to_string()),
                table = reference_edges::TABLE.name(),
                alias = REFERENCE_EDGES_ALIAS,
                from_key = aliased_column(
                    REFERENCE_EDGES_ALIAS,
                    reference_edges::columns::FROM_RECORD_KEY
                ),
                record_key = record_column(records::columns::RECORD_KEY),
                to_key = aliased_column(
                    REFERENCE_EDGES_ALIAS,
                    reference_edges::columns::TO_RECORD_KEY
                ),
            )),
            SearchFilterNode::LinkedFrom { source } => Ok(format!(
                "EXISTS (SELECT 1 FROM {table} {alias} WHERE {from_key} = {} AND {to_key} = {record_key})",
                self.text(&source.to_string()),
                table = reference_edges::TABLE.name(),
                alias = REFERENCE_EDGES_ALIAS,
                from_key = aliased_column(
                    REFERENCE_EDGES_ALIAS,
                    reference_edges::columns::FROM_RECORD_KEY
                ),
                to_key = aliased_column(
                    REFERENCE_EDGES_ALIAS,
                    reference_edges::columns::TO_RECORD_KEY
                ),
                record_key = record_column(records::columns::RECORD_KEY),
            )),
            SearchFilterNode::MetadataPredicate { predicate } => self.metadata_predicate(predicate),
            SearchFilterNode::Metric { metric, op, value } => {
                self.metric_predicate(metric, *op, value)
            }
            SearchFilterNode::MetricCompare {
                left_metric,
                op,
                right_metric,
            } => Ok(format!(
                "EXISTS (SELECT 1 FROM {table} lm JOIN {table} rm ON {rm_record_key} = {lm_record_key} WHERE {lm_record_key} = {record_key} AND {lm_metric_key} = {} AND {rm_metric_key} = {} AND {lm_value_type} = 'number' AND {rm_value_type} = 'number' AND {lm_number_value} {} {rm_number_value})",
                self.text(left_metric),
                self.text(right_metric),
                metric_operator_sql(*op),
                table = record_metrics::TABLE.name(),
                lm_record_key = aliased_column("lm", record_metrics::columns::RECORD_KEY),
                rm_record_key = aliased_column("rm", record_metrics::columns::RECORD_KEY),
                record_key = record_column(records::columns::RECORD_KEY),
                lm_metric_key = aliased_column("lm", record_metrics::columns::METRIC_KEY),
                rm_metric_key = aliased_column("rm", record_metrics::columns::METRIC_KEY),
                lm_value_type = aliased_column("lm", record_metrics::columns::VALUE_TYPE),
                rm_value_type = aliased_column("rm", record_metrics::columns::VALUE_TYPE),
                lm_number_value = aliased_column("lm", record_metrics::columns::NUMBER_VALUE),
                rm_number_value = aliased_column("rm", record_metrics::columns::NUMBER_VALUE),
            )),
            SearchFilterNode::AnyOf { children } => self.boolean_group(children, " OR ", "0"),
            SearchFilterNode::AllOf { children } => self.boolean_group(children, " AND ", "1"),
            SearchFilterNode::Not { child } => Ok(format!("NOT ({})", self.compile_node(child)?)),
        }
    }

    fn boolean_group(
        &mut self,
        children: &[SearchFilterNode],
        joiner: &str,
        empty_sql: &str,
    ) -> Result<String, FilterCompileError> {
        if children.is_empty() {
            return Ok(empty_sql.to_string());
        }
        let compiled = children
            .iter()
            .map(|child| self.compile_node(child).map(|sql| format!("({sql})")))
            .collect::<Result<Vec<_>, _>>()?;
        Ok(compiled.join(joiner))
    }

    fn metadata_predicate(
        &mut self,
        predicate: &MetadataPredicate,
    ) -> Result<String, FilterCompileError> {
        match predicate {
            MetadataPredicate::Set { field, op, value } => {
                self.metadata_set_predicate(*field, *op, value.as_deref())
            }
            MetadataPredicate::EnumString {
                field,
                op,
                value,
                values,
            } => self.metadata_enum_predicate(*field, *op, value.as_deref(), values.as_deref()),
            MetadataPredicate::Text { field, op, value } => {
                self.metadata_text_predicate(*field, *op, value.as_deref())
            }
            MetadataPredicate::Number {
                field,
                op,
                value,
                min,
                max,
            } => self.metadata_number_predicate(*field, *op, *value, *min, *max),
            MetadataPredicate::Boolean { field, op, value } => {
                self.metadata_boolean_predicate(*field, *op, *value)
            }
        }
    }

    fn metadata_set_predicate(
        &mut self,
        field: MetadataSetField,
        op: CollectionOperator,
        value: Option<&str>,
    ) -> Result<String, FilterCompileError> {
        let set = match field {
            MetadataSetField::Traits => SetStorage::Rows {
                table: record_traits::TABLE,
                key_column: record_traits::columns::RECORD_KEY,
                value_column: record_traits::columns::TRAIT,
            },
            MetadataSetField::TaxonomyFamilies => {
                SetStorage::JsonColumn(records::columns::TAXONOMY_FAMILIES_JSON)
            }
            MetadataSetField::Traditions => SetStorage::JsonSideTable {
                table: spell_records::TABLE,
                column: spell_records::columns::TRADITIONS_JSON,
            },
            MetadataSetField::SpellKinds => SetStorage::JsonSideTable {
                table: spell_records::TABLE,
                column: spell_records::columns::SPELL_KINDS_JSON,
            },
            MetadataSetField::DamageTypes => {
                return self.multi_json_side_table_set(
                    op,
                    value,
                    &[
                        (
                            item_records::TABLE,
                            item_records::columns::DAMAGE_TYPES_JSON,
                        ),
                        (
                            spell_records::TABLE,
                            spell_records::columns::DAMAGE_TYPES_JSON,
                        ),
                    ],
                );
            }
            MetadataSetField::Languages => SetStorage::JsonSideTable {
                table: actor_records::TABLE,
                column: actor_records::columns::LANGUAGES_JSON,
            },
            MetadataSetField::SpeedTypes => SetStorage::JsonSideTable {
                table: actor_records::TABLE,
                column: actor_records::columns::SPEED_TYPES_JSON,
            },
            MetadataSetField::Senses => SetStorage::JsonSideTable {
                table: actor_records::TABLE,
                column: actor_records::columns::SENSES_JSON,
            },
            MetadataSetField::Immunities => SetStorage::JsonSideTable {
                table: actor_records::TABLE,
                column: actor_records::columns::IMMUNITIES_JSON,
            },
            MetadataSetField::Resistances => SetStorage::JsonSideTable {
                table: actor_records::TABLE,
                column: actor_records::columns::RESISTANCES_JSON,
            },
            MetadataSetField::Weaknesses => SetStorage::JsonSideTable {
                table: actor_records::TABLE,
                column: actor_records::columns::WEAKNESSES_JSON,
            },
            MetadataSetField::DisableSkills => SetStorage::JsonSideTable {
                table: actor_records::TABLE,
                column: actor_records::columns::DISABLE_SKILLS_JSON,
            },
            MetadataSetField::VariantAxes => {
                SetStorage::JsonColumn(records::columns::VARIANT_AXES_JSON)
            }
            MetadataSetField::DerivedTags => {
                return Err(FilterCompileError::Unsupported {
                    filter: "metadata.set.derived_tags".to_string(),
                });
            }
        };
        self.set_predicate(set, op, value)
    }

    fn set_predicate(
        &mut self,
        set: SetStorage,
        op: CollectionOperator,
        value: Option<&str>,
    ) -> Result<String, FilterCompileError> {
        match op {
            CollectionOperator::Includes => {
                let value = value.ok_or_else(|| FilterCompileError::MissingValue {
                    filter: "metadata.set.includes".to_string(),
                    value: "value".to_string(),
                })?;
                Ok(match set {
                    SetStorage::Rows {
                        table,
                        key_column,
                        value_column,
                    } => format!(
                        "EXISTS (SELECT 1 FROM {table} s WHERE {key_column} = {record_key} AND {value_column} = {})",
                        self.text(value),
                        table = table.name(),
                        key_column = aliased_column("s", key_column),
                        record_key = record_column(records::columns::RECORD_KEY),
                        value_column = aliased_column("s", value_column),
                    ),
                    SetStorage::JsonColumn(column) => {
                        json_array_contains_sql(&record_column(column), &self.text(value))
                    }
                    SetStorage::JsonSideTable { table, column } => format!(
                        "EXISTS (SELECT 1 FROM {table} s WHERE {side_record_key} = {record_key} AND {})",
                        json_array_contains_sql(&aliased_column("s", column), &self.text(value)),
                        table = table.name(),
                        side_record_key = aliased_column("s", record_key_column(table)),
                        record_key = record_column(records::columns::RECORD_KEY),
                    ),
                })
            }
            CollectionOperator::IsNull => Ok(match set {
                SetStorage::Rows {
                    table, key_column, ..
                } => format!(
                    "NOT EXISTS (SELECT 1 FROM {table} s WHERE {key_column} = {record_key})",
                    table = table.name(),
                    key_column = aliased_column("s", key_column),
                    record_key = record_column(records::columns::RECORD_KEY),
                ),
                SetStorage::JsonColumn(column) => json_array_empty_sql(&record_column(column)),
                SetStorage::JsonSideTable { table, column } => format!(
                    "NOT EXISTS (SELECT 1 FROM {table} s WHERE {side_record_key} = {record_key} AND NOT {})",
                    json_array_empty_sql(&aliased_column("s", column)),
                    table = table.name(),
                    side_record_key = aliased_column("s", record_key_column(table)),
                    record_key = record_column(records::columns::RECORD_KEY),
                ),
            }),
            CollectionOperator::IsNotNull => Ok(match set {
                SetStorage::Rows {
                    table, key_column, ..
                } => {
                    format!(
                        "EXISTS (SELECT 1 FROM {table} s WHERE {key_column} = {record_key})",
                        table = table.name(),
                        key_column = aliased_column("s", key_column),
                        record_key = record_column(records::columns::RECORD_KEY),
                    )
                }
                SetStorage::JsonColumn(column) => {
                    format!("NOT ({})", json_array_empty_sql(&record_column(column)))
                }
                SetStorage::JsonSideTable { table, column } => format!(
                    "EXISTS (SELECT 1 FROM {table} s WHERE {side_record_key} = {record_key} AND NOT {})",
                    json_array_empty_sql(&aliased_column("s", column)),
                    table = table.name(),
                    side_record_key = aliased_column("s", record_key_column(table)),
                    record_key = record_column(records::columns::RECORD_KEY),
                ),
            }),
        }
    }

    fn multi_json_side_table_set(
        &mut self,
        op: CollectionOperator,
        value: Option<&str>,
        tables: &[(Table, Column)],
    ) -> Result<String, FilterCompileError> {
        let predicates = tables
            .iter()
            .map(|(table, column)| {
                self.set_predicate(
                    SetStorage::JsonSideTable {
                        table: *table,
                        column: *column,
                    },
                    op,
                    value,
                )
                .map(|sql| format!("({sql})"))
            })
            .collect::<Result<Vec<_>, _>>()?;
        let joiner = match op {
            CollectionOperator::IsNull => " AND ",
            CollectionOperator::Includes | CollectionOperator::IsNotNull => " OR ",
        };
        Ok(predicates.join(joiner))
    }

    fn metadata_enum_predicate(
        &mut self,
        field: MetadataEnumStringField,
        op: StringOperator,
        value: Option<&str>,
        values: Option<&[String]>,
    ) -> Result<String, FilterCompileError> {
        let column = match field {
            MetadataEnumStringField::PublicationFamily => records::columns::PUBLICATION_FAMILY,
            MetadataEnumStringField::Size => actor_records::columns::SIZE,
            MetadataEnumStringField::Usage => records::columns::SYSTEM_USAGE,
            MetadataEnumStringField::SystemGroup => records::columns::SYSTEM_GROUP,
            MetadataEnumStringField::FoundryRecordType => records::columns::FOUNDRY_RECORD_TYPE,
            MetadataEnumStringField::BaseItem => records::columns::SYSTEM_BASE_ITEM,
            MetadataEnumStringField::SaveType => spell_records::columns::SAVE_TYPE,
            MetadataEnumStringField::AreaType => spell_records::columns::AREA_TYPE,
            MetadataEnumStringField::DurationUnit => records::columns::DURATION_UNIT,
            MetadataEnumStringField::Rarity => records::columns::RARITY,
            MetadataEnumStringField::VariantGroupKey => records::columns::VARIANT_GROUP_KEY,
        };
        self.with_field_source(column, |compiler, qualified_column| {
            compiler.string_operator(qualified_column, op, value, values)
        })
    }

    fn metadata_text_predicate(
        &mut self,
        field: MetadataTextStringField,
        op: TextOperator,
        value: Option<&str>,
    ) -> Result<String, FilterCompileError> {
        let column = match field {
            MetadataTextStringField::PublicationTitle => records::columns::PUBLICATION_TITLE,
            MetadataTextStringField::RangeText => spell_records::columns::RANGE_TEXT,
            MetadataTextStringField::DurationText => records::columns::DURATION_TEXT,
            MetadataTextStringField::TargetText => spell_records::columns::TARGET_TEXT,
            MetadataTextStringField::DisableText => actor_records::columns::DISABLE_TEXT,
            MetadataTextStringField::VariantBaseName => records::columns::VARIANT_BASE_NAME,
            MetadataTextStringField::VariantLabel => records::columns::VARIANT_LABEL,
        };
        self.with_field_source(column, |compiler, qualified_column| {
            compiler.text_operator(qualified_column, op, value)
        })
    }

    fn metadata_number_predicate(
        &mut self,
        field: MetadataNumberField,
        op: NumberOperator,
        value: Option<f64>,
        min: Option<f64>,
        max: Option<f64>,
    ) -> Result<String, FilterCompileError> {
        let column = match field {
            MetadataNumberField::Level => records::columns::LEVEL,
            MetadataNumberField::PriceCp => records::columns::PRICE_CP,
            MetadataNumberField::BulkValue => item_records::columns::BULK_VALUE,
            MetadataNumberField::ActionCost => records::columns::ACTIVATION_TIME_ACTIONS,
            MetadataNumberField::RangeValue => spell_records::columns::RANGE_VALUE,
            MetadataNumberField::AreaValue => spell_records::columns::AREA_VALUE,
            MetadataNumberField::Hands => {
                return Err(FilterCompileError::Unsupported {
                    filter: "metadata.number.hands".to_string(),
                });
            }
        };
        self.with_field_source(column, |compiler, qualified_column| {
            compiler.number_operator(qualified_column, op, value, min, max)
        })
    }

    fn metadata_boolean_predicate(
        &mut self,
        field: MetadataBooleanField,
        op: BooleanOperator,
        value: Option<bool>,
    ) -> Result<String, FilterCompileError> {
        let column = match field {
            MetadataBooleanField::PublicationRemaster => records::columns::PUBLICATION_REMASTER,
            MetadataBooleanField::Sustained => spell_records::columns::SUSTAINED,
            MetadataBooleanField::BasicSave => spell_records::columns::BASIC_SAVE,
            MetadataBooleanField::IsComplex => actor_records::columns::IS_COMPLEX,
        };
        self.with_field_source(column, |compiler, qualified_column| {
            compiler.boolean_operator(qualified_column, op, value)
        })
    }

    fn with_field_source(
        &mut self,
        column: Column,
        compile: impl FnOnce(&mut Self, &str) -> Result<String, FilterCompileError>,
    ) -> Result<String, FilterCompileError> {
        if let Some((alias, table)) = side_table_for_column(column) {
            let qualified_column = aliased_column(alias, column);
            let predicate = compile(self, &qualified_column)?;
            Ok(format!(
                "EXISTS (SELECT 1 FROM {table} {alias} WHERE {side_record_key} = {record_key} AND {predicate})",
                table = table.name(),
                side_record_key = aliased_column(alias, record_key_column(table)),
                record_key = record_column(records::columns::RECORD_KEY),
            ))
        } else {
            compile(self, &record_column(column))
        }
    }

    fn metric_predicate(
        &mut self,
        metric: &str,
        op: MetricOperator,
        value: &ScalarValue,
    ) -> Result<String, FilterCompileError> {
        let metric_param = self.text(metric);
        let (value_type, column, value_param) = match value {
            ScalarValue::String(value) => (
                "text",
                aliased_column("m", record_metrics::columns::TEXT_VALUE),
                self.text(value),
            ),
            ScalarValue::Number(value) => (
                "number",
                aliased_column("m", record_metrics::columns::NUMBER_VALUE),
                self.number(*value),
            ),
            ScalarValue::Boolean(value) => (
                "boolean",
                aliased_column("m", record_metrics::columns::BOOL_VALUE),
                self.boolean(*value),
            ),
        };
        Ok(format!(
            "EXISTS (SELECT 1 FROM {table} m WHERE {metric_record_key} = {record_key} AND {metric_key} = {metric_param} AND {metric_value_type} = '{value_type}' AND {column} {} {value_param})",
            metric_operator_sql(op),
            table = record_metrics::TABLE.name(),
            metric_record_key = aliased_column("m", record_metrics::columns::RECORD_KEY),
            record_key = record_column(records::columns::RECORD_KEY),
            metric_key = aliased_column("m", record_metrics::columns::METRIC_KEY),
            metric_value_type = aliased_column("m", record_metrics::columns::VALUE_TYPE),
        ))
    }

    fn numeric_match(
        &mut self,
        column: &str,
        r#match: NumericMatch,
    ) -> Result<String, FilterCompileError> {
        match r#match {
            NumericMatch::Eq { value } => Ok(format!("{column} = {}", self.number(value))),
            NumericMatch::Gt { value } => Ok(format!("{column} > {}", self.number(value))),
            NumericMatch::Gte { value } => Ok(format!("{column} >= {}", self.number(value))),
            NumericMatch::Lt { value } => Ok(format!("{column} < {}", self.number(value))),
            NumericMatch::Lte { value } => Ok(format!("{column} <= {}", self.number(value))),
            NumericMatch::Between { min, max } => Ok(format!(
                "{column} BETWEEN {} AND {}",
                self.number(min),
                self.number(max)
            )),
        }
    }

    fn nullable_numeric_match(
        &mut self,
        column: &str,
        r#match: NullableNumericMatch,
    ) -> Result<String, FilterCompileError> {
        match r#match {
            NullableNumericMatch::Eq { value } => Ok(format!("{column} = {}", self.number(value))),
            NullableNumericMatch::Gt { value } => Ok(format!("{column} > {}", self.number(value))),
            NullableNumericMatch::Gte { value } => {
                Ok(format!("{column} >= {}", self.number(value)))
            }
            NullableNumericMatch::Lt { value } => Ok(format!("{column} < {}", self.number(value))),
            NullableNumericMatch::Lte { value } => {
                Ok(format!("{column} <= {}", self.number(value)))
            }
            NullableNumericMatch::Between { min, max } => Ok(format!(
                "{column} BETWEEN {} AND {}",
                self.number(min),
                self.number(max)
            )),
            NullableNumericMatch::IsNull => Ok(format!("{column} IS NULL")),
            NullableNumericMatch::IsNotNull => Ok(format!("{column} IS NOT NULL")),
        }
    }

    fn nullable_string_match(
        &mut self,
        column: &str,
        r#match: &NullableStringMatch,
    ) -> Result<String, FilterCompileError> {
        match r#match {
            NullableStringMatch::Eq { value } => Ok(format!("{column} = {}", self.text(value))),
            NullableStringMatch::In { values } => self.in_list(column, values, false),
            NullableStringMatch::NotIn { values } => self.in_list(column, values, true),
            NullableStringMatch::IsNull => Ok(format!("{column} IS NULL")),
            NullableStringMatch::IsNotNull => Ok(format!("{column} IS NOT NULL")),
        }
    }

    fn string_operator(
        &mut self,
        column: &str,
        op: StringOperator,
        value: Option<&str>,
        values: Option<&[String]>,
    ) -> Result<String, FilterCompileError> {
        match op {
            StringOperator::Eq => Ok(format!(
                "{column} = {}",
                self.required_text(value, "metadata.string.eq")?
            )),
            StringOperator::NotEq => Ok(format!(
                "{column} <> {}",
                self.required_text(value, "metadata.string.not_eq")?
            )),
            StringOperator::In => self.in_list(
                column,
                values.ok_or_else(|| FilterCompileError::MissingValue {
                    filter: "metadata.string.in".to_string(),
                    value: "values".to_string(),
                })?,
                false,
            ),
            StringOperator::NotIn => self.in_list(
                column,
                values.ok_or_else(|| FilterCompileError::MissingValue {
                    filter: "metadata.string.not_in".to_string(),
                    value: "values".to_string(),
                })?,
                true,
            ),
            StringOperator::IsNull => Ok(format!("{column} IS NULL")),
            StringOperator::IsNotNull => Ok(format!("{column} IS NOT NULL")),
        }
    }

    fn text_operator(
        &mut self,
        column: &str,
        op: TextOperator,
        value: Option<&str>,
    ) -> Result<String, FilterCompileError> {
        match op {
            TextOperator::Eq => Ok(format!(
                "{column} = {}",
                self.required_text(value, "metadata.text.eq")?
            )),
            TextOperator::NotEq => Ok(format!(
                "{column} <> {}",
                self.required_text(value, "metadata.text.not_eq")?
            )),
            TextOperator::Contains => {
                let value = value.ok_or_else(|| FilterCompileError::MissingValue {
                    filter: "metadata.text.contains".to_string(),
                    value: "value".to_string(),
                })?;
                Ok(format!(
                    "{column} LIKE {} ESCAPE '\\'",
                    self.text(&contains_like_pattern(value))
                ))
            }
            TextOperator::NotContains => {
                let value = value.ok_or_else(|| FilterCompileError::MissingValue {
                    filter: "metadata.text.not_contains".to_string(),
                    value: "value".to_string(),
                })?;
                Ok(format!(
                    "{column} NOT LIKE {} ESCAPE '\\'",
                    self.text(&contains_like_pattern(value))
                ))
            }
            TextOperator::IsNull => Ok(format!("{column} IS NULL")),
            TextOperator::IsNotNull => Ok(format!("{column} IS NOT NULL")),
        }
    }

    fn number_operator(
        &mut self,
        column: &str,
        op: NumberOperator,
        value: Option<f64>,
        min: Option<f64>,
        max: Option<f64>,
    ) -> Result<String, FilterCompileError> {
        match op {
            NumberOperator::Eq => Ok(format!(
                "{column} = {}",
                self.required_number(value, "metadata.number.eq")?
            )),
            NumberOperator::Gt => Ok(format!(
                "{column} > {}",
                self.required_number(value, "metadata.number.gt")?
            )),
            NumberOperator::Gte => Ok(format!(
                "{column} >= {}",
                self.required_number(value, "metadata.number.gte")?
            )),
            NumberOperator::Lt => Ok(format!(
                "{column} < {}",
                self.required_number(value, "metadata.number.lt")?
            )),
            NumberOperator::Lte => Ok(format!(
                "{column} <= {}",
                self.required_number(value, "metadata.number.lte")?
            )),
            NumberOperator::Between => Ok(format!(
                "{column} BETWEEN {} AND {}",
                self.required_number(min, "metadata.number.between")?,
                self.required_number(max, "metadata.number.between")?
            )),
            NumberOperator::IsNull => Ok(format!("{column} IS NULL")),
            NumberOperator::IsNotNull => Ok(format!("{column} IS NOT NULL")),
        }
    }

    fn boolean_operator(
        &mut self,
        column: &str,
        op: BooleanOperator,
        value: Option<bool>,
    ) -> Result<String, FilterCompileError> {
        match op {
            BooleanOperator::Eq => Ok(format!(
                "{column} = {}",
                self.boolean(value.ok_or_else(|| FilterCompileError::MissingValue {
                    filter: "metadata.boolean.eq".to_string(),
                    value: "value".to_string(),
                })?)
            )),
            BooleanOperator::IsNull => Ok(format!("{column} IS NULL")),
            BooleanOperator::IsNotNull => Ok(format!("{column} IS NOT NULL")),
        }
    }

    fn in_list(
        &mut self,
        column: &str,
        values: &[String],
        negated: bool,
    ) -> Result<String, FilterCompileError> {
        if values.is_empty() {
            return Ok(if negated {
                "1".to_string()
            } else {
                "0".to_string()
            });
        }
        let params = values
            .iter()
            .map(|value| self.text(value))
            .collect::<Vec<_>>()
            .join(", ");
        let op = if negated { "NOT IN" } else { "IN" };
        Ok(format!("{column} {op} ({params})"))
    }

    fn required_text(
        &mut self,
        value: Option<&str>,
        filter: &str,
    ) -> Result<String, FilterCompileError> {
        value
            .map(|value| self.text(value))
            .ok_or_else(|| FilterCompileError::MissingValue {
                filter: filter.to_string(),
                value: "value".to_string(),
            })
    }

    fn required_number(
        &mut self,
        value: Option<f64>,
        filter: &str,
    ) -> Result<String, FilterCompileError> {
        value
            .map(|value| self.number(value))
            .ok_or_else(|| FilterCompileError::MissingValue {
                filter: filter.to_string(),
                value: "value".to_string(),
            })
    }

    fn text(&mut self, value: &str) -> String {
        self.push(Value::Text(value.to_string()))
    }

    fn number(&mut self, value: f64) -> String {
        self.push(Value::Real(value))
    }

    fn boolean(&mut self, value: bool) -> String {
        self.push(Value::Integer(if value { 1 } else { 0 }))
    }

    fn push(&mut self, value: Value) -> String {
        self.parameters.push(value);
        format!("?{}", self.parameters.len())
    }
}

#[derive(Clone, Copy)]
enum SetStorage {
    Rows {
        table: Table,
        key_column: Column,
        value_column: Column,
    },
    JsonColumn(Column),
    JsonSideTable {
        table: Table,
        column: Column,
    },
}

const RECORDS_ALIAS: &str = "r";
const REFERENCE_EDGES_ALIAS: &str = "re";

fn side_table_for_column(column: Column) -> Option<(&'static str, Table)> {
    match column.table() {
        table if table == actor_records::TABLE => Some(("a", actor_records::TABLE)),
        table if table == item_records::TABLE => Some(("i", item_records::TABLE)),
        table if table == spell_records::TABLE => Some(("s", spell_records::TABLE)),
        _ => None,
    }
}

fn record_key_column(table: Table) -> Column {
    if table == actor_records::TABLE {
        actor_records::columns::RECORD_KEY
    } else if table == item_records::TABLE {
        item_records::columns::RECORD_KEY
    } else if table == spell_records::TABLE {
        spell_records::columns::RECORD_KEY
    } else if table == record_traits::TABLE {
        record_traits::columns::RECORD_KEY
    } else if table == record_metrics::TABLE {
        record_metrics::columns::RECORD_KEY
    } else {
        records::columns::RECORD_KEY
    }
}

fn record_column(column: Column) -> String {
    aliased_column(RECORDS_ALIAS, column)
}

fn aliased_column(alias: &str, column: Column) -> String {
    format!("{alias}.{}", column.name())
}

fn metric_operator_sql(op: MetricOperator) -> &'static str {
    match op {
        MetricOperator::Eq => "=",
        MetricOperator::NotEq => "<>",
        MetricOperator::Gt => ">",
        MetricOperator::Gte => ">=",
        MetricOperator::Lt => "<",
        MetricOperator::Lte => "<=",
    }
}

fn json_array_contains_sql(column: &str, placeholder: &str) -> String {
    format!(
        "EXISTS (SELECT 1 FROM json_each(COALESCE({column}, '[]')) j WHERE j.value = {placeholder})"
    )
}

fn json_array_empty_sql(column: &str) -> String {
    format!("NOT EXISTS (SELECT 1 FROM json_each(COALESCE({column}, '[]')))")
}

fn push_integer_parameter(parameters: &mut Vec<Value>, value: u32) -> String {
    parameters.push(Value::Integer(i64::from(value)));
    format!("?{}", parameters.len())
}

fn contains_like_pattern(value: &str) -> String {
    let mut pattern = String::with_capacity(value.len() + 2);
    pattern.push('%');
    for character in value.chars() {
        match character {
            '%' | '_' | '\\' => {
                pattern.push('\\');
                pattern.push(character);
            }
            _ => pattern.push(character),
        }
    }
    pattern.push('%');
    pattern
}
