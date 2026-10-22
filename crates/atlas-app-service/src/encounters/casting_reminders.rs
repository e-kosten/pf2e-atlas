//! Condition reminders for the selected spell. Costs and rolls belong to their existing workflows.
use super::rules::action_context::{
    ActionContext, ActionFact, ActionJudgment, ActionKind, evaluate_action,
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
    let mut facts = vec![ActionJudgment {
        source_id: None,
        fact: ActionFact::VoluntaryIngestion,
        value: false,
    }];
    if let Some(traits) = traits {
        facts.push(ActionJudgment {
            source_id: None,
            fact: ActionFact::Cantrip,
            value: traits.iter().any(|value| value == "cantrip"),
        });
        for (fact, trait_key) in [
            (ActionFact::Manipulate, "manipulate"),
            (ActionFact::Auditory, "auditory"),
            (ActionFact::Concentrate, "concentrate"),
            (ActionFact::Move, "move"),
            (ActionFact::Attack, "attack"),
        ] {
            facts.push(ActionJudgment {
                source_id: None,
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

    let evaluated = evaluate_action(
        participant,
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
        .map(|issue| super::action_projection::issue_view(issue, participant))
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
            &issues,
            check.as_ref(),
        ),
    })
}
fn casting_advisories(
    participant: &atlas_local_state::EncounterParticipant,
    roster: &[atlas_local_state::EncounterParticipant],
    issues: &[EncounterActionIssueView],
    check: Option<&EncounterActionCheckView>,
) -> Vec<RuntimeEffectNoteView> {
    use super::conditions::ConditionRule as R;
    let mut notes = vec![];
    let unknown_stupefied = issues.iter().any(|issue| {
        issue.kind == EncounterActionIssueKindView::NeedsValue
            && participant
                .conditions
                .iter()
                .find(|condition| condition.condition_id == issue.source_condition_id)
                .is_some_and(|condition| {
                    super::conditions::condition_rule_for_key(condition.condition_key.as_deref())
                        == Some(R::Stupefied)
                })
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
        let Some(condition) = participant
            .conditions
            .iter()
            .find(|condition| condition.condition_id == issue.source_condition_id)
        else {
            continue;
        };
        let rule = super::conditions::condition_rule_for_key(condition.condition_key.as_deref());
        let subject = issue.subject.as_ref().map(|subject| match subject {
            ConditionSubjectView::Participant { participant_key } => roster
                .iter()
                .find(|p| p.participant_key == *participant_key)
                .map(|p| p.display_name.clone())
                .unwrap_or_else(|| "the named subject".into()),
            ConditionSubjectView::Other { description } => description.clone(),
        });
        let reason=match rule {
            Some(R::Grabbed) if check.is_none()=>Some("Manipulate actions require a DC 5 flat check.".into()),
            Some(R::Deafened) if check.is_none()=>Some("Auditory actions require a DC 5 flat check.".into()),
            Some(R::Restrained)=>Some("Cannot use attack or manipulate actions while Restrained.".into()),
            Some(R::Paralyzed)=>Some("Only purely mental actions are possible while Paralyzed.".into()),
            Some(R::Confused)=>Some("Use an offensive cantrip or an action the GM permits to facilitate an attack. The GM selects the target.".into()),
            Some(R::Controlled)=>Some(issue.instructions.clone().unwrap_or_else(||"The controller chooses your actions.".into())),
            Some(R::Fascinated)=>Some(format!("Concentrate actions must relate to {}.",subject.as_deref().unwrap_or("the subject of fascination"))),
            Some(R::Fleeing)=>Some(format!("Actions must help escape {}.",subject.as_deref().unwrap_or("the source of flight"))),
            Some(R::Stupefied) if unknown_stupefied=>Some("The Stupefied value is missing; the casting flat-check DC is unknown.".into()),
            _ if issue.kind==EncounterActionIssueKindView::Prohibited=>Some(issue.reason.clone()),
            _=>None,
        };
        if let Some(reason) = reason
            && !notes.iter().any(|note| note.reason == reason)
        {
            notes.push(RuntimeEffectNoteView {
                source: format!("condition:{}", condition.condition_id),
                label: condition.name.clone(),
                reason,
            });
        }
    }
    notes
}
