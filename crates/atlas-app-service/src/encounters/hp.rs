use super::projection::{encounter_not_found, participant_side_view, participant_variant_view};
use crate::error::{AppServiceError, AppServiceResult};
use crate::service::AtlasAppService;
use atlas_app_model::{
    ApplyEncounterHpActionRequest, EncounterHpAction, EncounterMutationView,
    UpdateEncounterParticipantRequest,
};

impl AtlasAppService {
    pub fn apply_encounter_hp_action(
        &self,
        encounter_ref: &str,
        request: ApplyEncounterHpActionRequest,
    ) -> AppServiceResult<EncounterMutationView> {
        let detail = self.with_state(|session| {
            session
                .local_state()
                .encounters()
                .get_with_participants(encounter_ref)?
                .ok_or_else(|| encounter_not_found(encounter_ref))
        })?;
        let existing = detail
            .participants
            .iter()
            .find(|p| p.participant_key == request.participant_key)
            .ok_or_else(|| {
                AppServiceError::new(
                    atlas_app_model::AppErrorCode::EncounterParticipantNotFound,
                    "Encounter participant was not found",
                )
            })?;
        if request.expected_revision != existing.revision {
            return Err(
                atlas_local_state::LocalStateError::StaleParticipantRevision {
                    participant_key: request.participant_key,
                    expected_revision: request.expected_revision,
                    actual_revision: existing.revision,
                }
                .into(),
            );
        }
        let mut update = UpdateEncounterParticipantRequest {
            participant_key: existing.participant_key.clone(),
            expected_revision: request.expected_revision,
            hp_edit: false,
            max_hp_edit: false,
            variant_edit: false,
            use_derived_max: false,
            display_name: existing.display_name.clone(),
            side: participant_side_view(existing.side),
            participant_variant: participant_variant_view(existing.participant_variant),
            initiative: existing.initiative,
            current_hp: existing.current_hp,
            max_hp: existing.max_hp,
            temporary_hp: existing.temporary_hp,
            defeated: existing.defeated,
            hazard_disabled: existing.hazard_disabled,
            hidden: existing.hidden,
            note: existing.note.clone(),
        };
        let damage_requested = matches!(&request.action, EncounterHpAction::Damage { .. });
        match request.action {
            EncounterHpAction::Damage { amount } | EncounterHpAction::Heal { amount } => {
                let amount = formula_value(&amount, false)?;
                let damage = if amount < 0 {
                    !damage_requested
                } else {
                    damage_requested
                };
                // The signed safe-integer bound excludes i64::MIN.
                let amount = amount.abs();
                if damage {
                    let current = existing.current_hp.ok_or_else(|| {
                        AppServiceError::invalid_request("Set current HP before applying damage")
                    })?;
                    if existing.temporary_hp < 0 {
                        return Err(AppServiceError::invalid_request(
                            "Correct temporary HP before applying damage",
                        ));
                    }
                    let temporary_damage = existing.temporary_hp.min(amount);
                    let remaining = amount - temporary_damage;
                    update.temporary_hp -= temporary_damage;
                    update.current_hp = Some(current.saturating_sub(remaining).max(0));
                    if existing.participant_kind == atlas_local_state::ParticipantKind::Hazard {
                        update.hazard_disabled =
                            update.current_hp == Some(0) || existing.hazard_disabled;
                    } else {
                        update.defeated = update.current_hp == Some(0) || existing.defeated;
                    }
                } else {
                    let current = existing.current_hp.ok_or_else(|| {
                        AppServiceError::invalid_request("Set current HP before applying healing")
                    })?;
                    let healed = i128::from(current) + i128::from(amount);
                    let healed = existing
                        .max_hp
                        .map_or(healed, |max| healed.min(i128::from(max)));
                    update.current_hp = Some(i64::try_from(healed).map_err(|_| {
                        AppServiceError::invalid_request("Healing would overflow current HP")
                    })?);
                }
                update.hp_edit = true;
            }
            EncounterHpAction::SetCurrent { input } => {
                update.current_hp = input
                    .as_deref()
                    .map(|value| formula_value(value, true))
                    .transpose()?;
                update.hp_edit = true;
            }
            EncounterHpAction::SetMaximum { input } => {
                update.max_hp = input
                    .as_deref()
                    .map(|value| formula_value(value, true))
                    .transpose()?;
                update.max_hp_edit = true;
            }
            EncounterHpAction::SetTemporary { input } => {
                update.temporary_hp = formula_value(&input, true)?
            }
            EncounterHpAction::SetValues {
                current,
                maximum,
                temporary,
            } => {
                if current.is_none() && maximum.is_none() && temporary.is_none() {
                    return Err(AppServiceError::invalid_request(
                        "Choose at least one HP field to correct",
                    ));
                }
                // Interpret every submitted field before the single guarded replacement.
                if let Some(current) = current {
                    update.current_hp = current
                        .input
                        .as_deref()
                        .map(|input| formula_value(input, true))
                        .transpose()?;
                    update.hp_edit = true;
                }
                if let Some(maximum) = maximum {
                    update.max_hp = maximum
                        .input
                        .as_deref()
                        .map(|input| formula_value(input, true))
                        .transpose()?;
                    update.max_hp_edit = true;
                }
                if let Some(temporary) = temporary {
                    update.temporary_hp = formula_value(&temporary, true)?;
                }
            }
            EncounterHpAction::UseDerivedMaximum => update.use_derived_max = true,
        }
        // Reuse provenance/derivation policy and the local-state revision check
        // under its write transaction. Never hydrate presentation after commit.
        self.update_encounter_participant(&detail.encounter.encounter_key, update)
    }
}

// App numeric inputs must remain exactly representable by browser controls.
const MAX_INPUT_VALUE: i128 = 9_007_199_254_740_991;
const MAX_FORMULA_BYTES: usize = 512;

/// Interpret the existing integer addition/subtraction shortcut, without a
/// general expression evaluator or any dice/rules execution.
fn evaluate_formula(input: &str) -> AppServiceResult<i128> {
    if input.len() > MAX_FORMULA_BYTES {
        return Err(AppServiceError::invalid_request("HP formula is too long"));
    }
    let mut chars = input.trim().chars().peekable();
    let mut result = 0_i128;
    let mut subtract = match chars.peek() {
        Some('+') => {
            chars.next();
            false
        }
        Some('-') => {
            chars.next();
            true
        }
        _ => false,
    };
    while chars.peek().is_some_and(|c| c.is_whitespace()) {
        chars.next();
    }
    loop {
        let mut term = 0_i128;
        let mut has_digit = false;
        while let Some(digit) = chars.peek().and_then(|c| c.to_digit(10)) {
            // Only ASCII decimal input is part of the shortcut grammar.
            if !chars.peek().is_some_and(char::is_ascii_digit) {
                break;
            }
            chars.next();
            has_digit = true;
            term = term
                .checked_mul(10)
                .and_then(|value| value.checked_add(i128::from(digit)))
                .ok_or_else(|| AppServiceError::invalid_request("HP formula overflow"))?;
        }
        if !has_digit {
            return Err(AppServiceError::invalid_request(
                "Enter a whole number or an integer addition/subtraction formula",
            ));
        }
        result = if subtract {
            result.checked_sub(term)
        } else {
            result.checked_add(term)
        }
        .ok_or_else(|| AppServiceError::invalid_request("HP formula overflow"))?;
        while chars.peek().is_some_and(|c| c.is_whitespace()) {
            chars.next();
        }
        match chars.next() {
            None => return Ok(result),
            Some('+') => subtract = false,
            Some('-') => subtract = true,
            _ => return Err(AppServiceError::invalid_request("Invalid HP formula")),
        }
        while chars.peek().is_some_and(|c| c.is_whitespace()) {
            chars.next();
        }
    }
}

fn formula_value(input: &str, correction: bool) -> AppServiceResult<i64> {
    let value = evaluate_formula(input)?;
    let value = if correction { value.max(0) } else { value };
    if !(-MAX_INPUT_VALUE..=MAX_INPUT_VALUE).contains(&value) {
        return Err(AppServiceError::invalid_request(
            "HP input must evaluate to a safe integer",
        ));
    }
    Ok(value as i64)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn signed_shortcuts_preserve_correction_floor_and_safe_bounds() {
        assert_eq!(formula_value(" 23 + 4 - 9 ", false).unwrap(), 18);
        assert_eq!(formula_value("2-3", true).unwrap(), 0);
        assert_eq!(formula_value("2-3", false).unwrap(), -1);
        assert_eq!(formula_value("-10", false).unwrap(), -10);
        assert_eq!(formula_value("+10", false).unwrap(), 10);
        assert_eq!(formula_value(" - 10 + 3 ", false).unwrap(), -7);
        assert_eq!(formula_value("+2-3", false).unwrap(), -1);
        assert_eq!(formula_value("-2+3", false).unwrap(), 1);
        assert_eq!(formula_value("-10", true).unwrap(), 0);
        assert_eq!(formula_value("+100", true).unwrap(), 100);
        assert_eq!(
            formula_value("-9007199254740991", false).unwrap(),
            -9_007_199_254_740_991
        );
        assert_eq!(
            formula_value("9007199254740991+1-1", false).unwrap(),
            9_007_199_254_740_991
        );
    }

    #[test]
    fn formula_grammar_and_bounds_are_explicit() {
        for input in [
            "",
            "-",
            "+",
            "--1",
            "+-1",
            "1+-2",
            "1-+2",
            "(1)",
            "−1",
            "1.5",
            "1d6",
            "1*2",
            "1 2",
            "1--2",
            "1+",
            "NaN",
            "9007199254740992",
            "-9007199254740992",
            "-9007199254740991-1",
            "170141183460469231731687303715884105728",
        ] {
            assert!(formula_value(input, false).is_err(), "accepted {input}");
        }
        assert_eq!(formula_value(&"0".repeat(512), false).unwrap(), 0);
        assert!(formula_value(&"0".repeat(513), false).is_err());
        assert!(formula_value(&format!("{}+1-{}", i128::MAX, i128::MAX), false).is_err());
    }
}
