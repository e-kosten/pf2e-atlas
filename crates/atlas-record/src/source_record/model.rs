use super::{FieldAvailability, SourceContentOutcome, SourceRelationshipOccurrence};
use crate::source_content::{ContentAudience, OwnedContentLocator, SourceContentLocator};
use atlas_domain::RecordKey;
use atlas_foundry_model::FoundryDocumentSource;
use serde::{Deserialize, Serialize};

/// The source body uses atlas-foundry-model's checked snapshot codec. Deliberately
/// no envelope Serde: it must not bypass that codec's identity and stack guards.
#[derive(Debug, PartialEq, Eq)]
pub struct SourceBackedRecord {
    pub key: RecordKey,
    pub source: FoundryDocumentSource,
    pub enrichment: SourceRecordEnrichment,
}

#[derive(Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct SourceRecordEnrichment {
    pub audience: ContentAudience,
    pub owned_nodes: Vec<OwnedNodeFact>,
    pub collections: Vec<OwnedCollectionFact>,
    pub content: Vec<SourceContentOutcome>,
    pub relationships: Vec<SourceRelationshipOccurrence>,
}

#[derive(Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct OwnedNodeFact {
    pub owners: Vec<OwnedContentLocator>,
    pub order: usize,
    pub document_kind: String,
    pub source_type: Option<String>,
}

#[derive(Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct OwnedCollectionFact {
    pub locator: SourceContentLocator,
    pub availability: FieldAvailability,
    /// Some(0) is a complete empty collection; None is unavailable.
    pub length: Option<usize>,
}
