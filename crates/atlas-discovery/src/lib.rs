use atlas_domain::{
    FilterFieldGroup, FilterFieldInfo, FilterFieldType, FilterOperator, FilterValuePolicy,
    FilterValueSort,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DiscoveryFieldDefinition {
    pub field: &'static str,
    pub field_type: FilterFieldType,
    pub group: FilterFieldGroup,
    pub value_policy: FilterValuePolicy,
    pub value_sql: &'static str,
    pub operators: &'static [FilterOperator],
    pub cli_flags: &'static [&'static str],
    pub applicable_families: &'static [&'static str],
    pub default_sort: FilterValueSort,
    pub policy_reason: &'static str,
}

impl DiscoveryFieldDefinition {
    pub fn info(self, catalog_available: bool) -> FilterFieldInfo {
        FilterFieldInfo {
            field: self.field.to_string(),
            field_type: self.field_type,
            group: self.group,
            value_policy: self.value_policy,
            operators: self.operators.to_vec(),
            applicable_families: self
                .applicable_families
                .iter()
                .map(|value| (*value).to_string())
                .collect(),
            cli_flags: self
                .cli_flags
                .iter()
                .map(|value| (*value).to_string())
                .collect(),
            catalog_available,
        }
    }
}

const SET_OPERATORS: &[FilterOperator] = &[
    FilterOperator::Includes,
    FilterOperator::IsNull,
    FilterOperator::IsNotNull,
];
const STRING_OPERATORS: &[FilterOperator] = &[
    FilterOperator::Eq,
    FilterOperator::NotEq,
    FilterOperator::IsNull,
    FilterOperator::IsNotNull,
];
const TEXT_OPERATORS: &[FilterOperator] = &[
    FilterOperator::Eq,
    FilterOperator::NotEq,
    FilterOperator::Contains,
    FilterOperator::NotContains,
    FilterOperator::IsNull,
    FilterOperator::IsNotNull,
];
const NUMBER_OPERATORS: &[FilterOperator] = &[
    FilterOperator::Eq,
    FilterOperator::Gt,
    FilterOperator::Gte,
    FilterOperator::Lt,
    FilterOperator::Lte,
    FilterOperator::Between,
    FilterOperator::IsNull,
    FilterOperator::IsNotNull,
];
const BOOLEAN_OPERATORS: &[FilterOperator] = &[
    FilterOperator::Eq,
    FilterOperator::IsNull,
    FilterOperator::IsNotNull,
];

pub const DISCOVERY_ALL_FAMILIES: &[&str] = &[
    "creature",
    "character",
    "companion",
    "army",
    "hazard",
    "vehicle",
    "equipment",
    "feat",
    "spell",
    "affliction",
    "rule",
    "character_option",
    "lore",
    "tooling",
    "campaign_feature",
];
const SPELL_FAMILY: &[&str] = &["spell"];
const ACTOR_FAMILIES: &[&str] = &[
    "army",
    "character",
    "companion",
    "creature",
    "hazard",
    "vehicle",
];
const ITEM_FAMILIES: &[&str] = &["equipment"];

macro_rules! field {
    ($field:literal, $field_type:ident, $group:ident, $policy:ident, $sql:literal, $operators:ident, [$($flag:literal),*], $families:expr, $sort:ident) => {
        DiscoveryFieldDefinition {
            field: $field,
            field_type: FilterFieldType::$field_type,
            group: FilterFieldGroup::$group,
            value_policy: FilterValuePolicy::$policy,
            value_sql: $sql,
            operators: $operators,
            cli_flags: &[$($flag),*],
            applicable_families: $families,
            default_sort: FilterValueSort::$sort,
            policy_reason: stringify!($policy),
        }
    };
}

pub const DISCOVERY_FIELD_DEFINITIONS: &[DiscoveryFieldDefinition] = &[
    field!(
        "record_family",
        EnumString,
        Record,
        Enumerable,
        "SELECT record_key, record_family FROM records",
        STRING_OPERATORS,
        ["--family"],
        DISCOVERY_ALL_FAMILIES,
        Canonical
    ),
    field!(
        "pack_name",
        EnumString,
        Record,
        Enumerable,
        "SELECT record_key, pack_name FROM records",
        STRING_OPERATORS,
        ["--pack-name"],
        DISCOVERY_ALL_FAMILIES,
        Count
    ),
    field!(
        "pack_label",
        EnumString,
        Record,
        Enumerable,
        "SELECT record_key, pack_label FROM records",
        STRING_OPERATORS,
        ["--pack-label"],
        DISCOVERY_ALL_FAMILIES,
        Count
    ),
    field!(
        "foundry_record_type",
        EnumString,
        Record,
        Enumerable,
        "SELECT record_key, foundry_record_type FROM records",
        STRING_OPERATORS,
        [],
        DISCOVERY_ALL_FAMILIES,
        Count
    ),
    field!(
        "publication_title",
        Text,
        Record,
        Enumerable,
        "SELECT record_key, publication_title FROM records",
        STRING_OPERATORS,
        ["--publication-title"],
        DISCOVERY_ALL_FAMILIES,
        Count
    ),
    field!(
        "publication_family",
        EnumString,
        Record,
        Enumerable,
        "SELECT record_key, publication_family FROM records",
        STRING_OPERATORS,
        [],
        DISCOVERY_ALL_FAMILIES,
        Count
    ),
    field!(
        "publication_remaster",
        Boolean,
        Record,
        BooleanCounts,
        "SELECT record_key, publication_remaster FROM records",
        BOOLEAN_OPERATORS,
        [],
        DISCOVERY_ALL_FAMILIES,
        Count
    ),
    field!(
        "rarity",
        EnumString,
        Record,
        Enumerable,
        "SELECT record_key, rarity FROM records",
        STRING_OPERATORS,
        ["--rarity"],
        DISCOVERY_ALL_FAMILIES,
        Canonical
    ),
    field!(
        "level",
        Number,
        Record,
        NumericStats,
        "SELECT record_key, level FROM records",
        NUMBER_OPERATORS,
        ["--level", "--min-level", "--max-level"],
        DISCOVERY_ALL_FAMILIES,
        Count
    ),
    field!(
        "action_cost",
        Number,
        Record,
        NumericStats,
        "SELECT record_key, activation_time_actions FROM records",
        NUMBER_OPERATORS,
        [],
        DISCOVERY_ALL_FAMILIES,
        Count
    ),
    field!(
        "traits",
        Set,
        Record,
        Enumerable,
        "SELECT record_key, trait FROM record_traits",
        SET_OPERATORS,
        ["--trait", "--any-trait"],
        DISCOVERY_ALL_FAMILIES,
        Count
    ),
    field!(
        "taxonomy_families",
        Set,
        Record,
        Enumerable,
        "SELECT record_key, j.value FROM records, json_each(COALESCE(taxonomy_families_json, '[]')) j",
        SET_OPERATORS,
        [],
        DISCOVERY_ALL_FAMILIES,
        Count
    ),
    field!(
        "traditions",
        Set,
        Spell,
        Enumerable,
        "SELECT record_key, j.value FROM spell_records, json_each(COALESCE(traditions_json, '[]')) j",
        SET_OPERATORS,
        [],
        SPELL_FAMILY,
        Count
    ),
    field!(
        "spell_kinds",
        Set,
        Spell,
        Enumerable,
        "SELECT record_key, j.value FROM spell_records, json_each(COALESCE(spell_kinds_json, '[]')) j",
        SET_OPERATORS,
        [],
        SPELL_FAMILY,
        Count
    ),
    field!(
        "damage_types",
        Set,
        Item,
        Enumerable,
        "SELECT record_key, j.value FROM item_records, json_each(COALESCE(damage_types_json, '[]')) j UNION ALL SELECT record_key, j.value FROM spell_records, json_each(COALESCE(damage_types_json, '[]')) j",
        SET_OPERATORS,
        [],
        &["equipment", "spell"],
        Count
    ),
    field!(
        "languages",
        Set,
        Actor,
        Enumerable,
        "SELECT record_key, j.value FROM actor_records, json_each(COALESCE(languages_json, '[]')) j",
        SET_OPERATORS,
        [],
        ACTOR_FAMILIES,
        Count
    ),
    field!(
        "speed_types",
        Set,
        Actor,
        Enumerable,
        "SELECT record_key, j.value FROM actor_records, json_each(COALESCE(speed_types_json, '[]')) j",
        SET_OPERATORS,
        [],
        ACTOR_FAMILIES,
        Count
    ),
    field!(
        "senses",
        Set,
        Actor,
        Enumerable,
        "SELECT record_key, j.value FROM actor_records, json_each(COALESCE(senses_json, '[]')) j",
        SET_OPERATORS,
        [],
        ACTOR_FAMILIES,
        Count
    ),
    field!(
        "immunities",
        Set,
        Actor,
        Enumerable,
        "SELECT record_key, j.value FROM actor_records, json_each(COALESCE(immunities_json, '[]')) j",
        SET_OPERATORS,
        [],
        ACTOR_FAMILIES,
        Count
    ),
    field!(
        "resistances",
        Set,
        Actor,
        Enumerable,
        "SELECT record_key, j.value FROM actor_records, json_each(COALESCE(resistances_json, '[]')) j",
        SET_OPERATORS,
        [],
        ACTOR_FAMILIES,
        Count
    ),
    field!(
        "weaknesses",
        Set,
        Actor,
        Enumerable,
        "SELECT record_key, j.value FROM actor_records, json_each(COALESCE(weaknesses_json, '[]')) j",
        SET_OPERATORS,
        [],
        ACTOR_FAMILIES,
        Count
    ),
    field!(
        "disable_skills",
        Set,
        Actor,
        Enumerable,
        "SELECT record_key, j.value FROM actor_records, json_each(COALESCE(disable_skills_json, '[]')) j",
        SET_OPERATORS,
        [],
        ACTOR_FAMILIES,
        Count
    ),
    field!(
        "size",
        EnumString,
        Actor,
        Enumerable,
        "SELECT record_key, size FROM actor_records",
        STRING_OPERATORS,
        [],
        ACTOR_FAMILIES,
        Count
    ),
    field!(
        "usage",
        EnumString,
        Item,
        Enumerable,
        "SELECT record_key, system_usage FROM records",
        STRING_OPERATORS,
        [],
        ITEM_FAMILIES,
        Count
    ),
    field!(
        "item_group",
        EnumString,
        Item,
        Enumerable,
        "SELECT record_key, system_group FROM records",
        STRING_OPERATORS,
        [],
        ITEM_FAMILIES,
        Count
    ),
    field!(
        "item_category",
        EnumString,
        Item,
        Enumerable,
        "SELECT record_key, system_category FROM records",
        STRING_OPERATORS,
        [],
        ITEM_FAMILIES,
        Count
    ),
    field!(
        "base_item",
        EnumString,
        Item,
        Enumerable,
        "SELECT record_key, system_base_item FROM records",
        STRING_OPERATORS,
        [],
        ITEM_FAMILIES,
        Count
    ),
    field!(
        "hands",
        EnumString,
        Item,
        Enumerable,
        "SELECT record_key, hands_requirement FROM item_records",
        STRING_OPERATORS,
        [],
        ITEM_FAMILIES,
        Count
    ),
    field!(
        "save_type",
        EnumString,
        Spell,
        Enumerable,
        "SELECT record_key, save_type FROM spell_records",
        STRING_OPERATORS,
        [],
        SPELL_FAMILY,
        Count
    ),
    field!(
        "area_type",
        EnumString,
        Spell,
        Enumerable,
        "SELECT record_key, area_type FROM spell_records",
        STRING_OPERATORS,
        [],
        SPELL_FAMILY,
        Count
    ),
    field!(
        "duration_unit",
        EnumString,
        Record,
        Enumerable,
        "SELECT record_key, duration_unit FROM records",
        STRING_OPERATORS,
        [],
        DISCOVERY_ALL_FAMILIES,
        Count
    ),
    field!(
        "sustained",
        Boolean,
        Spell,
        BooleanCounts,
        "SELECT record_key, sustained FROM spell_records",
        BOOLEAN_OPERATORS,
        [],
        SPELL_FAMILY,
        Count
    ),
    field!(
        "basic_save",
        Boolean,
        Spell,
        BooleanCounts,
        "SELECT record_key, basic_save FROM spell_records",
        BOOLEAN_OPERATORS,
        [],
        SPELL_FAMILY,
        Count
    ),
    field!(
        "is_complex",
        Boolean,
        Actor,
        BooleanCounts,
        "SELECT record_key, is_complex FROM actor_records",
        BOOLEAN_OPERATORS,
        [],
        ACTOR_FAMILIES,
        Count
    ),
    field!(
        "price_cp",
        Number,
        Item,
        NumericStats,
        "SELECT record_key, price_cp FROM records",
        NUMBER_OPERATORS,
        ["--price", "--min-price", "--max-price"],
        ITEM_FAMILIES,
        Count
    ),
    field!(
        "bulk_value",
        Number,
        Item,
        NumericStats,
        "SELECT record_key, bulk_value FROM item_records",
        NUMBER_OPERATORS,
        [],
        ITEM_FAMILIES,
        Count
    ),
    field!(
        "range_value",
        Number,
        Spell,
        NumericStats,
        "SELECT record_key, range_value FROM spell_records",
        NUMBER_OPERATORS,
        [],
        SPELL_FAMILY,
        Count
    ),
    field!(
        "area_value",
        Number,
        Spell,
        NumericStats,
        "SELECT record_key, area_value FROM spell_records",
        NUMBER_OPERATORS,
        [],
        SPELL_FAMILY,
        Count
    ),
    field!(
        "variant_group_key",
        EnumString,
        Variant,
        Enumerable,
        "SELECT record_key, variant_group_key FROM records",
        STRING_OPERATORS,
        [],
        DISCOVERY_ALL_FAMILIES,
        Count
    ),
    field!(
        "variant_axes",
        Set,
        Variant,
        Enumerable,
        "SELECT record_key, j.value FROM records, json_each(COALESCE(variant_axes_json, '[]')) j",
        SET_OPERATORS,
        [],
        DISCOVERY_ALL_FAMILIES,
        Count
    ),
    field!(
        "range_text",
        Text,
        Spell,
        Sample,
        "SELECT record_key, range_text FROM spell_records",
        TEXT_OPERATORS,
        [],
        SPELL_FAMILY,
        Count
    ),
    field!(
        "duration_text",
        Text,
        Record,
        Sample,
        "SELECT record_key, duration_text FROM records",
        TEXT_OPERATORS,
        [],
        DISCOVERY_ALL_FAMILIES,
        Count
    ),
    field!(
        "target_text",
        Text,
        Spell,
        Sample,
        "SELECT record_key, target_text FROM spell_records",
        TEXT_OPERATORS,
        [],
        SPELL_FAMILY,
        Count
    ),
    field!(
        "disable_text",
        Text,
        Actor,
        Sample,
        "SELECT record_key, disable_text FROM actor_records",
        TEXT_OPERATORS,
        [],
        ACTOR_FAMILIES,
        Count
    ),
    field!(
        "variant_base_name",
        Text,
        Variant,
        Sample,
        "SELECT record_key, variant_base_name FROM records",
        TEXT_OPERATORS,
        [],
        DISCOVERY_ALL_FAMILIES,
        Count
    ),
    field!(
        "variant_label",
        Text,
        Variant,
        Sample,
        "SELECT record_key, variant_label FROM records",
        TEXT_OPERATORS,
        [],
        DISCOVERY_ALL_FAMILIES,
        Count
    ),
];

pub fn discovery_field_definition(field: &str) -> Option<DiscoveryFieldDefinition> {
    DISCOVERY_FIELD_DEFINITIONS
        .iter()
        .copied()
        .find(|definition| definition.field == field)
}

pub fn all_discovery_field_definitions() -> &'static [DiscoveryFieldDefinition] {
    DISCOVERY_FIELD_DEFINITIONS
}

pub fn metric_filter_field_info(catalog_available: bool) -> FilterFieldInfo {
    FilterFieldInfo {
        field: "metric".to_string(),
        field_type: FilterFieldType::Metric,
        group: FilterFieldGroup::Metric,
        value_policy: FilterValuePolicy::MetricKeys,
        operators: vec![
            FilterOperator::Eq,
            FilterOperator::Gt,
            FilterOperator::Gte,
            FilterOperator::Lt,
            FilterOperator::Lte,
        ],
        applicable_families: Vec::new(),
        cli_flags: vec!["--metric".to_string()],
        catalog_available,
    }
}

#[cfg(test)]
mod tests {
    use atlas_domain::RecordFamily;

    use super::DISCOVERY_ALL_FAMILIES;

    #[test]
    fn discovery_families_match_canonical_record_families() {
        let expected = RecordFamily::ALL
            .iter()
            .map(|family| family.as_str())
            .collect::<Vec<_>>();

        assert_eq!(DISCOVERY_ALL_FAMILIES, expected.as_slice());
    }
}
