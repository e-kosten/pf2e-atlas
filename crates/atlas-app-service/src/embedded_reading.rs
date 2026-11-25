//! One checked parent and one attributed prepared batch per bounded reading request.
use crate::projection::relationship_views;
use crate::{AppServiceError, AppServiceResult, AtlasAppService};
use atlas_app_model::*;
use atlas_domain::RecordKey;
use atlas_record::source_content::SourceContentLocator;
use atlas_record::source_record::{
    SourceContentFormat, SourceContentRole, SourceFieldView, SourceNodeView,
};
use atlas_search::GetRecordRequest;

const ENTRY_LIMIT: usize = 32;
const FIELD_LIMIT: usize = 256;

impl AtlasAppService {
    pub fn embedded_reading(
        &self,
        request: EmbeddedReadingRequest,
    ) -> AppServiceResult<EmbeddedReadingBatchView> {
        let key = RecordKey::parse(&request.parent.record_key)
            .map_err(|e| AppServiceError::invalid_request(e.to_string()))?;
        self.with_retrieval(&crate::ReadExecutionContext::uncancellable(), move |r| {
            let generation = r.artifact_generation()?;
            if request.artifact_generation != generation {
                return Err(AppServiceError::artifact_generation_changed(
                    &request.artifact_generation,
                    &generation,
                    atlas_app_model::AppRecoverableAction::ReloadRecord,
                ));
            }
            let parent = RecordNavigationView {
                spell_rank: None,
                record_key: key.to_string(),
                owners: vec![],
                field: None,
                passage: None,
                source_fingerprint: None,
            };
            let detail = r
                .get_record(GetRecordRequest {
                    record_key: &key,
                    selected_content: &[],
                })?
                .ok_or_else(|| {
                    AppServiceError::new(AppErrorCode::RecordNotFound, "parent record unavailable")
                })?;
            let node = detail
                .source
                .node_at(&[])
                .ok_or_else(|| AppServiceError::invalid_request("parent record unavailable"))?;
            let table_reading = matches!(
                &request.selection,
                EmbeddedReadingSelection::TableResults { .. }
            );
            if !(matches!(node, SourceNodeView::Actor(_)) && !table_reading
                || matches!(node, SourceNodeView::Table(_)) && table_reading)
            {
                return Err(AppServiceError::invalid_request(
                    "reading selection does not match the parent record",
                ));
            }
            let actor_content = matches!(request.selection, EmbeddedReadingSelection::ActorContent);
            let availability = if actor_content {
                atlas_domain::QueryFieldState::Value
            } else if let SourceNodeView::Table(table) = node {
                crate::projection::field_state(SourceFieldView::from(&table.results).availability())
            } else {
                crate::projection::field_state(node.actor_items().availability())
            };
            let (page, total, start, single) = match &request.selection {
                EmbeddedReadingSelection::ActorContent => (vec![parent.clone()], 1, 0, true),
                EmbeddedReadingSelection::TableResults { cursor } => {
                    let candidates: Vec<_> = crate::presentation::owned_links(
                        &detail.source,
                        node,
                        &[],
                        r.source_fingerprint(),
                    )
                    .into_iter()
                    .map(|link| link.navigation)
                    .collect();
                    let total = candidates.len();
                    let start = page_offset(cursor.as_ref(), &generation, total)?;
                    let page = candidates
                        .into_iter()
                        .skip(start)
                        .take(ENTRY_LIMIT)
                        .collect();
                    (page, total, start, false)
                }
                EmbeddedReadingSelection::Abilities {
                    group,
                    cursor,
                    target,
                } => {
                    if cursor.is_some() && target.is_some() {
                        return Err(AppServiceError::invalid_request(
                            "select a reading cursor or target, not both",
                        ));
                    }
                    let owners =
                        crate::presentation::actor::ability_owners(&detail.source, node, *group);
                    let total = owners.len();
                    let start = if let Some(target) = target {
                        checked_activity(target, &key, r.source_fingerprint())?;
                        let index = owners
                            .iter()
                            .position(|owner| owner == &target.owner)
                            .ok_or_else(|| {
                                AppServiceError::invalid_request(
                                    "target ability does not exist in this source snapshot",
                                )
                            })?;
                        index / ENTRY_LIMIT * ENTRY_LIMIT
                    } else {
                        page_offset(cursor.as_ref(), &generation, total)?
                    };
                    let page = owners
                        .into_iter()
                        .skip(start)
                        .take(ENTRY_LIMIT)
                        .map(|owner| {
                            crate::definitions::record_navigation(
                                key.clone(),
                                vec![owner],
                                r.source_fingerprint(),
                            )
                        })
                        .collect();
                    (page, total, start, false)
                }
                EmbeddedReadingSelection::Activity { address } => {
                    checked_activity(address, &key, r.source_fingerprint())?;
                    let owners = std::slice::from_ref(&address.owner);
                    let item = match detail.source.node_at(owners) {
                        Some(SourceNodeView::Item(item))
                            if address.owner.collection == "/items" =>
                        {
                            item
                        }
                        _ => {
                            return Err(AppServiceError::invalid_request(
                                "requested activity does not exist in this source snapshot",
                            ));
                        }
                    };
                    let mut navigation = crate::definitions::record_navigation(
                        key.clone(),
                        owners.to_vec(),
                        r.source_fingerprint(),
                    );
                    if let atlas_record::source_record::ItemSourceView::SpellSource(source) = item
                        && let [rank] =
                            crate::presentation::spell_context::ranks(&detail.source, source)
                                .as_slice()
                    {
                        navigation.spell_rank = Some(*rank);
                    }
                    (vec![navigation], 1, 0, true)
                }
            };
            let mut selected = vec![];
            let mut selected_fields = 0;
            for navigation in &page {
                let child = detail.source.node_at(&navigation.owners).ok_or_else(|| {
                    AppServiceError::invalid_request("embedded activity unavailable")
                })?;
                let fields: Vec<_> = child
                    .content_selections()
                    .into_iter()
                    .filter(|s| {
                        s.role != SourceContentRole::Name
                            && (matches!(s.text, SourceFieldView::Value(_))
                                || actor_content
                                    && !matches!(
                                        s.text,
                                        SourceFieldView::Missing | SourceFieldView::NotApplicable
                                    )
                                || s.role == SourceContentRole::Prerequisite)
                    })
                    .collect();
                if selected_fields + fields.len() > FIELD_LIMIT {
                    if selected.is_empty() {
                        return Err(AppServiceError::invalid_request(
                            "embedded activity exceeds 256 selected fields",
                        ));
                    }
                    break;
                }
                selected_fields += fields.len();
                selected.push((navigation, fields));
            }
            let locators: Vec<_> = selected
                .iter()
                .flat_map(|(navigation, fields)| {
                    fields
                        .iter()
                        .filter(|s| {
                            matches!(s.text, SourceFieldView::Value(_))
                                && matches!(
                                    s.format,
                                    SourceFieldView::Value(SourceContentFormat::Html)
                                )
                        })
                        .map(|s| SourceContentLocator {
                            record: key.clone(),
                            owners: navigation.owners.clone(),
                            field: s.field.to_string(),
                        })
                })
                .collect();
            let content = r.read_attributed_content(&locators)?;
            let mut entries = vec![];
            for (navigation, fields) in &selected {
                let table_links = if table_reading {
                    r.read_non_content_relationships(&atlas_search::SourceRelationshipRequest {
                        record: key.clone(),
                        owners: Some(navigation.owners.clone()),
                        fields: vec![],
                        direction: atlas_search::SourceRelationshipDirection::Outgoing,
                        limit: 1024,
                    })?
                    .occurrences
                } else {
                    vec![]
                };
                entries.push(EmbeddedReadingEntryView {
                    navigation: (*navigation).clone(),
                    content: crate::prepared_content::prepared_fields(
                        &key,
                        &navigation.owners,
                        fields,
                        &content.content.fields,
                        &content.bounded,
                        r.artifact_context().audience,
                        &crate::prepared_content::PreparedContentContext::source(
                            r,
                            &detail.source,
                            &navigation.owners,
                            None,
                        )
                        .with_section_rendering(request.include_section_rendering),
                    )?,
                    relationships: relationship_views(
                        content
                            .relationships
                            .iter()
                            .filter(|edge| edge.locator.owners == navigation.owners)
                            .cloned()
                            .chain(table_links)
                            .collect(),
                        r.source_fingerprint(),
                    ),
                });
            }
            crate::inline_content::compose_inline_content(
                r,
                entries
                    .iter_mut()
                    .flat_map(|entry| entry.content.iter_mut()),
                &content.relationships,
            )?;
            let end = start + entries.len();
            let cursor = |offset| EmbeddedReadingCursor {
                artifact_generation: generation.clone(),
                offset,
            };
            Ok(EmbeddedReadingBatchView {
                artifact_generation: generation.clone(),
                parent,
                availability,
                total,
                start,
                end,
                previous: (!single && start > 0).then(|| cursor(start.saturating_sub(ENTRY_LIMIT))),
                next: (!single && end < total).then(|| cursor(end)),
                entries,
            })
        })
    }
}

fn page_offset(
    cursor: Option<&EmbeddedReadingCursor>,
    generation: &str,
    total: usize,
) -> AppServiceResult<usize> {
    let Some(cursor) = cursor else {
        return Ok(0);
    };
    if cursor.artifact_generation != generation {
        return Err(AppServiceError::artifact_generation_changed(
            &cursor.artifact_generation,
            generation,
            AppRecoverableAction::ReloadRecord,
        ));
    }
    if cursor.offset != 0 && cursor.offset >= total {
        return Err(AppServiceError::invalid_request(
            "reading cursor is outside the embedded collection",
        ));
    }
    Ok(cursor.offset)
}

fn checked_activity(
    address: &EmbeddedActivityAddress,
    key: &RecordKey,
    fingerprint: &str,
) -> AppServiceResult<()> {
    crate::records::validate_address_bounds(std::slice::from_ref(&address.owner))?;
    if address.record_key != key.to_string() {
        return Err(AppServiceError::invalid_request(
            "select an immediate activity of this parent",
        ));
    }
    if !crate::records::address_fingerprint_matches(
        std::slice::from_ref(&address.owner),
        address.source_fingerprint.as_deref(),
        fingerprint,
    ) {
        return Err(AppServiceError::invalid_request(
            "snapshot-local activity belongs to a different artifact; reload the parent",
        ));
    }
    Ok(())
}
