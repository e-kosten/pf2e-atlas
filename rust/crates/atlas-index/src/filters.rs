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
    let base = "SELECT r.record_key FROM records r WHERE r.is_default_visible = 1";
    let sql = match filter {
        Some(filter) => format!("{base} AND ({})", compiler.compile_node(filter)?),
        None => base.to_string(),
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
             SELECT r.record_key
             FROM eligible e
             JOIN records r ON r.record_key = e.record_key
             ORDER BY {}",
            self.sql,
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
    fn sql(self) -> &'static str {
        match self {
            Self::RecordKeyAsc => "r.record_key ASC",
            Self::NameAsc => "r.normalized_name ASC, r.record_key ASC",
            Self::LevelAsc => {
                "r.level IS NULL ASC, r.level ASC, r.normalized_name ASC, r.record_key ASC"
            }
            Self::LevelDesc => {
                "r.level IS NULL ASC, r.level DESC, r.normalized_name ASC, r.record_key ASC"
            }
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
            SearchFilterNode::Pack { value } => Ok(format!("r.pack_name = {}", self.text(value))),
            SearchFilterNode::RecordFamily { value } => {
                Ok(format!("r.record_family = {}", self.text(value.as_str())))
            }
            SearchFilterNode::Level { r#match } => self.numeric_match("r.level", *r#match),
            SearchFilterNode::Price { r#match } => self.numeric_match("r.price_cp", *r#match),
            SearchFilterNode::Rarity { r#match } => self.nullable_string_match("r.rarity", r#match),
            SearchFilterNode::ActionCost { r#match } => {
                self.nullable_numeric_match("r.activation_time_actions", *r#match)
            }
            SearchFilterNode::LinksTo { target } => Ok(format!(
                "EXISTS (SELECT 1 FROM reference_edges re WHERE re.from_record_key = r.record_key AND re.to_record_key = {})",
                self.text(&target.to_string())
            )),
            SearchFilterNode::LinkedFrom { source } => Ok(format!(
                "EXISTS (SELECT 1 FROM reference_edges re WHERE re.from_record_key = {} AND re.to_record_key = r.record_key)",
                self.text(&source.to_string())
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
                "EXISTS (SELECT 1 FROM record_metrics lm JOIN record_metrics rm ON rm.record_key = lm.record_key WHERE lm.record_key = r.record_key AND lm.metric_key = {} AND rm.metric_key = {} AND lm.value_type = 'number' AND rm.value_type = 'number' AND lm.number_value {} rm.number_value)",
                self.text(left_metric),
                self.text(right_metric),
                metric_operator_sql(*op)
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
                table: "record_traits",
                key_column: "record_key",
                value_column: "trait",
            },
            MetadataSetField::TaxonomyFamilies => {
                SetStorage::JsonColumn("r.taxonomy_families_json")
            }
            MetadataSetField::Traditions => SetStorage::JsonSideTable {
                table: "spell_records",
                column: "traditions_json",
            },
            MetadataSetField::SpellKinds => SetStorage::JsonSideTable {
                table: "spell_records",
                column: "spell_kinds_json",
            },
            MetadataSetField::DamageTypes => {
                return self.multi_json_side_table_set(
                    op,
                    value,
                    &[
                        ("item_records", "damage_types_json"),
                        ("spell_records", "damage_types_json"),
                    ],
                );
            }
            MetadataSetField::Languages => SetStorage::JsonSideTable {
                table: "actor_records",
                column: "languages_json",
            },
            MetadataSetField::SpeedTypes => SetStorage::JsonSideTable {
                table: "actor_records",
                column: "speed_types_json",
            },
            MetadataSetField::Senses => SetStorage::JsonSideTable {
                table: "actor_records",
                column: "senses_json",
            },
            MetadataSetField::Immunities => SetStorage::JsonSideTable {
                table: "actor_records",
                column: "immunities_json",
            },
            MetadataSetField::Resistances => SetStorage::JsonSideTable {
                table: "actor_records",
                column: "resistances_json",
            },
            MetadataSetField::Weaknesses => SetStorage::JsonSideTable {
                table: "actor_records",
                column: "weaknesses_json",
            },
            MetadataSetField::DisableSkills => SetStorage::JsonSideTable {
                table: "actor_records",
                column: "disable_skills_json",
            },
            MetadataSetField::VariantAxes => SetStorage::JsonColumn("r.variant_axes_json"),
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
                        "EXISTS (SELECT 1 FROM {table} s WHERE s.{key_column} = r.record_key AND s.{value_column} = {})",
                        self.text(value)
                    ),
                    SetStorage::JsonColumn(column) => {
                        json_array_contains_sql(column, &self.text(value))
                    }
                    SetStorage::JsonSideTable { table, column } => format!(
                        "EXISTS (SELECT 1 FROM {table} s WHERE s.record_key = r.record_key AND {})",
                        json_array_contains_sql(&format!("s.{column}"), &self.text(value))
                    ),
                })
            }
            CollectionOperator::IsNull => Ok(match set {
                SetStorage::Rows {
                    table, key_column, ..
                } => format!(
                    "NOT EXISTS (SELECT 1 FROM {table} s WHERE s.{key_column} = r.record_key)"
                ),
                SetStorage::JsonColumn(column) => json_array_empty_sql(column),
                SetStorage::JsonSideTable { table, column } => format!(
                    "NOT EXISTS (SELECT 1 FROM {table} s WHERE s.record_key = r.record_key AND NOT {})",
                    json_array_empty_sql(&format!("s.{column}"))
                ),
            }),
            CollectionOperator::IsNotNull => Ok(match set {
                SetStorage::Rows {
                    table, key_column, ..
                } => {
                    format!("EXISTS (SELECT 1 FROM {table} s WHERE s.{key_column} = r.record_key)")
                }
                SetStorage::JsonColumn(column) => format!("NOT ({})", json_array_empty_sql(column)),
                SetStorage::JsonSideTable { table, column } => format!(
                    "EXISTS (SELECT 1 FROM {table} s WHERE s.record_key = r.record_key AND NOT {})",
                    json_array_empty_sql(&format!("s.{column}"))
                ),
            }),
        }
    }

    fn multi_json_side_table_set(
        &mut self,
        op: CollectionOperator,
        value: Option<&str>,
        tables: &[(&str, &str)],
    ) -> Result<String, FilterCompileError> {
        let predicates = tables
            .iter()
            .map(|(table, column)| {
                self.set_predicate(SetStorage::JsonSideTable { table, column }, op, value)
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
            MetadataEnumStringField::PublicationFamily => "r.publication_family",
            MetadataEnumStringField::Size => "a.size",
            MetadataEnumStringField::Usage => "r.system_usage",
            MetadataEnumStringField::SystemGroup => "r.system_group",
            MetadataEnumStringField::FoundryRecordType => "r.foundry_record_type",
            MetadataEnumStringField::BaseItem => "r.system_base_item",
            MetadataEnumStringField::SaveType => "s.save_type",
            MetadataEnumStringField::AreaType => "s.area_type",
            MetadataEnumStringField::DurationUnit => "r.duration_unit",
            MetadataEnumStringField::Rarity => "r.rarity",
            MetadataEnumStringField::VariantGroupKey => "r.variant_group_key",
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
            MetadataTextStringField::PublicationTitle => "r.publication_title",
            MetadataTextStringField::RangeText => "s.range_text",
            MetadataTextStringField::DurationText => "r.duration_text",
            MetadataTextStringField::TargetText => "s.target_text",
            MetadataTextStringField::DisableText => "a.disable_text",
            MetadataTextStringField::VariantBaseName => "r.variant_base_name",
            MetadataTextStringField::VariantLabel => "r.variant_label",
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
            MetadataNumberField::Level => "r.level",
            MetadataNumberField::PriceCp => "r.price_cp",
            MetadataNumberField::BulkValue => "i.bulk_value",
            MetadataNumberField::ActionCost => "r.activation_time_actions",
            MetadataNumberField::RangeValue => "s.range_value",
            MetadataNumberField::AreaValue => "s.area_value",
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
            MetadataBooleanField::PublicationRemaster => "r.publication_remaster",
            MetadataBooleanField::Sustained => "s.sustained",
            MetadataBooleanField::BasicSave => "s.basic_save",
            MetadataBooleanField::IsComplex => "a.is_complex",
        };
        self.with_field_source(column, |compiler, qualified_column| {
            compiler.boolean_operator(qualified_column, op, value)
        })
    }

    fn with_field_source(
        &mut self,
        column: &str,
        compile: impl FnOnce(&mut Self, &str) -> Result<String, FilterCompileError>,
    ) -> Result<String, FilterCompileError> {
        if let Some((alias, table)) = side_table_for_column(column) {
            let predicate = compile(self, column)?;
            Ok(format!(
                "EXISTS (SELECT 1 FROM {table} {alias} WHERE {alias}.record_key = r.record_key AND {predicate})"
            ))
        } else {
            compile(self, column)
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
            ScalarValue::String(value) => ("text", "m.text_value", self.text(value)),
            ScalarValue::Number(value) => ("number", "m.number_value", self.number(*value)),
            ScalarValue::Boolean(value) => ("boolean", "m.bool_value", self.boolean(*value)),
        };
        Ok(format!(
            "EXISTS (SELECT 1 FROM record_metrics m WHERE m.record_key = r.record_key AND m.metric_key = {metric_param} AND m.value_type = '{value_type}' AND {column} {} {value_param})",
            metric_operator_sql(op)
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
enum SetStorage<'a> {
    Rows {
        table: &'a str,
        key_column: &'a str,
        value_column: &'a str,
    },
    JsonColumn(&'a str),
    JsonSideTable {
        table: &'a str,
        column: &'a str,
    },
}

fn side_table_for_column(column: &str) -> Option<(&'static str, &'static str)> {
    match column.chars().next() {
        Some('a') if column.starts_with("a.") => Some(("a", "actor_records")),
        Some('i') if column.starts_with("i.") => Some(("i", "item_records")),
        Some('s') if column.starts_with("s.") => Some(("s", "spell_records")),
        _ => None,
    }
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
