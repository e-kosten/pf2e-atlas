//! Navigation to declared recipient grants; never applies an effect to an actor.
use crate::AppServiceResult;
use atlas_app_model::{
    PreparedFieldBodyView, RecordBodyView, RecordDetailView, RecordGrantLinkView, RecordGrantsView,
    RecordNavigationView,
};
use atlas_domain::RecordKey;
use atlas_record::source_record::{SourceBackedRecord, SourceNodeView};
use atlas_search::{AtlasRetrievalService, GetRecordsRequest, SourceUnitLocation};

fn same_document(left: &RecordNavigationView, right: &RecordNavigationView) -> bool {
    left.record_key == right.record_key && left.owners == right.owners
}

pub(crate) fn attach(
    source: &SourceBackedRecord,
    detail: &mut RecordDetailView,
    retrieval: &AtlasRetrievalService,
    truncated: bool,
) -> AppServiceResult<()> {
    // Actor-owned abilities describe the actor's composition, not an external recipient.
    if !matches!(source.node_at(&[]), Some(SourceNodeView::Item(_)))
        || !matches!(
            source.node_at(&detail.selected.owners),
            Some(SourceNodeView::Item(_))
        )
    {
        return Ok(());
    }
    let mut shown: Vec<_> = detail
        .relationships
        .iter()
        .filter(|reference| reference.origin == "content")
        .filter_map(|reference| reference.target.as_ref())
        .collect();
    for field in &detail.presentation.content {
        if let PreparedFieldBodyView::Html { inclusions, .. } = &field.body {
            shown.extend(inclusions.iter().flat_map(|inclusion| {
                inclusion
                    .references
                    .iter()
                    .filter_map(|reference| reference.target.as_ref())
            }));
        }
    }
    shown.extend(
        detail
            .presentation
            .owned
            .iter()
            .map(|entry| &entry.navigation),
    );
    let features = match &detail.presentation.body {
        RecordBodyView::Ancestry(body) => body.grants.value.as_ref(),
        RecordBodyView::Class(body) => body.grants.value.as_ref(),
        RecordBodyView::Deity(body) => body.spells.value.as_ref(),
        _ => None,
    };
    shown.extend(
        features
            .into_iter()
            .flatten()
            .filter_map(|entry| entry.navigation.as_ref()),
    );

    let mut destinations = Vec::new();
    let mut unavailable = std::collections::BTreeSet::new();
    for reference in detail
        .relationships
        .iter()
        .filter(|reference| reference.kind == "DeclaredGrant")
    {
        let Some(target) = &reference.target else {
            // Keep missing references visible without making up a destination or display name.
            unavailable.insert(reference.authored_target.clone());
            continue;
        };
        if !same_document(target, &detail.selected)
            && !shown.iter().any(|entry| same_document(entry, target))
            && !destinations
                .iter()
                .any(|entry| same_document(entry, target))
        {
            destinations.push(target.clone());
        }
    }
    if destinations.is_empty() && unavailable.is_empty() && !truncated {
        return Ok(());
    }
    let keys = destinations
        .iter()
        .map(|nav| RecordKey::parse(&nav.record_key))
        .collect::<Result<std::collections::BTreeSet<_>, _>>()
        .expect("resolved relationship keys are canonical")
        .into_iter()
        .collect::<Vec<_>>();
    let summaries = retrieval
        .get_records(GetRecordsRequest { record_keys: &keys })?
        .records;
    let locations = destinations
        .iter()
        .filter(|nav| !nav.owners.is_empty())
        .map(|nav| SourceUnitLocation {
            unit_id: 0,
            record: RecordKey::parse(&nav.record_key).expect("resolved relationship key"),
            owners: nav.owners.clone(),
            field: None,
            address: None,
        })
        .collect::<Vec<_>>();
    let mut labels = Vec::new();
    for batch in locations.chunks(256) {
        labels.extend(retrieval.owned_search_labels(batch)?);
    }
    let mut unavailable_count = unavailable.len();
    let entries = destinations
        .into_iter()
        .filter_map(|navigation| {
            let record = if navigation.owners.is_empty() {
                let Some(summary) = summaries
                    .iter()
                    .find(|record| record.key.to_string() == navigation.record_key)
                else {
                    unavailable_count += 1;
                    return None;
                };
                Some(crate::projection::localized_summary_view(
                    summary, retrieval,
                ))
            } else {
                None
            };
            let title = record
                .as_ref()
                .map(|record| record.title.clone())
                .or_else(|| {
                    labels
                        .iter()
                        .find(|(key, owners, _)| {
                            key.to_string() == navigation.record_key && owners == &navigation.owners
                        })
                        .map(|(_, _, title)| title.clone())
                })
                .unwrap_or_else(|| "Unnamed entry".into());
            Some(RecordGrantLinkView {
                title,
                record,
                navigation,
            })
        })
        .collect();
    detail.presentation.grants = Some(RecordGrantsView {
        entries,
        unavailable_count,
        truncated,
    });
    Ok(())
}
