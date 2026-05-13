use std::collections::BTreeMap;

use atlas_domain::RecordKey;
pub use atlas_record::{
    ActorSideData, AliasSource, ItemSideData, MetricRow, MetricValue,
    NormalizedRecord as LoadedRecord, NormalizedTime, RecordAlias, ReferenceCandidate,
    ReferenceEdge, RemasterLink, SpellSideData,
};

#[derive(Debug, Clone, Default)]
pub(crate) struct RecordReferenceIndex {
    pub(crate) by_key: BTreeMap<String, LoadedRecord>,
    pub(crate) by_pack_id: BTreeMap<(String, String), RecordKey>,
    pub(crate) by_pack_name: BTreeMap<(String, String), Vec<RecordKey>>,
    pub(crate) by_name: BTreeMap<String, Vec<RecordKey>>,
}
