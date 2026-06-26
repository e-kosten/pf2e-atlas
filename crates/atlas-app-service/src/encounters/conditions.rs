use atlas_app_model::{
    EncounterConditionApplicabilityView, EncounterConditionAutomationLevelView,
    EncounterConditionCatalogView, EncounterConditionCategoryView,
    EncounterConditionDefinitionView,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum ConditionRule {
    Frightened,
    Sickened,
    OffGuard,
    Clumsy,
    Enfeebled,
    Stupefied,
    Slowed,
    Quickened,
    Stunned,
    Immobilized,
    Grabbed,
    Restrained,
    Encumbered,
    Prone,
}

#[derive(Debug, Clone, Copy)]
pub(super) struct ModeledCondition {
    pub condition_ref: &'static str,
    pub name: &'static str,
    pub automation_level: EncounterConditionAutomationLevelView,
    pub applies_to: &'static [EncounterConditionApplicabilityView],
    pub categories: &'static [EncounterConditionCategoryView],
    pub has_value: bool,
    pub default_value: Option<i64>,
    pub rule: Option<ConditionRule>,
}

use EncounterConditionApplicabilityView as AppliesTo;
use EncounterConditionAutomationLevelView as Automation;
use EncounterConditionCategoryView as Category;

const CREATURES: &[AppliesTo] = &[AppliesTo::Creature];
const CREATURES_AND_HAZARDS: &[AppliesTo] = &[AppliesTo::Creature, AppliesTo::Hazard];
const OBJECTS: &[AppliesTo] = &[AppliesTo::Object];
const STAT_MODIFIER: &[Category] = &[Category::StatModifier];
const DETECTION: &[Category] = &[Category::Detection];
const ATTITUDE: &[Category] = &[Category::Attitude];
const DEATH_AND_DYING: &[Category] = &[Category::DeathAndDying];
const ACTION_ECONOMY: &[Category] = &[Category::ActionEconomy];
const OBJECT_STATE: &[Category] = &[Category::ObjectState];
const RUNTIME_STATE: &[Category] = &[Category::RuntimeState];

const MODELED_CONDITIONS: &[ModeledCondition] = &[
    tracked(
        "conditionitems:XgEqL1kFApUbl5Z2",
        "Blinded",
        false,
        None,
        CREATURES,
        DETECTION,
    ),
    tracked(
        "conditionitems:6dNUvdb1dhToNDj3",
        "Broken",
        false,
        None,
        OBJECTS,
        OBJECT_STATE,
    ),
    automated(
        "conditionitems:i3OJZU2nk64Df3xm",
        "Clumsy",
        true,
        Some(1),
        CREATURES_AND_HAZARDS,
        STAT_MODIFIER,
        ConditionRule::Clumsy,
    ),
    tracked(
        "conditionitems:DmAIPqOBomZ7H95W",
        "Concealed",
        false,
        None,
        CREATURES,
        DETECTION,
    ),
    tracked(
        "conditionitems:yblD8fOR1J8rDwEQ",
        "Confused",
        false,
        None,
        CREATURES,
        RUNTIME_STATE,
    ),
    tracked(
        "conditionitems:9qGBRpbX9NEwtAAr",
        "Controlled",
        false,
        None,
        CREATURES,
        RUNTIME_STATE,
    ),
    tracked(
        "conditionitems:zXZjC8HLaRoLR17U",
        "Cursebound",
        true,
        Some(1),
        CREATURES,
        RUNTIME_STATE,
    ),
    tracked(
        "conditionitems:TkIyaNPgTZFBCCuh",
        "Dazzled",
        false,
        None,
        CREATURES,
        DETECTION,
    ),
    tracked(
        "conditionitems:9PR9y0bi4JPKnHPR",
        "Deafened",
        false,
        None,
        CREATURES,
        RUNTIME_STATE,
    ),
    tracked(
        "conditionitems:3uh1r86TzbQvosxv",
        "Doomed",
        true,
        Some(1),
        CREATURES,
        DEATH_AND_DYING,
    ),
    tracked(
        "conditionitems:4D2KBtexWXa6oUMR",
        "Drained",
        true,
        Some(1),
        CREATURES,
        DEATH_AND_DYING,
    ),
    tracked(
        "conditionitems:yZRUzMqrMmfLu0V1",
        "Dying",
        true,
        Some(1),
        CREATURES,
        DEATH_AND_DYING,
    ),
    automated(
        "conditionitems:D5mg6Tc7Jzrj6ro7",
        "Encumbered",
        false,
        None,
        CREATURES,
        RUNTIME_STATE,
        ConditionRule::Encumbered,
    ),
    automated(
        "conditionitems:MIRkyAjyBeXivMa7",
        "Enfeebled",
        true,
        Some(1),
        CREATURES_AND_HAZARDS,
        STAT_MODIFIER,
        ConditionRule::Enfeebled,
    ),
    tracked(
        "conditionitems:AdPVz7rbaVSRxHFg",
        "Fascinated",
        false,
        None,
        CREATURES,
        RUNTIME_STATE,
    ),
    tracked(
        "conditionitems:HL2l2VRSaQHu9lUw",
        "Fatigued",
        false,
        None,
        CREATURES,
        RUNTIME_STATE,
    ),
    tracked(
        "conditionitems:sDPxOjQ9kx2RZE8D",
        "Fleeing",
        false,
        None,
        CREATURES,
        RUNTIME_STATE,
    ),
    tracked(
        "conditionitems:v66R7FdOf11l94im",
        "Friendly",
        false,
        None,
        CREATURES,
        ATTITUDE,
    ),
    automated(
        "conditionitems:TBSHQspnbcqxsmjL",
        "Frightened",
        true,
        Some(1),
        CREATURES_AND_HAZARDS,
        STAT_MODIFIER,
        ConditionRule::Frightened,
    ),
    automated(
        "conditionitems:kWc1fhmv9LBiTuei",
        "Grabbed",
        false,
        None,
        CREATURES,
        RUNTIME_STATE,
        ConditionRule::Grabbed,
    ),
    tracked(
        "conditionitems:v44P3WUcU1j0115l",
        "Helpful",
        false,
        None,
        CREATURES,
        ATTITUDE,
    ),
    tracked(
        "conditionitems:iU0fEDdBp3rXpTMC",
        "Hidden",
        false,
        None,
        CREATURES,
        DETECTION,
    ),
    tracked(
        "conditionitems:ud7gTLwPeklzYSXG",
        "Hostile",
        false,
        None,
        CREATURES,
        ATTITUDE,
    ),
    automated(
        "conditionitems:eIcWbB5o3pP6OIMe",
        "Immobilized",
        false,
        None,
        CREATURES,
        RUNTIME_STATE,
        ConditionRule::Immobilized,
    ),
    tracked(
        "conditionitems:fuG8dgthlDWfWjIA",
        "Indifferent",
        false,
        None,
        CREATURES,
        ATTITUDE,
    ),
    tracked(
        "conditionitems:zJxUflt9np0q4yML",
        "Invisible",
        false,
        None,
        CREATURES,
        DETECTION,
    ),
    tracked(
        "conditionitems:1wQY3JYyhMYeeV2G",
        "Observed",
        false,
        None,
        CREATURES,
        DETECTION,
    ),
    automated(
        "conditionitems:AJh5ex99aV6VTggg",
        "Off-Guard",
        false,
        None,
        CREATURES_AND_HAZARDS,
        STAT_MODIFIER,
        ConditionRule::OffGuard,
    ),
    tracked(
        "conditionitems:6uEgoh53GbXuHpTF",
        "Paralyzed",
        false,
        None,
        CREATURES,
        RUNTIME_STATE,
    ),
    tracked(
        "conditionitems:lDVqvLKA6eF3Df60",
        "Persistent Damage",
        false,
        None,
        CREATURES_AND_HAZARDS,
        RUNTIME_STATE,
    ),
    tracked(
        "conditionitems:dTwPJuKgBQCMxixg",
        "Petrified",
        false,
        None,
        CREATURES_AND_HAZARDS,
        RUNTIME_STATE,
    ),
    automated(
        "conditionitems:j91X7x0XSomq8d60",
        "Prone",
        false,
        None,
        CREATURES,
        RUNTIME_STATE,
        ConditionRule::Prone,
    ),
    automated(
        "conditionitems:nlCjDvLMf2EkV2dl",
        "Quickened",
        false,
        None,
        CREATURES,
        ACTION_ECONOMY,
        ConditionRule::Quickened,
    ),
    automated(
        "conditionitems:VcDeM8A5oI6VqhbM",
        "Restrained",
        false,
        None,
        CREATURES,
        RUNTIME_STATE,
        ConditionRule::Restrained,
    ),
    automated(
        "conditionitems:fesd1n5eVhpCSS18",
        "Sickened",
        true,
        Some(1),
        CREATURES_AND_HAZARDS,
        STAT_MODIFIER,
        ConditionRule::Sickened,
    ),
    automated(
        "conditionitems:xYTAsEpcJE1Ccni3",
        "Slowed",
        true,
        Some(1),
        CREATURES,
        ACTION_ECONOMY,
        ConditionRule::Slowed,
    ),
    automated(
        "conditionitems:dfCMdR4wnpbYNTix",
        "Stunned",
        true,
        Some(1),
        CREATURES,
        ACTION_ECONOMY,
        ConditionRule::Stunned,
    ),
    automated(
        "conditionitems:e1XGnhKNSQIm5IXg",
        "Stupefied",
        true,
        Some(1),
        CREATURES_AND_HAZARDS,
        STAT_MODIFIER,
        ConditionRule::Stupefied,
    ),
    tracked(
        "conditionitems:fBnFDH2MTzgFijKf",
        "Unconscious",
        false,
        None,
        CREATURES,
        DEATH_AND_DYING,
    ),
    tracked(
        "conditionitems:VRSef5y1LmL2Hkjf",
        "Undetected",
        false,
        None,
        CREATURES,
        DETECTION,
    ),
    tracked(
        "conditionitems:I1ffBVISxLr2gC4u",
        "Unfriendly",
        false,
        None,
        CREATURES,
        ATTITUDE,
    ),
    tracked(
        "conditionitems:9evPzg9E6muFcoSk",
        "Unnoticed",
        false,
        None,
        CREATURES,
        DETECTION,
    ),
    tracked(
        "conditionitems:Yl48xTdMh3aeQYL2",
        "Wounded",
        true,
        Some(1),
        CREATURES,
        DEATH_AND_DYING,
    ),
];

const fn tracked(
    condition_ref: &'static str,
    name: &'static str,
    has_value: bool,
    default_value: Option<i64>,
    applies_to: &'static [AppliesTo],
    categories: &'static [Category],
) -> ModeledCondition {
    ModeledCondition {
        condition_ref,
        name,
        automation_level: Automation::Tracked,
        applies_to,
        categories,
        has_value,
        default_value,
        rule: None,
    }
}

const fn automated(
    condition_ref: &'static str,
    name: &'static str,
    has_value: bool,
    default_value: Option<i64>,
    applies_to: &'static [AppliesTo],
    categories: &'static [Category],
    rule: ConditionRule,
) -> ModeledCondition {
    ModeledCondition {
        condition_ref,
        name,
        automation_level: Automation::Automated,
        applies_to,
        categories,
        has_value,
        default_value,
        rule: Some(rule),
    }
}

pub(super) fn condition_catalog() -> EncounterConditionCatalogView {
    EncounterConditionCatalogView {
        conditions: MODELED_CONDITIONS
            .iter()
            .map(|condition| EncounterConditionDefinitionView {
                condition_ref: condition.condition_ref.to_string(),
                name: condition.name.to_string(),
                automation_level: condition.automation_level,
                applies_to: condition.applies_to.to_vec(),
                categories: condition.categories.to_vec(),
                has_value: condition.has_value,
                default_value: condition.default_value,
            })
            .collect(),
    }
}

pub(super) fn modeled_condition_by_ref(condition_ref: &str) -> Option<ModeledCondition> {
    MODELED_CONDITIONS
        .iter()
        .copied()
        .find(|condition| condition.condition_ref == condition_ref)
}

pub(super) fn condition_rule_for_key(condition_key: Option<&str>) -> Option<ConditionRule> {
    let condition_key = condition_key?;
    modeled_condition_by_ref(condition_key).and_then(|condition| condition.rule)
}
