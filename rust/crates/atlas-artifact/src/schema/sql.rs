use super::{
    ACTOR_RECORD_COLUMNS, Column, ColumnCheck, ColumnDescriptor, ForeignKeyAction,
    ITEM_RECORD_COLUMNS, PERSISTED_RECORD_COLUMNS, RECORD_ALIAS_COLUMNS, RECORD_CONTENT_COLUMNS,
    RECORD_METRIC_COLUMNS, REFERENCE_EDGE_COLUMNS, REMASTER_LINK_COLUMNS, SPELL_RECORD_COLUMNS,
    SqlType, TABLE_DESCRIPTORS, Table, TableConstraint, TableDescriptor, TableKind, actor_records,
    artifact_metadata, document_embedding_cache, filter_field_catalog, filter_numeric_catalog,
    filter_sample_catalog, filter_value_catalog, item_records, metric_key_catalog,
    metric_value_catalog, packs, record_aliases, record_content, record_metrics, record_traits,
    record_vector_index, records, records_fts, reference_edges, remaster_links, spell_records,
    table_descriptor,
};

pub fn artifact_metadata_insert_sql() -> String {
    insert_sql_for_table(artifact_metadata::TABLE)
}

pub fn pack_insert_sql() -> String {
    insert_sql_for_table(packs::TABLE)
}

pub fn record_insert_sql() -> String {
    insert_sql_for_table(records::TABLE)
}

pub fn record_trait_insert_sql() -> String {
    insert_sql_for_table(record_traits::TABLE)
}

pub fn record_content_insert_sql() -> String {
    insert_sql_for_table(record_content::TABLE)
}

pub fn record_metric_insert_sql() -> String {
    insert_sql_for_table(record_metrics::TABLE)
}

pub fn actor_record_insert_sql() -> String {
    insert_sql_for_table(actor_records::TABLE)
}

pub fn item_record_insert_sql() -> String {
    insert_sql_for_table(item_records::TABLE)
}

pub fn spell_record_insert_sql() -> String {
    insert_sql_for_table(spell_records::TABLE)
}

pub fn records_fts_insert_sql() -> String {
    insert_sql_for_table(records_fts::TABLE)
}

pub fn document_embedding_cache_insert_sql() -> String {
    insert_sql_for_table(document_embedding_cache::TABLE)
}

pub fn reference_edge_insert_sql() -> String {
    insert_or_ignore_sql_for_table(reference_edges::TABLE)
}

pub fn record_alias_insert_sql() -> String {
    insert_or_ignore_sql_for_table(record_aliases::TABLE)
}

pub fn remaster_link_insert_sql() -> String {
    insert_or_ignore_sql_for_table(remaster_links::TABLE)
}

pub fn metric_key_catalog_insert_select_sql() -> String {
    format!(
        "INSERT INTO {table} ({columns})
            SELECT
              rm.metric_domain,
              NULL AS record_family,
              CASE
                WHEN instr(rm.metric_key, '.') > 0 THEN substr(rm.metric_key, 1, instr(rm.metric_key, '.'))
                ELSE ''
              END AS namespace_prefix,
              rm.metric_key,
              rm.value_type,
              COUNT(*) AS catalog_count,
              CASE WHEN rm.value_type = 'number' THEN MIN(rm.number_value) ELSE NULL END AS numeric_min,
              CASE WHEN rm.value_type = 'number' THEN MAX(rm.number_value) ELSE NULL END AS numeric_max
            FROM record_metrics rm
            JOIN records r ON r.record_key = rm.record_key
            WHERE r.is_default_visible = 1
            GROUP BY rm.metric_domain, namespace_prefix, rm.metric_key, rm.value_type
            UNION ALL
            SELECT
              rm.metric_domain,
              r.record_family,
              CASE
                WHEN instr(rm.metric_key, '.') > 0 THEN substr(rm.metric_key, 1, instr(rm.metric_key, '.'))
                ELSE ''
              END AS namespace_prefix,
              rm.metric_key,
              rm.value_type,
              COUNT(*) AS catalog_count,
              CASE WHEN rm.value_type = 'number' THEN MIN(rm.number_value) ELSE NULL END AS numeric_min,
              CASE WHEN rm.value_type = 'number' THEN MAX(rm.number_value) ELSE NULL END AS numeric_max
            FROM record_metrics rm
            JOIN records r ON r.record_key = rm.record_key
            WHERE r.is_default_visible = 1
            GROUP BY rm.metric_domain, r.record_family, namespace_prefix, rm.metric_key, rm.value_type",
        table = metric_key_catalog::TABLE.name(),
        columns = column_names(metric_key_catalog::ALL_COLUMNS).join(", ")
    )
}

pub fn metric_value_catalog_insert_select_sql() -> String {
    format!(
        "INSERT INTO {table} ({columns})
            SELECT
              rm.metric_domain,
              NULL AS record_family,
              rm.metric_key,
              CASE
                WHEN rm.value_type = 'text' THEN rm.text_value
                WHEN rm.value_type = 'boolean' THEN CAST(rm.bool_value AS TEXT)
                ELSE NULL
              END AS value,
              COUNT(*) AS catalog_count
            FROM record_metrics rm
            JOIN records r ON r.record_key = rm.record_key
            WHERE r.is_default_visible = 1
              AND rm.value_type IN ('text', 'boolean')
              AND value IS NOT NULL
            GROUP BY rm.metric_domain, rm.metric_key, value
            UNION ALL
            SELECT
              rm.metric_domain,
              r.record_family,
              rm.metric_key,
              CASE
                WHEN rm.value_type = 'text' THEN rm.text_value
                WHEN rm.value_type = 'boolean' THEN CAST(rm.bool_value AS TEXT)
                ELSE NULL
              END AS value,
              COUNT(*) AS catalog_count
            FROM record_metrics rm
            JOIN records r ON r.record_key = rm.record_key
            WHERE r.is_default_visible = 1
              AND rm.value_type IN ('text', 'boolean')
              AND value IS NOT NULL
            GROUP BY rm.metric_domain, r.record_family, rm.metric_key, value",
        table = metric_value_catalog::TABLE.name(),
        columns = column_names(metric_value_catalog::ALL_COLUMNS).join(", ")
    )
}

pub fn filter_field_catalog_insert_sql() -> String {
    insert_sql_for_table(filter_field_catalog::TABLE)
}

pub fn filter_value_catalog_insert_sql() -> String {
    insert_sql_for_table(filter_value_catalog::TABLE)
}

pub fn filter_sample_catalog_insert_sql() -> String {
    insert_sql_for_table(filter_sample_catalog::TABLE)
}

pub fn filter_numeric_catalog_insert_sql() -> String {
    insert_sql_for_table(filter_numeric_catalog::TABLE)
}

pub fn record_vector_index_create_sql(dimensions: usize) -> String {
    format!(
        "CREATE VIRTUAL TABLE {} USING vec0(embedding FLOAT[{dimensions}])",
        record_vector_index::TABLE.name()
    )
}

pub fn record_vector_index_insert_sql() -> String {
    format!(
        "INSERT INTO {} (rowid, embedding) VALUES (?1, ?2)",
        record_vector_index::TABLE.name()
    )
}

pub fn persisted_record_select_sql() -> String {
    ordered_select_sql(
        records::TABLE,
        PERSISTED_RECORD_COLUMNS,
        &[records::columns::RECORD_KEY],
    )
}

pub fn record_metric_select_sql() -> String {
    ordered_select_sql(
        record_metrics::TABLE,
        RECORD_METRIC_COLUMNS,
        &[
            record_metrics::columns::RECORD_KEY,
            record_metrics::columns::METRIC_DOMAIN,
            record_metrics::columns::METRIC_KEY,
        ],
    )
}

pub fn record_content_select_sql() -> String {
    ordered_select_sql(
        record_content::TABLE,
        RECORD_CONTENT_COLUMNS,
        &[
            record_content::columns::RECORD_KEY,
            record_content::columns::ORDINAL,
        ],
    )
}

pub fn actor_record_select_sql() -> String {
    ordered_select_sql(
        actor_records::TABLE,
        ACTOR_RECORD_COLUMNS,
        &[actor_records::columns::RECORD_KEY],
    )
}

pub fn item_record_select_sql() -> String {
    ordered_select_sql(
        item_records::TABLE,
        ITEM_RECORD_COLUMNS,
        &[item_records::columns::RECORD_KEY],
    )
}

pub fn spell_record_select_sql() -> String {
    ordered_select_sql(
        spell_records::TABLE,
        SPELL_RECORD_COLUMNS,
        &[spell_records::columns::RECORD_KEY],
    )
}

pub fn reference_edge_select_sql() -> String {
    ordered_select_sql(
        reference_edges::TABLE,
        REFERENCE_EDGE_COLUMNS,
        &[
            reference_edges::columns::FROM_RECORD_KEY,
            reference_edges::columns::TO_RECORD_KEY,
            reference_edges::columns::REFERENCE_TEXT,
        ],
    )
}

pub fn record_alias_select_sql() -> String {
    ordered_select_sql(
        record_aliases::TABLE,
        RECORD_ALIAS_COLUMNS,
        &[
            record_aliases::columns::CANONICAL_RECORD_KEY,
            record_aliases::columns::NORMALIZED_ALIAS,
            record_aliases::columns::SOURCE_KIND,
            record_aliases::columns::SOURCE_REF,
        ],
    )
}

pub fn remaster_link_select_sql() -> String {
    ordered_select_sql(
        remaster_links::TABLE,
        REMASTER_LINK_COLUMNS,
        &[
            remaster_links::columns::REMASTER_RECORD_KEY,
            remaster_links::columns::LEGACY_RECORD_KEY,
            remaster_links::columns::SOURCE_KIND,
            remaster_links::columns::SOURCE_REF,
        ],
    )
}

pub fn insert_sql(table: Table, columns: &[Column]) -> String {
    let placeholders = (1..=columns.len())
        .map(|index| format!("?{index}"))
        .collect::<Vec<_>>()
        .join(", ");
    format!(
        "INSERT INTO {table} ({columns}) VALUES ({placeholders})",
        table = table.name(),
        columns = column_names(columns).join(", ")
    )
}

pub fn ordered_select_sql(table: Table, columns: &[Column], order_by: &[Column]) -> String {
    format!(
        "SELECT {columns} FROM {table} ORDER BY {order_by}",
        table = table.name(),
        columns = column_names(columns).join(", "),
        order_by = column_names(order_by).join(", ")
    )
}

fn column_names(columns: &[Column]) -> Vec<&'static str> {
    columns.iter().map(|column| column.name()).collect()
}

pub fn create_artifact_schema_sql() -> String {
    let mut statements = vec!["PRAGMA foreign_keys = ON".to_string()];
    statements.extend(TABLE_DESCRIPTORS.iter().map(create_table_sql));
    statements.extend(
        ARTIFACT_INDEX_SQL
            .iter()
            .map(|statement| (*statement).to_string()),
    );
    format!("{};", statements.join(";\n\n"))
}

fn insert_sql_for_table(table: Table) -> String {
    let descriptor = table_descriptor(table).expect("artifact table descriptor should exist");
    insert_sql(table, &descriptor.columns())
}

fn insert_or_ignore_sql_for_table(table: Table) -> String {
    let descriptor = table_descriptor(table).expect("artifact table descriptor should exist");
    insert_or_ignore_sql(table, &descriptor.columns())
}

pub fn insert_or_ignore_sql(table: Table, columns: &[Column]) -> String {
    let placeholders = (1..=columns.len())
        .map(|index| format!("?{index}"))
        .collect::<Vec<_>>()
        .join(", ");
    format!(
        "INSERT OR IGNORE INTO {table} ({columns}) VALUES ({placeholders})",
        table = table.name(),
        columns = column_names(columns).join(", ")
    )
}

fn create_table_sql(descriptor: &TableDescriptor) -> String {
    match descriptor.kind {
        TableKind::Ordinary => ordinary_create_table_sql(descriptor),
        TableKind::Fts5 { unindexed } => fts5_create_table_sql(descriptor, unindexed),
        TableKind::Vector => panic!("vec tables require runtime dimensions"),
    }
}

fn ordinary_create_table_sql(descriptor: &TableDescriptor) -> String {
    let mut clauses = descriptor
        .column_descriptors
        .iter()
        .map(column_definition_sql)
        .collect::<Vec<_>>();
    clauses.extend(
        descriptor
            .table_constraints
            .iter()
            .map(table_constraint_sql),
    );
    format!(
        "CREATE TABLE {} ({})",
        descriptor.table.name(),
        clauses.join(", ")
    )
}

fn fts5_create_table_sql(descriptor: &TableDescriptor, unindexed: &[Column]) -> String {
    let columns = descriptor
        .column_descriptors
        .iter()
        .map(|column| {
            if unindexed.contains(&column.column) {
                format!("{} UNINDEXED", column.column.name())
            } else {
                column.column.name().to_string()
            }
        })
        .collect::<Vec<_>>()
        .join(", ");
    format!(
        "CREATE VIRTUAL TABLE {} USING fts5({columns})",
        descriptor.table.name()
    )
}

fn column_definition_sql(descriptor: &ColumnDescriptor) -> String {
    let mut parts = vec![
        descriptor.column.name().to_string(),
        sql_type_name(descriptor.sql_type).to_string(),
    ];
    if descriptor.primary_key {
        parts.push("PRIMARY KEY".to_string());
    }
    if !descriptor.nullable {
        parts.push("NOT NULL".to_string());
    }
    if let Some(check) = descriptor.check {
        parts.push(column_check_sql(descriptor.column, check));
    }
    parts.join(" ")
}

fn sql_type_name(sql_type: SqlType) -> &'static str {
    match sql_type {
        SqlType::Text => "TEXT",
        SqlType::Integer => "INTEGER",
        SqlType::Real => "REAL",
        SqlType::Blob => "BLOB",
    }
}

fn column_check_sql(column: Column, check: ColumnCheck) -> String {
    match check {
        ColumnCheck::Boolean => format!("CHECK ({} IN (0, 1))", column.name()),
        ColumnCheck::ClosedSet(values) => format!(
            "CHECK ({} IN ({}))",
            column.name(),
            values
                .iter()
                .map(|value| format!("'{value}'"))
                .collect::<Vec<_>>()
                .join(", ")
        ),
    }
}

fn table_constraint_sql(constraint: &TableConstraint) -> String {
    match constraint {
        TableConstraint::PrimaryKey(columns) => {
            format!("PRIMARY KEY ({})", column_names(columns).join(", "))
        }
        TableConstraint::ForeignKey {
            columns,
            target_table,
            target_columns,
            on_delete,
        } => {
            let mut sql = format!(
                "FOREIGN KEY ({}) REFERENCES {}({})",
                column_names(columns).join(", "),
                target_table.name(),
                column_names(target_columns).join(", ")
            );
            if matches!(on_delete, Some(ForeignKeyAction::Cascade)) {
                sql.push_str(" ON DELETE CASCADE");
            }
            sql
        }
    }
}

const ARTIFACT_INDEX_SQL: &[&str] = &[
    "CREATE INDEX records_pack_name_idx ON records(pack_name)",
    "CREATE INDEX records_default_visible_idx ON records(is_default_visible)",
    "CREATE INDEX record_content_record_idx ON record_content(record_key)",
    "CREATE INDEX record_content_visibility_idx ON record_content(visibility, source_kind)",
    "CREATE INDEX reference_edges_from_idx ON reference_edges(from_record_key)",
    "CREATE INDEX reference_edges_to_idx ON reference_edges(to_record_key)",
    "CREATE INDEX record_aliases_canonical_idx ON record_aliases(canonical_record_key)",
    "CREATE INDEX record_aliases_normalized_alias_idx ON record_aliases(normalized_alias)",
    "CREATE INDEX remaster_links_remaster_idx ON remaster_links(remaster_record_key)",
    "CREATE INDEX remaster_links_legacy_idx ON remaster_links(legacy_record_key)",
    "CREATE INDEX record_metrics_record_idx ON record_metrics(record_key)",
    "CREATE INDEX record_metrics_catalog_source_idx ON record_metrics(metric_domain, metric_key, value_type)",
    "CREATE INDEX metric_key_catalog_coverage_idx ON metric_key_catalog(metric_domain, record_family, metric_key)",
    "CREATE INDEX metric_value_catalog_coverage_idx ON metric_value_catalog(metric_domain, record_family, metric_key, value)",
    "CREATE INDEX filter_field_catalog_scope_idx ON filter_field_catalog(field, record_family)",
    "CREATE INDEX filter_value_catalog_scope_idx ON filter_value_catalog(field, record_family, value)",
    "CREATE INDEX filter_sample_catalog_scope_idx ON filter_sample_catalog(field, record_family, sample_rank)",
    "CREATE INDEX filter_numeric_catalog_scope_idx ON filter_numeric_catalog(field, record_family, metric_domain, metric_key)",
    "CREATE INDEX document_embedding_cache_record_idx ON document_embedding_cache(record_key)",
    "CREATE INDEX document_embedding_cache_hash_idx ON document_embedding_cache(semantic_input_hash)",
];

#[cfg(test)]
mod tests {
    use crate::schema::{RECORD_COLUMNS, required_columns};

    use super::*;

    #[test]
    fn record_insert_sql_uses_record_column_descriptor_order() {
        assert_eq!(
            record_insert_sql(),
            format!(
                "INSERT INTO records ({}) VALUES ({})",
                column_names(RECORD_COLUMNS).join(", "),
                (1..=RECORD_COLUMNS.len())
                    .map(|index| format!("?{index}"))
                    .collect::<Vec<_>>()
                    .join(", ")
            )
        );
    }

    #[test]
    fn side_table_insert_sql_uses_column_descriptor_order() {
        assert_eq!(
            record_trait_insert_sql(),
            "INSERT INTO record_traits (record_key, trait) VALUES (?1, ?2)"
        );
        assert_eq!(
            record_content_insert_sql(),
            "INSERT INTO record_content (record_key, ordinal, source_kind, visibility, contributes_to_search, contributes_to_references, label, content_json) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)"
        );
        assert_eq!(
            record_metric_insert_sql(),
            format!(
                "INSERT INTO record_metrics ({}) VALUES ({})",
                column_names(RECORD_METRIC_COLUMNS).join(", "),
                (1..=RECORD_METRIC_COLUMNS.len())
                    .map(|index| format!("?{index}"))
                    .collect::<Vec<_>>()
                    .join(", ")
            )
        );
        assert_eq!(
            actor_record_insert_sql(),
            format!(
                "INSERT INTO actor_records ({}) VALUES ({})",
                column_names(ACTOR_RECORD_COLUMNS).join(", "),
                (1..=ACTOR_RECORD_COLUMNS.len())
                    .map(|index| format!("?{index}"))
                    .collect::<Vec<_>>()
                    .join(", ")
            )
        );
        assert_eq!(
            item_record_insert_sql(),
            format!(
                "INSERT INTO item_records ({}) VALUES ({})",
                column_names(ITEM_RECORD_COLUMNS).join(", "),
                (1..=ITEM_RECORD_COLUMNS.len())
                    .map(|index| format!("?{index}"))
                    .collect::<Vec<_>>()
                    .join(", ")
            )
        );
        assert_eq!(
            spell_record_insert_sql(),
            format!(
                "INSERT INTO spell_records ({}) VALUES ({})",
                column_names(SPELL_RECORD_COLUMNS).join(", "),
                (1..=SPELL_RECORD_COLUMNS.len())
                    .map(|index| format!("?{index}"))
                    .collect::<Vec<_>>()
                    .join(", ")
            )
        );
        assert_eq!(
            records_fts_insert_sql(),
            "INSERT INTO records_fts (record_key, title, aliases, traits, headings, body, facts, reference_terms, embedded_content) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)"
        );
        assert_eq!(
            document_embedding_cache_insert_sql(),
            "INSERT INTO document_embedding_cache (embedding_unit_key, record_key, unit_kind, label, ordinal, semantic_input_hash, dimensions, vector_blob) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)"
        );
        assert_eq!(
            record_vector_index_insert_sql(),
            "INSERT INTO record_vector_index (rowid, embedding) VALUES (?1, ?2)"
        );
        assert_eq!(
            record_vector_index_create_sql(384),
            "CREATE VIRTUAL TABLE record_vector_index USING vec0(embedding FLOAT[384])"
        );
    }

    #[test]
    fn persisted_record_select_sql_uses_persisted_column_descriptor_order() {
        assert_eq!(
            persisted_record_select_sql(),
            format!(
                "SELECT {} FROM records ORDER BY record_key",
                column_names(PERSISTED_RECORD_COLUMNS).join(", ")
            )
        );
        assert!(PERSISTED_RECORD_COLUMNS.contains(&records::columns::DESCRIPTION_JSON));
        assert!(PERSISTED_RECORD_COLUMNS.contains(&records::columns::BLURB_JSON));
    }

    #[test]
    fn created_schema_columns_match_descriptor_order() {
        let connection =
            rusqlite::Connection::open_in_memory().expect("in-memory database should open");
        connection
            .execute_batch(&create_artifact_schema_sql())
            .expect("artifact schema should create");

        for (table, columns) in required_columns() {
            let table_name = table.name();
            let mut statement = connection
                .prepare(&format!("PRAGMA table_xinfo({table_name})"))
                .expect("table info statement should prepare");
            let present = statement
                .query_map([], |row| {
                    Ok((row.get::<_, String>(1)?, row.get::<_, i64>(6)?))
                })
                .expect("table info should query")
                .collect::<Result<Vec<_>, _>>()
                .expect("table info rows should load")
                .into_iter()
                .filter_map(|(name, hidden)| (hidden == 0).then_some(name))
                .collect::<Vec<_>>();
            let expected = columns
                .iter()
                .map(|column| column.name().to_string())
                .collect::<Vec<_>>();
            assert_eq!(
                present, expected,
                "created schema columns drifted from descriptor columns for {table_name}"
            );
        }
    }
}
