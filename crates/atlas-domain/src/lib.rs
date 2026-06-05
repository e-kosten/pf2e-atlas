#![deny(unsafe_code)]

pub mod categories;
pub mod detail;
pub mod discovery;
pub mod metadata;
pub mod record;
pub mod record_key;
pub mod search_filter;

pub use categories::RecordKind;
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
    ActionCost, Level, MetricDomain, MetricValueType, Publication, PublicationCategory, Rarity,
    RecordSummary, RemasterLinkSource, SourceProvenance, TextStatus, TimeKind, TimeUnit,
    normalize_record_name,
};
pub use record_key::{PackName, RecordId, RecordKey, RecordKeyParseError};
pub use search_filter::{
    MetricMatch, NullableNumericMatch, NullableStringMatch, NumericMatch, ScalarValue,
    SearchFilterNode, SearchFilterValidationError,
};
