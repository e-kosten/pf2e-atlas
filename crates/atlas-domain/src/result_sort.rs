//! Requested result ordering is distinct from retrieval and its matching set.
use serde::{Deserialize, Serialize};
use ts_rs::TS;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
pub enum ResultSort {
    Relevance,
    SavedOrder,
    NameAsc,
    NameDesc,
    LevelAsc,
    LevelDesc,
    UnitPriceAsc,
    UnitPriceDesc,
}
impl ResultSort {
    pub const fn is_attribute(self) -> bool {
        !matches!(self, Self::Relevance | Self::SavedOrder)
    }
    pub const fn is_unit_price(self) -> bool {
        matches!(self, Self::UnitPriceAsc | Self::UnitPriceDesc)
    }
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Relevance => "relevance",
            Self::SavedOrder => "saved_order",
            Self::NameAsc => "name_asc",
            Self::NameDesc => "name_desc",
            Self::LevelAsc => "level_asc",
            Self::LevelDesc => "level_desc",
            Self::UnitPriceAsc => "unit_price_asc",
            Self::UnitPriceDesc => "unit_price_desc",
        }
    }
}
impl std::fmt::Display for ResultSort {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}
impl std::str::FromStr for ResultSort {
    type Err = String;
    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value {
            "relevance" => Ok(Self::Relevance),
            "saved_order" => Ok(Self::SavedOrder),
            "name_asc" => Ok(Self::NameAsc),
            "name_desc" => Ok(Self::NameDesc),
            "level_asc" => Ok(Self::LevelAsc),
            "level_desc" => Ok(Self::LevelDesc),
            "unit_price_asc" => Ok(Self::UnitPriceAsc),
            "unit_price_desc" => Ok(Self::UnitPriceDesc),
            _ => Err("unknown result sort".into()),
        }
    }
}
