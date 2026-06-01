use atlas_domain::{RecordKey, SearchFilterNode};
use atlas_embedding::EmbeddingUnitKind;
use diesel::sql_types::{BigInt, Binary, Double, Nullable, Text};
use diesel::{QueryableByName, RunQueryDsl, SqliteConnection};

use crate::artifact::inventory::{TABLE_DOCUMENT_EMBEDDING_CACHE, TABLE_RECORD_VECTOR_INDEX};
use crate::artifact::storage::{decode_f32_vector_blob, encode_f32_vector_blob};
use crate::read::search::filters::SqliteEligibleRecordKeyset;
use crate::read::sql::{SqlBindValue, bind_sql_query};

use super::types::{RecordEmbeddingVector, VectorKnnQuery, VectorQueryError, VectorSearchHit};

pub(crate) fn compile_vector_knn_query(
    query_vector: &[f32],
    filter: Option<&SearchFilterNode>,
    limit: u32,
    include_child_units: bool,
) -> Result<VectorKnnQuery, VectorQueryError> {
    if limit == 0 {
        return Err(VectorQueryError::InvalidLimit);
    }
    if query_vector.is_empty() {
        return Err(VectorQueryError::EmptyQueryVector);
    }

    let unit_filter = if include_child_units {
        ""
    } else {
        "AND candidate.unit_kind = 'parent'"
    };
    let query = SqliteEligibleRecordKeyset::new(filter)
        .compile()?
        .with_eligible_cte(|builder| {
            let vector_placeholder = builder.push_blob(encode_f32_vector_blob(query_vector));
            let limit_placeholder = builder.push_integer(i64::from(limit));
            format!(
                "SELECT e.record_key, e.unit_kind, e.label, v.distance
         FROM {vector_table} v
         JOIN {cache_table} e ON e.rowid = v.rowid
         WHERE v.embedding MATCH {vector_placeholder}
           AND k = {limit_placeholder}
           AND v.rowid IN (
             SELECT candidate.rowid
             FROM {cache_table} candidate
             WHERE candidate.record_key IN (SELECT record_key FROM eligible)
               {unit_filter}
         )
         ORDER BY v.distance ASC",
                vector_table = TABLE_RECORD_VECTOR_INDEX,
                cache_table = TABLE_DOCUMENT_EMBEDDING_CACHE,
                unit_filter = unit_filter,
            )
        });

    Ok(VectorKnnQuery {
        sql: query.sql,
        parameters: query.parameters,
    })
}

pub fn query_vector_index(
    connection: &mut SqliteConnection,
    query_vector: &[f32],
    filter: Option<&SearchFilterNode>,
    limit: u32,
    include_child_units: bool,
) -> Result<Vec<VectorSearchHit>, VectorQueryError> {
    let compiled = compile_vector_knn_query(query_vector, filter, limit, include_child_units)?;
    let rows = bind_sql_query(compiled.sql, &compiled.parameters)
        .load::<VectorSearchHitRow>(connection)
        .map_err(|error| VectorQueryError::QueryFailed(error.to_string()))?;
    rows.into_iter()
        .map(|row| {
            Ok(VectorSearchHit {
                record_key: row.record_key,
                unit_kind: parse_embedding_unit_kind(row.unit_kind)?,
                label: row.label,
                distance: row.distance,
            })
        })
        .collect()
}

pub fn load_record_embedding_vectors(
    connection: &mut SqliteConnection,
    record_key: &RecordKey,
) -> Result<Vec<RecordEmbeddingVector>, VectorQueryError> {
    let rows = bind_sql_query(
        "SELECT unit_kind, label, ordinal, dimensions, vector_blob
         FROM document_embedding_cache
         WHERE record_key = ?1
         ORDER BY ordinal ASC, embedding_unit_key ASC"
            .to_string(),
        &[SqlBindValue::Text(record_key.to_string())],
    )
    .load::<RecordEmbeddingVectorRow>(connection)
    .map_err(|error| VectorQueryError::QueryFailed(error.to_string()))?;
    let record_key = record_key.to_string();
    rows.into_iter()
        .map(|row| {
            let vector = decode_f32_vector_blob(&row.vector_blob).map_err(|error| {
                VectorQueryError::InvalidStoredVector {
                    record_key: record_key.clone(),
                    unit_kind: row.unit_kind.clone(),
                    ordinal: row.ordinal,
                    message: error.to_string(),
                }
            })?;
            let declared_dimensions = usize::try_from(row.dimensions).map_err(|error| {
                VectorQueryError::QueryFailed(format!(
                    "stored embedding vector for `{record_key}` ({} #{}) had invalid dimensions `{}`: {error}",
                    row.unit_kind, row.ordinal, row.dimensions
                ))
            })?;
            if declared_dimensions != vector.len() {
                return Err(VectorQueryError::InvalidStoredDimensions {
                    record_key: record_key.clone(),
                    unit_kind: row.unit_kind.clone(),
                    ordinal: row.ordinal,
                    declared_dimensions,
                    decoded_dimensions: vector.len(),
                });
            }
            Ok(RecordEmbeddingVector {
                unit_kind: parse_embedding_unit_kind(row.unit_kind)?,
                label: row.label,
                ordinal: row.ordinal,
                vector,
            })
        })
        .collect()
}

fn parse_embedding_unit_kind(value: String) -> Result<String, VectorQueryError> {
    value
        .parse::<EmbeddingUnitKind>()
        .map(|unit_kind| unit_kind.as_str().to_string())
        .map_err(|_| VectorQueryError::InvalidUnitKind(value))
}

#[derive(QueryableByName)]
struct VectorSearchHitRow {
    #[diesel(sql_type = Text)]
    record_key: String,
    #[diesel(sql_type = Text)]
    unit_kind: String,
    #[diesel(sql_type = Nullable<Text>)]
    label: Option<String>,
    #[diesel(sql_type = Double)]
    distance: f64,
}

#[derive(QueryableByName)]
struct RecordEmbeddingVectorRow {
    #[diesel(sql_type = Text)]
    unit_kind: String,
    #[diesel(sql_type = Nullable<Text>)]
    label: Option<String>,
    #[diesel(sql_type = BigInt)]
    ordinal: i64,
    #[diesel(sql_type = BigInt)]
    dimensions: i64,
    #[diesel(sql_type = Binary)]
    vector_blob: Vec<u8>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_known_embedding_unit_kind_to_public_string() {
        assert_eq!(
            parse_embedding_unit_kind("heading_section".to_string())
                .expect("known unit kind should parse"),
            "heading_section"
        );
    }

    #[test]
    fn rejects_unknown_embedding_unit_kind() {
        assert!(parse_embedding_unit_kind("unknown".to_string()).is_err());
    }
}
