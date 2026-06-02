use atlas_domain::{
    FilterFieldGroup, FilterFieldInfo, FilterFieldType, FilterOperator, FilterValuePolicy,
    FilterValueSort,
};

use crate::artifact::inventory::{
    Column, Table, actor_records, item_records, record_traits, records, spell_records,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct DiscoveryFieldDefinition {
    pub(crate) field: &'static str,
    pub(crate) field_type: FilterFieldType,
    pub(crate) group: FilterFieldGroup,
    pub(crate) value_policy: FilterValuePolicy,
    extractor: DiscoveryFieldExtractor,
    pub(crate) operators: &'static [FilterOperator],
    pub(crate) cli_flags: &'static [&'static str],
    pub(crate) applicable_families: &'static [&'static str],
    pub(crate) default_sort: FilterValueSort,
    pub(crate) policy_reason: &'static str,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum DiscoveryFieldExtractor {
    Column(Column),
    JsonArrayColumn(Column),
    UnionJsonArrayColumns(&'static [Column]),
}

pub(crate) type FieldDefinition = DiscoveryFieldDefinition;

impl DiscoveryFieldDefinition {
    pub(crate) fn info(self, catalog_available: bool) -> FilterFieldInfo {
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

    pub(crate) fn value_sql(self) -> String {
        self.extractor.value_sql()
    }
}

impl DiscoveryFieldExtractor {
    fn value_sql(self) -> String {
        match self {
            Self::Column(column) => column_value_sql(column),
            Self::JsonArrayColumn(column) => json_array_value_sql(column),
            Self::UnionJsonArrayColumns(columns) => columns
                .iter()
                .copied()
                .map(json_array_value_sql)
                .collect::<Vec<_>>()
                .join(" UNION ALL "),
        }
    }
}

fn column_value_sql(column: Column) -> String {
    let table = column.table();
    format!(
        "SELECT {table}.{record_key}, {table}.{column} FROM {table}",
        table = table.name(),
        record_key = record_key_column(table).name(),
        column = column.name(),
    )
}

fn json_array_value_sql(column: Column) -> String {
    let table = column.table();
    format!(
        "SELECT {table}.{record_key}, j.value FROM {table}, json_each(COALESCE({table}.{column}, '[]')) j",
        table = table.name(),
        record_key = record_key_column(table).name(),
        column = column.name(),
    )
}

fn record_key_column(table: Table) -> Column {
    if table == records::TABLE {
        records::columns::RECORD_KEY
    } else if table == record_traits::TABLE {
        record_traits::columns::RECORD_KEY
    } else if table == actor_records::TABLE {
        actor_records::columns::RECORD_KEY
    } else if table == item_records::TABLE {
        item_records::columns::RECORD_KEY
    } else if table == spell_records::TABLE {
        spell_records::columns::RECORD_KEY
    } else {
        records::columns::RECORD_KEY
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

pub(crate) const DISCOVERY_ALL_FAMILIES: &[&str] = &[
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
const DAMAGE_TYPE_COLUMNS: &[Column] = &[
    item_records::columns::DAMAGE_TYPES_JSON,
    spell_records::columns::DAMAGE_TYPES_JSON,
];

macro_rules! field {
    ($field:literal, $field_type:ident, $group:ident, $policy:ident, $extractor:expr, $operators:ident, [$($flag:literal),*], $families:expr, $sort:ident) => {
        DiscoveryFieldDefinition {
            field: $field,
            field_type: FilterFieldType::$field_type,
            group: FilterFieldGroup::$group,
            value_policy: FilterValuePolicy::$policy,
            extractor: $extractor,
            operators: $operators,
            cli_flags: &[$($flag),*],
            applicable_families: $families,
            default_sort: FilterValueSort::$sort,
            policy_reason: stringify!($policy),
        }
    };
}

pub(crate) const DISCOVERY_FIELD_DEFINITIONS: &[DiscoveryFieldDefinition] = &[
    field!(
        "record_family",
        EnumString,
        Record,
        Enumerable,
        DiscoveryFieldExtractor::Column(records::columns::RECORD_FAMILY),
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
        DiscoveryFieldExtractor::Column(records::columns::PACK_NAME),
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
        DiscoveryFieldExtractor::Column(records::columns::PACK_LABEL),
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
        DiscoveryFieldExtractor::Column(records::columns::FOUNDRY_RECORD_TYPE),
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
        DiscoveryFieldExtractor::Column(records::columns::PUBLICATION_TITLE),
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
        DiscoveryFieldExtractor::Column(records::columns::PUBLICATION_FAMILY),
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
        DiscoveryFieldExtractor::Column(records::columns::PUBLICATION_REMASTER),
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
        DiscoveryFieldExtractor::Column(records::columns::RARITY),
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
        DiscoveryFieldExtractor::Column(records::columns::LEVEL),
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
        DiscoveryFieldExtractor::Column(records::columns::ACTIVATION_TIME_ACTIONS),
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
        DiscoveryFieldExtractor::Column(record_traits::columns::TRAIT),
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
        DiscoveryFieldExtractor::JsonArrayColumn(records::columns::TAXONOMY_FAMILIES_JSON),
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
        DiscoveryFieldExtractor::JsonArrayColumn(spell_records::columns::TRADITIONS_JSON),
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
        DiscoveryFieldExtractor::JsonArrayColumn(spell_records::columns::SPELL_KINDS_JSON),
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
        DiscoveryFieldExtractor::UnionJsonArrayColumns(DAMAGE_TYPE_COLUMNS),
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
        DiscoveryFieldExtractor::JsonArrayColumn(actor_records::columns::LANGUAGES_JSON),
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
        DiscoveryFieldExtractor::JsonArrayColumn(actor_records::columns::SPEED_TYPES_JSON),
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
        DiscoveryFieldExtractor::JsonArrayColumn(actor_records::columns::SENSES_JSON),
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
        DiscoveryFieldExtractor::JsonArrayColumn(actor_records::columns::IMMUNITIES_JSON),
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
        DiscoveryFieldExtractor::JsonArrayColumn(actor_records::columns::RESISTANCES_JSON),
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
        DiscoveryFieldExtractor::JsonArrayColumn(actor_records::columns::WEAKNESSES_JSON),
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
        DiscoveryFieldExtractor::JsonArrayColumn(actor_records::columns::DISABLE_SKILLS_JSON),
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
        DiscoveryFieldExtractor::Column(actor_records::columns::SIZE),
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
        DiscoveryFieldExtractor::Column(records::columns::SYSTEM_USAGE),
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
        DiscoveryFieldExtractor::Column(records::columns::SYSTEM_GROUP),
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
        DiscoveryFieldExtractor::Column(records::columns::SYSTEM_CATEGORY),
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
        DiscoveryFieldExtractor::Column(records::columns::SYSTEM_BASE_ITEM),
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
        DiscoveryFieldExtractor::Column(item_records::columns::HANDS_REQUIREMENT),
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
        DiscoveryFieldExtractor::Column(spell_records::columns::SAVE_TYPE),
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
        DiscoveryFieldExtractor::Column(spell_records::columns::AREA_TYPE),
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
        DiscoveryFieldExtractor::Column(records::columns::DURATION_UNIT),
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
        DiscoveryFieldExtractor::Column(spell_records::columns::SUSTAINED),
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
        DiscoveryFieldExtractor::Column(spell_records::columns::BASIC_SAVE),
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
        DiscoveryFieldExtractor::Column(actor_records::columns::IS_COMPLEX),
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
        DiscoveryFieldExtractor::Column(records::columns::PRICE_CP),
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
        DiscoveryFieldExtractor::Column(item_records::columns::BULK_VALUE),
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
        DiscoveryFieldExtractor::Column(spell_records::columns::RANGE_VALUE),
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
        DiscoveryFieldExtractor::Column(spell_records::columns::AREA_VALUE),
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
        DiscoveryFieldExtractor::Column(records::columns::VARIANT_GROUP_KEY),
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
        DiscoveryFieldExtractor::JsonArrayColumn(records::columns::VARIANT_AXES_JSON),
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
        DiscoveryFieldExtractor::Column(spell_records::columns::RANGE_TEXT),
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
        DiscoveryFieldExtractor::Column(records::columns::DURATION_TEXT),
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
        DiscoveryFieldExtractor::Column(spell_records::columns::TARGET_TEXT),
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
        DiscoveryFieldExtractor::Column(actor_records::columns::DISABLE_TEXT),
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
        DiscoveryFieldExtractor::Column(records::columns::VARIANT_BASE_NAME),
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
        DiscoveryFieldExtractor::Column(records::columns::VARIANT_LABEL),
        TEXT_OPERATORS,
        [],
        DISCOVERY_ALL_FAMILIES,
        Count
    ),
];

pub(crate) fn definition_for(field: &str) -> Option<DiscoveryFieldDefinition> {
    DISCOVERY_FIELD_DEFINITIONS
        .iter()
        .copied()
        .find(|definition| definition.field == field)
}

pub(crate) fn all_definitions() -> &'static [DiscoveryFieldDefinition] {
    DISCOVERY_FIELD_DEFINITIONS
}

pub(crate) fn metric_field_info(catalog_available: bool) -> FilterFieldInfo {
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
    use atlas_domain::RecordKind;

    use super::DISCOVERY_ALL_FAMILIES;

    #[test]
    fn discovery_families_match_canonical_record_families() {
        let expected = RecordKind::ALL
            .iter()
            .map(|family| family.as_str())
            .collect::<Vec<_>>();

        assert_eq!(DISCOVERY_ALL_FAMILIES, expected.as_slice());
    }
}
