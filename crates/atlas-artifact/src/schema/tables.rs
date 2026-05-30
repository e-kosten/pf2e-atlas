use super::descriptors::{
    blob_not_null, boolean, boolean_not_null, cascade_foreign_key, closed_text_not_null, integer,
    integer_not_null, primary_key, real, text, text_not_null, text_primary_key,
};
use super::{Column, ColumnDescriptor, Table, TableConstraint, TableDescriptor};

mod discovery;
mod embeddings;
mod identity;
mod metrics;
mod record_tables;
mod relationships;
mod search;

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

const CONTENT_SOURCE_KINDS: &[&str] = &[
    "description",
    "blurb",
    "disable",
    "routine",
    "reset",
    "stealth_details",
    "details_description",
    "public_notes",
    "gm_notes",
    "private_notes",
    "embedded_item_description",
    "embedded_spell_description",
    "generated_affliction",
];
const CONTENT_VISIBILITIES: &[&str] = &["public", "gm_only", "private", "internal"];
const ALIAS_SOURCES: &[&str] = &["remaster_journal", "migration", "compendium_source"];
const REMASTER_LINK_SOURCES: &[&str] = &["remaster_journal", "migration"];
const METRIC_DOMAINS: &[&str] = &["actor", "item"];
const METRIC_VALUE_TYPES: &[&str] = &["number", "text", "boolean"];

pub const TABLE_ARTIFACT_METADATA: &str = artifact_metadata::TABLE.name();
pub const TABLE_PACKS: &str = packs::TABLE.name();
pub const TABLE_RECORDS: &str = records::TABLE.name();
pub const TABLE_RECORD_CONTENT: &str = record_content::TABLE.name();
pub const TABLE_RECORD_TRAITS: &str = record_traits::TABLE.name();
pub const TABLE_REFERENCE_EDGES: &str = reference_edges::TABLE.name();
pub const TABLE_REFERENCE_OCCURRENCES: &str = reference_occurrences::TABLE.name();
pub const TABLE_RECORD_ALIASES: &str = record_aliases::TABLE.name();
pub const TABLE_REMASTER_LINKS: &str = remaster_links::TABLE.name();
pub const TABLE_RECORD_METRICS: &str = record_metrics::TABLE.name();
pub const TABLE_METRIC_KEY_CATALOG: &str = metric_key_catalog::TABLE.name();
pub const TABLE_METRIC_VALUE_CATALOG: &str = metric_value_catalog::TABLE.name();
pub const TABLE_FILTER_FIELD_CATALOG: &str = filter_field_catalog::TABLE.name();
pub const TABLE_FILTER_VALUE_CATALOG: &str = filter_value_catalog::TABLE.name();
pub const TABLE_FILTER_SAMPLE_CATALOG: &str = filter_sample_catalog::TABLE.name();
pub const TABLE_FILTER_NUMERIC_CATALOG: &str = filter_numeric_catalog::TABLE.name();
pub const TABLE_ACTOR_RECORDS: &str = actor_records::TABLE.name();
pub const TABLE_ITEM_RECORDS: &str = item_records::TABLE.name();
pub const TABLE_SPELL_RECORDS: &str = spell_records::TABLE.name();
pub const TABLE_RECORDS_FTS: &str = records_fts::TABLE.name();
pub const TABLE_DOCUMENT_EMBEDDING_CACHE: &str = document_embedding_cache::TABLE.name();
pub const TABLE_RECORD_VECTOR_INDEX: &str = record_vector_index::TABLE.name();

pub const RECORD_COLUMNS: &[Column] = records::ALL_COLUMNS;
pub const PERSISTED_RECORD_COLUMNS: &[Column] = records::PERSISTED_COLUMNS;
pub const RECORD_CONTENT_COLUMNS: &[Column] = record_content::ALL_COLUMNS;
pub const RECORD_TRAIT_COLUMNS: &[Column] = record_traits::ALL_COLUMNS;
pub const REFERENCE_EDGE_COLUMNS: &[Column] = reference_edges::ALL_COLUMNS;
pub const REFERENCE_OCCURRENCE_COLUMNS: &[Column] = reference_occurrences::ALL_COLUMNS;
pub const RECORD_ALIAS_COLUMNS: &[Column] = record_aliases::ALL_COLUMNS;
pub const REMASTER_LINK_COLUMNS: &[Column] = remaster_links::ALL_COLUMNS;
pub const RECORD_METRIC_COLUMNS: &[Column] = record_metrics::ALL_COLUMNS;
pub const FILTER_FIELD_CATALOG_COLUMNS: &[Column] = filter_field_catalog::ALL_COLUMNS;
pub const FILTER_VALUE_CATALOG_COLUMNS: &[Column] = filter_value_catalog::ALL_COLUMNS;
pub const FILTER_SAMPLE_CATALOG_COLUMNS: &[Column] = filter_sample_catalog::ALL_COLUMNS;
pub const FILTER_NUMERIC_CATALOG_COLUMNS: &[Column] = filter_numeric_catalog::ALL_COLUMNS;
pub const ACTOR_RECORD_COLUMNS: &[Column] = actor_records::ALL_COLUMNS;
pub const ITEM_RECORD_COLUMNS: &[Column] = item_records::ALL_COLUMNS;
pub const SPELL_RECORD_COLUMNS: &[Column] = spell_records::ALL_COLUMNS;
pub const RECORDS_FTS_COLUMNS: &[Column] = records_fts::ALL_COLUMNS;
pub const DOCUMENT_EMBEDDING_CACHE_COLUMNS: &[Column] = document_embedding_cache::ALL_COLUMNS;
pub const RECORD_VECTOR_INDEX_COLUMNS: &[Column] = record_vector_index::ALL_COLUMNS;

const ARTIFACT_METADATA_COLUMN_DESCRIPTORS: &[ColumnDescriptor] = &[
    text_primary_key(artifact_metadata::columns::KEY),
    text_not_null(artifact_metadata::columns::VALUE),
];

const PACK_COLUMN_DESCRIPTORS: &[ColumnDescriptor] = &[
    text_primary_key(packs::columns::NAME),
    text_not_null(packs::columns::LABEL),
    text_not_null(packs::columns::DOCUMENT_TYPE),
    text_not_null(packs::columns::DECLARED_PATH),
    text_not_null(packs::columns::RESOLVED_PATH),
    integer_not_null(packs::columns::RECORD_COUNT),
];

const RECORD_COLUMN_DESCRIPTORS: &[ColumnDescriptor] = &[
    text_primary_key(records::columns::RECORD_KEY),
    text_not_null(records::columns::ID),
    text_not_null(records::columns::NAME),
    text_not_null(records::columns::NORMALIZED_NAME),
    text_not_null(records::columns::RECORD_FAMILY),
    text_not_null(records::columns::PACK_NAME),
    text_not_null(records::columns::PACK_LABEL),
    text_not_null(records::columns::FOUNDRY_DOCUMENT_TYPE),
    text_not_null(records::columns::FOUNDRY_RECORD_TYPE),
    integer(records::columns::LEVEL),
    text(records::columns::RARITY),
    text_not_null(records::columns::TRAITS_JSON),
    text_not_null(records::columns::PREREQUISITES_JSON),
    text(records::columns::SYSTEM_CATEGORY),
    text(records::columns::SYSTEM_GROUP),
    text(records::columns::SYSTEM_BASE_ITEM),
    text(records::columns::SYSTEM_USAGE),
    text(records::columns::SYSTEM_PRICE_JSON),
    integer(records::columns::SYSTEM_ACTIONS_VALUE),
    text(records::columns::SYSTEM_TIME_VALUE),
    text(records::columns::SYSTEM_DURATION_VALUE),
    integer(records::columns::PRICE_CP),
    text(records::columns::ACTIVATION_TIME_KIND),
    integer(records::columns::ACTIVATION_TIME_ACTIONS),
    integer(records::columns::ACTIVATION_TIME_DURATION_VALUE),
    text(records::columns::ACTIVATION_TIME_DURATION_UNIT),
    text(records::columns::ACTIVATION_TIME_TEXT),
    text(records::columns::DURATION_KIND),
    integer(records::columns::DURATION_VALUE),
    text(records::columns::DURATION_UNIT),
    text(records::columns::DURATION_TEXT),
    text(records::columns::PUBLICATION_TITLE),
    boolean_not_null(records::columns::PUBLICATION_REMASTER),
    text(records::columns::DESCRIPTION_JSON),
    text(records::columns::BLURB_JSON),
    text_not_null(records::columns::PUBLICATION_FAMILY),
    text(records::columns::FOLDER_ID),
    text_not_null(records::columns::TAXONOMY_FAMILIES_JSON),
    text(records::columns::VARIANT_GROUP_KEY),
    text(records::columns::VARIANT_BASE_NAME),
    text(records::columns::VARIANT_LABEL),
    text_not_null(records::columns::VARIANT_AXES_JSON),
    real(records::columns::VARIANT_CONFIDENCE),
    text_not_null(records::columns::VARIANT_SOURCE),
    text_not_null(records::columns::SOURCE_PATH),
    boolean_not_null(records::columns::IS_DEFAULT_VISIBLE),
    text_not_null(records::columns::RAW_JSON),
];

const RECORD_CONTENT_COLUMN_DESCRIPTORS: &[ColumnDescriptor] = &[
    text_not_null(record_content::columns::RECORD_KEY),
    text_not_null(record_content::columns::CONTENT_KEY),
    integer_not_null(record_content::columns::ORDINAL),
    closed_text_not_null(record_content::columns::SOURCE_KIND, CONTENT_SOURCE_KINDS),
    closed_text_not_null(record_content::columns::VISIBILITY, CONTENT_VISIBILITIES),
    boolean_not_null(record_content::columns::CONTRIBUTES_TO_SEARCH),
    boolean_not_null(record_content::columns::CONTRIBUTES_TO_REFERENCES),
    text(record_content::columns::LABEL),
    text_not_null(record_content::columns::CONTENT_JSON),
];
const RECORD_CONTENT_CONSTRAINTS: &[TableConstraint] = &[
    primary_key(&[
        record_content::columns::RECORD_KEY,
        record_content::columns::CONTENT_KEY,
    ]),
    cascade_foreign_key(
        &[record_content::columns::RECORD_KEY],
        records::TABLE,
        &[records::columns::RECORD_KEY],
    ),
];

const RECORD_TRAIT_COLUMN_DESCRIPTORS: &[ColumnDescriptor] = &[
    text_not_null(record_traits::columns::RECORD_KEY),
    text_not_null(record_traits::columns::TRAIT),
];
const RECORD_TRAIT_CONSTRAINTS: &[TableConstraint] = &[
    primary_key(&[
        record_traits::columns::RECORD_KEY,
        record_traits::columns::TRAIT,
    ]),
    cascade_foreign_key(
        &[record_traits::columns::RECORD_KEY],
        records::TABLE,
        &[records::columns::RECORD_KEY],
    ),
];

const REFERENCE_EDGE_COLUMN_DESCRIPTORS: &[ColumnDescriptor] = &[
    text_not_null(reference_edges::columns::FROM_RECORD_KEY),
    text_not_null(reference_edges::columns::TO_RECORD_KEY),
    text(reference_edges::columns::DISPLAY_TEXT),
    text_not_null(reference_edges::columns::REFERENCE_TEXT),
    closed_text_not_null(reference_edges::columns::SOURCE_KIND, CONTENT_SOURCE_KINDS),
    closed_text_not_null(reference_edges::columns::VISIBILITY, CONTENT_VISIBILITIES),
];
const REFERENCE_EDGE_CONSTRAINTS: &[TableConstraint] = &[
    primary_key(&[
        reference_edges::columns::FROM_RECORD_KEY,
        reference_edges::columns::TO_RECORD_KEY,
        reference_edges::columns::REFERENCE_TEXT,
        reference_edges::columns::SOURCE_KIND,
    ]),
    cascade_foreign_key(
        &[reference_edges::columns::FROM_RECORD_KEY],
        records::TABLE,
        &[records::columns::RECORD_KEY],
    ),
    cascade_foreign_key(
        &[reference_edges::columns::TO_RECORD_KEY],
        records::TABLE,
        &[records::columns::RECORD_KEY],
    ),
];

const REFERENCE_OCCURRENCE_COLUMN_DESCRIPTORS: &[ColumnDescriptor] = &[
    text_not_null(reference_occurrences::columns::RECORD_KEY),
    text_not_null(reference_occurrences::columns::CONTENT_KEY),
    integer_not_null(reference_occurrences::columns::OCCURRENCE_ORDINAL),
    text_not_null(reference_occurrences::columns::TARGET_RECORD_KEY),
    closed_text_not_null(
        reference_occurrences::columns::SOURCE_KIND,
        CONTENT_SOURCE_KINDS,
    ),
    closed_text_not_null(
        reference_occurrences::columns::VISIBILITY,
        CONTENT_VISIBILITIES,
    ),
    text(reference_occurrences::columns::DISPLAY_TEXT),
    text_not_null(reference_occurrences::columns::REFERENCE_TEXT),
];
const REFERENCE_OCCURRENCE_CONSTRAINTS: &[TableConstraint] = &[
    primary_key(&[
        reference_occurrences::columns::RECORD_KEY,
        reference_occurrences::columns::CONTENT_KEY,
        reference_occurrences::columns::OCCURRENCE_ORDINAL,
    ]),
    cascade_foreign_key(
        &[reference_occurrences::columns::RECORD_KEY],
        records::TABLE,
        &[records::columns::RECORD_KEY],
    ),
    cascade_foreign_key(
        &[reference_occurrences::columns::TARGET_RECORD_KEY],
        records::TABLE,
        &[records::columns::RECORD_KEY],
    ),
];

const RECORD_ALIAS_COLUMN_DESCRIPTORS: &[ColumnDescriptor] = &[
    text_not_null(record_aliases::columns::CANONICAL_RECORD_KEY),
    text_not_null(record_aliases::columns::ALIAS_TEXT),
    text_not_null(record_aliases::columns::NORMALIZED_ALIAS),
    closed_text_not_null(record_aliases::columns::SOURCE_KIND, ALIAS_SOURCES),
    text_not_null(record_aliases::columns::SOURCE_REF),
];
const RECORD_ALIAS_CONSTRAINTS: &[TableConstraint] = &[
    primary_key(&[
        record_aliases::columns::CANONICAL_RECORD_KEY,
        record_aliases::columns::NORMALIZED_ALIAS,
        record_aliases::columns::SOURCE_KIND,
        record_aliases::columns::SOURCE_REF,
    ]),
    cascade_foreign_key(
        &[record_aliases::columns::CANONICAL_RECORD_KEY],
        records::TABLE,
        &[records::columns::RECORD_KEY],
    ),
];

const REMASTER_LINK_COLUMN_DESCRIPTORS: &[ColumnDescriptor] = &[
    text_not_null(remaster_links::columns::REMASTER_RECORD_KEY),
    text_not_null(remaster_links::columns::LEGACY_RECORD_KEY),
    closed_text_not_null(remaster_links::columns::SOURCE_KIND, REMASTER_LINK_SOURCES),
    text_not_null(remaster_links::columns::SOURCE_REF),
];
const REMASTER_LINK_CONSTRAINTS: &[TableConstraint] = &[
    primary_key(&[
        remaster_links::columns::REMASTER_RECORD_KEY,
        remaster_links::columns::LEGACY_RECORD_KEY,
        remaster_links::columns::SOURCE_KIND,
        remaster_links::columns::SOURCE_REF,
    ]),
    cascade_foreign_key(
        &[remaster_links::columns::REMASTER_RECORD_KEY],
        records::TABLE,
        &[records::columns::RECORD_KEY],
    ),
    cascade_foreign_key(
        &[remaster_links::columns::LEGACY_RECORD_KEY],
        records::TABLE,
        &[records::columns::RECORD_KEY],
    ),
];

const RECORD_METRIC_COLUMN_DESCRIPTORS: &[ColumnDescriptor] = &[
    text_not_null(record_metrics::columns::RECORD_KEY),
    closed_text_not_null(record_metrics::columns::METRIC_DOMAIN, METRIC_DOMAINS),
    text_not_null(record_metrics::columns::METRIC_KEY),
    closed_text_not_null(record_metrics::columns::VALUE_TYPE, METRIC_VALUE_TYPES),
    real(record_metrics::columns::NUMBER_VALUE),
    text(record_metrics::columns::TEXT_VALUE),
    boolean(record_metrics::columns::BOOL_VALUE),
];
const RECORD_METRIC_CONSTRAINTS: &[TableConstraint] = &[
    primary_key(&[
        record_metrics::columns::RECORD_KEY,
        record_metrics::columns::METRIC_DOMAIN,
        record_metrics::columns::METRIC_KEY,
    ]),
    cascade_foreign_key(
        &[record_metrics::columns::RECORD_KEY],
        records::TABLE,
        &[records::columns::RECORD_KEY],
    ),
];

const METRIC_KEY_CATALOG_COLUMN_DESCRIPTORS: &[ColumnDescriptor] = &[
    closed_text_not_null(metric_key_catalog::columns::METRIC_DOMAIN, METRIC_DOMAINS),
    text(metric_key_catalog::columns::RECORD_FAMILY),
    text_not_null(metric_key_catalog::columns::NAMESPACE_PREFIX),
    text_not_null(metric_key_catalog::columns::METRIC_KEY),
    closed_text_not_null(metric_key_catalog::columns::VALUE_TYPE, METRIC_VALUE_TYPES),
    integer_not_null(metric_key_catalog::columns::CATALOG_COUNT),
    real(metric_key_catalog::columns::NUMERIC_MIN),
    real(metric_key_catalog::columns::NUMERIC_MAX),
];
const METRIC_KEY_CATALOG_CONSTRAINTS: &[TableConstraint] = &[primary_key(&[
    metric_key_catalog::columns::METRIC_DOMAIN,
    metric_key_catalog::columns::RECORD_FAMILY,
    metric_key_catalog::columns::METRIC_KEY,
])];

const METRIC_VALUE_CATALOG_COLUMN_DESCRIPTORS: &[ColumnDescriptor] = &[
    closed_text_not_null(metric_value_catalog::columns::METRIC_DOMAIN, METRIC_DOMAINS),
    text(metric_value_catalog::columns::RECORD_FAMILY),
    text_not_null(metric_value_catalog::columns::METRIC_KEY),
    text_not_null(metric_value_catalog::columns::VALUE),
    integer_not_null(metric_value_catalog::columns::CATALOG_COUNT),
];
const METRIC_VALUE_CATALOG_CONSTRAINTS: &[TableConstraint] = &[primary_key(&[
    metric_value_catalog::columns::METRIC_DOMAIN,
    metric_value_catalog::columns::RECORD_FAMILY,
    metric_value_catalog::columns::METRIC_KEY,
    metric_value_catalog::columns::VALUE,
])];

const FILTER_FIELD_CATALOG_COLUMN_DESCRIPTORS: &[ColumnDescriptor] = &[
    text_not_null(filter_field_catalog::columns::FIELD),
    text(filter_field_catalog::columns::RECORD_FAMILY),
    text_not_null(filter_field_catalog::columns::FIELD_TYPE),
    text_not_null(filter_field_catalog::columns::FIELD_GROUP),
    text_not_null(filter_field_catalog::columns::VALUE_POLICY),
    text_not_null(filter_field_catalog::columns::OPERATORS_JSON),
    text_not_null(filter_field_catalog::columns::CLI_FLAGS_JSON),
    text_not_null(filter_field_catalog::columns::APPLICABLE_FAMILIES_JSON),
    integer_not_null(filter_field_catalog::columns::VALUE_COUNT),
    integer_not_null(filter_field_catalog::columns::MATCHING_RECORD_COUNT),
    integer_not_null(filter_field_catalog::columns::NULL_COUNT),
    integer_not_null(filter_field_catalog::columns::DISTINCT_COUNT),
    integer_not_null(filter_field_catalog::columns::SINGLETON_COUNT),
    real(filter_field_catalog::columns::SINGLETON_RATIO),
    real(filter_field_catalog::columns::OBSERVATION_SINGLETON_RATIO),
    text_not_null(filter_field_catalog::columns::POLICY_REASON),
];
const FILTER_FIELD_CATALOG_CONSTRAINTS: &[TableConstraint] = &[primary_key(&[
    filter_field_catalog::columns::FIELD,
    filter_field_catalog::columns::RECORD_FAMILY,
])];

const FILTER_VALUE_CATALOG_COLUMN_DESCRIPTORS: &[ColumnDescriptor] = &[
    text_not_null(filter_value_catalog::columns::FIELD),
    text(filter_value_catalog::columns::RECORD_FAMILY),
    text_not_null(filter_value_catalog::columns::VALUE),
    integer_not_null(filter_value_catalog::columns::CATALOG_COUNT),
];
const FILTER_VALUE_CATALOG_CONSTRAINTS: &[TableConstraint] = &[primary_key(&[
    filter_value_catalog::columns::FIELD,
    filter_value_catalog::columns::RECORD_FAMILY,
    filter_value_catalog::columns::VALUE,
])];

const FILTER_SAMPLE_CATALOG_COLUMN_DESCRIPTORS: &[ColumnDescriptor] = &[
    text_not_null(filter_sample_catalog::columns::FIELD),
    text(filter_sample_catalog::columns::RECORD_FAMILY),
    text_not_null(filter_sample_catalog::columns::VALUE),
    integer_not_null(filter_sample_catalog::columns::CATALOG_COUNT),
    integer_not_null(filter_sample_catalog::columns::SAMPLE_RANK),
];
const FILTER_SAMPLE_CATALOG_CONSTRAINTS: &[TableConstraint] = &[primary_key(&[
    filter_sample_catalog::columns::FIELD,
    filter_sample_catalog::columns::RECORD_FAMILY,
    filter_sample_catalog::columns::VALUE,
])];

const FILTER_NUMERIC_CATALOG_COLUMN_DESCRIPTORS: &[ColumnDescriptor] = &[
    text_not_null(filter_numeric_catalog::columns::FIELD),
    text(filter_numeric_catalog::columns::RECORD_FAMILY),
    text(filter_numeric_catalog::columns::METRIC_DOMAIN),
    text(filter_numeric_catalog::columns::METRIC_KEY),
    integer_not_null(filter_numeric_catalog::columns::CATALOG_COUNT),
    integer_not_null(filter_numeric_catalog::columns::NULL_COUNT),
    real(filter_numeric_catalog::columns::MIN),
    real(filter_numeric_catalog::columns::P05),
    real(filter_numeric_catalog::columns::P25),
    real(filter_numeric_catalog::columns::P50),
    real(filter_numeric_catalog::columns::MEAN),
    real(filter_numeric_catalog::columns::P75),
    real(filter_numeric_catalog::columns::P95),
    real(filter_numeric_catalog::columns::MAX),
];
const FILTER_NUMERIC_CATALOG_CONSTRAINTS: &[TableConstraint] = &[primary_key(&[
    filter_numeric_catalog::columns::FIELD,
    filter_numeric_catalog::columns::RECORD_FAMILY,
    filter_numeric_catalog::columns::METRIC_DOMAIN,
    filter_numeric_catalog::columns::METRIC_KEY,
])];

const ACTOR_RECORD_COLUMN_DESCRIPTORS: &[ColumnDescriptor] = &[
    text_primary_key(actor_records::columns::RECORD_KEY),
    text(actor_records::columns::SIZE),
    text_not_null(actor_records::columns::LANGUAGES_JSON),
    text_not_null(actor_records::columns::SPEED_TYPES_JSON),
    text_not_null(actor_records::columns::SENSES_JSON),
    text_not_null(actor_records::columns::IMMUNITIES_JSON),
    text_not_null(actor_records::columns::RESISTANCES_JSON),
    text_not_null(actor_records::columns::WEAKNESSES_JSON),
    text(actor_records::columns::DISABLE_TEXT),
    text_not_null(actor_records::columns::DISABLE_SKILLS_JSON),
    boolean_not_null(actor_records::columns::IS_COMPLEX),
];
const ACTOR_RECORD_CONSTRAINTS: &[TableConstraint] = &[cascade_foreign_key(
    &[actor_records::columns::RECORD_KEY],
    records::TABLE,
    &[records::columns::RECORD_KEY],
)];

const ITEM_RECORD_COLUMN_DESCRIPTORS: &[ColumnDescriptor] = &[
    text_primary_key(item_records::columns::RECORD_KEY),
    text(item_records::columns::SYSTEM_CATEGORY),
    text(item_records::columns::SYSTEM_BASE_ITEM),
    text(item_records::columns::SYSTEM_GROUP),
    text(item_records::columns::SYSTEM_USAGE),
    integer(item_records::columns::PRICE_CP),
    real(item_records::columns::BULK_VALUE),
    text(item_records::columns::HANDS_REQUIREMENT),
    text_not_null(item_records::columns::DAMAGE_TYPES_JSON),
];
const ITEM_RECORD_CONSTRAINTS: &[TableConstraint] = &[cascade_foreign_key(
    &[item_records::columns::RECORD_KEY],
    records::TABLE,
    &[records::columns::RECORD_KEY],
)];

const SPELL_RECORD_COLUMN_DESCRIPTORS: &[ColumnDescriptor] = &[
    text_primary_key(spell_records::columns::RECORD_KEY),
    text_not_null(spell_records::columns::TRADITIONS_JSON),
    text_not_null(spell_records::columns::SPELL_KINDS_JSON),
    text(spell_records::columns::RANGE_TEXT),
    real(spell_records::columns::RANGE_VALUE),
    text(spell_records::columns::TARGET_TEXT),
    text(spell_records::columns::AREA_TYPE),
    real(spell_records::columns::AREA_VALUE),
    text(spell_records::columns::SAVE_TYPE),
    boolean_not_null(spell_records::columns::SUSTAINED),
    boolean_not_null(spell_records::columns::BASIC_SAVE),
    text_not_null(spell_records::columns::DAMAGE_TYPES_JSON),
];
const SPELL_RECORD_CONSTRAINTS: &[TableConstraint] = &[cascade_foreign_key(
    &[spell_records::columns::RECORD_KEY],
    records::TABLE,
    &[records::columns::RECORD_KEY],
)];

const RECORDS_FTS_COLUMN_DESCRIPTORS: &[ColumnDescriptor] = &[
    text_not_null(records_fts::columns::RECORD_KEY),
    text(records_fts::columns::TITLE),
    text(records_fts::columns::ALIASES),
    text(records_fts::columns::TRAITS),
    text(records_fts::columns::TAXONOMY_TERMS),
    text(records_fts::columns::CONSTRAINT_TERMS),
    text(records_fts::columns::MECHANIC_TERMS),
    text(records_fts::columns::SOURCE_TERMS),
    text(records_fts::columns::METRIC_TERMS),
    text(records_fts::columns::HEADINGS),
    text(records_fts::columns::BODY),
    text(records_fts::columns::FACTS),
    text(records_fts::columns::REFERENCE_TERMS),
    text(records_fts::columns::EMBEDDED_CONTENT),
];
const RECORDS_FTS_UNINDEXED_COLUMNS: &[Column] = &[records_fts::columns::RECORD_KEY];

const DOCUMENT_EMBEDDING_CACHE_COLUMN_DESCRIPTORS: &[ColumnDescriptor] = &[
    text_primary_key(document_embedding_cache::columns::EMBEDDING_UNIT_KEY),
    text_not_null(document_embedding_cache::columns::RECORD_KEY),
    text_not_null(document_embedding_cache::columns::UNIT_KIND),
    text(document_embedding_cache::columns::LABEL),
    integer_not_null(document_embedding_cache::columns::ORDINAL),
    text_not_null(document_embedding_cache::columns::SEMANTIC_INPUT_HASH),
    integer_not_null(document_embedding_cache::columns::DIMENSIONS),
    blob_not_null(document_embedding_cache::columns::VECTOR_BLOB),
];
const DOCUMENT_EMBEDDING_CACHE_CONSTRAINTS: &[TableConstraint] = &[cascade_foreign_key(
    &[document_embedding_cache::columns::RECORD_KEY],
    records::TABLE,
    &[records::columns::RECORD_KEY],
)];

pub const TABLE_DESCRIPTORS: &[TableDescriptor] = &[
    TableDescriptor::ordinary(
        artifact_metadata::TABLE,
        ARTIFACT_METADATA_COLUMN_DESCRIPTORS,
        &[],
    ),
    TableDescriptor::ordinary(packs::TABLE, PACK_COLUMN_DESCRIPTORS, &[]),
    TableDescriptor::ordinary(records::TABLE, RECORD_COLUMN_DESCRIPTORS, &[]),
    TableDescriptor::ordinary(
        record_content::TABLE,
        RECORD_CONTENT_COLUMN_DESCRIPTORS,
        RECORD_CONTENT_CONSTRAINTS,
    ),
    TableDescriptor::ordinary(
        record_traits::TABLE,
        RECORD_TRAIT_COLUMN_DESCRIPTORS,
        RECORD_TRAIT_CONSTRAINTS,
    ),
    TableDescriptor::ordinary(
        reference_edges::TABLE,
        REFERENCE_EDGE_COLUMN_DESCRIPTORS,
        REFERENCE_EDGE_CONSTRAINTS,
    ),
    TableDescriptor::ordinary(
        reference_occurrences::TABLE,
        REFERENCE_OCCURRENCE_COLUMN_DESCRIPTORS,
        REFERENCE_OCCURRENCE_CONSTRAINTS,
    ),
    TableDescriptor::ordinary(
        record_aliases::TABLE,
        RECORD_ALIAS_COLUMN_DESCRIPTORS,
        RECORD_ALIAS_CONSTRAINTS,
    ),
    TableDescriptor::ordinary(
        remaster_links::TABLE,
        REMASTER_LINK_COLUMN_DESCRIPTORS,
        REMASTER_LINK_CONSTRAINTS,
    ),
    TableDescriptor::ordinary(
        record_metrics::TABLE,
        RECORD_METRIC_COLUMN_DESCRIPTORS,
        RECORD_METRIC_CONSTRAINTS,
    ),
    TableDescriptor::ordinary(
        metric_key_catalog::TABLE,
        METRIC_KEY_CATALOG_COLUMN_DESCRIPTORS,
        METRIC_KEY_CATALOG_CONSTRAINTS,
    ),
    TableDescriptor::ordinary(
        metric_value_catalog::TABLE,
        METRIC_VALUE_CATALOG_COLUMN_DESCRIPTORS,
        METRIC_VALUE_CATALOG_CONSTRAINTS,
    ),
    TableDescriptor::ordinary(
        filter_field_catalog::TABLE,
        FILTER_FIELD_CATALOG_COLUMN_DESCRIPTORS,
        FILTER_FIELD_CATALOG_CONSTRAINTS,
    ),
    TableDescriptor::ordinary(
        filter_value_catalog::TABLE,
        FILTER_VALUE_CATALOG_COLUMN_DESCRIPTORS,
        FILTER_VALUE_CATALOG_CONSTRAINTS,
    ),
    TableDescriptor::ordinary(
        filter_sample_catalog::TABLE,
        FILTER_SAMPLE_CATALOG_COLUMN_DESCRIPTORS,
        FILTER_SAMPLE_CATALOG_CONSTRAINTS,
    ),
    TableDescriptor::ordinary(
        filter_numeric_catalog::TABLE,
        FILTER_NUMERIC_CATALOG_COLUMN_DESCRIPTORS,
        FILTER_NUMERIC_CATALOG_CONSTRAINTS,
    ),
    TableDescriptor::ordinary(
        actor_records::TABLE,
        ACTOR_RECORD_COLUMN_DESCRIPTORS,
        ACTOR_RECORD_CONSTRAINTS,
    ),
    TableDescriptor::ordinary(
        item_records::TABLE,
        ITEM_RECORD_COLUMN_DESCRIPTORS,
        ITEM_RECORD_CONSTRAINTS,
    ),
    TableDescriptor::ordinary(
        spell_records::TABLE,
        SPELL_RECORD_COLUMN_DESCRIPTORS,
        SPELL_RECORD_CONSTRAINTS,
    ),
    TableDescriptor::fts5(
        records_fts::TABLE,
        RECORDS_FTS_COLUMN_DESCRIPTORS,
        RECORDS_FTS_UNINDEXED_COLUMNS,
    ),
    TableDescriptor::ordinary(
        document_embedding_cache::TABLE,
        DOCUMENT_EMBEDDING_CACHE_COLUMN_DESCRIPTORS,
        DOCUMENT_EMBEDDING_CACHE_CONSTRAINTS,
    ),
];

pub fn table_descriptor(table: Table) -> Option<&'static TableDescriptor> {
    TABLE_DESCRIPTORS
        .iter()
        .find(|descriptor| descriptor.table == table)
}

pub fn required_tables() -> Vec<Table> {
    TABLE_DESCRIPTORS
        .iter()
        .map(|descriptor| descriptor.table)
        .collect()
}

pub fn required_columns() -> Vec<(Table, Vec<Column>)> {
    TABLE_DESCRIPTORS
        .iter()
        .filter(|descriptor| descriptor.validate_columns)
        .map(|descriptor| (descriptor.table, descriptor.columns()))
        .collect()
}
