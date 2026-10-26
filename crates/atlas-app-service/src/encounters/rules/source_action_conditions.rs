//! Structured authored action-allotment causes; never infer mechanics from prose.
use super::super::conditions::ConditionRule;
use crate::AppServiceResult;
use atlas_domain::OwnedContentLocator;
use atlas_foundry_model::{RuleSource, SourceContext, SourcePresence, generated::*};
use atlas_record::source_record::{ItemSourceView, SourceBackedRecord, SourceFieldView};

#[derive(Debug, Default, Clone, PartialEq)]
pub(crate) struct SourceActionConditions {
    pub entries: Vec<SourceActionCondition>,
    pub unavailable_reason: Option<String>,
    pub source_fingerprint: Option<String>,
}

impl SourceActionConditions {
    pub fn unavailable() -> Self {
        Self {
            unavailable_reason: Some("This creature's source conditions are unavailable.".into()),
            ..Default::default()
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct SourceActionCondition {
    pub owner: OwnedContentLocator,
    pub grant_ordinal: Option<usize>,
    pub source_label: String,
    pub rule: ConditionRule,
    pub value: Option<i64>,
}

/// The resolver uses the existing reference identity policy, including qualified
/// authored aliases. A display label is never a condition identity.
pub(in crate::encounters) fn source_action_conditions(
    record: &SourceBackedRecord,
    mut resolve: impl FnMut(&str) -> AppServiceResult<Option<ConditionRule>>,
) -> AppServiceResult<SourceActionConditions> {
    let mut items = vec![];
    let availability = record.visit_immediate_actor_items(|index, owner, item| {
        items.push((index, owner.clone(), item));
    });
    if matches!(
        availability,
        SourceFieldView::Null | SourceFieldView::Invalid(_)
    ) {
        return Ok(SourceActionConditions::unavailable());
    }
    let mut conditions = vec![];
    let immunities = super::source_facts::source_applicability(record, false).immunities;
    let mut unavailable_reason = None;
    for (index, owner, item) in items {
        let label = atlas_record::source_record::SourceNodeView::Item(item)
            .name()
            .value()
            .cloned()
            .unwrap_or_default();
        if let ItemSourceView::ConditionSource(condition) = item {
            let system = SourceFieldView::from(&condition.system);
            let rule = match system.and_then(|s| (&s.slug).into()).value() {
                Some(SourceFromSchemaAfflictionConditionSchemaSlug::Slowed) => {
                    ConditionRule::Slowed
                }
                Some(SourceFromSchemaAfflictionConditionSchemaSlug::Quickened) => {
                    ConditionRule::Quickened
                }
                _ => continue,
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
            conditions.push(SourceActionCondition {
                owner,
                grant_ordinal: None,
                source_label: label,
                rule,
                value,
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
            let Some(rule @ (ConditionRule::Slowed | ConditionRule::Quickened)) = resolve(uuid)?
            else {
                continue;
            };
            // An empty predicate is equivalent to the schema's omitted default;
            // null, rejected or nonempty predicates never prove activation.
            let unconditional = match &grant.predicate {
                SourcePresence::Missing => true,
                SourcePresence::Value(terms) => terms.is_empty(),
                _ => false,
            };
            let passive = match item {
                ItemSourceView::AbilitySource(source) => {
                    SourceFieldView::from(&source.system)
                        .and_then(|s| (&s.action_type).into())
                        .and_then(|a| (&a.value).into())
                        .value()
                        == Some(&AbilitySystemSourceActionTypeSourceFromSchemaValue::Passive)
                }
                ItemSourceView::FeatSource(source) => {
                    SourceFieldView::from(&source.system)
                        .and_then(|s| (&s.action_type).into())
                        .and_then(|a| (&a.value).into())
                        .value()
                        == Some(&AbilitySystemSourceActionTypeSourceFromSchemaValue::Passive)
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
            let no_alterations = match &grant.alterations {
                SourcePresence::Missing => true,
                SourcePresence::Value(alterations) => alterations.is_empty(),
                _ => false,
            };
            let supported = grant.in_memory_only == SourcePresence::Value(true)
                && matches!(
                    grant.ignored,
                    SourcePresence::Missing | SourcePresence::Value(false)
                )
                && unconditional
                && passive
                && no_requirements
                && no_alterations
                && grant.additional_fields.fields().is_empty();
            if !supported {
                continue;
            }
            use super::applicability::{ConditionImmunityState, condition_immunity};
            match condition_immunity(&immunities, rule.definition().slug) {
                ConditionImmunityState::Immune => continue,
                ConditionImmunityState::Conditional => {
                    unavailable_reason = Some(format!(
                        "Confirm whether {} immunity's exceptions apply.",
                        rule.definition().name
                    ));
                    continue;
                }
                ConditionImmunityState::Unknown => {
                    unavailable_reason = Some("Condition immunities are unavailable.".into());
                    continue;
                }
                ConditionImmunityState::Available => {}
            }
            conditions.push(SourceActionCondition {
                owner: owner.clone(),
                grant_ordinal: Some(ordinal),
                source_label: label.clone(),
                rule,
                value: rule.definition().default_value,
            });
        }
    }
    Ok(SourceActionConditions {
        entries: conditions,
        unavailable_reason,
        source_fingerprint: None,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn facts(items: serde_json::Value) -> SourceActionConditions {
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
        source_action_conditions(&record, |uuid| {
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
    fn conditional_altered_and_nonpassive_grants_do_not_become_unconditional_effects() {
        for extra in [
            json!({"predicate":["enabled"]}),
            json!({"alterations":[{"property":"badge-value","mode":"override","value":2}]}),
            json!({"requiresEquipped":true}),
        ] {
            let mut grant = json!({"key":"GrantItem","inMemoryOnly":true,"uuid":"Compendium.pf2e.conditionitems.Item.Slowed"});
            grant
                .as_object_mut()
                .unwrap()
                .extend(extra.as_object().unwrap().clone());
            let result = facts(json!([ability(json!([grant]))]));
            assert!(result.entries.is_empty());
        }
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
