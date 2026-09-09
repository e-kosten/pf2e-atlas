use super::{Column, Table};

mod discovery;
mod embeddings;
mod identity;
mod metrics;
mod record_tables;
mod relationships;
mod search;

pub use canonical::{
    canonical_consumable_spell_children, canonical_creature_entities,
    canonical_creature_occurrences, canonical_creature_records, canonical_creature_relationships,
    canonical_creature_resources, canonical_hazard_entities, canonical_hazard_occurrences,
    canonical_hazard_records, canonical_hazard_relationships, canonical_journal_records,
    canonical_roll_table_records, canonical_spell_records, record_content_exclusions,
    spell_damage_types, spell_traditions,
};
pub use discovery::{
    filter_field_catalog, filter_numeric_catalog, filter_sample_catalog, filter_value_catalog,
};
pub use embeddings::{document_embedding_cache, record_vector_index};
pub use identity::{artifact_metadata, packs};
pub use metrics::{metric_key_catalog, metric_value_catalog, record_metrics};
pub use record_tables::{
    actor_records, item_records, record_content, record_traits, records, spell_records,
};
pub use relationships::{record_aliases, reference_edges, reference_occurrences, remaster_links};
pub use search::records_fts;

pub const TABLE_PACKS: &str = packs::TABLE.name();
pub const TABLE_RECORDS: &str = records::TABLE.name();
pub const TABLE_REFERENCE_EDGES: &str = reference_edges::TABLE.name();
pub const TABLE_REFERENCE_OCCURRENCES: &str = reference_occurrences::TABLE.name();
pub const TABLE_RECORD_ALIASES: &str = record_aliases::TABLE.name();
pub const TABLE_REMASTER_LINKS: &str = remaster_links::TABLE.name();
pub const TABLE_METRIC_VALUE_CATALOG: &str = metric_value_catalog::TABLE.name();
pub const TABLE_RECORDS_FTS: &str = records_fts::TABLE.name();
pub const TABLE_DOCUMENT_EMBEDDING_CACHE: &str = document_embedding_cache::TABLE.name();
pub const TABLE_RECORD_VECTOR_INDEX: &str = record_vector_index::TABLE.name();

pub const REQUIRED_TABLES: &[Table] = &[
    artifact_metadata::TABLE,
    packs::TABLE,
    records::TABLE,
    record_content::TABLE,
    canonical_creature_records::TABLE,
    canonical_creature_resources::TABLE,
    canonical_creature_entities::TABLE,
    canonical_creature_occurrences::TABLE,
    canonical_creature_relationships::TABLE,
    canonical_hazard_records::TABLE,
    canonical_hazard_entities::TABLE,
    canonical_hazard_occurrences::TABLE,
    canonical_hazard_relationships::TABLE,
    canonical_spell_records::TABLE,
    canonical_journal_records::TABLE,
    canonical_roll_table_records::TABLE,
    canonical_consumable_spell_children::TABLE,
    spell_traditions::TABLE,
    spell_damage_types::TABLE,
    record_content_exclusions::TABLE,
    record_traits::TABLE,
    reference_edges::TABLE,
    reference_occurrences::TABLE,
    record_aliases::TABLE,
    remaster_links::TABLE,
    record_metrics::TABLE,
    metric_key_catalog::TABLE,
    metric_value_catalog::TABLE,
    filter_field_catalog::TABLE,
    filter_value_catalog::TABLE,
    filter_sample_catalog::TABLE,
    filter_numeric_catalog::TABLE,
    actor_records::TABLE,
    item_records::TABLE,
    spell_records::TABLE,
    records_fts::TABLE,
    document_embedding_cache::TABLE,
];

pub const REQUIRED_COLUMNS: &[(Table, &[Column])] = &[
    (artifact_metadata::TABLE, artifact_metadata::ALL_COLUMNS),
    (packs::TABLE, packs::ALL_COLUMNS),
    (records::TABLE, records::ALL_COLUMNS),
    (record_content::TABLE, record_content::ALL_COLUMNS),
    (
        canonical_creature_records::TABLE,
        canonical_creature_records::ALL_COLUMNS,
    ),
    (
        canonical_creature_resources::TABLE,
        canonical_creature_resources::ALL_COLUMNS,
    ),
    (
        canonical_creature_entities::TABLE,
        canonical_creature_entities::ALL_COLUMNS,
    ),
    (
        canonical_creature_occurrences::TABLE,
        canonical_creature_occurrences::ALL_COLUMNS,
    ),
    (
        canonical_creature_relationships::TABLE,
        canonical_creature_relationships::ALL_COLUMNS,
    ),
    (
        canonical_hazard_records::TABLE,
        canonical_hazard_records::ALL_COLUMNS,
    ),
    (
        canonical_hazard_entities::TABLE,
        canonical_hazard_entities::ALL_COLUMNS,
    ),
    (
        canonical_hazard_occurrences::TABLE,
        canonical_hazard_occurrences::ALL_COLUMNS,
    ),
    (
        canonical_hazard_relationships::TABLE,
        canonical_hazard_relationships::ALL_COLUMNS,
    ),
    (
        canonical_spell_records::TABLE,
        canonical_spell_records::ALL_COLUMNS,
    ),
    (
        canonical_journal_records::TABLE,
        canonical_journal_records::ALL_COLUMNS,
    ),
    (
        canonical_roll_table_records::TABLE,
        canonical_roll_table_records::ALL_COLUMNS,
    ),
    (
        canonical_consumable_spell_children::TABLE,
        canonical_consumable_spell_children::ALL_COLUMNS,
    ),
    (spell_traditions::TABLE, spell_traditions::ALL_COLUMNS),
    (spell_damage_types::TABLE, spell_damage_types::ALL_COLUMNS),
    (
        record_content_exclusions::TABLE,
        record_content_exclusions::ALL_COLUMNS,
    ),
    (record_traits::TABLE, record_traits::ALL_COLUMNS),
    (reference_edges::TABLE, reference_edges::ALL_COLUMNS),
    (
        reference_occurrences::TABLE,
        reference_occurrences::ALL_COLUMNS,
    ),
    (record_aliases::TABLE, record_aliases::ALL_COLUMNS),
    (remaster_links::TABLE, remaster_links::ALL_COLUMNS),
    (record_metrics::TABLE, record_metrics::ALL_COLUMNS),
    (metric_key_catalog::TABLE, metric_key_catalog::ALL_COLUMNS),
    (
        metric_value_catalog::TABLE,
        metric_value_catalog::ALL_COLUMNS,
    ),
    (
        filter_field_catalog::TABLE,
        filter_field_catalog::ALL_COLUMNS,
    ),
    (
        filter_value_catalog::TABLE,
        filter_value_catalog::ALL_COLUMNS,
    ),
    (
        filter_sample_catalog::TABLE,
        filter_sample_catalog::ALL_COLUMNS,
    ),
    (
        filter_numeric_catalog::TABLE,
        filter_numeric_catalog::ALL_COLUMNS,
    ),
    (actor_records::TABLE, actor_records::ALL_COLUMNS),
    (item_records::TABLE, item_records::ALL_COLUMNS),
    (spell_records::TABLE, spell_records::ALL_COLUMNS),
    (records_fts::TABLE, records_fts::ALL_COLUMNS),
    (
        document_embedding_cache::TABLE,
        document_embedding_cache::ALL_COLUMNS,
    ),
];

pub fn required_tables() -> &'static [Table] {
    REQUIRED_TABLES
}

pub fn required_columns() -> &'static [(Table, &'static [Column])] {
    REQUIRED_COLUMNS
}
mod canonical;
