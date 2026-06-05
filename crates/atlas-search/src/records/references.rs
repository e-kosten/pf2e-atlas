use atlas_record::AtlasRecord;

use crate::text::TextSearchRecord;
use crate::{AtlasRetrievalService, SearchError};

use super::RecordResolutionResult;

impl AtlasRetrievalService {
    pub(crate) fn enrich_reference_labels(
        &self,
        _records: &mut [AtlasRecord],
    ) -> Result<(), SearchError> {
        Ok(())
    }

    pub(crate) fn enrich_resolution_reference_labels(
        &self,
        _matches: &mut [RecordResolutionResult],
    ) -> Result<(), SearchError> {
        Ok(())
    }

    pub(crate) fn enrich_text_record_reference_labels(
        &self,
        _records: &mut [TextSearchRecord],
    ) -> Result<(), SearchError> {
        Ok(())
    }
}
