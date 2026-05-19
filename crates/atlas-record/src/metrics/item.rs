use atlas_domain::{MetricDomain, MetricValueType};

use super::model::{MetricDefinition, MetricGroup, MetricLabelTemplate, static_definition};

pub mod armor {
    use super::*;

    pub const AC_BONUS: MetricDefinition = static_definition(
        MetricDomain::Item,
        "armor.ac_bonus",
        MetricValueType::Number,
        "armor",
        MetricLabelTemplate::Static("Armor AC bonus"),
        Some(MetricLabelTemplate::Static("AC bonus")),
        MetricGroup::Items,
    );
    pub const DEX_CAP: MetricDefinition = static_definition(
        MetricDomain::Item,
        "armor.dex_cap",
        MetricValueType::Number,
        "armor",
        MetricLabelTemplate::Static("Armor Dexterity cap"),
        Some(MetricLabelTemplate::Static("Dex cap")),
        MetricGroup::Items,
    );
    pub const STRENGTH: MetricDefinition = static_definition(
        MetricDomain::Item,
        "armor.strength",
        MetricValueType::Number,
        "armor",
        MetricLabelTemplate::Static("Armor Strength requirement"),
        Some(MetricLabelTemplate::Static("Strength")),
        MetricGroup::Items,
    );
    pub const CHECK_PENALTY: MetricDefinition = static_definition(
        MetricDomain::Item,
        "armor.check_penalty",
        MetricValueType::Number,
        "armor",
        MetricLabelTemplate::Static("Armor check penalty"),
        Some(MetricLabelTemplate::Static("Check penalty")),
        MetricGroup::Items,
    );
    pub const SPEED_PENALTY: MetricDefinition = static_definition(
        MetricDomain::Item,
        "armor.speed_penalty",
        MetricValueType::Number,
        "armor",
        MetricLabelTemplate::Static("Armor Speed penalty"),
        Some(MetricLabelTemplate::Static("Speed penalty")),
        MetricGroup::Items,
    );
}

pub mod shield {
    use super::*;

    pub const AC_BONUS: MetricDefinition = static_definition(
        MetricDomain::Item,
        "shield.ac_bonus",
        MetricValueType::Number,
        "shield",
        MetricLabelTemplate::Static("Shield AC bonus"),
        Some(MetricLabelTemplate::Static("AC bonus")),
        MetricGroup::Items,
    );
    pub const HARDNESS: MetricDefinition = static_definition(
        MetricDomain::Item,
        "shield.hardness",
        MetricValueType::Number,
        "shield",
        MetricLabelTemplate::Static("Shield Hardness"),
        Some(MetricLabelTemplate::Static("Hardness")),
        MetricGroup::Items,
    );
    pub const HP: MetricDefinition = static_definition(
        MetricDomain::Item,
        "shield.hp",
        MetricValueType::Number,
        "shield",
        MetricLabelTemplate::Static("Shield Hit Points"),
        Some(MetricLabelTemplate::Static("HP")),
        MetricGroup::Items,
    );
    pub const BROKEN_THRESHOLD: MetricDefinition = static_definition(
        MetricDomain::Item,
        "shield.bt",
        MetricValueType::Number,
        "shield",
        MetricLabelTemplate::Static("Shield Broken Threshold"),
        Some(MetricLabelTemplate::Static("BT")),
        MetricGroup::Items,
    );
}

pub mod weapon {
    use super::*;

    pub const RANGE_INCREMENT: MetricDefinition = static_definition(
        MetricDomain::Item,
        "weapon.range_increment",
        MetricValueType::Number,
        "weapon",
        MetricLabelTemplate::Static("Weapon range increment"),
        Some(MetricLabelTemplate::Static("Range")),
        MetricGroup::Items,
    );
    pub const RELOAD: MetricDefinition = static_definition(
        MetricDomain::Item,
        "weapon.reload",
        MetricValueType::Number,
        "weapon",
        MetricLabelTemplate::Static("Weapon reload"),
        Some(MetricLabelTemplate::Static("Reload")),
        MetricGroup::Items,
    );
    pub const DAMAGE_DICE: MetricDefinition = static_definition(
        MetricDomain::Item,
        "weapon.damage_dice",
        MetricValueType::Number,
        "weapon",
        MetricLabelTemplate::Static("Weapon damage dice"),
        Some(MetricLabelTemplate::Static("Damage dice")),
        MetricGroup::Items,
    );
    pub const DAMAGE_DIE_FACES: MetricDefinition = static_definition(
        MetricDomain::Item,
        "weapon.damage_die_faces",
        MetricValueType::Number,
        "weapon",
        MetricLabelTemplate::Static("Weapon damage die faces"),
        Some(MetricLabelTemplate::Static("Damage die")),
        MetricGroup::Items,
    );
}
