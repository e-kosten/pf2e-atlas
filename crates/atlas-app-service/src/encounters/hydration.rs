use std::collections::BTreeMap;

use atlas_app_model::AppErrorCode;
use atlas_domain::RecordKey;
use atlas_local_state::EncounterParticipant;
use atlas_search::{
    GetRecordsRequest, RecordRefResolutionResult, RecordRetrieval, ResolveRecordRefRequest,
};

use crate::error::{AppServiceError, AppServiceResult};
use crate::retrieval::{VerifiedRemasterLookup, verified_remaster_lookups_for_records};
use crate::service::AtlasAppService;

pub(super) struct HydratedParticipantRecords {
    pub(super) records_by_key: BTreeMap<String, atlas_record::RetrievedRecord>,
    pub(super) remaster_lookups_by_key: BTreeMap<String, VerifiedRemasterLookup>,
}

pub(super) fn hydrate_participant_records(
    service: &AtlasAppService,
    participants: &[EncounterParticipant],
) -> AppServiceResult<HydratedParticipantRecords> {
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
        return Ok(HydratedParticipantRecords {
            records_by_key: BTreeMap::new(),
            remaster_lookups_by_key: BTreeMap::new(),
        });
    }
    service.submit_retrieval(move |retrieval| {
        let records_by_key = retrieval
            .get_records(GetRecordsRequest {
                record_keys: &record_keys,
            })?
            .into_iter()
            .map(|retrieved| (retrieved.record.identity.key.to_string(), retrieved))
            .collect::<BTreeMap<_, _>>();
        let remaster_lookups_by_key =
            verified_remaster_lookups_for_records(retrieval, records_by_key.values())?;
        Ok(HydratedParticipantRecords {
            records_by_key,
            remaster_lookups_by_key,
        })
    })
}

pub(super) fn resolve_record_ref(
    service: &AtlasAppService,
    record_ref: &str,
) -> AppServiceResult<atlas_record::AtlasRecord> {
    resolve_retrieved_record_ref(service, record_ref).map(|retrieved| retrieved.record)
}

pub(super) fn resolve_retrieved_record_ref(
    service: &AtlasAppService,
    record_ref: &str,
) -> AppServiceResult<atlas_record::RetrievedRecord> {
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
        records.into_iter().next().ok_or_else(|| {
            AppServiceError::new(
                AppErrorCode::RecordNotFound,
                format!("record not found: {key}"),
            )
        })
    })
}

pub(super) fn default_hp(retrieved: &atlas_record::RetrievedRecord) -> (Option<i64>, Option<i64>) {
    let (maximum, current) = match retrieved.body.as_ref() {
        Some(atlas_record::RecordBody::Creature(creature)) => {
            let Some(hit_points) = creature
                .defenses
                .value
                .as_value()
                .and_then(|defenses| defenses.hit_points.as_value())
            else {
                return (None, None);
            };
            (
                hit_points.maximum.as_value().copied(),
                hit_points.value.as_value().and_then(|value| match value {
                    atlas_record::CreatureNumber::Integer(value) => Some(*value),
                    atlas_record::CreatureNumber::Unsupported(_) => None,
                }),
            )
        }
        Some(atlas_record::RecordBody::Hazard(hazard)) => {
            let Some(hit_points) = hazard
                .defenses
                .typed()
                .and_then(|defenses| defenses.hit_points.typed())
            else {
                return (None, None);
            };
            return (
                hit_points.maximum.typed().copied(),
                hit_points.current.typed().copied(),
            );
        }
        Some(atlas_record::RecordBody::Spell(_))
        | Some(atlas_record::RecordBody::Consumable(_))
        | None => return (None, None),
    };
    (maximum.or(current), current.or(maximum))
}
