use super::*;

pub(super) fn refresh(body: &mut RecordBodyView) {
    if let RecordBodyView::Hazard(hazard) = body {
        hazard.detection_dc = derived(&hazard.stealth, |value| value.checked_add(10));
        hazard.broken_threshold = if hazard.actor.maximum_hp.value == Some(0.into()) {
            number(SourceFieldView::NotApplicable)
        } else {
            derived(&hazard.actor.maximum_hp, |value| {
                (value > 0).then_some(value / 2)
            })
        };
    }
}

fn derived(source: &NumberFactView, calculate: impl FnOnce(i64) -> Option<i64>) -> NumberFactView {
    let mut result = NumberFactView {
        state: source.state,
        value: None,
        adjustment: None,
    };
    if source.state == QueryFieldState::Value {
        result.value = source
            .value
            .as_ref()
            .and_then(serde_json::Number::as_i64)
            .filter(|value| value.unsigned_abs() <= 9_007_199_254_740_991)
            .and_then(calculate)
            .filter(|value| value.unsigned_abs() <= 9_007_199_254_740_991)
            .map(Into::into);
        if result.value.is_none() {
            result.state = QueryFieldState::Invalid;
        }
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn derived_readouts_preserve_zero_and_unknown_without_overflow() {
        let source = |value| NumberFactView {
            state: QueryFieldState::Value,
            value: Some(value),
            adjustment: None,
        };
        assert_eq!(
            derived(&source((-1).into()), |n| n.checked_add(10)).value,
            Some(9.into())
        );
        assert_eq!(
            derived(&source(25.into()), |n| (n > 0).then_some(n / 2)).value,
            Some(12.into())
        );
        assert_eq!(
            derived(&source(i64::MAX.into()), |n| n.checked_add(10)).state,
            QueryFieldState::Invalid
        );
        let missing = number(SourceFieldView::Missing);
        assert_eq!(derived(&missing, |n| Some(n / 2)), missing);
    }
}
