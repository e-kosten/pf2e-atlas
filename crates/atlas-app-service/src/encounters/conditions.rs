use atlas_app_model::{
    EncounterConditionApplicabilityView, EncounterConditionAutomationLevelView,
    EncounterConditionCatalogView, EncounterConditionCategoryView,
    EncounterConditionDefinitionView,
};

use atlas_foundry_model::generated::condition_definitions::CONDITION_DEFINITIONS;
pub(super) use atlas_foundry_model::generated::condition_definitions::ConditionId as ConditionRule;

#[derive(Debug, Clone, Copy)]
pub(super) struct ModeledCondition {
    pub condition_ref: &'static str,
    pub name: &'static str,
    pub automation_level: EncounterConditionAutomationLevelView,
    pub applies_to: &'static [EncounterConditionApplicabilityView],
    pub categories: &'static [EncounterConditionCategoryView],
    pub has_value: bool,
    pub default_value: Option<i64>,
    pub rule: ConditionRule,
}

impl ModeledCondition {
    pub(super) fn scope_label(self) -> Option<&'static str> {
        match self.rule {
            ConditionRule::OffGuard => Some("Against"),
            ConditionRule::Concealed
            | ConditionRule::Hidden
            | ConditionRule::Undetected
            | ConditionRule::Unnoticed => Some("From"),
            _ => None,
        }
    }
}

use EncounterConditionApplicabilityView as AppliesTo;
use EncounterConditionAutomationLevelView as Automation;
use EncounterConditionCategoryView as Category;

const CREATURES: &[AppliesTo] = &[AppliesTo::Creature];
const CREATURES_AND_HAZARDS: &[AppliesTo] = &[AppliesTo::Creature, AppliesTo::Hazard];
const STAT_MODIFIER: &[Category] = &[Category::StatModifier];
const DETECTION: &[Category] = &[Category::Detection];
const ATTITUDE: &[Category] = &[Category::Attitude];
const DEATH_AND_DYING: &[Category] = &[Category::DeathAndDying];
const ACTION_ECONOMY: &[Category] = &[Category::ActionEconomy];
const OBJECT_STATE: &[Category] = &[Category::ObjectState];
const RUNTIME_STATE: &[Category] = &[Category::RuntimeState];

fn condition_policy(
    rule: ConditionRule,
) -> Option<(Automation, &'static [AppliesTo], &'static [Category])> {
    use ConditionRule::*;
    Some(match rule {
        Cursebound => return None,
        Blinded => (Automation::Automated, CREATURES, DETECTION),
        Broken => (Automation::Tracked, &[AppliesTo::Hazard], OBJECT_STATE),
        Clumsy | Enfeebled | Frightened | OffGuard | Sickened | Stupefied => {
            (Automation::Automated, CREATURES_AND_HAZARDS, STAT_MODIFIER)
        }
        Concealed | Dazzled | Hidden | Invisible | Observed | Undetected | Unnoticed => {
            (Automation::Tracked, CREATURES, DETECTION)
        }
        Confused | Controlled | Deafened | Fascinated | Fatigued | Fleeing | Paralyzed => {
            (Automation::Tracked, CREATURES, RUNTIME_STATE)
        }
        Doomed | Drained | Dying | Unconscious | Wounded => {
            (Automation::Tracked, CREATURES, DEATH_AND_DYING)
        }
        Encumbered | Grabbed | Immobilized | Prone | Restrained => {
            (Automation::Automated, CREATURES, RUNTIME_STATE)
        }
        Friendly | Helpful | Hostile | Indifferent | Unfriendly => {
            (Automation::Tracked, CREATURES, ATTITUDE)
        }
        PersistentDamage | Petrified => (Automation::Tracked, CREATURES_AND_HAZARDS, RUNTIME_STATE),
        Quickened | Slowed | Stunned => (Automation::Automated, CREATURES, ACTION_ECONOMY),
    })
}

pub(super) fn condition_catalog() -> EncounterConditionCatalogView {
    EncounterConditionCatalogView {
        standard_persistent_recovery_dc: super::rules::persistent_damage::STANDARD_RECOVERY_DC,
        damage_types: atlas_foundry_model::generated::ModularConfigDamageType::ALL
            .iter()
            .map(|kind| kind.as_str())
            .map(|value| atlas_app_model::EncounterDamageTypeView {
                value: value.into(),
                label: value
                    .chars()
                    .enumerate()
                    .map(|(index, character)| {
                        if index == 0 {
                            character.to_ascii_uppercase()
                        } else {
                            character
                        }
                    })
                    .collect(),
            })
            .collect(),
        conditions: CONDITION_DEFINITIONS
            .iter()
            .filter_map(|definition| modeled_condition_by_rule(definition.id))
            .map(|condition| EncounterConditionDefinitionView {
                editor_kind: match condition.rule {
                    ConditionRule::Helpful
                    | ConditionRule::Friendly
                    | ConditionRule::Indifferent
                    | ConditionRule::Unfriendly
                    | ConditionRule::Hostile => atlas_app_model::ConditionEditorKindView::Attitude,
                    ConditionRule::PersistentDamage => {
                        atlas_app_model::ConditionEditorKindView::PersistentDamage
                    }
                    ConditionRule::Stunned => atlas_app_model::ConditionEditorKindView::Stunned,
                    ConditionRule::Sickened => atlas_app_model::ConditionEditorKindView::Sickened,
                    ConditionRule::Unconscious => {
                        atlas_app_model::ConditionEditorKindView::Unconscious
                    }
                    ConditionRule::Frightened => {
                        atlas_app_model::ConditionEditorKindView::Frightened
                    }
                    ConditionRule::Fleeing => atlas_app_model::ConditionEditorKindView::Fleeing,
                    ConditionRule::Fascinated => {
                        atlas_app_model::ConditionEditorKindView::Fascinated
                    }
                    ConditionRule::Broken => atlas_app_model::ConditionEditorKindView::Broken,
                    _ => atlas_app_model::ConditionEditorKindView::Ordinary,
                },
                scope_label: condition.scope_label().map(str::to_owned),
                supports_extra_action: condition.rule == ConditionRule::Quickened,
                eligibility: atlas_app_model::EncounterConditionEligibilityView::Available,
                condition_ref: condition.condition_ref.to_string(),
                name: condition.name.to_string(),
                automation_level: condition.automation_level,
                applies_to: condition.applies_to.to_vec(),
                categories: condition.categories.to_vec(),
                supports_duration_instead_of_value: condition.rule == ConditionRule::Stunned,
                has_value: condition.has_value,
                default_value: condition.default_value,
            })
            .collect(),
    }
}

pub(super) fn modeled_condition_by_ref(condition_ref: &str) -> Option<ModeledCondition> {
    modeled_condition_by_rule(ConditionRule::from_record_key(condition_ref)?)
}

pub(super) fn condition_rule_for_key(condition_key: Option<&str>) -> Option<ConditionRule> {
    modeled_condition_by_ref(condition_key?).map(|condition| condition.rule)
}

pub(super) fn modeled_condition_by_rule(rule: ConditionRule) -> Option<ModeledCondition> {
    let (automation_level, applies_to, categories) = condition_policy(rule)?;
    let definition = rule.definition();
    Some(ModeledCondition {
        condition_ref: definition.record_key,
        name: definition.name,
        has_value: definition.has_value,
        default_value: definition.default_value,
        automation_level,
        applies_to,
        categories,
        rule,
    })
}
