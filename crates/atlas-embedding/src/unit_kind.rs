use std::fmt;
use std::str::FromStr;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum EmbeddingUnitKind {
    Parent,
    HeadingSection,
    TitledOption,
}

impl EmbeddingUnitKind {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Parent => "parent",
            Self::HeadingSection => "heading_section",
            Self::TitledOption => "titled_option",
        }
    }
}

impl fmt::Display for EmbeddingUnitKind {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.as_str())
    }
}

impl FromStr for EmbeddingUnitKind {
    type Err = ParseEmbeddingUnitKindError;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value {
            "parent" => Ok(Self::Parent),
            "heading_section" => Ok(Self::HeadingSection),
            "titled_option" => Ok(Self::TitledOption),
            _ => Err(ParseEmbeddingUnitKindError {
                value: value.to_string(),
            }),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParseEmbeddingUnitKindError {
    value: String,
}

impl fmt::Display for ParseEmbeddingUnitKindError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "unsupported embedding unit kind `{}`",
            self.value
        )
    }
}

impl std::error::Error for ParseEmbeddingUnitKindError {}
