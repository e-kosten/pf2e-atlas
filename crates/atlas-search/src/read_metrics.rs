//! Optional development read counters over index-owned instrumentation.
use crate::AtlasRetrievalService;
use atlas_index::SourceReadMetrics;

impl AtlasRetrievalService {
    /// Snapshot the instrumented index read counters for this service.
    pub fn read_metrics(&self) -> SourceReadMetrics {
        self.index.read_metrics()
    }

    /// Clear the instrumented index read counters for this service.
    pub fn reset_read_metrics(&self) {
        self.index.reset_read_metrics();
    }
}
