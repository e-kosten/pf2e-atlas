use atlas_app_model::{
    RecordBadgeView, RecordDetailView, RecordSummaryView, ResultMatchSummary, SearchPageView,
};
use atlas_record::{AtlasRecord, build_record_presentation_document};
use atlas_search::SearchPageInfo;

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

pub(crate) fn record_summary(record: &AtlasRecord) -> RecordSummaryView {
    RecordSummaryView {
        record_key: record.identity.key.to_string(),
        title: record.identity.name.clone(),
        kind: record.classification.kind.as_str().to_string(),
        kind_label: kind_label(record.classification.kind.as_str()),
        level_label: record.classification.level.map(|level| level.to_string()),
        rarity: record
            .classification
            .rarity
            .as_ref()
            .map(|rarity| rarity.as_str().to_string()),
        traits: record
            .classification
            .traits
            .iter()
            .map(|value| RecordBadgeView {
                kind: "trait".to_string(),
                label: value.clone(),
                value: value.clone(),
            })
            .collect(),
        taxonomy: record
            .classification
            .taxonomy
            .inferred_groups
            .iter()
            .map(|value| RecordBadgeView {
                kind: "taxonomy".to_string(),
                label: value.clone(),
                value: value.clone(),
            })
            .collect(),
        publication: record.publication.title.clone(),
        pack_label: Some(record.foundry.pack_label.clone()),
        preview: None,
    }
}

pub(crate) fn record_detail(record: &AtlasRecord) -> AppServiceResult<RecordDetailView> {
    let presentation = build_record_presentation_document(record);
    Ok(RecordDetailView {
        record_key: record.identity.key.to_string(),
        title: record.identity.name.clone(),
        kind: record.classification.kind.as_str().to_string(),
        presentation,
    })
}

pub(crate) fn text_match_summary(label: impl Into<String>) -> ResultMatchSummary {
    ResultMatchSummary {
        label: label.into(),
    }
}

fn kind_label(value: &str) -> String {
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
