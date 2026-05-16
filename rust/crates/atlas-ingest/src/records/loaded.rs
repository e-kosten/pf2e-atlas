use std::collections::BTreeMap;

use atlas_domain::RecordKey;
pub use atlas_record::{
    ActorSideData, AliasSource, ItemSideData, MetricRow, MetricValue, NormalizedRecord,
    NormalizedTime, RecordAlias, ReferenceEdge, RemasterLink, SpellSideData,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ReferenceCandidate {
    pub(crate) raw_target: String,
    pub(crate) display_text: Option<String>,
    pub(crate) reference_text: String,
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct LoadedSourceRecord {
    pub(crate) record: NormalizedRecord,
    pub(crate) facts: SourceConstructionFacts,
}

impl LoadedSourceRecord {
    pub(crate) const fn new(record: NormalizedRecord, facts: SourceConstructionFacts) -> Self {
        Self { record, facts }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct SourceConstructionFacts {
    pub(crate) reference_candidates: Vec<ReferenceCandidate>,
    pub(crate) source_description_markup: Option<String>,
}

#[derive(Debug, Clone, Default)]
pub(crate) struct RecordReferenceIndex {
    pub(crate) by_key: BTreeMap<String, NormalizedRecord>,
    pub(crate) by_pack_id: BTreeMap<(String, String), RecordKey>,
    pub(crate) by_pack_name: BTreeMap<(String, String), Vec<RecordKey>>,
    pub(crate) by_name: BTreeMap<String, Vec<RecordKey>>,
}
