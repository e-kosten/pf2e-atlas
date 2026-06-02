pub(crate) mod aliases;
pub(crate) mod loaded;
pub(crate) mod metrics;
pub(crate) mod references;
pub(crate) mod taxonomy;
pub(crate) mod variants;
pub(crate) mod visibility;

pub(crate) use loaded::{
    EmbeddedItemContentRef, EmbeddedItemFact, JournalPageFact, JournalPageSkipReason,
    LoadedSourceRecord, RecordReferenceIndex, ReferenceCandidate, SkippedJournalPageFact,
    SourceConstructionFacts, SourceRecordFacts,
};
