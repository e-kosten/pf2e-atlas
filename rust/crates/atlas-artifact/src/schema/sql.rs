use super::{
    ACTOR_RECORD_COLUMNS, Column, ITEM_RECORD_COLUMNS, PERSISTED_RECORD_COLUMNS,
    RECORD_ALIAS_COLUMNS, RECORD_COLUMNS, RECORD_METRIC_COLUMNS, RECORD_TRAIT_COLUMNS,
    RECORDS_FTS_COLUMNS, REFERENCE_EDGE_COLUMNS, REMASTER_LINK_COLUMNS, SPELL_RECORD_COLUMNS,
    Table, actor_records, item_records, record_aliases, record_metrics, record_traits, records,
    records_fts, reference_edges, remaster_links, spell_records,
};

pub fn record_insert_sql() -> String {
    insert_sql(records::TABLE, RECORD_COLUMNS)
}

pub fn record_trait_insert_sql() -> String {
    insert_sql(record_traits::TABLE, RECORD_TRAIT_COLUMNS)
}

pub fn record_metric_insert_sql() -> String {
    insert_sql(record_metrics::TABLE, RECORD_METRIC_COLUMNS)
}

pub fn actor_record_insert_sql() -> String {
    insert_sql(actor_records::TABLE, ACTOR_RECORD_COLUMNS)
}

pub fn item_record_insert_sql() -> String {
    insert_sql(item_records::TABLE, ITEM_RECORD_COLUMNS)
}

pub fn spell_record_insert_sql() -> String {
    insert_sql(spell_records::TABLE, SPELL_RECORD_COLUMNS)
}

pub fn records_fts_insert_sql() -> String {
    insert_sql(records_fts::TABLE, RECORDS_FTS_COLUMNS)
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

pub const CREATE_ARTIFACT_SCHEMA_SQL: &str = r#"            PRAGMA foreign_keys = ON;

            CREATE TABLE artifact_metadata (
              key TEXT PRIMARY KEY,
              value TEXT NOT NULL
            );

            CREATE TABLE packs (
              name TEXT PRIMARY KEY,
              label TEXT NOT NULL,
              document_type TEXT NOT NULL,
              declared_path TEXT NOT NULL,
              resolved_path TEXT NOT NULL,
              record_count INTEGER NOT NULL
            );

            CREATE TABLE records (
              record_key TEXT PRIMARY KEY,
              id TEXT NOT NULL,
              name TEXT NOT NULL,
              normalized_name TEXT NOT NULL,
              record_family TEXT NOT NULL,
              pack_name TEXT NOT NULL,
              pack_label TEXT NOT NULL,
              foundry_document_type TEXT NOT NULL,
              foundry_record_type TEXT NOT NULL,
              level INTEGER,
              rarity TEXT,
              traits_json TEXT NOT NULL,
              system_category TEXT,
              system_group TEXT,
              system_base_item TEXT,
              system_usage TEXT,
              system_price_json TEXT,
              system_actions_value INTEGER,
              system_time_value TEXT,
              system_duration_value TEXT,
              price_cp INTEGER,
              activation_time_kind TEXT,
              activation_time_actions INTEGER,
              activation_time_duration_value INTEGER,
              activation_time_duration_unit TEXT,
              activation_time_text TEXT,
              duration_kind TEXT,
              duration_value INTEGER,
              duration_unit TEXT,
              duration_text TEXT,
              publication_title TEXT,
              publication_remaster INTEGER NOT NULL CHECK (publication_remaster IN (0, 1)),
              description_text TEXT,
              blurb_text TEXT,
              description_snippet TEXT,
              publication_family TEXT NOT NULL,
              folder_id TEXT,
              taxonomy_families_json TEXT NOT NULL,
              variant_group_key TEXT,
              variant_base_name TEXT,
              variant_label TEXT,
              variant_axes_json TEXT NOT NULL,
              variant_confidence REAL,
              variant_source TEXT NOT NULL,
              source_path TEXT NOT NULL,
              is_default_visible INTEGER NOT NULL CHECK (is_default_visible IN (0, 1)),
              search_text_projection TEXT NOT NULL,
              raw_json TEXT NOT NULL
            );

            CREATE TABLE record_traits (
              record_key TEXT NOT NULL,
              trait TEXT NOT NULL,
              PRIMARY KEY (record_key, trait),
              FOREIGN KEY (record_key) REFERENCES records(record_key) ON DELETE CASCADE
            );

            CREATE TABLE reference_edges (
              from_record_key TEXT NOT NULL,
              to_record_key TEXT NOT NULL,
              display_text TEXT,
              reference_text TEXT NOT NULL,
              PRIMARY KEY (from_record_key, to_record_key, reference_text),
              FOREIGN KEY (from_record_key) REFERENCES records(record_key) ON DELETE CASCADE,
              FOREIGN KEY (to_record_key) REFERENCES records(record_key) ON DELETE CASCADE
            );

            CREATE TABLE record_aliases (
              canonical_record_key TEXT NOT NULL,
              alias_text TEXT NOT NULL,
              normalized_alias TEXT NOT NULL,
              source_kind TEXT NOT NULL CHECK (source_kind IN ('remaster_journal', 'migration', 'compendium_source')),
              source_ref TEXT NOT NULL,
              PRIMARY KEY (canonical_record_key, normalized_alias, source_kind, source_ref),
              FOREIGN KEY (canonical_record_key) REFERENCES records(record_key) ON DELETE CASCADE
            );

            CREATE TABLE remaster_links (
              remaster_record_key TEXT NOT NULL,
              legacy_record_key TEXT NOT NULL,
              source_kind TEXT NOT NULL CHECK (source_kind IN ('remaster_journal', 'migration')),
              source_ref TEXT NOT NULL,
              PRIMARY KEY (remaster_record_key, legacy_record_key, source_kind, source_ref),
              FOREIGN KEY (remaster_record_key) REFERENCES records(record_key) ON DELETE CASCADE,
              FOREIGN KEY (legacy_record_key) REFERENCES records(record_key) ON DELETE CASCADE
            );

            CREATE TABLE record_metrics (
              record_key TEXT NOT NULL,
              metric_domain TEXT NOT NULL CHECK (metric_domain IN ('actor', 'item')),
              metric_key TEXT NOT NULL,
              value_type TEXT NOT NULL CHECK (value_type IN ('number', 'text', 'boolean')),
              number_value REAL,
              text_value TEXT,
              bool_value INTEGER CHECK (bool_value IN (0, 1)),
              PRIMARY KEY (record_key, metric_domain, metric_key),
              FOREIGN KEY (record_key) REFERENCES records(record_key) ON DELETE CASCADE
            );

            CREATE TABLE metric_key_catalog (
              metric_domain TEXT NOT NULL CHECK (metric_domain IN ('actor', 'item')),
              record_family TEXT NOT NULL,
              namespace_prefix TEXT NOT NULL,
              metric_key TEXT NOT NULL,
              value_type TEXT NOT NULL CHECK (value_type IN ('number', 'text', 'boolean')),
              catalog_count INTEGER NOT NULL,
              numeric_min REAL,
              numeric_max REAL,
              PRIMARY KEY (metric_domain, record_family, metric_key)
            );

            CREATE TABLE metric_value_catalog (
              metric_domain TEXT NOT NULL CHECK (metric_domain IN ('actor', 'item')),
              record_family TEXT NOT NULL,
              metric_key TEXT NOT NULL,
              value TEXT NOT NULL,
              catalog_count INTEGER NOT NULL,
              PRIMARY KEY (metric_domain, record_family, metric_key, value)
            );

            CREATE TABLE actor_records (
              record_key TEXT PRIMARY KEY,
              size TEXT,
              languages_json TEXT NOT NULL,
              speed_types_json TEXT NOT NULL,
              senses_json TEXT NOT NULL,
              immunities_json TEXT NOT NULL,
              resistances_json TEXT NOT NULL,
              weaknesses_json TEXT NOT NULL,
              disable_text TEXT,
              disable_skills_json TEXT NOT NULL,
              is_complex INTEGER NOT NULL CHECK (is_complex IN (0, 1)),
              FOREIGN KEY (record_key) REFERENCES records(record_key) ON DELETE CASCADE
            );

            CREATE TABLE item_records (
              record_key TEXT PRIMARY KEY,
              system_category TEXT,
              system_base_item TEXT,
              system_group TEXT,
              system_usage TEXT,
              price_cp INTEGER,
              bulk_value REAL,
              hands_requirement TEXT,
              damage_types_json TEXT NOT NULL,
              FOREIGN KEY (record_key) REFERENCES records(record_key) ON DELETE CASCADE
            );

            CREATE TABLE spell_records (
              record_key TEXT PRIMARY KEY,
              traditions_json TEXT NOT NULL,
              spell_kinds_json TEXT NOT NULL,
              range_text TEXT,
              range_value REAL,
              target_text TEXT,
              area_type TEXT,
              area_value REAL,
              save_type TEXT,
              sustained INTEGER NOT NULL CHECK (sustained IN (0, 1)),
              basic_save INTEGER NOT NULL CHECK (basic_save IN (0, 1)),
              damage_types_json TEXT NOT NULL,
              FOREIGN KEY (record_key) REFERENCES records(record_key) ON DELETE CASCADE
            );

            CREATE VIRTUAL TABLE records_fts USING fts5(
              record_key UNINDEXED,
              name,
              search_text_projection
            );

            CREATE INDEX records_pack_name_idx ON records(pack_name);
            CREATE INDEX records_default_visible_idx ON records(is_default_visible);
            CREATE INDEX reference_edges_from_idx ON reference_edges(from_record_key);
            CREATE INDEX reference_edges_to_idx ON reference_edges(to_record_key);
            CREATE INDEX record_aliases_canonical_idx ON record_aliases(canonical_record_key);
            CREATE INDEX record_aliases_normalized_alias_idx ON record_aliases(normalized_alias);
            CREATE INDEX remaster_links_remaster_idx ON remaster_links(remaster_record_key);
            CREATE INDEX remaster_links_legacy_idx ON remaster_links(legacy_record_key);
            CREATE INDEX record_metrics_record_idx ON record_metrics(record_key);
            CREATE INDEX record_metrics_catalog_source_idx ON record_metrics(metric_domain, metric_key, value_type);
            CREATE INDEX metric_key_catalog_coverage_idx ON metric_key_catalog(metric_domain, record_family, metric_key);
            CREATE INDEX metric_value_catalog_coverage_idx ON metric_value_catalog(metric_domain, record_family, metric_key, value);
"#;

#[cfg(test)]
mod tests {
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
            "INSERT INTO records_fts (record_key, name, search_text_projection) VALUES (?1, ?2, ?3)"
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
        assert!(!PERSISTED_RECORD_COLUMNS.contains(&records::columns::DESCRIPTION_SNIPPET));
        assert!(RECORD_COLUMNS.contains(&records::columns::DESCRIPTION_SNIPPET));
    }
}
