use atlas_domain::metadata::{BooleanOperator, NumberOperator, StringOperator, TextOperator};

use super::FilterCompiler;
use super::error::FilterCompileError;
use super::sql_render::contains_like_pattern;

impl FilterCompiler {
    pub(super) fn string_operator(
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

    pub(super) fn text_operator(
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

    pub(super) fn number_operator(
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

    pub(super) fn boolean_operator(
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

    pub(super) fn in_list(
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

    pub(super) fn required_text(
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

    pub(super) fn required_number(
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
}
