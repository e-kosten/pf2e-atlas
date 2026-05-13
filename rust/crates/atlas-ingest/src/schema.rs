#![allow(clippy::too_many_lines)]

use rusqlite::Connection;

use crate::IngestError;

pub(crate) fn create_artifact_schema(connection: &Connection) -> Result<(), IngestError> {
    connection
        .execute_batch(
            "
            PRAGMA foreign_keys = ON;

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
            ",
        )
        .map_err(|error| IngestError::ArtifactWriteFailed(error.to_string()))
}
