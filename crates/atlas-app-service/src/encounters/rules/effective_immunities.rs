//! Checked authored immunities plus the pinned engine's generated trait grants.
use atlas_foundry_model::generated::condition_definitions::{ImmunityPredicate, TRAIT_IMMUNITIES};
use atlas_record::source_record::{
    ActorIwrEntry, ActorIwrKind, ActorIwrType, SourceBackedRecord, SourceQueryView,
};

pub(super) struct EffectiveImmunity<'a> {
    pub kind: String,
    pub authored: Option<ActorIwrEntry<'a>>,
}
pub(super) struct EffectiveImmunities<'a> {
    pub entries: Vec<EffectiveImmunity<'a>>,
    pub authored_complete: bool,
    pub uncertain_types: Vec<&'static str>,
}
struct Context<'a> {
    actor_kind: Option<&'a str>,
    traits: Option<&'a [String]>,
    authored: &'a [String],
    authored_complete: bool,
}
impl Context<'_> {
    fn evaluate(&self, predicate: ImmunityPredicate) -> Option<bool> {
        match predicate {
            ImmunityPredicate::Trait(trait_key) => self
                .traits
                .map(|traits| traits.iter().any(|value| value == trait_key)),
            ImmunityPredicate::ActorKind(kind) => self.actor_kind.map(|value| value == kind),
            ImmunityPredicate::Authored(kind) => {
                if self.authored.iter().any(|value| value == kind) {
                    Some(true)
                } else {
                    self.authored_complete.then_some(false)
                }
            }
            ImmunityPredicate::Not(term) => self.evaluate(*term).map(|value| !value),
            ImmunityPredicate::All(terms) => {
                let mut complete = true;
                for term in terms {
                    match self.evaluate(*term) {
                        Some(false) => return Some(false),
                        None => complete = false,
                        _ => {}
                    }
                }
                complete.then_some(true)
            }
            ImmunityPredicate::Any(terms) => {
                let mut complete = true;
                for term in terms {
                    match self.evaluate(*term) {
                        Some(true) => return Some(true),
                        None => complete = false,
                        _ => {}
                    }
                }
                complete.then_some(false)
            }
        }
    }
}

pub(super) fn effective_immunities(record: &SourceBackedRecord) -> EffectiveImmunities<'_> {
    let query = SourceQueryView::new(record.source(), record.key().pack().as_str(), "");
    let authored = query.actor().iwr(ActorIwrKind::Immunity).value();
    let mut complete = authored.is_some();
    let mut entries = Vec::new();
    if let Some(authored) = authored {
        for entry in authored.iter() {
            let kind = match entry.iwr_type().value() {
                Some(ActorIwrType::Immunity(kind)) => {
                    serde_json::to_value(kind).ok().and_then(|value| {
                        value
                            .as_str()
                            .or_else(|| value.get("$value").and_then(serde_json::Value::as_str))
                            .map(str::to_owned)
                    })
                }
                _ => None,
            };
            if let Some(kind) = kind {
                entries.push(EffectiveImmunity {
                    kind,
                    authored: Some(entry),
                });
            } else {
                complete = false;
            }
        }
    }
    let mut result = EffectiveImmunities {
        entries,
        authored_complete: complete,
        uncertain_types: vec![],
    };
    let actor_kind = query.source.source_type().value();
    // The selected upstream helper is called for CreaturePF2e, not arbitrary actors.
    if actor_kind.is_some_and(|kind| !matches!(kind, "npc" | "character" | "familiar")) {
        return result;
    }
    let authored_kinds: Vec<_> = result
        .entries
        .iter()
        .map(|entry| entry.kind.clone())
        .collect();
    let context = Context {
        actor_kind,
        traits: query.traits().value(),
        authored: &authored_kinds,
        authored_complete: complete,
    };
    for rule in TRAIT_IMMUNITIES {
        match context.evaluate(rule.predicate) {
            Some(true) => result.entries.push(EffectiveImmunity {
                kind: rule.immunity.into(),
                authored: None,
            }),
            Some(false) => {}
            None => {
                if !result.uncertain_types.contains(&rule.immunity) {
                    result.uncertain_types.push(rule.immunity);
                }
            }
        }
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    fn actor(
        kind: &str,
        traits: serde_json::Value,
        immunities: serde_json::Value,
    ) -> SourceBackedRecord {
        atlas_search::test_support::record(
            "test-actors",
            "Actor",
            json!({
                "_id":"nativeImmunity01","name":"Native immunity fixture","type":kind,
                "system":{"traits":{"value":traits},"attributes":{"immunities":immunities}}
            }),
        )
        .record
    }
    fn generated(record: &SourceBackedRecord) -> Vec<String> {
        effective_immunities(record)
            .entries
            .into_iter()
            .filter(|entry| entry.authored.is_none())
            .map(|entry| entry.kind)
            .collect()
    }
    #[test]
    fn runtime_predicates_support_new_trait_keys_without_trait_specific_dispatch() {
        let traits = vec!["future-trait".to_owned()];
        let context = Context {
            actor_kind: Some("npc"),
            traits: Some(&traits),
            authored: &[],
            authored_complete: true,
        };
        let predicate = ImmunityPredicate::All(&[
            ImmunityPredicate::Trait("future-trait"),
            ImmunityPredicate::Not(&ImmunityPredicate::Authored("prone")),
        ]);
        assert_eq!(context.evaluate(predicate), Some(true));
        let authored = vec!["prone".to_owned()];
        assert_eq!(
            Context {
                authored: &authored,
                ..context
            }
            .evaluate(predicate),
            Some(false)
        );
    }
    #[test]
    fn native_trait_guards_cover_construct_swarm_and_mindless_without_rust_trait_branches() {
        let construct = actor("npc", json!(["construct"]), json!([]));
        assert!(generated(&construct).contains(&"unconscious".into()));
        assert!(generated(&construct).contains(&"bleed".into()));
        let swarm = actor("npc", json!(["swarm"]), json!([]));
        assert_eq!(generated(&swarm), vec!["grabbed", "prone", "restrained"]);
        let mindless = actor("npc", json!(["mindless"]), json!([]));
        assert_eq!(generated(&mindless), vec!["mental"]);
    }
    #[test]
    fn source_actor_and_trait_exclusions_preserve_native_boundaries() {
        assert!(
            generated(&actor(
                "character",
                json!(["construct", "swarm"]),
                json!([])
            ))
            .is_empty()
        );
        assert!(generated(&actor("npc", json!(["construct", "eidolon"]), json!([]))).is_empty());
        assert!(generated(&actor("hazard", json!(["construct"]), json!([]))).is_empty());
    }
    #[test]
    fn authored_conditional_immunity_blocks_the_unconditional_trait_default() {
        let record = actor(
            "npc",
            json!(["swarm"]),
            json!([{"type":"prone","exceptions":["magical"]}]),
        );
        let facts = effective_immunities(&record);
        assert_eq!(
            facts
                .entries
                .iter()
                .filter(|entry| entry.kind == "prone")
                .count(),
            1
        );
        assert!(
            facts
                .entries
                .iter()
                .find(|entry| entry.kind == "prone")
                .unwrap()
                .authored
                .is_some()
        );
    }
    #[test]
    fn unknown_traits_and_unknown_authored_lists_do_not_turn_into_empty_facts() {
        let record = actor("npc", serde_json::Value::Null, json!([]));
        let facts = effective_immunities(&record);
        assert!(facts.entries.is_empty());
        assert!(facts.uncertain_types.contains(&"prone"));
        assert!(!facts.uncertain_types.contains(&"clumsy"));
        let record = actor("npc", json!(["swarm"]), serde_json::Value::Null);
        let facts = effective_immunities(&record);
        assert!(!facts.authored_complete);
        assert!(facts.entries.is_empty());
        assert!(facts.uncertain_types.contains(&"prone"));
    }
    #[test]
    fn real_pinned_creatures_receive_immunities_missing_from_their_authored_lists() {
        use atlas_foundry_model::generated::{
            REFERENCE_SOURCE_COMMIT, condition_definitions::CONDITION_DEFINITIONS,
        };
        let fixture: serde_json::Value = serde_json::from_str(include_str!(
            "../../../tests/fixtures/condition-actor-immunities.json"
        ))
        .unwrap();
        assert_eq!(fixture["source_commit"], REFERENCE_SOURCE_COMMIT);
        let participant = super::super::numerical_tests::participant();
        for actor in fixture["actors"].as_array().unwrap() {
            let record =
                atlas_search::test_support::record("actors", "Actor", actor["record"].clone())
                    .record;
            let condition = CONDITION_DEFINITIONS
                .iter()
                .find(|definition| definition.slug == actor["expected_condition"].as_str().unwrap())
                .unwrap();
            let facts = super::super::source_applicability(&record, false);
            let definition =
                crate::encounters::conditions::modeled_condition_by_rule(condition.id).unwrap();
            assert!(
                matches!(
                    super::super::condition_applicability(definition, &participant, &facts),
                    Some(atlas_app_model::EncounterConditionEligibilityView::Immune { .. })
                ),
                "{}",
                actor["source_file"]
            );
            if condition.slug == "unconscious" {
                let protection = super::super::damage::persistent_protection(
                    Some(&record),
                    &participant,
                    "bleed",
                )
                .unwrap();
                assert!(protection.immune);
                assert_eq!(protection.apply(10), 0);
            }
        }
    }
}
