//! Request vocabulary for the bounded hybrid-ranking comparison.
use serde::{Deserialize, Serialize};
use ts_rs::TS;

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS)]
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
