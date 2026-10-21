//! Product order defaults and body-free display of the compared unit-price fact.
use crate::{AppServiceError, AppServiceResult};
use atlas_app_model::{RecordSummaryView, SearchCriteria};
use atlas_domain::{RecordKey, ResultSort};

pub(crate) fn search_sort(
    requested: Option<ResultSort>,
    criteria: &SearchCriteria,
) -> AppServiceResult<ResultSort> {
    let has_matching = !matches!(criteria, SearchCriteria::ListRecords { .. });
    let sort = requested.unwrap_or(if has_matching {
        ResultSort::Relevance
    } else {
        ResultSort::NameAsc
    });
    if sort == ResultSort::SavedOrder || (sort == ResultSort::Relevance && !has_matching) {
        return Err(AppServiceError::invalid_request(
            "Choose an attribute sort for browsing; Relevance requires text or Similar",
        ));
    }
    Ok(sort)
}
pub(crate) fn saved_list_sort(requested: Option<ResultSort>) -> AppServiceResult<ResultSort> {
    let sort = requested.unwrap_or(ResultSort::SavedOrder);
    if sort == ResultSort::Relevance {
        return Err(AppServiceError::invalid_request(
            "Choose Saved order or an attribute sort for a saved list",
        ));
    }
    Ok(sort)
}
pub(crate) fn price_labels(
    retrieval: &atlas_search::AtlasRetrievalService,
    records: &mut [RecordSummaryView],
) -> AppServiceResult<()> {
    let keys = records
        .iter()
        .map(|r| {
            RecordKey::parse(&r.record_key)
                .map_err(|_| AppServiceError::invalid_request("The result record is unavailable"))
        })
        .collect::<AppServiceResult<Vec<_>>>()?;
    let facts = retrieval.read_unit_prices(&keys)?;
    for record in records {
        record.unit_price_label = facts
            .iter()
            .find(|p| p.key.to_string() == record.record_key)
            .and_then(|p| p.copper.as_value())
            .map(|v| format!("{} gp each", gold_label(*v)));
    }
    Ok(())
}
/// Shift the stored copper's shortest decimal representation, avoiding another rounded numeric conversion.
fn gold_label(copper: f64) -> String {
    let raw = copper.to_string();
    let (sign, raw) = raw
        .strip_prefix('-')
        .map(|s| ("-", s))
        .unwrap_or(("", raw.as_str()));
    let (whole, fraction) = raw.split_once('.').unwrap_or((raw, ""));
    let digits = format!("{whole}{fraction}");
    let point = whole.len() as isize - 2;
    let mut result = if point <= 0 {
        format!("0.{}{}", "0".repeat((-point) as usize), digits)
    } else {
        let point = point as usize;
        format!("{}.{}", &digits[..point], &digits[point..])
    };
    while result.ends_with('0') {
        result.pop();
    }
    if result.ends_with('.') {
        result.pop();
    }
    format!("{sign}{result}")
}
#[cfg(test)]
mod tests {
    use super::gold_label;
    #[test]
    fn unit_price_labels_preserve_zero_fraction_and_adjacent_large_copper() {
        assert_eq!(gold_label(0.0), "0");
        assert_eq!(gold_label(0.5), "0.005");
        assert_eq!(gold_label(100.0), "1");
        assert_eq!(gold_label(150.0), "1.5");
        assert_eq!(gold_label(9_007_199_254_740_991.0), "90071992547409.91");
        assert_eq!(gold_label(9_007_199_254_740_992.0), "90071992547409.92");
    }
}
