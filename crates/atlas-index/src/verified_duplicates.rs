//! Active duplicate authority and shared catalog membership.
use crate::{IndexError, PublicationSpool, SqliteIndexReader, query::ValidatedQuery};
use atlas_domain::RecordKey;
use rusqlite::{Connection, OptionalExtension, params, params_from_iter, types::Value};
use serde::Serialize;
use sha2::{Digest, Sha256};
use std::collections::{BTreeMap, BTreeSet};

pub const VERIFIED_DUPLICATE_ALIAS_EVIDENCE: &str = "Verified duplicate source title";

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum DiscoveryScope {
    #[default]
    Exact,
    Catalog,
}

#[derive(Debug, Clone, Serialize)]
pub struct VerifiedDuplicateInput {
    pub duplicate: RecordKey,
    pub canonical: RecordKey,
    pub duplicate_sha256: String,
    pub canonical_sha256: String,
}

pub(crate) fn inventory_identity(
    pairs: &[VerifiedDuplicateInput],
) -> Result<Option<String>, IndexError> {
    if pairs.is_empty() {
        return Ok(None);
    }
    let mut ordered = pairs.iter().collect::<Vec<_>>();
    ordered.sort_unstable_by(|a, b| a.duplicate.cmp(&b.duplicate));
    Ok(Some(format!(
        "{:x}",
        Sha256::digest(serde_json::to_vec(&ordered)?)
    )))
}

pub(crate) fn publish(
    connection: &Connection,
    input: &PublicationSpool,
    ids: &BTreeMap<RecordKey, i64>,
) -> Result<(), IndexError> {
    let mut kept = BTreeSet::new();
    for pair in &input.verified_duplicates {
        let duplicate = crate::writer::product_id(ids, &pair.duplicate)?;
        let canonical = crate::writer::product_id(ids, &pair.canonical)?;
        if !kept.insert(duplicate) {
            return Err(IndexError::Invalid("conflicting verified duplicate".into()));
        }
        connection.execute(
            "INSERT INTO verified_duplicate_records VALUES (?1,?2,?3,?4)
             ON CONFLICT(duplicate_record_id) DO UPDATE SET canonical_record_id=excluded.canonical_record_id,
                 duplicate_sha256=excluded.duplicate_sha256,canonical_sha256=excluded.canonical_sha256
             WHERE (canonical_record_id,duplicate_sha256,canonical_sha256) IS NOT
                   (excluded.canonical_record_id,excluded.duplicate_sha256,excluded.canonical_sha256)",
            params![duplicate, canonical, pair.duplicate_sha256, pair.canonical_sha256],
        )?;
    }
    connection.execute(
        "DELETE FROM verified_duplicate_records WHERE duplicate_record_id NOT IN (SELECT value FROM json_each(?1))",
        [serde_json::to_string(&kept)?],
    )?;
    connection.execute(
        "DELETE FROM developer_diagnostics WHERE stage='duplicate_review'",
        [],
    )?;
    for (key, diagnostic) in &input.duplicate_diagnostics {
        crate::writer::write_diagnostic(
            connection,
            crate::writer::product_id(ids, key)?,
            diagnostic,
        )?;
    }
    Ok(())
}

pub(crate) fn validate(connection: &Connection, expected: Option<&str>) -> Result<(), IndexError> {
    let invalid: bool = connection.query_row(
        "SELECT EXISTS (
            SELECT 1 FROM verified_duplicate_records d
            LEFT JOIN records a ON a.record_id=d.duplicate_record_id
            LEFT JOIN records b ON b.record_id=d.canonical_record_id
            WHERE a.source_type IS NOT 'npc' OR b.source_type IS NOT 'npc'
               OR a.content_hash IS NOT d.duplicate_sha256 OR b.content_hash IS NOT d.canonical_sha256
               OR a.publication_remaster IS NULL OR a.publication_remaster IS NOT b.publication_remaster
               OR EXISTS(SELECT 1 FROM verified_duplicate_records next WHERE next.duplicate_record_id=d.canonical_record_id)
               OR (a.name_lookup_key IS NOT b.name_lookup_key AND NOT EXISTS(
                    SELECT 1 FROM verified_aliases v WHERE v.record_id=b.record_id AND v.alias=a.name
               ))
        ) OR EXISTS (
            SELECT 1 FROM verified_aliases v
            WHERE v.evidence_json=json_quote(?1)
              AND NOT EXISTS (
                SELECT 1 FROM verified_duplicate_records d
                JOIN records a ON a.record_id=d.duplicate_record_id
                WHERE d.canonical_record_id=v.record_id AND a.name=v.alias
              )
        )", [VERIFIED_DUPLICATE_ALIAS_EVIDENCE], |row| row.get(0),
    )?;
    if invalid {
        return Err(IndexError::Invalid(
            "invalid verified duplicate relation or missing title alias".into(),
        ));
    }
    let mut statement = connection.prepare(
        "SELECT a.key,b.key,d.duplicate_sha256,d.canonical_sha256 FROM verified_duplicate_records d
         JOIN records a ON a.record_id=d.duplicate_record_id
         JOIN records b ON b.record_id=d.canonical_record_id",
    )?;
    let pairs = statement
        .query_map([], |row| {
            Ok((
                row.get::<_, String>(0)?,
                row.get::<_, String>(1)?,
                row.get(2)?,
                row.get(3)?,
            ))
        })?
        .map(|row| {
            let (duplicate, canonical, duplicate_sha256, canonical_sha256) = row?;
            Ok(VerifiedDuplicateInput {
                duplicate: crate::read_units::parse_key(duplicate)?,
                canonical: crate::read_units::parse_key(canonical)?,
                duplicate_sha256,
                canonical_sha256,
            })
        })
        .collect::<Result<Vec<_>, IndexError>>()?;
    if inventory_identity(&pairs)?.as_deref() != expected {
        return Err(IndexError::Invalid(
            "verified duplicate inventory differs from publication context".into(),
        ));
    }
    Ok(())
}

/// First test the complete predicate on exact source rows, then collapse discovery IDs.
pub(crate) fn eligible_sql(
    query: &ValidatedQuery,
    keys: Option<&[RecordKey]>,
    scope: DiscoveryScope,
    exclude: Option<&RecordKey>,
) -> Result<(String, Vec<Value>), IndexError> {
    let (mut sql, mut parameters) = crate::read_units::eligible_sql(query, keys)?;
    if scope == DiscoveryScope::Catalog {
        sql = format!(
            "SELECT DISTINCT coalesce(d.canonical_record_id,m.record_id) AS record_id
             FROM ({sql}) m LEFT JOIN verified_duplicate_records d ON d.duplicate_record_id=m.record_id"
        );
    }
    if let Some(exclude) = exclude {
        let seed = if scope == DiscoveryScope::Catalog {
            "SELECT coalesce(d.canonical_record_id,r.record_id) FROM records r
             LEFT JOIN verified_duplicate_records d ON d.duplicate_record_id=r.record_id"
        } else {
            "SELECT r.record_id FROM records r"
        };
        parameters.push(Value::Text(exclude.to_string()));
        sql = format!(
            "SELECT record_id FROM ({sql}) WHERE record_id NOT IN ({seed} WHERE r.key=?{})",
            parameters.len()
        );
    }
    Ok((sql, parameters))
}

impl SqliteIndexReader {
    pub fn verified_identity_members(
        &self,
        mut keys: Vec<RecordKey>,
    ) -> Result<Vec<RecordKey>, IndexError> {
        if keys.is_empty() {
            return Ok(keys);
        }
        let connection = self.native()?;
        let mut statement = connection.prepare(
            "SELECT a.key FROM verified_duplicate_records d
             JOIN records a ON a.record_id=d.duplicate_record_id
             JOIN records b ON b.record_id=d.canonical_record_id
             WHERE b.key IN (SELECT value FROM json_each(?1))",
        )?;
        let selected = serde_json::to_string(&keys)?;
        for row in statement.query_map([selected], |row| row.get(0))? {
            keys.push(crate::read_units::parse_key(row?)?);
        }
        keys.sort_unstable();
        keys.dedup();
        Ok(keys)
    }

    pub fn verified_canonical_key(&self, key: &RecordKey) -> Result<Option<RecordKey>, IndexError> {
        let value = self
            .native()?
            .query_row(
                "SELECT b.key FROM verified_duplicate_records d
             JOIN records a ON a.record_id=d.duplicate_record_id
             JOIN records b ON b.record_id=d.canonical_record_id WHERE a.key=?1",
                [key.to_string()],
                |row| row.get::<_, String>(0),
            )
            .optional()?;
        value.map(crate::read_units::parse_key).transpose()
    }

    pub fn verified_alternate_publications(
        &self,
        key: &RecordKey,
    ) -> Result<Vec<String>, IndexError> {
        let connection = self.native()?;
        let mut statement = connection.prepare(
            "SELECT DISTINCT a.publication_title FROM verified_duplicate_records d
             JOIN records a ON a.record_id=d.duplicate_record_id
             JOIN records b ON b.record_id=d.canonical_record_id
             WHERE b.key=?1 AND a.publication_title IS NOT NULL
               AND a.publication_title IS NOT b.publication_title ORDER BY a.publication_title",
        )?;
        Ok(statement
            .query_map([key.to_string()], |row| row.get(0))?
            .collect::<Result<_, _>>()?)
    }

    pub fn verified_duplicate_title_hits(
        &self,
        text: &str,
        query: &ValidatedQuery,
        keys: Option<&[RecordKey]>,
    ) -> Result<Vec<RecordKey>, IndexError> {
        let (eligible, mut parameters) = eligible_sql(query, keys, DiscoveryScope::Catalog, None)?;
        parameters.push(Value::Text(atlas_domain::normalize_record_name(text)));
        let connection = self.native()?;
        let mut statement = connection.prepare(&format!(
            "SELECT DISTINCT b.key FROM verified_duplicate_records d
             JOIN records a ON a.record_id=d.duplicate_record_id
             JOIN records b ON b.record_id=d.canonical_record_id
             WHERE a.name_lookup_key=?{} AND b.record_id IN ({eligible}) ORDER BY b.key",
            parameters.len(),
        ))?;
        statement
            .query_map(params_from_iter(&parameters), |row| row.get(0))?
            .map(|key| crate::read_units::parse_key(key?))
            .collect()
    }
}
