use crate::artifact::inventory::{
    TABLE_METRIC_VALUE_CATALOG, TABLE_PACKS, TABLE_RECORD_ALIASES, TABLE_RECORDS,
    TABLE_REFERENCE_EDGES, TABLE_REFERENCE_OCCURRENCES, TABLE_REMASTER_LINKS, required_tables,
};
use rusqlite::Connection;
use serde::Serialize;
use std::collections::BTreeMap;

use crate::IndexValidationError;

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct IndexInspectionReport {
    pub status: String,
    pub index: String,
    pub validation: crate::ArtifactValidationReport,
    pub tables: BTreeMap<String, usize>,
    pub records: RecordCoverageReport,
    pub text: TextCoverageReport,
    pub taxonomy: TaxonomyCoverageReport,
    pub variants: VariantCoverageReport,
    pub relationships: RelationshipCoverageReport,
    pub metrics: MetricCoverageReport,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct RecordCoverageReport {
    pub total_records: usize,
    pub default_visible_records: usize,
    pub records_with_level: usize,
    pub records_with_rarity: usize,
    pub by_kind: BTreeMap<String, usize>,
    pub by_foundry_taxonomy: BTreeMap<String, usize>,
    pub by_publication_category: BTreeMap<String, usize>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct TextCoverageReport {
    pub records_with_description: usize,
    pub records_with_blurb: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct TaxonomyCoverageReport {
    pub records_with_taxonomy_families: usize,
    pub distinct_taxonomy_families: usize,
    pub top_taxonomy_families: BTreeMap<String, usize>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct VariantCoverageReport {
    pub grouped_records: usize,
    pub distinct_groups: usize,
    pub by_source: BTreeMap<String, usize>,
    pub by_axis: BTreeMap<String, usize>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct RelationshipCoverageReport {
    pub reference_edges: usize,
    pub reference_occurrences: usize,
    pub record_aliases: usize,
    pub remaster_links: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct MetricCoverageReport {
    pub metric_rows_by_domain: BTreeMap<String, usize>,
    pub metric_keys_by_domain: BTreeMap<String, usize>,
    pub metric_value_catalog_rows: usize,
}

impl IndexInspectionReport {
    pub fn records_table_count(&self) -> usize {
        self.tables.get(TABLE_RECORDS).copied().unwrap_or_default()
    }

    pub fn packs_table_count(&self) -> usize {
        self.tables.get(TABLE_PACKS).copied().unwrap_or_default()
    }

    pub fn reference_edges_table_count(&self) -> usize {
        self.tables
            .get(TABLE_REFERENCE_EDGES)
            .copied()
            .unwrap_or_default()
    }

    pub fn record_aliases_table_count(&self) -> usize {
        self.tables
            .get(TABLE_RECORD_ALIASES)
            .copied()
            .unwrap_or_default()
    }

    pub fn remaster_links_table_count(&self) -> usize {
        self.tables
            .get(TABLE_REMASTER_LINKS)
            .copied()
            .unwrap_or_default()
    }
}

pub(crate) fn inspect_index_connection(
    index: String,
    validation: crate::ArtifactValidationReport,
    connection: &Connection,
) -> Result<IndexInspectionReport, IndexValidationError> {
    if validation.status != crate::ValidationStatus::Ok {
        return Err(IndexValidationError::InvalidArtifact(validation.message));
    }

    Ok(IndexInspectionReport {
        status: "ok".to_string(),
        index,
        validation,
        tables: inspect_tables(connection)?,
        records: inspect_records(connection)?,
        text: inspect_text(connection)?,
        taxonomy: inspect_taxonomy(connection)?,
        variants: inspect_variants(connection)?,
        relationships: inspect_relationships(connection)?,
        metrics: inspect_metrics(connection)?,
    })
}

fn inspect_tables(
    connection: &Connection,
) -> Result<BTreeMap<String, usize>, IndexValidationError> {
    let mut tables = BTreeMap::new();
    for table in required_tables() {
        let table_name = table.name();
        tables.insert(table_name.to_string(), count_rows(connection, table_name)?);
    }
    Ok(tables)
}

fn inspect_records(connection: &Connection) -> Result<RecordCoverageReport, IndexValidationError> {
    Ok(RecordCoverageReport {
        total_records: count_rows(connection, TABLE_RECORDS)?,
        default_visible_records: count_sql(
            connection,
            "SELECT COUNT(*) FROM records WHERE is_default_visible = 1",
        )?,
        records_with_level: count_sql(
            connection,
            "SELECT COUNT(*) FROM records WHERE level IS NOT NULL",
        )?,
        records_with_rarity: count_sql(
            connection,
            "SELECT COUNT(*) FROM records WHERE rarity IS NOT NULL AND TRIM(rarity) <> ''",
        )?,
        by_kind: count_grouped(
            connection,
            "SELECT record_family AS group_key, COUNT(*) AS row_count
             FROM records
             GROUP BY record_family",
        )?,
        by_foundry_taxonomy: count_grouped(
            connection,
            "SELECT foundry_document_type || '|' || foundry_record_type AS group_key,
                    COUNT(*) AS row_count
             FROM records
             GROUP BY foundry_document_type, foundry_record_type",
        )?,
        by_publication_category: count_grouped(
            connection,
            "SELECT publication_family AS group_key, COUNT(*) AS row_count
             FROM records
             GROUP BY publication_family",
        )?,
    })
}

fn inspect_text(connection: &Connection) -> Result<TextCoverageReport, IndexValidationError> {
    Ok(TextCoverageReport {
        records_with_description: count_sql(
            connection,
            "SELECT COUNT(*) FROM records WHERE description_json IS NOT NULL AND TRIM(description_json) <> ''",
        )?,
        records_with_blurb: count_sql(
            connection,
            "SELECT COUNT(*) FROM records WHERE blurb_json IS NOT NULL AND TRIM(blurb_json) <> ''",
        )?,
    })
}

fn inspect_taxonomy(
    connection: &Connection,
) -> Result<TaxonomyCoverageReport, IndexValidationError> {
    Ok(TaxonomyCoverageReport {
        records_with_taxonomy_families: count_sql(
            connection,
            "SELECT COUNT(*) FROM records WHERE taxonomy_families_json <> '[]'",
        )?,
        distinct_taxonomy_families: count_sql(
            connection,
            "SELECT COUNT(DISTINCT taxonomy_family.value)
             FROM records, json_each(records.taxonomy_families_json) AS taxonomy_family",
        )?,
        top_taxonomy_families: count_grouped(
            connection,
            "SELECT taxonomy_family.value AS group_key, COUNT(*) AS row_count
             FROM records, json_each(records.taxonomy_families_json) AS taxonomy_family
             GROUP BY taxonomy_family.value
             ORDER BY COUNT(*) DESC, taxonomy_family.value ASC
             LIMIT 20",
        )?,
    })
}

fn inspect_variants(
    connection: &Connection,
) -> Result<VariantCoverageReport, IndexValidationError> {
    Ok(VariantCoverageReport {
        grouped_records: count_sql(
            connection,
            "SELECT COUNT(*) FROM records WHERE variant_group_key IS NOT NULL",
        )?,
        distinct_groups: count_sql(
            connection,
            "SELECT COUNT(DISTINCT variant_group_key) FROM records WHERE variant_group_key IS NOT NULL",
        )?,
        by_source: count_grouped(
            connection,
            "SELECT variant_source AS group_key, COUNT(*) AS row_count
             FROM records
             WHERE variant_group_key IS NOT NULL
             GROUP BY variant_source",
        )?,
        by_axis: count_grouped(
            connection,
            "SELECT variant_axis.value AS group_key, COUNT(*) AS row_count
             FROM records, json_each(records.variant_axes_json) AS variant_axis
             WHERE records.variant_group_key IS NOT NULL
             GROUP BY variant_axis.value",
        )?,
    })
}

fn inspect_relationships(
    connection: &Connection,
) -> Result<RelationshipCoverageReport, IndexValidationError> {
    Ok(RelationshipCoverageReport {
        reference_edges: count_rows(connection, TABLE_REFERENCE_EDGES)?,
        reference_occurrences: count_rows(connection, TABLE_REFERENCE_OCCURRENCES)?,
        record_aliases: count_rows(connection, TABLE_RECORD_ALIASES)?,
        remaster_links: count_rows(connection, TABLE_REMASTER_LINKS)?,
    })
}

fn inspect_metrics(connection: &Connection) -> Result<MetricCoverageReport, IndexValidationError> {
    Ok(MetricCoverageReport {
        metric_rows_by_domain: count_grouped(
            connection,
            "SELECT metric_domain AS group_key, COUNT(*) AS row_count
             FROM record_metrics
             GROUP BY metric_domain",
        )?,
        metric_keys_by_domain: count_grouped(
            connection,
            "SELECT metric_domain AS group_key,
                    COUNT(DISTINCT metric_key) AS row_count
             FROM record_metrics
             GROUP BY metric_domain",
        )?,
        metric_value_catalog_rows: count_rows(connection, TABLE_METRIC_VALUE_CATALOG)?,
    })
}

fn count_rows(connection: &Connection, table: &str) -> Result<usize, IndexValidationError> {
    let sql = format!("SELECT COUNT(*) FROM {table}");
    count_sql(connection, &sql)
}

fn count_sql(connection: &Connection, sql: &str) -> Result<usize, IndexValidationError> {
    connection
        .query_row(sql, [], |row| row.get::<_, usize>(0))
        .map_err(|error| IndexValidationError::QueryFailed(error.to_string()))
}

fn count_grouped(
    connection: &Connection,
    sql: &str,
) -> Result<BTreeMap<String, usize>, IndexValidationError> {
    let mut statement = connection
        .prepare(sql)
        .map_err(|error| IndexValidationError::QueryFailed(error.to_string()))?;
    let rows = statement
        .query_map([], |row| {
            Ok((
                row.get::<_, String>("group_key")?,
                row.get::<_, usize>("row_count")?,
            ))
        })
        .map_err(|error| IndexValidationError::QueryFailed(error.to_string()))?;
    let mut counts = BTreeMap::new();
    for row in rows {
        let (key, value) =
            row.map_err(|error| IndexValidationError::QueryFailed(error.to_string()))?;
        counts.insert(key, value);
    }
    Ok(counts)
}
