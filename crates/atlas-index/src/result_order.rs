//! Ordering reads projected facts; neither membership nor source bodies change.
//! Random v1 hashes the ASCII domain `pf2e-atlas/result-order/random/v1` plus NUL,
//! the seed as u32 big-endian, canonical-key UTF-8 length as u64 big-endian, then
//! canonical-key UTF-8 bytes. Digest bytes ascend, with canonical-key ties.
use crate::query::{
    catalog::descriptor,
    compiler::{bound_field, bound_unit_price_cp},
};
use crate::{IndexError, SqliteIndexReader};
use atlas_domain::{QueryFieldState, RecordKey, ResultSort, SourceQueryFact};
use rusqlite::{params_from_iter, types::Value};

#[derive(Debug, Clone)]
pub struct SourceUnitPrice {
    pub key: RecordKey,
    pub copper: SourceQueryFact<f64>,
}

/// Each rule puts unavailable values last independently; later rules break unavailable ties.
pub(crate) fn attribute_order(sort: &ResultSort) -> Result<String, IndexError> {
    sort.validate()
        .map_err(|message| IndexError::Invalid(message.into()))?;
    let ResultSort::Attributes { rules } = sort else {
        return Err(IndexError::Invalid(
            "attribute ordering requires attribute rules".into(),
        ));
    };
    let mut comparisons = Vec::new();
    for rule in rules {
        let direction = match rule.direction {
            atlas_domain::SortDirection::Asc => "ASC",
            atlas_domain::SortDirection::Desc => "DESC",
        };
        if rule.field == atlas_domain::SortField::Name {
            comparisons.push(format!("CASE WHEN r.name_state='value' THEN 0 ELSE 1 END,CASE WHEN r.name_state='value' THEN r.name_lookup_key ELSE NULL END {direction}"));
        } else {
            let field = if rule.field == atlas_domain::SortField::UnitPrice {
                bound_unit_price_cp()?
            } else {
                bound_field(
                    descriptor(None, "level_or_rank")?.ok_or_else(|| {
                        IndexError::Invalid("level/rank binding is unavailable".into())
                    })?,
                    None,
                )
            };
            comparisons.push(format!("CASE WHEN ({})='value' THEN 0 ELSE 1 END,CASE WHEN ({})='value' THEN ({}) ELSE NULL END {direction}", field.state, field.state, field.value));
        }
    }
    comparisons.push("CASE WHEN r.name_state='value' THEN 0 ELSE 1 END,CASE WHEN r.name_state='value' THEN r.name_lookup_key ELSE NULL END ASC,r.key ASC".into());
    Ok(comparisons.join(","))
}
/// Stable Random v1: SHA-256 of the ASCII domain ending in NUL, seed u32 BE,
/// canonical-key UTF-8 length u64 BE, then canonical-key UTF-8. Digest ASC, key ASC.
pub(crate) fn random_order(keys: &mut [RecordKey], seed: u32) {
    use sha2::{Digest, Sha256};
    keys.sort_by_cached_key(|key| {
        let text = key.to_string();
        let mut hash = Sha256::new();
        hash.update(b"pf2e-atlas/result-order/random/v1\0");
        hash.update(seed.to_be_bytes());
        hash.update((text.len() as u64).to_be_bytes());
        hash.update(text.as_bytes());
        let digest: [u8; 32] = hash.finalize().into();
        (digest, text)
    });
}
fn key_values(keys: &[RecordKey]) -> Result<String, IndexError> {
    Ok(serde_json::to_string(
        &keys.iter().map(ToString::to_string).collect::<Vec<_>>(),
    )?)
}
impl SqliteIndexReader {
    /// Order every supplied candidate at once. A missing candidate is reported by the caller's length/identity check.
    pub fn order_record_keys(
        &self,
        keys: &[RecordKey],
        sort: ResultSort,
    ) -> Result<Vec<RecordKey>, IndexError> {
        sort.validate()
            .map_err(|message| IndexError::Invalid(message.into()))?;
        if let ResultSort::Random { seed } = sort {
            self.count_statement();
            let mut statement = self.native()?.prepare("SELECT r.key FROM records r JOIN json_each(?) requested ON requested.value=r.key WHERE r.document_kind<>'Macro'")?;
            let strings = statement
                .query_map([key_values(keys)?], |row| row.get::<_, String>(0))?
                .collect::<Result<Vec<_>, _>>()?;
            let mut ordered = strings
                .into_iter()
                .map(crate::read_units::parse_key)
                .collect::<Result<Vec<_>, _>>()?;
            random_order(&mut ordered, seed);
            return Ok(ordered);
        }
        let order = attribute_order(&sort)?;
        if keys.is_empty() {
            return Ok(Vec::new());
        }
        let sql = format!(
            "SELECT r.key FROM records r JOIN json_each(?) requested ON requested.value=r.key WHERE r.document_kind<>'Macro' ORDER BY {order}"
        );
        self.count_statement();
        let mut statement = self.native()?.prepare(&sql)?;
        let strings = statement
            .query_map([key_values(keys)?], |row| row.get::<_, String>(0))?
            .collect::<Result<Vec<_>, _>>()?;
        strings
            .into_iter()
            .map(crate::read_units::parse_key)
            .collect()
    }
    /// One body-free batch, including explicit availability for every product key.
    pub fn read_unit_prices(&self, keys: &[RecordKey]) -> Result<Vec<SourceUnitPrice>, IndexError> {
        if keys.is_empty() {
            return Ok(Vec::new());
        }
        let field = bound_unit_price_cp()?;
        let sql = format!(
            "SELECT r.key,{},{} FROM records r JOIN json_each(?) requested ON requested.value=r.key WHERE r.document_kind<>'Macro'",
            field.state, field.value
        );
        self.count_statement();
        let mut statement = self.native()?.prepare(&sql)?;
        let rows = statement
            .query_map(params_from_iter([Value::Text(key_values(keys)?)]), |row| {
                Ok((
                    row.get::<_, String>(0)?,
                    row.get::<_, String>(1)?,
                    row.get::<_, Option<f64>>(2)?,
                ))
            })?
            .collect::<Result<Vec<_>, _>>()?;
        rows.into_iter()
            .map(|(key, state, value)| {
                let state: QueryFieldState =
                    serde_json::from_value(serde_json::Value::String(state))?;
                let copper = SourceQueryFact { state, value };
                if !copper.is_valid() || copper.value.is_some_and(|v| !v.is_finite()) {
                    return Err(IndexError::Invalid("invalid stored unit-price fact".into()));
                }
                Ok(SourceUnitPrice {
                    key: crate::read_units::parse_key(key)?,
                    copper,
                })
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn random_v1_encoding_has_fixed_seed_key_order_vectors() {
        let key = |i| RecordKey::parse(&format!("items:{i:016}")).unwrap();
        for (seed, expected) in [
            (0, vec![3, 2, 1, 0]),
            (42, vec![0, 1, 2, 3]),
            (u32::MAX, vec![1, 2, 0, 3]),
        ] {
            let mut keys = (0..4).rev().map(key).collect::<Vec<_>>();
            random_order(&mut keys, seed);
            assert_eq!(keys, expected.into_iter().map(key).collect::<Vec<_>>());
        }
    }
}
