pub(crate) mod aliases;
pub(crate) mod loaded;
pub(crate) mod metrics;
pub(crate) mod references;
pub(crate) mod taxonomy;
pub(crate) mod variants;

pub(crate) use loaded::{
    EmbeddedItemContentRef, EmbeddedItemFact, LoadedSourceRecord, RecordReferenceIndex,
    SourceConstructionFacts, SourceContentFact, SourceRecordFacts,
};
