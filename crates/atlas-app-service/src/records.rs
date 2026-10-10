use crate::projection::{navigation_fingerprint, relationship_view};
use crate::{AppServiceError, AppServiceResult, AtlasAppService};
use atlas_app_model::{
    AppErrorCode, RecordDetailRequest, RecordDetailView, RecordNavigationView,
    RecordSurfaceProfileView,
};
use atlas_domain::{QueryPredicate, RecordKey, SourcePassageAddress};
use atlas_record::source_content::{OwnedContentIdentity, SourceContentLocator};
use atlas_record::source_record::{
    SourceContentFormat, SourceContentRole, SourceFieldView, recover_source_passage,
};
use atlas_search::{
    GetRecordRequest, GetRecordsRequest, RecordRefResolutionResult, ResolveRecordRefRequest,
    ResolveRecordRequest, SourceRelationshipDirection, SourceRelationshipRequest,
};

impl AtlasAppService {
    pub fn get_records(
        &self,
        record_keys: Vec<RecordKey>,
    ) -> AppServiceResult<Vec<atlas_app_model::RecordSummaryView>> {
        self.submit_retrieval(move |r| {
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
        self.submit_retrieval(move |r| {
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
        self.submit_retrieval(move |r| {
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
        validate_request(&request)?;
        let key = RecordKey::parse(&request.record_key)
            .map_err(|e| AppServiceError::new(AppErrorCode::InvalidRecordKey, e.to_string()))?;
        self.submit_retrieval(move |r| {
            if request.owners.iter().any(|o|matches!(o.identity,OwnedContentIdentity::SnapshotLocal{..})) && request.source_fingerprint.as_deref()!=Some(r.source_fingerprint()) {
                return Err(AppServiceError::invalid_request("snapshot-local navigation belongs to a different artifact; reopen the search result or parent record"));
            }
            let detail=r.get_record(GetRecordRequest{record_key:&key,selected_content:&[]})?.ok_or_else(||AppServiceError::new(AppErrorCode::RecordNotFound,format!("record `{key}` was not found")))?;
            let node=detail.source.node_at(&request.owners).ok_or_else(||AppServiceError::invalid_request("selected owner does not exist in this source snapshot"))?;
            let selections=node.content_selections();
            if request.fields.iter().any(|f|!selections.iter().any(|s|s.field==f)) {
                return Err(AppServiceError::invalid_request("selected field is not declared content of the selected owner"));
            }
            let selected:Vec<_>=selections.into_iter().filter(|s|if request.fields.is_empty(){s.role!=SourceContentRole::Name && matches!(s.text,SourceFieldView::Value(_))}else{request.fields.iter().any(|f|f==s.field)}).collect();
            if selected.len()>256 {return Err(AppServiceError::invalid_request("select at most 256 content fields per detail request"));}
            let locators:Vec<_>=selected.iter().filter(|s|matches!(s.format,SourceFieldView::Value(SourceContentFormat::Html))).map(|s|SourceContentLocator{record:key.clone(),owners:request.owners.clone(),field:s.field.into()}).collect();
            let content=r.read_content(&locators)?;
            let fields=crate::surfaces::prepared_fields(&key,&request.owners,&selected,&content.fields,r.artifact_context().audience,r.source_fingerprint())?;
            if let Some(address)=&request.passage {
                if matches!(address,SourcePassageAddress::Identity{}) {
                    if !request.fields.is_empty(){return Err(AppServiceError::invalid_request("identity navigation cannot select a prose field"));}
                }else{
                    if request.fields.len()!=1{return Err(AppServiceError::invalid_request("passage navigation requires exactly one selected field"));}
                    let field=fields.iter().find(|f|f.locator.field==request.fields[0]).ok_or_else(||AppServiceError::invalid_request("selected passage field is unavailable"))?;
                    let(html,plain)=match &field.body {
                        atlas_app_model::PreparedFieldBodyView::Html{html,..}=>(Some(html.as_str()),None),
                        atlas_app_model::PreparedFieldBodyView::Plain{text}=>(None,Some(text.as_str())),
                        _=>return Err(AppServiceError::invalid_request("selected passage field is unavailable")),
                    };
                    recover_source_passage(html,plain,address).map_err(|e|AppServiceError::invalid_request(format!("stale or invalid passage: {e}")))?;
                }
            }
            let summary=crate::projection::localized_summary_view(&detail.summary,r);
            let surface=crate::surfaces::record_surface(&detail.source,node,&summary,&request.owners,fields,RecordSurfaceProfileView::RecordDetail,r);
            let relationships=r.read_relationships(&SourceRelationshipRequest{record:key.clone(),owners:Some(request.owners.clone()),field:if request.fields.len()==1{Some(request.fields[0].clone())}else{None},direction:SourceRelationshipDirection::Outgoing,limit:1024})?;
            Ok(RecordDetailView{record:summary,surface,selected:RecordNavigationView{record_key:key.to_string(),source_fingerprint:navigation_fingerprint(&request.owners,r.source_fingerprint()),owners:request.owners,field:if request.fields.len()==1{Some(request.fields[0].clone())}else{None},passage:request.passage},relationships:relationships.occurrences.into_iter().map(|o|relationship_view(o,r.source_fingerprint())).collect(),relationships_truncated:relationships.truncated})
        })
    }
}
fn validate_request(r: &RecordDetailRequest) -> AppServiceResult<()> {
    if r.owners.len() > 32
        || r.fields.len() > 256
        || r.fields.iter().any(|f| f.len() > 512)
        || r.owners.iter().any(|o| {
            o.collection.len() > 512
                || matches!(&o.identity,OwnedContentIdentity::Stable(id) if id.len()>512)
        })
    {
        return Err(AppServiceError::invalid_request(
            "detail selection exceeds its owner or field bounds",
        ));
    }
    Ok(())
}
