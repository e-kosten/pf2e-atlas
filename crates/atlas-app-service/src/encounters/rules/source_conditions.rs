//! Source-backed condition declarations; activation never comes from authored prose.
use super::super::conditions::ConditionRule;
use crate::AppServiceResult;
use atlas_domain::OwnedContentLocator;
use atlas_foundry_model::{RuleSource, SourceContext, SourcePresence, generated::*};
use atlas_record::source_record::{ItemSourceView, SourceBackedRecord, SourceFieldView};

#[derive(Debug, Default, Clone, PartialEq)]
pub(crate) struct SourceConditions {
    pub options: Vec<super::source_ability_state::SourceOption>,
    pub uses: Vec<super::source_ability_state::SourceUses>,
    pub entries: Vec<SourceCondition>,
    pub unavailable_reason: Option<String>,
    pub source_fingerprint: Option<String>,
}

impl SourceConditions {
    pub fn unresolved_entries<'a>(
        &'a self,
        participant: &'a atlas_local_state::EncounterParticipant,
    ) -> Vec<&'a SourceCondition> {
        let rules = super::causes::condition_causes(participant, None, self)
            .iter()
            .map(|cause| cause.rule)
            .collect::<Vec<_>>();
        self.entries
            .iter()
            .filter(|source| source.active(&rules, self, participant).is_none())
            .collect()
    }
    pub fn active_entries<'a>(
        &'a self,
        participant: &'a atlas_local_state::EncounterParticipant,
    ) -> Vec<&'a SourceCondition> {
        super::causes::condition_causes(participant, None, self)
            .into_iter()
            .filter_map(|cause| match cause.source {
                super::causes::ConditionSource::Authored(source, ..)
                    if cause.rule == source.rule =>
                {
                    Some(source)
                }
                _ => None,
            })
            .collect()
    }

    pub fn empty() -> &'static Self {
        static EMPTY: SourceConditions = SourceConditions {
            entries: vec![],
            options: vec![],
            uses: vec![],
            unavailable_reason: None,
            source_fingerprint: None,
        };
        &EMPTY
    }
    pub fn unavailable() -> Self {
        Self {
            unavailable_reason: Some("This creature's source conditions are unavailable.".into()),
            ..Default::default()
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct SourceChoiceActivation {
    pub option_ordinal: usize,
    pub value: String,
}
#[derive(Debug, Clone, PartialEq)]
pub(crate) struct SourceCondition {
    pub record_key: String,
    pub owner: OwnedContentLocator,
    pub grant_ordinal: Option<usize>,
    pub source_label: String,
    pub rule: ConditionRule,
    pub value: Option<i64>,
    pub predicate: SourcePresence<Vec<PredicateStatement>>,
    pub choice: Option<SourceChoiceActivation>,
    pub activation_unavailable: Option<String>,
}

/// The resolver uses the existing reference identity policy, including qualified
/// authored aliases. A display label is never a condition identity.
pub(in crate::encounters) fn source_conditions(
    record: &SourceBackedRecord,
    mut resolve: impl FnMut(&str) -> AppServiceResult<Option<ConditionRule>>,
) -> AppServiceResult<SourceConditions> {
    let mut items = vec![];
    let availability = record.visit_immediate_actor_items(|index, owner, item| {
        items.push((index, owner.clone(), item));
    });
    if matches!(
        availability,
        SourceFieldView::Null | SourceFieldView::Invalid(_)
    ) {
        return Ok(SourceConditions::unavailable());
    }
    let mut conditions = vec![];
    let mut options = vec![];
    let mut uses = vec![];
    let immunities = super::source_facts::source_applicability(record, false).immunities;
    let unavailable_reason = None;
    for (index, owner, item) in items {
        let label = atlas_record::source_record::SourceNodeView::Item(item)
            .name()
            .value()
            .cloned()
            .unwrap_or_default();
        let option_context = SourceContext::new(
            record.key().to_string(),
            "checked snapshot",
            format!("$.items[{index}].system.rules"),
        );
        let item_options = super::source_ability_state::options(
            item,
            &owner,
            &label,
            &option_context,
            &mut resolve,
        )?;
        options.extend(item_options.clone());
        if let Some(frequency) = super::source_ability_state::uses(item, &owner, &label) {
            uses.push(frequency);
        }
        if let ItemSourceView::ConditionSource(condition) = item {
            let system = SourceFieldView::from(&condition.system);
            let Some(slug) = system.and_then(|s| (&s.slug).into()).value() else {
                continue;
            };
            use SourceFromSchemaAfflictionConditionSchemaSlug as Slug;
            let rule = match slug {
                Slug::Blinded => ConditionRule::Blinded,
                Slug::Broken => ConditionRule::Broken,
                Slug::Clumsy => ConditionRule::Clumsy,
                Slug::Concealed => ConditionRule::Concealed,
                Slug::Confused => ConditionRule::Confused,
                Slug::Controlled => ConditionRule::Controlled,
                Slug::Cursebound => ConditionRule::Cursebound,
                Slug::Dazzled => ConditionRule::Dazzled,
                Slug::Deafened => ConditionRule::Deafened,
                Slug::Doomed => ConditionRule::Doomed,
                Slug::Drained => ConditionRule::Drained,
                Slug::Dying => ConditionRule::Dying,
                Slug::Encumbered => ConditionRule::Encumbered,
                Slug::Enfeebled => ConditionRule::Enfeebled,
                Slug::Fascinated => ConditionRule::Fascinated,
                Slug::Fatigued => ConditionRule::Fatigued,
                Slug::Fleeing => ConditionRule::Fleeing,
                Slug::Friendly => ConditionRule::Friendly,
                Slug::Frightened => ConditionRule::Frightened,
                Slug::Grabbed => ConditionRule::Grabbed,
                Slug::Helpful => ConditionRule::Helpful,
                Slug::Hidden => ConditionRule::Hidden,
                Slug::Hostile => ConditionRule::Hostile,
                Slug::Immobilized => ConditionRule::Immobilized,
                Slug::Indifferent => ConditionRule::Indifferent,
                Slug::Invisible => ConditionRule::Invisible,
                Slug::Observed => ConditionRule::Observed,
                Slug::OffGuard => ConditionRule::OffGuard,
                Slug::Paralyzed => ConditionRule::Paralyzed,
                Slug::PersistentDamage => ConditionRule::PersistentDamage,
                Slug::Petrified => ConditionRule::Petrified,
                Slug::Prone => ConditionRule::Prone,
                Slug::Quickened => ConditionRule::Quickened,
                Slug::Restrained => ConditionRule::Restrained,
                Slug::Sickened => ConditionRule::Sickened,
                Slug::Slowed => ConditionRule::Slowed,
                Slug::Stunned => ConditionRule::Stunned,
                Slug::Stupefied => ConditionRule::Stupefied,
                Slug::Unconscious => ConditionRule::Unconscious,
                Slug::Undetected => ConditionRule::Undetected,
                Slug::Unfriendly => ConditionRule::Unfriendly,
                Slug::Unnoticed => ConditionRule::Unnoticed,
                Slug::Wounded => ConditionRule::Wounded,
                Slug::Malevolence => continue,
            };
            let value = if rule.definition().has_value {
                system
                    .and_then(|s| (&s.value).into())
                    .and_then(|value| {
                        if value.is_valued == SourcePresence::Value(true) {
                            SourceFieldView::Value(value)
                        } else {
                            SourceFieldView::Missing
                        }
                    })
                    .and_then(|v| (&v.value).into())
                    .value()
                    .and_then(serde_json::Number::as_i64)
                    .filter(|value| *value >= 0)
            } else {
                None
            };
            conditions.push(SourceCondition {
                record_key: record.key().to_string(),
                owner,
                grant_ordinal: None,
                source_label: label,
                rule,
                value,
                predicate: SourcePresence::Missing,
                choice: None,
                activation_unavailable: None,
            });
            continue;
        }
        for (ordinal, retained) in item.rules().value().unwrap_or_default().iter().enumerate() {
            if retained.key.as_value().map(String::as_str) != Some("GrantItem") {
                continue;
            }
            let context = SourceContext::new(
                record.key().to_string(),
                "checked snapshot",
                format!("$.items[{index}].system.rules[{ordinal}]"),
            );
            let Ok(RuleSource::GrantItem(grant)) = retained.interpret(context) else {
                continue;
            };
            if grant.ignored == SourcePresence::Value(true) {
                continue;
            }
            let Some(uuid) = grant.uuid.as_value() else {
                continue;
            };
            let targets = if uuid.starts_with("{item|flags.system.rulesSelections.") {
                item_options
                    .iter()
                    .find(|option| option.flag() == *uuid)
                    .map(|option| {
                        option
                            .choices
                            .iter()
                            .map(|choice| {
                                (
                                    choice.rule,
                                    Some(SourceChoiceActivation {
                                        option_ordinal: option.ordinal,
                                        value: choice.value.clone(),
                                    }),
                                )
                            })
                            .collect::<Vec<_>>()
                    })
                    .unwrap_or_default()
            } else {
                resolve(uuid)?
                    .map(|rule| vec![(rule, None)])
                    .unwrap_or_default()
            };
            for (rule, choice) in targets {
                let passive = match item {
                    ItemSourceView::AbilitySource(source) => SourceFieldView::from(&source.system)
                        .and_then(|s| (&s.action_type).into())
                        .and_then(|a| (&a.value).into())
                        .value()
                        .is_some(),
                    ItemSourceView::FeatSource(source) => SourceFieldView::from(&source.system)
                        .and_then(|s| (&s.action_type).into())
                        .and_then(|a| (&a.value).into())
                        .value()
                        .is_some(),
                    ItemSourceView::EffectSource(source) => {
                        SourceFieldView::from(&source.system)
                            .and_then(|s| (&s.duration).into())
                            .and_then(|s| (&s.unit).into())
                            .value()
                            == Some(&SourceFromSchemaDurationDataSchemaUnit::Unlimited)
                    }
                    _ => false,
                };
                let no_requirements = [&grant.requires_equipped, &grant.requires_investment]
                    .iter()
                    .all(|requirement| {
                        matches!(
                            requirement,
                            SourcePresence::Missing
                                | SourcePresence::Null
                                | SourcePresence::Value(false)
                        )
                    });
                let value = grant_value(rule, &grant.alterations).unwrap_or(None);
                let supported = grant.in_memory_only == SourcePresence::Value(true)
                    && matches!(
                        grant.ignored,
                        SourcePresence::Missing | SourcePresence::Value(false)
                    )
                    && passive
                    && no_requirements
                    && grant.additional_fields.fields().is_empty();
                if !supported {
                    continue;
                }
                use super::applicability::{ConditionImmunityState, condition_immunity};
                let activation_unavailable =
                    match condition_immunity(&immunities, rule.definition().slug) {
                        ConditionImmunityState::Immune => continue,
                        ConditionImmunityState::Conditional => Some(format!(
                            "Confirm whether {} immunity's exceptions apply.",
                            rule.definition().name
                        )),
                        ConditionImmunityState::Unknown => {
                            Some("Condition immunities are unavailable.".into())
                        }
                        ConditionImmunityState::Available => None,
                    };
                conditions.push(SourceCondition {
                    record_key: record.key().to_string(),
                    owner: owner.clone(),
                    grant_ordinal: Some(ordinal),
                    source_label: label.clone(),
                    rule,
                    value,
                    predicate: grant.predicate.clone(),
                    choice,
                    activation_unavailable,
                });
            }
        }
    }
    options.retain(|option| {
        conditions.iter().any(|condition| {
            condition.owner == option.owner
                && (condition
                    .choice
                    .as_ref()
                    .is_some_and(|choice| choice.option_ordinal == option.ordinal)
                    || predicate_mentions(&condition.predicate, &option.option))
        })
    });
    Ok(SourceConditions {
        options,
        uses,
        entries: conditions,
        unavailable_reason,
        source_fingerprint: None,
    })
}

/// Only constant badge overrides are supported; expressions retain an unavailable value.
fn grant_value(
    rule: ConditionRule,
    alterations: &SourcePresence<SourceFromSchemaAuraEffectSchemaAlterations>,
) -> Option<Option<i64>> {
    let mut value = rule.definition().default_value;
    let alterations = match alterations {
        SourcePresence::Missing => return Some(value),
        SourcePresence::Value(value) => value,
        _ => return None,
    };
    for alteration in alterations {
        if alteration.property.as_value()
            != Some(&SourceFromSchemaItemAlterationSchemaProperty::BadgeValue)
            || alteration.mode.as_value() != Some(&SourceFromSchemaAELikeSchemaMode::Override)
            || !alteration.additional_fields.fields().is_empty()
            || alteration.from_equipment == SourcePresence::Value(true)
        {
            return None;
        }
        let Some(JSONValue::Number(number)) = alteration.value.as_value() else {
            return None;
        };
        value = Some(number.as_i64().filter(|value| *value >= 0)?);
    }
    Some(value)
}

impl SourceCondition {
    pub fn active(
        &self,
        conditions: &[ConditionRule],
        sources: &SourceConditions,
        participant: &atlas_local_state::EncounterParticipant,
    ) -> Option<bool> {
        if self.activation_unavailable.is_some() {
            return None;
        }
        if let Some(choice) = &self.choice {
            let option = sources.options.iter().find(|option| {
                option.owner == self.owner && option.ordinal == choice.option_ordinal
            })?;
            let (active, selection) =
                option.state(participant, sources.source_fingerprint.as_deref());
            if !active || selection.as_deref() != Some(choice.value.as_str()) {
                return Some(false);
            }
        }

        match &self.predicate {
            SourcePresence::Missing => Some(true),
            SourcePresence::Value(terms) => predicate_all(terms, conditions, sources, participant),
            _ => None,
        }
    }
}

fn predicate_all(
    terms: &[PredicateStatement],
    conditions: &[ConditionRule],
    sources: &SourceConditions,
    participant: &atlas_local_state::EncounterParticipant,
) -> Option<bool> {
    let mut unknown = false;
    for term in terms {
        match predicate_term(term, conditions, sources, participant) {
            Some(false) => return Some(false),
            None => unknown = true,
            _ => {}
        }
    }
    (!unknown).then_some(true)
}
fn predicate_term(
    term: &PredicateStatement,
    conditions: &[ConditionRule],
    sources: &SourceConditions,
    participant: &atlas_local_state::EncounterParticipant,
) -> Option<bool> {
    match term {
        PredicateStatement::String(value) => {
            if let Some(option) = sources
                .options
                .iter()
                .find(|option| &option.option == value)
            {
                let active = option
                    .state(participant, sources.source_fingerprint.as_deref())
                    .0;
                return match &option.predicate {
                    SourcePresence::Missing => Some(active),
                    SourcePresence::Value(terms) if terms.is_empty() => Some(active),
                    _ => None,
                };
            }
            let slug = value.strip_prefix("self:condition:")?;
            atlas_foundry_model::generated::condition_definitions::CONDITION_DEFINITIONS
                .iter()
                .any(|definition| definition.slug == slug)
                .then(|| conditions.iter().any(|rule| rule.definition().slug == slug))
        }
        PredicateStatement::Negation(value) if value.additional_fields.fields().is_empty() => {
            predicate_term(value.not.as_value()?, conditions, sources, participant)
                .map(|value| !value)
        }
        PredicateStatement::Conjunction(value) if value.additional_fields.fields().is_empty() => {
            predicate_all(value.and.as_value()?, conditions, sources, participant)
        }
        PredicateStatement::Disjunction(value) if value.additional_fields.fields().is_empty() => {
            let values = value
                .or
                .as_value()?
                .iter()
                .map(|term| predicate_term(term, conditions, sources, participant))
                .collect::<Vec<_>>();
            if values.contains(&Some(true)) {
                Some(true)
            } else if values.contains(&None) {
                None
            } else {
                Some(false)
            }
        }
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn facts(items: serde_json::Value) -> SourceConditions {
        let source = atlas_foundry_model::admit_document_source(
            "Actor",
            SourceContext::new("test", "fixture", "$"),
            &serde_json::to_vec(&json!({"_id":"aaaaaaaaaaaaaaaa","type":"npc","system":{"traits":{"value":[]},"attributes":{"immunities":[]}},"items":items}))
                .unwrap(),
        )
        .unwrap()
        .model
        .unwrap();
        let record = SourceBackedRecord::new("test", source).unwrap();
        source_conditions(&record, |uuid| {
            Ok(match uuid {
                "Compendium.pf2e.conditionitems.Item.Slowed" => Some(ConditionRule::Slowed),
                "Compendium.pf2e.conditionitems.Item.Quickened" => Some(ConditionRule::Quickened),
                _ => None,
            })
        })
        .unwrap()
    }

    fn ability(rules: serde_json::Value) -> serde_json::Value {
        json!({"_id":"bbbbbbbbbbbbbbbb","name":"Renamed ability","type":"action",
            "system":{"actionType":{"value":"passive"},"rules":rules}})
    }

    #[test]
    fn structured_grants_keep_owner_identity_without_matching_creature_or_ability_names() {
        let result = facts(json!([ability(
            json!([{"key":"GrantItem","inMemoryOnly":true,
            "uuid":"Compendium.pf2e.conditionitems.Item.Slowed"}])
        )]));
        assert_eq!(result.entries.len(), 1);
        let condition = &result.entries[0];
        assert_eq!(condition.rule, ConditionRule::Slowed);
        assert_eq!(condition.value, Some(1));
        assert_eq!(condition.grant_ordinal, Some(0));
        assert_eq!(
            condition.owner.identity,
            atlas_domain::OwnedContentIdentity::Stable("bbbbbbbbbbbbbbbb".into())
        );
    }

    #[test]
    fn non_zombie_quickened_grant_uses_the_same_adapter() {
        let result = facts(json!([ability(
            json!([{"key":"GrantItem","inMemoryOnly":true,
            "uuid":"Compendium.pf2e.conditionitems.Item.Quickened"}])
        )]));
        assert_eq!(result.entries[0].rule, ConditionRule::Quickened);
    }

    #[test]
    fn owned_conditions_keep_authored_values_and_do_not_default_missing_values() {
        let result = facts(json!([
            {"_id":"bbbbbbbbbbbbbbbb","type":"condition","system":{"slug":"slowed","value":{"isValued":true,"value":2}}},
            {"_id":"cccccccccccccccc","type":"condition","system":{"slug":"slowed","value":{"isValued":true,"value":null}}}
        ]));
        assert_eq!(
            result.entries.iter().map(|c| c.value).collect::<Vec<_>>(),
            vec![Some(2), None]
        );
        assert!(result.entries.iter().all(|c| c.grant_ordinal.is_none()));
    }

    #[test]
    fn conditional_grants_are_retained_but_unknown_activation_is_not_assumed() {
        let result = facts(json!([ability(
            json!([{"key":"GrantItem","inMemoryOnly":true,
            "uuid":"Compendium.pf2e.conditionitems.Item.Slowed","predicate":["enabled"]}])
        )]));
        let participant = super::super::numerical_tests::participant();
        assert_eq!(result.entries.len(), 1);
        assert_eq!(result.entries[0].active(&[], &result, &participant), None);
        assert!(result.active_entries(&participant).is_empty());
    }
    #[test]
    fn numeric_badge_overrides_are_used_without_evaluating_expressions() {
        let result = facts(json!([ability(
            json!([{"key":"GrantItem","inMemoryOnly":true,
            "uuid":"Compendium.pf2e.conditionitems.Item.Slowed",
            "alterations":[{"property":"badge-value","mode":"override","value":2}]}])
        )]));
        assert_eq!(result.entries[0].value, Some(2));
        let unsupported = facts(json!([ability(
            json!([{"key":"GrantItem","inMemoryOnly":true,
            "uuid":"Compendium.pf2e.conditionitems.Item.Slowed","requiresEquipped":true}])
        )]));
        assert!(unsupported.entries.is_empty());
    }
    #[test]
    fn known_condition_activates_a_source_grant_and_removal_restores_the_forecast() {
        let result = facts(json!([
            {"_id":"cccccccccccccccc","type":"condition","system":{"slug":"deafened"}},
            ability(json!([{"key":"GrantItem","inMemoryOnly":true,
                "uuid":"Compendium.pf2e.conditionitems.Item.Slowed","predicate":["self:condition:deafened"]}]))
        ]));
        let participant = super::super::numerical_tests::participant();
        assert_eq!(
            super::super::capabilities::action_budget(&participant, &result)
                .actions
                .adjusted_value,
            Some(2)
        );
        let without = facts(json!([ability(
            json!([{"key":"GrantItem","inMemoryOnly":true,
            "uuid":"Compendium.pf2e.conditionitems.Item.Slowed","predicate":["self:condition:deafened"]}])
        )]));
        assert_eq!(
            super::super::capabilities::action_budget(&participant, &without)
                .actions
                .adjusted_value,
            Some(3)
        );
    }
    #[test]
    fn owned_unlimited_effects_grant_quickened_without_creature_specific_policy() {
        let result = facts(
            json!([{"_id":"bbbbbbbbbbbbbbbb","name":"Owned effect","type":"effect",
            "system":{"duration":{"unit":"unlimited"},"rules":[{"key":"GrantItem","inMemoryOnly":true,"uuid":"Compendium.pf2e.conditionitems.Item.Quickened"}]}}]),
        );
        let participant = super::super::numerical_tests::participant();
        assert_eq!(
            super::super::capabilities::action_budget(&participant, &result)
                .actions
                .adjusted_value,
            Some(4)
        );
    }
    #[test]
    fn ignored_rules_and_prose_only_mechanics_never_produce_effects() {
        let mut source = ability(
            json!([{"key":"GrantItem","ignored":true,"inMemoryOnly":true,"uuid":"Compendium.pf2e.conditionitems.Item.Slowed"}]),
        );
        source["system"]["description"] =
            json!({"value":"Permanently slowed 1; cannot use reactions."});
        assert!(facts(json!([source])).entries.is_empty());
    }
}

fn predicate_mentions(predicate: &SourcePresence<Vec<PredicateStatement>>, option: &str) -> bool {
    fn mentions(term: &PredicateStatement, option: &str) -> bool {
        match term {
            PredicateStatement::String(value) => value == option,
            PredicateStatement::Negation(value) => value
                .not
                .as_value()
                .is_some_and(|term| mentions(term, option)),
            PredicateStatement::Conjunction(value) => value
                .and
                .as_value()
                .is_some_and(|terms| terms.iter().any(|term| mentions(term, option))),
            PredicateStatement::Disjunction(value) => value
                .or
                .as_value()
                .is_some_and(|terms| terms.iter().any(|term| mentions(term, option))),
            _ => false,
        }
    }
    predicate
        .as_value()
        .is_some_and(|terms| terms.iter().any(|term| mentions(term, option)))
}
