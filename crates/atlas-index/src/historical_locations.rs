//! Shared source addresses across profiles; digests index full checked metadata.
use crate::{IndexError, embedding_values::hash_bytes};
use rusqlite::{Connection, params};

pub(crate) fn location_id(
    c: &Connection,
    record: i64,
    owners: &str,
    field: Option<&str>,
    address: &str,
    ordinal: usize,
) -> Result<i64, IndexError> {
    let hash = hash_bytes(&serde_json::to_vec(&(
        record, owners, field, address, ordinal,
    ))?);
    c.execute("INSERT INTO semantic_locations(record_id,location_hash,owners_json,field_path,section_json,chunk_ordinal) VALUES(?,?,?,?,?,?) ON CONFLICT DO NOTHING",params![record,hash,owners,field,address,ordinal])?;
    let (id,r,o,f,a,n):(i64,i64,String,Option<String>,String,usize)=c.query_row("SELECT location_id,record_id,owners_json,field_path,section_json,chunk_ordinal FROM semantic_locations WHERE location_hash=?",[hash],|r|Ok((r.get(0)?,r.get(1)?,r.get(2)?,r.get(3)?,r.get(4)?,r.get(5)?)))?;
    if (r, o.as_str(), f.as_deref(), a.as_str(), n) != (record, owners, field, address, ordinal) {
        return Err(IndexError::Invalid(
            "semantic location digest collision".into(),
        ));
    }
    Ok(id)
}
pub(crate) fn validate(c: &Connection) -> Result<(), IndexError> {
    let mut s=c.prepare("SELECT location_id,record_id,location_hash,owners_json,field_path,section_json,chunk_ordinal FROM semantic_locations")?;
    let mut rows = s.query([])?;
    while let Some(row) = rows.next()? {
        let id: i64 = row.get(0)?;
        let record: i64 = row.get(1)?;
        let hash: Vec<u8> = row.get(2)?;
        let owners: String = row.get(3)?;
        let field: Option<String> = row.get(4)?;
        let address: String = row.get(5)?;
        let ordinal: usize = row.get(6)?;
        if hash
            != hash_bytes(&serde_json::to_vec(&(
                record, &owners, &field, &address, ordinal,
            ))?)
        {
            return Err(IndexError::Invalid(format!(
                "semantic location {id} checksum mismatch"
            )));
        }
    }
    let invalid:bool=c.query_row("SELECT EXISTS(SELECT 1 FROM semantic_units u JOIN semantic_locations l USING(location_id) JOIN embedding_inputs i USING(input_id) JOIN embedding_models m USING(model_id) JOIN search_profiles p USING(profile_id) WHERE u.record_id<>l.record_id OR m.identity_json<>p.identity_json OR (u.unit_kind='identity')<>(l.field_path IS NULL))",[],|r|r.get(0))?;
    if invalid {
        return Err(IndexError::Invalid(
            "semantic unit model or attribution mismatch".into(),
        ));
    }
    Ok(())
}
