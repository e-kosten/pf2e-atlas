//! Bounded state declarations from checked owned abilities, never description parsing.
use super::super::conditions::ConditionRule;
use crate::AppServiceResult;
use atlas_domain::OwnedContentLocator;
use atlas_foundry_model::{RuleSource, SourceContext, SourcePresence, generated::*};
use atlas_local_state::{AbilityStateValue, EncounterParticipant};
use atlas_record::source_record::ItemSourceView;

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct SourceOption {
    pub owner: OwnedContentLocator,
    pub ordinal: usize,
    pub label: String,
    pub authored_label: Option<String>,
    pub option: String,
    pub always_active: bool,
    pub initial_active: bool,
    pub initial_selection: Option<String>,
    pub choices: Vec<SourceOptionChoice>,
    pub predicate: SourcePresence<Vec<PredicateStatement>>,
}
#[derive(Debug, Clone, PartialEq)]
pub(crate) struct SourceOptionChoice {
    pub value: String,
    pub rule: ConditionRule,
}
#[derive(Debug, Clone, PartialEq)]
pub(crate) struct SourceUses {
    pub owner: OwnedContentLocator,
    pub label: String,
    pub maximum: i64,
    pub initial: i64,
    pub per: AbilitySystemSourceFrequencyPer,
}
impl SourceOption {
    pub fn state(
        &self,
        participant: &EncounterParticipant,
        fingerprint: Option<&str>,
    ) -> (bool, Option<String>) {
        participant
            .ability_states
            .iter()
            .find(|state| {
                state.owner == self.owner
                    && state.rule_ordinal == Some(self.ordinal)
                    && Some(state.source_fingerprint.as_str()) == fingerprint
            })
            .and_then(|state| match &state.value {
                AbilityStateValue::Option { active, selection } => {
                    Some((self.always_active || *active, selection.clone()))
                }
                _ => None,
            })
            .unwrap_or_else(|| {
                (
                    self.always_active || self.initial_active,
                    self.initial_selection.clone(),
                )
            })
    }
    pub fn flag(&self) -> String {
        let mut parts = self.option.split('-');
        let mut flag = parts.next().unwrap_or_default().to_owned();
        for part in parts {
            let mut chars = part.chars();
            if let Some(first) = chars.next() {
                flag.extend(first.to_uppercase());
                flag.extend(chars);
            }
        }
        format!("{{item|flags.system.rulesSelections.{flag}}}")
    }
}
impl SourceUses {
    pub fn remaining(&self, participant: &EncounterParticipant, fingerprint: Option<&str>) -> i64 {
        participant
            .ability_states
            .iter()
            .find(|state| {
                state.owner == self.owner
                    && state.rule_ordinal.is_none()
                    && Some(state.source_fingerprint.as_str()) == fingerprint
            })
            .and_then(|state| match state.value {
                AbilityStateValue::Uses { remaining } => Some(remaining),
                _ => None,
            })
            .unwrap_or(self.initial)
    }
}

pub(super) fn options(
    item: ItemSourceView<'_>,
    owner: &OwnedContentLocator,
    label: &str,
    context: &SourceContext,
    resolve: &mut impl FnMut(&str) -> AppServiceResult<Option<ConditionRule>>,
) -> AppServiceResult<Vec<SourceOption>> {
    let mut result = vec![];
    if !matches!(
        owner.identity,
        atlas_domain::OwnedContentIdentity::Stable(_)
    ) {
        return Ok(result);
    }
    for (ordinal, retained) in item.rules().value().unwrap_or_default().iter().enumerate() {
        if retained.key.as_value().map(String::as_str) != Some("RollOption") {
            continue;
        }
        let Ok(RuleSource::RollOption(rule)) = retained.interpret(context.clone()) else {
            continue;
        };
        if rule.ignored == SourcePresence::Value(true)
            || rule.toggleable.as_value()
                != Some(&SourceFromSchemaRollOptionSchemaToggleable::Boolean(true))
            || rule.requires_equipped == SourcePresence::Value(true)
            || rule.requires_investment == SourcePresence::Value(true)
            || rule
                .disabled_if
                .as_value()
                .is_some_and(|terms| !terms.is_empty())
            || rule.count == SourcePresence::Value(true)
            || rule.remove_after_roll == SourcePresence::Value(true)
            || rule.mergeable == SourcePresence::Value(true)
            || !rule.additional_fields.fields().is_empty()
        {
            continue;
        }
        let Some(option) = rule
            .option
            .as_value()
            .filter(|option| !option.contains(['{', '|', '}']))
        else {
            continue;
        };
        let initial_active = match &rule.value {
            SourcePresence::Missing => false,
            SourcePresence::Value(JSONValue::Boolean(value)) => *value,
            _ => continue,
        };
        let mut choices = vec![];
        match &rule.suboptions {
            SourcePresence::Missing => {}
            SourcePresence::Value(SourceFromSchemaRollOptionSchemaSuboptions::Array(values)) => {
                let mut complete = true;
                for choice in values {
                    if !choice.additional_fields.fields().is_empty()
                        || choice.predicate.as_value().is_some_and(|p| !p.is_empty())
                    {
                        complete = false;
                        break;
                    }
                    let Some(value) = choice.value.as_value() else {
                        complete = false;
                        break;
                    };
                    let Some(condition) = resolve(value)? else {
                        complete = false;
                        break;
                    };
                    choices.push(SourceOptionChoice {
                        value: value.clone(),
                        rule: condition,
                    });
                }
                if !complete {
                    continue;
                }
            }
            _ => continue,
        }
        let initial_selection = rule
            .selection
            .as_value()
            .filter(|value| choices.iter().any(|choice| &choice.value == *value))
            .cloned()
            .or_else(|| choices.first().map(|choice| choice.value.clone()));
        result.push(SourceOption {
            owner: owner.clone(),
            ordinal,
            label: label.into(),
            authored_label: rule.label.as_value().cloned(),
            option: option.clone(),
            always_active: rule.always_active == SourcePresence::Value(true),
            initial_active,
            initial_selection,
            choices,
            predicate: rule.predicate,
        });
    }
    Ok(result)
}

pub(super) fn uses(
    item: ItemSourceView<'_>,
    owner: &OwnedContentLocator,
    label: &str,
) -> Option<SourceUses> {
    if !matches!(
        owner.identity,
        atlas_domain::OwnedContentIdentity::Stable(_)
    ) {
        return None;
    }
    let frequency = item.activity_frequency().value()?;
    let maximum = frequency
        .max
        .as_value()?
        .as_i64()
        .filter(|value| *value > 0)?;
    let initial = match &frequency.value {
        SourcePresence::Missing => maximum,
        SourcePresence::Value(value) => value
            .as_i64()
            .filter(|value| (0..=maximum).contains(value))?,
        _ => return None,
    };
    Some(SourceUses {
        owner: owner.clone(),
        label: label.into(),
        maximum,
        initial,
        per: *frequency.per.as_value()?,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::encounters::{ability_state, rules::source_conditions};
    use atlas_record::source_record::SourceBackedRecord;
    use serde_json::{Value, json};

    fn views(labels: &[&str]) -> Vec<atlas_app_model::EncounterAbilityStateView> {
        let mut rules: Vec<Value> = labels
            .iter()
            .enumerate()
            .map(|(ordinal, label)| {
                json!({"key":"RollOption","label":label,"option":format!("star-{ordinal}"),"toggleable":true})
            })
            .collect();
        rules.push(json!({"key":"GrantItem","inMemoryOnly":true,
            "uuid":"Compendium.pf2e.conditionitems.Item.Slowed",
            "predicate":(0..labels.len()).map(|ordinal|format!("star-{ordinal}")).collect::<Vec<_>>() }));
        let source = atlas_foundry_model::admit_document_source(
            "Actor",
            SourceContext::new("test", "fixture", "$"),
            &serde_json::to_vec(&json!({"_id":"aaaaaaaaaaaaaaaa","type":"npc",
                "system":{"traits":{"value":[]},"attributes":{"immunities":[]}},
                "items":[{"_id":"bbbbbbbbbbbbbbbb","name":"Aeon Stars","type":"action",
                    "system":{"actionType":{"value":"passive"},"rules":rules}}]}))
            .unwrap(),
        )
        .unwrap()
        .model
        .unwrap();
        let record = SourceBackedRecord::new("test", source).unwrap();
        let mut sources = source_conditions::source_conditions(&record, |uuid| {
            Ok((uuid == "Compendium.pf2e.conditionitems.Item.Slowed")
                .then_some(ConditionRule::Slowed))
        })
        .unwrap();
        sources.source_fingerprint = Some("test-source".into());
        let mut participant = super::super::numerical_tests::participant();
        participant.record_key = Some(record.key().to_string());
        ability_state::views(&participant, &sources, Some(5))
    }

    #[test]
    fn multiple_switches_project_authored_labels_with_exact_owner_and_ordinals() {
        let abilities = views(&[
            "PF2E.NPCAbility.SanctumKnight.AeonStar.Black.RollOptionLabel",
            "PF2E.NPCAbility.SanctumKnight.AeonStar.Gold.RollOptionLabel",
            "PF2E.NPCAbility.SanctumKnight.AeonStar.Green.RollOptionLabel",
            "PF2E.NPCAbility.SanctumKnight.AeonStar.White.RollOptionLabel",
        ]);
        assert_eq!(
            abilities
                .iter()
                .map(|ability| (ability.label.as_str(), ability.rule_ordinal))
                .collect::<Vec<_>>(),
            [
                ("Black Aeon Star used", Some(0)),
                ("Gold Aeon Star used", Some(1)),
                ("Green Aeon Star used", Some(2)),
                ("White Aeon Star used", Some(3)),
            ]
        );
        let expected_owner = OwnedContentLocator {
            collection: "/items".into(),
            identity: atlas_domain::OwnedContentIdentity::Stable("bbbbbbbbbbbbbbbb".into()),
        };
        assert!(abilities.iter().all(|ability| {
            ability.navigation.record_key == "test:aaaaaaaaaaaaaaaa"
                && ability.navigation.owners == [expected_owner.clone()]
                && ability.navigation.source_fingerprint.as_deref() == Some("test-source")
        }));
    }

    #[test]
    fn multiple_switches_retain_literal_authored_labels() {
        assert_eq!(
            views(&["First star used", "Second star used"])
                .iter()
                .map(|ability| ability.label.as_str())
                .collect::<Vec<_>>(),
            ["First star used", "Second star used"]
        );
    }

    #[test]
    fn single_switch_keeps_its_owning_ability_label() {
        assert_eq!(views(&["One star used"])[0].label, "Aeon Stars");
    }
}
