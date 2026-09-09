use atlas_app_model::{AppErrorCode, RecordDetailRequest, RecordDetailView};
use atlas_domain::{RecordKey, SearchFilterNode};
use atlas_search::{
    GetRecordRequest, GetRecordsRequest, GraphContextRequest, GraphRetrieval,
    RecordRefResolutionResult, RecordResolutionResult, RecordRetrieval, ResolveRecordRefRequest,
    ResolveRecordRequest,
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
        let RecordDetailRequest {
            child_locator,
            spell_form_id,
            spell_cast_rank,
            reference_outgoing_limit,
            reference_backlink_limit,
        } = request;
        let child_locator = child_locator
            .map(|value| {
                atlas_record::decode_content_child_locator(&value).map_err(|_| {
                    AppServiceError::invalid_request("child_locator is not a valid opaque locator")
                })
            })
            .transpose()?;
        if child_locator
            .as_ref()
            .is_some_and(|locator| locator.parent != record_key)
        {
            return Err(AppServiceError::invalid_request(
                "child_locator does not belong to the requested parent record",
            ));
        }
        let spell_selection = match (spell_form_id, spell_cast_rank) {
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
        let outgoing_limit = reference_outgoing_limit.unwrap_or(8);
        let backlink_limit = reference_backlink_limit.unwrap_or(0);
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
            if let Some(locator) = child_locator.as_ref() {
                let belongs = match &record.body {
                    Some(atlas_record::RecordBody::Journal(journal)) => journal
                        .pages
                        .as_value()
                        .and_then(atlas_record::H8FieldValue::known)
                        .is_some_and(|pages| {
                            pages.iter().any(|page| match page {
                                atlas_record::JournalPageEntry::Page(page) => {
                                    &page.locator == locator
                                }
                                atlas_record::JournalPageEntry::Unsupported(page) => {
                                    &page.locator == locator
                                }
                            })
                        }),
                    Some(atlas_record::RecordBody::RollTable(table)) => table
                        .results
                        .as_value()
                        .and_then(atlas_record::H8FieldValue::known)
                        .is_some_and(|results| {
                            results.iter().any(|result| match result {
                                atlas_record::TableResultEntry::Result(result) => {
                                    &result.locator == locator
                                }
                                atlas_record::TableResultEntry::Unsupported(result) => {
                                    &result.locator == locator
                                }
                            })
                        }),
                    _ => false,
                };
                if !belongs {
                    return Err(AppServiceError::invalid_request(
                        "child_locator does not identify a child of this record",
                    ));
                }
            }
            let remaster_lookup = verified_remaster_lookup(retrieval, &record)?;
            let mut detail = record_detail(&record, spell_selection, &remaster_lookup)?;
            detail.surface.references = Some(if outgoing_limit == 0 && backlink_limit == 0 {
                crate::record_references::not_requested_record_references()
            } else {
                match retrieval.graph_context(
                    GraphContextRequest::new(record_key)
                        .with_outgoing_limit(usize::from(outgoing_limit))
                        .with_backlink_limit(usize::from(backlink_limit)),
                ) {
                    Ok(Some(result)) => crate::record_references::project_record_references(
                        result,
                        outgoing_limit,
                        backlink_limit,
                    ),
                    Ok(None) => crate::record_references::missing_record_references(
                        outgoing_limit,
                        backlink_limit,
                    ),
                    Err(error) => crate::record_references::unavailable_record_references(
                        outgoing_limit,
                        backlink_limit,
                        &error,
                    ),
                }
            });
            Ok(detail)
        })
    }
}

#[cfg(test)]
mod tests {
    use atlas_app_model::{
        AppErrorCode, HazardSurfaceProvenanceTextView, HazardSurfaceRuleView, RecordDetailRequest,
        RecordSurfaceEditionStatusView, RecordSurfacePresentationView, RecordSurfaceProfileView,
        RecordSurfaceReferenceSectionView, SurfaceUnavailableReasonView,
    };
    use atlas_domain::RecordKey;
    use atlas_record::{
        ContentChildIdentity, ContentChildKind, ContentChildLocator, SourceDocumentId,
        encode_content_child_locator,
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
        let references = detail
            .surface
            .references
            .expect("record detail should own reference state");
        assert!(matches!(
            references.outgoing,
            RecordSurfaceReferenceSectionView::Available {
                requested_limit: 8,
                total_records: 0,
                total_edges: 0,
                truncated: false,
                ..
            }
        ));
        assert_eq!(
            references.backlinks,
            RecordSurfaceReferenceSectionView::NotRequested
        );

        let no_references = worker
            .record_detail(
                "actions:testAction1",
                RecordDetailRequest {
                    reference_outgoing_limit: Some(0),
                    reference_backlink_limit: Some(0),
                    ..RecordDetailRequest::default()
                },
            )
            .expect("explicit zero limits should remain a valid detail request");
        let references = no_references
            .surface
            .references
            .expect("record detail should retain explicit not-requested states");
        assert_eq!(
            references.outgoing,
            RecordSurfaceReferenceSectionView::NotRequested
        );
        assert_eq!(
            references.backlinks,
            RecordSurfaceReferenceSectionView::NotRequested
        );

        let explicit_backlinks = worker
            .record_detail(
                "actions:testAction1",
                RecordDetailRequest {
                    reference_outgoing_limit: Some(0),
                    reference_backlink_limit: Some(8),
                    ..RecordDetailRequest::default()
                },
            )
            .expect("explicit backlink request should remain a record-detail concern");
        let references = explicit_backlinks
            .surface
            .references
            .expect("record detail should own reference state");
        assert_eq!(
            references.outgoing,
            RecordSurfaceReferenceSectionView::NotRequested
        );
        assert!(matches!(
            references.backlinks,
            RecordSurfaceReferenceSectionView::Available {
                requested_limit: 8,
                total_records: 0,
                total_edges: 0,
                truncated: false,
                ..
            }
        ));

        let unavailable_references = worker
            .record_detail(
                "actions:testAction1",
                RecordDetailRequest {
                    reference_outgoing_limit: Some(51),
                    ..RecordDetailRequest::default()
                },
            )
            .expect("reference failure must not discard record detail");
        let references = unavailable_references
            .surface
            .references
            .expect("record detail should retain typed reference failure");
        assert!(matches!(
            references.outgoing,
            RecordSurfaceReferenceSectionView::Unavailable {
                requested_limit: 51,
                code: AppErrorCode::InvalidRequest,
                ..
            }
        ));
        assert_eq!(
            references.backlinks,
            RecordSurfaceReferenceSectionView::NotRequested
        );

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
                ..RecordDetailRequest::default()
            },
            RecordDetailRequest {
                spell_form_id: None,
                spell_cast_rank: Some(5),
                ..RecordDetailRequest::default()
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
                    ..RecordDetailRequest::default()
                },
            )
            .expect_err("spell selection should be rejected for non-spell records")
            .into_app_error();
        assert_eq!(non_spell.code, AppErrorCode::InvalidRequest);

        let malformed_child = worker
            .record_detail(
                "actions:testAction1",
                RecordDetailRequest {
                    child_locator: Some("not-an-opaque-child-locator".to_string()),
                    ..RecordDetailRequest::default()
                },
            )
            .expect_err("malformed child locators should fail before retrieval")
            .into_app_error();
        assert_eq!(malformed_child.code, AppErrorCode::InvalidRequest);

        let child_id = SourceDocumentId::new("page-id").expect("child source ID");
        let wrong_parent = encode_content_child_locator(&ContentChildLocator {
            parent: RecordKey::parse("journals:other").expect("other parent key"),
            kind: ContentChildKind::JournalPage,
            identity: ContentChildIdentity::Stable(child_id.clone()),
        });
        let wrong_parent = worker
            .record_detail(
                "actions:testAction1",
                RecordDetailRequest {
                    child_locator: Some(wrong_parent),
                    ..RecordDetailRequest::default()
                },
            )
            .expect_err("a child locator must bind the requested parent")
            .into_app_error();
        assert_eq!(wrong_parent.code, AppErrorCode::InvalidRequest);

        let non_child = encode_content_child_locator(&ContentChildLocator {
            parent: RecordKey::parse("actions:testAction1").expect("fixture parent key"),
            kind: ContentChildKind::JournalPage,
            identity: ContentChildIdentity::Stable(child_id),
        });
        let non_child = worker
            .record_detail(
                "actions:testAction1",
                RecordDetailRequest {
                    child_locator: Some(non_child),
                    ..RecordDetailRequest::default()
                },
            )
            .expect_err("a locator must identify a child owned by the loaded record")
            .into_app_error();
        assert_eq!(non_child.code, AppErrorCode::InvalidRequest);
    }
}
