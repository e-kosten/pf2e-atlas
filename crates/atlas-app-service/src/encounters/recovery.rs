//! Preview and execute one guarded reset or overnight recovery.
use super::{hydration, projection};
use crate::{AppServiceError, AppServiceResult, AtlasAppService};
use atlas_app_model::*;
use atlas_local_state::{EncounterWithParticipants, ParticipantGameplay};

struct RecoveryPlan {
    view: EncounterRecoveryPreviewView,
    replacements: Vec<(String, ParticipantGameplay)>,
    reset_encounter: bool,
    end_turn: bool,
    reference_revision: Option<u64>,
}
impl AtlasAppService {
    pub fn preview_encounter_recovery(
        &self,
        encounter_ref: &str,
        request: PreviewEncounterRecoveryRequest,
    ) -> AppServiceResult<EncounterRecoveryPreviewView> {
        Ok(self
            .prepare_encounter_recovery(encounter_ref, &request)?
            .view)
    }
    pub fn apply_encounter_recovery(
        &self,
        encounter_ref: &str,
        request: ApplyEncounterRecoveryRequest,
    ) -> AppServiceResult<EncounterMutationView> {
        let intent =
            serde_json::to_value(&request).map_err(atlas_local_state::LocalStateError::from)?;
        let key = self.with_state(|session| {
            let store = session.local_state().encounters();
            let encounter = store
                .get(encounter_ref)?
                .ok_or_else(|| projection::encounter_not_found(encounter_ref))?;
            Ok((
                encounter.encounter_key.clone(),
                store.recovery_applied(&encounter.encounter_key, &request.request_id, &intent)?,
            ))
        })?;
        if key.1 {
            return Ok(EncounterMutationView {
                encounter_key: key.0,
            });
        }
        let plan = self.prepare_encounter_recovery(
            encounter_ref,
            &PreviewEncounterRecoveryRequest {
                expected_encounter_revision: request.expected_encounter_revision,
                action: request.action,
            },
        )?;
        if !plan.view.can_apply {
            return Err(AppServiceError::invalid_request(
                "Review recovery: one or more participants need attention",
            ));
        }
        self.with_write(|session| {
            // Retry recovery precedes source/revision checks, including after another rest.
            let store = session.local_state().encounters();
            if store.recovery_applied(&key.0, &request.request_id, &intent)? {
                return Ok(EncounterMutationView {
                    encounter_key: key.0.clone(),
                });
            }
            if let Some(revision) = plan.reference_revision {
                session.require_reference_revision(revision)?;
            }
            session.local_state().encounters().apply_recovery(
                &key.0,
                &request.request_id,
                &intent,
                request.expected_encounter_revision,
                plan.reset_encounter,
                plan.end_turn,
                &plan.replacements,
            )?;
            Ok(EncounterMutationView {
                encounter_key: key.0.clone(),
            })
        })
    }
    fn prepare_encounter_recovery(
        &self,
        encounter_ref: &str,
        request: &PreviewEncounterRecoveryRequest,
    ) -> AppServiceResult<RecoveryPlan> {
        let (snapshot, baselines) = self.with_state(|session| {
            let store = session.local_state().encounters();
            let snapshot = store
                .get_with_participants(encounter_ref)?
                .ok_or_else(|| projection::encounter_not_found(encounter_ref))?;
            let baselines = snapshot
                .participants
                .iter()
                .map(|p| {
                    Ok((
                        p.participant_key.clone(),
                        store.participant_baseline(&p.participant_key)?,
                    ))
                })
                .collect::<AppServiceResult<std::collections::BTreeMap<_, _>>>()?;
            Ok((snapshot, baselines))
        })?;
        require_revision(&snapshot, request.expected_encounter_revision)?;
        let reset_encounter = matches!(request.action, EncounterRecoveryAction::ResetEncounter);
        let rest = matches!(
            request.action,
            EncounterRecoveryAction::RestOvernight { .. }
        );
        let mut keys = std::collections::BTreeSet::new();
        match &request.action {
            EncounterRecoveryAction::ResetEncounter => {
                keys.extend(
                    snapshot
                        .participants
                        .iter()
                        .map(|p| p.participant_key.as_str()),
                );
            }
            EncounterRecoveryAction::ResetParticipant { participant_key } => {
                keys.insert(participant_key.as_str());
            }
            EncounterRecoveryAction::RestOvernight { participants } => {
                if participants.is_empty() {
                    return Err(AppServiceError::invalid_request(
                        "Choose at least one participant to rest",
                    ));
                }
                for p in participants {
                    if !keys.insert(p.participant_key.as_str()) {
                        return Err(AppServiceError::invalid_request(
                            "A participant was selected more than once",
                        ));
                    }
                    if p.healing
                        .is_some_and(|h| !(0..=9_007_199_254_740_991).contains(&h))
                    {
                        return Err(AppServiceError::invalid_request(
                            "Healing must be a nonnegative safe integer",
                        ));
                    }
                }
            }
        }
        if keys.iter().any(|key| {
            !snapshot
                .participants
                .iter()
                .any(|p| &p.participant_key == key)
        }) {
            return Err(AppServiceError::invalid_request(
                "A participant is no longer in this encounter",
            ));
        }
        let selected = snapshot
            .participants
            .iter()
            .filter(|p| keys.contains(p.participant_key.as_str()))
            .cloned()
            .collect::<Vec<_>>();
        let source_participants = selected.clone();
        let (records, reference_revision) =
            if rest && selected.iter().any(|p| p.record_key.is_some()) {
                self.with_workspace(
                    &crate::ReadExecutionContext::uncancellable(),
                    move |session| {
                        let (_, records) = hydration::hydrate_participant_records(
                            session.retrieval_mut(),
                            &source_participants,
                        )?;
                        Ok((records, Some(session.reference_revision()?)))
                    },
                )?
            } else {
                (Default::default(), None)
            };
        let mut rows = vec![];
        let mut replacements = vec![];
        for p in &selected {
            let mut row = EncounterRecoveryParticipantView {
                participant_key: p.participant_key.clone(),
                display_name: p.display_name.clone(),
                current_hp: p.current_hp,
                resulting_hp: p.current_hp,
                healing: None,
                changes: vec![],
                unavailable_reason: None,
            };
            let state = if let EncounterRecoveryAction::RestOvernight { participants } =
                &request.action
            {
                let input = participants
                    .iter()
                    .find(|v| v.participant_key == p.participant_key)
                    .ok_or_else(|| AppServiceError::invalid_request("Missing rest participant"))?;
                let record = p.record_key.as_ref().and_then(|k| records.get(k));
                super::rules::overnight::prepare(p, input.healing, record, &mut row)?
            } else if let Some(Some(baseline)) = baselines.get(&p.participant_key) {
                row.resulting_hp = baseline.current_hp;
                row.changes.push(format!(
                    "Restore creation HP ({} / {}), initiative, variant, conditions and resources",
                    baseline.current_hp.map_or("—".into(), |n| n.to_string()),
                    baseline.max_hp.map_or("—".into(), |n| n.to_string())
                ));
                Some(baseline.clone())
            } else {
                row.unavailable_reason=Some("No creation baseline was saved for this participant. Add it again to capture one.".into());
                None
            };
            if let Some(state) = state {
                replacements.push((p.participant_key.clone(), state));
            }
            rows.push(row);
        }
        let end_turn = rest
            || snapshot
                .encounter
                .current_turn_participant_key
                .as_deref()
                .is_some_and(|k| keys.contains(k));
        let changes = if reset_encounter {
            vec!["Return encounter to Draft, round 1; clear the current turn and pending turn decisions.".into()]
        } else if end_turn {
            vec!["End the current turn and clear pending turn decisions.".into()]
        } else {
            vec!["Clear pending turn decisions for the reset participant.".into()]
        };
        Ok(RecoveryPlan {
            view: EncounterRecoveryPreviewView {
                encounter_key: snapshot.encounter.encounter_key,
                encounter_revision: request.expected_encounter_revision,
                can_apply: rows.iter().all(|r| r.unavailable_reason.is_none()),
                participants: rows,
                changes,
            },
            replacements,
            reset_encounter,
            end_turn,
            reference_revision,
        })
    }
}
fn require_revision(snapshot: &EncounterWithParticipants, expected: i64) -> AppServiceResult<()> {
    if snapshot.encounter.revision != expected {
        return Err(atlas_local_state::LocalStateError::StaleEncounterRevision {
            encounter_key: snapshot.encounter.encounter_key.clone(),
            expected_revision: expected,
            actual_revision: snapshot.encounter.revision,
        }
        .into());
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_support::encounter_fixture_worker;
    #[test]
    fn overnight_uses_explicit_manual_healing_caps_hp_and_does_not_repeat() {
        let f = encounter_fixture_worker();
        let w = &f.worker;
        let key = w
            .create_encounter(CreateEncounterRequest {
                name: "Recovery".into(),
                description: None,
                note: None,
            })
            .unwrap()
            .encounter
            .encounter_key;
        w.add_encounter_manual_participant(AddEncounterManualParticipantRequest {
            encounter_ref: key.clone(),
            display_name: "PC".into(),
            max_hp: Some(20),
            current_hp: Some(12),
            initiative: None,
        })
        .unwrap();
        let p = w.encounter(&key).unwrap().participants.remove(0);
        w.set_encounter_life_settings(
            &key,
            &p.participant_key,
            SetEncounterLifeSettingsRequest {
                knockout_context: None,
                mortality: None,
                request_id: "level".into(),
                expected_encounter_revision: w.encounter(&key).unwrap().revision,
                track_dying: true,
                level_override: Some(LevelOverrideInputView { value: Some(1) }),
                base_death_threshold: None,
            },
        )
        .unwrap();
        w.with_write(|s| {
            for (rule, value) in [
                (super::super::conditions::ConditionRule::Fatigued, None),
                (super::super::conditions::ConditionRule::Drained, Some(2)),
                (super::super::conditions::ConditionRule::Doomed, Some(1)),
                (super::super::conditions::ConditionRule::Wounded, Some(1)),
            ] {
                let def = super::super::conditions::modeled_condition_by_rule(rule).unwrap();
                s.local_state().encounters().add_condition(
                    atlas_local_state::AddEncounterParticipantCondition {
                        participant_key: p.participant_key.clone(),
                        condition_key: Some(def.condition_ref.into()),
                        name: def.name.into(),
                        value,
                        source_participant_key: None,
                        scope: Default::default(),
                        extra_action: Default::default(),
                        immunity_override: false,
                        parameters: Default::default(),
                        origin: None,
                        duration: None,
                        note: None,
                    },
                )?;
            }
            Ok(())
        })
        .unwrap();
        let revision = w.encounter(&key).unwrap().revision;
        let action = EncounterRecoveryAction::RestOvernight {
            participants: vec![OvernightParticipantInput {
                participant_key: p.participant_key.clone(),
                healing: None,
            }],
        };
        assert!(
            !w.preview_encounter_recovery(
                &key,
                PreviewEncounterRecoveryRequest {
                    expected_encounter_revision: revision,
                    action: action.clone()
                }
            )
            .unwrap()
            .can_apply
        );
        let action = EncounterRecoveryAction::RestOvernight {
            participants: vec![OvernightParticipantInput {
                participant_key: p.participant_key.clone(),
                healing: Some(100),
            }],
        };
        let preview = w
            .preview_encounter_recovery(
                &key,
                PreviewEncounterRecoveryRequest {
                    expected_encounter_revision: revision,
                    action: action.clone(),
                },
            )
            .unwrap();
        assert!(preview.can_apply);
        assert_eq!(preview.participants[0].resulting_hp, Some(19));
        let request = ApplyEncounterRecoveryRequest {
            request_id: "night-one".into(),
            expected_encounter_revision: revision,
            action,
        };
        w.apply_encounter_recovery(&key, request.clone()).unwrap();
        let after = w.encounter(&key).unwrap();
        assert_eq!(after.participants[0].current_hp, Some(19));
        let conditions = &after.participants[0].conditions;
        assert_eq!(conditions.len(), 1);
        assert_eq!(conditions[0].name, "Drained");
        assert_eq!(conditions[0].value, Some(1));
        w.apply_encounter_recovery(&key, request).unwrap();
        assert_eq!(w.encounter(&key).unwrap(), after);
        w.apply_encounter_recovery(
            &key,
            ApplyEncounterRecoveryRequest {
                request_id: "reset".into(),
                expected_encounter_revision: after.revision,
                action: EncounterRecoveryAction::ResetEncounter,
            },
        )
        .unwrap();
        let reset = w.encounter(&key).unwrap();
        assert_eq!(reset.participants[0].current_hp, Some(12));
        assert!(reset.participants[0].conditions.is_empty());
    }
}
