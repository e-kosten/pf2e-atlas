//! Ordering reads projected facts; neither membership nor source bodies change.
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

pub(crate) fn attribute_order(sort: ResultSort) -> Result<String, IndexError> {
    let name_missing = "CASE WHEN r.name_state='value' THEN 0 ELSE 1 END";
    let name_ties = format!("{name_missing},r.name_lookup_key ASC,r.key ASC");
    let descending = matches!(
        sort,
        ResultSort::NameDesc | ResultSort::LevelDesc | ResultSort::UnitPriceDesc
    );
    let direction = if descending { "DESC" } else { "ASC" };
    match sort {
        ResultSort::NameAsc | ResultSort::NameDesc => Ok(format!(
            "{name_missing},r.name_lookup_key {direction},r.key ASC"
        )),
        ResultSort::LevelAsc
        | ResultSort::LevelDesc
        | ResultSort::UnitPriceAsc
        | ResultSort::UnitPriceDesc => {
            let field = if sort.is_unit_price() {
                bound_unit_price_cp()?
            } else {
                bound_field(
                    descriptor(None, "level_or_rank")?.ok_or_else(|| {
                        IndexError::Invalid("level/rank binding is unavailable".into())
                    })?,
                    None,
                )
            };
            Ok(format!(
                "CASE WHEN ({})='value' THEN 0 ELSE 1 END,CASE WHEN ({})='value' THEN ({}) ELSE NULL END {direction},{name_ties}",
                field.state, field.state, field.value
            ))
        }
        _ => Err(IndexError::Invalid(
            "attribute ordering requires a name, level/rank or unit-price sort".into(),
        )),
    }
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
        let order = attribute_order(sort)?;
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
