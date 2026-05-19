#[derive(Debug, Default, Clone, Copy)]
pub(super) struct FieldStats {
    pub value_count: u64,
    pub matching_record_count: u64,
    pub null_count: u64,
    pub distinct_count: u64,
    pub singleton_count: u64,
}

impl FieldStats {
    pub(super) fn singleton_ratio(self) -> f64 {
        ratio(self.singleton_count, self.distinct_count)
    }

    pub(super) fn observation_singleton_ratio(self) -> f64 {
        ratio(self.singleton_count, self.value_count)
    }
}

fn ratio(numerator: u64, denominator: u64) -> f64 {
    if denominator == 0 {
        0.0
    } else {
        numerator as f64 / denominator as f64
    }
}
