mod get;
pub(crate) mod references;
pub(crate) mod resolution;
mod types;

use atlas_record::AtlasRecord;

use crate::SearchError;

pub use types::{
    BrowseRecordsRequest, BrowseRecordsResult, GetRecordRequest, GetRecordsRequest,
    RecordBrowseSort, RecordResolutionMatchKind, RecordResolutionResult, ResolveRecordRequest,
};

pub trait RecordRetrieval {
    fn get_records(&self, request: GetRecordsRequest<'_>) -> Result<Vec<AtlasRecord>, SearchError>;

    fn get_record(&self, request: GetRecordRequest<'_>)
    -> Result<Option<AtlasRecord>, SearchError>;

    fn browse_records(
        &self,
        request: BrowseRecordsRequest<'_>,
    ) -> Result<BrowseRecordsResult, SearchError>;

    fn resolve_record(
        &self,
        request: ResolveRecordRequest<'_>,
    ) -> Result<Vec<RecordResolutionResult>, SearchError>;
}
