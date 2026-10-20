//! Typed PF2e modifier competition, independent of presentation contracts.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum ModifierType {
    Status,
    Circumstance,
    Adjustment,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct Modifier {
    pub source: String,
    pub label: String,
    pub modifier_type: ModifierType,
    pub value: i64,
}

/// Positive and negative modifiers compete independently; adjustments always stack.
pub(super) fn competes(kind: ModifierType, old: i64, candidate: i64) -> bool {
    kind != ModifierType::Adjustment && old.signum() == candidate.signum()
}

pub(super) fn stronger(candidate: i64, old: i64) -> bool {
    candidate.unsigned_abs() > old.unsigned_abs()
}

/// Multiple attack progression consumes established source facts only.
pub(crate) fn attack_progression(modifier: Option<i64>, agile: Option<bool>) -> Vec<(u8, i32)> {
    let Some(modifier) = modifier.and_then(|value| i32::try_from(value).ok()) else {
        return vec![];
    };
    let penalty = agile.map(|agile| if agile { 4 } else { 5 });
    (1..=3)
        .filter_map(|number| {
            let value = if number == 1 {
                modifier
            } else {
                modifier.checked_sub(penalty? * i32::from(number - 1))?
            };
            Some((number, value))
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn bonus_and_penalty_do_not_suppress_one_another() {
        assert!(!competes(ModifierType::Status, 2, -3));
        assert!(competes(ModifierType::Circumstance, -2, -3));
        assert!(!competes(ModifierType::Adjustment, 2, 3));
    }
    #[test]
    fn minimum_integer_is_a_valid_penalty() {
        assert!(stronger(i64::MIN, -1));
    }
}
