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
            return Ok(project(
                reference,
                &initial.participants,
                &Default::default(),
            ));
        }
        let encounter_ref = encounter_ref.to_owned();
        // One workspace snapshot owns saved state and all source-backed XP inputs.
        self.with_workspace(
            &crate::ReadExecutionContext::uncancellable(),
            move |session| {
                let snapshot = session
                    .local_state()
                    .encounters()
                    .get_with_participants(&encounter_ref)?
                    .ok_or_else(|| super::super::projection::encounter_not_found(&encounter_ref))?;
                guard_revision(&snapshot.encounter, request.expected_revision)?;
                let (_, records) = super::super::hydration::hydrate_participant_records(
                    session.retrieval_mut(),
                    &snapshot.participants,
                )?;
                let mut view = project(reference, &snapshot.participants, &records);
                if let Some(addition) = request.addition {
                    let record = super::super::hydration::resolve_record_ref(
                        session.retrieval_mut(),
                        &addition.record_ref,
                    )?;
                    let prepared = super::super::record_addition::prepare(
                        &record,
                        addition.participant_variant,
                    )?;
                    let baseline = super::super::hydration::source_baseline(
                        &record.source,
                        session.retrieval_mut(),
                        None,
                    )?;
                    let source = atlas_record::source_record::SourceQueryView::new(
                        record.source.source(),
                        record.source.key().pack().as_str(),
                        "",
                    );
                    let level = super::super::rules::effective_level(
                        prepared.level_override,
                        prepared.participant_variant,
                        prepared.variant_origin,
                        baseline.as_ref().and_then(|b| b.level.as_ref()),
                        !matches!(
                            source.actor().authored_adjustment(),
                            atlas_record::source_record::SourceFieldView::NotApplicable
                        ),
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
            },
        )
    }
}
