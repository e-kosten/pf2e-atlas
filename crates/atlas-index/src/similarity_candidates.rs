//! Exact configured selection and operation-local reuse of its unit eligibility.
use crate::eligibility_buffer::EligibilityStore;
use crate::{IndexError, SourceSimilarityCandidateRequest, SourceVectorHit, SqliteIndexReader};
use atlas_domain::{SearchContentScope, SimilarityComparisonKey};
use rusqlite::{Statement, params, params_from_iter, types::Value};
use serde::{Serialize, Serializer, ser::SerializeSeq};

/// An operation-local owner borrowing one already-admitted artifact snapshot.
///
/// Only exact unit eligibility is retained. Every vector/k still passes the same
/// guards and native query; dropping this owner releases all prepared relations.
pub struct ConfiguredSimilarityCandidates<'a> {
    index: &'a SqliteIndexReader,
    eligibility: EligibilityStore,
}

struct CandidatePlan {
    table: String,
    eligibility_prefix: usize,
    eligibility_units: std::ops::Range<usize>,
    native_sql: String,
    values: Vec<Value>,
    setup_parameters: usize,
}

impl SqliteIndexReader {
    /// Begin lazy, bounded configured eligibility reuse within this reader only.
    pub fn configured_similarity_candidates(&self) -> ConfiguredSimilarityCandidates<'_> {
        ConfiguredSimilarityCandidates {
            index: self,
            eligibility: EligibilityStore::new(),
        }
    }

    /// Execute the exact configured native selection without retaining eligibility.
    pub fn similarity_candidates(
        &self,
        request: SourceSimilarityCandidateRequest<'_>,
    ) -> Result<Vec<SourceVectorHit>, IndexError> {
        let Some(plan) = self.configured_candidate_plan(request)? else {
            return Ok(vec![]);
        };
        self.count_statement();
        let mut statement = self.native()?.prepare(&plan.native_sql)?;
        collect(&mut statement, params_from_iter(plan.values.iter()))
    }

    fn configured_candidate_plan(
        &self,
        request: SourceSimilarityCandidateRequest<'_>,
    ) -> Result<Option<CandidatePlan>, IndexError> {
        let SourceSimilarityCandidateRequest {
            profile,
            vector,
            query,
            keys,
            exclude,
            comparison_key,
            content_scope,
            include_identity,
            limit,
        } = request;
        // This order applies to every selection, including retained/empty sets.
        if limit == 0 || limit > 4096 {
            return Err(IndexError::Limit(
                "similarity KNN window must be 1..4096".into(),
            ));
        }
        if comparison_key.is_some_and(|key| !key.corresponds_to(key)) {
            return Ok(None);
        }
        let model = self.context().search_profile(profile)?;
        crate::validation::check_vector(vector, model.dimensions)?;
        self.require_vectors()?;
        let table = crate::schema::vector_table(model)?;
        let (mut eligible, mut values) = crate::read_units::eligible_sql(query, keys)?;
        if let Some(exclude) = exclude {
            eligible.push_str(&format!(" AND r.key<>?{}", values.len() + 1));
            values.push(Value::Text(exclude.to_string()));
        }
        values.push(Value::Text(profile.to_string()));
        let p = values.len();
        let role = match comparison_key {
            Some(SimilarityComparisonKey::Role(role)) => Some(role),
            _ => None,
        };
        let role_sql = if let Some(role) = role {
            values.push(Value::Text(
                serde_json::to_value(role)?
                    .as_str()
                    .ok_or_else(|| IndexError::Invalid("invalid comparison role encoding".into()))?
                    .to_owned(),
            ));
            format!(" AND sp.role=?{}", values.len())
        } else {
            String::new()
        };
        let prose_sql = if comparison_key == Some(SimilarityComparisonKey::Identity) {
            "0".to_owned()
        } else {
            format!(
                "su.unit_kind='passage' AND (sp.section_id IS NULL OR (sp.selector IS NOT NULL {role_sql}))"
            )
        };
        let identity_allowed =
            (include_identity || content_scope == SearchContentScope::Core) && role.is_none();
        let identity_sql = if identity_allowed {
            "su.unit_kind='identity'"
        } else {
            "0"
        };
        let content_sql = crate::read_units::semantic_content_sql(content_scope);
        // One eligibility formula supplies materialization, identity and bypass.
        let prefix = format!("WITH eligible AS ({eligible}) ");
        let units = format!(
            "SELECT su.unit_id FROM semantic_units su JOIN semantic_locations sl USING(location_id) LEFT JOIN selected_sections sp ON sp.section_id=sl.section_id AND sp.record_id=su.record_id WHERE su.profile_id=?{p} AND su.record_id IN (SELECT record_id FROM eligible){content_sql} AND (({prose_sql}) OR ({identity_sql}))",
        );
        let setup_parameters = values.len();
        values.push(Value::Blob(crate::read_units::vector_bytes(vector)));
        let v = values.len();
        values.push(Value::Integer(limit as i64));
        let k = values.len();
        let (head, tail) = selection_parts(&table, v, k);
        let eligibility_prefix = prefix.len();
        let start = prefix.len() + head.len();
        let eligibility_units = start..start + units.len();
        let native_sql = format!("{prefix}{head}{units}{tail}");
        Ok(Some(CandidatePlan {
            table,
            eligibility_prefix,
            eligibility_units,
            native_sql,
            values,
            setup_parameters,
        }))
    }
}

// Keep the exact native traversal, outer joins, location decoding and ordering in
// one place. Reuse changes only the source of the IN-set, never k or traversal.
fn selection_parts(table: &str, vector: usize, k: usize) -> (String, &'static str) {
    (
        format!(
            "SELECT {},knn.distance FROM (SELECT unit_id,distance FROM {table} WHERE embedding MATCH ?{vector} AND k=?{k} AND unit_id IN (",
            crate::read_units::SEMANTIC_LOCATION_COLUMNS
        ),
        ")) knn JOIN semantic_units su USING(unit_id) JOIN semantic_locations sl USING(location_id) JOIN records r ON r.record_id=su.record_id LEFT JOIN selected_sections ss ON ss.section_id=sl.section_id AND ss.record_id=su.record_id ORDER BY knn.distance,su.unit_id",
    )
}
fn selection_sql(table: &str, units: &str, vector: usize, k: usize) -> String {
    let (head, tail) = selection_parts(table, vector, k);
    format!("{head}{units}{tail}")
}
impl CandidatePlan {
    fn key(&self) -> EligibilityKey<'_> {
        EligibilityKey {
            prefix: &self.native_sql[..self.eligibility_prefix],
            units: &self.native_sql[self.eligibility_units.clone()],
            values: SqlValues(&self.values[..self.setup_parameters]),
        }
    }
    fn setup_sql(&self) -> String {
        let key = self.key();
        // Called only after complete-key admission (<=64 KiB). The store's
        // fixed charge reserves64 KiB plus allowance for this one scratch command.
        let mut sql = String::with_capacity(key.prefix.len() + key.units.len());
        sql.push_str(key.prefix);
        sql.push_str(key.units);
        sql
    }
}

fn collect(
    statement: &mut Statement<'_>,
    parameters: impl rusqlite::Params,
) -> Result<Vec<SourceVectorHit>, IndexError> {
    Ok(statement
        .query_map(parameters, |row| {
            Ok(SourceVectorHit {
                location: crate::read_units::location(row)?,
                distance: row.get("distance")?,
            })
        })?
        .collect::<Result<Vec<_>, _>>()?)
}

impl ConfiguredSimilarityCandidates<'_> {
    /// Select using bounded exact eligibility, with cooperative setup checks.
    ///
    /// Checks occur before setup, every 256 emitted rows and after setup. They do
    /// not interrupt SQLite work before a row or an executing native selection.
    pub fn candidates<E: From<IndexError>>(
        &mut self,
        request: SourceSimilarityCandidateRequest<'_>,
        mut check: impl FnMut() -> Result<(), E>,
    ) -> Result<Vec<SourceVectorHit>, E> {
        let index = self.index;
        let Some(plan) = index.configured_candidate_plan(request)? else {
            return Ok(vec![]);
        };
        let connection = index.native()?;
        // Preserve original SQL/schema/native-plan error precedence before setup.
        index.count_statement();
        let mut original = connection
            .prepare(&plan.native_sql)
            .map_err(IndexError::from)?;
        let optimized_sql = selection_sql(&plan.table, "SELECT value FROM json_each(?3)", 1, 2);
        index.count_statement();
        let mut optimized = connection
            .prepare(&optimized_sql)
            .map_err(IndexError::from)?;
        let key = plan.key();
        let payload = self.eligibility.get_or_try_insert(&key, |writer| {
            check()?;
            index.count_statement();
            {
                let setup_sql = plan.setup_sql();
                let mut statement = connection.prepare(&setup_sql).map_err(IndexError::from)?;
                let mut rows = statement
                    .query(params_from_iter(
                        plan.values[..plan.setup_parameters].iter(),
                    ))
                    .map_err(IndexError::from)?;
                let mut emitted = 0usize;
                while let Some(row) = rows.next().map_err(IndexError::from)? {
                    if !writer.push_id(row.get::<_, i64>(0).map_err(IndexError::from)?) {
                        break;
                    }
                    emitted += 1;
                    if emitted.is_multiple_of(256) {
                        check()?;
                    }
                }
            }
            check()?;
            Ok::<(), E>(())
        })?;
        match payload {
            Some(ids) => collect(
                &mut optimized,
                params![
                    &plan.values[plan.setup_parameters],
                    &plan.values[plan.setup_parameters + 1],
                    ids
                ],
            )
            .map_err(E::from),
            None => collect(&mut original, params_from_iter(plan.values.iter())).map_err(E::from),
        }
    }

    #[cfg(any(test, feature = "test-support"))]
    pub fn stats(&self) -> crate::EligibilityStats {
        self.eligibility.stats()
    }
}

#[derive(Serialize)]
struct EligibilityKey<'a> {
    prefix: &'a str,
    units: &'a str,
    values: SqlValues<'a>,
}
struct SqlValues<'a>(&'a [Value]);
impl Serialize for SqlValues<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut seq = serializer.serialize_seq(Some(self.0.len()))?;
        for value in self.0 {
            match value {
                Value::Null => seq.serialize_element(&("null", ()))?,
                Value::Integer(n) => seq.serialize_element(&("integer", n))?,
                Value::Real(n) => seq.serialize_element(&("real_bits", n.to_bits()))?,
                Value::Text(text) => seq.serialize_element(&("text", text))?,
                Value::Blob(bytes) => seq.serialize_element(&("blob", bytes))?,
            }
        }
        seq.end()
    }
}

#[cfg(test)]
mod tests;
