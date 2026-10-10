#![deny(unsafe_code)]

pub mod categories;
mod name_lookup;
pub mod query;
pub mod query_discovery;
pub mod record_key;
pub use query_discovery::{QueryFieldCounts, QueryStateCount, QueryValueOption, QueryValueOptions};
pub mod source_address;
pub mod source_summary;

pub use source_summary::{SourceLevelBasis, SourceQueryFact, SourceRecordSummary};

pub use query::{
    QueryCapability, QueryCompare, QueryError, QueryExpression, QueryFieldDefinition,
    QueryFieldState, QueryFieldType, QueryLimits, QueryLiteral, QueryPredicate, QuerySetMatch,
    QueryTruth, QueryValueDiscovery,
};
pub use source_address::{InvalidSourceByteRange, SourceByteRange, SourcePassageAddress};

pub use categories::RecordKind;
pub use name_lookup::normalize_record_name;
pub use record_key::{PackName, RecordId, RecordKey, RecordKeyParseError};
