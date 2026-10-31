//! One checked parent and one attributed prepared batch per bounded reading request.
use crate::projection::relationship_views;
use crate::{AppServiceError, AppServiceResult, AtlasAppService};
use atlas_app_model::*;
use atlas_domain::OwnedContentIdentity;
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
        validate_navigation(&request.parent)?;
        if !request.parent.owners.is_empty()
            || request.parent.field.is_some()
            || request.parent.passage.is_some()
        {
            return Err(AppServiceError::invalid_request(
                "embedded reading requires the parent record address",
            ));
        }
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
            let activities = crate::presentation::actor::authored_activities(
                &detail.source,
                node,
                r.source_fingerprint(),
            );
            let actor_content = matches!(request.selection, EmbeddedReadingSelection::ActorContent);
            let availability = if actor_content {
                atlas_domain::QueryFieldState::Value
            } else if let SourceNodeView::Table(table) = node {
                crate::projection::field_state(SourceFieldView::from(&table.results).availability())
            } else {
                activities.state
            };
            let (candidates, start, single) = match &request.selection {
                EmbeddedReadingSelection::ActorContent => (vec![request.parent.clone()], 0, true),
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
                    let start = if let Some(cursor) = cursor {
                        if cursor.artifact_generation != generation {
                            return Err(AppServiceError::artifact_generation_changed(
                                &cursor.artifact_generation,
                                &generation,
                                AppRecoverableAction::ReloadRecord,
                            ));
                        }
                        cursor.offset
                    } else {
                        0
                    };
                    if start > candidates.len() || (start == candidates.len() && start != 0) {
                        return Err(AppServiceError::invalid_request(
                            "reading cursor is outside the embedded collection",
                        ));
                    }
                    (candidates, start, false)
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
                    let mut selected: Vec<_> = activities
                        .value
                        .unwrap_or_default()
                        .into_iter()
                        .filter(|a| {
                            a.kind == ActorActivityKindView::Ability
                                && group.is_none_or(|group| a.ability_group == Some(group))
                        })
                        .collect();
                    if group.is_some() {
                        selected.sort_by_key(|a| a.reading_order);
                    }
                    let candidates: Vec<_> = selected.into_iter().map(|a| a.navigation).collect();
                    let start = if let Some(target) = target {
                        checked_activity(target, &key, r.source_fingerprint())?;
                        let index = candidates
                            .iter()
                            .position(|a| a.owners == target.owners)
                            .ok_or_else(|| {
                                AppServiceError::invalid_request(
                                    "target ability does not exist in this source snapshot",
                                )
                            })?;
                        index / ENTRY_LIMIT * ENTRY_LIMIT
                    } else if let Some(cursor) = cursor {
                        if cursor.artifact_generation != generation {
                            return Err(AppServiceError::artifact_generation_changed(
                                &cursor.artifact_generation,
                                &generation,
                                atlas_app_model::AppRecoverableAction::ReloadRecord,
                            ));
                        }
                        cursor.offset
                    } else {
                        0
                    };
                    if start > candidates.len() || (start == candidates.len() && start != 0) {
                        return Err(AppServiceError::invalid_request(
                            "reading cursor is outside the embedded collection",
                        ));
                    }
                    (candidates, start, false)
                }
                EmbeddedReadingSelection::Activity { navigation } => {
                    checked_activity(navigation, &key, r.source_fingerprint())?;
                    let activity = activities
                        .value
                        .unwrap_or_default()
                        .into_iter()
                        .find(|a| a.navigation.owners == navigation.owners)
                        .ok_or_else(|| {
                            AppServiceError::invalid_request(
                                "requested activity does not exist in this source snapshot",
                            )
                        })?;
                    (vec![activity.navigation], 0, true)
                }
            };
            let total = candidates.len();
            let mut selected = vec![];
            let mut selected_fields = 0;
            for navigation in
                candidates
                    .iter()
                    .skip(start)
                    .take(if single { 1 } else { ENTRY_LIMIT })
            {
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
                        field: None,
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
                        ),
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
                parent: RecordNavigationView {
                    spell_rank: None,
                    record_key: key.to_string(),
                    owners: vec![],
                    field: None,
                    passage: None,
                    source_fingerprint: None,
                },
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

fn validate_navigation(navigation: &RecordNavigationView) -> AppServiceResult<()> {
    crate::records::validate_request(&RecordDetailRequest {
        spell_rank: None,
        record_key: navigation.record_key.clone(),
        owners: navigation.owners.clone(),
        fields: navigation.field.iter().cloned().collect(),
        passage: navigation.passage.clone(),
        source_fingerprint: navigation.source_fingerprint.clone(),
    })
}
fn checked_activity(
    navigation: &RecordNavigationView,
    key: &RecordKey,
    fingerprint: &str,
) -> AppServiceResult<()> {
    validate_navigation(navigation)?;
    if navigation.record_key != key.to_string() || navigation.owners.len() != 1 {
        return Err(AppServiceError::invalid_request(
            "select an immediate activity of this parent",
        ));
    }
    if navigation
        .owners
        .iter()
        .any(|o| matches!(o.identity, OwnedContentIdentity::SnapshotLocal { .. }))
        && navigation.source_fingerprint.as_deref() != Some(fingerprint)
    {
        return Err(AppServiceError::invalid_request(
            "snapshot-local activity belongs to a different artifact; reload the parent",
        ));
    }
    // Field/passage selection is separate from the activity anchor and cannot change its owner.
    if navigation.field.is_some() || navigation.passage.is_some() {
        return Err(AppServiceError::invalid_request(
            "activity reading selects its complete authored fields",
        ));
    }
    Ok(())
}
