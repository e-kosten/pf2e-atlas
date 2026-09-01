use std::collections::{BTreeMap, BTreeSet};

use serde::{Deserialize, Serialize};

use crate::SourcePresence;

use super::{
    FinalOwnerStage, FixtureProvenance, MapKeyPolicy, SourceAccessorPurpose, SourceLeafContract,
    SourceLeafCoverageLedger, SourceLeafDisposition, SourceLeafIdentity, SourceLeafReceipt,
    SourceLeafValue, SourceMemberKind, lint_source_leaf_ledger,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CoverageFailureCode {
    InvalidContract,
    RegistryBindingMismatch,
    SourcePinMismatch,
    UndeclaredLeaf,
    BroadDeclaration,
    DuplicateOwnership,
    ReaderNotObserved,
    DtoMismatch,
    CanonicalMismatch,
    PostProjectionMismatch,
    ArtifactHydrationMismatch,
    PublicSurfaceMismatch,
    ProvenanceNotDurable,
    IgnoredPromoted,
    StaleDeclaration,
    FixtureNotSourceGrounded,
    ReceiptProvenanceInvalid,
    MapKeyPolicyMismatch,
    SourcePrevalenceMismatch,
    ReceiptSetMismatch,
    UnresolvedDeferred,
    UnconsumedLeaf,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CoverageFailure {
    pub code: CoverageFailureCode,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub identity: Option<SourceLeafIdentity>,
    pub message: String,
}

impl CoverageFailure {
    pub(crate) fn ledger(code: CoverageFailureCode, message: impl Into<String>) -> Self {
        Self {
            code,
            identity: None,
            message: message.into(),
        }
    }

    pub(crate) fn for_identity(
        code: CoverageFailureCode,
        identity: SourceLeafIdentity,
        message: impl Into<String>,
    ) -> Self {
        Self {
            code,
            identity: Some(identity),
            message: message.into(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CoverageReport {
    pub contract_version: String,
    pub passed: bool,
    pub declaration_count: usize,
    pub receipt_count: usize,
    pub failures: Vec<CoverageFailure>,
}

pub fn evaluate_source_leaf_coverage(
    ledger: &SourceLeafCoverageLedger,
    receipts: &[SourceLeafReceipt],
) -> CoverageReport {
    let mut failures = lint_source_leaf_ledger(ledger);
    let declarations = ledger
        .leaves
        .iter()
        .map(|leaf| (ledger.identity_for(leaf), leaf))
        .collect::<BTreeMap<_, _>>();
    let mut receipts_by_identity = BTreeMap::<_, Vec<&SourceLeafReceipt>>::new();

    for receipt in receipts {
        if !declarations.contains_key(receipt.identity()) {
            failures.push(CoverageFailure::for_identity(
                CoverageFailureCode::UndeclaredLeaf,
                receipt.identity().clone(),
                "no exact registry tuple and leaf declaration owns this receipt",
            ));
            continue;
        }
        receipts_by_identity
            .entry(receipt.identity().clone())
            .or_default()
            .push(receipt);
    }

    for (identity, declaration) in declarations {
        evaluate_declaration(
            identity.clone(),
            declaration,
            receipts_by_identity
                .get(&identity)
                .map(Vec::as_slice)
                .unwrap_or_default(),
            &mut failures,
        );
    }

    failures.sort_by(|left, right| {
        left.code
            .cmp(&right.code)
            .then_with(|| left.identity.cmp(&right.identity))
            .then_with(|| left.message.cmp(&right.message))
    });
    failures.dedup();
    CoverageReport {
        contract_version: ledger.contract_version.clone(),
        passed: failures.is_empty(),
        declaration_count: ledger.leaves.len(),
        receipt_count: receipts.len(),
        failures,
    }
}

fn evaluate_declaration(
    identity: SourceLeafIdentity,
    declaration: &SourceLeafContract,
    receipts: &[&SourceLeafReceipt],
    failures: &mut Vec<CoverageFailure>,
) {
    match declaration.disposition {
        SourceLeafDisposition::Deferred => {
            failures.push(CoverageFailure::for_identity(
                CoverageFailureCode::UnresolvedDeferred,
                identity,
                "deferred leaves cannot pass a completed type gate",
            ));
            return;
        }
        SourceLeafDisposition::Unconsumed => {
            failures.push(CoverageFailure::for_identity(
                CoverageFailureCode::UnconsumedLeaf,
                identity,
                "unconsumed is the fail-closed default and cannot pass",
            ));
            return;
        }
        SourceLeafDisposition::Promoted
        | SourceLeafDisposition::ProvenanceOnly
        | SourceLeafDisposition::Ignored => {}
    }

    if receipts.is_empty() {
        let code = match declaration.disposition {
            SourceLeafDisposition::Promoted => CoverageFailureCode::ReaderNotObserved,
            SourceLeafDisposition::ProvenanceOnly => CoverageFailureCode::ProvenanceNotDurable,
            SourceLeafDisposition::Ignored => CoverageFailureCode::FixtureNotSourceGrounded,
            SourceLeafDisposition::Deferred | SourceLeafDisposition::Unconsumed => unreachable!(),
        };
        failures.push(CoverageFailure::for_identity(
            code,
            identity,
            "the exact declaration has no authenticated actual-read or negative-proof receipt",
        ));
        return;
    }

    validate_receipt_set(declaration, receipts, identity.clone(), failures);
    for receipt in receipts {
        evaluate_receipt(identity.clone(), declaration, receipt, failures);
    }
}

fn validate_receipt_set(
    declaration: &SourceLeafContract,
    receipts: &[&SourceLeafReceipt],
    identity: SourceLeafIdentity,
    failures: &mut Vec<CoverageFailure>,
) {
    let expected_cases = declaration
        .fixtures
        .iter()
        .map(|fixture| fixture.case_id.as_str())
        .collect::<BTreeSet<_>>();
    let actual_cases = receipts
        .iter()
        .map(|receipt| receipt.fixture().case_id())
        .collect::<BTreeSet<_>>();
    if expected_cases != actual_cases {
        failures.push(CoverageFailure::for_identity(
            CoverageFailureCode::ReceiptSetMismatch,
            identity.clone(),
            "receipt cases do not exactly cover the declared fixture set",
        ));
    }

    let mut receipt_keys = BTreeSet::new();
    for receipt in receipts {
        let (kind, member, ordinal) = match receipt.source() {
            SourcePresence::Value(value) => (
                value.member_kind,
                value.member_identity.as_deref(),
                value.ordinal,
            ),
            SourcePresence::Missing | SourcePresence::Null => (None, None, None),
        };
        if !receipt_keys.insert((
            receipt.fixture().case_id(),
            receipt.fixture().record_key(),
            kind,
            member,
            ordinal,
        )) {
            failures.push(CoverageFailure::for_identity(
                CoverageFailureCode::ReceiptSetMismatch,
                identity.clone(),
                "duplicate receipt evidence exists for one fixture/member occurrence",
            ));
        }
    }

    let pinned = receipts
        .iter()
        .filter(|receipt| receipt.fixture().provenance() == FixtureProvenance::PinnedSource)
        .copied()
        .collect::<Vec<_>>();
    let pinned_records = pinned
        .iter()
        .map(|receipt| receipt.fixture().record_key())
        .collect::<BTreeSet<_>>();
    if pinned.len() != declaration.source_prevalence.occurrence_count
        || pinned_records.len() != declaration.source_prevalence.record_count
    {
        failures.push(CoverageFailure::for_identity(
            CoverageFailureCode::SourcePrevalenceMismatch,
            identity.clone(),
            format!(
                "authenticated receipt prevalence is {}/{} records/occurrences, declaration requires {}/{}",
                pinned_records.len(),
                pinned.len(),
                declaration.source_prevalence.record_count,
                declaration.source_prevalence.occurrence_count
            ),
        ));
    }

    let mut array_ordinals = BTreeMap::<&str, Vec<usize>>::new();
    let mut member_identities = BTreeSet::new();
    for receipt in receipts {
        let SourcePresence::Value(value) = receipt.source() else {
            continue;
        };
        if value.multiplicity != 1 {
            failures.push(CoverageFailure::for_identity(
                CoverageFailureCode::ReceiptSetMismatch,
                identity.clone(),
                "one receipt proves exactly one occurrence and cannot self-claim multiplicity",
            ));
        }
        if let (Some(kind), Some(member)) = (value.member_kind, value.member_identity.as_deref())
            && !member_identities.insert((receipt.fixture().record_key(), kind, member))
        {
            failures.push(CoverageFailure::for_identity(
                CoverageFailureCode::ReceiptSetMismatch,
                identity.clone(),
                "collection member identities must be unique within each source record",
            ));
        }
        if value.member_kind == Some(SourceMemberKind::Array)
            && let Some(ordinal) = value.ordinal
        {
            array_ordinals
                .entry(receipt.fixture().record_key())
                .or_default()
                .push(ordinal);
        }
    }
    for ordinals in array_ordinals.values() {
        if ordinals.iter().copied().ne(0..ordinals.len()) {
            failures.push(CoverageFailure::for_identity(
                CoverageFailureCode::ReceiptSetMismatch,
                identity.clone(),
                "array receipts must be complete and emitted in contiguous authored order",
            ));
        }
    }
}

fn evaluate_receipt(
    identity: SourceLeafIdentity,
    declaration: &SourceLeafContract,
    receipt: &SourceLeafReceipt,
    failures: &mut Vec<CoverageFailure>,
) {
    if !receipt.integrity_is_valid()
        || !receipt.fixture().has_authenticated_source_binding()
        || receipt.reader_binding().trim().is_empty()
        || receipt.reader_mutation_digest() == receipt.fixture().excerpt_digest()
    {
        failures.push(CoverageFailure::for_identity(
            CoverageFailureCode::ReceiptProvenanceInvalid,
            identity.clone(),
            "receipt is not sealed to an accessor execution and distinct mutation sentinel",
        ));
    }
    let fixture_matches = declaration.fixtures.iter().any(|fixture| {
        fixture.case_id == receipt.fixture().case_id()
            && fixture.record_key == receipt.fixture().record_key()
            && fixture.source_path == receipt.fixture().source_path()
            && fixture.source_file_digest == receipt.fixture().source_file_digest()
            && fixture.excerpt_digest == receipt.fixture().excerpt_digest()
            && fixture.provenance == receipt.fixture().provenance()
    });
    if !fixture_matches {
        failures.push(CoverageFailure::for_identity(
            CoverageFailureCode::FixtureNotSourceGrounded,
            identity.clone(),
            "receipt fixture does not match the declaration-bound record/path/digest/provenance",
        ));
    }
    if !shape_is_allowed(declaration, receipt.source())
        || !source_value_is_well_formed(receipt.source())
    {
        failures.push(CoverageFailure::for_identity(
            CoverageFailureCode::FixtureNotSourceGrounded,
            identity.clone(),
            "observed source state/type/member metadata is not valid for expected_shapes",
        ));
    }
    validate_map_policy(declaration, receipt, identity.clone(), failures);

    match declaration.disposition {
        SourceLeafDisposition::Promoted => {
            if receipt.reader_purpose() != SourceAccessorPurpose::Parser
                || Some(receipt.reader_id()) != declaration.reader.reader_id.as_deref()
            {
                failures.push(CoverageFailure::for_identity(
                    CoverageFailureCode::ReaderNotObserved,
                    identity.clone(),
                    "the declared parser accessor did not generate this receipt",
                ));
            }
            compare_final_owners(declaration, receipt, identity, failures);
        }
        SourceLeafDisposition::ProvenanceOnly => {
            if receipt.reader_purpose() != SourceAccessorPurpose::ProvenanceReader
                || Some(receipt.reader_id()) != declaration.reader.reader_id.as_deref()
                || receipt
                    .semantic_output()
                    .is_none_or(|(observed, mutation, binding)| {
                        binding.trim().is_empty() || observed || observed == mutation
                    })
            {
                failures.push(CoverageFailure::for_identity(
                    CoverageFailureCode::ProvenanceNotDurable,
                    identity.clone(),
                    "provenance-only evidence requires its exact provenance reader and an accessor-bound negative semantic proof",
                ));
            }
            compare_provenance(declaration, receipt, identity, failures);
        }
        SourceLeafDisposition::Ignored => {
            if receipt.reader_purpose() != SourceAccessorPurpose::Inventory
                || !receipt.observations().is_empty()
                || receipt
                    .semantic_output()
                    .is_none_or(|(observed, mutation, binding)| {
                        binding.trim().is_empty() || observed || observed == mutation
                    })
            {
                failures.push(CoverageFailure::for_identity(
                    CoverageFailureCode::IgnoredPromoted,
                    identity,
                    "ignored evidence requires inventory observation and an accessor-bound negative semantic proof",
                ));
            }
        }
        SourceLeafDisposition::Deferred | SourceLeafDisposition::Unconsumed => unreachable!(),
    }
}

fn validate_map_policy(
    declaration: &SourceLeafContract,
    receipt: &SourceLeafReceipt,
    identity: SourceLeafIdentity,
    failures: &mut Vec<CoverageFailure>,
) {
    let Some(policy) = &declaration.map_key_policy else {
        return;
    };
    let SourcePresence::Value(value) = receipt.source() else {
        return;
    };
    let Some(key) = value.member_identity.as_deref() else {
        failures.push(CoverageFailure::for_identity(
            CoverageFailureCode::MapKeyPolicyMismatch,
            identity,
            "map receipt is missing its actual source member identity",
        ));
        return;
    };
    let accepted = match policy {
        MapKeyPolicy::ClosedVocabulary { keys } => keys.iter().any(|known| known == key),
        MapKeyPolicy::OpenVocabularyRetainedIdentity => true,
        MapKeyPolicy::TypedUnsupported => value.unsupported.is_some(),
    };
    if !accepted {
        failures.push(CoverageFailure::for_identity(
            CoverageFailureCode::MapKeyPolicyMismatch,
            identity,
            match policy {
                MapKeyPolicy::ClosedVocabulary { .. } => {
                    format!("map member {key} is outside the closed vocabulary")
                }
                MapKeyPolicy::TypedUnsupported => {
                    format!("map member {key} was not retained as typed unsupported")
                }
                MapKeyPolicy::OpenVocabularyRetainedIdentity => unreachable!(),
            },
        ));
    }
}

fn source_value_is_well_formed(source: &SourcePresence<SourceLeafValue>) -> bool {
    let SourcePresence::Value(value) = source else {
        return true;
    };
    if value.multiplicity == 0
        || (value.value.is_none() && value.stable_digest.is_none() && value.unsupported.is_none())
        || value
            .unsupported
            .as_ref()
            .is_some_and(|unsupported| unsupported.reason.trim().is_empty())
    {
        return false;
    }
    match value.member_kind {
        None => value.member_identity.is_none() && value.ordinal.is_none(),
        Some(SourceMemberKind::Array) => {
            value
                .member_identity
                .as_deref()
                .is_some_and(|member| !member.is_empty())
                && value.ordinal.is_some()
        }
        Some(SourceMemberKind::Map) => {
            value
                .member_identity
                .as_deref()
                .is_some_and(|member| !member.is_empty())
                && value.ordinal.is_none()
        }
    }
}

fn shape_is_allowed(
    declaration: &SourceLeafContract,
    source: &SourcePresence<SourceLeafValue>,
) -> bool {
    use super::{ExpectedSourceShape, SourceJsonType};

    match source {
        SourcePresence::Missing => declaration
            .expected_shapes
            .contains(&ExpectedSourceShape::Missing),
        SourcePresence::Null => declaration
            .expected_shapes
            .contains(&ExpectedSourceShape::Null),
        SourcePresence::Value(value) => {
            let value_shape = match value.json_type {
                SourceJsonType::Boolean => ExpectedSourceShape::Boolean,
                SourceJsonType::Number => ExpectedSourceShape::Number,
                SourceJsonType::String => ExpectedSourceShape::String,
                SourceJsonType::Array => ExpectedSourceShape::Array,
                SourceJsonType::Object => ExpectedSourceShape::Object,
            };
            let member_allowed = match value.member_kind {
                None => true,
                Some(SourceMemberKind::Array) => declaration
                    .expected_shapes
                    .contains(&ExpectedSourceShape::ArrayMember),
                Some(SourceMemberKind::Map) => declaration
                    .expected_shapes
                    .contains(&ExpectedSourceShape::MapMember),
            };
            declaration.expected_shapes.contains(&value_shape) && member_allowed
        }
    }
}

fn compare_final_owners(
    declaration: &SourceLeafContract,
    receipt: &SourceLeafReceipt,
    identity: SourceLeafIdentity,
    failures: &mut Vec<CoverageFailure>,
) {
    for observation in receipt.observations() {
        if !declaration.final_owners.iter().any(|owner| {
            owner.stage == observation.stage() && owner.destination == observation.destination()
        }) {
            failures.push(CoverageFailure::for_identity(
                observation.stage().mismatch_code(),
                identity.clone(),
                "receipt emitted an undeclared final-owner observation",
            ));
        }
    }
    for expected in &declaration.final_owners {
        let observations = receipt
            .observations()
            .iter()
            .filter(|observation| observation.stage() == expected.stage)
            .collect::<Vec<_>>();
        let code = expected.stage.mismatch_code();
        if observations.len() != 1 {
            failures.push(CoverageFailure::for_identity(
                code,
                identity.clone(),
                format!(
                    "expected exactly one {:?} accessor observation, found {}",
                    expected.stage,
                    observations.len()
                ),
            ));
            continue;
        }
        let observation = observations[0];
        if observation.destination() != expected.destination
            || observation.accessor_binding().trim().is_empty()
            || observation.mutation_digest().trim().is_empty()
        {
            failures.push(CoverageFailure::for_identity(
                code,
                identity.clone(),
                format!("wrong accessor-bound {:?} owner", expected.stage),
            ));
            continue;
        }
        if let Some(dimensions) = parity_mismatches(receipt.source(), observation.value()) {
            failures.push(CoverageFailure::for_identity(
                code,
                identity.clone(),
                format!(
                    "{:?} parity mismatch in {}",
                    expected.stage,
                    dimensions.join(", ")
                ),
            ));
        }
    }
}

fn compare_provenance(
    declaration: &SourceLeafContract,
    receipt: &SourceLeafReceipt,
    identity: SourceLeafIdentity,
    failures: &mut Vec<CoverageFailure>,
) {
    if receipt.observations().len() != 1
        || receipt.observations()[0].stage() != FinalOwnerStage::DurableProvenance
        || declaration.final_owners.len() != 1
        || declaration.final_owners[0].stage != FinalOwnerStage::DurableProvenance
        || receipt.observations()[0].destination() != declaration.final_owners[0].destination
        || receipt.observations()[0]
            .accessor_binding()
            .trim()
            .is_empty()
        || receipt.observations()[0]
            .mutation_digest()
            .trim()
            .is_empty()
        || parity_mismatches(receipt.source(), receipt.observations()[0].value()).is_some()
    {
        failures.push(CoverageFailure::for_identity(
            CoverageFailureCode::ProvenanceNotDurable,
            identity,
            "provenance-only receipt must end solely at its exact durable read-back owner with parity",
        ));
    }
}

fn parity_mismatches(
    expected: &SourcePresence<SourceLeafValue>,
    observed: &SourcePresence<SourceLeafValue>,
) -> Option<Vec<&'static str>> {
    match (expected, observed) {
        (SourcePresence::Missing, SourcePresence::Missing)
        | (SourcePresence::Null, SourcePresence::Null) => None,
        (SourcePresence::Missing, _)
        | (SourcePresence::Null, _)
        | (_, SourcePresence::Missing)
        | (_, SourcePresence::Null) => Some(vec!["state"]),
        (SourcePresence::Value(expected), SourcePresence::Value(observed)) => {
            let mut dimensions = Vec::new();
            if expected.json_type != observed.json_type
                || expected.member_kind != observed.member_kind
            {
                dimensions.push("type");
            }
            if expected.value != observed.value
                || expected.stable_digest != observed.stable_digest
                || expected.unsupported != observed.unsupported
            {
                dimensions.push("value");
            }
            if expected.member_identity != observed.member_identity {
                dimensions.push("identity");
            }
            if expected.ordinal != observed.ordinal {
                dimensions.push("order");
            }
            if expected.multiplicity != observed.multiplicity {
                dimensions.push("multiplicity");
            }
            (!dimensions.is_empty()).then_some(dimensions)
        }
    }
}

impl FinalOwnerStage {
    pub(crate) fn mismatch_code(self) -> CoverageFailureCode {
        match self {
            Self::SourceDto => CoverageFailureCode::DtoMismatch,
            Self::Canonical => CoverageFailureCode::CanonicalMismatch,
            Self::PostProjection => CoverageFailureCode::PostProjectionMismatch,
            Self::ArtifactHydration => CoverageFailureCode::ArtifactHydrationMismatch,
            Self::PublicSurface => CoverageFailureCode::PublicSurfaceMismatch,
            Self::DurableProvenance => CoverageFailureCode::ProvenanceNotDurable,
        }
    }
}

#[cfg(test)]
mod tests {
    use serde_json::Value;
    use sha2::{Digest, Sha256};

    use super::*;
    use crate::source_coverage::{
        ExpectedSourceShape, FinalOwnerAccessor, FinalOwnerContract, FixtureContract,
        FixtureReference, ReaderContract, SemanticOutputAccessor, SourceDocumentRole,
        SourceJsonType, SourceLeafAccessor, SourceLeafSelector, SourceParentContextSelector,
        SourcePin, SourcePrevalence, SurfaceContract, SurfaceDecision, SurfaceDisposition,
        TypedUnsupportedValue,
    };

    #[derive(Clone)]
    struct OwnerValue(SourcePresence<SourceLeafValue>);

    fn read_value(excerpt: &[u8]) -> Result<SourcePresence<SourceLeafValue>, String> {
        let document: Value = serde_json::from_slice(excerpt).map_err(|error| error.to_string())?;
        match document.get("state").and_then(Value::as_str) {
            Some("missing") => return Ok(SourcePresence::Missing),
            Some("null") => return Ok(SourcePresence::Null),
            _ => {}
        }
        let raw = document
            .get("value")
            .cloned()
            .ok_or_else(|| "fixture value is missing".to_string())?;
        let json_type = match raw {
            Value::Bool(_) => SourceJsonType::Boolean,
            Value::Number(_) => SourceJsonType::Number,
            Value::String(_) => SourceJsonType::String,
            Value::Array(_) => SourceJsonType::Array,
            Value::Object(_) => SourceJsonType::Object,
            Value::Null => return Ok(SourcePresence::Null),
        };
        let member_kind = match document.get("kind").and_then(Value::as_str) {
            Some("array") => Some(SourceMemberKind::Array),
            Some("map") => Some(SourceMemberKind::Map),
            _ => None,
        };
        let unsupported = document
            .get("unsupported_reason")
            .and_then(Value::as_str)
            .map(|reason| TypedUnsupportedValue {
                value: raw.clone(),
                reason: reason.to_string(),
            });
        Ok(SourcePresence::Value(SourceLeafValue {
            json_type,
            value: Some(raw),
            stable_digest: None,
            member_kind,
            member_identity: document
                .get("id")
                .and_then(Value::as_str)
                .map(str::to_string),
            ordinal: document
                .get("ordinal")
                .and_then(Value::as_u64)
                .map(|value| value as usize),
            multiplicity: document
                .get("multiplicity")
                .and_then(Value::as_u64)
                .unwrap_or(1) as usize,
            unsupported,
        }))
    }

    struct ParserAccessor;

    impl SourceLeafAccessor for ParserAccessor {
        const ID: &'static str = "source::dto::FixtureAccessor::read";
        const PURPOSE: SourceAccessorPurpose = SourceAccessorPurpose::Parser;

        fn read(excerpt: &[u8]) -> Result<SourcePresence<SourceLeafValue>, String> {
            read_value(excerpt)
        }
    }

    struct WrongParserAccessor;

    impl SourceLeafAccessor for WrongParserAccessor {
        const ID: &'static str = "source::dto::WrongAccessor::read";
        const PURPOSE: SourceAccessorPurpose = SourceAccessorPurpose::Parser;

        fn read(excerpt: &[u8]) -> Result<SourcePresence<SourceLeafValue>, String> {
            read_value(excerpt)
        }
    }

    struct ProvenanceAccessor;

    impl SourceLeafAccessor for ProvenanceAccessor {
        const ID: &'static str = "source::provenance::FixtureAccessor::read";
        const PURPOSE: SourceAccessorPurpose = SourceAccessorPurpose::ProvenanceReader;

        fn read(excerpt: &[u8]) -> Result<SourcePresence<SourceLeafValue>, String> {
            read_value(excerpt)
        }
    }

    struct InventoryAccessor;

    impl SourceLeafAccessor for InventoryAccessor {
        const ID: &'static str = "source::inventory::FixtureAccessor::observe";
        const PURPOSE: SourceAccessorPurpose = SourceAccessorPurpose::Inventory;

        fn read(excerpt: &[u8]) -> Result<SourcePresence<SourceLeafValue>, String> {
            read_value(excerpt)
        }
    }

    struct ConstantAccessor;

    impl SourceLeafAccessor for ConstantAccessor {
        const ID: &'static str = "source::dto::ConstantAccessor::read";
        const PURPOSE: SourceAccessorPurpose = SourceAccessorPurpose::Parser;

        fn read(_: &[u8]) -> Result<SourcePresence<SourceLeafValue>, String> {
            Ok(SourcePresence::Missing)
        }
    }

    macro_rules! owner_accessor {
        ($name:ident, $stage:expr, $destination:literal) => {
            struct $name;
            impl FinalOwnerAccessor<OwnerValue> for $name {
                const STAGE: FinalOwnerStage = $stage;
                const DESTINATION: &'static str = $destination;

                fn observe(value: &OwnerValue) -> SourcePresence<SourceLeafValue> {
                    value.0.clone()
                }
            }
        };
    }

    owner_accessor!(DtoOwner, FinalOwnerStage::SourceDto, "FixtureDto.value");
    owner_accessor!(
        CanonicalOwner,
        FinalOwnerStage::Canonical,
        "FixtureRecord.value"
    );
    owner_accessor!(
        PostOwner,
        FinalOwnerStage::PostProjection,
        "IndexBuildInput.value"
    );
    owner_accessor!(
        HydrationOwner,
        FinalOwnerStage::ArtifactHydration,
        "HydratedRecord.value"
    );
    owner_accessor!(
        PublicOwner,
        FinalOwnerStage::PublicSurface,
        "RecordSurface.value"
    );
    owner_accessor!(
        ProvenanceOwner,
        FinalOwnerStage::DurableProvenance,
        "AtlasRecord.raw_json"
    );
    owner_accessor!(
        WrongDtoOwner,
        FinalOwnerStage::SourceDto,
        "DeclarationString.value"
    );
    owner_accessor!(
        WrongCanonicalOwner,
        FinalOwnerStage::Canonical,
        "DeclarationString.value"
    );
    owner_accessor!(
        WrongPostOwner,
        FinalOwnerStage::PostProjection,
        "DeclarationString.value"
    );
    owner_accessor!(
        WrongHydrationOwner,
        FinalOwnerStage::ArtifactHydration,
        "DeclarationString.value"
    );
    owner_accessor!(
        WrongPublicOwner,
        FinalOwnerStage::PublicSurface,
        "DeclarationString.value"
    );

    struct WrongValueDtoOwner;

    impl FinalOwnerAccessor<OwnerValue> for WrongValueDtoOwner {
        const STAGE: FinalOwnerStage = FinalOwnerStage::SourceDto;
        const DESTINATION: &'static str = "FixtureDto.value";

        fn observe(value: &OwnerValue) -> SourcePresence<SourceLeafValue> {
            let mut observed = value.0.clone();
            if let SourcePresence::Value(value) = &mut observed {
                value.json_type = SourceJsonType::Number;
                value.multiplicity = 2;
            }
            observed
        }
    }

    struct SemanticState(bool);
    struct SemanticProbe;

    impl SemanticOutputAccessor<SemanticState> for SemanticProbe {
        fn semantic_output_observed(value: &SemanticState) -> bool {
            value.0
        }
    }

    fn digest(excerpt: &[u8]) -> String {
        format!("sha256:{:x}", Sha256::digest(excerpt))
    }

    fn fixture_contract(case_id: &str, record_key: &str, excerpt: &[u8]) -> FixtureContract {
        FixtureContract {
            case_id: case_id.to_string(),
            record_key: record_key.to_string(),
            source_path: format!("packs/fixture/{case_id}.json"),
            source_file_digest: digest(excerpt),
            excerpt_digest: digest(excerpt),
            provenance: FixtureProvenance::PinnedSource,
        }
    }

    fn promoted_ledger(fixtures: Vec<FixtureContract>) -> SourceLeafCoverageLedger {
        let occurrence_count = fixtures.len();
        let record_count = fixtures
            .iter()
            .map(|fixture| fixture.record_key.as_str())
            .collect::<BTreeSet<_>>()
            .len();
        SourceLeafCoverageLedger {
            contract_version: "atlas-source-leaf-coverage/v1".to_string(),
            type_id: "actor--npc--top-level--root--root--root".to_string(),
            source_pin: SourcePin::pinned(),
            selector: SourceLeafSelector {
                source_contract_version: "pf2e-serialized-source/v1".to_string(),
                document_class: "Actor".to_string(),
                type_discriminator: "npc".to_string(),
                role: SourceDocumentRole::TopLevel,
                parent_context: SourceParentContextSelector::root(),
            },
            leaves: vec![SourceLeafContract {
                normalized_path: "$.system.value".to_string(),
                expected_shapes: vec![
                    ExpectedSourceShape::Missing,
                    ExpectedSourceShape::Null,
                    ExpectedSourceShape::String,
                    ExpectedSourceShape::Number,
                    ExpectedSourceShape::Object,
                ],
                source_prevalence: SourcePrevalence {
                    record_count,
                    occurrence_count,
                },
                map_key_policy: None,
                disposition: SourceLeafDisposition::Promoted,
                reader: ReaderContract {
                    reader_id: Some(ParserAccessor::ID.to_string()),
                    parity_case_ids: fixtures
                        .iter()
                        .map(|fixture| fixture.case_id.clone())
                        .collect(),
                },
                fixtures,
                final_owners: vec![
                    FinalOwnerContract {
                        stage: FinalOwnerStage::SourceDto,
                        destination: "FixtureDto.value".to_string(),
                    },
                    FinalOwnerContract {
                        stage: FinalOwnerStage::Canonical,
                        destination: "FixtureRecord.value".to_string(),
                    },
                    FinalOwnerContract {
                        stage: FinalOwnerStage::PostProjection,
                        destination: "IndexBuildInput.value".to_string(),
                    },
                    FinalOwnerContract {
                        stage: FinalOwnerStage::ArtifactHydration,
                        destination: "HydratedRecord.value".to_string(),
                    },
                    FinalOwnerContract {
                        stage: FinalOwnerStage::PublicSurface,
                        destination: "RecordSurface.value".to_string(),
                    },
                ],
                surfaces: SurfaceContract {
                    artifact: promoted_surface(),
                    app: promoted_surface(),
                    ..SurfaceContract::default()
                },
                rationale: "typed product fact".to_string(),
                owner: "atlas-ingest::fixture".to_string(),
                acceptance_checkpoint: "A1".to_string(),
                future_owner: None,
                future_task: None,
                prerequisite: None,
            }],
        }
    }

    fn promoted_surface() -> SurfaceDecision {
        SurfaceDecision {
            disposition: SurfaceDisposition::Promoted,
            rationale: "selected".to_string(),
        }
    }

    fn capture<A: SourceLeafAccessor>(
        ledger: &SourceLeafCoverageLedger,
        fixture_index: usize,
        excerpt: &[u8],
        sentinel: &[u8],
    ) -> crate::source_coverage::SourceLeafReceiptBuilder {
        let fixture = FixtureReference::authenticate(
            &ledger.leaves[0].fixtures[fixture_index],
            excerpt,
            excerpt,
        )
        .expect("authenticated fixture");
        SourceLeafReceipt::capture::<A>(ledger.identity_for(&ledger.leaves[0]), fixture, sentinel)
            .expect("actual reader")
    }

    fn complete_receipt(
        ledger: &SourceLeafCoverageLedger,
        fixture_index: usize,
        excerpt: &[u8],
        sentinel: &[u8],
    ) -> SourceLeafReceipt {
        let mut builder = capture::<ParserAccessor>(ledger, fixture_index, excerpt, sentinel);
        let owner = OwnerValue(read_value(excerpt).expect("source value"));
        let mutation = OwnerValue(read_value(sentinel).expect("sentinel value"));
        observe_standard_owners(&mut builder, &owner, &mutation);
        builder.finish()
    }

    fn observe_standard_owners(
        builder: &mut crate::source_coverage::SourceLeafReceiptBuilder,
        owner: &OwnerValue,
        mutation: &OwnerValue,
    ) {
        builder
            .observe_final_owner::<_, DtoOwner>(&owner, &mutation)
            .expect("dto accessor");
        builder
            .observe_final_owner::<_, CanonicalOwner>(&owner, &mutation)
            .expect("canonical accessor");
        builder
            .observe_final_owner::<_, PostOwner>(&owner, &mutation)
            .expect("post accessor");
        builder
            .observe_final_owner::<_, HydrationOwner>(&owner, &mutation)
            .expect("hydration accessor");
        builder
            .observe_final_owner::<_, PublicOwner>(&owner, &mutation)
            .expect("public accessor");
    }

    fn codes(report: &CoverageReport) -> Vec<CoverageFailureCode> {
        report.failures.iter().map(|failure| failure.code).collect()
    }

    #[test]
    fn accessor_bound_receipt_passes_all_exact_owners() {
        let excerpt = br#"{"value":"alpha"}"#;
        let sentinel = br#"{"value":"beta"}"#;
        let ledger = promoted_ledger(vec![fixture_contract("case-1", "pack:id", excerpt)]);
        let receipt = complete_receipt(&ledger, 0, excerpt, sentinel);
        assert!(evaluate_source_leaf_coverage(&ledger, &[receipt]).passed);
    }

    #[test]
    fn missing_null_and_value_states_remain_distinct() {
        for (excerpt, sentinel) in [
            (
                br#"{"state":"missing"}"#.as_slice(),
                br#"{"state":"null"}"#.as_slice(),
            ),
            (
                br#"{"state":"null"}"#.as_slice(),
                br#"{"value":"alpha"}"#.as_slice(),
            ),
            (
                br#"{"value":"alpha"}"#.as_slice(),
                br#"{"state":"missing"}"#.as_slice(),
            ),
        ] {
            let ledger = promoted_ledger(vec![fixture_contract("case-1", "pack:id", excerpt)]);
            let receipt = complete_receipt(&ledger, 0, excerpt, sentinel);
            assert!(evaluate_source_leaf_coverage(&ledger, &[receipt]).passed);
        }
    }

    #[test]
    fn undeclared_selector_and_wrong_reader_have_exact_codes() {
        let excerpt = br#"{"value":"alpha"}"#;
        let sentinel = br#"{"value":"beta"}"#;
        let ledger = promoted_ledger(vec![fixture_contract("case-1", "pack:id", excerpt)]);
        let mut undeclared = complete_receipt(&ledger, 0, excerpt, sentinel);
        undeclared.corrupt_identity_path("$.system");
        assert!(
            codes(&evaluate_source_leaf_coverage(&ledger, &[undeclared]))
                .contains(&CoverageFailureCode::UndeclaredLeaf)
        );

        let mut builder = capture::<WrongParserAccessor>(&ledger, 0, excerpt, sentinel);
        let owner = OwnerValue(read_value(excerpt).expect("source"));
        let mutation = OwnerValue(read_value(sentinel).expect("sentinel"));
        observe_standard_owners(&mut builder, &owner, &mutation);
        assert!(
            codes(&evaluate_source_leaf_coverage(&ledger, &[builder.finish()]))
                .contains(&CoverageFailureCode::ReaderNotObserved)
        );
    }

    #[test]
    fn every_wrong_final_owner_has_its_exact_failure_code() {
        let excerpt = br#"{"value":"alpha"}"#;
        let sentinel = br#"{"value":"beta"}"#;
        let ledger = promoted_ledger(vec![fixture_contract("case-1", "pack:id", excerpt)]);
        macro_rules! assert_wrong_owner {
            ($wrong:ty, $code:expr) => {{
                let mut builder = capture::<ParserAccessor>(&ledger, 0, excerpt, sentinel);
                let owner = OwnerValue(read_value(excerpt).expect("source"));
                let mutation = OwnerValue(read_value(sentinel).expect("sentinel"));
                builder
                    .observe_final_owner::<_, $wrong>(&owner, &mutation)
                    .expect("wrong owner accessor");
                observe_standard_owners(&mut builder, &owner, &mutation);
                assert!(
                    codes(&evaluate_source_leaf_coverage(&ledger, &[builder.finish()]))
                        .contains(&$code)
                );
            }};
        }
        assert_wrong_owner!(WrongDtoOwner, CoverageFailureCode::DtoMismatch);
        assert_wrong_owner!(WrongCanonicalOwner, CoverageFailureCode::CanonicalMismatch);
        assert_wrong_owner!(WrongPostOwner, CoverageFailureCode::PostProjectionMismatch);
        assert_wrong_owner!(
            WrongHydrationOwner,
            CoverageFailureCode::ArtifactHydrationMismatch
        );
        assert_wrong_owner!(WrongPublicOwner, CoverageFailureCode::PublicSurfaceMismatch);
    }

    #[test]
    fn parity_rejects_value_type_identity_order_and_multiplicity() {
        let excerpt = br#"{"value":"alpha"}"#;
        let sentinel = br#"{"value":"beta"}"#;
        let ledger = promoted_ledger(vec![fixture_contract("case-1", "pack:id", excerpt)]);
        let mut builder = capture::<ParserAccessor>(&ledger, 0, excerpt, sentinel);
        let owner = OwnerValue(read_value(excerpt).expect("source"));
        let mutation = OwnerValue(read_value(sentinel).expect("sentinel"));
        builder
            .observe_final_owner::<_, WrongValueDtoOwner>(&owner, &mutation)
            .expect("wrong value accessor");
        builder
            .observe_final_owner::<_, CanonicalOwner>(&owner, &mutation)
            .expect("canonical accessor");
        builder
            .observe_final_owner::<_, PostOwner>(&owner, &mutation)
            .expect("post accessor");
        builder
            .observe_final_owner::<_, HydrationOwner>(&owner, &mutation)
            .expect("hydration accessor");
        builder
            .observe_final_owner::<_, PublicOwner>(&owner, &mutation)
            .expect("public accessor");
        assert!(
            codes(&evaluate_source_leaf_coverage(&ledger, &[builder.finish()]))
                .contains(&CoverageFailureCode::DtoMismatch)
        );
    }

    #[test]
    fn map_policy_checks_actual_keys_and_typed_unsupported() {
        let excerpt = br#"{"value":4,"kind":"map","id":"intimidate"}"#;
        let sentinel = br#"{"value":5,"kind":"map","id":"intimidate"}"#;
        let mut ledger = promoted_ledger(vec![fixture_contract("case-1", "pack:id", excerpt)]);
        ledger.leaves[0].normalized_path = "$.system.skills.*".to_string();
        ledger.leaves[0]
            .expected_shapes
            .push(ExpectedSourceShape::MapMember);
        ledger.leaves[0].map_key_policy = Some(MapKeyPolicy::ClosedVocabulary {
            keys: vec!["acrobatics".to_string(), "intimidation".to_string()],
        });
        let receipt = complete_receipt(&ledger, 0, excerpt, sentinel);
        assert!(
            codes(&evaluate_source_leaf_coverage(&ledger, &[receipt]))
                .contains(&CoverageFailureCode::MapKeyPolicyMismatch)
        );

        ledger.leaves[0].map_key_policy = Some(MapKeyPolicy::TypedUnsupported);
        let receipt = complete_receipt(&ledger, 0, excerpt, sentinel);
        assert!(
            codes(&evaluate_source_leaf_coverage(&ledger, &[receipt]))
                .contains(&CoverageFailureCode::MapKeyPolicyMismatch)
        );

        let unsupported =
            br#"{"value":4,"kind":"map","id":"intimidate","unsupported_reason":"unknown_skill"}"#;
        let unsupported_sentinel =
            br#"{"value":5,"kind":"map","id":"intimidate","unsupported_reason":"unknown_skill"}"#;
        ledger.leaves[0].fixtures[0] = fixture_contract("case-1", "pack:id", unsupported);
        let receipt = complete_receipt(&ledger, 0, unsupported, unsupported_sentinel);
        assert!(evaluate_source_leaf_coverage(&ledger, &[receipt]).passed);
    }

    #[test]
    fn aggregate_receipts_enforce_prevalence_uniqueness_order_and_unit_multiplicity() {
        let first = br#"{"value":"a","kind":"array","id":"a","ordinal":0}"#;
        let second = br#"{"value":"b","kind":"array","id":"b","ordinal":1}"#;
        let mut ledger = promoted_ledger(vec![
            fixture_contract("case-1", "pack:id", first),
            fixture_contract("case-2", "pack:id", second),
        ]);
        ledger.leaves[0].normalized_path = "$.system.values[]".to_string();
        ledger.leaves[0]
            .expected_shapes
            .push(ExpectedSourceShape::ArrayMember);
        let first_receipt = complete_receipt(
            &ledger,
            0,
            first,
            br#"{"value":"changed","kind":"array","id":"a","ordinal":0}"#,
        );
        let second_receipt = complete_receipt(
            &ledger,
            1,
            second,
            br#"{"value":"changed","kind":"array","id":"b","ordinal":1}"#,
        );
        assert!(
            evaluate_source_leaf_coverage(
                &ledger,
                &[first_receipt.clone(), second_receipt.clone()]
            )
            .passed
        );
        assert!(
            codes(&evaluate_source_leaf_coverage(
                &ledger,
                &[second_receipt, first_receipt.clone()]
            ))
            .contains(&CoverageFailureCode::ReceiptSetMismatch)
        );
        assert!(
            codes(&evaluate_source_leaf_coverage(
                &ledger,
                &[first_receipt.clone(), first_receipt]
            ))
            .contains(&CoverageFailureCode::ReceiptSetMismatch)
        );
        let incomplete = complete_receipt(
            &ledger,
            0,
            first,
            br#"{"value":"changed","kind":"array","id":"a","ordinal":0}"#,
        );
        let incomplete_codes = codes(&evaluate_source_leaf_coverage(&ledger, &[incomplete]));
        assert!(incomplete_codes.contains(&CoverageFailureCode::ReceiptSetMismatch));
        assert!(incomplete_codes.contains(&CoverageFailureCode::SourcePrevalenceMismatch));

        let claimed = br#"{"value":"a","kind":"array","id":"a","ordinal":0,"multiplicity":2}"#;
        let mut one = promoted_ledger(vec![fixture_contract("case-1", "pack:id", claimed)]);
        one.leaves[0].normalized_path = "$.system.values[]".to_string();
        one.leaves[0]
            .expected_shapes
            .push(ExpectedSourceShape::ArrayMember);
        let receipt = complete_receipt(
            &one,
            0,
            claimed,
            br#"{"value":"changed","kind":"array","id":"a","ordinal":0,"multiplicity":2}"#,
        );
        assert!(
            codes(&evaluate_source_leaf_coverage(&one, &[receipt]))
                .contains(&CoverageFailureCode::ReceiptSetMismatch)
        );
    }

    #[test]
    fn authenticated_fixture_and_mutation_provenance_fail_closed() {
        let excerpt = br#"{"value":"alpha"}"#;
        let sentinel = br#"{"value":"beta"}"#;
        let ledger = promoted_ledger(vec![fixture_contract("case-1", "pack:id", excerpt)]);
        let error = FixtureReference::authenticate(
            &ledger.leaves[0].fixtures[0],
            br#"{"value":"tampered"}"#,
            br#"{"value":"tampered"}"#,
        )
        .expect_err("digest mismatch");
        assert_eq!(error.code, CoverageFailureCode::ReceiptProvenanceInvalid);

        let fixture =
            FixtureReference::authenticate(&ledger.leaves[0].fixtures[0], excerpt, excerpt)
                .expect("fixture");
        let error = SourceLeafReceipt::capture::<ConstantAccessor>(
            ledger.identity_for(&ledger.leaves[0]),
            fixture,
            sentinel,
        )
        .expect_err("constant accessor cannot prove a read");
        assert_eq!(error.code, CoverageFailureCode::ReaderNotObserved);

        let mut receipt = complete_receipt(&ledger, 0, excerpt, sentinel);
        receipt.corrupt_evidence_digest();
        assert!(
            codes(&evaluate_source_leaf_coverage(&ledger, &[receipt]))
                .contains(&CoverageFailureCode::ReceiptProvenanceInvalid)
        );
    }

    #[test]
    fn provenance_only_ends_at_durable_owner_without_semantic_output() {
        let excerpt = br#"{"value":{"raw":true},"unsupported_reason":"provenance"}"#;
        let sentinel = br#"{"value":{"raw":false},"unsupported_reason":"provenance"}"#;
        let mut ledger = promoted_ledger(vec![fixture_contract("case-1", "pack:id", excerpt)]);
        let leaf = &mut ledger.leaves[0];
        leaf.disposition = SourceLeafDisposition::ProvenanceOnly;
        leaf.reader.reader_id = Some(ProvenanceAccessor::ID.to_string());
        leaf.final_owners = vec![FinalOwnerContract {
            stage: FinalOwnerStage::DurableProvenance,
            destination: "AtlasRecord.raw_json".to_string(),
        }];
        leaf.surfaces = SurfaceContract::default();

        let mut builder = capture::<ProvenanceAccessor>(&ledger, 0, excerpt, sentinel);
        let owner = OwnerValue(read_value(excerpt).expect("source"));
        let mutation = OwnerValue(read_value(sentinel).expect("sentinel"));
        builder
            .observe_final_owner::<_, ProvenanceOwner>(&owner, &mutation)
            .expect("provenance accessor");
        builder
            .observe_semantic_output::<_, SemanticProbe>(
                &SemanticState(false),
                &SemanticState(true),
            )
            .expect("semantic probe");
        assert!(evaluate_source_leaf_coverage(&ledger, &[builder.finish()]).passed);

        let mut builder = capture::<ProvenanceAccessor>(&ledger, 0, excerpt, sentinel);
        builder
            .observe_final_owner::<_, ProvenanceOwner>(&owner, &mutation)
            .expect("provenance accessor");
        builder
            .observe_semantic_output::<_, SemanticProbe>(
                &SemanticState(true),
                &SemanticState(false),
            )
            .expect("semantic probe");
        assert!(
            codes(&evaluate_source_leaf_coverage(&ledger, &[builder.finish()]))
                .contains(&CoverageFailureCode::ProvenanceNotDurable)
        );
    }

    #[test]
    fn ignored_deferred_and_unconsumed_dispositions_fail_or_prove_exactly() {
        let excerpt = br#"{"value":"cache"}"#;
        let sentinel = br#"{"value":"changed"}"#;
        let mut ignored = promoted_ledger(vec![fixture_contract("case-1", "pack:id", excerpt)]);
        let leaf = &mut ignored.leaves[0];
        leaf.disposition = SourceLeafDisposition::Ignored;
        leaf.reader.reader_id = None;
        leaf.final_owners.clear();
        leaf.surfaces = SurfaceContract::default();
        let mut builder = capture::<InventoryAccessor>(&ignored, 0, excerpt, sentinel);
        builder
            .observe_semantic_output::<_, SemanticProbe>(
                &SemanticState(false),
                &SemanticState(true),
            )
            .expect("semantic probe");
        assert!(evaluate_source_leaf_coverage(&ignored, &[builder.finish()]).passed);

        for (disposition, code) in [
            (
                SourceLeafDisposition::Deferred,
                CoverageFailureCode::UnresolvedDeferred,
            ),
            (
                SourceLeafDisposition::Unconsumed,
                CoverageFailureCode::UnconsumedLeaf,
            ),
        ] {
            let mut ledger = promoted_ledger(vec![fixture_contract("case-1", "pack:id", excerpt)]);
            let leaf = &mut ledger.leaves[0];
            leaf.disposition = disposition;
            leaf.reader = ReaderContract::default();
            leaf.fixtures.clear();
            leaf.final_owners.clear();
            leaf.surfaces = SurfaceContract::default();
            if disposition == SourceLeafDisposition::Deferred {
                leaf.future_owner = Some("H9".to_string());
                leaf.future_task = Some("actor-npc".to_string());
                leaf.prerequisite = Some("A2".to_string());
            }
            assert!(codes(&evaluate_source_leaf_coverage(&ledger, &[])).contains(&code));
        }
    }
}
