//! Independent persistent sources compete within a damage type; reads never roll.
pub(in crate::encounters) const STANDARD_RECOVERY_DC: i64 = 15;
use atlas_local_state::{
    ConditionParameters, DamageMultiplier, EncounterParticipantCondition, PersistentRecovery,
};
use std::collections::BTreeMap;

pub(in crate::encounters) struct PersistentGroup<'a> {
    pub damage_type: &'a str,
    pub episode_id: &'a str,
    pub sources: Vec<&'a EncounterParticipantCondition>,
    pub selected_source_id: Option<i64>,
}

pub(in crate::encounters) fn participant_groups(
    participant: &atlas_local_state::EncounterParticipant,
) -> Vec<PersistentGroup<'_>> {
    let mut result = groups(&participant.conditions);
    for group in &mut result {
        if group.selected_source_id.is_some() {
            continue;
        }
        let members: Vec<_> = group
            .sources
            .iter()
            .map(|source| source.condition_id)
            .collect();
        group.selected_source_id = participant
            .persistent_selections
            .iter()
            .find(|selection| {
                selection.episode_id == group.episode_id
                    && selection.source_condition_ids == members
                    && members.contains(&selection.selected_source_id)
            })
            .map(|selection| selection.selected_source_id);
    }
    result
}

pub(in crate::encounters) fn groups(
    conditions: &[EncounterParticipantCondition],
) -> Vec<PersistentGroup<'_>> {
    let mut groups = BTreeMap::<(&str, &str), Vec<&EncounterParticipantCondition>>::new();
    for condition in conditions {
        if let ConditionParameters::PersistentDamage {
            damage_type,
            episode_id,
            ..
        } = &condition.parameters
        {
            groups
                .entry((damage_type, episode_id))
                .or_default()
                .push(condition);
        }
    }
    groups
        .into_iter()
        .map(|((damage_type, episode_id), mut sources)| {
            sources.sort_by_key(|source| source.condition_id);
            let selected_source_id = sources
                .iter()
                .find(|candidate| {
                    sources
                        .iter()
                        .all(|other| at_least_as_much(candidate, other))
                })
                .map(|source| source.condition_id);
            PersistentGroup {
                damage_type,
                episode_id,
                sources,
                selected_source_id,
            }
        })
        .collect()
}

fn at_least_as_much(
    candidate: &EncounterParticipantCondition,
    other: &EncounterParticipantCondition,
) -> bool {
    if candidate.condition_id == other.condition_id {
        return true;
    }
    let (
        ConditionParameters::PersistentDamage {
            formula: a,
            multiplier: am,
            ..
        },
        ConditionParameters::PersistentDamage {
            formula: b,
            multiplier: bm,
            ..
        },
    ) = (&candidate.parameters, &other.parameters)
    else {
        return false;
    };
    if a == b && am == bm {
        return same_known_recovery(candidate, other);
    }
    let (Ok(a), Ok(b)) = (atlas_dice::prepare(a), atlas_dice::prepare(b)) else {
        return false;
    };
    if am == bm {
        match a.compare_magnitude(&b) {
            Some(std::cmp::Ordering::Greater) => return true,
            Some(std::cmp::Ordering::Equal) => return same_known_recovery(candidate, other),
            _ => {}
        }
    }
    let (low, high) = (scaled(a.bounds().0, *am), scaled(b.bounds().1, *bm));
    low > high || (low == high && same_known_recovery(candidate, other))
}

fn same_known_recovery(
    a: &EncounterParticipantCondition,
    b: &EncounterParticipantCondition,
) -> bool {
    recovery_dc(a)
        .zip(recovery_dc(b))
        .is_some_and(|(a, b)| a == b)
}

pub(in crate::encounters) fn scaled(amount: i64, multiplier: DamageMultiplier) -> i64 {
    let amount = amount.max(0);
    match multiplier {
        DamageMultiplier::Normal => amount,
        DamageMultiplier::Double => amount.saturating_mul(2),
        DamageMultiplier::Half => amount / 2,
    }
}

pub(in crate::encounters) fn recovery_dc(condition: &EncounterParticipantCondition) -> Option<i64> {
    match &condition.parameters {
        ConditionParameters::PersistentDamage { recovery, .. } => match recovery {
            PersistentRecovery::Standard => Some(STANDARD_RECOVERY_DC),
            PersistentRecovery::Specified { dc } => Some(*dc),
            PersistentRecovery::SourceDependent => None,
        },
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn source(id: i64, formula: &str, damage_type: &str) -> EncounterParticipantCondition {
        EncounterParticipantCondition {
            condition_id: id,
            condition_key: Some("conditionitems:lDVqvLKA6eF3Df60".into()),
            name: "Persistent Damage".into(),
            value: None,
            source_participant_key: None,
            scope: Default::default(),
            extra_action: Default::default(),
            immunity_override: false,
            parameters: ConditionParameters::PersistentDamage {
                formula: formula.into(),
                damage_type: damage_type.into(),
                multiplier: DamageMultiplier::Normal,
                recovery: PersistentRecovery::Standard,
                episode_id: format!("episode-{damage_type}"),
            },
            origin: None,
            duration: None,
            note: None,
            created_at: String::new(),
            updated_at: String::new(),
        }
    }

    #[test]
    fn retains_suppressed_sources_and_independent_types() {
        let mut sources = vec![
            source(1, "2", "fire"),
            source(2, "7", "fire"),
            source(3, "1d6", "acid"),
        ];
        let before = sources.clone();
        let result = groups(&sources);
        assert_eq!(result.len(), 2);
        let fire = result
            .iter()
            .find(|group| group.damage_type == "fire")
            .unwrap();
        assert_eq!(fire.selected_source_id, Some(2));
        assert_eq!(fire.sources.len(), 2);
        assert_eq!(sources, before);
        sources.remove(1);
        assert_eq!(
            groups(&sources)
                .iter()
                .find(|group| group.damage_type == "fire")
                .unwrap()
                .selected_source_id,
            Some(1)
        );
    }

    #[test]
    fn ambiguous_damage_requires_choice_instead_of_expected_value_sorting() {
        let sources = vec![source(1, "1d12", "fire"), source(2, "2d6", "fire")];
        assert_eq!(groups(&sources)[0].selected_source_id, None);
    }

    #[test]
    fn larger_matching_pool_is_selected_but_equal_damage_with_distinct_recovery_is_not() {
        assert_eq!(
            groups(&[source(1, "1d6", "fire"), source(2, "2d6", "fire")])[0].selected_source_id,
            Some(2)
        );
        let mut distinct = source(2, "1+1", "fire");
        if let ConditionParameters::PersistentDamage { recovery, .. } = &mut distinct.parameters {
            *recovery = PersistentRecovery::Specified { dc: 10 };
        }
        assert_eq!(
            groups(&[source(1, "2", "fire"), distinct])[0].selected_source_id,
            None
        );
    }

    #[test]
    fn equal_sources_have_order_independent_identity_without_deleting_duplicates() {
        let mut sources = vec![source(9, "1d6", "fire"), source(2, "1d6", "fire")];
        assert_eq!(groups(&sources)[0].selected_source_id, Some(2));
        sources.reverse();
        assert_eq!(groups(&sources)[0].selected_source_id, Some(2));
        assert_eq!(groups(&sources)[0].sources.len(), 2);
    }

    #[test]
    fn multiplier_and_recovery_are_source_inputs_not_note_parsing() {
        let mut a = source(1, "2", "fire");
        if let ConditionParameters::PersistentDamage {
            multiplier,
            recovery,
            ..
        } = &mut a.parameters
        {
            *multiplier = DamageMultiplier::Double;
            *recovery = PersistentRecovery::Specified { dc: 10 };
        }
        a.note = Some("DC 3".into());
        assert_eq!(recovery_dc(&a), Some(10));
        assert_eq!(
            groups(&[a, source(2, "3", "fire")])[0].selected_source_id,
            Some(1)
        );
        assert_eq!(scaled(5, DamageMultiplier::Half), 2);
    }
}
