use std::collections::{BTreeMap, BTreeSet};

use serde::{Deserialize, Serialize};

use crate::SourcePresence;

use super::{
    FinalOwnerStage, MapKeyPolicy, SourceAccessorPurpose, SourceLeafContract,
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

    let fixture_records = receipts
        .iter()
        .map(|receipt| receipt.fixture().record_key())
        .collect::<BTreeSet<_>>();
    if receipts.len() != declaration.fixture_prevalence.occurrence_count
        || fixture_records.len() != declaration.fixture_prevalence.record_count
    {
        failures.push(CoverageFailure::for_identity(
            CoverageFailureCode::ReceiptSetMismatch,
            identity.clone(),
            format!(
                "focused receipt prevalence is {}/{} records/occurrences, fixture declaration requires {}/{}; global source prevalence is independent metadata",
                fixture_records.len(),
                receipts.len(),
                declaration.fixture_prevalence.record_count,
                declaration.fixture_prevalence.occurrence_count
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
    if !map_policy_accepts(policy, key, value.unsupported.is_some()) {
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

fn map_policy_accepts(policy: &MapKeyPolicy, key: &str, typed_unsupported: bool) -> bool {
    match policy {
        MapKeyPolicy::ClosedVocabulary { keys } => keys.iter().any(|known| known == key),
        MapKeyPolicy::OpenVocabularyRetainedIdentity => true,
        MapKeyPolicy::TypedUnsupported => typed_unsupported,
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
                None => declaration.leaf_kind == super::SourceLeafKind::Scalar,
                Some(SourceMemberKind::Array) => {
                    declaration.leaf_kind == super::SourceLeafKind::ArrayMember
                }
                Some(SourceMemberKind::Map) => {
                    declaration.leaf_kind == super::SourceLeafKind::MapMember
                }
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
    use serde_json::json;

    use super::*;
    use crate::source_coverage::SourceJsonType;
    use crate::source_coverage::receipt::TypedUnsupportedValue;

    fn scalar(
        value: serde_json::Value,
        json_type: SourceJsonType,
    ) -> SourcePresence<SourceLeafValue> {
        SourcePresence::Value(SourceLeafValue {
            json_type,
            value: Some(value),
            stable_digest: None,
            member_kind: None,
            member_identity: None,
            ordinal: None,
            multiplicity: 1,
            unsupported: None,
        })
    }

    #[test]
    fn parity_preserves_missing_null_type_value_identity_order_and_multiplicity() {
        assert_eq!(
            parity_mismatches(&SourcePresence::Missing, &SourcePresence::Null),
            Some(vec!["state"])
        );
        let text = scalar(json!("4"), SourceJsonType::String);
        let number = scalar(json!(4), SourceJsonType::Number);
        assert_eq!(
            parity_mismatches(&text, &number),
            Some(vec!["type", "value"])
        );

        let mut expected = match scalar(json!(4), SourceJsonType::Number) {
            SourcePresence::Value(value) => value,
            _ => unreachable!(),
        };
        expected.member_kind = Some(SourceMemberKind::Array);
        expected.member_identity = Some("member-a".to_string());
        expected.ordinal = Some(0);
        let mut observed = expected.clone();
        observed.member_identity = Some("member-b".to_string());
        observed.ordinal = Some(1);
        observed.multiplicity = 2;
        assert_eq!(
            parity_mismatches(
                &SourcePresence::Value(expected),
                &SourcePresence::Value(observed),
            ),
            Some(vec!["identity", "order", "multiplicity"])
        );
    }

    #[test]
    fn map_policy_uses_actual_member_identity_and_typed_unsupported_state() {
        let closed = MapKeyPolicy::ClosedVocabulary {
            keys: vec!["intimidation".to_string()],
        };
        assert!(map_policy_accepts(&closed, "intimidation", false));
        assert!(!map_policy_accepts(&closed, "intimidate", false));
        assert!(!map_policy_accepts(
            &MapKeyPolicy::TypedUnsupported,
            "intimidate",
            false
        ));
        assert!(map_policy_accepts(
            &MapKeyPolicy::TypedUnsupported,
            "intimidate",
            true
        ));

        let unsupported = TypedUnsupportedValue {
            value: json!(4),
            reason: "unknown skill key".to_string(),
        };
        assert_eq!(unsupported.reason, "unknown skill key");
    }

    #[test]
    fn each_real_owner_stage_maps_to_its_typed_failure() {
        assert_eq!(
            FinalOwnerStage::SourceDto.mismatch_code(),
            CoverageFailureCode::DtoMismatch
        );
        assert_eq!(
            FinalOwnerStage::Canonical.mismatch_code(),
            CoverageFailureCode::CanonicalMismatch
        );
        assert_eq!(
            FinalOwnerStage::PostProjection.mismatch_code(),
            CoverageFailureCode::PostProjectionMismatch
        );
        assert_eq!(
            FinalOwnerStage::ArtifactHydration.mismatch_code(),
            CoverageFailureCode::ArtifactHydrationMismatch
        );
        assert_eq!(
            FinalOwnerStage::PublicSurface.mismatch_code(),
            CoverageFailureCode::PublicSurfaceMismatch
        );
        assert_eq!(
            FinalOwnerStage::DurableProvenance.mismatch_code(),
            CoverageFailureCode::ProvenanceNotDurable
        );
    }
}
