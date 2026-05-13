use super::{
    TABLE_ACTOR_RECORDS, TABLE_ITEM_RECORDS, TABLE_PACKS, TABLE_RECORD_ALIASES,
    TABLE_RECORD_METRICS, TABLE_RECORD_TRAITS, TABLE_RECORDS, TABLE_REFERENCE_EDGES,
    TABLE_REMASTER_LINKS, TABLE_SPELL_RECORDS,
};

pub struct BooleanColumn {
    pub key: &'static str,
    pub table: &'static str,
    pub column: &'static str,
    pub nullable: bool,
}

pub const BOOLEAN_COLUMNS: &[BooleanColumn] = &[
    BooleanColumn {
        key: "records.publication_remaster",
        table: TABLE_RECORDS,
        column: "publication_remaster",
        nullable: false,
    },
    BooleanColumn {
        key: "records.is_default_visible",
        table: TABLE_RECORDS,
        column: "is_default_visible",
        nullable: false,
    },
    BooleanColumn {
        key: "record_metrics.bool_value",
        table: TABLE_RECORD_METRICS,
        column: "bool_value",
        nullable: true,
    },
    BooleanColumn {
        key: "actor_records.is_complex",
        table: TABLE_ACTOR_RECORDS,
        column: "is_complex",
        nullable: false,
    },
    BooleanColumn {
        key: "spell_records.sustained",
        table: TABLE_SPELL_RECORDS,
        column: "sustained",
        nullable: false,
    },
    BooleanColumn {
        key: "spell_records.basic_save",
        table: TABLE_SPELL_RECORDS,
        column: "basic_save",
        nullable: false,
    },
];

pub struct RequiredReference {
    pub key: &'static str,
    pub table: &'static str,
    pub column: &'static str,
    pub referenced_table: &'static str,
    pub referenced_column: &'static str,
}

pub const REQUIRED_REFERENCES: &[RequiredReference] = &[
    RequiredReference {
        key: "records.pack_name",
        table: TABLE_RECORDS,
        column: "pack_name",
        referenced_table: TABLE_PACKS,
        referenced_column: "name",
    },
    RequiredReference {
        key: "record_traits.record_key",
        table: TABLE_RECORD_TRAITS,
        column: "record_key",
        referenced_table: TABLE_RECORDS,
        referenced_column: "record_key",
    },
    RequiredReference {
        key: "reference_edges.from_record_key",
        table: TABLE_REFERENCE_EDGES,
        column: "from_record_key",
        referenced_table: TABLE_RECORDS,
        referenced_column: "record_key",
    },
    RequiredReference {
        key: "reference_edges.to_record_key",
        table: TABLE_REFERENCE_EDGES,
        column: "to_record_key",
        referenced_table: TABLE_RECORDS,
        referenced_column: "record_key",
    },
    RequiredReference {
        key: "record_aliases.canonical_record_key",
        table: TABLE_RECORD_ALIASES,
        column: "canonical_record_key",
        referenced_table: TABLE_RECORDS,
        referenced_column: "record_key",
    },
    RequiredReference {
        key: "remaster_links.remaster_record_key",
        table: TABLE_REMASTER_LINKS,
        column: "remaster_record_key",
        referenced_table: TABLE_RECORDS,
        referenced_column: "record_key",
    },
    RequiredReference {
        key: "remaster_links.legacy_record_key",
        table: TABLE_REMASTER_LINKS,
        column: "legacy_record_key",
        referenced_table: TABLE_RECORDS,
        referenced_column: "record_key",
    },
    RequiredReference {
        key: "record_metrics.record_key",
        table: TABLE_RECORD_METRICS,
        column: "record_key",
        referenced_table: TABLE_RECORDS,
        referenced_column: "record_key",
    },
    RequiredReference {
        key: "actor_records.record_key",
        table: TABLE_ACTOR_RECORDS,
        column: "record_key",
        referenced_table: TABLE_RECORDS,
        referenced_column: "record_key",
    },
    RequiredReference {
        key: "item_records.record_key",
        table: TABLE_ITEM_RECORDS,
        column: "record_key",
        referenced_table: TABLE_RECORDS,
        referenced_column: "record_key",
    },
    RequiredReference {
        key: "spell_records.record_key",
        table: TABLE_SPELL_RECORDS,
        column: "record_key",
        referenced_table: TABLE_RECORDS,
        referenced_column: "record_key",
    },
];

pub fn invalid_boolean_column_sql(check: &BooleanColumn) -> String {
    if check.nullable {
        format!(
            "SELECT COUNT(*) FROM {} WHERE {} IS NOT NULL AND {} NOT IN (0, 1)",
            check.table, check.column, check.column
        )
    } else {
        format!(
            "SELECT COUNT(*) FROM {} WHERE {} NOT IN (0, 1)",
            check.table, check.column
        )
    }
}

pub fn orphan_reference_sql(reference: &RequiredReference) -> String {
    format!(
        "SELECT COUNT(*)
         FROM {table} child
         LEFT JOIN {referenced_table} parent ON parent.{referenced_column} = child.{column}
         WHERE parent.{referenced_column} IS NULL",
        table = reference.table,
        referenced_table = reference.referenced_table,
        referenced_column = reference.referenced_column,
        column = reference.column
    )
}
