//! Stateless reference dice: resolve checked source, validate the whole pool, then roll.
use crate::{AppServiceError, AppServiceResult, AtlasAppService, ReadExecutionContext};
use atlas_app_model::{
    ContentControlKindView, ContentDamageRollComponentView, ContentDamageRollView,
    PreparedFieldBodyView, RecordDetailRequest, RollContentDamageRequest,
};

impl AtlasAppService {
    pub fn roll_content_damage(
        &self,
        request: RollContentDamageRequest,
    ) -> AppServiceResult<ContentDamageRollView> {
        self.with_retrieval(&ReadExecutionContext::uncancellable(), move |retrieval| {
            if request.source_fingerprint != retrieval.source_fingerprint() {
                return Err(AppServiceError::invalid_request(
                    "This source changed. Reopen the record before rolling.",
                ));
            }
            let (_, detail) = crate::records::read_record_detail(
                retrieval,
                RecordDetailRequest {
                    record_key: request.source.record.to_string(),
                    owners: request.source.owners.clone(),
                    fields: vec![request.source.field.clone()],
                    passage: None,
                    spell_rank: None,
                    source_fingerprint: Some(request.source_fingerprint.clone()),
                },
            )?;
            let control = detail
                .presentation
                .content
                .iter()
                .find(|field| field.locator == request.source)
                .and_then(|field| match &field.body {
                    PreparedFieldBodyView::Html { controls, .. } => {
                        controls.iter().find(|c| c.ordinal == request.ordinal)
                    }
                    _ => None,
                })
                .ok_or_else(|| {
                    AppServiceError::invalid_request(
                        "This damage expression is unavailable. Reopen the record.",
                    )
                })?;
            let ContentControlKindView::Damage {
                roll: Some(current),
                literal: Some(parts),
                ..
            } = &control.control
            else {
                return Err(AppServiceError::invalid_request(
                    "This damage expression cannot be rolled as a reference.",
                ));
            };
            if current != &request {
                return Err(AppServiceError::invalid_request(
                    "This damage expression changed. Reopen the record before rolling.",
                ));
            }
            // Validate every instance before consuming randomness; no partial mixed-type result.
            let prepared = parts
                .iter()
                .map(|part| {
                    atlas_dice::prepare(&part.formula)
                        .map(|roll| (part.label.clone(), roll))
                        .map_err(|e| AppServiceError::invalid_request(e.to_string()))
                })
                .collect::<AppServiceResult<Vec<_>>>()?;
            let components = prepared
                .into_iter()
                .map(|(label, roll)| {
                    roll.roll()
                        .map(|outcome| ContentDamageRollComponentView { label, outcome })
                        .map_err(|e| AppServiceError::invalid_request(e.to_string()))
                })
                .collect::<AppServiceResult<Vec<_>>>()?;
            Ok(ContentDamageRollView { components })
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{executor::RetrievalExecutor, test_support::fixture_worker_with_executor};
    use atlas_app_model::ContentControlKindView;
    use atlas_search::test_support::{open_source_fixture, record};
    use serde_json::json;

    #[test]
    fn reference_rolls_preserve_typed_totals_and_reject_missing_context_or_stale_sources() {
        let fixture = fixture_worker_with_executor(RetrievalExecutor::from_test_fixture_factory(
            1,
            16,
            || {
                Ok(open_source_fixture(
                    vec![record(
                        "items",
                        "Item",
                        json!({
                            "_id":"damageTest100000", "name":"Damage examples", "type":"equipment",
                            "system":{"description":{"value":"<p>@Damage[(4d8+10)[bludgeoning],1d6[fire],2[cold]] @Damage[1[fire]] @Damage[(@actor.level+2d6)[fire]] @Damage[1d6[fire]|unknown:option] @Damage[1d6[fire],broken] @Damage[1d6[fire]|options:area-damage|traits:fire] @Damage[1d6[fire]|domains:@actor.domain]</p>"}}
                        }),
                    )],
                    false,
                )?)
            },
        ));
        let detail = fixture
            .worker
            .record_detail("items:damageTest100000")
            .unwrap();
        let PreparedFieldBodyView::Html { controls, .. } = &detail
            .presentation
            .content
            .iter()
            .find(|f| f.locator.field == "/system/description/value")
            .unwrap()
            .body
        else {
            panic!("html")
        };
        let ContentControlKindView::Damage {
            roll: Some(request),
            ..
        } = &controls[0].control
        else {
            panic!("roll offer")
        };
        for _ in 0..2 {
            let result = fixture.worker.roll_content_damage(request.clone()).unwrap();
            assert_eq!(result.components.len(), 3);
            assert_eq!(result.components[0].label, "bludgeoning damage");
            assert!((14..=42).contains(&result.components[0].outcome.total));
            assert!((1..=6).contains(&result.components[1].outcome.total));
            assert_eq!(result.components[2].outcome.total, 2);
        }
        let ContentControlKindView::Damage {
            roll: Some(area), ..
        } = &controls[5].control
        else {
            panic!("literal reference with retained metadata")
        };
        assert!(fixture.worker.roll_content_damage(area.clone()).is_ok());
        for control in controls.iter().skip(1).filter(|c| c.ordinal != 5) {
            assert!(matches!(
                control.control,
                ContentControlKindView::Damage { roll: None, .. }
            ));
            let mut invalid = request.clone();
            invalid.ordinal = control.ordinal;
            assert!(fixture.worker.roll_content_damage(invalid).is_err());
        }
        let mut old_policy = request.clone();
        old_policy.content_interpretation_version = "foundry-content/v10".into();
        assert!(fixture.worker.roll_content_damage(old_policy).is_err());
        let mut stale = request.clone();
        stale.source_fingerprint = "old".into();
        assert!(fixture.worker.roll_content_damage(stale).is_err());
        let mut changed = request.clone();
        changed.authored_formula = "100d100[fire]".into();
        assert!(fixture.worker.roll_content_damage(changed).is_err());
        let mut missing = request.clone();
        missing.source.field = "/not-content".into();
        assert!(fixture.worker.roll_content_damage(missing).is_err());
    }
}
