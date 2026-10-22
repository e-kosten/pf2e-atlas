//! Common event guards; domain preparation stays with each workflow and storage owns atomicity.
use crate::{AppServiceResult, AtlasAppService};
use atlas_local_state::{
    ConditionEventBoundary, EncounterEventReceipt, EncounterWithParticipants, LocalStateResult,
    PreparedConditionEvent,
};

pub(super) fn recover(
    service: &AtlasAppService,
    encounter_ref: &str,
    request_id: &str,
    intent: &serde_json::Value,
) -> AppServiceResult<Option<EncounterEventReceipt>> {
    service.with_state(|session| {
        let store = session.local_state().encounters();
        let encounter = store
            .get(encounter_ref)?
            .ok_or_else(|| super::projection::encounter_not_found(encounter_ref))?;
        Ok(store.condition_event_receipt(&encounter.encounter_key, request_id, intent)?)
    })
}

pub(super) fn snapshot(
    service: &AtlasAppService,
    encounter_ref: &str,
    expected: i64,
) -> AppServiceResult<EncounterWithParticipants> {
    service.with_state(|session| {
        let snapshot = session
            .local_state()
            .encounters()
            .get_with_participants(encounter_ref)?
            .ok_or_else(|| super::projection::encounter_not_found(encounter_ref))?;
        if snapshot.encounter.revision != expected {
            return Err(atlas_local_state::LocalStateError::StaleEncounterRevision {
                encounter_key: snapshot.encounter.encounter_key,
                expected_revision: expected,
                actual_revision: snapshot.encounter.revision,
            }
            .into());
        }
        Ok(snapshot)
    })
}

pub(super) struct EncounterEvent<'a> {
    pub encounter_key: &'a str,
    pub request_id: &'a str,
    pub intent: &'a serde_json::Value,
    pub expected_revision: i64,
    pub reference_revision: Option<u64>,
    pub participant_revision: Option<(&'a str, i64)>,
}
impl EncounterEvent<'_> {
    pub fn commit(
        self,
        service: &AtlasAppService,
        changes: PreparedConditionEvent,
    ) -> AppServiceResult<EncounterEventReceipt> {
        self.evaluate(
            service,
            ConditionEventBoundary {
                claims: &[],
                outgoing_boundary_key: None,
            },
            || Ok(changes),
        )
    }

    pub fn evaluate(
        self,
        service: &AtlasAppService,
        boundary: ConditionEventBoundary<'_>,
        evaluate: impl FnOnce() -> LocalStateResult<PreparedConditionEvent>,
    ) -> AppServiceResult<EncounterEventReceipt> {
        service.with_write(|session| self.evaluate_in(session, boundary, evaluate))
    }

    /// Source-free workflows may already be preparing inside a workspace write.
    pub fn evaluate_in(
        self,
        session: &atlas_runtime::WorkspaceWrite,
        boundary: ConditionEventBoundary<'_>,
        evaluate: impl FnOnce() -> LocalStateResult<PreparedConditionEvent>,
    ) -> AppServiceResult<EncounterEventReceipt> {
        let store = session.local_state().encounters();
        // Recover before source checks, even if reference data became unavailable.
        if let Some(receipt) =
            store.condition_event_receipt(self.encounter_key, self.request_id, self.intent)?
        {
            return Ok(receipt);
        }
        if let Some((key, expected_revision)) = self.participant_revision {
            let snapshot = store
                .get_with_participants(self.encounter_key)?
                .ok_or_else(|| super::projection::encounter_not_found(self.encounter_key))?;
            let participant = snapshot
                .participants
                .iter()
                .find(|participant| participant.participant_key == key)
                .ok_or_else(|| {
                    crate::AppServiceError::invalid_request("participant no longer exists")
                })?;
            if participant.revision != expected_revision {
                return Err(
                    atlas_local_state::LocalStateError::StaleParticipantRevision {
                        participant_key: key.into(),
                        expected_revision,
                        actual_revision: participant.revision,
                    }
                    .into(),
                );
            }
        }
        if let Some(revision) = self.reference_revision {
            session.require_reference_revision(revision)?;
        }
        // Storage checks the current revision/claims before invoking a callback or RNG.
        let receipt = store.apply_condition_event_with_claims(
            self.encounter_key,
            self.request_id,
            self.intent,
            self.expected_revision,
            boundary,
            evaluate,
        )?;
        super::hp_events::retire_stale_hp_decisions(session, self.encounter_key)?;
        Ok(receipt)
    }
}

pub(super) fn mutation(receipt: EncounterEventReceipt) -> atlas_app_model::EncounterMutationView {
    atlas_app_model::EncounterMutationView {
        encounter_key: receipt.encounter_key,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use atlas_app_model::CreateEncounterRequest;
    use std::cell::Cell;

    #[test]
    fn replay_recovers_before_reference_checks_and_guards_precede_evaluation() {
        let fixture = crate::test_support::encounter_fixture_worker();
        let service = &fixture.worker;
        let key = service
            .create_encounter(CreateEncounterRequest {
                name: "Event guards".into(),
                description: None,
                note: None,
            })
            .unwrap()
            .encounter
            .encounter_key;
        let revision = service.encounter(&key).unwrap().revision;
        let intent = serde_json::json!({ "action": "guard-test" });
        let evaluations = Cell::new(0);
        let event = |id, expected_revision, reference_revision| EncounterEvent {
            encounter_key: &key,
            request_id: id,
            intent: &intent,
            expected_revision,
            reference_revision,
            participant_revision: None,
        };
        let boundary = || ConditionEventBoundary {
            claims: &[],
            outgoing_boundary_key: None,
        };
        let evaluate = || {
            evaluations.set(evaluations.get() + 1);
            Ok(PreparedConditionEvent::default())
        };
        event("once", revision, None)
            .evaluate(service, boundary(), evaluate)
            .unwrap();
        event("once", revision, Some(u64::MAX))
            .evaluate(service, boundary(), evaluate)
            .unwrap();
        assert_eq!(
            evaluations.get(),
            1,
            "replay must not evaluate again, even with unavailable reference facts"
        );
        assert!(
            event("stale", i64::MIN, None)
                .evaluate(service, boundary(), evaluate)
                .is_err()
        );
        assert!(
            event(
                "source",
                service.encounter(&key).unwrap().revision,
                Some(u64::MAX)
            )
            .evaluate(service, boundary(), evaluate)
            .is_err()
        );
        assert_eq!(
            evaluations.get(),
            1,
            "invalid state must not invoke evaluation or RNG"
        );
        let changed_intent = serde_json::json!({ "action": "different" });
        assert!(
            EncounterEvent {
                intent: &changed_intent,
                ..event("once", revision, None)
            }
            .evaluate(service, boundary(), evaluate)
            .is_err()
        );
        assert_eq!(evaluations.get(), 1);
    }
}
