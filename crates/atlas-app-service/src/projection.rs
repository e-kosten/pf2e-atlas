use atlas_app_model::{
    RecordDetailView, RecordSummaryView, RecordSurfaceProfileView, ResultMatchSummary,
    SearchPageView,
};
use atlas_record::RetrievedRecord;
use atlas_search::{RemasterLinksResult, SearchPageInfo};

use crate::AppServiceResult;

pub(crate) fn search_page_view(page: SearchPageInfo) -> SearchPageView {
    SearchPageView {
        number: page.number,
        size: page.size,
        count: page.count,
        total: page.total,
        has_more: page.has_more,
        next_page: page.next_page,
    }
}

pub(crate) fn record_summary(
    record: &RetrievedRecord,
    remaster_links: Option<&RemasterLinksResult>,
) -> RecordSummaryView {
    RecordSummaryView {
        surface: crate::surface::record_surface(
            record,
            RecordSurfaceProfileView::SearchCompact,
            None,
            remaster_links,
        ),
    }
}

pub(crate) fn record_detail(
    record: &RetrievedRecord,
    remaster_links: Option<&RemasterLinksResult>,
) -> AppServiceResult<RecordDetailView> {
    Ok(RecordDetailView {
        surface: crate::surface::record_surface(
            record,
            RecordSurfaceProfileView::RecordDetail,
            None,
            remaster_links,
        ),
    })
}

pub(crate) fn text_match_summary(label: impl Into<String>) -> ResultMatchSummary {
    ResultMatchSummary {
        label: label.into(),
    }
}

pub(crate) fn kind_label(value: &str) -> String {
    value
        .split('_')
        .map(|part| {
            let mut chars = part.chars();
            match chars.next() {
                Some(first) => first.to_uppercase().chain(chars).collect::<String>(),
                None => String::new(),
            }
        })
        .collect::<Vec<_>>()
        .join(" ")
}
