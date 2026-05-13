use super::{
    Column, Table, actor_records, document_embedding_cache, item_records, packs, record_aliases,
    record_metrics, record_traits, records, reference_edges, remaster_links, spell_records,
};

pub struct BooleanColumn {
    pub key: &'static str,
    pub table: Table,
    pub column: Column,
    pub nullable: bool,
}

pub const BOOLEAN_COLUMNS: &[BooleanColumn] = &[
    BooleanColumn {
        key: "records.publication_remaster",
        table: records::TABLE,
        column: records::columns::PUBLICATION_REMASTER,
        nullable: false,
    },
    BooleanColumn {
        key: "records.is_default_visible",
        table: records::TABLE,
        column: records::columns::IS_DEFAULT_VISIBLE,
        nullable: false,
    },
    BooleanColumn {
        key: "record_metrics.bool_value",
        table: record_metrics::TABLE,
        column: record_metrics::columns::BOOL_VALUE,
        nullable: true,
    },
    BooleanColumn {
        key: "actor_records.is_complex",
        table: actor_records::TABLE,
        column: actor_records::columns::IS_COMPLEX,
        nullable: false,
    },
    BooleanColumn {
        key: "spell_records.sustained",
        table: spell_records::TABLE,
        column: spell_records::columns::SUSTAINED,
        nullable: false,
    },
    BooleanColumn {
        key: "spell_records.basic_save",
        table: spell_records::TABLE,
        column: spell_records::columns::BASIC_SAVE,
        nullable: false,
    },
];

pub struct RequiredReference {
    pub key: &'static str,
    pub table: Table,
    pub column: Column,
    pub referenced_table: Table,
    pub referenced_column: Column,
}

pub const REQUIRED_REFERENCES: &[RequiredReference] = &[
    RequiredReference {
        key: "records.pack_name",
        table: records::TABLE,
        column: records::columns::PACK_NAME,
        referenced_table: packs::TABLE,
        referenced_column: packs::columns::NAME,
    },
    RequiredReference {
        key: "record_traits.record_key",
        table: record_traits::TABLE,
        column: record_traits::columns::RECORD_KEY,
        referenced_table: records::TABLE,
        referenced_column: records::columns::RECORD_KEY,
    },
    RequiredReference {
        key: "reference_edges.from_record_key",
        table: reference_edges::TABLE,
        column: reference_edges::columns::FROM_RECORD_KEY,
        referenced_table: records::TABLE,
        referenced_column: records::columns::RECORD_KEY,
    },
    RequiredReference {
        key: "reference_edges.to_record_key",
        table: reference_edges::TABLE,
        column: reference_edges::columns::TO_RECORD_KEY,
        referenced_table: records::TABLE,
        referenced_column: records::columns::RECORD_KEY,
    },
    RequiredReference {
        key: "record_aliases.canonical_record_key",
        table: record_aliases::TABLE,
        column: record_aliases::columns::CANONICAL_RECORD_KEY,
        referenced_table: records::TABLE,
        referenced_column: records::columns::RECORD_KEY,
    },
    RequiredReference {
        key: "remaster_links.remaster_record_key",
        table: remaster_links::TABLE,
        column: remaster_links::columns::REMASTER_RECORD_KEY,
        referenced_table: records::TABLE,
        referenced_column: records::columns::RECORD_KEY,
    },
    RequiredReference {
        key: "remaster_links.legacy_record_key",
        table: remaster_links::TABLE,
        column: remaster_links::columns::LEGACY_RECORD_KEY,
        referenced_table: records::TABLE,
        referenced_column: records::columns::RECORD_KEY,
    },
    RequiredReference {
        key: "record_metrics.record_key",
        table: record_metrics::TABLE,
        column: record_metrics::columns::RECORD_KEY,
        referenced_table: records::TABLE,
        referenced_column: records::columns::RECORD_KEY,
    },
    RequiredReference {
        key: "actor_records.record_key",
        table: actor_records::TABLE,
        column: actor_records::columns::RECORD_KEY,
        referenced_table: records::TABLE,
        referenced_column: records::columns::RECORD_KEY,
    },
    RequiredReference {
        key: "item_records.record_key",
        table: item_records::TABLE,
        column: item_records::columns::RECORD_KEY,
        referenced_table: records::TABLE,
        referenced_column: records::columns::RECORD_KEY,
    },
    RequiredReference {
        key: "spell_records.record_key",
        table: spell_records::TABLE,
        column: spell_records::columns::RECORD_KEY,
        referenced_table: records::TABLE,
        referenced_column: records::columns::RECORD_KEY,
    },
    RequiredReference {
        key: "document_embedding_cache.record_key",
        table: document_embedding_cache::TABLE,
        column: document_embedding_cache::columns::RECORD_KEY,
        referenced_table: records::TABLE,
        referenced_column: records::columns::RECORD_KEY,
    },
];

pub fn invalid_boolean_column_sql(check: &BooleanColumn) -> String {
    if check.nullable {
        format!(
            "SELECT COUNT(*) FROM {} WHERE {} IS NOT NULL AND {} NOT IN (0, 1)",
            check.table.name(),
            check.column.name(),
            check.column.name()
        )
    } else {
        format!(
            "SELECT COUNT(*) FROM {} WHERE {} NOT IN (0, 1)",
            check.table.name(),
            check.column.name()
        )
    }
}

pub fn orphan_reference_sql(reference: &RequiredReference) -> String {
    format!(
        "SELECT COUNT(*)
         FROM {table} child
         LEFT JOIN {referenced_table} parent ON parent.{referenced_column} = child.{column}
         WHERE parent.{referenced_column} IS NULL",
        table = reference.table.name(),
        referenced_table = reference.referenced_table.name(),
        referenced_column = reference.referenced_column.name(),
        column = reference.column.name()
    )
}
