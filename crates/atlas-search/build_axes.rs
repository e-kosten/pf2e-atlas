use std::collections::BTreeSet;

use super::catalog::Family;

pub(super) fn validate(family: &Family) -> Result<(), String> {
    let axes = crate::family_axis_validation::ValidatedAxes::new(&family.axes)?;
    axes.validate_members(
        family
            .members
            .iter()
            .map(|member| (&member.key, &member.coordinates)),
    )?;
    if !family.axes.is_empty()
        && let Some(member) = family
            .members
            .iter()
            .find(|member| !member.attributes.is_empty() || member.fit.is_some())
    {
        return Err(format!("invalid axis coordinates for {}", member.key));
    }

    let mut covered = BTreeSet::new();
    for general in &family.general_records {
        if general.key.pack().as_str() != "equipment-srd"
            || general.name.trim().is_empty()
            || (!general.scope.is_empty() && general.scope.len() >= family.axes.len())
            || !axes.contains(&general.scope)
        {
            return Err(format!("invalid general record scope for {}", general.key));
        }
        let mut count = 0;
        for member in family
            .members
            .iter()
            .filter(|member| general.covers(member))
        {
            count += 1;
            if !covered.insert(&member.key) {
                return Err(format!(
                    "overlapping general record scopes cover {}",
                    member.key
                ));
            }
        }
        if count == 0 {
            return Err(format!(
                "general record {} covers no concrete members",
                general.key
            ));
        }
    }
    Ok(())
}
