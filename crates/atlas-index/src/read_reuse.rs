//! Bounded old-source reconstruction for tokenizer-backed ingest reuse checks.
use crate::{IndexError, SqliteIndexReader};
use atlas_domain::RecordKey;
use atlas_record::source_content::{ContentReferenceOccurrence, SourceContentLocator};
use atlas_record::source_record::{
    SourceBackedRecord, SourceSearchSelection, select_record_search,
};
use rusqlite::params_from_iter;
use std::collections::BTreeMap;

impl SqliteIndexReader {
    /// Checked old DTO/cache selections, including the exact verified alias
    /// ordering used by the producer. This performs four batched reads and no
    /// per-unit or per-marker queries. It does not verify assembled input hashes.
    pub fn read_search_selections_for_reuse(
        &self,
        keys: &[RecordKey],
    ) -> Result<Vec<SourceSearchSelection>, IndexError> {
        if keys.len() > 128 {
            return Err(IndexError::Invalid(
                "reuse source batch exceeds 128 roots".into(),
            ));
        }
        if keys.is_empty() {
            return Ok(vec![]);
        }
        let c = self.native()?;
        let slots = vec!["?"; keys.len()].join(",");
        self.count_statement();
        let mut statement = c.prepare(&format!("SELECT r.record_id,r.key,r.pack_id,p.label,b.codec_version,b.encoding,b.snapshot FROM records r JOIN packs p USING(pack_id) JOIN record_bodies b USING(record_id) WHERE r.document_kind<>'Macro' AND r.key IN ({slots}) ORDER BY r.record_id"))?;
        let mut rows = statement.query(params_from_iter(keys.iter().map(ToString::to_string)))?;
        let mut records = BTreeMap::new();
        while let Some(row) = rows.next()? {
            let version: i64 = row.get(4)?;
            let encoding: String = row.get(5)?;
            if version != i64::from(atlas_foundry_model::SNAPSHOT_VERSION) || encoding != "gzip6" {
                return Err(IndexError::Unsupported(
                    "reuse source snapshot codec".into(),
                ));
            }
            let source = atlas_foundry_model::decode_snapshot(&crate::codec::gunzip(
                &row.get::<_, Vec<u8>>(6)?,
            )?)?;
            let record =
                SourceBackedRecord::new(&row.get::<_, String>(2)?, source).map_err(|e| {
                    IndexError::Invalid(format!("reuse source identity {:?}", e.reason))
                })?;
            if record.key().to_string() != row.get::<_, String>(1)? {
                return Err(IndexError::Invalid(
                    "reuse snapshot identity mismatch".into(),
                ));
            }
            self.count_source_decode();
            records.insert(
                row.get::<_, i64>(0)?,
                (record, row.get::<_, String>(3)?, vec![], vec![]),
            );
        }
        if records.len() != keys.len() {
            return Err(IndexError::Invalid(
                "reuse source root missing or duplicated".into(),
            ));
        }
        let ids = records.keys().copied().collect::<Vec<_>>();
        let slots = vec!["?"; ids.len()].join(",");
        self.count_statement();
        let mut statement = c.prepare(&format!(
            "{} WHERE o.origin='content' AND o.record_id IN ({slots}) ORDER BY o.record_id,o.occurrence_id",
            crate::read_content::RELATIONSHIP_SELECT
        ))?;
        let mut rows = statement.query(params_from_iter(&ids))?;
        let mut references = BTreeMap::<
            crate::read_content::ContentReferenceKey,
            Vec<ContentReferenceOccurrence>,
        >::new();
        while let Some(row) = rows.next()? {
            let relationship = crate::read_content::relationship_row(row)?;
            let locator = crate::read_content::content_reference_key(&relationship.locator)?;
            references
                .entry(locator)
                .or_default()
                .push(crate::read_content::content_reference(relationship)?);
        }
        self.count_statement();
        self.count_reuse_content_batch();
        let mut statement=c.prepare(&format!("SELECT owners_json,field_path,role,visibility,authored_markup_sha256,html_gzip,interactions_json,preparation_context_hash,record_id FROM prepared_content WHERE outcome='prepared' AND record_id IN ({slots}) ORDER BY record_id,content_id"))?;
        let mut rows = statement.query(params_from_iter(&ids))?;
        while let Some(row) = rows.next()? {
            let (record, _, outcomes, _) = records
                .get_mut(&row.get::<_, i64>(8)?)
                .ok_or_else(|| IndexError::Invalid("reuse cache root unavailable".into()))?;
            let locator = SourceContentLocator {
                record: record.key().clone(),
                owners: serde_json::from_str(&row.get::<_, String>(0)?)?,
                field: row.get(1)?,
            };
            outcomes.push(crate::validation_units::stored_content_outcome(
                row,
                record,
                self.context(),
                references
                    .remove(&crate::read_content::content_reference_key(&locator)?)
                    .unwrap_or_default(),
            )?);
        }
        if !references.is_empty() {
            return Err(IndexError::Invalid(
                "reuse content reference has no prepared cache".into(),
            ));
        }
        self.count_statement();
        let mut statement=c.prepare(&format!("SELECT record_id,alias FROM verified_aliases WHERE record_id IN ({slots}) ORDER BY record_id,alias"))?;
        let mut rows = statement.query(params_from_iter(&ids))?;
        while let Some(row) = rows.next()? {
            let (_, _, _, aliases) = records
                .get_mut(&row.get::<_, i64>(0)?)
                .ok_or_else(|| IndexError::Invalid("reuse alias root unavailable".into()))?;
            aliases.push(row.get::<_, String>(1)?);
        }
        records
            .into_values()
            .map(|(record, label, outcomes, aliases)| {
                let mut selection = select_record_search(
                    &record,
                    &outcomes,
                    self.context().audience,
                    &label,
                    &self.context().used_trait_labels,
                )
                .map_err(|e| IndexError::Invalid(e.to_string()))?;
                selection.identity_optional.extend(aliases);
                Ok(selection)
            })
            .collect()
    }
}
