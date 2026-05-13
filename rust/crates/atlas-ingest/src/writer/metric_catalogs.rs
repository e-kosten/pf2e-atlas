use rusqlite::Connection;

use crate::IngestError;

pub(super) fn write_metric_catalogs(connection: &Connection) -> Result<(), IngestError> {
    connection
        .execute_batch(
            "
            INSERT INTO metric_key_catalog (
              metric_domain,
              record_family,
              namespace_prefix,
              metric_key,
              value_type,
              catalog_count,
              numeric_min,
              numeric_max
            )
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
            GROUP BY rm.metric_domain, r.record_family, namespace_prefix, rm.metric_key, rm.value_type;

            INSERT INTO metric_value_catalog (
              metric_domain,
              record_family,
              metric_key,
              value,
              catalog_count
            )
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
            GROUP BY rm.metric_domain, r.record_family, rm.metric_key, value;
            ",
        )
        .map_err(|error| IngestError::ArtifactWriteFailed(error.to_string()))
}
