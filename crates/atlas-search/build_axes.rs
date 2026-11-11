use std::collections::{BTreeMap, BTreeSet};

use super::catalog::Family;

fn valid_id(id: &str) -> bool {
    id.split('-').all(|part| {
        !part.is_empty()
            && part
                .bytes()
                .all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit())
    })
}

pub(super) fn validate(family: &Family) -> Result<(), String> {
    if !family.axes.is_empty() && !(2..=8).contains(&family.axes.len()) {
        return Err("an axes family must declare 2 to 8 axes".into());
    }

    let mut axes = BTreeMap::new();
    for axis in &family.axes {
        let options = axis
            .options
            .iter()
            .map(|option| option.id.as_str())
            .collect::<BTreeSet<_>>();
        if !valid_id(&axis.id)
            || axis.label.trim().is_empty()
            || !(2..=256).contains(&options.len())
            || options.len() != axis.options.len()
            || axis
                .options
                .iter()
                .any(|option| !valid_id(&option.id) || option.label.trim().is_empty())
            || axes.insert(axis.id.as_str(), options).is_some()
        {
            return Err(format!(
                "invalid or duplicate axis or options for {}",
                axis.id
            ));
        }
    }
    let valid_values = |values: &BTreeMap<String, String>| {
        values.iter().all(|(axis, value)| {
            axes.get(axis.as_str())
                .is_some_and(|options| options.contains(value.as_str()))
        })
    };

    let mut combinations = BTreeSet::new();
    for member in &family.members {
        let invalid = if axes.is_empty() {
            !member.coordinates.is_empty()
        } else {
            member.coordinates.len() != axes.len()
                || !valid_values(&member.coordinates)
                || !member.attributes.is_empty()
                || member.fit.is_some()
                || !combinations.insert(&member.coordinates)
        };
        if invalid {
            return Err(format!(
                "invalid or duplicate axis coordinates for {}",
                member.key
            ));
        }
    }

    let mut covered = BTreeSet::new();
    for general in &family.general_records {
        if general.key.pack().as_str() != "equipment-srd"
            || general.name.trim().is_empty()
            || (!general.scope.is_empty() && general.scope.len() >= axes.len())
            || !valid_values(&general.scope)
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
