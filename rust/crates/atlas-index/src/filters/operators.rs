use atlas_domain::metadata::{
    MetadataBooleanMatch, MetadataNumberMatch, MetadataStringMatch, MetadataTextMatch,
};

use super::FilterCompiler;
use super::error::FilterCompileError;
use super::sql_render::contains_like_pattern;

impl FilterCompiler {
    pub(super) fn string_operator(
        &mut self,
        column: &str,
        r#match: &MetadataStringMatch,
    ) -> Result<String, FilterCompileError> {
        match r#match {
            MetadataStringMatch::Eq { value } => Ok(format!("{column} = {}", self.text(value))),
            MetadataStringMatch::NotEq { value } => Ok(format!("{column} <> {}", self.text(value))),
            MetadataStringMatch::IsNull => Ok(format!("{column} IS NULL")),
            MetadataStringMatch::IsNotNull => Ok(format!("{column} IS NOT NULL")),
        }
    }

    pub(super) fn text_operator(
        &mut self,
        column: &str,
        r#match: &MetadataTextMatch,
    ) -> Result<String, FilterCompileError> {
        match r#match {
            MetadataTextMatch::Eq { value } => Ok(format!("{column} = {}", self.text(value))),
            MetadataTextMatch::NotEq { value } => Ok(format!("{column} <> {}", self.text(value))),
            MetadataTextMatch::Contains { value } => Ok(format!(
                "{column} LIKE {} ESCAPE '\\'",
                self.text(&contains_like_pattern(value))
            )),
            MetadataTextMatch::NotContains { value } => Ok(format!(
                "{column} NOT LIKE {} ESCAPE '\\'",
                self.text(&contains_like_pattern(value))
            )),
            MetadataTextMatch::IsNull => Ok(format!("{column} IS NULL")),
            MetadataTextMatch::IsNotNull => Ok(format!("{column} IS NOT NULL")),
        }
    }

    pub(super) fn number_operator(
        &mut self,
        column: &str,
        r#match: MetadataNumberMatch,
    ) -> Result<String, FilterCompileError> {
        match r#match {
            MetadataNumberMatch::Eq { value } => Ok(format!("{column} = {}", self.number(value))),
            MetadataNumberMatch::Gt { value } => Ok(format!("{column} > {}", self.number(value))),
            MetadataNumberMatch::Gte { value } => Ok(format!("{column} >= {}", self.number(value))),
            MetadataNumberMatch::Lt { value } => Ok(format!("{column} < {}", self.number(value))),
            MetadataNumberMatch::Lte { value } => Ok(format!("{column} <= {}", self.number(value))),
            MetadataNumberMatch::Between { min, max } => Ok(format!(
                "{column} BETWEEN {} AND {}",
                self.number(min),
                self.number(max)
            )),
            MetadataNumberMatch::IsNull => Ok(format!("{column} IS NULL")),
            MetadataNumberMatch::IsNotNull => Ok(format!("{column} IS NOT NULL")),
        }
    }

    pub(super) fn boolean_operator(
        &mut self,
        column: &str,
        r#match: MetadataBooleanMatch,
    ) -> Result<String, FilterCompileError> {
        match r#match {
            MetadataBooleanMatch::Eq { value } => Ok(format!("{column} = {}", self.boolean(value))),
            MetadataBooleanMatch::IsNull => Ok(format!("{column} IS NULL")),
            MetadataBooleanMatch::IsNotNull => Ok(format!("{column} IS NOT NULL")),
        }
    }
}
