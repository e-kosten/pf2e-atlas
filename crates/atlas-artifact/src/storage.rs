use std::{error::Error, fmt};

pub const F32_VECTOR_ELEMENT_BYTES: usize = size_of::<f32>();

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct VectorBlobDecodeError {
    len: usize,
}

impl VectorBlobDecodeError {
    pub const fn len(&self) -> usize {
        self.len
    }

    pub const fn is_empty(&self) -> bool {
        self.len == 0
    }
}

impl fmt::Display for VectorBlobDecodeError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "f32 vector blob length {} is not divisible by {F32_VECTOR_ELEMENT_BYTES}",
            self.len
        )
    }
}

impl Error for VectorBlobDecodeError {}

pub fn encode_f32_vector_blob(vector: &[f32]) -> Vec<u8> {
    let mut bytes = Vec::with_capacity(vector.len() * F32_VECTOR_ELEMENT_BYTES);
    for value in vector {
        bytes.extend_from_slice(&value.to_le_bytes());
    }
    bytes
}

pub fn decode_f32_vector_blob(blob: &[u8]) -> Result<Vec<f32>, VectorBlobDecodeError> {
    if !blob.len().is_multiple_of(F32_VECTOR_ELEMENT_BYTES) {
        return Err(VectorBlobDecodeError { len: blob.len() });
    }

    Ok(blob
        .chunks_exact(F32_VECTOR_ELEMENT_BYTES)
        .map(|chunk| f32::from_le_bytes(chunk.try_into().expect("chunk length is fixed")))
        .collect())
}

pub const fn f32_vector_blob_len(dimensions: usize) -> usize {
    dimensions * F32_VECTOR_ELEMENT_BYTES
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn encodes_vectors_as_little_endian_f32_blobs() {
        assert_eq!(
            encode_f32_vector_blob(&[1.0, -2.5]),
            [1.0f32.to_le_bytes(), (-2.5f32).to_le_bytes()].concat()
        );
    }

    #[test]
    fn decodes_little_endian_f32_blobs() {
        let blob = [1.0f32.to_le_bytes(), (-2.5f32).to_le_bytes()].concat();

        assert_eq!(decode_f32_vector_blob(&blob), Ok(vec![1.0, -2.5]));
    }

    #[test]
    fn rejects_invalid_blob_lengths() {
        let error = decode_f32_vector_blob(&[0, 1, 2]).expect_err("blob length should be invalid");

        assert_eq!(error.len(), 3);
    }
}
