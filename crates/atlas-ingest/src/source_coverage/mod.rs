//! Exact source-leaf coverage contracts and test-owned parity evidence.
//!
//! This module deliberately does not trace production ingest. Coverage is
//! established by source-grounded focused tests that submit actual-read
//! receipts for exact selectors and final owners.

mod contract;
mod parity;
mod receipt;

pub use contract::{
    ATLAS_SOURCE_LEAF_COVERAGE_VERSION, CoverageContractError, ExpectedSourceShape,
    FinalOwnerContract, FinalOwnerStage, MapKeyPolicy, ReaderContract, SourceDocumentRole,
    SourceLeafContract, SourceLeafCoverageLedger, SourceLeafDisposition, SourceLeafIdentity,
    SourceLeafSelector, SourcePin, SourcePrevalence, SurfaceContract, SurfaceDecision,
    SurfaceDisposition, lint_source_leaf_ledger, lint_source_leaf_ledgers,
    parse_source_leaf_ledger,
};
pub use parity::{
    CoverageFailure, CoverageFailureCode, CoverageReport, evaluate_source_leaf_coverage,
};
pub use receipt::{
    FixtureReference, SourceJsonType, SourceLeafReceipt, SourceLeafValue, SourceMemberKind,
    StageObservation, TypedUnsupportedValue,
};
