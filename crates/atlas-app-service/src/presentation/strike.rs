//! Backend-owned ordinary strike attack progression shared by reads and roll intent.
use atlas_app_model::{ActorActivityKindView, ActorActivityView, StrikeAttackOptionView};

pub(crate) fn attack_options(activity: &ActorActivityView) -> Vec<StrikeAttackOptionView> {
    if activity.kind != ActorActivityKindView::Strike {
        return vec![];
    }
    let Some(modifier) = activity
        .attack
        .value
        .as_ref()
        .and_then(|n| n.as_i64())
        .and_then(|v| i32::try_from(v).ok())
    else {
        return vec![];
    };
    let penalty = if activity.traits.iter().any(|t| t.value == "agile") {
        4
    } else {
        5
    };
    (1..=3)
        .filter_map(|number| {
            Some(StrikeAttackOptionView {
                attack_number: number,
                label: match number {
                    1 => "Attack",
                    2 => "Second attack",
                    _ => "Third attack",
                }
                .into(),
                modifier: modifier.checked_sub(penalty * i32::from(number - 1))?,
            })
        })
        .collect()
}
