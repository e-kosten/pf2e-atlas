#![deny(unsafe_code)]

pub mod categories;
pub mod detail;
pub mod discovery;
pub mod metadata;
pub mod record;
pub mod record_key;
pub mod rule_graph;
pub mod search_request;

pub use categories::RecordFamily;
pub use detail::DetailLevel;
pub use discovery::{
    BooleanFieldCounts, FilterDiscoveryExecution, FilterFieldDiscovery, FilterFieldGroup,
    FilterFieldInfo, FilterFieldStats, FilterFieldType, FilterOperator, FilterSample,
    FilterSampleExample, FilterValueCount, FilterValueDiscovery, FilterValuePayload,
    FilterValuePolicy, FilterValueSort, MetricKeyDiscovery, MetricValuePayload, NumericFieldStats,
};
pub use metadata::{
    MetadataBooleanField, MetadataBooleanMatch, MetadataEnumStringField, MetadataNumberField,
    MetadataNumberMatch, MetadataPredicate, MetadataSetField, MetadataSetMatch,
    MetadataStringMatch, MetadataTextMatch, MetadataTextStringField, NumericMetricOperator,
};
pub use record::{
    ActionCost, Level, MetricDomain, MetricValueType, Publication, PublicationFamily, Rarity,
    RecordSummary, SourceProvenance, TextStatus, TimeKind, TimeUnit,
};
pub use record_key::{PackName, RecordId, RecordKey, RecordKeyParseError};
pub use rule_graph::{
    ReferenceDirection, ReferenceEdge, ReferenceRelationship, ReferenceSource, RemasterLink,
    RemasterLinkSource, RuleContextRequest, RuleContextResult, RuleGraphCollectionResult,
    RuleGraphRequest, RuleGraphResult,
};
pub use search_request::{
    BrowseSortSpec, LookupSortKind, LookupSortPolicy, LookupSortSpec, MetricMatch,
    NullableNumericMatch, NullableStringMatch, NumericMatch, ScalarValue, SearchFilterNode,
    SearchFilterValidationError, SearchFusionMethod, SearchRequest, SearchRetrievalMode,
};
