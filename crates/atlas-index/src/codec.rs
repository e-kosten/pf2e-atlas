use crate::IndexError;
use flate2::{Compression, read::GzDecoder, write::GzEncoder};
use sha2::{Digest, Sha256};
use std::io::{Read, Write};
pub(crate) const MAX_DECODED_BYTES: u64 = 256 * 1024 * 1024;
pub(crate) fn gzip(bytes: &[u8]) -> Result<Vec<u8>, IndexError> {
    let mut writer = GzEncoder::new(Vec::new(), Compression::new(6));
    writer.write_all(bytes)?;
    Ok(writer.finish()?)
}
pub(crate) fn gunzip(bytes: &[u8]) -> Result<Vec<u8>, IndexError> {
    let mut output = Vec::new();
    GzDecoder::new(bytes)
        .take(MAX_DECODED_BYTES + 1)
        .read_to_end(&mut output)
        .map_err(|e| IndexError::Invalid(format!("compressed field: {e}")))?;
    if output.len() as u64 > MAX_DECODED_BYTES {
        return Err(IndexError::Invalid(
            "compressed field exceeds decoded byte bound".into(),
        ));
    }
    Ok(output)
}
pub(crate) fn sha256(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}
pub(crate) fn checked_hash(hash: &str) -> bool {
    hash.len() == 64
        && hash
            .bytes()
            .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
}
