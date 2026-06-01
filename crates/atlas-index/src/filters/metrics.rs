use crate::schema_inventory::{record_metrics, records};
use atlas_domain::{MetricMatch, NumericMetricOperator, ScalarValue};

use super::FilterCompiler;
use super::error::FilterCompileError;
use super::sql_render::{aliased_column, metric_operator_sql, record_column};

impl FilterCompiler {
    pub(super) fn metric_predicate(
        &mut self,
        metric: &str,
        r#match: &MetricMatch,
    ) -> Result<String, FilterCompileError> {
        let (operator, value) = match r#match {
            MetricMatch::Eq { value } => ("=", value.clone()),
            MetricMatch::NotEq { value } => ("<>", value.clone()),
            MetricMatch::Gt { value } => (">", ScalarValue::Number(*value)),
            MetricMatch::Gte { value } => (">=", ScalarValue::Number(*value)),
            MetricMatch::Lt { value } => ("<", ScalarValue::Number(*value)),
            MetricMatch::Lte { value } => ("<=", ScalarValue::Number(*value)),
        };
        let metric_param = self.text(metric);
        let (value_type, column, value_param) = match value {
            ScalarValue::String(value) => (
                "text",
                aliased_column("m", record_metrics::columns::TEXT_VALUE),
                self.text(&value),
            ),
            ScalarValue::Number(value) => (
                "number",
                aliased_column("m", record_metrics::columns::NUMBER_VALUE),
                self.number(value),
            ),
            ScalarValue::Boolean(value) => (
                "boolean",
                aliased_column("m", record_metrics::columns::BOOL_VALUE),
                self.boolean(value),
            ),
        };
        Ok(format!(
            "EXISTS (SELECT 1 FROM {table} m WHERE {metric_record_key} = {record_key} AND {metric_key} = {metric_param} AND {metric_value_type} = '{value_type}' AND {column} {} {value_param})",
            operator,
            table = record_metrics::TABLE.name(),
            metric_record_key = aliased_column("m", record_metrics::columns::RECORD_KEY),
            record_key = record_column(records::columns::RECORD_KEY),
            metric_key = aliased_column("m", record_metrics::columns::METRIC_KEY),
            metric_value_type = aliased_column("m", record_metrics::columns::VALUE_TYPE),
        ))
    }

    pub(super) fn metric_compare_predicate(
        &mut self,
        left_metric: &str,
        op: NumericMetricOperator,
        right_metric: &str,
    ) -> Result<String, FilterCompileError> {
        Ok(format!(
            "EXISTS (SELECT 1 FROM {table} lm JOIN {table} rm ON {rm_record_key} = {lm_record_key} WHERE {lm_record_key} = {record_key} AND {lm_metric_key} = {} AND {rm_metric_key} = {} AND {lm_value_type} = 'number' AND {rm_value_type} = 'number' AND {lm_number_value} {} {rm_number_value})",
            self.text(left_metric),
            self.text(right_metric),
            metric_operator_sql(op),
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
        ))
    }
}
