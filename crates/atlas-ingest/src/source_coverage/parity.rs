use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use crate::SourcePresence;

use super::{
    FinalOwnerStage, SourceLeafContract, SourceLeafCoverageLedger, SourceLeafDisposition,
    SourceLeafIdentity, SourceLeafReceipt, SourceLeafValue, lint_source_leaf_ledger,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CoverageFailureCode {
    InvalidContract,
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
        if !declarations.contains_key(&receipt.identity) {
            failures.push(CoverageFailure::for_identity(
                CoverageFailureCode::UndeclaredLeaf,
                receipt.identity.clone(),
                "no exact type/selector/parent-context/leaf declaration owns this receipt",
            ));
            continue;
        }
        receipts_by_identity
            .entry(receipt.identity.clone())
            .or_default()
            .push(receipt);
    }

    for (identity, declaration) in declarations {
        evaluate_declaration(
            ledger,
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
    ledger: &SourceLeafCoverageLedger,
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
            "the exact declaration has no actual-read or negative-proof receipt",
        ));
        return;
    }

    for case_id in &declaration.reader.parity_case_ids {
        if !receipts
            .iter()
            .any(|receipt| &receipt.fixture.case_id == case_id)
        {
            failures.push(CoverageFailure::for_identity(
                CoverageFailureCode::FixtureNotSourceGrounded,
                identity.clone(),
                format!("declared parity case {case_id} has no receipt"),
            ));
        }
    }

    for receipt in receipts {
        evaluate_receipt(ledger, identity.clone(), declaration, receipt, failures);
    }
}

fn evaluate_receipt(
    ledger: &SourceLeafCoverageLedger,
    identity: SourceLeafIdentity,
    declaration: &SourceLeafContract,
    receipt: &SourceLeafReceipt,
    failures: &mut Vec<CoverageFailure>,
) {
    if !receipt.inventory_observed {
        failures.push(CoverageFailure::for_identity(
            CoverageFailureCode::StaleDeclaration,
            identity.clone(),
            "the bound source inventory did not observe this declared leaf",
        ));
    }
    if !receipt.fixture.source_grounded
        || receipt.source_pin != ledger.source_pin
        || receipt.fixture.source_commit != ledger.source_pin.upstream_commit
        || receipt.fixture.record_key.trim().is_empty()
        || receipt.fixture.source_path.trim().is_empty()
        || receipt.fixture.excerpt_digest.trim().is_empty()
        || receipt.comparator_assertion_ids.is_empty()
        || !declaration
            .reader
            .parity_case_ids
            .contains(&receipt.fixture.case_id)
    {
        failures.push(CoverageFailure::for_identity(
            CoverageFailureCode::FixtureNotSourceGrounded,
            identity.clone(),
            "fixture identity, source pin, digest, or parity case does not match the declaration",
        ));
    }
    if !shape_is_allowed(declaration, &receipt.source) {
        failures.push(CoverageFailure::for_identity(
            CoverageFailureCode::FixtureNotSourceGrounded,
            identity.clone(),
            "observed source state/type is not listed in expected_shapes",
        ));
    }
    if !source_value_is_well_formed(&receipt.source) {
        failures.push(CoverageFailure::for_identity(
            CoverageFailureCode::FixtureNotSourceGrounded,
            identity.clone(),
            "populated source values require typed payload and valid member metadata",
        ));
    }

    match declaration.disposition {
        SourceLeafDisposition::Promoted => {
            if receipt.reader_id.as_deref() != declaration.reader.reader_id.as_deref() {
                failures.push(CoverageFailure::for_identity(
                    CoverageFailureCode::ReaderNotObserved,
                    identity.clone(),
                    "the declared reader identity was not observed",
                ));
            }
            compare_final_owners(declaration, receipt, identity, failures);
        }
        SourceLeafDisposition::ProvenanceOnly => {
            compare_provenance(declaration, receipt, identity, failures)
        }
        SourceLeafDisposition::Ignored => {
            if receipt.semantic_output_observed || !receipt.observations.is_empty() {
                failures.push(CoverageFailure::for_identity(
                    CoverageFailureCode::IgnoredPromoted,
                    identity,
                    "an ignored source leaf populated semantic output",
                ));
            }
        }
        SourceLeafDisposition::Deferred | SourceLeafDisposition::Unconsumed => unreachable!(),
    }
}

fn source_value_is_well_formed(source: &SourcePresence<SourceLeafValue>) -> bool {
    use super::SourceMemberKind;

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
                .is_some_and(|identity| !identity.is_empty())
                && value.ordinal.is_some()
        }
        Some(SourceMemberKind::Map) => value
            .member_identity
            .as_deref()
            .is_some_and(|identity| !identity.is_empty()),
    }
}

fn shape_is_allowed(
    declaration: &SourceLeafContract,
    source: &SourcePresence<SourceLeafValue>,
) -> bool {
    use super::{ExpectedSourceShape, SourceJsonType, SourceMemberKind};

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
    for expected in &declaration.final_owners {
        let mut observations = receipt
            .observations
            .iter()
            .filter(|observation| observation.stage == expected.stage);
        let observation = observations.next();
        let code = expected.stage.mismatch_code();
        let Some(observation) = observation else {
            failures.push(CoverageFailure::for_identity(
                code,
                identity.clone(),
                format!("no {:?} observation was emitted", expected.stage),
            ));
            continue;
        };
        if observations.next().is_some() {
            failures.push(CoverageFailure::for_identity(
                code,
                identity.clone(),
                format!("more than one {:?} observation was emitted", expected.stage),
            ));
            continue;
        }
        if observation.destination != expected.destination {
            failures.push(CoverageFailure::for_identity(
                code,
                identity.clone(),
                format!(
                    "wrong {:?} owner: expected {}, observed {}",
                    expected.stage, expected.destination, observation.destination
                ),
            ));
            continue;
        }
        if let Some(dimensions) = parity_mismatches(&receipt.source, &observation.value) {
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
    let observation = receipt
        .observations
        .iter()
        .find(|observation| observation.stage == FinalOwnerStage::DurableProvenance);
    let Some(observation) = observation else {
        failures.push(CoverageFailure::for_identity(
            CoverageFailureCode::ProvenanceNotDurable,
            identity,
            "no durable provenance read-back observation was emitted",
        ));
        return;
    };
    let expected_destination = declaration
        .final_owners
        .iter()
        .find(|owner| owner.stage == FinalOwnerStage::DurableProvenance)
        .map(|owner| owner.destination.as_str());
    if expected_destination != Some(observation.destination.as_str()) {
        failures.push(CoverageFailure::for_identity(
            CoverageFailureCode::ProvenanceNotDurable,
            identity.clone(),
            "durable provenance was observed at the wrong final owner",
        ));
        return;
    }
    if let Some(dimensions) = parity_mismatches(&receipt.source, &observation.value) {
        failures.push(CoverageFailure::for_identity(
            CoverageFailureCode::ProvenanceNotDurable,
            identity,
            format!(
                "durable provenance parity mismatch in {}",
                dimensions.join(", ")
            ),
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
    use crate::source_coverage::{
        ExpectedSourceShape, FinalOwnerContract, FixtureReference, ReaderContract,
        SourceDocumentRole, SourceJsonType, SourceLeafSelector, SourceMemberKind, SourcePin,
        SourcePrevalence, StageObservation, SurfaceContract, SurfaceDecision, SurfaceDisposition,
        TypedUnsupportedValue,
    };

    fn promoted_ledger() -> SourceLeafCoverageLedger {
        SourceLeafCoverageLedger {
            contract_version: "atlas-source-leaf-coverage/v1".to_string(),
            type_id: "actor:npc".to_string(),
            source_pin: SourcePin {
                upstream_commit: "source-commit".to_string(),
                source_signature: "source-signature".to_string(),
            },
            selector: SourceLeafSelector {
                source_contract_version: "pf2e-serialized-source/v1".to_string(),
                document_class: "Actor".to_string(),
                type_discriminator: "npc".to_string(),
                role: SourceDocumentRole::TopLevel,
                parent_context: "root".to_string(),
            },
            leaves: vec![SourceLeafContract {
                normalized_path: "$.system.values[]".to_string(),
                expected_shapes: vec![
                    ExpectedSourceShape::Missing,
                    ExpectedSourceShape::Null,
                    ExpectedSourceShape::String,
                    ExpectedSourceShape::Object,
                    ExpectedSourceShape::ArrayMember,
                ],
                source_prevalence: SourcePrevalence {
                    record_count: 1,
                    occurrence_count: 2,
                },
                map_key_policy: None,
                disposition: SourceLeafDisposition::Promoted,
                reader: ReaderContract {
                    reader_id: Some("reader::values".to_string()),
                    parity_case_ids: vec!["case-1".to_string()],
                },
                final_owners: [
                    FinalOwnerStage::SourceDto,
                    FinalOwnerStage::Canonical,
                    FinalOwnerStage::PostProjection,
                    FinalOwnerStage::ArtifactHydration,
                    FinalOwnerStage::PublicSurface,
                ]
                .into_iter()
                .map(|stage| FinalOwnerContract {
                    stage,
                    destination: format!("owner::{stage:?}"),
                })
                .collect(),
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

    fn value() -> SourceLeafValue {
        SourceLeafValue {
            json_type: SourceJsonType::String,
            value: Some(json!("alpha")),
            stable_digest: None,
            member_kind: Some(SourceMemberKind::Array),
            member_identity: Some("member-a".to_string()),
            ordinal: Some(0),
            multiplicity: 2,
            unsupported: None,
        }
    }

    fn receipt(ledger: &SourceLeafCoverageLedger) -> SourceLeafReceipt {
        let value = SourcePresence::Value(value());
        SourceLeafReceipt {
            identity: ledger.identity_for(&ledger.leaves[0]),
            fixture: FixtureReference {
                case_id: "case-1".to_string(),
                record_key: "pack:id".to_string(),
                source_path: "packs/fixture/id.json".to_string(),
                source_commit: "source-commit".to_string(),
                excerpt_digest: "sha256:fixture".to_string(),
                source_grounded: true,
            },
            source_pin: ledger.source_pin.clone(),
            inventory_observed: true,
            source: value.clone(),
            reader_id: Some("reader::values".to_string()),
            observations: ledger.leaves[0]
                .final_owners
                .iter()
                .map(|owner| StageObservation {
                    stage: owner.stage,
                    destination: owner.destination.clone(),
                    value: value.clone(),
                })
                .collect(),
            semantic_output_observed: true,
            comparator_assertion_ids: vec!["assert::values".to_string()],
        }
    }

    fn exact_codes(report: &CoverageReport) -> Vec<CoverageFailureCode> {
        report.failures.iter().map(|failure| failure.code).collect()
    }

    #[test]
    fn exact_receipt_passes_all_final_owner_boundaries() {
        let ledger = promoted_ledger();
        let report = evaluate_source_leaf_coverage(&ledger, &[receipt(&ledger)]);
        assert!(report.passed, "{:#?}", report.failures);
    }

    #[test]
    fn missing_null_and_value_states_are_distinct() {
        let ledger = promoted_ledger();
        for source in [
            SourcePresence::Missing,
            SourcePresence::Null,
            SourcePresence::Value(value()),
        ] {
            let mut receipt = receipt(&ledger);
            receipt.source = source.clone();
            for observation in &mut receipt.observations {
                observation.value = source.clone();
            }
            assert!(evaluate_source_leaf_coverage(&ledger, &[receipt]).passed);
        }

        let mut wrong = receipt(&ledger);
        wrong.source = SourcePresence::Null;
        assert_eq!(
            exact_codes(&evaluate_source_leaf_coverage(&ledger, &[wrong]))[0],
            CoverageFailureCode::DtoMismatch
        );
    }

    #[test]
    fn exact_selector_does_not_accept_parent_prefix_or_wrong_context() {
        let ledger = promoted_ledger();
        let mut wrong = receipt(&ledger);
        wrong.identity.normalized_path = "$.system.values".to_string();
        assert_eq!(
            exact_codes(&evaluate_source_leaf_coverage(&ledger, &[wrong]))[0],
            CoverageFailureCode::UndeclaredLeaf
        );

        let mut wrong = receipt(&ledger);
        wrong.identity.selector.parent_context = "embedded:action".to_string();
        assert_eq!(
            exact_codes(&evaluate_source_leaf_coverage(&ledger, &[wrong]))[0],
            CoverageFailureCode::UndeclaredLeaf
        );
    }

    #[test]
    fn missing_or_wrong_reader_has_exact_failure_code() {
        let ledger = promoted_ledger();
        let mut wrong = receipt(&ledger);
        wrong.reader_id = Some("reader::declaration_only".to_string());
        assert_eq!(
            exact_codes(&evaluate_source_leaf_coverage(&ledger, &[wrong]))[0],
            CoverageFailureCode::ReaderNotObserved
        );
    }

    #[test]
    fn every_wrong_final_owner_has_its_exact_failure_code() {
        let ledger = promoted_ledger();
        let expectations = [
            (FinalOwnerStage::SourceDto, CoverageFailureCode::DtoMismatch),
            (
                FinalOwnerStage::Canonical,
                CoverageFailureCode::CanonicalMismatch,
            ),
            (
                FinalOwnerStage::PostProjection,
                CoverageFailureCode::PostProjectionMismatch,
            ),
            (
                FinalOwnerStage::ArtifactHydration,
                CoverageFailureCode::ArtifactHydrationMismatch,
            ),
            (
                FinalOwnerStage::PublicSurface,
                CoverageFailureCode::PublicSurfaceMismatch,
            ),
        ];
        for (stage, code) in expectations {
            let mut wrong = receipt(&ledger);
            wrong
                .observations
                .iter_mut()
                .find(|observation| observation.stage == stage)
                .expect("stage")
                .destination = "wrong::transient_owner".to_string();
            assert_eq!(
                exact_codes(&evaluate_source_leaf_coverage(&ledger, &[wrong]))[0],
                code
            );
        }
    }

    #[test]
    fn value_type_identity_order_and_multiplicity_mismatches_are_rejected() {
        let ledger = promoted_ledger();
        let mutations: Vec<Box<dyn Fn(&mut SourceLeafValue)>> = vec![
            Box::new(|value| value.value = Some(json!("wrong"))),
            Box::new(|value| value.json_type = SourceJsonType::Number),
            Box::new(|value| value.member_identity = Some("wrong".to_string())),
            Box::new(|value| value.ordinal = Some(1)),
            Box::new(|value| value.multiplicity = 1),
        ];
        for mutate in mutations {
            let mut wrong = receipt(&ledger);
            let SourcePresence::Value(value) = &mut wrong.observations[0].value else {
                panic!("value observation")
            };
            mutate(value);
            assert_eq!(
                exact_codes(&evaluate_source_leaf_coverage(&ledger, &[wrong]))[0],
                CoverageFailureCode::DtoMismatch
            );
        }
    }

    #[test]
    fn source_shape_outside_the_exact_contract_is_rejected() {
        let ledger = promoted_ledger();
        let mut wrong = receipt(&ledger);
        let SourcePresence::Value(value) = &mut wrong.source else {
            panic!("source value")
        };
        value.json_type = SourceJsonType::Object;
        value.member_kind = None;
        let report = evaluate_source_leaf_coverage(&ledger, &[wrong]);
        assert!(
            report
                .failures
                .iter()
                .any(|failure| { failure.code == CoverageFailureCode::FixtureNotSourceGrounded })
        );
    }

    #[test]
    fn promoted_typed_unsupported_payload_must_reach_canonical_owner() {
        let ledger = promoted_ledger();
        let mut wrong = receipt(&ledger);
        let unsupported = TypedUnsupportedValue {
            value: json!({"unexpected": true}),
            reason: "unsupported_object".to_string(),
        };
        let SourcePresence::Value(source) = &mut wrong.source else {
            panic!("source value")
        };
        source.json_type = SourceJsonType::Object;
        source.value = Some(json!({"unexpected": true}));
        source.unsupported = Some(unsupported.clone());
        for observation in &mut wrong.observations {
            let SourcePresence::Value(value) = &mut observation.value else {
                panic!("owner value")
            };
            value.json_type = SourceJsonType::Object;
            value.value = Some(json!({"unexpected": true}));
            value.unsupported = Some(unsupported.clone());
        }
        let canonical = wrong
            .observations
            .iter_mut()
            .find(|observation| observation.stage == FinalOwnerStage::Canonical)
            .expect("canonical observation");
        let SourcePresence::Value(value) = &mut canonical.value else {
            panic!("canonical value")
        };
        value.unsupported = None;
        assert_eq!(
            exact_codes(&evaluate_source_leaf_coverage(&ledger, &[wrong]))[0],
            CoverageFailureCode::CanonicalMismatch
        );
    }

    #[test]
    fn deferred_and_unconsumed_fail_closed_with_exact_codes() {
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
            let mut ledger = promoted_ledger();
            let leaf = &mut ledger.leaves[0];
            leaf.disposition = disposition;
            leaf.reader = ReaderContract::default();
            leaf.final_owners.clear();
            leaf.surfaces = SurfaceContract::default();
            leaf.rationale = "not accepted".to_string();
            if disposition == SourceLeafDisposition::Deferred {
                leaf.future_owner = Some("H9".to_string());
                leaf.future_task = Some("actor-npc".to_string());
                leaf.prerequisite = Some("A2".to_string());
            }
            assert_eq!(
                exact_codes(&evaluate_source_leaf_coverage(&ledger, &[]))[0],
                code
            );
        }
    }

    #[test]
    fn stale_and_unbound_fixtures_have_exact_codes() {
        let ledger = promoted_ledger();
        let mut stale = receipt(&ledger);
        stale.inventory_observed = false;
        assert_eq!(
            exact_codes(&evaluate_source_leaf_coverage(&ledger, &[stale]))[0],
            CoverageFailureCode::StaleDeclaration
        );

        let mut unbound = receipt(&ledger);
        unbound.fixture.source_grounded = false;
        assert_eq!(
            exact_codes(&evaluate_source_leaf_coverage(&ledger, &[unbound]))[0],
            CoverageFailureCode::FixtureNotSourceGrounded
        );
    }

    #[test]
    fn provenance_and_typed_unsupported_require_durable_exact_parity() {
        let mut ledger = promoted_ledger();
        let leaf = &mut ledger.leaves[0];
        leaf.disposition = SourceLeafDisposition::ProvenanceOnly;
        leaf.reader.reader_id = None;
        leaf.final_owners = vec![FinalOwnerContract {
            stage: FinalOwnerStage::DurableProvenance,
            destination: "AtlasRecord.provenance.raw_json".to_string(),
        }];
        leaf.surfaces = SurfaceContract::default();
        leaf.rationale = "durable raw provenance only".to_string();

        let mut receipt = receipt(&ledger);
        let unsupported = TypedUnsupportedValue {
            value: json!({"unexpected": true}),
            reason: "unsupported_object".to_string(),
        };
        let mut source_value = value();
        source_value.json_type = SourceJsonType::Object;
        source_value.value = Some(json!({"unexpected": true}));
        source_value.unsupported = Some(unsupported.clone());
        receipt.source = SourcePresence::Value(source_value.clone());
        receipt.observations = vec![StageObservation {
            stage: FinalOwnerStage::DurableProvenance,
            destination: "AtlasRecord.provenance.raw_json".to_string(),
            value: SourcePresence::Value(source_value),
        }];
        assert!(evaluate_source_leaf_coverage(&ledger, &[receipt.clone()]).passed);

        let SourcePresence::Value(observed) = &mut receipt.observations[0].value else {
            panic!("unsupported observation")
        };
        observed.unsupported = None;
        assert_eq!(
            exact_codes(&evaluate_source_leaf_coverage(&ledger, &[receipt]))[0],
            CoverageFailureCode::ProvenanceNotDurable
        );
    }

    #[test]
    fn ignored_leaf_cannot_populate_output() {
        let mut ledger = promoted_ledger();
        let leaf = &mut ledger.leaves[0];
        leaf.disposition = SourceLeafDisposition::Ignored;
        leaf.reader.reader_id = None;
        leaf.final_owners.clear();
        leaf.surfaces = SurfaceContract::default();
        leaf.rationale = "non-authored cache".to_string();
        let mut receipt = receipt(&ledger);
        receipt.observations.clear();
        receipt.semantic_output_observed = true;
        assert_eq!(
            exact_codes(&evaluate_source_leaf_coverage(&ledger, &[receipt]))[0],
            CoverageFailureCode::IgnoredPromoted
        );
    }
}
