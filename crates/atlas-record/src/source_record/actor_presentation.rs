//! Consumer-required borrowed authored facts; no runtime defaults or rule execution.
use super::{ActorQueryView, ItemSourceView, SourceFieldView, SourceNodeView};
use atlas_foundry_model::generated::*;
use serde_json::Number;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SourceNpcAdjustment {
    Normal,
    Elite,
    Weak,
}

impl<'a> ActorQueryView<'a> {
    /// Qualifications on the authored NPC movement, not an additional speed.
    pub fn movement_details(self) -> SourceFieldView<'a, &'a str> {
        match self.source {
            super::SourceNodeView::Actor(atlas_foundry_model::ActorSourcePF2e::NPCSource(s)) => {
                SourceFieldView::from(&s.system)
                    .and_then(|s| (&s.attributes).into())
                    .and_then(|a| (&a.speed).into())
                    .and_then(|speed| (&speed.details).into())
                    .map(String::as_str)
            }
            _ => SourceFieldView::NotApplicable,
        }
    }

    /// Authored NPC hardness, retained outside the upstream declaration.
    pub fn creature_hardness(self) -> SourceFieldView<'a, &'a Number> {
        let super::SourceNodeView::Actor(atlas_foundry_model::ActorSourcePF2e::NPCSource(s)) =
            self.source
        else {
            return SourceFieldView::NotApplicable;
        };
        SourceFieldView::from(&s.system)
            .and_then(|s| (&s.attributes).into())
            .and_then(|a| {
                retained_member(
                    &a.additional_fields,
                    "hardness",
                    "/system/attributes/hardness",
                )
            })
            .and_then(|hardness| match hardness {
                atlas_foundry_model::SourceValue::Object(object) => {
                    retained_member(object, "value", "/system/attributes/hardness/value")
                }
                _ => SourceFieldView::ProjectionInvalid {
                    source_path: "/system/attributes/hardness",
                    reason: "expected a hardness object",
                },
            })
            .and_then(|value| match value {
                atlas_foundry_model::SourceValue::Number(number) => SourceFieldView::Value(number),
                _ => SourceFieldView::ProjectionInvalid {
                    source_path: "/system/attributes/hardness/value",
                    reason: "expected a number",
                },
            })
    }

    pub fn focus_capacity(self) -> SourceFieldView<'a, &'a Number> {
        match self.source {
            super::SourceNodeView::Actor(atlas_foundry_model::ActorSourcePF2e::NPCSource(s)) => {
                SourceFieldView::from(&s.system)
                    .and_then(|s| (&s.resources).into())
                    .and_then(|r| (&r.focus).into())
                    .and_then(|f| (&f.max).into())
            }
            _ => SourceFieldView::NotApplicable,
        }
    }
    pub fn ritual_difficulty_class(self) -> SourceFieldView<'a, &'a Number> {
        match self.source {
            super::SourceNodeView::Actor(atlas_foundry_model::ActorSourcePF2e::NPCSource(s)) => {
                SourceFieldView::from(&s.system)
                    .and_then(|s| (&s.spellcasting).into())
                    .and_then(|s| (&s.rituals).into())
                    .and_then(|r| (&r.dc).into())
            }
            _ => SourceFieldView::NotApplicable,
        }
    }
    pub fn armor_class_details(self) -> SourceFieldView<'a, &'a str> {
        use super::SourceNodeView;
        use atlas_foundry_model::ActorSourcePF2e;
        match self.source {
            SourceNodeView::Actor(ActorSourcePF2e::NPCSource(s)) => {
                SourceFieldView::from(&s.system)
                    .and_then(|s| (&s.attributes).into())
                    .and_then(|a| (&a.ac).into())
                    .and_then(|v| (&v.details).into())
                    .map(String::as_str)
            }
            SourceNodeView::Actor(ActorSourcePF2e::HazardSource(s)) => {
                SourceFieldView::from(&s.system)
                    .and_then(|s| (&s.attributes).into())
                    .and_then(|a| (&a.ac).into())
                    .and_then(|v| {
                        retained_text(
                            &v.additional_fields,
                            "details",
                            "/system/attributes/ac/details",
                        )
                    })
            }
            _ => SourceFieldView::NotApplicable,
        }
    }
    pub fn hit_point_details(self) -> SourceFieldView<'a, &'a str> {
        use super::SourceNodeView;
        use atlas_foundry_model::ActorSourcePF2e;
        match self.source {
            SourceNodeView::Actor(ActorSourcePF2e::NPCSource(s)) => {
                SourceFieldView::from(&s.system)
                    .and_then(|s| (&s.attributes).into())
                    .and_then(|a| (&a.hp).into())
                    .and_then(|v| (&v.details).into())
                    .map(String::as_str)
            }
            SourceNodeView::Actor(ActorSourcePF2e::HazardSource(s)) => {
                SourceFieldView::from(&s.system)
                    .and_then(|s| (&s.attributes).into())
                    .and_then(|a| (&a.hp).into())
                    .and_then(|v| (&v.details).into())
                    .map(String::as_str)
            }
            _ => SourceFieldView::NotApplicable,
        }
    }
    pub fn general_save_details(self) -> SourceFieldView<'a, &'a str> {
        match self.source {
            super::SourceNodeView::Actor(atlas_foundry_model::ActorSourcePF2e::NPCSource(s)) => {
                SourceFieldView::from(&s.system)
                    .and_then(|s| (&s.attributes).into())
                    .and_then(|a| (&a.all_saves).into())
                    .and_then(|v| (&v.value).into())
                    .map(String::as_str)
            }
            _ => SourceFieldView::NotApplicable,
        }
    }
    pub fn save_details(self, save: super::ActorSave) -> SourceFieldView<'a, &'a str> {
        use super::ActorSave;
        match self.source {
            super::SourceNodeView::Actor(atlas_foundry_model::ActorSourcePF2e::NPCSource(s)) => {
                SourceFieldView::from(&s.system)
                    .and_then(|s| (&s.saves).into())
                    .and_then(|s| match save {
                        ActorSave::Fortitude => (&s.fortitude).into(),
                        ActorSave::Reflex => (&s.reflex).into(),
                        ActorSave::Will => (&s.will).into(),
                    })
                    .and_then(|s| (&s.save_detail).into())
                    .map(String::as_str)
            }
            _ => self.hazard_save_detail(save),
        }
    }
    /// Legacy authored hazard explanations retained outside the current declaration.
    pub fn hazard_save_detail(self, save: super::ActorSave) -> SourceFieldView<'a, &'a str> {
        use super::{ActorSave, SourceNodeView};
        use atlas_foundry_model::SourceValue;
        let source_path = match save {
            ActorSave::Fortitude => "/system/saves/fortitude/saveDetail",
            ActorSave::Reflex => "/system/saves/reflex/saveDetail",
            ActorSave::Will => "/system/saves/will/saveDetail",
        };
        let SourceNodeView::Actor(atlas_foundry_model::ActorSourcePF2e::HazardSource(source)) =
            self.source
        else {
            return SourceFieldView::NotApplicable;
        };
        SourceFieldView::from(&source.system)
            .and_then(|s| (&s.saves).into())
            .and_then(|s| match save {
                ActorSave::Fortitude => (&s.fortitude).into(),
                ActorSave::Reflex => (&s.reflex).into(),
                ActorSave::Will => (&s.will).into(),
            })
            .and_then(|s| {
                let mut values = s
                    .additional_fields
                    .fields()
                    .iter()
                    .filter_map(|(key, value)| (key == "saveDetail").then_some(value));
                match (values.next(), values.next()) {
                    (None, _) => SourceFieldView::Missing,
                    (Some(SourceValue::Null), None) => SourceFieldView::Null,
                    (Some(SourceValue::String(value)), None) => {
                        SourceFieldView::Value(value.as_str())
                    }
                    (Some(_), None) => SourceFieldView::ProjectionInvalid {
                        source_path,
                        reason: "expected a string",
                    },
                    (Some(_), Some(_)) => SourceFieldView::ProjectionInvalid {
                        source_path,
                        reason: "duplicate saveDetail members",
                    },
                }
            })
    }

    /// Distinguishes an omitted leaf from unavailable enclosing source data.
    pub fn npc_adjustment_is_unset(self) -> bool {
        match self.source {
            super::SourceNodeView::Actor(atlas_foundry_model::ActorSourcePF2e::NPCSource(s)) => {
                matches!(SourceFieldView::from(&s.system).and_then(|s| (&s.attributes).into()), SourceFieldView::Value(a) if matches!(a.adjustment, atlas_foundry_model::SourcePresence::Missing))
            }
            _ => false,
        }
    }
    pub fn hazard_stealth(self) -> SourceFieldView<'a, &'a Number> {
        match self.source {
            super::SourceNodeView::Actor(atlas_foundry_model::ActorSourcePF2e::HazardSource(s)) => {
                SourceFieldView::from(&s.system)
                    .and_then(|s| (&s.attributes).into())
                    .and_then(|a| (&a.stealth).into())
                    .and_then(|s| (&s.value).into())
            }
            _ => SourceFieldView::NotApplicable,
        }
    }
    pub fn authored_adjustment(self) -> SourceFieldView<'a, SourceNpcAdjustment> {
        match self.source {
            super::SourceNodeView::Actor(atlas_foundry_model::ActorSourcePF2e::NPCSource(s)) => {
                SourceFieldView::from(&s.system)
                    .and_then(|s| (&s.attributes).into())
                    .and_then(|a| match SourceFieldView::from(&a.adjustment) {
                        SourceFieldView::Null => {
                            SourceFieldView::Value(SourceNpcAdjustment::Normal)
                        }
                        value => value.map(|v| match v {
                            NPCAttributesSourceAdjustment::Elite => SourceNpcAdjustment::Elite,
                            NPCAttributesSourceAdjustment::Weak => SourceNpcAdjustment::Weak,
                        }),
                    })
            }
            _ => SourceFieldView::NotApplicable,
        }
    }
    pub fn visit_abilities(
        self,
        mut visit: impl FnMut(&'static str, SourceFieldView<'a, &'a Number>),
    ) -> SourceFieldView<'a, ()> {
        self.npc_abilities().map(|s| {
            for (key, value) in [
                ("str", &s.str),
                ("dex", &s.dex),
                ("con", &s.con),
                ("int", &s.int),
                ("wis", &s.wis),
                ("cha", &s.cha),
            ] {
                visit(
                    key,
                    SourceFieldView::from(value).and_then(|v| (&v.r#mod).into()),
                );
            }
        })
    }
    pub fn ability_modifier(self, key: &str) -> SourceFieldView<'a, &'a Number> {
        macro_rules! project {
            ($source:expr) => {
                SourceFieldView::from(&$source.system)
                    .and_then(|s| (&s.abilities).into())
                    .and_then(|a| match key {
                        "str" => SourceFieldView::from(&a.str).and_then(|v| (&v.r#mod).into()),
                        "dex" => SourceFieldView::from(&a.dex).and_then(|v| (&v.r#mod).into()),
                        "con" => SourceFieldView::from(&a.con).and_then(|v| (&v.r#mod).into()),
                        "int" => SourceFieldView::from(&a.int).and_then(|v| (&v.r#mod).into()),
                        "wis" => SourceFieldView::from(&a.wis).and_then(|v| (&v.r#mod).into()),
                        "cha" => SourceFieldView::from(&a.cha).and_then(|v| (&v.r#mod).into()),
                        _ => SourceFieldView::NotApplicable,
                    })
            };
        }
        match self.source {
            SourceNodeView::Actor(ActorSourcePF2e::NPCSource(source)) => project!(source),
            SourceNodeView::Actor(ActorSourcePF2e::CharacterSource(source)) => project!(source),
            _ => SourceFieldView::NotApplicable,
        }
    }

    pub fn skill_modifier(self, key: &str) -> SourceFieldView<'a, &'a Number> {
        self.npc_skills().and_then(|skills| {
            match skills.entries.iter().find(|(name, _)| name == key) {
                Some((_, skill)) => SourceFieldView::from(&skill.base),
                None => SourceFieldView::Missing,
            }
        })
    }

    pub fn skill_rank(self, key: &str) -> SourceFieldView<'a, &'a Number> {
        let SourceNodeView::Actor(ActorSourcePF2e::CharacterSource(source)) = self.source else {
            return SourceFieldView::NotApplicable;
        };
        SourceFieldView::from(&source.system)
            .and_then(|s| (&s.skills).into())
            .and_then(|skills| match key {
                "acrobatics" => SourceFieldView::from(&skills.acrobatics),
                "arcana" => SourceFieldView::from(&skills.arcana),
                "athletics" => SourceFieldView::from(&skills.athletics),
                "computers" => SourceFieldView::from(&skills.computers),
                "crafting" => SourceFieldView::from(&skills.crafting),
                "deception" => SourceFieldView::from(&skills.deception),
                "diplomacy" => SourceFieldView::from(&skills.diplomacy),
                "intimidation" => SourceFieldView::from(&skills.intimidation),
                "medicine" => SourceFieldView::from(&skills.medicine),
                "nature" => SourceFieldView::from(&skills.nature),
                "occultism" => SourceFieldView::from(&skills.occultism),
                "performance" => SourceFieldView::from(&skills.performance),
                "piloting" => SourceFieldView::from(&skills.piloting),
                "religion" => SourceFieldView::from(&skills.religion),
                "society" => SourceFieldView::from(&skills.society),
                "stealth" => SourceFieldView::from(&skills.stealth),
                "survival" => SourceFieldView::from(&skills.survival),
                "thievery" => SourceFieldView::from(&skills.thievery),
                _ => SourceFieldView::NotApplicable,
            })
            .and_then(|skill| (&skill.rank).into())
    }

    pub fn visit_skills(
        self,
        mut visit: impl FnMut(&'a str, SourceFieldView<'a, &'a NPCSkillSource>),
    ) -> SourceFieldView<'a, ()> {
        self.npc_skills().map(|s| {
            for (key, value) in &s.entries {
                visit(key, SourceFieldView::Value(value));
            }
        })
    }
}
impl<'a> ItemSourceView<'a> {
    pub fn self_effect(self) -> SourceFieldView<'a, &'a AbilitySystemSourceSelfEffect> {
        match self {
            Self::AbilitySource(s) => {
                SourceFieldView::from(&s.system).and_then(|s| (&s.self_effect).into())
            }
            Self::FeatSource(s) => {
                SourceFieldView::from(&s.system).and_then(|s| (&s.self_effect).into())
            }
            _ => SourceFieldView::NotApplicable,
        }
    }
    pub fn strike_effects(self) -> SourceFieldView<'a, &'a Vec<String>> {
        match self {
            Self::MeleeSource(s) => SourceFieldView::from(&s.system)
                .and_then(|s| (&s.attack_effects).into())
                .and_then(|s| (&s.value).into()),
            _ => SourceFieldView::NotApplicable,
        }
    }

    pub fn activity_frequency(self) -> SourceFieldView<'a, &'a AbilitySystemSourceFrequency> {
        match self {
            Self::AbilitySource(s) => {
                SourceFieldView::from(&s.system).and_then(|s| (&s.frequency).into())
            }
            Self::CampaignFeatureSource(s) => {
                SourceFieldView::from(&s.system).and_then(|s| (&s.frequency).into())
            }
            Self::FeatSource(s) => {
                SourceFieldView::from(&s.system).and_then(|s| (&s.frequency).into())
            }
            _ => SourceFieldView::NotApplicable,
        }
    }

    pub fn spell_range(self) -> SourceFieldView<'a, &'a str> {
        match self {
            Self::SpellSource(s) => SourceFieldView::from(&s.system)
                .and_then(|s| (&s.range).into())
                .and_then(|v| (&v.value).into())
                .map(String::as_str),
            _ => SourceFieldView::NotApplicable,
        }
    }
    pub fn spell_target(self) -> SourceFieldView<'a, &'a str> {
        match self {
            Self::SpellSource(s) => SourceFieldView::from(&s.system)
                .and_then(|s| (&s.target).into())
                .and_then(|v| (&v.value).into())
                .map(String::as_str),
            _ => SourceFieldView::NotApplicable,
        }
    }
    pub fn melee_attack(self) -> SourceFieldView<'a, &'a Number> {
        match self {
            Self::MeleeSource(s) => SourceFieldView::from(&s.system)
                .and_then(|s| (&s.bonus).into())
                .and_then(|s| (&s.value).into()),
            _ => SourceFieldView::NotApplicable,
        }
    }
    pub fn melee_damage(self) -> SourceFieldView<'a, &'a MeleeSystemSourceDamageRolls> {
        match self {
            Self::MeleeSource(s) => {
                SourceFieldView::from(&s.system).and_then(|s| (&s.damage_rolls).into())
            }
            _ => SourceFieldView::NotApplicable,
        }
    }
    pub fn lore_modifier(self) -> SourceFieldView<'a, &'a Number> {
        match self {
            Self::LoreSource(s) => SourceFieldView::from(&s.system)
                .and_then(|s| (&s.r#mod).into())
                .and_then(|s| (&s.value).into()),
            _ => SourceFieldView::NotApplicable,
        }
    }
    pub fn lore_variants(self) -> SourceFieldView<'a, &'a LoreSystemSourceVariants> {
        match self {
            Self::LoreSource(s) => {
                SourceFieldView::from(&s.system).and_then(|s| (&s.variants).into())
            }
            _ => SourceFieldView::NotApplicable,
        }
    }
}

fn retained_member<'a>(
    object: &'a atlas_foundry_model::SourceObject,
    field: &str,
    path: &'static str,
) -> SourceFieldView<'a, &'a atlas_foundry_model::SourceValue> {
    let mut values = object
        .fields()
        .iter()
        .filter_map(|(key, value)| (key == field).then_some(value));
    match (values.next(), values.next()) {
        (None, _) => SourceFieldView::Missing,
        (Some(atlas_foundry_model::SourceValue::Null), None) => SourceFieldView::Null,
        (Some(value), None) => SourceFieldView::Value(value),
        (Some(_), Some(_)) => SourceFieldView::ProjectionInvalid {
            source_path: path,
            reason: "duplicate retained members",
        },
    }
}
fn retained_text<'a>(
    object: &'a atlas_foundry_model::SourceObject,
    field: &str,
    path: &'static str,
) -> SourceFieldView<'a, &'a str> {
    retained_member(object, field, path).and_then(|value| match value {
        atlas_foundry_model::SourceValue::String(text) => SourceFieldView::Value(text.as_str()),
        _ => SourceFieldView::ProjectionInvalid {
            source_path: path,
            reason: "expected a string",
        },
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::source_record::{FieldAvailability, SourceBackedRecord, SourceQueryView};
    use atlas_foundry_model::{SourceContext, admit_document_source};
    use serde_json::{Value, json};
    fn record(value: Value) -> SourceBackedRecord {
        let source = admit_document_source(
            "Actor",
            SourceContext::new("actors", "fixture", "$"),
            &serde_json::to_vec(&value).unwrap(),
        )
        .unwrap()
        .model
        .unwrap();
        SourceBackedRecord::new("actors", source).unwrap()
    }
    #[test]
    fn retained_hardness_rejects_duplicate_outer_and_inner_members() {
        for attributes in [
            r#"{"hardness":{"value":8,"value":9}}"#,
            r#"{"hardness":{"value":8},"hardness":{"value":9}}"#,
        ] {
            let bytes = format!(
                r#"{{"_id":"aaaaaaaaaaaaaaaa","type":"npc","system":{{"attributes":{attributes}}}}}"#
            );
            let source = admit_document_source(
                "Actor",
                SourceContext::new("actors", "fixture", "$"),
                bytes.as_bytes(),
            )
            .unwrap()
            .model
            .unwrap();
            let source = SourceBackedRecord::new("actors", source).unwrap();
            assert!(matches!(
                SourceQueryView::new(source.source(), "actors", "")
                    .actor()
                    .creature_hardness(),
                SourceFieldView::ProjectionInvalid { .. }
            ));
        }
    }
    #[test]
    fn adjustment_distinguishes_declared_normal_missing_and_invalid() {
        for (adjustment, expected) in [
            (Some(Value::Null), FieldAvailability::Value),
            (None, FieldAvailability::Missing),
            (
                Some(json!("other")),
                FieldAvailability::Invalid {
                    source_path: String::new(),
                },
            ),
        ] {
            let mut attributes = json!({"hp":{"max":60}});
            if let Some(value) = adjustment {
                attributes["adjustment"] = value;
            }
            let r = record(
                json!({"_id":"aaaaaaaaaaaaaaaa","type":"npc","system":{"attributes":attributes}}),
            );
            let view = SourceQueryView::new(r.source(), "actors", "").actor();
            match expected {
                FieldAvailability::Value => assert_eq!(
                    view.authored_adjustment().value(),
                    Some(SourceNpcAdjustment::Normal)
                ),
                FieldAvailability::Missing => assert!(matches!(
                    view.authored_adjustment(),
                    SourceFieldView::Missing
                )),
                _ => assert!(matches!(
                    view.authored_adjustment().availability(),
                    FieldAvailability::Invalid { .. }
                )),
            }
        }
    }
    #[test]
    fn hazard_stealth_and_npc_skills_borrow_exact_authored_numbers() {
        let r = record(
            json!({"_id":"aaaaaaaaaaaaaaaa","type":"hazard","system":{"attributes":{"stealth":{"value":0}}}}),
        );
        let view = SourceQueryView::new(r.source(), "actors", "").actor();
        assert_eq!(view.hazard_stealth().value(), Some(&Number::from(0)));
        assert!(matches!(
            view.authored_adjustment(),
            SourceFieldView::NotApplicable
        ));
        let r = record(
            json!({"_id":"aaaaaaaaaaaaaaaa","type":"npc","system":{"abilities":{"str":{"mod":0}},"skills":{"arcana":{"base":7,"special":[{"base":11,"label":"Identifying magic","predicate":["identify-magic"]}]}}}}),
        );
        let view = SourceQueryView::new(r.source(), "actors", "").actor();
        let mut abilities = vec![];
        view.visit_abilities(|k, v| {
            if let Some(value) = v.value() {
                abilities.push((k, value.clone()));
            }
        });
        assert_eq!(abilities, vec![("str", Number::from(0))]);
        let mut skills = Vec::new();
        view.visit_skills(|k, v| {
            skills.push(k);
            if k == "arcana" {
                let skill = v.value().unwrap();
                assert_eq!(
                    SourceFieldView::from(&skill.base).value(),
                    Some(&Number::from(7))
                );
                assert_eq!(
                    SourceFieldView::from(&skill.special).value().unwrap()[0].base,
                    atlas_foundry_model::SourcePresence::Value(Number::from(11))
                );
            }
        });
        assert_eq!(skills, ["arcana"]);
    }
}
