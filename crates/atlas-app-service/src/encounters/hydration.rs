use crate::{AppServiceError, AppServiceResult, AtlasAppService};
use atlas_app_model::AppErrorCode;
use atlas_domain::RecordKey;
use atlas_local_state::EncounterParticipant;
use atlas_search::{
    GetRecordRequest, RecordRefResolutionResult, ResolveRecordRefRequest, SourceRecordDetail,
};
use std::collections::{BTreeMap, BTreeSet};

pub(super) fn hydrate_participant_records(
    service: &AtlasAppService,
    participants: &[EncounterParticipant],
) -> AppServiceResult<BTreeMap<String, HydratedParticipantRecord>> {
    let keys = participants
        .iter()
        .filter_map(|p| {
            p.record_key
                .as_deref()
                .and_then(|s| RecordKey::parse(s).ok())
        })
        .collect::<BTreeSet<_>>();
    service.submit_retrieval(move |r| {
        let mut records = BTreeMap::new();
        for key in keys {
            if let Some(detail) = r.get_record(GetRecordRequest {
                record_key: &key,
                selected_content: &[],
            })? {
                records.insert(
                    key.to_string(),
                    HydratedParticipantRecord {
                        view: crate::projection::localized_summary_view(&detail.summary, r),
                        detail,
                        source_fingerprint: r.source_fingerprint().into(),
                    },
                );
            }
        }
        Ok(records)
    })
}
pub(super) fn resolve_record_ref(
    service: &AtlasAppService,
    record_ref: &str,
) -> AppServiceResult<SourceRecordDetail> {
    let reference = record_ref.to_owned();
    service.submit_retrieval(move |r| {
        let key = match r.resolve_record_ref(ResolveRecordRefRequest {
            record_ref: &reference,
            filter: None,
        })? {
            RecordRefResolutionResult::Key(k) => k,
            RecordRefResolutionResult::Miss => {
                return Err(AppServiceError::new(
                    AppErrorCode::RecordResolutionMiss,
                    format!("record `{reference}` was not found"),
                ));
            }
            RecordRefResolutionResult::Ambiguous(_) => {
                return Err(AppServiceError::new(
                    AppErrorCode::RecordResolutionAmbiguous,
                    format!("record `{reference}` is ambiguous"),
                ));
            }
        };
        r.get_record(GetRecordRequest {
            record_key: &key,
            selected_content: &[],
        })?
        .ok_or_else(|| {
            AppServiceError::new(
                AppErrorCode::RecordNotFound,
                format!("record `{key}` was not found"),
            )
        })
    })
}

pub(super) struct HydratedParticipantRecord {
    pub detail: SourceRecordDetail,
    pub source_fingerprint: String,
    pub view: atlas_app_model::RecordSummaryView,
}
