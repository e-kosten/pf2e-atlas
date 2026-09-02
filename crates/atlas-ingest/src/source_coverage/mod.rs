//! Exact source-leaf coverage contracts and test-owned parity evidence.
//!
//! Coverage is established by sealed, source-grounded focused witnesses that
//! execute registered production readers and final-owner stages.

mod contract;
mod parity;
mod prevalence;
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
pub use prevalence::{PF2E_SOURCE_LEAF_PREVALENCE_SHA256, PF2E_SOURCE_LEAF_PREVALENCE_VERSION};
pub(crate) use receipt::{
    SourceAccessorPurpose, SourceJsonType, SourceLeafValue, SourceMemberKind,
};
pub use receipt::{SourceLeafReceipt, capture_registered_source_leaf_receipt};
pub use registry::{
    PF2E_TYPE_REGISTRY_ENTRY_COUNT, PF2E_TYPE_REGISTRY_SHA256, PF2E_TYPE_REGISTRY_VERSION,
};
