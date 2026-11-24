use serde::{Deserialize, Serialize};

/// Select the FTS columns and the interpretation of the final token independently.
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct LexicalOptions {
    #[serde(default)]
    pub fields: LexicalFields,
    #[serde(default, rename = "match")]
    pub matching: LexicalMatch,
}

#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum LexicalFields {
    #[default]
    All,
    RecordNames,
}

#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum LexicalMatch {
    #[default]
    CompleteTerms,
    LastTokenPrefix,
}
