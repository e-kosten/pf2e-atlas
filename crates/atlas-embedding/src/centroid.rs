//! Deterministic aggregation for complete vocabulary definition passages.
pub fn normalized_centroid(vectors: &[&[f32]]) -> Result<Vec<f32>, crate::EmbeddingError> {
    let dimensions = vectors.first().map_or(0, |v| v.len());
    if dimensions == 0 {
        return Err(crate::EmbeddingError::InvalidVector);
    }
    let mut sum = vec![0f64; dimensions];
    for vector in vectors {
        if vector.len() != dimensions || vector.iter().any(|v| !v.is_finite()) {
            return Err(crate::EmbeddingError::InvalidVector);
        }
        let norm = vector
            .iter()
            .map(|v| f64::from(*v).powi(2))
            .sum::<f64>()
            .sqrt();
        if norm == 0.0 {
            return Err(crate::EmbeddingError::InvalidVector);
        }
        for (sum, v) in sum.iter_mut().zip(*vector) {
            *sum += f64::from(*v) / norm;
        }
    }
    let norm = sum.iter().map(|v| v * v).sum::<f64>().sqrt();
    if norm == 0.0 {
        return Err(crate::EmbeddingError::InvalidVector);
    }
    Ok(sum.into_iter().map(|v| (v / norm) as f32).collect())
}
#[cfg(test)]
mod tests {
    #[test]
    fn centroid_uses_all_chunks_deterministically() {
        let a = [2., 0.];
        let b = [0., 4.];
        let result = super::normalized_centroid(&[&a, &b]).unwrap();
        assert_eq!(result, super::normalized_centroid(&[&a, &b]).unwrap());
        assert!((result[0] - 0.70710677).abs() < 1e-6);
        assert_eq!(result[0], result[1]);
        assert!(super::normalized_centroid(&[&a, &[0., 0.]]).is_err());
    }
}
