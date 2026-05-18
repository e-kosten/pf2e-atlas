use atlas_domain::{FilterFieldStats, FilterSampleExample, FilterValueCount, NumericFieldStats};

const SAMPLE_TEXT_LIMIT: usize = 160;

pub(super) fn stats_from_counts(values: &[FilterValueCount]) -> FilterFieldStats {
    let value_count = values.iter().map(|value| value.count).sum::<u64>();
    let distinct_count = values.len() as u64;
    let singleton_count = values.iter().filter(|value| value.count == 1).count() as u64;
    FilterFieldStats {
        value_count,
        null_count: 0,
        distinct_count,
        singleton_count,
        singleton_ratio: ratio(singleton_count, distinct_count),
        observation_singleton_ratio: ratio(singleton_count, value_count),
    }
}

pub(super) fn numeric_stats_from_values(
    sorted_values: &[f64],
    matching_count: u64,
) -> NumericFieldStats {
    let count = sorted_values.len() as u64;
    let null_count = matching_count.saturating_sub(count);
    NumericFieldStats {
        count,
        null_count,
        min: sorted_values.first().copied(),
        p05: percentile(sorted_values, 0.05),
        p25: percentile(sorted_values, 0.25),
        p50: percentile(sorted_values, 0.50),
        mean: (!sorted_values.is_empty())
            .then(|| sorted_values.iter().sum::<f64>() / sorted_values.len() as f64),
        p75: percentile(sorted_values, 0.75),
        p95: percentile(sorted_values, 0.95),
        max: sorted_values.last().copied(),
    }
}

pub(super) fn numeric_stats_from_row(
    row: &rusqlite::Row<'_>,
) -> rusqlite::Result<NumericFieldStats> {
    Ok(NumericFieldStats {
        count: row.get(0)?,
        null_count: row.get(1)?,
        min: row.get(2)?,
        p05: row.get(3)?,
        p25: row.get(4)?,
        p50: row.get(5)?,
        mean: row.get(6)?,
        p75: row.get(7)?,
        p95: row.get(8)?,
        max: row.get(9)?,
    })
}

pub(super) fn sample_example(value: &FilterValueCount) -> FilterSampleExample {
    let (text, truncated) = truncate_sample_text(&value.value);
    FilterSampleExample {
        text,
        count: value.count,
        truncated,
    }
}

fn percentile(sorted_values: &[f64], percentile: f64) -> Option<f64> {
    if sorted_values.is_empty() {
        return None;
    }
    let rank = ((sorted_values.len() as f64) * percentile).ceil() as usize;
    let index = rank.saturating_sub(1).min(sorted_values.len() - 1);
    sorted_values.get(index).copied()
}

fn truncate_sample_text(value: &str) -> (String, bool) {
    if value.chars().count() <= SAMPLE_TEXT_LIMIT {
        return (value.to_string(), false);
    }
    (value.chars().take(SAMPLE_TEXT_LIMIT).collect(), true)
}

fn ratio(numerator: u64, denominator: u64) -> f64 {
    if denominator == 0 {
        0.0
    } else {
        numerator as f64 / denominator as f64
    }
}
