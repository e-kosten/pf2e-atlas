use super::*;
use atlas_app_model::PreviewEncounterDifficultyRequest;

impl AtlasAppService {
    pub fn preview_encounter_difficulty(
        &self,
        encounter_ref: &str,
        request: PreviewEncounterDifficultyRequest,
    ) -> AppServiceResult<EncounterDifficultyView> {
        let reference = local_reference(request.reference);
        reference.validate()?;
        if let Some(addition) = &request.addition {
            super::super::record_addition::validate_quantity(addition.quantity)?;
        }
        let initial = self.with_state(|session| {
            session
                .local_state()
                .encounters()
                .get_with_participants(encounter_ref)?
                .ok_or_else(|| super::super::projection::encounter_not_found(encounter_ref))
        })?;
        if request.addition.is_none()
            && !initial.participants.iter().any(|p| {
                p.record_key
                    .as_deref()
                    .is_some_and(|key| atlas_domain::RecordKey::parse(key).is_ok())
            })
        {
            guard_revision(&initial.encounter, request.expected_revision)?;
            return Ok(project(reference, &initial.participants, |_| None));
        }
        let encounter_ref = encounter_ref.to_owned();
        // One workspace snapshot owns saved state and all source-backed XP inputs.
        self.with_workspace(
            &crate::ReadExecutionContext::uncancellable(),
            move |session| read_preview(session, &encounter_ref, request, reference),
        )
    }
}

fn read_preview(
    session: &mut atlas_runtime::WorkspaceRead,
    encounter_ref: &str,
    request: PreviewEncounterDifficultyRequest,
    reference: EncounterDifficultyReference,
) -> AppServiceResult<EncounterDifficultyView> {
    let snapshot = session
        .local_state()
        .encounters()
        .get_with_participants(encounter_ref)?
        .ok_or_else(|| super::super::projection::encounter_not_found(encounter_ref))?;
    guard_revision(&snapshot.encounter, request.expected_revision)?;
    let mut records = std::collections::BTreeMap::new();
    super::super::hydration::load_participant_sources(
        session.retrieval_mut(),
        &snapshot.participants,
        &mut records,
    )?;
    let mut view = project(reference, &snapshot.participants, |key| records.get(key));
    if let Some(addition) = request.addition {
        let key = super::super::hydration::resolve_record_key(
            session.retrieval_mut(),
            &addition.record_ref,
        )?;
        let record = match records.entry(key.to_string()) {
            std::collections::btree_map::Entry::Occupied(entry) => entry.into_mut(),
            std::collections::btree_map::Entry::Vacant(entry) => entry.insert(
                session
                    .retrieval_mut()
                    .get_record(atlas_search::GetRecordRequest {
                        record_key: &key,
                        selected_content: &[],
                    })?
                    .ok_or_else(|| {
                        crate::AppServiceError::new(
                            atlas_app_model::AppErrorCode::RecordNotFound,
                            format!("record `{key}` was not found"),
                        )
                    })?,
            ),
        };
        let prepared =
            super::super::record_addition::prepare(record, addition.participant_variant)?;
        let source = atlas_record::source_record::SourceQueryView::new(
            record.source.source(),
            record.source.key().pack().as_str(),
            "",
        );
        let level = super::super::rules::effective_level(
            prepared.level_override,
            prepared.participant_variant,
            prepared.variant_origin,
            source.actor().level().value(),
            super::super::hydration::adjustment_applicable(&record.source),
        )
        .as_ref()
        .and_then(super::super::mechanics::integer)
        .and_then(|l| i32::try_from(l).ok());
        view.opposition.push(opponent(
            None,
            prepared.display_name,
            prepared.participant_kind,
            level,
            source.actor().hazard_complexity().value(),
            view.party_level,
            addition.quantity,
        ));
        recalculate(&mut view, &snapshot.participants);
    }
    Ok(view)
}

#[cfg(test)]
mod tests {
    use super::*;
    use atlas_app_model::*;
    use atlas_search::test_support::{open_source_fixture, record};
    use serde_json::json;

    #[test]
    fn repeated_root_difficulty_and_prospective_variant_skip_arithmetic_and_presentation() {
        let f = crate::test_support::fixture_worker_with_executor(
            crate::executor::RetrievalExecutor::from_test_fixture_factory(1, 16, || {
                Ok(open_source_fixture(
                    vec![record(
                        "actors",
                        "Actor",
                        json!({
                            "_id":"aaaaaaaaaaaaaaaa","name":"Shared root","type":"npc",
                            "system":{"details":{"level":{"value":5}},"attributes":{"adjustment":null,"hp":{"max":60}}}
                        }),
                    )],
                    false,
                )?)
            }),
        );
        let encounter = f
            .worker
            .create_encounter(CreateEncounterRequest {
                name: "Source-only difficulty".into(),
                description: None,
                note: None,
            })
            .unwrap()
            .encounter;
        for variant in [
            EncounterParticipantVariantView::Normal,
            EncounterParticipantVariantView::Elite,
        ] {
            f.worker
                .add_encounter_record_participant(AddEncounterRecordParticipantRequest {
                    encounter_ref: encounter.encounter_key.clone(),
                    record_ref: "actors:aaaaaaaaaaaaaaaa".into(),
                    quantity: 1,
                    initiative: None,
                    participant_variant: Some(variant),
                })
                .unwrap();
        }
        let before = f.worker.encounter(&encounter.encounter_key).unwrap();
        let key = encounter.encounter_key.clone();
        let request = PreviewEncounterDifficultyRequest {
            expected_revision: before.revision,
            reference: EncounterDifficultyReferenceView {
                party_level: Some(5),
                party_size: Some(4),
            },
            addition: Some(EncounterDifficultyAddition {
                record_ref: "actors:aaaaaaaaaaaaaaaa".into(),
                quantity: 1,
                participant_variant: Some(EncounterParticipantVariantView::Weak),
            }),
        };
        let (preview, metrics, authored, baselines) = f
            .worker
            .with_workspace(
                &crate::ReadExecutionContext::uncancellable(),
                move |workspace| {
                    workspace.retrieval().reset_read_metrics();
                    crate::presentation::actor::authored_projection_count(true);
                    super::super::super::mechanics::baseline_count(true);
                    let reference = local_reference(request.reference);
                    let view = read_preview(workspace, &key, request, reference)?;
                    Ok((
                        view,
                        workspace.retrieval().read_metrics(),
                        crate::presentation::actor::authored_projection_count(false),
                        super::super::super::mechanics::baseline_count(false),
                    ))
                },
            )
            .unwrap();
        assert_eq!(metrics.source_body_decodes, 1);
        assert_eq!((authored, baselines), (0, 0));
        assert_eq!(
            preview
                .opposition
                .iter()
                .map(|row| row.level)
                .collect::<Vec<_>>(),
            [Some(5), Some(6), Some(4)]
        );
        assert_eq!(
            preview
                .opposition
                .iter()
                .map(|row| row.xp)
                .collect::<Vec<_>>(),
            [Some(40), Some(60), Some(30)]
        );
        assert_eq!(preview.total_xp, Some(130));
        assert_eq!(
            f.worker.encounter(&encounter.encounter_key).unwrap(),
            before
        );
    }
}
