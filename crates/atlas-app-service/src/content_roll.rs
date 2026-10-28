//! Stateless reference dice: resolve checked source, validate the whole pool, then roll.
use crate::{AppServiceError, AppServiceResult, AtlasAppService, ReadExecutionContext};
use atlas_app_model::{
    ContentControlKindView, ContentRollComponentResultView, ContentRollResultView,
    PreparedFieldBodyView, RecordDetailRequest, RollContentRequest,
};

impl AtlasAppService {
    pub fn roll_content(
        &self,
        request: RollContentRequest,
    ) -> AppServiceResult<ContentRollResultView> {
        self.with_workspace(&ReadExecutionContext::uncancellable(), move |workspace| {
            if request.source_fingerprint != workspace.retrieval_mut().source_fingerprint() {
                return Err(AppServiceError::invalid_request(
                    "This source changed. Reopen the record before rolling.",
                ));
            }
            let selection = RecordDetailRequest {
                record_key: request.source.record.to_string(),
                owners: request.source.owners.clone(),
                fields: vec![request.source.field.clone()],
                passage: None,
                spell_rank: request.spell_rank,
                source_fingerprint: Some(request.source_fingerprint.clone()),
            };
            let detail = if let Some(binding) = &request.encounter {
                crate::encounters::reference::read_encounter_reference(
                    workspace,
                    &binding.encounter_ref,
                    &binding.participant_key,
                    atlas_app_model::EncounterReferenceRequest {
                        artifact_generation: binding.artifact_generation.clone(),
                        selection,
                        condition_id: binding.condition_id,
                    },
                )?
            } else {
                crate::records::read_record_detail(workspace.retrieval_mut(), selection)?.1
            };
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
                        "This roll expression is unavailable. Reopen the record.",
                    )
                })?;
            let (current, parts) = match &control.control {
                ContentControlKindView::Damage {
                    roll: Some(current),
                    literal: Some(parts),
                    ..
                }
                | ContentControlKindView::Command {
                    roll: Some(current),
                    literal: Some(parts),
                    ..
                } => (current, parts),
                _ => {
                    return Err(AppServiceError::invalid_request(
                        "This roll expression cannot be rolled as a reference.",
                    ));
                }
            };
            if current != &request {
                return Err(AppServiceError::invalid_request(
                    "This roll expression changed. Reopen the record before rolling.",
                ));
            }
            // Validate every instance before consuming randomness; no partial mixed-type result.
            let prepared = parts
                .components
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
                        .map(|outcome| ContentRollComponentResultView { label, outcome })
                        .map_err(|e| AppServiceError::invalid_request(e.to_string()))
                })
                .collect::<AppServiceResult<Vec<_>>>()?;
            Ok(ContentRollResultView { components })
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
            let result = fixture.worker.roll_content(request.clone()).unwrap();
            assert_eq!(
                result
                    .components
                    .iter()
                    .map(|part| (part.label.as_str(), part.outcome.formula.as_str()))
                    .collect::<Vec<_>>(),
                [
                    ("bludgeoning damage", "4d8+10"),
                    ("fire damage", "1d6"),
                    ("cold damage", "2")
                ]
            );
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
        assert!(fixture.worker.roll_content(area.clone()).is_ok());
        for control in controls.iter().skip(1).filter(|c| c.ordinal != 5) {
            assert!(matches!(
                control.control,
                ContentControlKindView::Damage { roll: None, .. }
            ));
            let mut invalid = request.clone();
            invalid.ordinal = control.ordinal;
            assert!(fixture.worker.roll_content(invalid).is_err());
        }
        let mut old_policy = request.clone();
        old_policy.content_interpretation_version = "foundry-content/v10".into();
        assert!(fixture.worker.roll_content(old_policy).is_err());
        let mut stale = request.clone();
        stale.source_fingerprint = "old".into();
        assert!(fixture.worker.roll_content(stale).is_err());
        let mut changed = request.clone();
        changed.authored_formula = "100d100[fire]".into();
        assert!(fixture.worker.roll_content(changed).is_err());
        let mut missing = request.clone();
        missing.source.field = "/not-content".into();
        assert!(fixture.worker.roll_content(missing).is_err());
    }
    #[test]
    fn inline_modes_roll_the_checked_formula_and_reject_command_or_source_changes() {
        let fixture = fixture_worker_with_executor(RetrievalExecutor::from_test_fixture_factory(
            1,
            16,
            || {
                Ok(open_source_fixture(
                    vec![record(
                        "items",
                        "Item",
                        json!({
                            "_id":"inlineRoll100000", "name":"Inline rolls", "type":"equipment",
                            "system":{"description":{"value":"<p>[[/r 2d6+3 #rounds]] [[/gmr 2d6+3 #rounds]] [[/br 2d6+3 #rounds]] [[/r @actor.level+2d6]] [[/r 4 #rounds]] [[/act seek]] [[/r 2d6kh]]</p>"}}
                        }),
                    )],
                    false,
                )?)
            },
        ));
        let detail = fixture
            .worker
            .record_detail("items:inlineRoll100000")
            .unwrap();
        let PreparedFieldBodyView::Html { controls, .. } = &detail
            .presentation
            .content
            .iter()
            .find(|field| field.locator.field == "/system/description/value")
            .unwrap()
            .body
        else {
            panic!("html")
        };
        for (index, mode) in ["r", "gmr", "br"].iter().enumerate() {
            let ContentControlKindView::Command {
                roll: Some(request),
                ..
            } = &controls[index].control
            else {
                panic!("roll offer")
            };
            assert_eq!(request.authored_command.as_deref(), Some(*mode));
            let result = fixture.worker.roll_content(request.clone()).unwrap();
            assert_eq!(result.components.len(), 1);
            assert_eq!(result.components[0].label, "rounds");
            assert_eq!(result.components[0].outcome.formula, "2d6+3");
            assert!((5..=15).contains(&result.components[0].outcome.total));
            let mut changed = request.clone();
            changed.authored_command = Some("different".into());
            assert!(fixture.worker.roll_content(changed).is_err());
            let mut changed = request.clone();
            changed.source_fingerprint = "stale".into();
            assert!(fixture.worker.roll_content(changed).is_err());
            for unrollable in controls.iter().skip(3) {
                assert!(matches!(
                    unrollable.control,
                    ContentControlKindView::Command { roll: None, .. }
                ));
                let mut changed = request.clone();
                changed.ordinal = unrollable.ordinal;
                assert!(fixture.worker.roll_content(changed).is_err());
            }
        }
    }
}
