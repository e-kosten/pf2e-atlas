mod get;
pub(crate) mod resolution;
mod types;

use atlas_record::AtlasRecord;

use crate::SearchError;

pub use types::{
    GetRecordRequest, GetRecordsRequest, ListRecordsRequest, ListRecordsResult, RecordListSort,
    RecordResolutionMatchKind, RecordResolutionResult, ResolveRecordRequest,
};

pub trait RecordRetrieval {
    fn get_records(&self, request: GetRecordsRequest<'_>) -> Result<Vec<AtlasRecord>, SearchError>;

    fn get_record(&self, request: GetRecordRequest<'_>)
    -> Result<Option<AtlasRecord>, SearchError>;

    fn list_records(
        &self,
        request: ListRecordsRequest<'_>,
    ) -> Result<ListRecordsResult, SearchError>;

    fn resolve_record(
        &self,
        request: ResolveRecordRequest<'_>,
    ) -> Result<Vec<RecordResolutionResult>, SearchError>;
}
