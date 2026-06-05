use atlas_domain::{RecordKey, SearchFilterNode};
use atlas_index::FilteredRecordSort;
use atlas_record::AtlasRecord;
use serde::{Deserialize, Serialize};

use crate::{SearchPage, SearchPageInfo};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RecordListSort {
    Alphabetical,
    LevelAsc,
    LevelDesc,
    PriceAsc,
    PriceDesc,
    RecordKey,
    Random { seed: u64 },
}

impl From<RecordListSort> for FilteredRecordSort {
    fn from(value: RecordListSort) -> Self {
        match value {
            RecordListSort::Alphabetical => Self::Alphabetical,
            RecordListSort::LevelAsc => Self::LevelAsc,
            RecordListSort::LevelDesc => Self::LevelDesc,
            RecordListSort::PriceAsc => Self::PriceAsc,
            RecordListSort::PriceDesc => Self::PriceDesc,
            RecordListSort::RecordKey => Self::RecordKey,
            RecordListSort::Random { seed } => Self::Random { seed },
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct GetRecordRequest<'a> {
    pub record_key: &'a RecordKey,
}

#[derive(Debug, Clone, PartialEq)]
pub struct GetRecordsRequest<'a> {
    pub record_keys: &'a [RecordKey],
}

#[derive(Debug, Clone, PartialEq)]
pub struct ResolveRecordRequest<'a> {
    pub query: &'a str,
    pub filter: Option<&'a SearchFilterNode>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ListRecordsRequest<'a> {
    pub filter: Option<&'a SearchFilterNode>,
    pub sort: RecordListSort,
    pub page: SearchPage,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ListRecordsResult {
    pub record_keys: Vec<RecordKey>,
    pub records: Vec<AtlasRecord>,
    pub total: u64,
    pub page: SearchPageInfo,
}

#[derive(Debug, Clone, PartialEq)]
pub struct RecordResolutionResult {
    pub query: String,
    pub normalized_query: String,
    pub match_kind: RecordResolutionMatchKind,
    pub matched_text: String,
    pub alias_source: Option<String>,
    pub alias_source_ref: Option<String>,
    pub record: AtlasRecord,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RecordResolutionMatchKind {
    Name,
    NormalizedName,
    Alias,
    VariantName,
}

impl RecordResolutionMatchKind {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Name => "name",
            Self::NormalizedName => "normalized_name",
            Self::Alias => "alias",
            Self::VariantName => "variant_name",
        }
    }
}
