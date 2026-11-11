use crate::family_axes::{Axis, Coordinates};
use atlas_domain::RecordKey;
use std::collections::{BTreeMap, BTreeSet};

fn valid_id(id: &str) -> bool {
    id.split('-').all(|part| {
        !part.is_empty()
            && part
                .bytes()
                .all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit())
    })
}

pub(crate) struct ValidatedAxes<'a>(BTreeMap<&'a str, BTreeSet<&'a str>>);

impl<'a> ValidatedAxes<'a> {
    pub(crate) fn new(declared: &'a [Axis]) -> Result<Self, String> {
        if !declared.is_empty() && !(2..=8).contains(&declared.len()) {
            return Err("an axes family must declare 2 to 8 axes".into());
        }

        let mut options_by_axis = BTreeMap::new();
        for axis in declared {
            let options = axis
                .options
                .iter()
                .map(|option| option.id.as_str())
                .collect::<BTreeSet<_>>();
            if !valid_id(&axis.id)
                || axis.label.trim().is_empty()
                || axis
                    .alternatives_label
                    .as_ref()
                    .is_some_and(|label| label.trim().is_empty())
                || !(2..=256).contains(&options.len())
                || options.len() != axis.options.len()
                || axis
                    .options
                    .iter()
                    .any(|option| !valid_id(&option.id) || option.label.trim().is_empty())
                || options_by_axis.insert(axis.id.as_str(), options).is_some()
            {
                return Err(format!(
                    "invalid or duplicate axis or options for {}",
                    axis.id
                ));
            }
        }
        Ok(Self(options_by_axis))
    }

    pub(crate) fn contains(&self, values: &Coordinates) -> bool {
        values.iter().all(|(axis, value)| {
            self.0
                .get(axis.as_str())
                .is_some_and(|options| options.contains(value.as_str()))
        })
    }

    pub(crate) fn validate_members<'m>(
        &self,
        members: impl IntoIterator<Item = (&'m RecordKey, &'m Coordinates)>,
    ) -> Result<(), String> {
        let mut combinations = BTreeSet::new();
        for (key, values) in members {
            if values.len() != self.0.len()
                || !self.contains(values)
                || (!self.0.is_empty() && !combinations.insert(values))
            {
                return Err(format!("invalid or duplicate axis coordinates for {key}"));
            }
        }
        Ok(())
    }
}
