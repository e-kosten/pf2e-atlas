//! Consumer-required borrowed authored facts; no runtime defaults or rule execution.
use super::{ActorQueryView, ItemSourceView, SourceFieldView};
use atlas_foundry_model::generated::*;
use serde_json::Number;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SourceNpcAdjustment {
    Normal,
    Elite,
    Weak,
}

impl<'a> ActorQueryView<'a> {
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
    pub fn visit_skills(
        self,
        mut visit: impl FnMut(&'static str, SourceFieldView<'a, &'a NPCSkillSource>),
    ) -> SourceFieldView<'a, ()> {
        self.npc_skills().map(|s| {
            for (key, value) in [
                ("acrobatics", &s.acrobatics),
                ("arcana", &s.arcana),
                ("athletics", &s.athletics),
                ("crafting", &s.crafting),
                ("deception", &s.deception),
                ("diplomacy", &s.diplomacy),
                ("intimidation", &s.intimidation),
                ("medicine", &s.medicine),
                ("nature", &s.nature),
                ("occultism", &s.occultism),
                ("performance", &s.performance),
                ("religion", &s.religion),
                ("society", &s.society),
                ("stealth", &s.stealth),
                ("survival", &s.survival),
                ("thievery", &s.thievery),
            ] {
                visit(key, value.into());
            }
        })
    }
}
impl<'a> ItemSourceView<'a> {
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
        view.visit_skills(|k, v| {
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
    }
}
