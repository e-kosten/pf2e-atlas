use atlas_artifact::schema::records;
use atlas_domain::{NullableNumericMatch, NullableStringMatch, NumericMatch, SearchFilterNode};
use rusqlite::types::Value;

use super::error::FilterCompileError;
use super::sql_render::record_column;

#[derive(Default)]
pub(super) struct FilterCompiler {
    pub(super) parameters: Vec<Value>,
}

impl FilterCompiler {
    pub(super) fn compile_node(
        &mut self,
        filter: &SearchFilterNode,
    ) -> Result<String, FilterCompileError> {
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
            SearchFilterNode::LinksTo { target } => self.links_to_predicate(target),
            SearchFilterNode::LinkedFrom { source } => self.linked_from_predicate(source),
            SearchFilterNode::MetadataPredicate { predicate } => self.metadata_predicate(predicate),
            SearchFilterNode::Metric { metric, op, value } => {
                self.metric_predicate(metric, *op, value)
            }
            SearchFilterNode::MetricCompare {
                left_metric,
                op,
                right_metric,
            } => self.metric_compare_predicate(left_metric, *op, right_metric),
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

    pub(super) fn numeric_match(
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

    pub(super) fn nullable_numeric_match(
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

    pub(super) fn nullable_string_match(
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

    pub(super) fn text(&mut self, value: &str) -> String {
        self.push(Value::Text(value.to_string()))
    }

    pub(super) fn number(&mut self, value: f64) -> String {
        self.push(Value::Real(value))
    }

    pub(super) fn boolean(&mut self, value: bool) -> String {
        self.push(Value::Integer(if value { 1 } else { 0 }))
    }

    pub(super) fn push(&mut self, value: Value) -> String {
        self.parameters.push(value);
        format!("?{}", self.parameters.len())
    }
}
