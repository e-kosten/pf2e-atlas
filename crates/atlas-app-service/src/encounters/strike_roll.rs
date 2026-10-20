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

    pub fn roll_encounter_strike(
        &self,
        encounter_ref: &str,
        request: RollEncounterStrikeRequest,
    ) -> AppServiceResult<EncounterStrikeRollView> {
        let intent = serde_json::json!({"action":"strike", "request":request});
        let (encounter, recovered) = self.with_state(|session| {
            let encounters = session.local_state().encounters();
            let encounter = encounters
                .get(encounter_ref)?
                .ok_or_else(|| super::projection::encounter_not_found(encounter_ref))?;
            let recovered = encounters.strike_roll_receipt(
                &encounter.encounter_key,
                &request.request_id,
                &intent,
            )?;
            Ok((encounter, recovered))
        })?;
        if let Some(receipt) = recovered {
            return Ok(strike_receipt_view(receipt));
        }
        let key = &encounter.encounter_key;
        if !(1..=3).contains(&request.attack_number) {
            return Err(AppServiceError::invalid_request("Choose attack 1, 2 or 3"));
        }
        let lookup_key = key.clone();
        let lookup = EncounterStrikeOptionsRequest {
            participant_key: request.participant_key.clone(),
            reference: request.reference.clone(),
        };
        let (options, activity, reference_revision) = self.with_workspace(
            &crate::ReadExecutionContext::uncancellable(),
            move |workspace| {
                let (options, activity) = resolve_strike(workspace, &lookup_key, lookup)?;
                Ok((options, activity, workspace.reference_revision()?))
            },
        )?;
        if options.expected_revision != request.expected_revision {
            return Err(atlas_local_state::LocalStateError::StaleEncounterRevision {
                encounter_key: key.clone(),
                expected_revision: request.expected_revision,
                actual_revision: options.expected_revision,
            }
            .into());
        }
        if options.expected_participant_revision != request.expected_participant_revision {
            return Err(
                atlas_local_state::LocalStateError::StaleParticipantRevision {
                    participant_key: request.participant_key.clone(),
                    expected_revision: request.expected_participant_revision,
                    actual_revision: options.expected_participant_revision,
                }
                .into(),
            );
        }
        let prepared = if request.roll_kind == StrikeRollKind::Attack {
            let modifier = request
                .modifier_override
                .or_else(|| {
                    options
                        .attacks
                        .iter()
                        .find(|a| a.attack_number == request.attack_number)
                        .map(|a| a.modifier)
                })
                .ok_or_else(|| AppServiceError::invalid_request("Enter an attack modifier"))?;
            if !(-10000..=10000).contains(&modifier) {
                return Err(AppServiceError::invalid_request(
                    "Attack modifier exceeds supported bounds",
                ));
            }
            let dice = match request.mode {
                RollMode::Normal => "1d20",
                RollMode::Fortune => "2d20kh1",
                RollMode::Misfortune => "2d20kl1",
            };
            Some(
                atlas_dice::prepare(&format!("{dice}{modifier:+}"))
                    .map_err(atlas_local_state::LocalStateError::from)?,
            )
        } else {
            if let Some(reason) = options.damage_unavailable {
                return Err(AppServiceError::invalid_request(reason));
            }
            None
        };
        let damage = if request.roll_kind == StrikeRollKind::Damage {
            activity
                .damage
                .value
                .unwrap_or_default()
                .into_iter()
                .filter(|p| {
                    p.formula
                        .value
                        .as_ref()
                        .is_none_or(|f| !f.trim().is_empty())
                })
                .map(|part| {
                    let formula = part.formula.value.ok_or_else(|| {
                        AppServiceError::invalid_request("Damage formula unavailable")
                    })?;
                    let prepared = atlas_dice::prepare(&formula)
                        .map_err(atlas_local_state::LocalStateError::from)?;
                    let label = [part.damage_type.value.clone(), part.category.value.clone()]
                        .into_iter()
                        .flatten()
                        .collect::<Vec<_>>()
                        .join(" ");
                    let mut characters = label.chars();
                    let label = characters
                        .next()
                        .map(|first| format!("{}{}", first.to_uppercase(), characters.as_str()))
                        .unwrap_or_else(|| "Untyped".into());
                    Ok((
                        part.id,
                        label,
                        formula,
                        part.damage_type.value,
                        part.category.value,
                        prepared,
                    ))
                })
                .collect::<AppServiceResult<Vec<_>>>()?
        } else {
            vec![]
        };
        self.with_write(|session| {
            let encounters = session.local_state().encounters();
            if let Some(receipt) =
                encounters.strike_roll_receipt(key, &request.request_id, &intent)?
            {
                return Ok(strike_receipt_view(receipt));
            }
            session.require_reference_revision(reference_revision)?;
            let receipt = encounters.roll_strike(
                key,
                &request.request_id,
                &intent,
                atlas_local_state::ParticipantRollGuard {
                    expected_revision: request.expected_revision,
                    participant_key: &request.participant_key,
                    expected_participant_revision: request.expected_participant_revision,
                },
                || {
                    let components = damage
                        .into_iter()
                        .map(|(id, label, formula, damage_type, category, p)| {
                            Ok(StrikeDamageRollView {
                                id,
                                label,
                                formula: Some(formula),
                                damage_type,
                                category,
                                outcome: Some(p.roll()?),
                                unavailable: None,
                            })
                        })
                        .collect::<atlas_local_state::LocalStateResult<Vec<_>>>()?;
                    let damage_totals = damage_totals(&components)?;
                    let (damage_total, damage_minimum_adjustment) =
                        damage_sum(&damage_totals, false)?;
                    let (persistent_total, persistent_minimum_adjustment) =
                        damage_sum(&damage_totals, true)?;
                    let outcome = if let Some(attack) = prepared {
                        atlas_local_state::StoredStrikeOutcome::Attack {
                            outcome: attack.roll()?,
                        }
                    } else {
                        atlas_local_state::StoredStrikeOutcome::Damage {
                            components: components
                                .into_iter()
                                .map(|p| {
                                    Ok(atlas_local_state::StoredStrikeDamage {
                                        id: p.id,
                                        formula: p.formula.ok_or(
                                            atlas_local_state::LocalStateError::InvalidRollRequest(
                                                "missing prepared formula",
                                            ),
                                        )?,
                                        damage_type: p.damage_type,
                                        category: p.category,
                                        outcome: p.outcome.ok_or(
                                            atlas_local_state::LocalStateError::InvalidRollRequest(
                                                "missing evaluated damage",
                                            ),
                                        )?,
                                    })
                                })
                                .collect::<atlas_local_state::LocalStateResult<Vec<_>>>()?,
                            totals: damage_totals
                                .into_iter()
                                .map(|t| atlas_local_state::StoredStrikeDamageTotal {
                                    damage_type: t.damage_type,
                                    persistent: t.persistent,
                                    raw_total: t.raw_total,
                                    total: t.total,
                                })
                                .collect(),
                            total: damage_total,
                            persistent_total,
                            minimum_adjustment: damage_minimum_adjustment,
                            persistent_minimum_adjustment,
                        }
                    };
                    Ok(atlas_local_state::StoredStrikeRoll {
                        title: options.title,
                        outcome,
                    })
                },
            )?;
            Ok(strike_receipt_view(receipt))
        })
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
            "casting_ranks":[],"attack_options":[],"has_reference_details":false,
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
                .map(|a| a.modifier)
                .collect::<Vec<_>>(),
            [12, 7, 2]
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
                .map(|a| a.modifier)
                .collect::<Vec<_>>(),
            [12, 8, 4]
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
        assert_eq!((groups[0].raw_total, groups[0].total), (-1, 0));
        assert_eq!(damage_sum(&groups, false).unwrap(), (Some(2), 0));
        assert_eq!(damage_sum(&groups, true).unwrap(), (Some(8), 0));
        assert_eq!(components[0].outcome.as_ref().unwrap().total, -5);
        let zero = damage_totals(&[component(-5, "fire", None)]).unwrap();
        assert_eq!(damage_sum(&zero, false).unwrap(), (Some(1), 1));
        assert_eq!(damage_sum(&zero, true).unwrap(), (None, 0));
    }
}

fn resolve_strike(
    workspace: &mut atlas_runtime::WorkspaceRead,
    encounter_ref: &str,
    request: EncounterStrikeOptionsRequest,
) -> AppServiceResult<(EncounterStrikeOptionsView, ActorActivityView)> {
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
    let selected = super::reference::read_encounter_reference(
        workspace,
        encounter_ref,
        &request.participant_key,
        request.reference,
    )?;
    let RecordBodyView::Activity(activity) = selected.presentation.body else {
        return Err(AppServiceError::invalid_request("Select a strike"));
    };
    if activity.kind != ActorActivityKindView::Strike {
        return Err(AppServiceError::invalid_request("Select a strike"));
    }
    let attacks = attack_options(&activity);
    let parts = activity.damage.value.as_deref().unwrap_or_default();
    let damage_unavailable = if parts.is_empty()
        || parts.iter().all(|p| {
            p.formula
                .value
                .as_ref()
                .is_some_and(|f| f.trim().is_empty())
        }) {
        Some("This strike has no numeric damage.".into())
    } else if parts.iter().any(|p| {
        p.formula
            .value
            .as_ref()
            .is_none_or(|f| !f.trim().is_empty() && atlas_dice::prepare(f).is_err())
    }) {
        Some("This strike has an unavailable or unsupported damage formula.".into())
    } else {
        None
    };
    Ok((
        EncounterStrikeOptionsView {
            expected_revision: snapshot.encounter.revision,
            expected_participant_revision: participant.revision,
            title: activity.title.clone(),
            unavailable: attacks
                .is_empty()
                .then(|| "Attack modifier unavailable.".into()),
            attacks,
            damage_available: damage_unavailable.is_none(),
            damage_unavailable,
        },
        *activity,
    ))
}

fn strike_receipt_view(receipt: atlas_local_state::StoredStrikeReceipt) -> EncounterStrikeRollView {
    let mut view = EncounterStrikeRollView {
        request_id: receipt.request_id,
        encounter_key: receipt.encounter_key,
        participant_key: receipt.participant_key,
        title: receipt.roll.title,
        attack: None,
        damage: vec![],
        damage_totals: vec![],
        damage_total: None,
        persistent_total: None,
        damage_minimum_adjustment: 0,
        persistent_minimum_adjustment: 0,
    };
    match receipt.roll.outcome {
        atlas_local_state::StoredStrikeOutcome::Attack { outcome } => view.attack = Some(outcome),
        atlas_local_state::StoredStrikeOutcome::Damage {
            components,
            totals,
            total,
            persistent_total,
            minimum_adjustment,
            persistent_minimum_adjustment,
        } => {
            view.damage = components
                .into_iter()
                .map(|p| {
                    let label = [p.damage_type.clone(), p.category.clone()]
                        .into_iter()
                        .flatten()
                        .collect::<Vec<_>>()
                        .join(" ");
                    let mut chars = label.chars();
                    let label = chars
                        .next()
                        .map(|first| format!("{}{}", first.to_uppercase(), chars.as_str()))
                        .unwrap_or_else(|| "Untyped".into());
                    StrikeDamageRollView {
                        id: p.id,
                        label,
                        formula: Some(p.formula),
                        damage_type: p.damage_type,
                        category: p.category,
                        outcome: Some(p.outcome),
                        unavailable: None,
                    }
                })
                .collect();
            view.damage_totals = totals
                .into_iter()
                .map(|t| StrikeDamageTotalView {
                    damage_type: t.damage_type,
                    persistent: t.persistent,
                    raw_total: t.raw_total,
                    total: t.total,
                })
                .collect();
            view.damage_total = total;
            view.persistent_total = persistent_total;
            view.damage_minimum_adjustment = minimum_adjustment;
            view.persistent_minimum_adjustment = persistent_minimum_adjustment;
        }
    }
    view
}
