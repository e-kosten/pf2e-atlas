//! One product projection for CLI remaster links and the existing Other editions surface.
use crate::{AppServiceResult, projection};
use atlas_app_model::{
    RecordNavigationView, RemasterLinkView, RemasterLinksView, RemasterProvenanceView,
};
use atlas_search::{AtlasRetrievalService, GetRecordRequest, RemasterLinksResult};
use std::collections::{BTreeMap, btree_map::Entry};

pub(crate) fn links_view(
    result: RemasterLinksResult,
    retrieval: &AtlasRetrievalService,
) -> AppServiceResult<RemasterLinksView> {
    let mut sources = BTreeMap::new();
    let mut links = Vec::with_capacity(result.links.len());
    for link in result.links {
        let locator = link.provenance;
        let source = match sources.entry(locator.record.clone()) {
            Entry::Occupied(entry) => entry.into_mut(),
            Entry::Vacant(entry) => entry.insert(retrieval.get_record(GetRecordRequest {
                record_key: &locator.record,
                selected_content: &[],
            })?),
        }
        .as_ref();
        let attribution = source
            .map(|s| {
                let root = projection::localized_summary_view(&s.summary, retrieval).title;
                let owner = if locator.owners.is_empty() {
                    None
                } else {
                    s.source
                        .node_at(&locator.owners)
                        .and_then(|n| n.name().value())
                };
                match owner {
                    Some(name) => format!("{root} — {name}"),
                    None => root,
                }
            })
            .unwrap_or_else(|| "Source unavailable".into());
        let available =
            source.is_some_and(|s| s.source.authored_content_at(&locator).value().is_some());
        let navigation = available.then(|| RecordNavigationView {
            spell_rank: None,
            record_key: locator.record.to_string(),
            owners: locator.owners.clone(),
            field: Some(locator.field.clone()),
            passage: None,
            source_fingerprint: projection::navigation_fingerprint(
                &locator.owners,
                retrieval.source_fingerprint(),
            ),
        });
        links.push(RemasterLinkView {
            legacy_record: projection::localized_summary_view(&link.legacy_record, retrieval),
            remaster_record: projection::localized_summary_view(&link.remaster_record, retrieval),
            provenance: RemasterProvenanceView {
                source: projection::content_location(locator),
                attribution,
                navigation,
                unavailable_reason: (!available)
                    .then(|| "Source field is unavailable in the current collection.".into()),
            },
        });
    }
    Ok(RemasterLinksView {
        seed: projection::localized_summary_view(&result.seed, retrieval),
        links,
    })
}

/// Counterpart failures are secondary availability, not a failed document read.
pub(crate) fn counterparts_view(
    key: &atlas_domain::RecordKey,
    retrieval: &AtlasRetrievalService,
) -> atlas_app_model::RelatedCategoryView<RemasterLinksView> {
    use atlas_app_model::RelatedCategoryView;
    let result = retrieval
        .remaster_links(atlas_search::RemasterLinksRequest { record_key: key })
        .map_err(crate::AppServiceError::from)
        .and_then(|result| match result {
            Some(result) if !result.links.is_empty() => Ok(RelatedCategoryView::Content {
                content: links_view(result, retrieval)?,
            }),
            Some(_) => Ok(RelatedCategoryView::Empty),
            None => Err(crate::AppServiceError::new(
                atlas_app_model::AppErrorCode::RecordNotFound,
                "edition seed not found",
            )),
        });
    result.unwrap_or_else(|error| RelatedCategoryView::Error {
        error: error.into_app_error(),
    })
}
