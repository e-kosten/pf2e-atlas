//! Deterministic, bounded aggregation for complete vocabulary definition passages.
/// Ordered normalized accumulation retains one f64 value per dimension.
pub struct NormalizedCentroid {
    sum: Vec<f64>,
    chunks: usize,
}
impl NormalizedCentroid {
    pub fn new(dimensions: usize) -> Result<Self, crate::EmbeddingError> {
        if dimensions == 0 {
            return Err(crate::EmbeddingError::InvalidVector);
        }
        Ok(Self {
            sum: vec![0.; dimensions],
            chunks: 0,
        })
    }
    pub fn chunks(&self) -> usize {
        self.chunks
    }
    pub fn push(&mut self, vector: &[f32]) -> Result<(), crate::EmbeddingError> {
        if vector.len() != self.sum.len() || vector.iter().any(|v| !v.is_finite()) {
            return Err(crate::EmbeddingError::InvalidVector);
        }
        let norm = vector
            .iter()
            .map(|v| f64::from(*v).powi(2))
            .sum::<f64>()
            .sqrt();
        if norm == 0. {
            return Err(crate::EmbeddingError::InvalidVector);
        }
        for (sum, v) in self.sum.iter_mut().zip(vector) {
            *sum += f64::from(*v) / norm;
        }
        self.chunks += 1;
        Ok(())
    }
    pub fn finish(self) -> Result<Vec<f32>, crate::EmbeddingError> {
        let norm = self.sum.iter().map(|v| v * v).sum::<f64>().sqrt();
        if norm == 0. {
            return Err(crate::EmbeddingError::InvalidVector);
        }
        Ok(self.sum.into_iter().map(|v| (v / norm) as f32).collect())
    }
}
pub fn normalized_centroid(vectors: &[&[f32]]) -> Result<Vec<f32>, crate::EmbeddingError> {
    let mut accumulator = NormalizedCentroid::new(vectors.first().map_or(0, |v| v.len()))?;
    for vector in vectors {
        accumulator.push(vector)?;
    }
    accumulator.finish()
}
#[cfg(test)]
mod tests {
    #[test]
    fn streaming_centroid_preserves_order_and_repeated_chunk_weights() {
        let vectors = [
            [0.1f32, 0.9, -0.2],
            [1., -0.3, 0.01],
            [0.1, 0.9, -0.2],
            [-0.8, 0.7, 0.2],
        ];
        // Independent frozen recipe: normalize each ordered chunk in f64, sum,
        // then normalize the ordered sum. Repeated chunks keep their full weight.
        let mut sum = [0f64; 3];
        for v in vectors {
            let norm = v.iter().map(|x| f64::from(*x).powi(2)).sum::<f64>().sqrt();
            for i in 0..3 {
                sum[i] += f64::from(v[i]) / norm;
            }
        }
        let norm = sum.iter().map(|x| x * x).sum::<f64>().sqrt();
        let expected = sum.map(|x| ((x / norm) as f32).to_bits());
        let mut accumulator = super::NormalizedCentroid::new(3).unwrap();
        for v in vectors {
            accumulator.push(&v).unwrap();
        }
        assert_eq!(accumulator.chunks(), 4);
        assert_eq!(
            accumulator
                .finish()
                .unwrap()
                .iter()
                .map(|x| x.to_bits())
                .collect::<Vec<_>>(),
            expected
        );
        assert!(super::NormalizedCentroid::new(0).is_err());
        assert!(super::NormalizedCentroid::new(3).unwrap().finish().is_err());
    }
    #[test]
    fn centroid_uses_all_chunks_deterministically() {
        let a = [2., 0.];
        let b = [0., 4.];
        let result = super::normalized_centroid(&[&a, &b]).unwrap();
        assert_eq!(result, super::normalized_centroid(&[&a, &b]).unwrap());
        assert_eq!(result.len(), 2);
        assert!((result[0] - 0.70710677).abs() < 1e-6);
        assert_eq!(result[0], result[1]);
        for (name, vectors) in [
            ("no chunks", vec![]),
            ("empty chunk", vec![&[][..]]),
            ("zero norm", vec![&a[..], &[0., 0.][..]]),
            ("dimension mismatch", vec![&a[..], &[1.][..]]),
            ("nonfinite chunk", vec![&a[..], &[f32::INFINITY, 0.][..]]),
            ("cancelled centroid", vec![&a[..], &[-2., 0.][..]]),
        ] {
            assert!(super::normalized_centroid(&vectors).is_err(), "{name}");
        }
    }
}
