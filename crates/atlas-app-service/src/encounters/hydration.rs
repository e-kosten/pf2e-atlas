use crate::{AppServiceError, AppServiceResult};
use atlas_app_model::AppErrorCode;
use atlas_domain::RecordKey;
use atlas_local_state::EncounterParticipant;
use atlas_search::{
    GetRecordRequest, RecordRefResolutionResult, ResolveRecordRefRequest, SourceRecordDetail,
};
use std::collections::{BTreeMap, BTreeSet};

pub(super) fn source_baseline(
    record: &atlas_record::source_record::SourceBackedRecord,
    retrieval: &mut atlas_search::AtlasRetrievalService,
    selection: Option<(&[atlas_domain::OwnedContentLocator], u8)>,
) -> AppServiceResult<Option<super::stat_block::StatBlockView>> {
    let Some(mut baseline) =
        super::rules::authored_stat_baseline(record, retrieval.source_fingerprint(), selection)
    else {
        return Ok(None);
    };
    baseline.source_conditions =
        super::rules::source_conditions::source_conditions(record, |uuid| {
            Ok(
                match retrieval
                    .resolve_source_root_reference(uuid)?
                    .map(|reference| reference.target)
                {
                    Some(atlas_record::source_content::ContentReferenceTarget::Record { key }) => {
                        super::conditions::condition_rule_for_key(Some(&key.to_string()))
                    }
                    _ => None,
                },
            )
        })?;
    if !baseline.source_conditions.entries.is_empty()
        || !baseline.source_conditions.options.is_empty()
        || !baseline.source_conditions.uses.is_empty()
    {
        baseline.source_conditions.source_fingerprint = Some(retrieval.source_fingerprint().into());
    }
    Ok(Some(baseline))
}

pub(super) fn hydrate_participant_records(
    r: &mut atlas_search::AtlasRetrievalService,
    participants: &[EncounterParticipant],
) -> AppServiceResult<(Option<String>, BTreeMap<String, HydratedParticipantRecord>)> {
    let keys = participants
        .iter()
        .filter_map(|p| {
            p.record_key
                .as_deref()
                .and_then(|s| RecordKey::parse(s).ok())
        })
        .collect::<BTreeSet<_>>();
    if keys.is_empty() {
        return Ok((None, BTreeMap::new()));
    }

    let mut records = BTreeMap::new();
    for key in keys {
        if let Some(detail) = r.get_record(GetRecordRequest {
            record_key: &key,
            selected_content: &[],
        })? {
            let view = crate::projection::localized_summary_view(&detail.summary, r);
            let node = atlas_record::source_record::SourceQueryView::new(
                detail.source.source(),
                key.pack().as_str(),
                "",
            )
            .source;
            let presentation = crate::presentation::record_presentation(
                &detail.source,
                node,
                &view,
                &[],
                vec![],
                r,
            )?;
            let arithmetic_baseline = source_baseline(&detail.source, r, None)?;
            let adjustment_applicable = !matches!(
                atlas_record::source_record::SourceQueryView::new(
                    detail.source.source(),
                    key.pack().as_str(),
                    "",
                )
                .actor()
                .authored_adjustment(),
                atlas_record::source_record::SourceFieldView::NotApplicable
            );
            records.insert(
                key.to_string(),
                HydratedParticipantRecord {
                    source_fingerprint: r.source_fingerprint().to_owned(),
                    view,
                    presentation,
                    arithmetic_baseline,
                    adjustment_applicable,
                    detail,
                },
            );
        }
    }
    Ok((Some(r.artifact_generation()?), records))
}

pub(super) fn participant_source_conditions(
    participant: &EncounterParticipant,
    records: &BTreeMap<String, HydratedParticipantRecord>,
) -> super::rules::source_conditions::SourceConditions {
    let Some(key) = &participant.record_key else {
        return Default::default();
    };
    records
        .get(key)
        .and_then(|record| record.arithmetic_baseline.as_ref())
        .map(|baseline| baseline.source_conditions.clone())
        .unwrap_or_else(super::rules::source_conditions::SourceConditions::unavailable)
}
pub(super) fn roster_source_conditions(
    participants: &[EncounterParticipant],
    records: &BTreeMap<String, HydratedParticipantRecord>,
) -> BTreeMap<String, super::rules::source_conditions::SourceConditions> {
    participants
        .iter()
        .map(|participant| {
            (
                participant.participant_key.clone(),
                participant_source_conditions(participant, records),
            )
        })
        .collect()
}

pub(super) fn resolve_record_ref(
    r: &mut atlas_search::AtlasRetrievalService,
    record_ref: &str,
) -> AppServiceResult<SourceRecordDetail> {
    let reference = record_ref.to_owned();

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
}

pub(super) struct HydratedParticipantRecord {
    pub source_fingerprint: String,
    // HP-edit policy reads the checked HP/level fields; participant overlays use
    // the already extracted semantic and arithmetic baselines below.
    pub detail: SourceRecordDetail,
    pub arithmetic_baseline: Option<super::mechanics::StatBlockView>,
    pub adjustment_applicable: bool,
    pub view: atlas_app_model::RecordSummaryView,
    pub presentation: atlas_app_model::RecordPresentationView,
}
