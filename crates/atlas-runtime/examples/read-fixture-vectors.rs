//! Reads selected published values from a disposable compatibility fixture.
use rusqlite::{Connection, OpenFlags, params};
use sha2::{Digest, Sha256};
use std::{error::Error, path::PathBuf};

fn main() -> Result<(), Box<dyn Error>> {
    let mut args = std::env::args_os().skip(1);
    let path = PathBuf::from(args.next().ok_or("expected fixture database path")?);
    let profile = args.next().ok_or("expected profile ID")?;
    let profile = profile.to_str().ok_or("invalid profile ID")?;
    atlas_sqlite_vec::register_sqlite_vec_auto_extension()?;
    let db = Connection::open_with_flags(path, OpenFlags::SQLITE_OPEN_READ_ONLY)?;
    let (model_id, identity): (i64, String) = db.query_row(
        "SELECT model_id, identity_json FROM embedding_models
         WHERE json_extract(identity_json, '$.profile_id') = ?",
        [profile],
        |row| Ok((row.get(0)?, row.get(1)?)),
    )?;
    let dimensions = serde_json::from_str::<serde_json::Value>(&identity)?["dimensions"]
        .as_u64()
        .ok_or("invalid vector dimensions")? as usize;
    let table = format!("native_vectors_{:x}", Sha256::digest(identity.as_bytes()));
    let mut samples = Vec::new();
    for hash in args {
        let hash = hash.to_str().ok_or("invalid input hash")?;
        let (tokens, expected, vector): (u64, Vec<u8>, Vec<u8>) = db.query_row(
            &format!(
                "SELECT i.token_count, i.published_hash, v.embedding FROM embedding_inputs i
                 JOIN {table} v USING(input_id)
                 WHERE i.model_id = ? AND lower(hex(i.input_hash)) = ?"
            ),
            params![model_id, hash],
            |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
        )?;
        if vector.len() != dimensions * 4 || Sha256::digest(&vector)[..] != expected {
            return Err(format!("Invalid stored vector for {profile} {hash}").into());
        }
        let values: Vec<_> = vector
            .as_chunks::<4>()
            .0
            .iter()
            .map(|bytes| f32::from_le_bytes(*bytes))
            .collect();
        if values.iter().any(|value| !value.is_finite()) {
            return Err(format!("Non-finite vector for {profile} {hash}").into());
        }
        samples.push(serde_json::json!({"hash": hash, "tokens": tokens, "values": values}));
    }
    println!("{}", serde_json::to_string(&samples)?);
    Ok(())
}
