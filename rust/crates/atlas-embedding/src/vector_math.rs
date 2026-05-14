use crate::error::EmbeddingError;

pub(crate) fn mean_pool_normalized(
    data: &[f32],
    tokens: usize,
    dimensions: usize,
    attention_mask: &[u32],
) -> Result<Vec<f32>, EmbeddingError> {
    let mut vector = vec![0.0; dimensions];
    let mut token_count = 0.0f32;

    for token_index in 0..tokens {
        if attention_mask.get(token_index).copied().unwrap_or(0) == 0 {
            continue;
        }
        token_count += 1.0;
        let offset = token_index * dimensions;
        for dimension in 0..dimensions {
            vector[dimension] += data[offset + dimension];
        }
    }

    if token_count == 0.0 {
        return Ok(vector);
    }

    for value in &mut vector {
        *value /= token_count;
    }
    normalize_vector(&mut vector);
    Ok(vector)
}

fn normalize_vector(vector: &mut [f32]) {
    let magnitude = vector.iter().map(|value| value * value).sum::<f32>().sqrt();
    if magnitude == 0.0 {
        return;
    }
    for value in vector {
        *value /= magnitude;
    }
}
