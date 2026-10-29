//! Bounded state declarations from checked owned abilities, never description parsing.
use super::super::conditions::ConditionRule;
use crate::AppServiceResult;
use atlas_domain::OwnedContentLocator;
use atlas_foundry_model::{RuleSource, SourceContext, SourcePresence, generated::*};
use atlas_local_state::{AbilityStateValue, EncounterParticipant};
use atlas_record::source_record::ItemSourceView;

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct SourceOption {
    pub declaration_identity: String,
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
    pub declaration_identity: String,
    pub owner: OwnedContentLocator,
    pub label: String,
    pub maximum: i64,
    pub initial: i64,
    pub per: AbilitySystemSourceFrequencyPer,
}
impl SourceOption {
    pub fn state(&self, participant: &EncounterParticipant) -> (bool, Option<String>) {
        participant
            .ability_states
            .iter()
            .find(|state| {
                state.owner == self.owner
                    && state.rule_ordinal == Some(self.ordinal)
                    && state.declaration_identity.as_deref()
                        == Some(self.declaration_identity.as_str())
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
    pub fn remaining(&self, participant: &EncounterParticipant) -> i64 {
        participant
            .ability_states
            .iter()
            .find(|state| {
                state.owner == self.owner
                    && state.rule_ordinal.is_none()
                    && state.declaration_identity.as_deref()
                        == Some(self.declaration_identity.as_str())
            })
            .and_then(|state| match state.value {
                AbilityStateValue::Uses { remaining } => Some(remaining),
                _ => None,
            })
            .unwrap_or(self.initial)
    }
}

/// Identity covers supported behavior, independently of navigation generation and defaults.
pub(super) fn bind_declarations(
    root: &str,
    options: &mut [SourceOption],
    uses: &mut [SourceUses],
    grants: &[super::source_conditions::SourceCondition],
) -> AppServiceResult<()> {
    for option in options {
        let linked = grants
            .iter()
            .filter(|grant| grant.owner == option.owner)
            .map(|grant| {
                (
                    grant.grant_ordinal,
                    grant.rule.definition().slug,
                    grant.value,
                    &grant.predicate,
                    grant
                        .choice
                        .as_ref()
                        .map(|choice| (choice.option_ordinal, choice.value.as_str())),
                )
            })
            .collect::<Vec<_>>();
        let choices = option
            .choices
            .iter()
            .map(|choice| (choice.value.as_str(), choice.rule.definition().slug))
            .collect::<Vec<_>>();
        option.declaration_identity = declaration_identity(&(
            "option",
            root,
            &option.owner,
            option.ordinal,
            &option.option,
            option.always_active,
            &option.predicate,
            choices,
            linked,
        ))?;
    }
    for ability in uses {
        ability.declaration_identity =
            declaration_identity(&("uses", root, &ability.owner, ability.maximum, ability.per))?;
    }
    Ok(())
}

fn declaration_identity(recipe: &impl serde::Serialize) -> AppServiceResult<String> {
    use sha2::{Digest, Sha256};
    let bytes = serde_json::to_vec(recipe).map_err(|error| {
        crate::AppServiceError::new(
            atlas_app_model::AppErrorCode::InternalError,
            format!("Could not identify the ability declaration: {error}"),
        )
    })?;
    Ok(format!(
        "ability-declaration/v1:{:x}",
        Sha256::digest(bytes)
    ))
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
            declaration_identity: String::new(),
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
        declaration_identity: String::new(),
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
    fn declaration_fixture() -> Value {
        json!({"_id":"aaaaaaaaaaaaaaaa","type":"npc","name":"Original",
        "system":{"traits":{"value":[]},"attributes":{"immunities":[]}},
        "items":[{"_id":"bbbbbbbbbbbbbbbb","type":"action","name":"Ability",
            "system":{"actionType":{"value":"passive"},"frequency":{"max":3,"value":2,"per":"day"},"rules":[
                {"key":"RollOption","option":"ability-active","toggleable":true,"value":false},
                {"key":"GrantItem","inMemoryOnly":true,"uuid":"fixture:slowed","predicate":["ability-active"]}
            ]}}]})
    }
    fn prepared(
        value: Value,
        fingerprint: &str,
    ) -> super::super::source_conditions::SourceConditions {
        let record = atlas_search::test_support::record("actors", "Actor", value).record;
        let mut sources =
            source_conditions::source_conditions(&record, |_| Ok(Some(ConditionRule::Slowed)))
                .unwrap();
        sources.source_fingerprint = Some(fingerprint.into());
        sources
    }
    fn saved(sources: &super::super::source_conditions::SourceConditions) -> EncounterParticipant {
        let mut participant = super::super::numerical_tests::participant();
        let option = &sources.options[0];
        let ability = &sources.uses[0];
        participant.ability_states = vec![
            atlas_local_state::ParticipantAbilityState {
                owner: option.owner.clone(),
                rule_ordinal: Some(option.ordinal),
                source_fingerprint: sources.source_fingerprint.clone().unwrap(),
                declaration_identity: Some(option.declaration_identity.clone()),
                value: AbilityStateValue::Option {
                    active: true,
                    selection: None,
                },
            },
            atlas_local_state::ParticipantAbilityState {
                owner: ability.owner.clone(),
                rule_ordinal: None,
                source_fingerprint: sources.source_fingerprint.clone().unwrap(),
                declaration_identity: Some(ability.declaration_identity.clone()),
                value: AbilityStateValue::Uses { remaining: 0 },
            },
        ];
        participant
    }
    #[test]
    fn unrelated_source_and_label_changes_preserve_bound_options_and_expenditure() {
        let original = prepared(declaration_fixture(), "old-generation");
        let participant = saved(&original);
        let mut updated = declaration_fixture();
        updated["name"] = json!("Translated creature");
        updated["system"]["details"] = json!({"description":"Unrelated correction"});
        updated["items"][0]["name"] = json!("Translated ability");
        updated["items"][0]["system"]["rules"][0]["label"] = json!("Translated switch");
        updated["items"][0]["system"]["frequency"]["value"] = json!(3);
        let current = prepared(updated, "new-generation");
        assert_eq!(current.options[0].state(&participant), (true, None));
        assert_eq!(current.uses[0].remaining(&participant), 0);
        assert!(
            super::super::causes::condition_causes(&participant, None, &current)
                .known_presence(ConditionRule::Slowed)
                .unwrap()
        );
    }
    #[test]
    fn changed_grant_capacity_and_interval_cannot_inherit_old_state() {
        let original = prepared(declaration_fixture(), "old");
        let participant = saved(&original);
        let mut changed = declaration_fixture();
        changed["items"][0]["system"]["rules"][1]["alterations"] =
            json!([{"mode":"override","property":"badge-value","value":2}]);
        changed["items"][0]["system"]["frequency"]["max"] = json!(4);
        changed["items"][0]["system"]["frequency"]["value"] = json!(4);
        let current = prepared(changed, "new");
        assert_eq!(current.options[0].state(&participant), (false, None));
        assert_eq!(current.uses[0].remaining(&participant), 4);
        let mut interval = declaration_fixture();
        interval["items"][0]["system"]["frequency"]["per"] = json!("round");
        assert_eq!(prepared(interval, "new").uses[0].remaining(&participant), 2);
    }
    #[test]
    fn legacy_and_mismatched_bindings_use_authored_defaults_without_confirmation() {
        let sources = prepared(declaration_fixture(), "current");
        let mut participant = saved(&sources);
        participant.ability_states[0].declaration_identity = None;
        participant.ability_states[1].declaration_identity =
            Some("ability-declaration/v0:obsolete".into());
        assert_eq!(sources.options[0].state(&participant), (false, None));
        assert_eq!(sources.uses[0].remaining(&participant), 2);
    }
    #[test]
    fn authored_default_changes_do_not_replace_an_explicit_saved_choice() {
        let original = prepared(declaration_fixture(), "old");
        let mut participant = saved(&original);
        participant.ability_states[0].value = AbilityStateValue::Option {
            active: false,
            selection: None,
        };
        let mut changed = declaration_fixture();
        changed["items"][0]["system"]["rules"][0]["value"] = json!(true);
        assert_eq!(
            prepared(changed, "new").options[0].state(&participant),
            (false, None)
        );
    }
    #[test]
    fn changed_choice_membership_resets_selection_but_labels_do_not() {
        let mut original = declaration_fixture();
        original["items"][0]["system"]["rules"][0]["suboptions"] = json!([
            {"label":"First","value":"fixture:first"},{"label":"Second","value":"fixture:second"}
        ]);
        original["items"][0]["system"]["rules"][1]["uuid"] =
            json!("{item|flags.system.rulesSelections.abilityActive}");
        let sources = prepared(original.clone(), "old");
        let mut participant = saved(&sources);
        participant.ability_states[0].value = AbilityStateValue::Option {
            active: true,
            selection: Some("fixture:second".into()),
        };
        let mut renamed = original.clone();
        renamed["items"][0]["system"]["rules"][0]["suboptions"][1]["label"] =
            json!("Translated second choice");
        assert_eq!(
            prepared(renamed, "new").options[0].state(&participant),
            (true, Some("fixture:second".into()))
        );
        original["items"][0]["system"]["rules"][0]["suboptions"] =
            json!([{"label":"First","value":"fixture:first"}]);
        assert_eq!(
            prepared(original, "new").options[0].state(&participant),
            (false, Some("fixture:first".into()))
        );
    }
    #[test]
    fn moved_or_removed_option_slots_do_not_rebind_saved_switches() {
        let original = declaration_fixture();
        let sources = prepared(original.clone(), "old");
        let participant = saved(&sources);
        let mut moved = original.clone();
        moved["items"][0]["system"]["rules"]
            .as_array_mut()
            .unwrap()
            .insert(0, json!({"key":"UnmodeledRule"}));
        let current = prepared(moved, "new");
        assert_eq!(current.options[0].ordinal, 1);
        assert_eq!(current.options[0].state(&participant), (false, None));
        assert_eq!(current.uses[0].remaining(&participant), 0);
        let mut removed = original;
        removed["items"][0]["system"]["rules"] = json!([]);
        assert!(prepared(removed, "new").options.is_empty());
    }
}
