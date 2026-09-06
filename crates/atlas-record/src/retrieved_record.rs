use crate::{AtlasRecord, ConsumableSpellChild, RecordBody};

/// Storage-neutral aggregate returned by product record retrieval.
#[derive(Debug, Clone, PartialEq)]
pub struct RetrievedRecord {
    pub record: AtlasRecord,
    pub body: Option<RecordBody>,
    pub spell_children: Vec<ConsumableSpellChild>,
}
