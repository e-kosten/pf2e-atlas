use crate::projection::{navigation_fingerprint, relationship_views};
use crate::{AppServiceError, AppServiceResult, AtlasAppService};
use atlas_app_model::{AppErrorCode, RecordDetailRequest, RecordDetailView, RecordNavigationView};
use atlas_domain::OwnedContentIdentity;
use atlas_domain::{QueryPredicate, RecordKey, SourcePassageAddress};
use atlas_record::source_content::SourceContentLocator;
use atlas_record::source_record::{
    SourceContentFormat, SourceContentRole, SourceFieldView, recover_source_passage,
};
use atlas_search::{
    GetRecordRequest, GetRecordsRequest, RecordRefResolutionResult, ResolveRecordRefRequest,
    ResolveRecordRequest, SourceRelationshipDirection, SourceRelationshipRequest,
};

impl AtlasAppService {
    /// Explicit source inspection bypasses presentation, using the checked codec.
    /// Product record eligibility still applies; tooling sources stay developer-only.
    pub fn record_source(&self, key: &str) -> AppServiceResult<serde_json::Value> {
        let key = RecordKey::parse(key).map_err(|error| {
            AppServiceError::new(AppErrorCode::InvalidRecordKey, error.to_string())
        })?;
        self.with_retrieval(&crate::ReadExecutionContext::uncancellable(), move |retrieval| {
            let detail = retrieval.get_record(GetRecordRequest {
                record_key: &key,
                selected_content: &[],
            })?.ok_or_else(|| AppServiceError::new(AppErrorCode::RecordNotFound, format!("record `{key}` was not found")))?;
            let snapshot = atlas_foundry_model::snapshot_json_value(detail.source.source())
                .map_err(|error| AppServiceError::new(AppErrorCode::InternalError, error.to_string()))?;
            let mut output = serde_json::json!({
                "key": key,
                "provenance": { "path": detail.summary.source_path, "sha256": detail.summary.content_hash }
            });
            output["checked_snapshot"] = snapshot;
            Ok(output)
        })
    }

    pub fn get_records(
        &self,
        record_keys: Vec<RecordKey>,
    ) -> AppServiceResult<Vec<atlas_app_model::RecordSummaryView>> {
        self.with_retrieval(&crate::ReadExecutionContext::uncancellable(), move |r| {
            Ok(r.get_records(GetRecordsRequest {
                record_keys: &record_keys,
            })?
            .records
            .iter()
            .map(|v| crate::projection::localized_summary_view(v, r))
            .collect())
        })
    }
    pub fn resolve_record(
        &self,
        query: String,
        filter: Option<QueryPredicate>,
    ) -> AppServiceResult<Vec<atlas_app_model::RecordResolutionCandidateView>> {
        self.with_retrieval(&crate::ReadExecutionContext::uncancellable(), move |r| {
            Ok(r.resolve_record(ResolveRecordRequest {
                query: &query,
                filter: filter.as_ref(),
            })?
            .into_iter()
            .map(|v| crate::projection::resolution_view(v, r))
            .collect())
        })
    }
    pub fn resolve_record_ref(
        &self,
        record_ref: String,
        filter: Option<QueryPredicate>,
    ) -> AppServiceResult<atlas_app_model::RecordRefResolutionView> {
        self.with_retrieval(&crate::ReadExecutionContext::uncancellable(), move |r| {
            Ok(
                match r.resolve_record_ref(ResolveRecordRefRequest {
                    record_ref: &record_ref,
                    filter: filter.as_ref(),
                })? {
                    RecordRefResolutionResult::Key(k) => {
                        atlas_app_model::RecordRefResolutionView::Key(k)
                    }
                    RecordRefResolutionResult::Miss => {
                        atlas_app_model::RecordRefResolutionView::Miss
                    }
                    RecordRefResolutionResult::Ambiguous(v) => {
                        atlas_app_model::RecordRefResolutionView::Ambiguous(
                            v.into_iter()
                                .map(|v| crate::projection::resolution_view(v, r))
                                .collect(),
                        )
                    }
                },
            )
        })
    }
    pub fn record_detail(&self, key: &str) -> AppServiceResult<RecordDetailView> {
        self.record_detail_at(RecordDetailRequest {
            spell_rank: None,
            record_key: key.into(),
            owners: vec![],
            fields: vec![],
            passage: None,
            source_fingerprint: None,
        })
    }
    pub fn record_detail_at(
        &self,
        request: RecordDetailRequest,
    ) -> AppServiceResult<RecordDetailView> {
        self.with_retrieval(&crate::ReadExecutionContext::uncancellable(), move |r| {
            read_record_detail(r, request).map(|(_, view)| view)
        })
    }
}

/// Decode the selected parent once, retaining it for workflows that compose runtime facts.
pub(crate) fn read_record_detail(
    r: &atlas_search::AtlasRetrievalService,
    request: RecordDetailRequest,
) -> AppServiceResult<(atlas_search::SourceRecordDetail, RecordDetailView)> {
    validate_request(&request)?;
    let whole_document = request.fields.is_empty() && request.passage.is_none();
    let key = RecordKey::parse(&request.record_key)
        .map_err(|e| AppServiceError::new(AppErrorCode::InvalidRecordKey, e.to_string()))?;
    if request
        .owners
        .iter()
        .any(|o| matches!(o.identity, OwnedContentIdentity::SnapshotLocal { .. }))
        && request.source_fingerprint.as_deref() != Some(r.source_fingerprint())
    {
        return Err(AppServiceError::invalid_request(
            "snapshot-local navigation belongs to a different artifact; reopen the search result or parent record",
        ));
    }
    let detail = r
        .get_record(GetRecordRequest {
            record_key: &key,
            selected_content: &[],
        })?
        .ok_or_else(|| {
            AppServiceError::new(
                AppErrorCode::RecordNotFound,
                format!("record `{key}` was not found"),
            )
        })?;
    let node = detail.source.node_at(&request.owners).ok_or_else(|| {
        AppServiceError::invalid_request("selected document does not exist in this source snapshot")
    })?;
    let selections = node.content_selections();
    if request
        .fields
        .iter()
        .any(|f| !selections.iter().any(|s| s.field == f.as_str()))
    {
        return Err(AppServiceError::invalid_request(
            "selected field is not declared content of the selected document",
        ));
    }
    let selected: Vec<_> = selections
        .into_iter()
        .filter(|s| {
            if request.fields.is_empty() {
                s.role != SourceContentRole::Name
                    && (matches!(s.text, SourceFieldView::Value(_))
                        || s.role == SourceContentRole::Prerequisite)
            } else {
                request.fields.iter().any(|f| f.as_str() == s.field)
            }
        })
        .collect();
    if selected.len() > 256 {
        return Err(AppServiceError::invalid_request(
            "select at most 256 content fields per detail request",
        ));
    }
    let locators: Vec<_> = selected
        .iter()
        .filter(|s| {
            matches!(s.text, SourceFieldView::Value(_))
                && matches!(s.format, SourceFieldView::Value(SourceContentFormat::Html))
        })
        .map(|s| SourceContentLocator {
            record: key.clone(),
            owners: request.owners.clone(),
            field: s.field.to_string(),
        })
        .collect();
    let content = r.read_attributed_content(&locators)?;
    let mut fields = crate::prepared_content::prepared_fields(
        &key,
        &request.owners,
        &selected,
        &content.content.fields,
        &content.bounded,
        r.artifact_context().audience,
        r.source_fingerprint(),
    )?;
    if let Some(address) = &request.passage {
        if matches!(address, SourcePassageAddress::Identity {}) {
            if !request.fields.is_empty() {
                return Err(AppServiceError::invalid_request(
                    "identity navigation cannot select a prose field",
                ));
            }
        } else {
            if request.fields.len() != 1 {
                return Err(AppServiceError::invalid_request(
                    "passage navigation requires exactly one selected field",
                ));
            }
            let field = fields
                .iter()
                .find(|f| f.locator.field == request.fields[0])
                .ok_or_else(|| {
                    AppServiceError::invalid_request("selected passage field is unavailable")
                })?;
            let (html, plain) = match &field.body {
                atlas_app_model::PreparedFieldBodyView::Html { html, .. } => {
                    (Some(html.as_str()), None)
                }
                atlas_app_model::PreparedFieldBodyView::Plain { text } => {
                    (None, Some(text.as_str()))
                }
                _ => {
                    return Err(AppServiceError::invalid_request(
                        "selected passage field is unavailable",
                    ));
                }
            };
            recover_source_passage(html, plain, address).map_err(|e| {
                AppServiceError::invalid_request(format!("stale or invalid passage: {e}"))
            })?;
        }
    }
    crate::inline_content::compose_inline_content(r, fields.iter_mut(), &content.relationships)?;
    let summary = crate::projection::localized_summary_view(&detail.summary, r);
    let mut presentation = crate::presentation::record_presentation(
        &detail.source,
        node,
        &summary,
        &request.owners,
        fields,
        r,
    )?;
    let spell_rank = crate::presentation::apply_casting_rank(
        &detail.source,
        node,
        request.spell_rank,
        &mut presentation,
        r,
    )?;
    let relationships = r.read_non_content_relationships(&SourceRelationshipRequest {
        record: key.clone(),
        owners: Some(request.owners.clone()),
        field: if request.fields.len() == 1 {
            Some(request.fields[0].clone())
        } else {
            None
        },
        direction: SourceRelationshipDirection::Outgoing,
        limit: 1024,
    })?;
    let grants_truncated = relationships.truncated;
    let mut view = RecordDetailView {
        encounter_adjustment: request
            .owners
            .is_empty()
            .then(|| {
                crate::encounters::mechanics::initial_variant(&detail.source)
                    .0
                    .map(|variant| match variant {
                        atlas_local_state::ParticipantVariant::Normal => {
                            atlas_app_model::EncounterParticipantVariantView::Normal
                        }
                        atlas_local_state::ParticipantVariant::Elite => {
                            atlas_app_model::EncounterParticipantVariantView::Elite
                        }
                        atlas_local_state::ParticipantVariant::Weak => {
                            atlas_app_model::EncounterParticipantVariantView::Weak
                        }
                    })
            })
            .flatten(),
        artifact_generation: r.artifact_generation()?,
        record: summary,
        presentation,
        selected: RecordNavigationView {
            spell_rank,
            record_key: key.to_string(),
            source_fingerprint: navigation_fingerprint(&request.owners, r.source_fingerprint()),
            owners: request.owners,
            field: if request.fields.len() == 1 {
                Some(request.fields[0].clone())
            } else {
                None
            },
            passage: request.passage,
        },
        relationships: relationship_views(
            content
                .relationships
                .into_iter()
                .chain(relationships.occurrences)
                .collect(),
            r.source_fingerprint(),
        )
        .into_iter()
        .map(|mut relationship| {
            if relationship.kind == "CompendiumProvenance"
                && relationship.status == "resolved_record"
                && let Some(target) = &mut relationship.target
            {
                target.spell_rank = spell_rank;
            }
            relationship
        })
        .collect(),
        relationships_truncated: relationships.truncated || !content.bounded.is_empty(),
    };
    crate::contextual_references::attach_consumable_spell(&mut view, r)?;
    crate::contextual_references::attach_ranked_effects(&mut view, r)?;
    if whole_document {
        crate::record_grants::attach(&detail.source, &mut view, r, grants_truncated)?;
    }
    Ok((detail, view))
}
pub(crate) fn validate_request(r: &RecordDetailRequest) -> AppServiceResult<()> {
    if r.owners.len() > 32
        || r.fields.len() > 256
        || r.fields.iter().any(|f| f.len() > 512)
        || r.owners.iter().any(|o| {
            o.collection.len() > 512
                || matches!(&o.identity,OwnedContentIdentity::Stable(id) if id.len()>512)
        })
    {
        return Err(AppServiceError::invalid_request(
            "detail selection exceeds its embedded path or field bounds",
        ));
    }
    Ok(())
}
