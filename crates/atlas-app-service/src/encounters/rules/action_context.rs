//! Pure action restrictions and flat-check obligations. Unknown facts stay questions.
use super::super::conditions::ConditionRule;
use super::causes::condition_causes;
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
    pub confusion: Option<&'a ConfusionChoice>,
    pub facts: &'a [ActionJudgment],
}

impl ActionContext<'_> {
    fn fact(&self, source_id: i64, fact: ActionFact) -> Option<bool> {
        let scoped: Vec<_> = self
            .facts
            .iter()
            .filter(|entry| entry.fact == fact && entry.source_id == Some(source_id))
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
        source_id: i64,
        rule: ConditionRule,
    },
    NeedsFact {
        source_id: i64,
        fact: ActionFact,
    },
    NeedsConfusion {
        source_id: i64,
    },
    NeedsValue {
        source_id: i64,
    },
    NeedsContext {
        source_id: i64,
        reason: &'static str,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(in crate::encounters) struct DisruptionCheck {
    pub dc: i64,
    pub source_ids: Vec<i64>,
}

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub(in crate::encounters) struct ActionEvaluation {
    pub issues: Vec<ActionIssue>,
    // These checks all prevent this action's effects after costs. Target-detection
    // checks belong to their later stage and must not be folded into this group.
    pub disruption: Option<DisruptionCheck>,
}

impl ActionEvaluation {
    fn prohibit(&mut self, source_id: i64, rule: ConditionRule) {
        let issue = ActionIssue::Prohibited { source_id, rule };
        if !self.issues.contains(&issue) {
            self.issues.push(issue);
        }
    }

    fn requires(
        &mut self,
        context: &ActionContext<'_>,
        source_id: i64,
        rule: ConditionRule,
        fact: ActionFact,
        permitted: bool,
    ) {
        match context.fact(source_id, fact) {
            Some(value) if value != permitted => self.prohibit(source_id, rule),
            None => self.issues.push(ActionIssue::NeedsFact { source_id, fact }),
            _ => {}
        }
    }

    fn check(&mut self, source_id: i64, dc: i64) {
        let check = self.disruption.get_or_insert_with(|| DisruptionCheck {
            dc,
            source_ids: vec![],
        });
        check.dc = check.dc.max(dc);
        if !check.source_ids.contains(&source_id) {
            check.source_ids.push(source_id);
        }
    }

    fn conditional_check(&mut self, context: &ActionContext<'_>, source_id: i64, fact: ActionFact) {
        match context.fact(source_id, fact) {
            Some(true) => self.check(source_id, 5),
            Some(false) => {}
            None => self.issues.push(ActionIssue::NeedsFact { source_id, fact }),
        }
    }
}

pub(in crate::encounters) fn evaluate_action(
    participant: &EncounterParticipant,
    context: &ActionContext<'_>,
) -> ActionEvaluation {
    let mut result = ActionEvaluation::default();
    for cause in condition_causes(participant, context.other_participant) {
        let id = cause.source.condition_id;
        let rule = cause.rule;
        match rule {
            ConditionRule::Unconscious | ConditionRule::Petrified => result.prohibit(id, rule),
            ConditionRule::Stunned if cause.value != Some(0) => result.prohibit(id, rule),
            ConditionRule::Paralyzed => {
                result.requires(context, id, rule, ActionFact::MentalOnly, true);
            }
            ConditionRule::Immobilized | ConditionRule::Prone => {
                result.requires(context, id, rule, ActionFact::Move, false);
            }
            ConditionRule::Restrained => {
                result.requires(context, id, rule, ActionFact::Attack, false);
                result.requires(context, id, rule, ActionFact::Manipulate, false);
            }
            ConditionRule::Grabbed => result.conditional_check(context, id, ActionFact::Manipulate),
            ConditionRule::Deafened => result.conditional_check(context, id, ActionFact::Auditory),
            ConditionRule::Stupefied if context.kind == ActionKind::Cast => {
                match stupefied_dc(cause.value) {
                    Some(dc) => result.check(id, dc),
                    _ => result
                        .issues
                        .push(ActionIssue::NeedsValue { source_id: id }),
                }
            }
            ConditionRule::Fascinated => match context.fact(id, ActionFact::Concentrate) {
                Some(true) => {
                    if matches!(
                        &cause.source.parameters,
                        atlas_local_state::ConditionParameters::Fascinated { subject: Some(_) }
                    ) {
                        result.requires(context, id, rule, ActionFact::RelatesToFascination, true)
                    } else {
                        result.issues.push(ActionIssue::NeedsContext {
                            source_id: id,
                            reason: "Choose the subject of fascination.",
                        });
                    }
                }
                Some(false) => {}
                None => result.issues.push(ActionIssue::NeedsFact {
                    source_id: id,
                    fact: ActionFact::Concentrate,
                }),
            },
            ConditionRule::Fleeing => {
                if context.reaction.is_none() {
                    result.issues.push(ActionIssue::NeedsFact {
                        source_id: id,
                        fact: ActionFact::Reaction,
                    });
                }
                if context.reaction == Some(false) {
                    if matches!(
                        &cause.source.parameters,
                        atlas_local_state::ConditionParameters::Fleeing { subject: Some(_) }
                    ) {
                        result.requires(context, id, rule, ActionFact::FurthersEscape, true);
                    } else {
                        result.issues.push(ActionIssue::NeedsContext {
                            source_id: id,
                            reason: "Choose the source you are fleeing from.",
                        });
                    }
                }
            }
            ConditionRule::Confused => {
                if context.reaction.is_none() {
                    result.issues.push(ActionIssue::NeedsFact {
                        source_id: id,
                        fact: ActionFact::Reaction,
                    });
                }
                if context.reaction == Some(true) {
                    result.prohibit(id, rule);
                } else if context.reaction == Some(false) {
                    let requirements = confusion_requirements(
                        participant,
                        context.confusion,
                        context.kind,
                        context.fact(id, ActionFact::Attack) == Some(true),
                    );
                    if requirements
                        .as_ref()
                        .is_some_and(|requirements| !requirements.required_facts.is_empty())
                    {
                        result
                            .issues
                            .push(ActionIssue::NeedsConfusion { source_id: id });
                    }
                    match context.confusion {
                        None => {}
                        Some(ConfusionChoice::UnableToAttack) => result.prohibit(id, rule),
                        Some(ConfusionChoice::Facilitate { .. })
                            if context.kind == ActionKind::Strike
                                || context.fact(id, ActionFact::Attack) == Some(true) =>
                        {
                            result.prohibit(id, rule);
                        }
                        Some(ConfusionChoice::Facilitate { .. }) => {}
                        Some(
                            choice @ (ConfusionChoice::Offensive { .. }
                            | ConfusionChoice::NoViableTargets),
                        ) => {
                            match context.kind {
                                ActionKind::Strike => {}
                                ActionKind::Cast => {
                                    result.requires(context, id, rule, ActionFact::Cantrip, true)
                                }
                                _ => result.prohibit(id, rule),
                            }
                            if let ConfusionChoice::Offensive {
                                offensive: Some(false),
                                ..
                            } = choice
                                && context.kind != ActionKind::Strike
                                && context.fact(id, ActionFact::Attack) != Some(true)
                            {
                                result.prohibit(id, rule);
                            }
                        }
                    }
                }
            }
            ConditionRule::Controlled => {
                if control_direction(participant, cause.source).is_some() {
                    result.requires(
                        context,
                        id,
                        rule,
                        ActionFact::FollowsControllerInstructions,
                        true,
                    );
                } else {
                    result.issues.push(ActionIssue::NeedsContext {
                        source_id: id,
                        reason: "Resolve which controller directs this action.",
                    });
                }
            }

            ConditionRule::Sickened => {
                result.requires(context, id, rule, ActionFact::VoluntaryIngestion, false)
            }
            _ => {}
        }
    }
    if let Some(check) = &mut result.disruption {
        check.source_ids.sort_unstable();
    }
    result
}

#[derive(Debug, Clone)]
pub(in crate::encounters) struct ConfusionRequirements {
    pub source_ids: Vec<i64>,
    pub required_facts: Vec<String>,
    pub selected_target_key: Option<String>,
    pub automatic_self_hit: bool,
    pub wastes_action: bool,
}
pub(in crate::encounters) fn confusion_requirements(
    participant: &EncounterParticipant,
    choice: Option<&ConfusionChoice>,
    kind: ActionKind,
    source_attack: bool,
) -> Option<ConfusionRequirements> {
    let source_ids = super::causes::condition_causes(participant, None)
        .into_iter()
        .filter(|cause| cause.rule == ConditionRule::Confused)
        .map(|cause| cause.source.condition_id)
        .collect::<Vec<_>>();
    if source_ids.is_empty() {
        return None;
    }
    let mut result = ConfusionRequirements {
        source_ids,
        required_facts: vec![],
        selected_target_key: None,
        automatic_self_hit: false,
        wastes_action: false,
    };
    match choice {
        None=>result.required_facts.push("Choose an offensive action, an action the GM permits to facilitate an attack, or the no-target procedure.".into()),
        Some(ConfusionChoice::Offensive {viable_target_keys,random_target_key,offensive})=>{
            if viable_target_keys.is_empty(){result.required_facts.push("Confirm viable targets, or choose no viable targets.".into());}
            if let Some(target)=random_target_key {if viable_target_keys.contains(target) {result.selected_target_key=Some(target.clone());}else{result.required_facts.push("Choose the GM’s random target from the confirmed viable targets.".into());}}
            else{result.required_facts.push("Record the GM’s random target from the confirmed viable targets.".into());}
            if kind!=ActionKind::Strike&&!source_attack&&offensive.is_none(){result.required_facts.push("Confirm this cantrip is offensive.".into());}
        }
        Some(ConfusionChoice::Facilitate {..})=>{},
        Some(ConfusionChoice::NoViableTargets)=>{result.selected_target_key=Some(participant.participant_key.clone());result.automatic_self_hit=true;},
        Some(ConfusionChoice::UnableToAttack)=>result.wastes_action=true,
    }
    Some(result)
}

/// Resolve a source's current authority without reviving a stale adjudication.
pub(in crate::encounters) fn control_direction(
    participant: &EncounterParticipant,
    source: &atlas_local_state::EncounterParticipantCondition,
) -> Option<(String, Option<String>)> {
    let controller = source.source_participant_key.as_deref()?;
    let instructions = source.note.as_ref();
    let conflicting = condition_causes(participant, Some(controller))
        .iter()
        .any(|cause| {
            cause.rule == ConditionRule::Controlled
                && (cause.source.source_participant_key.is_none()
                    || (cause.source.source_participant_key.as_deref() == Some(controller)
                        && cause.source.note.as_ref() != instructions))
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
) -> Option<Result<i64, ()>> {
    let mut checks = condition_causes(participant, opponent)
        .into_iter()
        .filter(|cause| cause.rule == ConditionRule::Stupefied)
        .peekable();
    checks.peek()?;
    let mut result = ActionEvaluation::default();
    for cause in checks {
        match stupefied_dc(cause.value) {
            Some(dc) => result.check(cause.source.condition_id, dc),
            None => return Some(Err(())),
        }
    }
    result.disruption.map(|check| Ok(check.dc))
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(in crate::encounters) enum ConfusionChoice {
    Offensive {
        viable_target_keys: Vec<String>,
        random_target_key: Option<String>,
        offensive: Option<bool>,
    },
    Facilitate {
        note: Option<String>,
    },
    NoViableTargets,
    UnableToAttack,
}
