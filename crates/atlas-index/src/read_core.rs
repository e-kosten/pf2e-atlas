//! Body-free Core membership and native root identity inventory.
use crate::{IndexError, SqliteIndexReader};
use atlas_domain::{
    CORE_SECTIONS_RECIPE, CoreSectionFamily, CoreSectionRepresentation, CoreSectionSelection,
    RecordKey, SearchProfileId, SimilarityComponent, SimilarityPartSelector,
};
use rusqlite::{OptionalExtension, params_from_iter};
use std::collections::BTreeMap;

struct IdentityMetadata {
    label: String,
    content_hash: String,
    inputs: Vec<(String, Vec<u8>, usize)>,
    has_vectors: bool,
}

impl SqliteIndexReader {
    pub(crate) fn require_core_vectors(
        &self,
        profile: &SearchProfileId,
        keys: &[RecordKey],
    ) -> Result<(), IndexError> {
        if keys.is_empty() {
            return Ok(());
        }
        let model = self.context().search_profile(profile)?;
        let table = crate::schema::vector_table(model)?;
        let placeholders = (2..keys.len() + 2)
            .map(|parameter| format!("?{parameter}"))
            .collect::<Vec<_>>()
            .join(",");
        let sql = format!(
            "WITH requested AS (SELECT r.record_id FROM records r WHERE r.key IN ({placeholders}) AND r.document_kind<>'Macro') SELECT EXISTS(SELECT 1 FROM requested r WHERE NOT EXISTS(SELECT 1 FROM semantic_units su JOIN {table} sv USING(unit_id) WHERE su.record_id=r.record_id AND su.unit_kind='identity' AND su.profile_id=?1)) OR EXISTS(SELECT 1 FROM core_parts cp JOIN similarity_parts sp USING(selector) WHERE cp.record_id IN(SELECT record_id FROM requested) AND NOT EXISTS(SELECT 1 FROM semantic_units su JOIN semantic_locations sl USING(location_id) JOIN {table} sv USING(unit_id) WHERE {} AND su.profile_id=?1))",
            crate::read_similarity::PART_JOIN
        );
        let mut values = vec![profile.to_string()];
        values.extend(keys.iter().map(ToString::to_string));
        self.count_statement();
        let missing: bool = self
            .native()?
            .query_row(&sql, params_from_iter(&values), |row| row.get(0))?;
        if missing {
            return Err(IndexError::Unavailable("a required Core comparison component has no stored vector for this profile; update the reference workspace".into()));
        }
        Ok(())
    }

    pub(crate) fn identity_components(
        &self,
        keys: &[RecordKey],
        profile: Option<&SearchProfileId>,
    ) -> Result<BTreeMap<RecordKey, SimilarityComponent>, IndexError> {
        if keys.is_empty() {
            return Ok(BTreeMap::new());
        }
        if keys.len() > 256 {
            return Err(IndexError::Limit(
                "identity inventory supports 256 roots per batch".into(),
            ));
        }
        let placeholders = vec!["?"; keys.len()].join(",");
        let sql = format!(
            "SELECT r.key,r.name,r.content_hash,su.profile_id,ei.input_hash,ei.token_count FROM records r LEFT JOIN semantic_units su ON su.record_id=r.record_id AND su.unit_kind='identity' LEFT JOIN embedding_inputs ei USING(input_id) WHERE r.document_kind<>'Macro' AND r.key IN ({placeholders}) ORDER BY r.key,su.profile_id"
        );
        self.count_statement();
        let c = self.native()?;
        let mut statement = c.prepare(&sql)?;
        let values = keys.iter().map(ToString::to_string).collect::<Vec<_>>();
        let mut rows = statement.query(params_from_iter(&values))?;
        let mut records = BTreeMap::<RecordKey, IdentityMetadata>::new();
        while let Some(row) = rows.next()? {
            let key = crate::read_units::parse_key(row.get(0)?)?;
            let label = row
                .get::<_, Option<String>>(1)?
                .unwrap_or_else(|| key.to_string());
            let metadata = records.entry(key).or_insert(IdentityMetadata {
                label,
                content_hash: row.get(2)?,
                inputs: Vec::new(),
                has_vectors: false,
            });
            if let Some(stored_profile) = row.get::<_, Option<String>>(3)? {
                metadata.has_vectors |=
                    profile.is_some_and(|profile| profile.as_str() == stored_profile);
                metadata
                    .inputs
                    .push((stored_profile, row.get(4)?, row.get(5)?));
            }
        }
        let preparation = self.context().preparation_sha256()?;
        records
            .into_iter()
            .map(|(key, metadata)| {
                let fingerprint = crate::codec::sha256(&serde_json::to_vec(&(
                    &key,
                    &metadata.content_hash,
                    &preparation,
                    &metadata.inputs,
                ))?);
                Ok((
                    key,
                    SimilarityComponent::Identity {
                        selector: SimilarityPartSelector(format!("identity:{fingerprint}")),
                        label: "Name and identity".into(),
                        owner_label: metadata.label,
                        has_vectors: metadata.has_vectors,
                    },
                ))
            })
            .collect()
    }

    /// Core selectors are exact existing sections plus the native identity component.
    pub fn core_selection(
        &self,
        record: &RecordKey,
        profile: Option<&SearchProfileId>,
    ) -> Result<CoreSectionSelection, IndexError> {
        self.count_statement();
        let c = self.native()?;
        let row:Option<(String,Option<String>,String)>=c.query_row("SELECT cr.family,cr.fallback_reason,cr.recipe FROM core_roots cr JOIN records r USING(record_id) WHERE r.key=?",[record.to_string()],|row|Ok((row.get(0)?,row.get(1)?,row.get(2)?))).optional()?;
        let Some((family, fallback_reason, recipe)) = row else {
            return Err(IndexError::Unavailable("Core section metadata is unavailable; update the reference workspace with atlas setup --offline".into()));
        };
        if recipe != CORE_SECTIONS_RECIPE
            || self.context().core_sections_recipe != CORE_SECTIONS_RECIPE
        {
            return Err(IndexError::Unsupported("Core section recipe differs; update the reference workspace and refresh the selection".into()));
        }
        let family: CoreSectionFamily = serde_json::from_value(serde_json::Value::String(family))?;
        let fallback_reason = fallback_reason
            .map(|value| serde_json::from_value(serde_json::Value::String(value)))
            .transpose()?;
        self.count_statement();
        let mut selectors=c.prepare("SELECT cp.selector FROM core_parts cp JOIN records r USING(record_id) WHERE r.key=? ORDER BY cp.selector")?.query_map([record.to_string()],|row|Ok(SimilarityPartSelector(row.get(0)?)))?.collect::<Result<Vec<_>,_>>()?;
        let representation = if selectors.is_empty() {
            CoreSectionRepresentation::IdentityOnly
        } else {
            CoreSectionRepresentation::IdentityAndProse
        };
        let identity = self
            .identity_components(std::slice::from_ref(record), profile)?
            .remove(record)
            .ok_or_else(|| IndexError::Unavailable(format!("record {record} was not found")))?;
        selectors.insert(0, identity.selector().clone());
        Ok(CoreSectionSelection {
            family,
            selectors,
            representation,
            fallback_reason,
            recipe,
        })
    }
}
