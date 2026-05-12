use std::fmt;
use std::str::FromStr;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum DetailLevel {
    Minimal,
    Standard,
    Full,
}

impl DetailLevel {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Minimal => "minimal",
            Self::Standard => "standard",
            Self::Full => "full",
        }
    }
}

impl fmt::Display for DetailLevel {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.as_str())
    }
}

impl FromStr for DetailLevel {
    type Err = DetailLevelParseError;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value.trim() {
            "minimal" => Ok(Self::Minimal),
            "standard" => Ok(Self::Standard),
            "full" => Ok(Self::Full),
            _ => Err(DetailLevelParseError {
                value: value.to_string(),
            }),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DetailLevelParseError {
    value: String,
}

impl fmt::Display for DetailLevelParseError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "unknown detail level `{}`", self.value)
    }
}

impl std::error::Error for DetailLevelParseError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_current_ts_detail_wire_values() {
        assert_eq!(
            serde_json::to_string(&DetailLevel::Minimal).expect("detail serializes"),
            "\"minimal\""
        );
        assert_eq!(
            serde_json::to_string(&DetailLevel::Standard).expect("detail serializes"),
            "\"standard\""
        );
        assert_eq!(
            serde_json::to_string(&DetailLevel::Full).expect("detail serializes"),
            "\"full\""
        );
    }

    #[test]
    fn does_not_accept_compact_as_wire_value() {
        let result: Result<DetailLevel, _> = serde_json::from_str("\"compact\"");
        assert!(result.is_err());
        assert!("compact".parse::<DetailLevel>().is_err());
    }
}
