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
    baseline.source_conditions = source_conditions(record, retrieval)?;
    Ok(Some(baseline))
}

pub(super) fn source_conditions(
    record: &atlas_record::source_record::SourceBackedRecord,
    retrieval: &mut atlas_search::AtlasRetrievalService,
) -> AppServiceResult<super::rules::source_conditions::SourceConditions> {
    if !matches!(
        record.node_at(&[]),
        Some(atlas_record::source_record::SourceNodeView::Actor(_))
    ) {
        return Ok(super::rules::source_conditions::SourceConditions::unavailable());
    }
    let mut sources = super::rules::source_conditions::source_conditions(record, |uuid| {
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
    if !sources.entries.is_empty() || !sources.options.is_empty() || !sources.uses.is_empty() {
        sources.source_fingerprint = Some(retrieval.source_fingerprint().into());
    }
    Ok(sources)
}

/// Populate this operation's checked roots; a selected read can seed the map.
/// Saved overlays and rank-specific mechanics are deliberately not cached here.
pub(super) fn load_participant_sources(
    r: &mut atlas_search::AtlasRetrievalService,
    participants: &[EncounterParticipant],
    records: &mut BTreeMap<String, SourceRecordDetail>,
) -> AppServiceResult<Option<String>> {
    let keys = participants
        .iter()
        .filter_map(|p| {
            p.record_key
                .as_deref()
                .and_then(|key| RecordKey::parse(key).ok())
        })
        .collect::<BTreeSet<_>>();
    if keys.is_empty() {
        return Ok(None);
    }
    for key in keys {
        if records.contains_key(&key.to_string()) {
            continue;
        }
        if let Some(detail) = r.get_record(GetRecordRequest {
            record_key: &key,
            selected_content: &[],
        })? {
            records.insert(key.to_string(), detail);
        }
    }
    Ok(Some(r.artifact_generation()?))
}

pub(super) fn prepare_participant_records(
    r: &mut atlas_search::AtlasRetrievalService,
    participants: &[EncounterParticipant],
) -> AppServiceResult<(Option<String>, BTreeMap<String, PreparedParticipantRecord>)> {
    let mut sources = BTreeMap::new();
    let generation = load_participant_sources(r, participants, &mut sources)?;
    let mut records = BTreeMap::new();
    for (key, detail) in sources {
        let arithmetic_baseline = source_baseline(&detail.source, r, None)?;
        let adjustment_applicable = adjustment_applicable(&detail.source);
        records.insert(
            key,
            PreparedParticipantRecord {
                source_fingerprint: r.source_fingerprint().to_owned(),
                arithmetic_baseline,
                adjustment_applicable,
                detail,
            },
        );
    }
    Ok((generation, records))
}

pub(super) fn adjustment_applicable(
    record: &atlas_record::source_record::SourceBackedRecord,
) -> bool {
    !matches!(
        atlas_record::source_record::SourceQueryView::new(
            record.source(),
            record.key().pack().as_str(),
            "",
        )
        .actor()
        .authored_adjustment(),
        atlas_record::source_record::SourceFieldView::NotApplicable
    )
}

/// Resolve authored conditions once per checked root, then associate them with
/// each participant; saved causes and option state are evaluated by the rules owner.
pub(super) fn roster_source_conditions_from_sources(
    r: &mut atlas_search::AtlasRetrievalService,
    participants: &[EncounterParticipant],
    records: &BTreeMap<String, SourceRecordDetail>,
    mut sources: BTreeMap<String, super::rules::source_conditions::SourceConditions>,
) -> AppServiceResult<BTreeMap<String, super::rules::source_conditions::SourceConditions>> {
    for (key, detail) in records {
        if !sources.contains_key(key) {
            sources.insert(key.clone(), source_conditions(&detail.source, r)?);
        }
    }
    Ok(participants
        .iter()
        .map(|participant| {
            let conditions = match participant.record_key.as_deref() {
                None => Default::default(),
                Some(key) => sources
                    .get(key)
                    .cloned()
                    .unwrap_or_else(super::rules::source_conditions::SourceConditions::unavailable),
            };
            (participant.participant_key.clone(), conditions)
        })
        .collect())
}

pub(super) fn participant_source_conditions(
    participant: &EncounterParticipant,
    records: &BTreeMap<String, PreparedParticipantRecord>,
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
    records: &BTreeMap<String, PreparedParticipantRecord>,
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
    let key = resolve_record_key(r, record_ref)?;
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

pub(super) fn resolve_record_key(
    r: &atlas_search::AtlasRetrievalService,
    record_ref: &str,
) -> AppServiceResult<RecordKey> {
    let reference = record_ref;

    match r.resolve_record_ref(ResolveRecordRefRequest {
        record_ref: reference,
        filter: None,
    })? {
        RecordRefResolutionResult::Key(k) => Ok(k),
        RecordRefResolutionResult::Miss => Err(AppServiceError::new(
            AppErrorCode::RecordResolutionMiss,
            format!("record `{reference}` was not found"),
        )),
        RecordRefResolutionResult::Ambiguous(_) => Err(AppServiceError::new(
            AppErrorCode::RecordResolutionAmbiguous,
            format!("record `{reference}` is ambiguous"),
        )),
    }
}

pub(super) struct PreparedParticipantRecord {
    pub source_fingerprint: String,
    // HP/resource policy keeps checked facts; arithmetic consumers prepare only
    // the shared runtime baseline, never semantic presentation.
    pub detail: SourceRecordDetail,
    pub arithmetic_baseline: Option<super::mechanics::StatBlockView>,
    pub adjustment_applicable: bool,
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn direct_source_conditions_preserve_known_and_unavailable_facts_without_arithmetic() {
        let records = [
            (json!({"_id":"aaaaaaaaaaaaaaaa","type":"npc","name":"Known empty","system":{"attributes":{"immunities":[]}},"items":[]}), false, true),
            (json!({"_id":"bbbbbbbbbbbbbbbb","type":"npc","name":"Missing system"}), false, false),
            (json!({"_id":"cccccccccccccccc","type":"npc","name":"Invalid system","system":17,"items":[]}), false, false),
            (json!({"_id":"dddddddddddddddd","type":"hazard","name":"Unavailable items","system":{},"items":null}), true, false),
            (json!({"_id":"eeeeeeeeeeeeeeee","type":"npc","name":"Invalid items","system":{},"items":17}), true, false),
            (json!({"_id":"ffffffffffffffff","type":"action","name":"Not an actor","system":{}}), true, false),
        ].into_iter().map(|(source, unavailable, immunity_complete)| {
            let kind = if source["type"] == "action" { "Item" } else { "Actor" };
            (atlas_search::test_support::record("fixture", kind, source).record, unavailable, immunity_complete)
        }).collect::<Vec<_>>();
        let f = crate::test_support::encounter_fixture_worker();
        f.worker
            .with_workspace(
                &crate::ReadExecutionContext::uncancellable(),
                move |workspace| {
                    let r = workspace.retrieval_mut();
                    for (record, unavailable, immunity_complete) in records {
                        super::super::mechanics::baseline_count(true);
                        let direct = source_conditions(&record, r)?;
                        assert_eq!(super::super::mechanics::baseline_count(false), 0);
                        assert_eq!(
                            direct.unavailable_reason.is_some(),
                            unavailable,
                            "{}",
                            record.key()
                        );
                        assert_eq!(
                            direct.immunities.authored_complete,
                            immunity_complete,
                            "{}",
                            record.key()
                        );
                        assert!(direct.entries.is_empty(), "{}", record.key());
                        assert!(direct.options.is_empty(), "{}", record.key());
                        assert!(direct.uses.is_empty(), "{}", record.key());
                        assert!(direct.source_fingerprint.is_none(), "{}", record.key());
                    }
                    Ok(())
                },
            )
            .unwrap();
    }
}
