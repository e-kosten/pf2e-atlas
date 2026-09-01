//! Exact source-leaf coverage contracts and test-owned parity evidence.
//!
//! This module deliberately does not trace production ingest. Coverage is
//! established by source-grounded focused tests that submit actual-read
//! receipts for exact selectors and final owners.

mod contract;
mod parity;
mod receipt;
mod registry;

pub use contract::{
    ATLAS_SOURCE_LEAF_COVERAGE_VERSION, CoverageContractError, ExpectedSourceShape,
    FinalOwnerContract, FinalOwnerStage, FixtureContract, FixturePrevalence, FixtureProvenance,
    MapKeyPolicy, ReaderContract, SourceDocumentRole, SourceLeafContract, SourceLeafCoverageLedger,
    SourceLeafDisposition, SourceLeafIdentity, SourceLeafKind, SourceLeafSelector,
    SourceParentContextSelector, SourcePin, SourcePrevalence, SurfaceContract, SurfaceDecision,
    SurfaceDisposition, lint_source_leaf_ledger, lint_source_leaf_ledgers,
    parse_source_leaf_ledger,
};
pub use parity::{
    CoverageFailure, CoverageFailureCode, CoverageReport, evaluate_source_leaf_coverage,
};
pub(crate) use receipt::{
    SourceAccessorPurpose, SourceJsonType, SourceLeafValue, SourceMemberKind,
};
pub use receipt::{SourceLeafReceipt, capture_registered_source_leaf_receipt};
pub use registry::{
    PF2E_TYPE_REGISTRY_ENTRY_COUNT, PF2E_TYPE_REGISTRY_SHA256, PF2E_TYPE_REGISTRY_VERSION,
};
