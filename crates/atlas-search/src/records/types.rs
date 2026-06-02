use atlas_domain::{RecordKey, SearchFilterNode};
use atlas_index::FilteredRecordSort;
use atlas_record::AtlasRecord;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RecordBrowseSort {
    Alphabetical,
    LevelAsc,
    LevelDesc,
    PriceAsc,
    PriceDesc,
    RecordKey,
    Random { seed: u64 },
}

impl From<RecordBrowseSort> for FilteredRecordSort {
    fn from(value: RecordBrowseSort) -> Self {
        match value {
            RecordBrowseSort::Alphabetical => Self::Alphabetical,
            RecordBrowseSort::LevelAsc => Self::LevelAsc,
            RecordBrowseSort::LevelDesc => Self::LevelDesc,
            RecordBrowseSort::PriceAsc => Self::PriceAsc,
            RecordBrowseSort::PriceDesc => Self::PriceDesc,
            RecordBrowseSort::RecordKey => Self::RecordKey,
            RecordBrowseSort::Random { seed } => Self::Random { seed },
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
pub struct BrowseRecordsRequest<'a> {
    pub filter: Option<&'a SearchFilterNode>,
    pub sort: RecordBrowseSort,
    pub limit: u32,
    pub offset: u32,
}

#[derive(Debug, Clone, PartialEq)]
pub struct BrowseRecordsResult {
    pub record_keys: Vec<RecordKey>,
    pub records: Vec<AtlasRecord>,
    pub total: u64,
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
