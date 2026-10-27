//! Bounded stored-vector reads over the metadata-only logical-part projection.
use crate::{IndexError, SourceUnitLocation, SourceVectorHit, SqliteIndexReader};
use atlas_domain::{
    RecordKey, SIMILARITY_ROLE_RECIPE, SearchProfileId, SimilarityPart, SimilarityPartSelector,
    SimilarityParts, SimilarityRole, SimilaritySeed, SourcePassageAddress,
};
use rusqlite::{params_from_iter, types::Value};

pub(crate) const PART_JOIN: &str = "sp.record_id=su.record_id AND sp.owners_json=sl.owners_json AND sp.field_path=sl.field_path AND sp.section_ordinal=json_extract(sl.section_json,'$.section_ordinal') AND sp.body_hash=coalesce(json_extract(sl.section_json,'$.canonical_text_sha256'),json_extract(sl.section_json,'$.source_text_sha256'))";

#[derive(Debug, Clone)]
pub struct SourceSimilarityUnit {
    pub part: SimilarityPart,
    pub location: Option<SourceUnitLocation>,
    pub address: SourcePassageAddress,
    pub vector: Vec<f32>,
    pub trait_text: Option<String>,
}
pub struct SourceSimilarityCandidateRequest<'a> {
    pub profile: &'a SearchProfileId,
    pub vector: &'a [f32],
    pub query: &'a crate::ValidatedQuery,
    pub keys: Option<&'a [RecordKey]>,
    pub exclude: Option<&'a RecordKey>,
    pub role: Option<SimilarityRole>,
    pub limit: usize,
}

fn parse_part(json: &str, recipe: &str) -> Result<SimilarityPart, IndexError> {
    if recipe != SIMILARITY_ROLE_RECIPE {
        return Err(IndexError::Unsupported(
            "similarity role projection recipe differs".into(),
        ));
    }
    Ok(serde_json::from_str(json)?)
}

impl SqliteIndexReader {
    /// Exact legacy max-cosine scores for every supplied root, including identity
    /// units. No KNN candidate selection or unit window participates in this read.
    pub fn baseline_similarity_scores(
        &self,
        profile: &SearchProfileId,
        keys: &[RecordKey],
        vector: &[f32],
    ) -> Result<Vec<(RecordKey, f64, usize)>, IndexError> {
        if keys.is_empty() {
            return Ok(vec![]);
        }
        if keys.len() > 256 {
            return Err(IndexError::Limit(
                "baseline scoring supports 256 roots per batch".into(),
            ));
        }
        let model = self.context().search_profile(profile)?;
        crate::validation::check_vector(vector, model.dimensions)?;
        self.require_vectors()?;
        let table = crate::schema::vector_table(model)?;
        let mut values = vec![
            Value::Text(profile.to_string()),
            Value::Blob(crate::read_units::vector_bytes(vector)),
        ];
        values.extend(keys.iter().map(|key| Value::Text(key.to_string())));
        let placeholders = (0..keys.len())
            .map(|i| format!("?{}", i + 3))
            .collect::<Vec<_>>()
            .join(",");
        let sql = format!(
            "SELECT r.key,max(1.0-vec_distance_cosine(sv.embedding,?2)),count(*) FROM semantic_units su JOIN records r USING(record_id) JOIN {table} sv ON sv.unit_id=su.unit_id WHERE su.profile_id=?1 AND r.document_kind<>'Macro' AND r.key IN ({placeholders}) GROUP BY r.key ORDER BY r.key"
        );
        self.count_statement();
        let rows = self
            .native()?
            .prepare(&sql)?
            .query_map(params_from_iter(values.iter()), |row| {
                Ok((
                    row.get::<_, String>(0)?,
                    row.get::<_, f64>(1)?,
                    row.get::<_, usize>(2)?,
                ))
            })?
            .collect::<Result<Vec<_>, _>>()?;
        rows.into_iter()
            .map(|(key, score, count)| {
                Ok((
                    key.parse()
                        .map_err(|e| IndexError::Invalid(format!("invalid stored key: {e}")))?,
                    score,
                    count,
                ))
            })
            .collect()
    }

    pub fn similarity_parts(
        &self,
        seed: &SimilaritySeed,
        profile: Option<&SearchProfileId>,
    ) -> Result<SimilarityParts, IndexError> {
        let c = self.native()?;
        self.count_statement();
        let parts = match seed {
            SimilaritySeed::Record { record_key } => {
                let sql = format!(
                    "SELECT sp.metadata_json,sp.recipe,EXISTS(SELECT 1 FROM semantic_units su JOIN semantic_locations sl USING(location_id) WHERE {PART_JOIN} AND su.profile_id=?2) FROM similarity_parts sp JOIN records r USING(record_id) WHERE r.key=?1 AND r.document_kind<>'Macro' ORDER BY sp.owners_json,sp.field_path,sp.section_ordinal"
                );
                let rows = c
                    .prepare(&sql)?
                    .query_map(
                        rusqlite::params![
                            record_key.to_string(),
                            profile.map(SearchProfileId::as_str)
                        ],
                        |row| {
                            Ok((
                                row.get::<_, String>(0)?,
                                row.get::<_, String>(1)?,
                                row.get::<_, bool>(2)?,
                            ))
                        },
                    )?
                    .collect::<Result<Vec<_>, _>>()?;
                rows.into_iter()
                    .map(|(json, recipe, has_vectors)| {
                        let mut part = parse_part(&json, &recipe)?;
                        part.has_vectors = has_vectors;
                        Ok(part)
                    })
                    .collect::<Result<Vec<_>, IndexError>>()?
            }
            SimilaritySeed::Trait { trait_key } => {
                let Some(definition) = self.trait_definition(trait_key)? else {
                    return Err(IndexError::Unavailable(format!(
                        "trait {trait_key} was not found"
                    )));
                };
                let fingerprint = crate::codec::sha256(&serde_json::to_vec(&(
                    trait_key,
                    &definition.label,
                    &definition.authored_markup,
                ))?);
                let has_vectors = c.query_row(
                    "SELECT EXISTS(SELECT 1 FROM trait_vectors WHERE trait_key=? AND profile_id=?)",
                    rusqlite::params![trait_key.to_string(), profile.map(SearchProfileId::as_str)],
                    |row| row.get(0),
                )?;
                vec![SimilarityPart {
                    selector: SimilarityPartSelector(format!("part:{fingerprint}")),
                    label: if definition.prepared.is_some() {
                        "Definition".into()
                    } else {
                        "Label only".into()
                    },
                    owner_label: definition.label,
                    owners: vec![],
                    field: "/definition".into(),
                    section_ordinal: 0,
                    role: SimilarityRole::TraitDefinition,
                    has_vectors,
                    label_only: definition.prepared.is_none(),
                }]
            }
        };
        Ok(SimilarityParts {
            seed: seed.clone(),
            parts,
            role_recipe: SIMILARITY_ROLE_RECIPE.into(),
        })
    }

    pub fn similarity_units(
        &self,
        profile: &SearchProfileId,
        keys: &[RecordKey],
        max_units: usize,
    ) -> Result<Vec<SourceSimilarityUnit>, IndexError> {
        self.selected_similarity_units(profile, keys, None, max_units)
    }

    pub fn selected_similarity_units(
        &self,
        profile: &SearchProfileId,
        keys: &[RecordKey],
        selectors: Option<&[SimilarityPartSelector]>,
        max_units: usize,
    ) -> Result<Vec<SourceSimilarityUnit>, IndexError> {
        if keys.is_empty() {
            return Ok(vec![]);
        }
        if keys.len() > 256 || max_units == 0 || max_units > 100_000 {
            return Err(IndexError::Limit(
                "similarity vector read supports at most 256 records and 100000 units per batch"
                    .into(),
            ));
        }
        let model = self.context().search_profile(profile)?;
        let table = crate::schema::vector_table(model)?;
        self.require_vectors()?;
        let mut values = vec![Value::Text(profile.to_string())];
        values.extend(keys.iter().map(|key| Value::Text(key.to_string())));
        let placeholders = (0..keys.len())
            .map(|i| format!("?{}", i + 2))
            .collect::<Vec<_>>()
            .join(",");
        let selection_sql = if let Some(selectors) = selectors {
            if selectors.is_empty() {
                return Ok(vec![]);
            }
            let start = values.len() + 1;
            values.extend(
                selectors
                    .iter()
                    .map(|selector| Value::Text(selector.0.clone())),
            );
            format!(
                " AND sp.selector IN ({})",
                (start..start + selectors.len())
                    .map(|i| format!("?{i}"))
                    .collect::<Vec<_>>()
                    .join(",")
            )
        } else {
            String::new()
        };
        values.push(Value::Integer((max_units + 1) as i64));
        let sql = format!(
            "SELECT sp.metadata_json,sp.recipe,su.unit_id,r.key,sl.owners_json,sl.field_path,sl.section_json,sv.embedding FROM semantic_units su JOIN semantic_locations sl USING(location_id) JOIN similarity_parts sp ON {PART_JOIN} JOIN records r ON r.record_id=su.record_id JOIN {table} sv ON sv.unit_id=su.unit_id WHERE su.profile_id=?1 AND r.document_kind<>'Macro' AND r.key IN ({placeholders}){selection_sql} ORDER BY r.key,sp.selector,sl.chunk_ordinal,su.unit_id LIMIT ?{}",
            values.len()
        );
        self.count_statement();
        let mut statement = self.native()?.prepare(&sql)?;
        let mut rows = statement.query(params_from_iter(values.iter()))?;
        let mut units = Vec::new();
        while let Some(row) = rows.next()? {
            if units.len() == max_units {
                return Err(IndexError::Limit(format!(
                    "similarity requires more than {max_units} candidate vectors; narrow the selected parts or candidate filters"
                )));
            }
            let mut part = parse_part(&row.get::<_, String>(0)?, &row.get::<_, String>(1)?)?;
            part.has_vectors = true;
            let address: SourcePassageAddress = serde_json::from_str(&row.get::<_, String>(6)?)?;
            let vector = crate::read_units::decode_vector(&row.get::<_, Vec<u8>>(7)?)?;
            crate::validation::check_vector(&vector, model.dimensions)?;
            let location = SourceUnitLocation {
                unit_id: row.get(2)?,
                record: row.get::<_, String>(3)?.parse().map_err(|error| {
                    IndexError::Invalid(format!("invalid similarity record key: {error}"))
                })?,
                owners: serde_json::from_str(&row.get::<_, String>(4)?)?,
                field: row.get(5)?,
                address: Some(address.clone()),
            };
            units.push(SourceSimilarityUnit {
                part,
                location: Some(location),
                address,
                vector,
                trait_text: None,
            });
        }
        Ok(units)
    }

    pub fn trait_similarity_units(
        &self,
        profile: &SearchProfileId,
        key: &atlas_domain::TraitKey,
        max_units: usize,
    ) -> Result<Vec<SourceSimilarityUnit>, IndexError> {
        let model = self.context().search_profile(profile)?;
        self.require_vectors()?;
        let count: usize = self.native()?.query_row(
            "SELECT count(*) FROM trait_seed_chunks WHERE trait_key=? AND profile_id=?",
            rusqlite::params![key.to_string(), profile.as_str()],
            |row| row.get(0),
        )?;
        if count > max_units {
            return Err(IndexError::Limit(format!(
                "trait comparison requires {count} seed units, exceeding {max_units}"
            )));
        }
        let seed = SimilaritySeed::Trait {
            trait_key: key.clone(),
        };
        let part = self
            .similarity_parts(&seed, Some(profile))?
            .parts
            .into_iter()
            .next()
            .ok_or_else(|| {
                IndexError::Invalid("trait comparison definition is unavailable".into())
            })?;
        let definition = self
            .trait_definition(key)?
            .ok_or_else(|| IndexError::Invalid("trait comparison definition disappeared".into()))?;
        let source = format!(
            "{}\n{}",
            definition.label,
            definition
                .prepared
                .as_ref()
                .map(|prepared| prepared.text.as_str())
                .unwrap_or("")
        );
        crate::traits::read_chunks(self.native()?, &key.to_string(), model)?
            .into_iter()
            .map(|chunk| {
                let range = match &chunk.address {
                    SourcePassageAddress::PlainSection { chunk_bytes, .. } => *chunk_bytes,
                    _ => {
                        return Err(IndexError::Invalid(
                            "trait seed is not a source-addressed definition passage".into(),
                        ));
                    }
                };
                let text = range
                    .extract(&source)
                    .map_err(|error| IndexError::Invalid(error.to_string()))?
                    .to_owned();
                Ok(SourceSimilarityUnit {
                    part: part.clone(),
                    location: None,
                    address: chunk.address,
                    vector: chunk.vector,
                    trait_text: Some(text),
                })
            })
            .collect()
    }

    pub fn similarity_candidates(
        &self,
        request: SourceSimilarityCandidateRequest<'_>,
    ) -> Result<Vec<SourceVectorHit>, IndexError> {
        let SourceSimilarityCandidateRequest {
            profile,
            vector,
            query,
            keys,
            exclude,
            role,
            limit,
        } = request;
        if limit == 0 || limit > 4096 {
            return Err(IndexError::Limit(
                "similarity KNN window must be 1..4096".into(),
            ));
        }
        if role.is_some_and(|role| !role.corresponds_to(role)) {
            return Ok(vec![]);
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
        values.push(Value::Blob(crate::read_units::vector_bytes(vector)));
        let v = values.len();
        values.push(Value::Integer(limit as i64));
        let k = values.len();
        let sql = format!(
            "WITH eligible AS ({eligible}) SELECT su.unit_id,r.key,sl.owners_json,sl.field_path,sl.section_json,knn.distance FROM (SELECT unit_id,distance FROM {table} WHERE embedding MATCH ?{v} AND k=?{k} AND unit_id IN (SELECT su.unit_id FROM semantic_units su JOIN semantic_locations sl USING(location_id) JOIN similarity_parts sp ON {PART_JOIN} WHERE su.profile_id=?{p} AND su.record_id IN (SELECT record_id FROM eligible){role_sql})) knn JOIN semantic_units su USING(unit_id) JOIN semantic_locations sl USING(location_id) JOIN records r ON r.record_id=su.record_id ORDER BY knn.distance,su.unit_id"
        );
        self.count_statement();
        Ok(self
            .native()?
            .prepare(&sql)?
            .query_map(params_from_iter(values.iter()), |row| {
                Ok(SourceVectorHit {
                    location: crate::read_units::location(row)?,
                    distance: row.get(5)?,
                })
            })?
            .collect::<Result<Vec<_>, _>>()?)
    }
}
