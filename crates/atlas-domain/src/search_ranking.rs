//! Request vocabulary for the bounded hybrid-ranking comparison.
use serde::{Deserialize, Serialize};

#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SearchRanking {
    Rrf,
    #[default]
    Score,
}
impl std::str::FromStr for SearchRanking {
    type Err = String;
    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value {
            "rrf" => Ok(Self::Rrf),
            "score" => Ok(Self::Score),
            _ => Err("ranking must be rrf or score".into()),
        }
    }
}

/// Amount of evidence to materialize after ranking a result page.
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SearchResultDetail {
    #[default]
    Records,
    Compact,
    Evidence,
}
impl SearchResultDetail {
    pub fn includes_evidence(self) -> bool {
        self != Self::Records
    }
}
