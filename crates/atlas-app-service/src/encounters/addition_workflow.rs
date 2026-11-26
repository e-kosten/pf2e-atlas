//! Prepare reference-backed participants before committing the complete owned mutation.
use super::{hydration::resolve_record_ref, workflow::encounter_receipt};
use crate::{AppServiceError, AppServiceResult, AtlasAppService};
use atlas_app_model::{
    AddEncounterRecordParticipantRequest, CreateEncounterWithAdditionRequest,
    CreatedEncounterAdditionView, EncounterRecordAdditionResult,
};
use atlas_local_state::{
    AddEncounterParticipant, EncountersRead, EncountersWrite, NewEncounter, derive_slug,
};

struct PreparedAddition {
    revision: u64,
    participants: Vec<AddEncounterParticipant>,
    baseline: Option<super::stat_block::StatBlockView>,
    adjustment_applicable: bool,
}

impl PreparedAddition {
    fn insert(
        self,
        session: &atlas_runtime::WorkspaceWrite,
        store: &EncountersWrite<'_>,
        encounter_key: &str,
    ) -> AppServiceResult<Vec<String>> {
        session.require_reference_revision(self.revision)?;
        let added = store.add_participants(encounter_key, self.participants)?;
        if let Some(baseline) = &self.baseline {
            for participant in added.iter().filter(|participant| {
                participant.hp_origin == atlas_local_state::ParticipantHpOrigin::DerivedPristine
            }) {
                let level = super::rules::effective_participant_level(
                    participant,
                    baseline.level.as_ref(),
                    self.adjustment_applicable,
                )
                .as_ref()
                .and_then(serde_json::Number::as_i64);
                let capacity = super::rules::effective_hp_maximum(
                    participant,
                    &baseline.source_conditions,
                    level,
                );
                if capacity != participant.current_hp {
                    store.initialize_participant_hp(&participant.participant_key, capacity)?;
                }
            }
        }

        Ok(added
            .into_iter()
            .map(|participant| participant.participant_key)
            .collect())
    }
}

fn previous_creation(
    store: &EncountersRead<'_>,
    request_id: &str,
    digest: &str,
) -> AppServiceResult<Option<CreatedEncounterAdditionView>> {
    let Some(encounter_key) = store.creation_receipt(request_id, digest)? else {
        return Ok(None);
    };
    let available = store
        .get(&encounter_key)?
        .is_some_and(|encounter| encounter.encounter_key == encounter_key);
    Ok(Some(CreatedEncounterAdditionView {
        encounter_key,
        available,
    }))
}

impl AtlasAppService {
    pub fn create_encounter_with_addition(
        &self,
        request: CreateEncounterWithAdditionRequest,
    ) -> AppServiceResult<CreatedEncounterAdditionView> {
        let name = request.name.trim().to_owned();
        crate::saved_edit::require_name(&name, "Encounter")?;
        let digest = crate::characters::intent_digest(&(
            &name,
            &request.addition,
            &request.expected_artifact_generation,
        ))?;
        let id = request.addition.request_id.clone();
        let recover = || {
            self.with_state(|session| {
                previous_creation(&session.local_state()?.encounters(), &id, &digest)
            })
        };
        if let Some(result) = recover()? {
            return Ok(result);
        }
        let commit = || {
            let prepared = self.prepare_record_addition(
                request.addition,
                Some(request.expected_artifact_generation),
            )?;
            self.with_write(|session| {
                let store = session.local_state()?.encounters();
                if let Some(result) = previous_creation(&store, &id, &digest)? {
                    return Ok(result);
                }
                let encounter = store.create(NewEncounter {
                    slug: derive_slug(&name),
                    name,
                    description: None,
                    note: None,
                })?;
                prepared.insert(session, &store, &encounter.encounter_key)?;
                store.save_creation_receipt(&id, &digest, &encounter.encounter_key)?;
                Ok(CreatedEncounterAdditionView {
                    encounter_key: encounter.encounter_key,
                    available: true,
                })
            })
        };
        match commit() {
            Ok(result) => Ok(result),
            Err(error) => recover()?.ok_or(error),
        }
    }

    pub fn add_encounter_record_participant(
        &self,
        encounter_ref: &str,
        request: AddEncounterRecordParticipantRequest,
    ) -> AppServiceResult<EncounterRecordAdditionResult> {
        self.add_reviewed_encounter_record_participant(encounter_ref, request, None, None)
    }

    pub fn add_reviewed_encounter_record_participant(
        &self,
        encounter_ref: &str,
        request: AddEncounterRecordParticipantRequest,
        expected_revision: Option<i64>,
        expected_artifact_generation: Option<String>,
    ) -> AppServiceResult<EncounterRecordAdditionResult> {
        let mut intent = serde_json::json!({
            "record_ref": request.record_ref,
            "quantity": request.quantity,
            "initiative": request.initiative,
            "participant_variant": request.participant_variant,
            "creature_scenario": request.creature_scenario,
        });
        if let Some(revision) = expected_revision {
            intent["expected_revision"] = revision.into();
        }
        if let Some(generation) = &expected_artifact_generation {
            intent["expected_artifact_generation"] = generation.clone().into();
        }
        let (receipt, committed) = self.with_state(|session| {
            let store = session.local_state()?.encounters();
            let receipt = encounter_receipt(&store, encounter_ref)?;
            let committed = store.record_addition_committed(
                &receipt.encounter_key,
                &request.request_id,
                &intent,
            )?;
            Ok((receipt, committed))
        })?;
        let request_id = request.request_id.clone();
        let participant_keys = if committed {
            None
        } else {
            match self.commit_record_addition(
                &receipt.encounter_key,
                request,
                &intent,
                expected_revision,
                expected_artifact_generation,
            ) {
                Ok(keys) => keys,
                Err(error) => {
                    // A concurrent writer can commit the intent while this read resolves its record.
                    let committed = self.with_state(|session| {
                        Ok(session
                            .local_state()?
                            .encounters()
                            .record_addition_committed(
                                &receipt.encounter_key,
                                &request_id,
                                &intent,
                            )?)
                    })?;
                    if !committed {
                        return Err(error);
                    }
                    None
                }
            }
        };
        Ok(atlas_app_model::EncounterRecordAdditionResult {
            encounter_key: receipt.encounter_key,
            request_id,
            participant_keys,
        })
    }

    fn commit_record_addition(
        &self,
        encounter_key: &str,
        request: AddEncounterRecordParticipantRequest,
        intent: &serde_json::Value,
        expected_revision: Option<i64>,
        expected_generation: Option<String>,
    ) -> AppServiceResult<Option<Vec<String>>> {
        let id = request.request_id.clone();
        let prepared = self.prepare_record_addition(request, expected_generation)?;
        self.with_write(|session| {
            let store = session.local_state()?.encounters();
            if store.record_addition_committed(encounter_key, &id, intent)? {
                return Ok(None);
            }
            if let Some(expected) = expected_revision {
                let encounter = store
                    .get(encounter_key)?
                    .ok_or_else(|| super::projection::encounter_not_found(encounter_key))?;
                super::difficulty::guard_revision(&encounter, expected)?;
            }
            let keys = prepared.insert(session, &store, encounter_key)?;
            store.save_record_addition_receipt(encounter_key, &id, intent)?;
            Ok(Some(keys))
        })
    }

    fn prepare_record_addition(
        &self,
        request: AddEncounterRecordParticipantRequest,
        expected_generation: Option<String>,
    ) -> AppServiceResult<PreparedAddition> {
        let record_ref = request.record_ref.clone();
        let expected = request
            .creature_scenario
            .as_ref()
            .and_then(|scenario| scenario.expected.clone());
        let (record, baseline, variant, revision) = self.with_workspace(
            &crate::ReadExecutionContext::uncancellable(),
            move |session| {
                if let Some(expected) = expected_generation {
                    let actual = session.retrieval().artifact_generation()?;
                    if expected != actual {
                        return Err(AppServiceError::artifact_generation_changed(
                            &expected,
                            &actual,
                            atlas_app_model::AppRecoverableAction::ReloadRecord,
                        ));
                    }
                }
                let base = resolve_record_ref(session.retrieval_mut(), &record_ref)?;
                let replacement = super::record_addition::published_replacement(
                    session.retrieval_mut(),
                    &base,
                    request.participant_variant,
                    request.creature_scenario.as_ref(),
                )?;
                let (record, variant) = match replacement {
                    Some(record) => (record, None),
                    None => (base, request.participant_variant),
                };
                let baseline = super::hydration::source_baseline(
                    &record.source,
                    session.retrieval_mut(),
                    None,
                )?;
                Ok((record, baseline, variant, session.reference_revision()?))
            },
        )?;
        super::record_addition::validate_quantity(request.quantity)?;
        let template = super::record_addition::prepare(&record, variant)?;
        if let Some(expected) = expected
            && super::record_addition::prepared_choice(&template).as_ref() != Some(&expected)
        {
            return Err(crate::AppServiceError::invalid_request(
                "The creature version changed. Refresh the encounter preview before adding it.",
            ));
        }
        let participants = (0..request.quantity)
            .map(|index| {
                let mut participant = template.clone();
                if request.quantity > 1 {
                    participant.display_name = format!("{} {}", template.display_name, index + 1);
                }
                participant.initiative = request.initiative;
                participant
            })
            .collect();
        Ok(PreparedAddition {
            revision,
            participants,
            baseline,
            adjustment_applicable: super::hydration::adjustment_applicable(&record.source),
        })
    }
}
