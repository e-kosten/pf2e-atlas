//! Pure action restrictions and flat-check obligations. Unknown facts stay contextual hints.
use super::super::conditions::ConditionRule;
use super::causes::{ConditionCause, ConditionSourceKey, condition_causes};
use super::source_conditions::SourceConditions;
use atlas_local_state::EncounterParticipant;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(in crate::encounters) enum ActionKind {
    Strike,
    Cast,
    Retch,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(in crate::encounters) enum ActionFact {
    Reaction,
    Manipulate,
    Auditory,
    Concentrate,
    Move,
    Attack,
    MentalOnly,
    RelatesToFascination,
    FurthersEscape,
    Cantrip,
    VoluntaryIngestion,
    FollowsControllerInstructions,
}

/// A judgment is tied to its condition source; one threat's escape is not another's.
#[derive(Debug, Clone, Copy)]
pub(in crate::encounters) struct ActionJudgment {
    pub source_id: Option<i64>,
    pub fact: ActionFact,
    pub value: bool,
}

pub(in crate::encounters) struct ActionContext<'a> {
    pub kind: ActionKind,
    pub reaction: Option<bool>,
    pub other_participant: Option<&'a str>,
    pub facts: &'a [ActionJudgment],
}

impl ActionContext<'_> {
    fn fact(&self, source_id: &ConditionSourceKey, fact: ActionFact) -> Option<bool> {
        let scoped: Vec<_> = self
            .facts
            .iter()
            .filter(|entry| {
                entry.fact == fact
                    && source_id.saved_id().is_some()
                    && entry.source_id == source_id.saved_id()
            })
            .collect();
        let values: Vec<_> = if scoped.is_empty() {
            self.facts
                .iter()
                .filter(|entry| entry.fact == fact && entry.source_id.is_none())
                .collect()
        } else {
            scoped
        };
        let first = values.first()?.value;
        values
            .iter()
            .all(|entry| entry.value == first)
            .then_some(first)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(in crate::encounters) enum ActionIssue {
    Prohibited {
        source_id: ConditionSourceKey,
        rule: ConditionRule,
    },
    NeedsFact {
        source_id: ConditionSourceKey,
        fact: ActionFact,
    },
    NeedsConfusion {
        source_id: ConditionSourceKey,
    },
    NeedsValue {
        source_id: ConditionSourceKey,
    },
    NeedsContext {
        source_id: ConditionSourceKey,
        reason: &'static str,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(in crate::encounters) struct DisruptionCheck {
    pub dc: i64,
    pub source_ids: Vec<ConditionSourceKey>,
}

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub(in crate::encounters) struct ActionEvaluation {
    pub issues: Vec<ActionIssue>,
    // These checks all prevent this action's effects after costs. Target-detection
    // checks belong to their later stage and must not be folded into this group.
    pub disruption: Option<DisruptionCheck>,
}

impl ActionEvaluation {
    fn prohibit(&mut self, source_id: &ConditionSourceKey, rule: ConditionRule) {
        let issue = ActionIssue::Prohibited {
            source_id: source_id.clone(),
            rule,
        };
        if !self.issues.contains(&issue) {
            self.issues.push(issue);
        }
    }

    fn requires(
        &mut self,
        context: &ActionContext<'_>,
        source_id: &ConditionSourceKey,
        rule: ConditionRule,
        fact: ActionFact,
        permitted: bool,
    ) {
        match context.fact(source_id, fact) {
            Some(value) if value != permitted => self.prohibit(source_id, rule),
            None => self.issues.push(ActionIssue::NeedsFact {
                source_id: source_id.clone(),
                fact,
            }),
            _ => {}
        }
    }

    fn check(&mut self, source_id: &ConditionSourceKey, dc: i64) {
        let check = self.disruption.get_or_insert_with(|| DisruptionCheck {
            dc,
            source_ids: vec![],
        });
        check.dc = check.dc.max(dc);
        if !check.source_ids.contains(source_id) {
            check.source_ids.push(source_id.clone());
        }
    }

    fn conditional_check(
        &mut self,
        context: &ActionContext<'_>,
        source_id: &ConditionSourceKey,
        fact: ActionFact,
    ) {
        match context.fact(source_id, fact) {
            Some(true) => self.check(source_id, 5),
            Some(false) => {}
            None => self.issues.push(ActionIssue::NeedsFact {
                source_id: source_id.clone(),
                fact,
            }),
        }
    }
}

pub(in crate::encounters) fn evaluate_action(
    participant: &EncounterParticipant,
    sources: &SourceConditions,
    context: &ActionContext<'_>,
) -> ActionEvaluation {
    let mut result = ActionEvaluation::default();
    let known = condition_causes(participant, context.other_participant, sources);
    for cause in &known {
        evaluate_cause(&mut result, participant, sources, context, *cause);
    }
    for cause in super::projection::uncertain_source_causes(
        participant,
        context.other_participant,
        &[],
        sources,
    ) {
        if known
            .iter()
            .any(|active| active.rule == cause.rule && active.source.key() == cause.source.key())
        {
            continue;
        }
        let mut potential = ActionEvaluation::default();
        evaluate_cause(&mut potential, participant, sources, context, cause);
        let check_uncertain = potential.disruption.as_ref().is_some_and(|check| {
            result
                .disruption
                .as_ref()
                .is_none_or(|known| check.dc > known.dc)
        });
        if !potential.issues.is_empty() || check_uncertain {
            let issue = ActionIssue::NeedsContext {
                source_id: cause.source.key(),
                reason: "Confirm whether this condition applies before resolving the action.",
            };
            if !result.issues.contains(&issue) {
                result.issues.push(issue);
            }
        }
    }
    if let Some(check) = &mut result.disruption {
        check.source_ids.sort_by_key(|source| format!("{source:?}"));
    }
    result
}

fn evaluate_cause(
    result: &mut ActionEvaluation,
    participant: &EncounterParticipant,
    sources: &SourceConditions,
    context: &ActionContext<'_>,
    cause: ConditionCause<'_>,
) {
    let id = cause.source.key();
    let rule = cause.rule;
    match rule {
        ConditionRule::Unconscious | ConditionRule::Petrified => result.prohibit(&id, rule),
        ConditionRule::Stunned if cause.value != Some(0) => result.prohibit(&id, rule),
        ConditionRule::Paralyzed => {
            result.requires(context, &id, rule, ActionFact::MentalOnly, true);
        }
        ConditionRule::Immobilized | ConditionRule::Prone => {
            result.requires(context, &id, rule, ActionFact::Move, false);
        }
        ConditionRule::Restrained => {
            result.requires(context, &id, rule, ActionFact::Attack, false);
            result.requires(context, &id, rule, ActionFact::Manipulate, false);
        }
        ConditionRule::Grabbed => result.conditional_check(context, &id, ActionFact::Manipulate),
        ConditionRule::Deafened => result.conditional_check(context, &id, ActionFact::Auditory),
        ConditionRule::Stupefied if context.kind == ActionKind::Cast => {
            match stupefied_dc(cause.value) {
                Some(dc) => result.check(&id, dc),
                _ => result.issues.push(ActionIssue::NeedsValue {
                    source_id: id.clone(),
                }),
            }
        }
        ConditionRule::Fascinated => match context.fact(&id, ActionFact::Concentrate) {
            Some(true) => {
                if matches!(
                    cause.source.saved().map(|source| &source.parameters),
                    Some(atlas_local_state::ConditionParameters::Fascinated { subject: Some(_) })
                ) {
                    result.requires(context, &id, rule, ActionFact::RelatesToFascination, true)
                } else {
                    result.issues.push(ActionIssue::NeedsContext {
                        source_id: id.clone(),
                        reason: "Choose the subject of fascination.",
                    });
                }
            }
            Some(false) => {}
            None => result.issues.push(ActionIssue::NeedsFact {
                source_id: id.clone(),
                fact: ActionFact::Concentrate,
            }),
        },
        ConditionRule::Fleeing => {
            if context.reaction.is_none() {
                result.issues.push(ActionIssue::NeedsFact {
                    source_id: id.clone(),
                    fact: ActionFact::Reaction,
                });
            }
            if context.reaction == Some(false) {
                if matches!(
                    cause.source.saved().map(|source| &source.parameters),
                    Some(atlas_local_state::ConditionParameters::Fleeing { subject: Some(_) })
                ) {
                    result.requires(context, &id, rule, ActionFact::FurthersEscape, true);
                } else {
                    result.issues.push(ActionIssue::NeedsContext {
                        source_id: id.clone(),
                        reason: "Choose the source you are fleeing from.",
                    });
                }
            }
        }
        ConditionRule::Confused => {
            if context.reaction.is_none() {
                result.issues.push(ActionIssue::NeedsFact {
                    source_id: id.clone(),
                    fact: ActionFact::Reaction,
                });
            }
            if context.reaction == Some(true) {
                result.prohibit(&id, rule);
            } else if context.reaction == Some(false) {
                result.issues.push(ActionIssue::NeedsConfusion {
                    source_id: id.clone(),
                });
            }
        }
        ConditionRule::Controlled => {
            if cause
                .source
                .saved()
                .and_then(|source| control_direction(participant, source, sources))
                .is_some()
            {
                result.requires(
                    context,
                    &id,
                    rule,
                    ActionFact::FollowsControllerInstructions,
                    true,
                );
            } else {
                result.issues.push(ActionIssue::NeedsContext {
                    source_id: id.clone(),
                    reason: "Resolve which controller directs this action.",
                });
            }
        }

        ConditionRule::Sickened => {
            result.requires(context, &id, rule, ActionFact::VoluntaryIngestion, false)
        }
        _ => {}
    }
}

pub(in crate::encounters) fn is_confused(
    participant: &EncounterParticipant,
    sources: &SourceConditions,
) -> bool {
    condition_causes(participant, None, sources)
        .iter()
        .any(|cause| cause.rule == ConditionRule::Confused)
}

/// Resolve a source's current authority without reviving a stale adjudication.
pub(in crate::encounters) fn control_direction(
    participant: &EncounterParticipant,
    source: &atlas_local_state::EncounterParticipantCondition,
    sources: &SourceConditions,
) -> Option<(String, Option<String>)> {
    let controller = source.source_participant_key.as_deref()?;
    let instructions = source.note.as_ref();
    let conflicting = condition_causes(participant, Some(controller), sources)
        .iter()
        .any(|cause| {
            cause.rule == ConditionRule::Controlled
                && (cause
                    .source
                    .saved()
                    .is_none_or(|source| source.source_participant_key.is_none())
                    || (cause
                        .source
                        .saved()
                        .and_then(|source| source.source_participant_key.as_deref())
                        == Some(controller)
                        && cause.source.saved().and_then(|source| source.note.as_ref())
                            != instructions))
        });
    (!conflicting).then(|| (controller.to_owned(), instructions.cloned()))
}

/// The same value policy feeds individual action obligations and the aggregate hint.
fn stupefied_dc(value: Option<i64>) -> Option<i64> {
    value.filter(|value| *value >= 0)?.checked_add(5)
}

pub(in crate::encounters) fn stupefied_casting_hint(
    participant: &EncounterParticipant,
    opponent: Option<&str>,
    sources: &SourceConditions,
) -> Option<Result<i64, ()>> {
    let mut checks = condition_causes(participant, opponent, sources)
        .into_iter()
        .filter(|cause| cause.rule == ConditionRule::Stupefied)
        .peekable();
    checks.peek()?;
    let mut result = ActionEvaluation::default();
    for cause in checks {
        match stupefied_dc(cause.value) {
            Some(dc) => result.check(&cause.source.key(), dc),
            None => return Some(Err(())),
        }
    }
    result.disruption.map(|check| Ok(check.dc))
}
