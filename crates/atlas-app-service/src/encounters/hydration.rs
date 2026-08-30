use std::collections::BTreeMap;

use atlas_app_model::AppErrorCode;
use atlas_domain::RecordKey;
use atlas_local_state::EncounterParticipant;
use atlas_record::MetricValue;
use atlas_search::{
    GetRecordsRequest, RecordRefResolutionResult, RecordRetrieval, ResolveRecordRefRequest,
};

use crate::error::{AppServiceError, AppServiceResult};
use crate::service::AtlasAppService;

pub(super) fn hydrate_participant_records(
    service: &AtlasAppService,
    participants: &[EncounterParticipant],
) -> AppServiceResult<BTreeMap<String, atlas_record::RetrievedRecord>> {
    let record_keys = participants
        .iter()
        .filter_map(|participant| {
            participant
                .record_key
                .as_deref()
                .and_then(|value| RecordKey::parse(value).ok())
        })
        .collect::<Vec<_>>();
    if record_keys.is_empty() {
        return Ok(BTreeMap::new());
    }
    service.submit_retrieval(move |retrieval| {
        Ok(retrieval
            .get_records(GetRecordsRequest {
                record_keys: &record_keys,
            })?
            .into_iter()
            .map(|retrieved| (retrieved.record.identity.key.to_string(), retrieved))
            .collect())
    })
}

pub(super) fn resolve_record_ref(
    service: &AtlasAppService,
    record_ref: &str,
) -> AppServiceResult<atlas_record::AtlasRecord> {
    let record_ref = record_ref.to_string();
    service.submit_retrieval(move |retrieval| {
        let resolution = retrieval.resolve_record_ref(ResolveRecordRefRequest {
            record_ref: &record_ref,
            filter: None,
        })?;
        let key = match resolution {
            RecordRefResolutionResult::Key(key) => key,
            RecordRefResolutionResult::Miss => {
                return Err(AppServiceError::new(
                    AppErrorCode::RecordResolutionMiss,
                    format!("record `{record_ref}` was not found"),
                ));
            }
            RecordRefResolutionResult::Ambiguous(_) => {
                return Err(AppServiceError::new(
                    AppErrorCode::RecordResolutionAmbiguous,
                    format!("record `{record_ref}` is ambiguous"),
                ));
            }
        };
        let records = retrieval.get_records(GetRecordsRequest {
            record_keys: std::slice::from_ref(&key),
        })?;
        records
            .into_iter()
            .next()
            .map(|record| record.record)
            .ok_or_else(|| {
                AppServiceError::new(
                    AppErrorCode::RecordNotFound,
                    format!("record not found: {key}"),
                )
            })
    })
}

pub(super) fn default_hp(record: &atlas_record::AtlasRecord) -> (Option<i64>, Option<i64>) {
    let metric = |key: &str| {
        record.mechanics.metrics.iter().find_map(|metric| {
            if metric.key == key {
                match metric.value {
                    MetricValue::Number(value) => Some(value.round() as i64),
                    MetricValue::Text(_) | MetricValue::Boolean(_) => None,
                }
            } else {
                None
            }
        })
    };
    let max_hp = atlas_record::metrics::actor::HP_MAX
        .exact_key()
        .and_then(metric);
    let current_hp = atlas_record::metrics::actor::HP_VALUE
        .exact_key()
        .and_then(metric)
        .or(max_hp);
    (max_hp.or(current_hp), current_hp)
}
