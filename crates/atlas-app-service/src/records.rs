use atlas_app_model::{AppErrorCode, RecordDetailRequest, RecordDetailView};
use atlas_domain::{RecordKey, SearchFilterNode};
use atlas_search::{
    GetRecordRequest, GetRecordsRequest, RecordRefResolutionResult, RecordResolutionResult,
    RecordRetrieval, ResolveRecordRefRequest, ResolveRecordRequest,
};

use crate::error::{AppServiceError, AppServiceResult};
use crate::projection::record_detail;
use crate::retrieval::verified_remaster_lookup;
use crate::service::AtlasAppService;

impl AtlasAppService {
    pub fn get_records(
        &self,
        record_keys: Vec<RecordKey>,
    ) -> AppServiceResult<Vec<atlas_record::RetrievedRecord>> {
        self.submit_retrieval(move |retrieval| {
            Ok(retrieval.get_records(GetRecordsRequest {
                record_keys: &record_keys,
            })?)
        })
    }

    pub fn resolve_record(
        &self,
        query: String,
        filter: Option<SearchFilterNode>,
    ) -> AppServiceResult<Vec<RecordResolutionResult>> {
        self.submit_retrieval(move |retrieval| {
            Ok(retrieval.resolve_record(ResolveRecordRequest {
                query: &query,
                filter: filter.as_ref(),
            })?)
        })
    }

    pub fn resolve_record_ref(
        &self,
        record_ref: String,
        filter: Option<SearchFilterNode>,
    ) -> AppServiceResult<RecordRefResolutionResult> {
        self.submit_retrieval(move |retrieval| {
            Ok(retrieval.resolve_record_ref(ResolveRecordRefRequest {
                record_ref: &record_ref,
                filter: filter.as_ref(),
            })?)
        })
    }

    pub fn record_detail(
        &self,
        record_key: &str,
        request: RecordDetailRequest,
    ) -> AppServiceResult<RecordDetailView> {
        let record_key = RecordKey::parse(record_key).map_err(|error| {
            AppServiceError::new(AppErrorCode::InvalidRecordKey, error.to_string())
        })?;
        let spell_selection = match (request.spell_form_id, request.spell_cast_rank) {
            (None, None) => None,
            (Some(form_id), Some(cast_rank)) => Some((
                atlas_record::SpellFormId::new(form_id).map_err(|_| {
                    AppServiceError::invalid_request(
                        "spell_form_id must be non-empty and contain no whitespace",
                    )
                })?,
                cast_rank,
            )),
            _ => {
                return Err(AppServiceError::invalid_request(
                    "spell_form_id and spell_cast_rank must be provided together",
                ));
            }
        };
        self.submit_retrieval(move |retrieval| {
            let record = retrieval
                .get_record(GetRecordRequest {
                    record_key: &record_key,
                })?
                .ok_or_else(|| {
                    AppServiceError::new(
                        AppErrorCode::RecordNotFound,
                        format!("record `{record_key}` was not found"),
                    )
                })?;
            if spell_selection.is_some()
                && !matches!(&record.body, Some(atlas_record::RecordBody::Spell(_)))
            {
                return Err(AppServiceError::invalid_request(
                    "spell form selection is only available for spell records",
                ));
            }
            let remaster_lookup = verified_remaster_lookup(retrieval, &record)?;
            record_detail(&record, spell_selection, &remaster_lookup)
        })
    }
}

#[cfg(test)]
mod tests {
    use atlas_app_model::{
        AppErrorCode, HazardSurfaceProvenanceTextView, HazardSurfaceRuleView, RecordDetailRequest,
        RecordSurfaceEditionStatusView, RecordSurfacePresentationView, RecordSurfaceProfileView,
        SurfaceUnavailableReasonView,
    };

    use crate::test_support::encounter_fixture_worker;

    #[test]
    fn worker_record_detail_reports_valid_invalid_and_missing_keys() {
        let fixture = encounter_fixture_worker();
        let worker = &fixture.worker;

        let detail = worker
            .record_detail("actions:testAction1", RecordDetailRequest::default())
            .expect("fixture record should load");
        assert_eq!(
            detail.surface.metadata.record_key.as_deref(),
            Some("actions:testAction1")
        );
        assert_eq!(detail.surface.metadata.title, "Test Action 1");
        assert_eq!(detail.surface.metadata.kind, "rule");
        let edition = detail
            .surface
            .metadata
            .edition
            .expect("canonical record should expose edition metadata");
        assert_eq!(edition.status, RecordSurfaceEditionStatusView::Legacy);
        assert!(edition.counterparts.is_empty());
        assert_eq!(
            detail.surface.profile,
            RecordSurfaceProfileView::RecordDetail
        );
        assert!(matches!(
            detail.surface.presentation,
            RecordSurfacePresentationView::Unavailable { unavailable }
                if unavailable.reason == SurfaceUnavailableReasonView::RecordFamilyNotMigrated
        ));

        let hazard = worker
            .record_detail("hazards:testHazard", RecordDetailRequest::default())
            .expect("canonical hazard should load");
        let RecordSurfacePresentationView::Hazard { body } = hazard.surface.presentation else {
            panic!("canonical hazard should use the tagged hazard presentation");
        };
        assert_eq!(
            body.defenses
                .as_ref()
                .and_then(|defenses| defenses.saves.as_ref())
                .and_then(|saves| saves.fortitude),
            Some(0)
        );
        let activities = body.activities.as_ref().expect("hazard activities");
        assert_eq!(
            activities
                .iter()
                .map(|activity| activity.occurrence_id.as_str())
                .collect::<Vec<_>>(),
            vec![
                "occurrence-action",
                "occurrence-strike",
                "occurrence-unsupported"
            ]
        );
        assert!(matches!(
            activities[0]
                .rules
                .as_ref()
                .and_then(|rules| rules.first()),
            Some(HazardSurfaceRuleView::Aura { slug: Some(slug), .. })
                if slug == "fixture-aura"
        ));
        assert!(body.unavailable_fields.as_ref().is_none_or(|fields| {
            fields.iter().all(|field| {
                field.field != "activity.slug" && field.field != "activity.publication"
            })
        }));
        assert_eq!(
            body.provenance.image,
            HazardSurfaceProvenanceTextView::Missing
        );
        assert_eq!(
            body.provenance.publication_license,
            HazardSurfaceProvenanceTextView::Missing
        );

        let invalid = worker
            .record_detail("not a key", RecordDetailRequest::default())
            .expect_err("invalid keys should be rejected")
            .into_app_error();
        assert_eq!(invalid.code, AppErrorCode::InvalidRecordKey);

        let missing = worker
            .record_detail("actions:missing", RecordDetailRequest::default())
            .expect_err("missing keys should return not found")
            .into_app_error();
        assert_eq!(missing.code, AppErrorCode::RecordNotFound);

        for request in [
            RecordDetailRequest {
                spell_form_id: Some("spell-form:test".to_string()),
                spell_cast_rank: None,
            },
            RecordDetailRequest {
                spell_form_id: None,
                spell_cast_rank: Some(5),
            },
        ] {
            let error = worker
                .record_detail("actions:testAction1", request)
                .expect_err("partial spell selection should be rejected")
                .into_app_error();
            assert_eq!(error.code, AppErrorCode::InvalidRequest);
        }

        let non_spell = worker
            .record_detail(
                "actions:testAction1",
                RecordDetailRequest {
                    spell_form_id: Some("spell-form:test".to_string()),
                    spell_cast_rank: Some(5),
                },
            )
            .expect_err("spell selection should be rejected for non-spell records")
            .into_app_error();
        assert_eq!(non_spell.code, AppErrorCode::InvalidRequest);
    }
}
