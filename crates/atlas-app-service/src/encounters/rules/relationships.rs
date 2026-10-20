//! Directed condition relationships; roster visibility is deliberately not an input.
use super::super::conditions::{ConditionRule, condition_rule_for_key};
use super::causes::ScopedConditionCause;
use super::projection::scope_applies;
use atlas_local_state::EncounterParticipant;

/// Hidden subjects make their observer off-guard against that subject, not vice versa.
pub(in crate::encounters) fn observer_defense_causes<'a>(
    observer_key: &str,
    participants: &'a [EncounterParticipant],
    rows: &[atlas_local_state::DirectedRelationship],
) -> Vec<ScopedConditionCause<'a>> {
    participants
        .iter()
        .filter(|subject| subject.participant_key != observer_key)
        .filter(|subject| {
            matches!(
                relationship_knowledge(
                    subject,
                    observer_key,
                    atlas_local_state::RelationshipProperty::Detection,
                    rows
                ),
                atlas_local_state::RelationshipKnowledge::Known {
                    value: atlas_local_state::RelationshipValue::Detection(
                        atlas_local_state::DetectionState::Hidden
                            | atlas_local_state::DetectionState::Undetected
                    ),
                    ..
                }
            )
        })
        .flat_map(|subject| {
            subject.conditions.iter().filter_map(move |source| {
                if !scope_applies(&source.scope, Some(observer_key)) {
                    return None;
                }
                matches!(
                    condition_rule_for_key(source.condition_key.as_deref()),
                    Some(
                        ConditionRule::Hidden
                            | ConditionRule::Undetected
                            | ConditionRule::Unnoticed
                    )
                )
                .then_some(ScopedConditionCause {
                    source,
                    rule: ConditionRule::OffGuard,
                    value: None,
                    against: Some(subject.participant_key.as_str()),
                })
            })
        })
        .collect()
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(in crate::encounters) enum TargetMode {
    Direct,
    Area,
    GuessedLocation,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(in crate::encounters) struct TargetingRequirements {
    pub flat_check_dc: Option<i64>,
    pub source_ids: Vec<i64>,
    pub requires_guessed_location: bool,
    pub secret: bool,
}

pub(in crate::encounters) fn targeting_requirements(
    subject: &EncounterParticipant,
    observer_key: &str,
    mode: TargetMode,
) -> TargetingRequirements {
    let mut result = TargetingRequirements {
        flat_check_dc: None,
        source_ids: vec![],
        requires_guessed_location: false,
        secret: false,
    };
    if mode == TargetMode::Area {
        return result;
    }
    for source in &subject.conditions {
        if !scope_applies(&source.scope, Some(observer_key)) {
            continue;
        }
        let rule = condition_rule_for_key(source.condition_key.as_deref());
        let dc = match rule {
            Some(ConditionRule::Concealed) => 5,
            Some(ConditionRule::Hidden) => 11,
            Some(ConditionRule::Undetected | ConditionRule::Unnoticed) => {
                result.requires_guessed_location = mode != TargetMode::GuessedLocation;
                result.secret = true;
                11
            }
            _ => continue,
        };
        result.flat_check_dc = Some(result.flat_check_dc.unwrap_or(0).max(dc));
        result.source_ids.push(source.condition_id);
    }
    result.source_ids.sort_unstable();
    result
}

/// Resolve a directed property's saved baseline and active temporary causes.
/// An absent detection row is never evidence that a subject is observed.
pub(in crate::encounters) fn resolve_relationship(
    subject: &EncounterParticipant,
    other_key: &str,
    property: atlas_local_state::RelationshipProperty,
    baseline: Option<&atlas_local_state::RelationshipKnowledge>,
    adjudication: Option<&atlas_local_state::RelationshipKnowledge>,
) -> atlas_local_state::RelationshipKnowledge {
    use atlas_local_state::{
        AttitudeState as A, DetectionState as D, RelationshipCandidate as C,
        RelationshipKnowledge as K, RelationshipProperty as P, RelationshipProvenance as S,
        RelationshipValue as V,
    };
    let sources = relationship_source_snapshot(subject, other_key, property);
    if let Some(
        knowledge @ K::Known {
            provenance: S::Adjudication { sources: saved },
            ..
        },
    ) = adjudication
        && !sources.is_empty()
    {
        if *saved == sources {
            return knowledge.clone();
        }
        return K::Invalidated {
            prior_source: S::Adjudication {
                sources: saved.clone(),
            },
        };
    }
    let causes = super::causes::condition_causes(subject, Some(other_key));
    let mut candidates = vec![];
    let mut unknown = vec![];
    for cause in causes {
        if property == P::Attitude
            && subject.participant_kind == atlas_local_state::ParticipantKind::Pc
            && matches!(
                cause.rule,
                ConditionRule::Helpful
                    | ConditionRule::Friendly
                    | ConditionRule::Indifferent
                    | ConditionRule::Unfriendly
                    | ConditionRule::Hostile
            )
            && !matches!(
                cause.source.parameters,
                atlas_local_state::ConditionParameters::Attitude { supernatural: true }
            )
        {
            unknown.push("Confirm the supernatural effect imposing this attitude".into());
            continue;
        }
        let value = match (property, cause.rule) {
            (P::Detection, ConditionRule::Hidden) => Some(V::Detection(D::Hidden)),
            (P::Detection, ConditionRule::Undetected | ConditionRule::Unnoticed) => {
                Some(V::Detection(D::Undetected))
            }
            (P::Awareness, ConditionRule::Unnoticed) => Some(V::Awareness(false)),
            (P::Attitude, ConditionRule::Helpful) => Some(V::Attitude(A::Helpful)),
            (P::Attitude, ConditionRule::Friendly) => Some(V::Attitude(A::Friendly)),
            (P::Attitude, ConditionRule::Indifferent) => Some(V::Attitude(A::Indifferent)),
            (P::Attitude, ConditionRule::Unfriendly) => Some(V::Attitude(A::Unfriendly)),
            (P::Attitude, ConditionRule::Hostile) => Some(V::Attitude(A::Hostile)),
            (P::Control, ConditionRule::Controlled) => {
                match cause.source.source_participant_key.as_deref() {
                    Some(controller) if controller == other_key => Some(V::Control {
                        controller_key: controller.into(),
                        instructions: cause.source.note.clone(),
                    }),
                    None => {
                        unknown.push("Choose the controller".into());
                        None
                    }
                    Some(_) => None,
                }
            }
            (P::Detection, ConditionRule::Invisible) => {
                unknown.push("Confirm how this observer detects the invisible subject".into());
                None
            }
            _ => None,
        };
        if let Some(value) = value {
            candidates.push(C {
                value,
                provenance: S::Source {
                    condition_id: Some(cause.source.condition_id),
                    record_key: cause.source.condition_key.clone(),
                    application_id: cause
                        .source
                        .origin
                        .as_ref()
                        .map(|origin| origin.application_id.clone()),
                },
            });
        }
    }
    // Detection limitations compose by the strongest applicable restriction.
    // Awareness remains independent, so Undetected does not imply Unnoticed.
    if property == P::Detection
        && candidates
            .iter()
            .any(|candidate| candidate.value == V::Detection(D::Undetected))
    {
        candidates.retain(|candidate| candidate.value == V::Detection(D::Undetected));
    }
    if !unknown.is_empty() {
        return K::Unknown {
            required_facts: unknown,
        };
    }
    if let Some(first) = candidates.first() {
        if candidates
            .iter()
            .any(|candidate| candidate.value != first.value)
        {
            return K::Conflicting { candidates };
        }
        return K::Known {
            value: first.value.clone(),
            provenance: first.provenance.clone(),
        };
    }
    if let Some(K::Known {
        provenance: S::Source {
            condition_id: Some(id),
            ..
        },
        ..
    }) = baseline
        && !subject
            .conditions
            .iter()
            .any(|source| source.condition_id == *id)
        && let Some(K::Known { provenance, .. }) = baseline
    {
        return K::Invalidated {
            prior_source: provenance.clone(),
        };
    }
    if let Some(K::Known {
        provenance: S::Sources { condition_ids },
        ..
    }) = baseline
        && condition_ids.iter().any(|id| {
            !subject
                .conditions
                .iter()
                .any(|source| source.condition_id == *id)
        })
    {
        return K::Invalidated {
            prior_source: S::Sources {
                condition_ids: condition_ids.clone(),
            },
        };
    }
    baseline.cloned().unwrap_or_else(|| match property {
        P::Attitude if subject.participant_kind != atlas_local_state::ParticipantKind::Pc => {
            K::Known {
                value: V::Attitude(A::Indifferent),
                provenance: S::SourceDefault,
            }
        }
        _ => K::Unknown {
            required_facts: vec![
                match property {
                    P::Detection => "Choose how the observer detects the subject",
                    P::Awareness => "Confirm whether the observer knows the subject is present",
                    P::Attitude => "The player determines this character's attitude",
                    P::Control => "No controller assigned",
                }
                .into(),
            ],
        },
    })
}

pub(in crate::encounters) fn observer_defense_modifiers(
    observer_key: &str,
    participants: &[EncounterParticipant],
    rows: &[atlas_local_state::DirectedRelationship],
) -> Vec<super::super::stat_block::ContextualModifier> {
    use super::super::stat_block::{ContextualModifier, Modifier, ModifierType, StatisticTarget};
    use atlas_local_state::{
        DetectionState as D, RelationshipKnowledge as K, RelationshipProperty as P,
        RelationshipValue as V,
    };
    let mut modifiers = vec![];
    for subject in participants
        .iter()
        .filter(|p| p.participant_key != observer_key)
    {
        // Authored condition causes already contribute their own attribution.
        if subject.conditions.iter().any(|source| {
            scope_applies(&source.scope, Some(observer_key))
                && matches!(
                    condition_rule_for_key(source.condition_key.as_deref()),
                    Some(
                        ConditionRule::Hidden
                            | ConditionRule::Undetected
                            | ConditionRule::Unnoticed
                    )
                )
        }) {
            continue;
        }
        if let K::Known {
            value: V::Detection(D::Hidden | D::Undetected),
            provenance,
        } = relationship_knowledge(subject, observer_key, P::Detection, rows)
        {
            modifiers.push(ContextualModifier {
                target: StatisticTarget::ArmorClass,
                against: subject.participant_key.clone(),
                provenance,
                modifier: Modifier {
                    source: format!("detection:{}", subject.participant_key),
                    label: format!("Off-guard against {}", subject.display_name),
                    modifier_type: ModifierType::Circumstance,
                    value: -2,
                },
            });
        }
    }
    modifiers
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(in crate::encounters) enum RequestOutcome {
    CheckRequired,
    Prohibited,
    NeedsContext,
}
pub(in crate::encounters) struct RequestRequirements {
    pub outcome: RequestOutcome,
    pub required_facts: Vec<String>,
    pub reason: String,
}
pub(in crate::encounters) fn request_requirements(
    attitude: &atlas_local_state::RelationshipKnowledge,
    reasonable: Option<bool>,
    against_goals: Option<bool>,
) -> RequestRequirements {
    use atlas_local_state::{
        AttitudeState as A, RelationshipKnowledge as K, RelationshipValue as V,
    };
    let result = |outcome, reason: &str, required_facts: Vec<String>| RequestRequirements {
        outcome,
        reason: reason.into(),
        required_facts,
    };
    let K::Known {
        value: V::Attitude(attitude),
        ..
    } = attitude
    else {
        return result(
            RequestOutcome::NeedsContext,
            "Resolve the creature's attitude first",
            vec!["Attitude toward the requester".into()],
        );
    };
    if !matches!(attitude, A::Friendly | A::Helpful) {
        return result(
            RequestOutcome::Prohibited,
            "Request requires a friendly or helpful attitude",
            vec![],
        );
    }
    if reasonable == Some(false) || (*attitude == A::Helpful && against_goals == Some(true)) {
        return result(
            RequestOutcome::Prohibited,
            "The creature would not accept this request",
            vec![],
        );
    }
    let mut facts = vec![];
    if reasonable.is_none() {
        facts.push("Is the request reasonable for this creature?".into());
    }
    if *attitude == A::Helpful && against_goals.is_none() {
        facts.push("Does it conflict with the creature's goals or quality of life?".into());
    }
    if !facts.is_empty() {
        return result(RequestOutcome::NeedsContext, "Confirm the request", facts);
    }
    result(
        RequestOutcome::CheckRequired,
        "Diplomacy against the GM's request DC",
        vec![],
    )
}

/// End named causes for one directed pair without erasing other observers' sources.
pub(in crate::encounters) fn end_condition_scopes(
    snapshot: &atlas_local_state::EncounterWithParticipants,
    subject: &EncounterParticipant,
    other: &str,
    revision: i64,
    rules: &[ConditionRule],
) -> Vec<atlas_local_state::ConditionEventChange> {
    use atlas_local_state::{
        ConditionEventChange, ConditionScope, UpdateEncounterParticipantCondition,
    };
    let mut changes = vec![];
    for source in &subject.conditions {
        if !condition_rule_for_key(source.condition_key.as_deref())
            .is_some_and(|rule| rules.contains(&rule))
            || !scope_applies(&source.scope, Some(other))
        {
            continue;
        }
        let remaining: Vec<_> = snapshot
            .participants
            .iter()
            .filter(|p| p.participant_key != other && p.participant_key != subject.participant_key)
            .filter(|p| scope_applies(&source.scope, Some(&p.participant_key)))
            .map(|p| p.participant_key.clone())
            .collect();
        if remaining.is_empty() {
            changes.push(ConditionEventChange::Remove {
                participant_key: subject.participant_key.clone(),
                condition_id: source.condition_id,
            });
        } else {
            changes.push(ConditionEventChange::Replace(
                UpdateEncounterParticipantCondition {
                    expected_encounter_revision: revision,
                    participant_key: subject.participant_key.clone(),
                    condition_id: source.condition_id,
                    condition_key: source.condition_key.clone(),
                    name: source.name.clone(),
                    value: source.value,
                    source_participant_key: source.source_participant_key.clone(),
                    scope: ConditionScope::SelectedParticipants {
                        participant_keys: remaining,
                    },
                    extra_action: source.extra_action.clone(),
                    immunity_override: source.immunity_override,
                    parameters: source.parameters.clone(),
                    origin: source.origin.clone(),
                    duration: source.duration.clone(),
                    note: source.note.clone(),
                },
            ));
        }
    }
    changes
}

pub(in crate::encounters) fn relationship_source_ids(
    subject: &EncounterParticipant,
    other: &str,
    property: atlas_local_state::RelationshipProperty,
) -> Vec<i64> {
    use atlas_local_state::RelationshipProperty as P;
    let mut ids: Vec<_> = super::causes::condition_causes(subject, Some(other))
        .into_iter()
        .filter(|cause| match property {
            P::Detection => matches!(
                cause.rule,
                ConditionRule::Hidden
                    | ConditionRule::Undetected
                    | ConditionRule::Unnoticed
                    | ConditionRule::Invisible
            ),
            P::Awareness => cause.rule == ConditionRule::Unnoticed,
            P::Attitude => matches!(
                cause.rule,
                ConditionRule::Helpful
                    | ConditionRule::Friendly
                    | ConditionRule::Indifferent
                    | ConditionRule::Unfriendly
                    | ConditionRule::Hostile
            ),
            P::Control => cause.rule == ConditionRule::Controlled,
        })
        .map(|cause| cause.source.condition_id)
        .collect();
    ids.sort_unstable();
    ids.dedup();
    ids
}

/// Select baseline and current-source adjudication independently.
pub(in crate::encounters) fn relationship_knowledge(
    subject: &EncounterParticipant,
    other: &str,
    property: atlas_local_state::RelationshipProperty,
    rows: &[atlas_local_state::DirectedRelationship],
) -> atlas_local_state::RelationshipKnowledge {
    use atlas_local_state::RelationshipLayer;
    let find = |layer| {
        rows.iter()
            .find(|row| {
                row.subject_key == subject.participant_key
                    && row.other_key == other
                    && row.property == property
                    && row.layer == layer
            })
            .map(|row| &row.knowledge)
    };
    resolve_relationship(
        subject,
        other,
        property,
        find(RelationshipLayer::Baseline),
        find(RelationshipLayer::Adjudication),
    )
}
pub(in crate::encounters) fn relationship_source_snapshot(
    subject: &EncounterParticipant,
    other: &str,
    property: atlas_local_state::RelationshipProperty,
) -> Vec<atlas_local_state::RelationshipSourceSnapshot> {
    let ids = relationship_source_ids(subject, other, property);
    let mut sources: Vec<_> = subject
        .conditions
        .iter()
        .filter(|source| ids.contains(&source.condition_id))
        .map(|source| atlas_local_state::RelationshipSourceSnapshot {
            condition_id: source.condition_id,
            condition_key: source.condition_key.clone(),
            value: source.value,
            source_participant_key: source.source_participant_key.clone(),
            scope: source.scope.clone(),
            parameters: source.parameters.clone(),
            note: source.note.clone(),
        })
        .collect();
    sources.sort_by_key(|source| source.condition_id);
    sources
}

#[cfg(test)]
mod tests {
    use super::super::numerical_tests::{condition, participant};
    use super::*;
    use atlas_local_state::*;
    fn resolve_relationship(
        subject: &EncounterParticipant,
        other: &str,
        property: RelationshipProperty,
        baseline: Option<&RelationshipKnowledge>,
    ) -> RelationshipKnowledge {
        super::resolve_relationship(subject, other, property, baseline, None)
    }

    #[test]
    fn formal_requests_require_attitude_and_do_not_turn_helpfulness_into_automatic_success() {
        let helpful = RelationshipKnowledge::Known {
            value: RelationshipValue::Attitude(AttitudeState::Helpful),
            provenance: RelationshipProvenance::Table,
        };
        assert_eq!(
            request_requirements(&helpful, None, None).outcome,
            RequestOutcome::NeedsContext
        );
        assert_eq!(
            request_requirements(&helpful, Some(true), Some(true)).outcome,
            RequestOutcome::Prohibited
        );
        assert_eq!(
            request_requirements(&helpful, Some(true), Some(false)).outcome,
            RequestOutcome::CheckRequired
        );
        let hostile = RelationshipKnowledge::Known {
            value: RelationshipValue::Attitude(AttitudeState::Hostile),
            provenance: RelationshipProvenance::Table,
        };
        assert_eq!(
            request_requirements(&hostile, Some(true), Some(false)).outcome,
            RequestOutcome::Prohibited
        );
    }
    #[test]
    fn absence_and_explicit_unknown_are_distinct_and_pc_attitudes_are_not_invented() {
        let mut subject = participant();
        assert!(matches!(
            resolve_relationship(&subject, "observer", RelationshipProperty::Detection, None),
            RelationshipKnowledge::Unknown { .. }
        ));
        assert!(matches!(
            resolve_relationship(&subject, "observer", RelationshipProperty::Attitude, None),
            RelationshipKnowledge::Known {
                value: RelationshipValue::Attitude(AttitudeState::Indifferent),
                provenance: RelationshipProvenance::SourceDefault
            }
        ));
        let unknown = RelationshipKnowledge::Unknown {
            required_facts: vec!["Imported attitude needs review".into()],
        };
        assert_eq!(
            resolve_relationship(
                &subject,
                "observer",
                RelationshipProperty::Attitude,
                Some(&unknown)
            ),
            unknown
        );
        subject.participant_kind = ParticipantKind::Pc;
        assert!(matches!(
            resolve_relationship(&subject, "observer", RelationshipProperty::Attitude, None),
            RelationshipKnowledge::Unknown { .. }
        ));
    }
    #[test]
    fn opposing_attitude_sources_conflict_only_in_overlapping_scope() {
        let mut subject = participant();
        let mut friendly = condition(1, ConditionRule::Friendly, None);
        friendly.scope = ConditionScope::SelectedParticipants {
            participant_keys: vec!["a".into()],
        };
        subject.conditions = vec![friendly, condition(2, ConditionRule::Hostile, None)];
        assert!(
            matches!(resolve_relationship(&subject,"a",RelationshipProperty::Attitude,None),RelationshipKnowledge::Conflicting{candidates} if candidates.len()==2)
        );
        assert!(matches!(
            resolve_relationship(&subject, "b", RelationshipProperty::Attitude, None),
            RelationshipKnowledge::Known {
                value: RelationshipValue::Attitude(AttitudeState::Hostile),
                ..
            }
        ));
    }
    #[test]
    fn observed_adjudication_suppresses_hidden_defense_penalty() {
        let mut subject = participant();
        subject.conditions = vec![condition(1, ConditionRule::Hidden, None)];
        let rows = vec![atlas_local_state::DirectedRelationship {
            subject_key: subject.participant_key.clone(),
            other_key: "observer".into(),
            property: RelationshipProperty::Detection,
            layer: atlas_local_state::RelationshipLayer::Adjudication,
            knowledge: RelationshipKnowledge::Known {
                value: RelationshipValue::Detection(DetectionState::Observed),
                provenance: RelationshipProvenance::Adjudication {
                    sources: relationship_source_snapshot(
                        &subject,
                        "observer",
                        RelationshipProperty::Detection,
                    ),
                },
            },
        }];
        let roster = vec![subject];
        assert_eq!(observer_defense_causes("observer", &roster, &[]).len(), 1);
        assert!(observer_defense_causes("observer", &roster, &rows).is_empty());
        assert!(observer_defense_modifiers("observer", &roster, &rows).is_empty());
    }
    #[test]
    fn invisible_invalidates_old_observed_and_requires_current_source_assessment() {
        let mut subject = participant();
        subject.conditions = vec![condition(1, ConditionRule::Invisible, None)];
        let known = |provenance| RelationshipKnowledge::Known {
            value: RelationshipValue::Detection(DetectionState::Observed),
            provenance,
        };
        assert!(matches!(
            resolve_relationship(
                &subject,
                "a",
                RelationshipProperty::Detection,
                Some(&known(RelationshipProvenance::Table))
            ),
            RelationshipKnowledge::Unknown { .. }
        ));
        let assessed = known(RelationshipProvenance::Adjudication {
            sources: relationship_source_snapshot(&subject, "a", RelationshipProperty::Detection),
        });
        assert_eq!(
            super::resolve_relationship(
                &subject,
                "a",
                RelationshipProperty::Detection,
                None,
                Some(&assessed)
            ),
            assessed
        );
        subject
            .conditions
            .push(condition(2, ConditionRule::Invisible, None));
        assert!(matches!(
            super::resolve_relationship(
                &subject,
                "a",
                RelationshipProperty::Detection,
                None,
                Some(&assessed)
            ),
            RelationshipKnowledge::Invalidated { .. }
        ));
    }
    #[test]
    fn stale_source_knowledge_is_invalidated_and_undetected_does_not_invent_unawareness() {
        let mut subject = participant();
        subject.conditions = vec![condition(1, ConditionRule::Undetected, None)];
        assert!(matches!(
            resolve_relationship(&subject, "a", RelationshipProperty::Awareness, None),
            RelationshipKnowledge::Unknown { .. }
        ));
        let source = RelationshipProvenance::Source {
            condition_id: Some(8),
            record_key: None,
            application_id: None,
        };
        let stale = RelationshipKnowledge::Known {
            value: RelationshipValue::Attitude(AttitudeState::Friendly),
            provenance: source.clone(),
        };
        assert_eq!(
            resolve_relationship(&subject, "a", RelationshipProperty::Attitude, Some(&stale)),
            RelationshipKnowledge::Invalidated {
                prior_source: source
            }
        );
    }
}
