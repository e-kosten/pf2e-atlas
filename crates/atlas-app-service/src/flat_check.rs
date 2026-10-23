use crate::{AppServiceError, AppServiceResult, AtlasAppService};
use atlas_app_model::{FlatCheckRollView, RollFlatCheckRequest};

impl AtlasAppService {
    pub fn roll_flat_check(
        &self,
        request: RollFlatCheckRequest,
    ) -> AppServiceResult<FlatCheckRollView> {
        roll(request)
    }
}
fn roll(request: RollFlatCheckRequest) -> AppServiceResult<FlatCheckRollView> {
    if !(0..=100).contains(&request.dc) {
        return Err(AppServiceError::invalid_request(
            "DC must be between 0 and 100.",
        ));
    }
    let outcome = atlas_dice::roll("1d20")
        .map_err(|error| AppServiceError::invalid_request(error.to_string()))?;
    let success = i64::from(outcome.total) >= request.dc;
    Ok(FlatCheckRollView {
        dc: request.dc,
        outcome,
        success,
    })
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn reference_checks_use_a_d20_and_compare_to_dc_without_critical_adjustments() {
        for dc in [0, 11, 21, 100] {
            let result = roll(RollFlatCheckRequest { dc }).unwrap();
            assert!((1..=20).contains(&result.outcome.total));
            assert_eq!(result.success, i64::from(result.outcome.total) >= dc);
        }
        assert!(roll(RollFlatCheckRequest { dc: -1 }).is_err());
        assert!(roll(RollFlatCheckRequest { dc: 101 }).is_err());
    }
}
