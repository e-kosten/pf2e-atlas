//! Atomic specific-rule interpretation of a retained generic Item rule.
use super::generated::{JSONValue, RuleElementSource, RuleSource};
use super::{SourceContext, SourceDiagnostic, SourceObject, SourcePresence, SourceValue};

impl RuleElementSource {
    /// Interpret this complete retained node with the generated specific parser.
    /// This does not read JSON, admit a parent again, apply defaults or execute rules.
    /// A rejected field makes the entire specific interpretation unavailable.
    pub fn interpret(&self, context: SourceContext) -> Result<RuleSource, SourceDiagnostic> {
        let mut fields = self.additional_fields.fields().to_vec();
        for (key, value) in [
            ("key", &self.key),
            ("label", &self.label),
            ("slug", &self.slug),
        ] {
            append(&mut fields, key, value, |value| {
                SourceValue::String(value.clone())
            })?;
        }
        for (key, value) in [
            ("ignored", &self.ignored),
            ("predicate", &self.predicate),
            ("priority", &self.priority),
            ("removeUponCreate", &self.remove_upon_create),
            ("requiresEquipped", &self.requires_equipped),
            ("requiresInvestment", &self.requires_investment),
        ] {
            append(&mut fields, key, value, |value| match value {
                JSONValue::String(value) => SourceValue::String(value.clone()),
                JSONValue::Number(value) => SourceValue::Number(value.clone()),
                JSONValue::Boolean(value) => SourceValue::Boolean(*value),
                JSONValue::Object(value) => value.clone(),
            })?;
        }
        let value = SourceValue::Object(SourceObject { fields });
        super::generated::parse_rule_source(&value, &context, &context.json_path)
    }
}

impl super::generated::SourceFromSchemaBattleFormRuleSchemaBracketsEntrySourceFromSchema {
    /// Interpret a retained bracket with the same generated contract as overrides.
    /// Selection and composition belong to the consumer; source values stay intact.
    pub fn interpret_overrides(
        &self,
        context: SourceContext,
    ) -> Result<super::generated::SourceFromSchemaBattleFormRuleOverrideSchema, SourceDiagnostic>
    {
        let value = match &self.value {
            SourcePresence::Value(value) => value,
            SourcePresence::Invalid(rejection) => return Err(rejection.diagnostic.clone()),
            _ => {
                return Err(context.message(
                    &context.json_path,
                    "battle-form override object",
                    "value unavailable",
                ));
            }
        };
        // The generated override parser is private to its generated module. Reuse
        // its public rule entry point rather than defining a second field schema.
        let wrapped = SourceValue::Object(SourceObject {
            fields: vec![
                ("key".into(), SourceValue::String("BattleForm".into())),
                ("overrides".into(), value.clone()),
            ],
        });
        let parsed =
            super::generated::parse_battle_form_rule(&wrapped, &context, &context.json_path)?;
        match parsed.overrides {
            SourcePresence::Value(overrides) => Ok(overrides),
            SourcePresence::Invalid(rejection) => Err(rejection.diagnostic),
            _ => Err(context.message(
                &context.json_path,
                "battle-form override object",
                "value unavailable",
            )),
        }
    }
}

fn append<T>(
    fields: &mut Vec<(String, SourceValue)>,
    key: &str,
    value: &SourcePresence<T>,
    convert: impl FnOnce(&T) -> SourceValue,
) -> Result<(), SourceDiagnostic> {
    let value = match value {
        SourcePresence::Missing => return Ok(()),
        SourcePresence::Null => SourceValue::Null,
        SourcePresence::Value(value) => convert(value),
        SourcePresence::Invalid(rejection) => return Err(rejection.diagnostic.clone()),
    };
    fields.push((key.into(), value));
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn generic(json: &str) -> RuleElementSource {
        let json = format!(r#"{{"type":"action","system":{{"rules":[{json}]}}}}"#);
        let crate::ItemSourcePF2e::AbilitySource(item) =
            crate::parse_item_source_pf2e(context(), json.as_bytes()).unwrap()
        else {
            panic!("expected an ability")
        };
        item.system.as_value().unwrap().rules.as_value().unwrap()[0].clone()
    }

    fn context() -> SourceContext {
        SourceContext::new("test:actor", "fixture.json", "$.system.rules[0]")
    }

    #[test]
    fn retained_grant_uses_the_specific_parser_without_changing_the_source() {
        let rule = generic(
            r#"{"key":"GrantItem","inMemoryOnly":true,"uuid":"Compendium.pf2e.conditionitems.Item.Slowed"}"#,
        );
        let original = rule.clone();
        let RuleSource::GrantItem(grant) = rule.interpret(context()).unwrap() else {
            panic!("expected a typed grant")
        };
        assert_eq!(grant.in_memory_only, SourcePresence::Value(true));
        assert_eq!(grant.predicate, SourcePresence::Missing);
        assert_eq!(rule, original);
    }

    #[test]
    fn predicate_and_alteration_rejections_never_become_partial_rules() {
        for json in [
            r#"{"key":"GrantItem","predicate":["enabled",3],"uuid":"target"}"#,
            r#"{"key":"GrantItem","alterations":[{"property":"badge-value","mode":7,"value":1}],"uuid":"target"}"#,
            r#"{"key":"GrantItem","inMemoryOnly":"true","uuid":"target"}"#,
        ] {
            let rule = generic(json);
            let original = rule.clone();
            assert!(rule.interpret(context()).is_err(), "{json}");
            assert_eq!(rule, original);
        }
    }

    #[test]
    fn duplicate_specialized_fields_are_not_collapsed_before_interpretation() {
        let rule = generic(r#"{"key":"GrantItem","uuid":"first","uuid":"second"}"#);
        assert!(rule.interpret(context()).is_err());
        assert_eq!(rule.additional_fields.fields().len(), 2);
    }

    #[test]
    fn conditional_and_ignored_grants_retain_their_activation_inputs() {
        let rule = generic(
            r#"{"key":"GrantItem","ignored":true,"predicate":["self:effect:rage"],"inMemoryOnly":true,"uuid":"target"}"#,
        );
        let RuleSource::GrantItem(grant) = rule.interpret(context()).unwrap() else {
            panic!("expected a typed grant")
        };
        assert_eq!(grant.ignored, SourcePresence::Value(true));
        assert_eq!(grant.predicate.as_value().unwrap().len(), 1);
    }
    #[test]
    fn battle_form_brackets_use_the_authoritative_override_contract_without_mutation() {
        let retained = generic(
            r#"{"key":"BattleForm","brackets":[{"start":4,"value":{"tempHP":15,"armorClass":{"modifier":"18 + @actor.level"}}}]}"#,
        );
        let original = retained.clone();
        let RuleSource::BattleForm(rule) = retained.interpret(context()).unwrap() else {
            panic!("battle form")
        };
        let bracket = &rule.brackets.as_value().unwrap()[0];
        let overrides = bracket.interpret_overrides(context()).unwrap();
        assert_eq!(
            overrides.temp_hp,
            SourcePresence::Value(JSONValue::Number(15.into()))
        );
        assert_eq!(retained, original);
        for value in [
            r#"{"tempHP":5,"tempHP":15}"#,
            r#"{"strikes":{"jaws":{"damage":{"dice":"bad"}}}}"#,
            "null",
        ] {
            let retained = generic(&format!(
                r#"{{"key":"BattleForm","brackets":[{{"start":4,"value":{value}}}]}}"#
            ));
            let RuleSource::BattleForm(rule) = retained.interpret(context()).unwrap() else {
                panic!("battle form")
            };
            assert!(
                rule.brackets.as_value().unwrap()[0]
                    .interpret_overrides(context())
                    .is_err(),
                "{value}"
            );
        }
    }
}
