//! Hybrid lane scores. Identity tiers and eligibility are independent of fusion.
use atlas_domain::SearchRanking;

pub(crate) fn lane_scores(values: &[f64], ranking: SearchRanking, weight: f64) -> Vec<f64> {
    match ranking {
        SearchRanking::Rrf => (1..=values.len())
            .map(|rank| 1.0 / (60.0 + rank as f64))
            .collect(),
        SearchRanking::Score => {
            let minimum = values.iter().copied().fold(f64::INFINITY, f64::min);
            let maximum = values.iter().copied().fold(f64::NEG_INFINITY, f64::max);
            values
                .iter()
                .map(|value| {
                    // A nonempty constant lane still supplies evidence; absent lanes do not.
                    weight
                        * if maximum == minimum {
                            1.0
                        } else {
                            (value - minimum) / (maximum - minimum)
                        }
                })
                .collect()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn normalization_keeps_empty_constant_and_unequal_lanes_distinct() {
        assert!(lane_scores(&[], SearchRanking::Score, 0.25).is_empty());
        assert_eq!(
            lane_scores(&[2.0, 2.0], SearchRanking::Score, 0.25),
            [0.25, 0.25]
        );
        assert_eq!(
            lane_scores(&[4.0, 2.0, 0.0], SearchRanking::Score, 0.25),
            [0.25, 0.125, 0.0]
        );
        assert_eq!(
            lane_scores(&[0.9, 0.1], SearchRanking::Rrf, 0.25),
            [1.0 / 61.0, 1.0 / 62.0]
        );
    }
}
