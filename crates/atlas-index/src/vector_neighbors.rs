//! Transient exact native neighbors. Metadata is never retained across sessions.
use crate::{IndexError, SourceVectorCandidateRequest, SourceVectorHit, SqliteIndexReader};
use rusqlite::{params_from_iter, types::Value};

/// Opaque, fully hydrated native result stripped to IDs and exact distance bits.
/// Each batch belongs to one exact k, query vector and admitted eligibility.
#[derive(Debug)]
pub struct SourceVectorNeighbors {
    neighbors: Box<[(i64, u64)]>,
}
impl SourceVectorNeighbors {
    /// Owned payload allocation, excluding the inline batch descriptor.
    pub fn allocated_bytes(&self) -> usize {
        std::mem::size_of_val(self.neighbors.as_ref())
    }

    #[cfg(any(test, feature = "test-support"))]
    pub fn fixture(neighbors: &[(i64, u64)]) -> Self {
        Self {
            neighbors: neighbors.into(),
        }
    }
}
impl SqliteIndexReader {
    /// Runs the unchanged native query and all location decoders before making
    /// an owned batch available for optional transient retention.
    pub fn vector_candidates_with_neighbors(
        &self,
        request: SourceVectorCandidateRequest<'_>,
    ) -> Result<(Vec<SourceVectorHit>, SourceVectorNeighbors), IndexError> {
        let hits = self.vector_candidates(request)?;
        let neighbors = SourceVectorNeighbors {
            neighbors: hits
                .iter()
                .map(|h| (h.location.unit_id, h.distance.to_bits()))
                .collect(),
        };
        Ok((hits, neighbors))
    }

    /// Repeats request validation and hydrates every retained neighbor through
    /// current profile/eligibility/Core joins in this admitted snapshot.
    pub fn hydrate_vector_neighbors(
        &self,
        request: SourceVectorCandidateRequest<'_>,
        neighbors: &SourceVectorNeighbors,
    ) -> Result<Vec<SourceVectorHit>, IndexError> {
        if request.limit == 0 || request.limit > 4096 {
            return Err(IndexError::Invalid(
                "sqlite-vec candidate window must be 1..=4096".into(),
            ));
        }
        let model = self.context().search_profile(request.profile)?;
        crate::schema::vector_table(model)?;
        crate::validation::check_vector(request.vector, model.dimensions)?;
        self.require_vectors()?;
        // Compile even an empty successful window, preserving request failures.
        let (mut eligible, mut eligibility_params) =
            crate::read_units::eligible_sql(request.query, request.keys)?;
        if let Some(exclude) = request.exclude {
            eligible.push_str(&format!(" AND r.key<>?{}", eligibility_params.len() + 1));
            eligibility_params.push(Value::Text(exclude.to_string()));
        }
        if neighbors.neighbors.len() > request.limit {
            return Err(IndexError::Invalid(
                "retained neighbors exceed their native window".into(),
            ));
        }
        let mut hits = Vec::with_capacity(neighbors.neighbors.len());
        for batch in neighbors.neighbors.chunks(256) {
            let mut params = eligibility_params.clone();
            let profile_param = params.len() + 1;
            params.push(Value::Text(request.profile.to_string()));
            let requested = batch
                .iter()
                .enumerate()
                .map(|(ordinal, (unit, _))| {
                    params.push(Value::Integer(*unit));
                    format!("(?{}, {ordinal})", params.len())
                })
                .collect::<Vec<_>>()
                .join(",");
            let content = crate::read_units::semantic_content_sql(request.content_scope);
            // Publication identity binds native membership. A vec0 point join
            // rereads vector blobs even when only its unit ID is selected.
            // Admission already checks native capabilities and complete DDL;
            // this phase validates current metadata and full eligibility.
            let sql = format!(
                "WITH eligible AS ({eligible}), requested(unit_id,ordinal) AS (VALUES {requested}) SELECT su.unit_id,r.key,sl.owners_json,sl.field_path,sl.section_json,q.ordinal FROM requested q JOIN semantic_units su ON su.unit_id=q.unit_id JOIN semantic_locations sl USING(location_id) JOIN records r ON r.record_id=su.record_id WHERE su.profile_id=?{profile_param} AND su.record_id IN (SELECT record_id FROM eligible){content} ORDER BY q.ordinal"
            );
            self.count_statement();
            let mut statement = self.native()?.prepare(&sql)?;
            let mut count = 0;
            for row in statement.query_map(params_from_iter(params.iter()), |row| {
                Ok((crate::read_units::location(row)?, row.get::<_, usize>(5)?))
            })? {
                let (location, ordinal) = row?;
                if ordinal != count || batch.get(ordinal).is_none_or(|n| n.0 != location.unit_id) {
                    return Err(IndexError::Invalid(
                        "retained neighbors have missing or unexpected members".into(),
                    ));
                }
                hits.push(SourceVectorHit {
                    location,
                    distance: f64::from_bits(batch[ordinal].1),
                });
                count += 1;
            }
            if count != batch.len() {
                return Err(IndexError::Invalid(
                    "retained neighbors have missing or unexpected members".into(),
                ));
            }
        }
        Ok(hits)
    }
}
