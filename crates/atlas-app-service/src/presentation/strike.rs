//! Backend-owned ordinary strike attack progression shared by reads and roll intent.
use atlas_app_model::StrikeAttackOptionView;
#[cfg(test)]
use atlas_app_model::{ActorActivityKindView, ActorActivityView};

#[cfg(test)]
pub(crate) fn attack_options(activity: &ActorActivityView) -> Vec<StrikeAttackOptionView> {
    if activity.kind != ActorActivityKindView::Strike {
        return vec![];
    }
    project_attack_options(
        activity
            .attack
            .value
            .as_ref()
            .and_then(|value| value.as_i64()),
        Some(activity.traits.iter().any(|value| value.value == "agile")),
    )
}

pub(crate) fn project_attack_options(
    modifier: Option<i64>,
    agile: Option<bool>,
) -> Vec<StrikeAttackOptionView> {
    crate::encounters::attack_progression(modifier, agile)
        .into_iter()
        .map(|(number, modifier)| StrikeAttackOptionView {
            attack_number: number,
            label: match number {
                1 => "Attack",
                2 => "Second attack",
                _ => "Third attack",
            }
            .into(),
            modifier,
        })
        .collect()
}
