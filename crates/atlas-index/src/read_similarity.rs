//! Bounded stored-vector reads over the metadata-only logical-part projection.
use crate::{IndexError, SourceUnitLocation, SqliteIndexReader};
use atlas_domain::{
    RecordKey, SIMILARITY_ROLE_RECIPE, SearchContentScope, SearchProfileId,
    SimilarityComparisonKey, SimilarityComponent, SimilarityPartSelector, SimilarityParts,
    SimilaritySeed, SourcePassageAddress,
};
use rusqlite::{params_from_iter, types::Value};

#[derive(Debug, Clone)]
pub struct SourceSimilarityUnit {
    pub component: SimilarityComponent,
    pub location: Option<SourceUnitLocation>,
    pub address: SourcePassageAddress,
    pub vector: Vec<f32>,
}
#[derive(Clone, Copy)]
pub struct SourceSimilarityCandidateRequest<'a> {
    pub discovery_scope: crate::DiscoveryScope,
    pub profile: &'a SearchProfileId,
    pub vector: &'a [f32],
    pub query: &'a crate::ValidatedQuery,
    pub keys: Option<&'a [RecordKey]>,
    pub exclude: Option<&'a RecordKey>,
    pub comparison_key: Option<SimilarityComparisonKey>,
    pub content_scope: SearchContentScope,
    pub include_identity: bool,
    pub limit: usize,
}

impl SqliteIndexReader {
    /// Exact legacy max-cosine scores for every supplied root, including identity
    /// units. No KNN candidate selection or unit window participates in this read.
    pub fn baseline_similarity_scores(
        &self,
        profile: &SearchProfileId,
        keys: &[RecordKey],
        vector: &[f32],
        content_scope: SearchContentScope,
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
        let table = crate::schema::native_vector_table(model)?;
        if content_scope == SearchContentScope::Core {
            self.require_core_vectors(profile, keys)?;
        }
        let mut values = vec![
            Value::Text(profile.to_string()),
            Value::Blob(crate::read_units::vector_bytes(vector)),
        ];
        values.extend(keys.iter().map(|key| Value::Text(key.to_string())));
        let placeholders = (0..keys.len())
            .map(|i| format!("?{}", i + 3))
            .collect::<Vec<_>>()
            .join(",");
        let content_sql = crate::read_units::semantic_content_sql(content_scope);
        let sql = format!(
            "SELECT r.key,max(1.0-vec_distance_cosine(sv.embedding,?2)),count(*) FROM semantic_units su JOIN records r USING(record_id) JOIN {table} sv ON sv.input_id=su.input_id WHERE su.profile_id=?1 AND r.document_kind<>'Macro' AND r.key IN ({placeholders}){content_sql} GROUP BY r.key ORDER BY r.key"
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
                    "SELECT {},EXISTS(SELECT 1 FROM semantic_units su JOIN semantic_locations sl USING(location_id) WHERE sl.section_id=ss.section_id AND su.profile_id=?2) FROM selected_sections ss JOIN records r USING(record_id) WHERE r.key=?1 AND r.document_kind<>'Macro' ORDER BY ss.owners_json,ss.field_path,ss.section_ordinal",
                    crate::selected_sections::COLUMNS
                );
                let mut statement = c.prepare(&sql)?;
                let mut rows = statement.query(rusqlite::params![
                    record_key.to_string(),
                    profile.map(SearchProfileId::as_str)
                ])?;
                let mut parts = Vec::new();
                while let Some(row) = rows.next()? {
                    let mut section = crate::selected_sections::SelectedSection::read(row, 0)?;
                    section.part.has_vectors = row.get(15)?;
                    parts.push(SimilarityComponent::Section { part: section.part });
                }
                let identity = self
                    .identity_components(std::slice::from_ref(record_key), profile)?
                    .remove(record_key)
                    .ok_or_else(|| {
                        IndexError::Unavailable(format!("record {record_key} was not found"))
                    })?;
                parts.insert(0, identity);
                parts
            }
        };
        let core = match seed {
            SimilaritySeed::Record { record_key } => self.core_selection(record_key, profile)?,
        };
        Ok(SimilarityParts {
            seed: seed.clone(),
            parts,
            core,
            role_recipe: SIMILARITY_ROLE_RECIPE.into(),
        })
    }

    pub fn similarity_units(
        &self,
        profile: &SearchProfileId,
        keys: &[RecordKey],
        content_scope: SearchContentScope,
        include_identity: bool,
        max_units: usize,
    ) -> Result<Vec<SourceSimilarityUnit>, IndexError> {
        self.selected_similarity_units(
            profile,
            keys,
            None,
            content_scope,
            include_identity,
            max_units,
        )
    }

    pub fn selected_similarity_units(
        &self,
        profile: &SearchProfileId,
        keys: &[RecordKey],
        selectors: Option<&[SimilarityPartSelector]>,
        content_scope: SearchContentScope,
        include_identity: bool,
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
        let table = crate::schema::native_vector_table(model)?;
        self.require_vectors()?;
        if content_scope == SearchContentScope::Core {
            self.require_core_vectors(profile, keys)?;
        }
        let include_identity = include_identity || content_scope == SearchContentScope::Core;
        let identities = if include_identity {
            self.identity_components(keys, Some(profile))?
        } else {
            std::collections::BTreeMap::new()
        };
        let selected_identity_keys = identities
            .iter()
            .filter(|(_, identity)| {
                selectors.is_none_or(|selectors| selectors.contains(identity.selector()))
            })
            .map(|(key, _)| key.to_string())
            .collect::<Vec<_>>();
        if identities.values().any(|identity| {
            selectors.is_none_or(|selectors| selectors.contains(identity.selector()))
                && !identity.has_vectors()
        }) {
            return Err(IndexError::Unavailable("a required Name and identity component has no stored vector for this profile; update the reference workspace".into()));
        }
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
                " AND ss.selector IN ({})",
                (start..start + selectors.len())
                    .map(|i| format!("?{i}"))
                    .collect::<Vec<_>>()
                    .join(",")
            )
        } else {
            String::new()
        };
        let identity_sql = if selected_identity_keys.is_empty() {
            "0".to_owned()
        } else {
            let start = values.len() + 1;
            values.extend(selected_identity_keys.into_iter().map(Value::Text));
            format!(
                "su.unit_kind='identity' AND r.key IN ({})",
                (start..=values.len())
                    .map(|parameter| format!("?{parameter}"))
                    .collect::<Vec<_>>()
                    .join(",")
            )
        };
        let content_sql = crate::read_units::semantic_content_sql(content_scope);
        values.push(Value::Integer((max_units + 1) as i64));
        let sql = format!(
            "SELECT {},{},sv.embedding FROM semantic_units su JOIN semantic_locations sl USING(location_id) LEFT JOIN selected_sections ss ON ss.section_id=sl.section_id AND ss.record_id=su.record_id JOIN records r ON r.record_id=su.record_id JOIN {table} sv ON sv.input_id=su.input_id WHERE su.profile_id=?1 AND r.document_kind<>'Macro' AND r.key IN ({placeholders}){content_sql} AND ((su.unit_kind='passage' AND (ss.section_id IS NULL OR (ss.selector IS NOT NULL {selection_sql}))) OR ({identity_sql})) ORDER BY r.key,ss.selector,sl.chunk_ordinal,su.unit_id LIMIT ?{}",
            crate::selected_sections::COLUMNS,
            crate::read_units::SEMANTIC_LOCATION_COLUMNS,
            values.len()
        );
        self.count_statement();
        let mut statement = self.native()?.prepare(&sql)?;
        let mut rows = statement.query(params_from_iter(values.iter()))?;
        let mut units = Vec::new();
        while let Some(row) = rows.next()? {
            if units.len() == max_units {
                return Err(IndexError::Limit(format!(
                    "This comparison is too broad to complete. Deselect some sections under Sections to compare, or add filters to narrow the results. (Vector limit: {max_units}.)"
                )));
            }
            let location = crate::read_units::location_at(row, 15)?;
            let address = location
                .address
                .clone()
                .ok_or_else(|| IndexError::Invalid("semantic unit has no source address".into()))?;
            let vector = crate::read_units::decode_vector(&row.get::<_, Vec<u8>>("embedding")?)?;
            crate::validation::check_vector(&vector, model.dimensions)?;
            let component = if row.get::<_, String>(28)? == "identity" {
                identities.get(&location.record).cloned().ok_or_else(|| {
                    IndexError::Invalid("identity component metadata disappeared".into())
                })?
            } else {
                let mut section = crate::selected_sections::SelectedSection::read(row, 0)?;
                section.part.has_vectors = true;
                SimilarityComponent::Section { part: section.part }
            };
            units.push(SourceSimilarityUnit {
                component,
                location: Some(location),
                address,
                vector,
            });
        }
        Ok(units)
    }
}
