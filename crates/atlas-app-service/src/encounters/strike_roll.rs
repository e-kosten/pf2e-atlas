#[cfg(test)]
use crate::presentation::strike::attack_options;
use crate::{AppServiceError, AppServiceResult, AtlasAppService};
use atlas_app_model::*;

impl AtlasAppService {
    pub fn encounter_strike_options(
        &self,
        encounter_ref: &str,
        request: EncounterStrikeOptionsRequest,
    ) -> AppServiceResult<EncounterStrikeOptionsView> {
        let encounter_ref = encounter_ref.to_owned();
        self.with_workspace(
            &crate::ReadExecutionContext::uncancellable(),
            move |workspace| Ok(resolve_strike(workspace, &encounter_ref, request)?.0),
        )
    }

    /// Display-only dice: evaluate against one checked snapshot without saved-state writes.
    pub fn roll_encounter_strike(
        &self,
        encounter_ref: &str,
        request: RollEncounterStrikeRequest,
    ) -> AppServiceResult<EncounterStrikeRollView> {
        if request.request_id.is_empty() || request.request_id.len() > 128 {
            return Err(AppServiceError::invalid_request("Invalid roll request ID."));
        }
        if !(1..=3).contains(&request.attack_number) {
            return Err(AppServiceError::invalid_request("Choose attack 1, 2 or 3"));
        }
        let encounter_ref = encounter_ref.to_owned();
        self.with_workspace(
            &crate::ReadExecutionContext::uncancellable(),
            move |workspace| {
                let encounter = workspace
                    .local_state()
                    .encounters()
                    .get(&encounter_ref)?
                    .ok_or_else(|| super::projection::encounter_not_found(&encounter_ref))?;
                let (options, activity) = resolve_strike(
                    workspace,
                    &encounter.encounter_key,
                    EncounterStrikeOptionsRequest {
                        participant_key: request.participant_key.clone(),
                        reference: request.reference,
                    },
                )?;
                if options.expected_revision != request.expected_revision {
                    return Err(atlas_local_state::LocalStateError::StaleEncounterRevision {
                        encounter_key: encounter.encounter_key,
                        expected_revision: request.expected_revision,
                        actual_revision: options.expected_revision,
                    }
                    .into());
                }
                if options.expected_participant_revision != request.expected_participant_revision {
                    return Err(
                        atlas_local_state::LocalStateError::StaleParticipantRevision {
                            participant_key: request.participant_key,
                            expected_revision: request.expected_participant_revision,
                            actual_revision: options.expected_participant_revision,
                        }
                        .into(),
                    );
                }
                let mut view = EncounterStrikeRollView {
                    request_id: request.request_id,
                    encounter_key: encounter.encounter_key,
                    participant_key: request.participant_key,
                    title: options.title,
                    attack: None,
                    attack_modifier: None,
                    damage: vec![],
                    damage_totals: vec![],
                    damage_total: None,
                    persistent_total: None,
                    damage_minimum_adjustment: 0,
                    persistent_minimum_adjustment: 0,
                };
                if request.roll_kind == StrikeRollKind::Attack {
                    let modifier = request
                        .modifier_override
                        .or_else(|| {
                            options
                                .attacks
                                .iter()
                                .find(|attack| attack.attack_number == request.attack_number)
                                .map(|attack| attack.modifier)
                        })
                        .filter(|modifier| (-10000..=10000).contains(modifier))
                        .ok_or_else(|| {
                            AppServiceError::invalid_request("Attack modifier unavailable.")
                        })?;
                    let die = match request.mode {
                        RollMode::Normal => "1d20",
                        RollMode::Fortune => "2d20kh1",
                        RollMode::Misfortune => "2d20kl1",
                    };
                    view.attack = Some(
                        atlas_dice::roll(&format!("{die}{modifier:+}"))
                            .map_err(atlas_local_state::LocalStateError::from)?,
                    );
                    view.attack_modifier = Some(modifier);
                    return Ok(view);
                }
                if let Some(reason) = options.damage_unavailable {
                    return Err(AppServiceError::invalid_request(reason));
                }
                // Validate the complete pool before drawing any dice.
                let damage = activity
                    .damage
                    .into_iter()
                    .filter(|part| !part.formula.trim().is_empty())
                    .map(|part| {
                        if let Some(reason) = part.unavailable_reason {
                            return Err(AppServiceError::invalid_request(reason));
                        }
                        let formula = part.adjusted_formula.unwrap_or(part.formula);
                        let prepared = atlas_dice::prepare(&formula)
                            .map_err(atlas_local_state::LocalStateError::from)?;
                        let label = [part.damage_type.clone(), part.category.clone()]
                            .into_iter()
                            .flatten()
                            .collect::<Vec<_>>()
                            .join(" ");
                        let mut chars = label.chars();
                        let label = chars
                            .next()
                            .map(|first| format!("{}{}", first.to_uppercase(), chars.as_str()))
                            .unwrap_or_else(|| "Untyped".into());
                        Ok((
                            StrikeDamageRollView {
                                id: part.damage_id,
                                label,
                                formula: Some(formula),
                                damage_type: part.damage_type,
                                category: part.category,
                                outcome: None,
                                unavailable: None,
                            },
                            prepared,
                        ))
                    })
                    .collect::<AppServiceResult<Vec<_>>>()?;
                view.damage = damage
                    .into_iter()
                    .map(|(mut component, prepared)| {
                        component.outcome = Some(prepared.roll()?);
                        Ok(component)
                    })
                    .collect::<atlas_local_state::LocalStateResult<Vec<_>>>()?;
                view.damage_totals = damage_totals(&view.damage)?;
                (view.damage_total, view.damage_minimum_adjustment) =
                    damage_sum(&view.damage_totals, false)?;
                (view.persistent_total, view.persistent_minimum_adjustment) =
                    damage_sum(&view.damage_totals, true)?;
                Ok(view)
            },
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn strike_attack_penalties_preserve_agile_and_unknown_modifiers() {
        let mut activity: ActorActivityView = serde_json::from_value(serde_json::json!({
            "effect_duration":{"state":"not_applicable","value":null},
            "self_effect":{"state":"not_applicable","value":null},
            "strike_effects":{"state":"value","value":[]},
            "focus_capacity":{"state":"not_applicable","value":null,"adjustment":null},
            "frequency":{"state":"not_applicable","value":null},
            "strike_range":{"state":"not_applicable","value":null},
            "reading_order":0,"casting_ranks":[],"attack_options":[],"has_reference_details":false,
            "navigation":{"record_key":"pack:id","embedded_path":[]},"is_ranged":{"state":"value","value":false},"title":"Claw","family":"melee","kind":"strike",
            "usage":null,"action_cost":{"state":"missing","value":null},"traits":[],
            "attack":{"state":"value","value":12,"adjustment":null},
            "lore_modifier":{"state":"missing","value":null,"adjustment":null},
            "damage":{"state":"missing","value":null},
            "difficulty_class":{"state":"missing","value":null,"adjustment":null},
            "casting_tradition":{"state":"missing","value":null},"preparation":{"state":"missing","value":null},"casting_entry":null,
            "association":{"state":"missing","value":null},
            "base_rank":{"state":"missing","value":null,"adjustment":null},"authored_cast_rank":{"state":"missing","value":null,"adjustment":null},"casting_context":null,"notes":[]
        })).unwrap();
        assert_eq!(
            attack_options(&activity)
                .iter()
                .map(|a| (a.attack_number, a.modifier))
                .collect::<Vec<_>>(),
            [(1, 12), (2, 7), (3, 2)]
        );
        activity.traits.push(RecordBadgeView {
            trait_key: Some("trait:agile".into()),
            kind: "trait".into(),
            label: "Agile".into(),
            value: "agile".into(),
        });
        assert_eq!(
            attack_options(&activity)
                .iter()
                .map(|a| (a.attack_number, a.modifier))
                .collect::<Vec<_>>(),
            [(1, 12), (2, 8), (3, 4)]
        );
        activity.attack.value = None;
        assert!(attack_options(&activity).is_empty());
    }
}

// PF2e 8.6.1 damage/formula.ts combines precision/splash into damage-type
// instances; persistent remains a separate instance. DamageInstance.total floors
// each instance at zero and DamageRoll._evaluateASTAsync floors the roll at one.
fn damage_totals(
    components: &[StrikeDamageRollView],
) -> atlas_local_state::LocalStateResult<Vec<StrikeDamageTotalView>> {
    let mut totals: Vec<StrikeDamageTotalView> = vec![];
    for component in components {
        let Some(outcome) = &component.outcome else {
            continue;
        };
        let damage_type = component.damage_type.as_deref().unwrap_or("untyped");
        let persistent = component.category.as_deref() == Some("persistent");
        if let Some(group) = totals
            .iter_mut()
            .find(|g| g.damage_type == damage_type && g.persistent == persistent)
        {
            group.raw_total = group
                .raw_total
                .checked_add(outcome.total)
                .ok_or(atlas_dice::DiceError::Limit)?;
            group.total = group.raw_total.max(0);
        } else {
            totals.push(StrikeDamageTotalView {
                damage_type: damage_type.into(),
                persistent,
                raw_total: outcome.total,
                total: outcome.total.max(0),
            });
        }
    }
    Ok(totals)
}
fn damage_sum(
    groups: &[StrikeDamageTotalView],
    persistent: bool,
) -> atlas_local_state::LocalStateResult<(Option<i32>, i32)> {
    let selected: Vec<_> = groups
        .iter()
        .filter(|g| g.persistent == persistent)
        .collect();
    if selected.is_empty() {
        return Ok((None, 0));
    }
    let total = selected.iter().try_fold(0_i32, |sum, g| {
        sum.checked_add(g.total).ok_or(atlas_dice::DiceError::Limit)
    })?;
    let minimum_adjustment = i32::from(total == 0);
    Ok((Some(total + minimum_adjustment), minimum_adjustment))
}

#[cfg(test)]
mod damage_tests {
    use super::*;
    fn component(value: i32, kind: &str, category: Option<&str>) -> StrikeDamageRollView {
        StrikeDamageRollView {
            id: format!("{kind}-{category:?}"),
            label: kind.into(),
            formula: Some(value.to_string()),
            damage_type: Some(kind.into()),
            category: category.map(str::to_owned),
            outcome: Some(
                atlas_dice::prepare(&value.to_string())
                    .unwrap()
                    .roll()
                    .unwrap(),
            ),
            unavailable: None,
        }
    }
    #[test]
    fn damage_groups_before_flooring_and_keeps_persistent_separate() {
        let components = vec![
            component(-5, "fire", None),
            component(3, "fire", Some("precision")),
            component(1, "fire", Some("splash")),
            component(8, "fire", Some("persistent")),
            component(2, "cold", None),
        ];
        let groups = damage_totals(&components).unwrap();
        assert_eq!(
            groups
                .iter()
                .map(|group| (
                    group.damage_type.as_str(),
                    group.persistent,
                    group.raw_total,
                    group.total
                ))
                .collect::<Vec<_>>(),
            [
                ("fire", false, -1, 0),
                ("fire", true, 8, 8),
                ("cold", false, 2, 2)
            ]
        );
        assert_eq!(damage_sum(&groups, false).unwrap(), (Some(2), 0));
        assert_eq!(damage_sum(&groups, true).unwrap(), (Some(8), 0));
        assert_eq!(components[0].outcome.as_ref().unwrap().total, -5);
        let zero = damage_totals(&[component(-5, "fire", None)]).unwrap();
        assert_eq!(damage_sum(&zero, false).unwrap(), (Some(1), 1));
        assert_eq!(damage_sum(&zero, true).unwrap(), (None, 0));
    }
}

pub(super) fn resolve_strike(
    workspace: &mut atlas_runtime::WorkspaceRead,
    encounter_ref: &str,
    request: EncounterStrikeOptionsRequest,
) -> AppServiceResult<(
    EncounterStrikeOptionsView,
    super::stat_block::MechanicActivityView,
)> {
    let snapshot = workspace
        .local_state()
        .encounters()
        .get_with_participants(encounter_ref)?
        .ok_or_else(|| super::projection::encounter_not_found(encounter_ref))?;
    let participant = snapshot
        .participants
        .iter()
        .find(|p| p.participant_key == request.participant_key)
        .ok_or_else(|| {
            AppServiceError::invalid_request("Participant is no longer in this encounter")
        })?;
    if participant.record_key.as_deref() != Some(request.reference.selection.record_key.as_str()) {
        return Err(AppServiceError::invalid_request(
            "Selected strike does not belong to this participant.",
        ));
    }
    let retrieval = workspace.retrieval_mut();
    let generation = retrieval.artifact_generation()?;
    if generation != request.reference.artifact_generation {
        return Err(AppServiceError::artifact_generation_changed(
            &request.reference.artifact_generation,
            &generation,
            AppRecoverableAction::ReloadRecord,
        ));
    }
    if request
        .reference
        .selection
        .source_fingerprint
        .as_deref()
        .is_some_and(|fingerprint| fingerprint != retrieval.source_fingerprint())
    {
        return Err(AppServiceError::invalid_request(
            "Strike source changed; reopen the attack.",
        ));
    }
    let (source, selected) =
        crate::records::read_record_detail(retrieval, request.reference.selection)?;
    let baseline = super::hydration::source_baseline(&source.source, retrieval, None)?
        .ok_or_else(|| AppServiceError::invalid_request("Actor statistics are unavailable."))?;
    let adjustment_applicable = !matches!(
        atlas_record::source_record::SourceQueryView::new(
            source.source.source(),
            source.source.key().pack().as_str(),
            ""
        )
        .actor()
        .authored_adjustment(),
        atlas_record::source_record::SourceFieldView::NotApplicable
    );
    let causes = super::rules::relationships::observer_defense_causes(
        &participant.participant_key,
        &snapshot.participants,
    );
    let block =
        super::rules::project_participant(participant, &baseline, adjustment_applicable, &causes);
    let activity = block
        .activities
        .into_iter()
        .find(|activity| {
            activity.navigation.owners == selected.selected.owners
                && activity.kind == super::stat_block::MechanicActivityKindView::Strike
        })
        .ok_or_else(|| AppServiceError::invalid_request("Select a strike."))?;
    if let Some(reason) = &activity.calculation_unavailable {
        return Err(AppServiceError::invalid_request(reason));
    }
    let attacks = crate::presentation::strike::project_attack_options(
        activity
            .rolls
            .first()
            .and_then(|roll| roll.adjusted_value.as_i64()),
        activity
            .traits_known
            .then(|| activity.traits.iter().any(|value| value == "agile")),
    );
    let parts = &activity.damage;
    let damage_unavailable = if parts.is_empty() {
        Some("This strike has no numeric damage.".into())
    } else if let Some(reason) = parts
        .iter()
        .find_map(|part| part.unavailable_reason.clone())
    {
        Some(reason)
    } else if parts.iter().any(|part| {
        atlas_dice::prepare(part.adjusted_formula.as_deref().unwrap_or(&part.formula)).is_err()
    }) {
        Some("This strike has an unavailable or unsupported damage formula.".into())
    } else {
        None
    };
    use super::rules::action_context::{ActionContext, ActionKind};
    let facts = super::action_projection::strike_facts(&activity);
    let evaluation = super::rules::action_context::evaluate_action(
        participant,
        &baseline.source_conditions,
        &ActionContext {
            kind: ActionKind::Strike,
            reaction: Some(false),
            other_participant: None,
            facts: &facts,
        },
    );
    let mut conditional = evaluation
        .issues
        .into_iter()
        .map(|issue| {
            let issue = super::action_projection::issue_view(
                issue,
                participant,
                &baseline.source_conditions,
            );
            RuntimeEffectNoteView {
                source: issue.source_key,
                label: issue.condition_name,
                reason: issue.reason,
            }
        })
        .collect::<Vec<_>>();
    if let Some(check) = evaluation.disruption {
        conditional.push(RuntimeEffectNoteView {
            source: "strike-flat-check".into(),
            label: check
                .source_ids
                .iter()
                .map(|id| id.label(std::slice::from_ref(participant)))
                .collect::<Vec<_>>()
                .join(", "),
            reason: format!("DC {} flat check or the action is disrupted", check.dc),
        });
    }
    for cause in
        super::rules::causes::condition_causes(participant, None, &baseline.source_conditions)
    {
        if cause.rule == super::conditions::ConditionRule::Blinded {
            conditional.push(RuntimeEffectNoteView {
                source: cause.source.explanation_key(),
                label: cause.source.name().to_owned(),
                reason: "Check detection".into(),
            });
        }
    }
    let applied = activity
        .rolls
        .first()
        .map(|roll| {
            roll.modifiers
                .iter()
                .map(|modifier| RuntimeEffectNoteView {
                    source: modifier.source.clone(),
                    label: modifier.label.clone(),
                    reason: format!("{:+} attack", modifier.value),
                })
                .collect()
        })
        .unwrap_or_default();
    Ok((
        EncounterStrikeOptionsView {
            applied,
            conditional,
            expected_revision: snapshot.encounter.revision,
            expected_participant_revision: participant.revision,
            title: activity.label.clone(),
            unavailable: attacks
                .is_empty()
                .then(|| "Attack modifier unavailable.".into()),
            attacks,
            damage_available: damage_unavailable.is_none(),
            damage_unavailable,
        },
        activity,
    ))
}
