use atlas_app_model::{
    EncounterConditionApplicabilityView, EncounterConditionAutomationLevelView,
    EncounterConditionCatalogView, EncounterConditionCategoryView,
    EncounterConditionDefinitionView,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub(super) enum ConditionRule {
    Blinded,
    Broken,
    Clumsy,
    Concealed,
    Confused,
    Controlled,
    Dazzled,
    Deafened,
    Doomed,
    Drained,
    Dying,
    Encumbered,
    Enfeebled,
    Fascinated,
    Fatigued,
    Fleeing,
    Friendly,
    Frightened,
    Grabbed,
    Helpful,
    Hidden,
    Hostile,
    Immobilized,
    Indifferent,
    Invisible,
    Observed,
    OffGuard,
    Paralyzed,
    PersistentDamage,
    Petrified,
    Prone,
    Quickened,
    Restrained,
    Sickened,
    Slowed,
    Stunned,
    Stupefied,
    Unconscious,
    Undetected,
    Unfriendly,
    Unnoticed,
    Wounded,
}

#[derive(Debug, Clone, Copy)]
pub(super) struct ModeledCondition {
    pub condition_ref: &'static str,
    pub name: &'static str,
    pub slug: &'static str,
    pub automation_level: EncounterConditionAutomationLevelView,
    pub applies_to: &'static [EncounterConditionApplicabilityView],
    pub categories: &'static [EncounterConditionCategoryView],
    pub has_value: bool,
    pub default_value: Option<i64>,
    pub rule: Option<ConditionRule>,
}

impl ModeledCondition {
    pub(super) fn scope_label(self) -> Option<&'static str> {
        match self.slug {
            "off-guard" => Some("Against"),
            "concealed" | "hidden" | "undetected" | "unnoticed" => Some("From"),
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

const MODELED_CONDITIONS: &[ModeledCondition] = &[
    automated(
        "conditionitems:XgEqL1kFApUbl5Z2",
        "Blinded",
        "blinded",
        false,
        None,
        CREATURES,
        DETECTION,
        ConditionRule::Blinded,
    ),
    tracked(
        "conditionitems:6dNUvdb1dhToNDj3",
        "Broken",
        "broken",
        false,
        None,
        &[AppliesTo::Hazard],
        OBJECT_STATE,
        ConditionRule::Broken,
    ),
    automated(
        "conditionitems:i3OJZU2nk64Df3xm",
        "Clumsy",
        "clumsy",
        true,
        Some(1),
        CREATURES_AND_HAZARDS,
        STAT_MODIFIER,
        ConditionRule::Clumsy,
    ),
    tracked(
        "conditionitems:DmAIPqOBomZ7H95W",
        "Concealed",
        "concealed",
        false,
        None,
        CREATURES,
        DETECTION,
        ConditionRule::Concealed,
    ),
    tracked(
        "conditionitems:yblD8fOR1J8rDwEQ",
        "Confused",
        "confused",
        false,
        None,
        CREATURES,
        RUNTIME_STATE,
        ConditionRule::Confused,
    ),
    tracked(
        "conditionitems:9qGBRpbX9NEwtAAr",
        "Controlled",
        "controlled",
        false,
        None,
        CREATURES,
        RUNTIME_STATE,
        ConditionRule::Controlled,
    ),
    tracked(
        "conditionitems:TkIyaNPgTZFBCCuh",
        "Dazzled",
        "dazzled",
        false,
        None,
        CREATURES,
        DETECTION,
        ConditionRule::Dazzled,
    ),
    tracked(
        "conditionitems:9PR9y0bi4JPKnHPR",
        "Deafened",
        "deafened",
        false,
        None,
        CREATURES,
        RUNTIME_STATE,
        ConditionRule::Deafened,
    ),
    tracked(
        "conditionitems:3uh1r86TzbQvosxv",
        "Doomed",
        "doomed",
        true,
        Some(1),
        CREATURES,
        DEATH_AND_DYING,
        ConditionRule::Doomed,
    ),
    tracked(
        "conditionitems:4D2KBtexWXa6oUMR",
        "Drained",
        "drained",
        true,
        Some(1),
        CREATURES,
        DEATH_AND_DYING,
        ConditionRule::Drained,
    ),
    tracked(
        "conditionitems:yZRUzMqrMmfLu0V1",
        "Dying",
        "dying",
        true,
        Some(1),
        CREATURES,
        DEATH_AND_DYING,
        ConditionRule::Dying,
    ),
    automated(
        "conditionitems:D5mg6Tc7Jzrj6ro7",
        "Encumbered",
        "encumbered",
        false,
        None,
        CREATURES,
        RUNTIME_STATE,
        ConditionRule::Encumbered,
    ),
    automated(
        "conditionitems:MIRkyAjyBeXivMa7",
        "Enfeebled",
        "enfeebled",
        true,
        Some(1),
        CREATURES_AND_HAZARDS,
        STAT_MODIFIER,
        ConditionRule::Enfeebled,
    ),
    tracked(
        "conditionitems:AdPVz7rbaVSRxHFg",
        "Fascinated",
        "fascinated",
        false,
        None,
        CREATURES,
        RUNTIME_STATE,
        ConditionRule::Fascinated,
    ),
    tracked(
        "conditionitems:HL2l2VRSaQHu9lUw",
        "Fatigued",
        "fatigued",
        false,
        None,
        CREATURES,
        RUNTIME_STATE,
        ConditionRule::Fatigued,
    ),
    tracked(
        "conditionitems:sDPxOjQ9kx2RZE8D",
        "Fleeing",
        "fleeing",
        false,
        None,
        CREATURES,
        RUNTIME_STATE,
        ConditionRule::Fleeing,
    ),
    tracked(
        "conditionitems:v66R7FdOf11l94im",
        "Friendly",
        "friendly",
        false,
        None,
        CREATURES,
        ATTITUDE,
        ConditionRule::Friendly,
    ),
    automated(
        "conditionitems:TBSHQspnbcqxsmjL",
        "Frightened",
        "frightened",
        true,
        Some(1),
        CREATURES_AND_HAZARDS,
        STAT_MODIFIER,
        ConditionRule::Frightened,
    ),
    automated(
        "conditionitems:kWc1fhmv9LBiTuei",
        "Grabbed",
        "grabbed",
        false,
        None,
        CREATURES,
        RUNTIME_STATE,
        ConditionRule::Grabbed,
    ),
    tracked(
        "conditionitems:v44P3WUcU1j0115l",
        "Helpful",
        "helpful",
        false,
        None,
        CREATURES,
        ATTITUDE,
        ConditionRule::Helpful,
    ),
    tracked(
        "conditionitems:iU0fEDdBp3rXpTMC",
        "Hidden",
        "hidden",
        false,
        None,
        CREATURES,
        DETECTION,
        ConditionRule::Hidden,
    ),
    tracked(
        "conditionitems:ud7gTLwPeklzYSXG",
        "Hostile",
        "hostile",
        false,
        None,
        CREATURES,
        ATTITUDE,
        ConditionRule::Hostile,
    ),
    automated(
        "conditionitems:eIcWbB5o3pP6OIMe",
        "Immobilized",
        "immobilized",
        false,
        None,
        CREATURES,
        RUNTIME_STATE,
        ConditionRule::Immobilized,
    ),
    tracked(
        "conditionitems:fuG8dgthlDWfWjIA",
        "Indifferent",
        "indifferent",
        false,
        None,
        CREATURES,
        ATTITUDE,
        ConditionRule::Indifferent,
    ),
    tracked(
        "conditionitems:zJxUflt9np0q4yML",
        "Invisible",
        "invisible",
        false,
        None,
        CREATURES,
        DETECTION,
        ConditionRule::Invisible,
    ),
    tracked(
        "conditionitems:1wQY3JYyhMYeeV2G",
        "Observed",
        "observed",
        false,
        None,
        CREATURES,
        DETECTION,
        ConditionRule::Observed,
    ),
    automated(
        "conditionitems:AJh5ex99aV6VTggg",
        "Off-Guard",
        "off-guard",
        false,
        None,
        CREATURES_AND_HAZARDS,
        STAT_MODIFIER,
        ConditionRule::OffGuard,
    ),
    tracked(
        "conditionitems:6uEgoh53GbXuHpTF",
        "Paralyzed",
        "paralyzed",
        false,
        None,
        CREATURES,
        RUNTIME_STATE,
        ConditionRule::Paralyzed,
    ),
    tracked(
        "conditionitems:lDVqvLKA6eF3Df60",
        "Persistent Damage",
        "persistent-damage",
        false,
        None,
        CREATURES_AND_HAZARDS,
        RUNTIME_STATE,
        ConditionRule::PersistentDamage,
    ),
    tracked(
        "conditionitems:dTwPJuKgBQCMxixg",
        "Petrified",
        "petrified",
        false,
        None,
        CREATURES_AND_HAZARDS,
        RUNTIME_STATE,
        ConditionRule::Petrified,
    ),
    automated(
        "conditionitems:j91X7x0XSomq8d60",
        "Prone",
        "prone",
        false,
        None,
        CREATURES,
        RUNTIME_STATE,
        ConditionRule::Prone,
    ),
    automated(
        "conditionitems:nlCjDvLMf2EkV2dl",
        "Quickened",
        "quickened",
        false,
        None,
        CREATURES,
        ACTION_ECONOMY,
        ConditionRule::Quickened,
    ),
    automated(
        "conditionitems:VcDeM8A5oI6VqhbM",
        "Restrained",
        "restrained",
        false,
        None,
        CREATURES,
        RUNTIME_STATE,
        ConditionRule::Restrained,
    ),
    automated(
        "conditionitems:fesd1n5eVhpCSS18",
        "Sickened",
        "sickened",
        true,
        Some(1),
        CREATURES_AND_HAZARDS,
        STAT_MODIFIER,
        ConditionRule::Sickened,
    ),
    automated(
        "conditionitems:xYTAsEpcJE1Ccni3",
        "Slowed",
        "slowed",
        true,
        Some(1),
        CREATURES,
        ACTION_ECONOMY,
        ConditionRule::Slowed,
    ),
    automated(
        "conditionitems:dfCMdR4wnpbYNTix",
        "Stunned",
        "stunned",
        true,
        Some(1),
        CREATURES,
        ACTION_ECONOMY,
        ConditionRule::Stunned,
    ),
    automated(
        "conditionitems:e1XGnhKNSQIm5IXg",
        "Stupefied",
        "stupefied",
        true,
        Some(1),
        CREATURES_AND_HAZARDS,
        STAT_MODIFIER,
        ConditionRule::Stupefied,
    ),
    tracked(
        "conditionitems:fBnFDH2MTzgFijKf",
        "Unconscious",
        "unconscious",
        false,
        None,
        CREATURES,
        DEATH_AND_DYING,
        ConditionRule::Unconscious,
    ),
    tracked(
        "conditionitems:VRSef5y1LmL2Hkjf",
        "Undetected",
        "undetected",
        false,
        None,
        CREATURES,
        DETECTION,
        ConditionRule::Undetected,
    ),
    tracked(
        "conditionitems:I1ffBVISxLr2gC4u",
        "Unfriendly",
        "unfriendly",
        false,
        None,
        CREATURES,
        ATTITUDE,
        ConditionRule::Unfriendly,
    ),
    tracked(
        "conditionitems:9evPzg9E6muFcoSk",
        "Unnoticed",
        "unnoticed",
        false,
        None,
        CREATURES,
        DETECTION,
        ConditionRule::Unnoticed,
    ),
    tracked(
        "conditionitems:Yl48xTdMh3aeQYL2",
        "Wounded",
        "wounded",
        true,
        Some(1),
        CREATURES,
        DEATH_AND_DYING,
        ConditionRule::Wounded,
    ),
];

#[expect(
    clippy::too_many_arguments,
    reason = "Catalog constructors name independent condition definition fields"
)]
const fn tracked(
    condition_ref: &'static str,
    name: &'static str,
    slug: &'static str,
    has_value: bool,
    default_value: Option<i64>,
    applies_to: &'static [AppliesTo],
    categories: &'static [Category],
    rule: ConditionRule,
) -> ModeledCondition {
    ModeledCondition {
        condition_ref,
        name,
        slug,
        automation_level: Automation::Tracked,
        applies_to,
        categories,
        has_value,
        default_value,
        rule: Some(rule),
    }
}

#[expect(
    clippy::too_many_arguments,
    reason = "Catalog constructors name independent condition definition fields"
)]
const fn automated(
    condition_ref: &'static str,
    name: &'static str,
    slug: &'static str,
    has_value: bool,
    default_value: Option<i64>,
    applies_to: &'static [AppliesTo],
    categories: &'static [Category],
    rule: ConditionRule,
) -> ModeledCondition {
    ModeledCondition {
        condition_ref,
        name,
        slug,
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
        standard_persistent_recovery_dc: super::rules::persistent_damage::STANDARD_RECOVERY_DC,
        damage_types: [
            "acid",
            "bleed",
            "bludgeoning",
            "cold",
            "electricity",
            "fire",
            "force",
            "mental",
            "piercing",
            "poison",
            "slashing",
            "sonic",
            "spirit",
            "untyped",
            "vitality",
            "void",
        ]
        .into_iter()
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
        conditions: MODELED_CONDITIONS
            .iter()
            .map(|condition| EncounterConditionDefinitionView {
                editor_kind: match condition.rule {
                    Some(
                        ConditionRule::Helpful
                        | ConditionRule::Friendly
                        | ConditionRule::Indifferent
                        | ConditionRule::Unfriendly
                        | ConditionRule::Hostile,
                    ) => atlas_app_model::ConditionEditorKindView::Attitude,
                    Some(ConditionRule::PersistentDamage) => {
                        atlas_app_model::ConditionEditorKindView::PersistentDamage
                    }
                    Some(ConditionRule::Stunned) => {
                        atlas_app_model::ConditionEditorKindView::Stunned
                    }
                    Some(ConditionRule::Sickened) => {
                        atlas_app_model::ConditionEditorKindView::Sickened
                    }
                    Some(ConditionRule::Unconscious) => {
                        atlas_app_model::ConditionEditorKindView::Unconscious
                    }
                    Some(ConditionRule::Frightened) => {
                        atlas_app_model::ConditionEditorKindView::Frightened
                    }
                    Some(ConditionRule::Fleeing) => {
                        atlas_app_model::ConditionEditorKindView::Fleeing
                    }
                    Some(ConditionRule::Fascinated) => {
                        atlas_app_model::ConditionEditorKindView::Fascinated
                    }
                    Some(ConditionRule::Broken) => atlas_app_model::ConditionEditorKindView::Broken,
                    _ => atlas_app_model::ConditionEditorKindView::Ordinary,
                },
                scope_label: condition.scope_label().map(str::to_owned),
                supports_extra_action: condition.rule == Some(ConditionRule::Quickened),
                eligibility: atlas_app_model::EncounterConditionEligibilityView::Available,
                condition_ref: condition.condition_ref.to_string(),
                name: condition.name.to_string(),
                automation_level: condition.automation_level,
                applies_to: condition.applies_to.to_vec(),
                categories: condition.categories.to_vec(),
                supports_duration_instead_of_value: condition.rule == Some(ConditionRule::Stunned),
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

pub(super) fn modeled_condition_by_rule(rule: ConditionRule) -> Option<ModeledCondition> {
    MODELED_CONDITIONS
        .iter()
        .copied()
        .find(|condition| condition.rule == Some(rule))
}
