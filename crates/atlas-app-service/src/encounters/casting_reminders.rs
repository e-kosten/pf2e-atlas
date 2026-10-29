//! Condition reminders for the selected spell. Costs and rolls belong to their existing workflows.
use super::rules::action_context::{
    ActionContext, ActionFact, ActionKind, KnownActionFact, evaluate_action,
};
use crate::{AppServiceError, AppServiceResult, AtlasAppService};
use atlas_app_model::*;
use atlas_record::source_record::{ItemSourceView, SourceNodeView, SourceQueryView};
impl AtlasAppService {
    pub fn encounter_casting_reminders(
        &self,
        encounter_ref: &str,
        request: EncounterCastingRemindersRequest,
    ) -> AppServiceResult<EncounterCastingRemindersView> {
        let encounter_ref = encounter_ref.to_owned();
        self.with_workspace(
            &crate::ReadExecutionContext::uncancellable(),
            move |workspace| reminders(workspace, &encounter_ref, &request),
        )
    }
}
fn reminders(
    workspace: &mut atlas_runtime::WorkspaceRead,
    encounter_ref: &str,
    request: &EncounterCastingRemindersRequest,
) -> AppServiceResult<EncounterCastingRemindersView> {
    let snapshot = workspace
        .local_state()
        .encounters()
        .get_with_participants(encounter_ref)?
        .ok_or_else(|| super::projection::encounter_not_found(encounter_ref))?;
    let participant = snapshot
        .participants
        .iter()
        .find(|participant| participant.participant_key == request.participant_key)
        .ok_or_else(|| {
            AppServiceError::invalid_request("Participant is no longer in this encounter.")
        })?;
    let selection = &request.selection;
    if participant.record_key.as_deref() != Some(selection.record_key.as_str())
        || selection.owners.len() != 1
        || selection.field.is_some()
        || selection.passage.is_some()
    {
        return Err(AppServiceError::invalid_request(
            "Select this participant's spell.",
        ));
    }
    let source =
        super::hydration::resolve_record_ref(workspace.retrieval_mut(), &selection.record_key)?;
    let fingerprint = workspace.retrieval_mut().source_fingerprint().to_owned();
    if selection.source_fingerprint.as_deref() != Some(fingerprint.as_str()) {
        return Err(AppServiceError::invalid_request(
            "Spell source changed; reopen the spell.",
        ));
    }
    let Some(SourceNodeView::Item(item @ ItemSourceView::SpellSource(_))) =
        source.source.node_at(&selection.owners)
    else {
        return Err(AppServiceError::invalid_request("Select a spell."));
    };
    let traits = SourceQueryView {
        source: SourceNodeView::Item(item),
        pack_id: "",
        pack_label: "",
    }
    .traits()
    .value();
    let mut facts = vec![KnownActionFact {
        fact: ActionFact::VoluntaryIngestion,
        value: false,
    }];
    if let Some(traits) = traits {
        for (fact, trait_key) in [
            (ActionFact::Manipulate, "manipulate"),
            (ActionFact::Auditory, "auditory"),
            (ActionFact::Concentrate, "concentrate"),
            (ActionFact::Move, "move"),
            (ActionFact::Attack, "attack"),
        ] {
            facts.push(KnownActionFact {
                fact,
                value: traits.iter().any(|value| value == trait_key),
            });
        }
    }
    let reaction = item
        .spell_casting_form()
        .value()
        .and_then(|form| match form {
            "reaction" => Some(true),
            "free" | "one_action" | "two_actions" | "three_actions" | "one_or_two"
            | "one_to_three" | "two_or_three" | "two_rounds" => Some(false),
            _ => None,
        });

    let sources = super::hydration::source_conditions(&source.source, workspace.retrieval_mut())?;
    let evaluated = evaluate_action(
        participant,
        &sources,
        &ActionContext {
            kind: ActionKind::Cast,
            reaction,
            other_participant: None,
            facts: &facts,
        },
    );
    let issues = evaluated
        .issues
        .into_iter()
        .map(|issue| super::action_projection::issue_view(issue, participant, &sources))
        .collect::<Vec<_>>();
    let check = evaluated.disruption.map(|check| {
        super::action_projection::check_view(
            check.dc,
            check.source_ids,
            std::slice::from_ref(participant),
        )
    });
    Ok(EncounterCastingRemindersView {
        advisories: casting_advisories(
            participant,
            &snapshot.participants,
            &sources,
            &issues,
            check.as_ref(),
        ),
    })
}
fn casting_advisories(
    participant: &atlas_local_state::EncounterParticipant,
    roster: &[atlas_local_state::EncounterParticipant],
    sources: &super::rules::source_conditions::SourceConditions,
    issues: &[EncounterActionIssueView],
    check: Option<&EncounterActionCheckView>,
) -> Vec<RuntimeEffectNoteView> {
    use super::conditions::ConditionRule as R;
    let mut notes = vec![];
    let authored_source = |issue: &EncounterActionIssueView| {
        sources.entries.iter().find(|source| {
            super::rules::causes::ConditionSource::Authored(
                source,
                sources.source_fingerprint.as_deref(),
                &participant.participant_key,
            )
            .key()
            .key()
                == issue.source_key
        })
    };
    let rule_for_issue = |issue: &EncounterActionIssueView| {
        participant
            .conditions
            .iter()
            .find(|condition| Some(condition.condition_id) == issue.source_condition_id)
            .and_then(|condition| {
                super::conditions::condition_rule_for_key(condition.condition_key.as_deref())
            })
            .or_else(|| authored_source(issue).map(|source| source.rule))
    };
    let unknown_stupefied = issues.iter().any(|issue| {
        matches!(
            issue.kind,
            EncounterActionIssueKindView::NeedsValue | EncounterActionIssueKindView::NeedsContext
        ) && rule_for_issue(issue) == Some(R::Stupefied)
    });
    if let Some(check) = check
        && !unknown_stupefied
    {
        let names = check
            .sources
            .iter()
            .map(|source| source.name.as_str())
            .collect::<std::collections::BTreeSet<_>>()
            .into_iter()
            .collect::<Vec<_>>()
            .join(", ");
        notes.push(RuntimeEffectNoteView {
            source: "casting-flat-check".into(),
            label: names,
            reason: format!("DC {} flat check or the spell is disrupted", check.dc),
        });
    }
    for issue in issues {
        let rule = rule_for_issue(issue);
        let activation_unknown = authored_source(issue)
            .is_some_and(|source| sources.unresolved_entries(participant).contains(&source));
        let subject = issue.subject.as_ref().map(|subject| match subject {
            ConditionSubjectView::Participant { participant_key } => roster
                .iter()
                .find(|p| p.participant_key == *participant_key)
                .map(|p| p.display_name.clone())
                .unwrap_or_else(|| "the named subject".into()),
            ConditionSubjectView::Other { description } => description.clone(),
        });
        let reason = match rule {
            Some(R::Grabbed) if check.is_none() => {
                Some("Manipulate actions require a DC 5 flat check.".into())
            }
            Some(R::Deafened) if check.is_none() => {
                Some("Auditory actions require a DC 5 flat check.".into())
            }
            Some(R::Restrained) => {
                Some("Cannot use attack or manipulate actions while Restrained.".into())
            }
            Some(R::Paralyzed) => {
                Some("Only purely mental actions are possible while Paralyzed.".into())
            }
            // The condition owns this general restriction; it is not spell-specific advice.
            Some(R::Confused) => None,
            Some(R::Controlled) => Some(
                issue
                    .instructions
                    .clone()
                    .unwrap_or_else(|| "The controller chooses your actions.".into()),
            ),
            Some(R::Fascinated) => Some(format!(
                "Concentrate actions must relate to {}.",
                subject.as_deref().unwrap_or("the subject of fascination")
            )),
            Some(R::Fleeing) => Some(format!(
                "Actions must help escape {}.",
                subject.as_deref().unwrap_or("the source of flight")
            )),
            Some(R::Stupefied) if unknown_stupefied => Some(if activation_unknown {
                "Confirm whether Stupefied applies; the casting flat-check DC is unknown.".into()
            } else {
                "The Stupefied value is missing; the casting flat-check DC is unknown.".into()
            }),
            Some(R::Unconscious | R::Petrified | R::Stunned) if activation_unknown => {
                Some("It prevents casting.".into())
            }
            _ if issue.kind == EncounterActionIssueKindView::Prohibited => {
                Some(issue.reason.clone())
            }
            _ => None,
        };
        let reason = reason.map(|reason| {
            if activation_unknown && rule != Some(R::Stupefied) {
                format!("Confirm whether {} applies. {reason}", issue.condition_name)
            } else {
                reason
            }
        });
        if let Some(reason) = reason
            && !notes.iter().any(|note| note.reason == reason)
        {
            notes.push(RuntimeEffectNoteView {
                source: issue.source_key.clone(),
                label: issue.condition_name.clone(),
                reason,
            });
        }
    }
    notes
}

#[cfg(test)]
mod tests {
    use super::super::rules::source_conditions::{SourceConditions, source_conditions};
    use super::*;
    use serde_json::json;

    fn participant() -> atlas_local_state::EncounterParticipant {
        let fixture = crate::test_support::encounter_fixture_worker();
        let encounter = fixture
            .worker
            .create_encounter(CreateEncounterRequest {
                name: "Source casting advice".into(),
                description: None,
                note: None,
            })
            .unwrap()
            .encounter
            .encounter_key;
        fixture
            .worker
            .add_encounter_manual_participant(AddEncounterManualParticipantRequest {
                encounter_ref: encounter.clone(),
                display_name: "Caster".into(),
                max_hp: Some(40),
                current_hp: Some(40),
                initiative: None,
            })
            .unwrap();
        fixture
            .worker
            .with_state(|session| {
                Ok(session
                    .local_state()
                    .encounters()
                    .get_with_participants(&encounter)?
                    .unwrap()
                    .participants
                    .remove(0))
            })
            .unwrap()
    }

    fn sources(items: serde_json::Value) -> SourceConditions {
        sources_with_rule(items, super::super::conditions::ConditionRule::Stupefied)
    }

    fn sources_with_rule(
        items: serde_json::Value,
        rule: super::super::conditions::ConditionRule,
    ) -> SourceConditions {
        let source = atlas_search::test_support::record(
            "actors",
            "Actor",
            json!({
                "_id":"sourceCaster0001", "type":"npc", "system":{"attributes":{"immunities":[]},"traits":{"value":[]}}, "items":items,
            }),
        );
        source_conditions(&source.record, |_| Ok(Some(rule))).unwrap()
    }

    fn advice(
        participant: &atlas_local_state::EncounterParticipant,
        sources: &SourceConditions,
    ) -> Vec<RuntimeEffectNoteView> {
        let evaluation = evaluate_action(
            participant,
            sources,
            &ActionContext {
                kind: ActionKind::Cast,
                reaction: Some(false),
                other_participant: None,
                facts: &[],
            },
        );
        let issues = evaluation
            .issues
            .into_iter()
            .map(|issue| super::super::action_projection::issue_view(issue, participant, sources))
            .collect::<Vec<_>>();
        let check = evaluation.disruption.map(|check| {
            super::super::action_projection::check_view(
                check.dc,
                check.source_ids,
                std::slice::from_ref(participant),
            )
        });
        casting_advisories(
            participant,
            std::slice::from_ref(participant),
            sources,
            &issues,
            check.as_ref(),
        )
    }

    #[test]
    fn source_casting_issues_keep_paralysis_and_missing_values_without_confusion_hint() {
        let participant = participant();
        let sources = sources(json!([
            {"_id":"paralyzedSource1","type":"condition","system":{"slug":"paralyzed"}},
            {"_id":"stupefiedSource1","type":"condition","system":{"slug":"stupefied","value":{"isValued":true,"value":null}}},
            {"_id":"confusedSource01","type":"condition","system":{"slug":"confused"}}
        ]));
        let notes = advice(&participant, &sources);
        assert_eq!(notes.len(), 2);
        assert!(
            notes
                .iter()
                .any(|note| note.reason
                    == "Only purely mental actions are possible while Paralyzed.")
        );
        assert!(notes.iter().any(|note| note.reason
            == "The Stupefied value is missing; the casting flat-check DC is unknown."));
        assert!(
            notes
                .iter()
                .all(|note| note.source.starts_with("source-condition:"))
        );
    }

    #[test]
    fn unknown_source_activation_warns_without_inventing_a_casting_check() {
        let participant = participant();
        let sources = sources(json!([
            {"_id":"unknownSource001","type":"action","system":{"actionType":{"value":"passive"},"rules":[
                {"key":"GrantItem","inMemoryOnly":true,"uuid":"fixture:stupefied","predicate":["unknown-context"]}
            ]}}
        ]));
        let notes = advice(&participant, &sources);
        assert_eq!(notes.len(), 1);
        assert_eq!(notes[0].label, "Stupefied");
        assert_eq!(
            notes[0].reason,
            "Confirm whether Stupefied applies; the casting flat-check DC is unknown."
        );
    }

    #[test]
    fn known_stronger_casting_check_survives_uncertain_weaker_source() {
        let participant = participant();
        let mut sources = sources(json!([
            {"_id":"knownStupefied01","type":"condition","system":{"slug":"stupefied","value":{"isValued":true,"value":3}}},
            {"_id":"unknownSource001","type":"action","system":{"actionType":{"value":"passive"},"rules":[
                {"key":"GrantItem","inMemoryOnly":true,"uuid":"fixture:stupefied","predicate":["unknown-context"]}
            ]}}
        ]));
        sources.source_fingerprint = Some("fixture".into());
        let notes = advice(&participant, &sources);
        assert_eq!(notes.len(), 1);
        assert_eq!(notes[0].reason, "DC 8 flat check or the spell is disrupted");
    }

    #[test]
    fn uncertain_action_prevention_keeps_conditional_casting_advice_without_confusion_hint() {
        use super::super::conditions::ConditionRule;
        let participant = participant();
        for rule in [
            ConditionRule::Unconscious,
            ConditionRule::Petrified,
            ConditionRule::Stunned,
            ConditionRule::Confused,
        ] {
            let sources = sources_with_rule(
                json!([
                    {"_id":"unknownSource001","type":"action","system":{"actionType":{"value":"passive"},"rules":[
                        {"key":"GrantItem","inMemoryOnly":true,"uuid":"fixture:condition","predicate":["unknown-context"]}
                    ]}}
                ]),
                rule,
            );
            let notes = advice(&participant, &sources);
            if rule == ConditionRule::Confused {
                assert!(notes.is_empty());
            } else {
                assert_eq!(notes.len(), 1);
                assert_eq!(
                    notes[0].reason,
                    format!(
                        "Confirm whether {} applies. It prevents casting.",
                        rule.definition().name
                    )
                );
            }
        }
    }
}
