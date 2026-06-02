use crate::artifact::inventory::records;
use crate::read::sql::SqlBindValue;
use atlas_domain::SearchFilterNode;

use super::error::FilterCompileError;
use super::sql_render::record_column;

#[derive(Default)]
pub(super) struct FilterCompiler {
    pub(super) parameters: Vec<SqlBindValue>,
}

impl FilterCompiler {
    pub(super) fn compile_node(
        &mut self,
        filter: &SearchFilterNode,
    ) -> Result<String, FilterCompileError> {
        match filter {
            SearchFilterNode::RecordKind { value } => Ok(format!(
                "{} = {}",
                record_column(records::columns::RECORD_KIND),
                self.text(value.as_str())
            )),
            SearchFilterNode::LinksTo { target } => self.links_to_predicate(target),
            SearchFilterNode::LinkedFrom { source } => self.linked_from_predicate(source),
            SearchFilterNode::MetadataPredicate { predicate } => self.metadata_predicate(predicate),
            SearchFilterNode::Metric { metric, r#match } => self.metric_predicate(metric, r#match),
            SearchFilterNode::MetricCompare {
                left_metric,
                op,
                right_metric,
            } => self.metric_compare_predicate(left_metric, *op, right_metric),
            SearchFilterNode::AnyOf { children } => self.boolean_group(children, " OR ", "any_of"),
            SearchFilterNode::AllOf { children } => self.boolean_group(children, " AND ", "all_of"),
            SearchFilterNode::Not { child } => Ok(format!("NOT ({})", self.compile_node(child)?)),
        }
    }

    fn boolean_group(
        &mut self,
        children: &[SearchFilterNode],
        joiner: &str,
        kind: &str,
    ) -> Result<String, FilterCompileError> {
        if children.is_empty() {
            return Err(FilterCompileError::InvalidValue(format!(
                "filter `{kind}` must contain at least one child"
            )));
        }
        let compiled = children
            .iter()
            .map(|child| self.compile_node(child).map(|sql| format!("({sql})")))
            .collect::<Result<Vec<_>, _>>()?;
        Ok(compiled.join(joiner))
    }

    pub(super) fn text(&mut self, value: &str) -> String {
        self.push(SqlBindValue::Text(value.to_string()))
    }

    pub(super) fn number(&mut self, value: f64) -> String {
        self.push(SqlBindValue::Real(value))
    }

    pub(super) fn boolean(&mut self, value: bool) -> String {
        self.push(SqlBindValue::Integer(if value { 1 } else { 0 }))
    }

    pub(super) fn push(&mut self, value: SqlBindValue) -> String {
        self.parameters.push(value);
        format!("?{}", self.parameters.len())
    }
}
