//! Read-only condition and targeting context alongside a transient strike result.
use super::rules::action_context::ActionKind;
use crate::{AppServiceError, AppServiceResult, AtlasAppService};
use atlas_app_model::*;
impl AtlasAppService {
    pub fn preview_encounter_strike_context(
        &self,
        encounter_ref: &str,
        request: EncounterStrikeContextRequest,
    ) -> AppServiceResult<EncounterStrikeContextView> {
        let encounter_ref = encounter_ref.to_owned();
        self.with_workspace(
            &crate::ReadExecutionContext::uncancellable(),
            move |workspace| prepare(workspace, &encounter_ref, request),
        )
    }
}
fn prepare(
    workspace: &mut atlas_runtime::WorkspaceRead,
    encounter_ref: &str,
    request: EncounterStrikeContextRequest,
) -> AppServiceResult<EncounterStrikeContextView> {
    let super::strike_roll::ResolvedStrike {
        options,
        activity,
        snapshot,
        baseline,
    } = super::strike_roll::resolve_strike(workspace, encounter_ref, request.strike.clone())?;
    let participant = snapshot
        .participants
        .iter()
        .find(|participant| participant.participant_key == request.strike.participant_key)
        .ok_or_else(|| {
            AppServiceError::invalid_request("Participant is no longer in this encounter.")
        })?;
    let sources = &baseline.source_conditions;
    let facts = super::action_projection::strike_facts(&activity);
    let evaluated = super::rules::action_context::evaluate_action(
        participant,
        sources,
        &super::rules::action_context::ActionContext {
            kind: ActionKind::Strike,
            reaction: Some(false),
            other_participant: request.target_key.as_deref(),
            facts: &facts,
        },
    );
    let mut issues = evaluated
        .issues
        .into_iter()
        .map(|issue| super::action_projection::issue_view(issue, participant, sources))
        .collect::<Vec<_>>();
    issues.extend(super::action_projection::inactive_issue(participant));
    let disruption = evaluated.disruption.map(|check| {
        super::action_projection::check_view(
            check.dc,
            check.source_ids,
            std::slice::from_ref(participant),
        )
    });
    let (targeting, targeting_check) = match request.target_key.as_deref() {
        Some(key) => {
            let subject = snapshot
                .participants
                .iter()
                .find(|participant| participant.participant_key == key)
                .filter(|subject| subject.participant_key != participant.participant_key)
                .ok_or_else(|| {
                    AppServiceError::invalid_request(
                        "Choose another participant as the strike target.",
                    )
                })?;
            let target_sources = if subject.record_key == participant.record_key {
                sources.clone()
            } else {
                let mut records = std::collections::BTreeMap::new();
                super::hydration::load_participant_sources(
                    workspace.retrieval_mut(),
                    std::slice::from_ref(subject),
                    &mut records,
                )?;
                super::hydration::roster_source_conditions_from_sources(
                    workspace.retrieval_mut(),
                    std::slice::from_ref(subject),
                    &records,
                    Default::default(),
                )?
                .remove(key)
                .unwrap_or_else(super::rules::source_conditions::SourceConditions::unavailable)
            };
            let requirements = super::rules::relationships::targeting_requirements(
                subject,
                &participant.participant_key,
                &target_sources,
            );
            let dazzled = super::rules::sensory_context::dazzled_concealment(
                participant,
                key,
                Some(&baseline),
                None,
                None,
            );
            let mut dc = requirements.flat_check_dc;
            let mut ids = requirements.source_ids;
            if dazzled.applies == Some(true) {
                dc = Some(dc.unwrap_or(0).max(5));
                ids.extend(dazzled.source_ids);
            }
            let causes = super::rules::causes::condition_causes(
                subject,
                Some(&participant.participant_key),
                &target_sources,
            )
            .active;
            let prevents_effect = activity.traits_known
                && causes.iter().any(|cause| {
                    (cause.rule == super::conditions::ConditionRule::Blinded
                        && activity.traits.iter().any(|name| name == "visual"))
                        || (cause.rule == super::conditions::ConditionRule::Deafened
                            && activity.traits.iter().any(|name| name == "auditory"))
                });
            let check =
                dc.map(|dc| super::action_projection::check_view(dc, ids, &snapshot.participants));
            (
                Some(EncounterTargetingRequirementsView {
                    prevents_effect,
                    requires_guessed_location: requirements.requires_guessed_location,
                    check_senses: dazzled.applies.is_none() || requirements.needs_context,
                }),
                check,
            )
        }
        None => (None, None),
    };
    let action = EncounterStrikeActionContextView {
        issues,
        confused: super::rules::action_context::is_confused(participant, sources),
        disruption,
        targeting,
    };
    Ok(EncounterStrikeContextView {
        options,
        action,
        targeting_check,
    })
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn direct_attack_uses_condition_adjustments_and_optional_context_is_read_only() {
        let fixture = crate::test_support::encounter_fixture_worker();
        let encounter = fixture
            .worker
            .create_encounter(CreateEncounterRequest {
                name: "Strike checks".into(),
                description: None,
                note: None,
            })
            .unwrap()
            .encounter;
        fixture
            .worker
            .add_encounter_record_participant(AddEncounterRecordParticipantRequest {
                participant_variant: None,
                encounter_ref: encounter.encounter_key.clone(),
                record_ref: "Test Creature 1".into(),
                quantity: 2,
                initiative: None,
            })
            .unwrap();
        let current = fixture.worker.encounter(&encounter.encounter_key).unwrap();
        let subject = &current.participants[1];
        let definition = super::super::conditions::modeled_condition_by_rule(
            super::super::conditions::ConditionRule::Concealed,
        )
        .unwrap();
        fixture
            .worker
            .add_encounter_participant_condition(
                &encounter.encounter_key,
                AddEncounterParticipantConditionRequest {
                    request_id: "concealed".into(),
                    expected_encounter_revision: current.revision,
                    event_intent: Default::default(),
                    decisions: Default::default(),
                    participant_key: subject.participant_key.clone(),
                    condition_ref: Some(definition.condition_ref.into()),
                    name: None,
                    value: None,
                    source_participant_key: None,
                    scope: Default::default(),
                    extra_action: Default::default(),
                    immunity_override: false,
                    parameters: Default::default(),
                    origin: None,
                    duration: None,
                    note: None,
                },
            )
            .unwrap();
        let current = fixture.worker.encounter(&encounter.encounter_key).unwrap();
        let actor = &current.participants[0];
        let RecordBodyView::Creature(body) = &actor.presentation.as_ref().unwrap().body else {
            panic!("actor");
        };
        let strike = body
            .activities
            .value
            .as_ref()
            .unwrap()
            .iter()
            .find(|activity| activity.title == "Claw")
            .unwrap();
        let roll = RollEncounterStrikeRequest {
            request_id: "strike-begin".into(),
            expected_revision: current.revision,
            expected_participant_revision: actor.revision,
            participant_key: actor.participant_key.clone(),
            reference: EncounterReferenceRequest {
                condition_id: None,
                artifact_generation: current.artifact_generation.clone().unwrap(),
                selection: RecordDetailRequest {
                    record_key: strike.navigation.record_key.clone(),
                    owners: strike.navigation.owners.clone(),
                    source_fingerprint: strike.navigation.source_fingerprint.clone(),
                    spell_rank: None,
                    fields: vec![],
                    passage: None,
                },
            },
            attack_number: 1,
            mode: RollMode::Normal,
            modifier_override: None,
            roll_kind: StrikeRollKind::Attack,
        };
        let baseline = fixture
            .worker
            .encounter_strike_options(
                &encounter.encounter_key,
                EncounterStrikeOptionsRequest {
                    participant_key: roll.participant_key.clone(),
                    reference: roll.reference.clone(),
                },
            )
            .unwrap();
        for rule in [
            super::super::conditions::ConditionRule::Frightened,
            super::super::conditions::ConditionRule::Blinded,
        ] {
            let current = fixture.worker.encounter(&encounter.encounter_key).unwrap();
            let definition = super::super::conditions::modeled_condition_by_rule(rule).unwrap();
            fixture
                .worker
                .add_encounter_participant_condition(
                    &encounter.encounter_key,
                    AddEncounterParticipantConditionRequest {
                        request_id: format!("attack-hint-{rule:?}"),
                        expected_encounter_revision: current.revision,
                        participant_key: roll.participant_key.clone(),
                        condition_ref: Some(definition.condition_ref.into()),
                        value: (rule == super::super::conditions::ConditionRule::Frightened)
                            .then_some(2),
                        name: None,
                        event_intent: Default::default(),
                        decisions: Default::default(),
                        source_participant_key: None,
                        scope: Default::default(),
                        extra_action: Default::default(),
                        immunity_override: false,
                        parameters: Default::default(),
                        origin: None,
                        duration: None,
                        note: None,
                    },
                )
                .unwrap();
        }
        let mut roll = roll;
        let options = fixture
            .worker
            .encounter_strike_options(
                &encounter.encounter_key,
                EncounterStrikeOptionsRequest {
                    participant_key: roll.participant_key.clone(),
                    reference: roll.reference.clone(),
                },
            )
            .unwrap();
        roll.expected_revision = options.expected_revision;
        roll.expected_participant_revision = options.expected_participant_revision;
        assert_eq!(
            options.attacks[0].modifier,
            baseline.attacks[0].modifier - 2
        );
        assert!(
            options
                .applied
                .iter()
                .any(|hint| hint.label.contains("Frightened") && hint.reason == "-2 attack")
        );
        assert!(
            options
                .conditional
                .iter()
                .any(|hint| hint.label.contains("Blinded") && hint.reason == "Check detection")
        );
        assert!(
            !options
                .conditional
                .iter()
                .any(|hint| hint.reason.contains("Concealed"))
        );
        let mut invalid_source = roll.clone();
        invalid_source.request_id = "invalid-source".into();
        invalid_source.reference.selection.source_fingerprint = Some("wrong-source".into());
        assert!(
            fixture
                .worker
                .roll_encounter_strike(&encounter.encounter_key, invalid_source)
                .is_err()
        );
        let original = fixture
            .worker
            .roll_encounter_strike(&encounter.encounter_key, roll.clone())
            .unwrap();
        assert!(original.attack.is_some());
        assert_eq!(original.attack_modifier, Some(options.attacks[0].modifier));

        let context_request = EncounterStrikeContextRequest {
            strike: EncounterStrikeOptionsRequest {
                participant_key: roll.participant_key.clone(),
                reference: roll.reference.clone(),
            },
            target_key: Some(subject.participant_key.clone()),
        };
        let before = fixture.worker.encounter(&encounter.encounter_key).unwrap();
        for _ in 0..2 {
            let key = encounter.encounter_key.clone();
            let request = context_request.clone();
            let (context, metrics, authored, baselines) = fixture
                .worker
                .with_workspace(
                    &crate::ReadExecutionContext::uncancellable(),
                    move |workspace| {
                        workspace.retrieval().reset_read_metrics();
                        crate::presentation::actor::authored_projection_count(true);
                        super::super::mechanics::baseline_count(true);
                        let context = prepare(workspace, &key, request)?;
                        Ok((
                            context,
                            workspace.retrieval().read_metrics(),
                            crate::presentation::actor::authored_projection_count(false),
                            super::super::mechanics::baseline_count(false),
                        ))
                    },
                )
                .unwrap();
            assert_eq!(metrics.source_body_decodes, 1);
            assert_eq!(baselines, 1);
            // Only the selected strike is presented; neither participant's root
            // presentation is hydrated for optional targeting context.
            assert_eq!(authored, 0);
            assert_eq!(context.targeting_check.unwrap().dc, 5);
            assert_eq!(
                context.options.attacks[0].modifier,
                options.attacks[0].modifier
            );
        }
        assert_eq!(
            fixture.worker.encounter(&encounter.encounter_key).unwrap(),
            before
        );
    }
}
