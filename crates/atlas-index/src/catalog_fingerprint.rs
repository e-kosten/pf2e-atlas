//! One compatibility fingerprint of the executable filter definitions.
use crate::IndexError;
use sha2::{Digest, Sha256};
use std::sync::OnceLock;

pub(crate) fn fingerprint_v1(fields: impl IntoIterator<Item = (String, String)>) -> [u8; 32] {
    let mut hash = Sha256::new();
    hash.update(b"atlas/query-field-catalog/v1\0");
    for (field, definition) in fields {
        for part in [field, definition] {
            hash.update((part.len() as u64).to_be_bytes());
            hash.update(part.as_bytes());
        }
    }
    hash.finalize().into()
}

pub(crate) fn current() -> Result<&'static [u8; 32], IndexError> {
    static FINGERPRINT: OnceLock<[u8; 32]> = OnceLock::new();
    if let Some(fingerprint) = FINGERPRINT.get() {
        return Ok(fingerprint);
    }
    let mut fields = crate::query::query_capabilities()?.fields;
    fields.sort_by(|left, right| left.id.cmp(&right.id));
    let entries = fields
        .into_iter()
        .map(|field| {
            let definition = serde_json::to_string(&field)?;
            Ok((field.id, definition))
        })
        .collect::<Result<Vec<_>, IndexError>>()?;
    Ok(FINGERPRINT.get_or_init(|| fingerprint_v1(entries)))
}
