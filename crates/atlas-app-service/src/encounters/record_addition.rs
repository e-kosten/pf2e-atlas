//! Source/default preparation shared by addition and its read-only difficulty preview.
use super::mechanics::{effective_max, record_add_variant};
use super::projection::participant_variant;
use crate::{AppServiceError, AppServiceResult};
use atlas_app_model::EncounterParticipantVariantView;
use atlas_domain::RecordKind;
use atlas_local_state::{AddEncounterParticipant, ParticipantKind, ParticipantSide};
use atlas_search::SourceRecordDetail;

pub(super) fn validate_quantity(quantity: u32) -> AppServiceResult<()> {
    if quantity == 0 || quantity > 50 {
        return Err(AppServiceError::invalid_request(
            "quantity must be between 1 and 50",
        ));
    }
    Ok(())
}
pub(super) fn prepare(
    record: &SourceRecordDetail,
    variant: Option<EncounterParticipantVariantView>,
) -> AppServiceResult<AddEncounterParticipant> {
    let participant_kind = match record.summary.record_kind.as_value().copied() {
        Some(RecordKind::Creature) => ParticipantKind::Creature,
        Some(RecordKind::Hazard) => ParticipantKind::Hazard,
        _ => {
            return Err(AppServiceError::invalid_request(
                "encounter participants must be creatures, hazards, or manually named PCs",
            ));
        }
    };
    let side = if participant_kind == ParticipantKind::Hazard {
        ParticipantSide::Hazard
    } else {
        ParticipantSide::Enemy
    };
    if variant.is_some() && participant_kind != ParticipantKind::Creature {
        return Err(AppServiceError::invalid_request(
            "Elite and Weak adjustments apply only to creatures.",
        ));
    }
    use atlas_local_state::{ParticipantVariant, ParticipantVariantOrigin};
    let inherited = record_add_variant(&record.source);
    let (variant, variant_origin) = match variant {
        Some(variant) => (
            participant_variant(variant),
            ParticipantVariantOrigin::Explicit,
        ),
        None => inherited,
    };
    let max_hp = effective_max(&record.source, Some(variant));
    let title = record
        .summary
        .name
        .as_value()
        .cloned()
        .unwrap_or_else(|| record.summary.key.to_string());
    let prefix = |variant| match variant {
        ParticipantVariant::Normal => None,
        ParticipantVariant::Elite => Some("Elite"),
        ParticipantVariant::Weak => Some("Weak"),
    };
    let name = match (inherited.0, title.split_once(char::is_whitespace)) {
        (source, Some((first, rest)))
            if source != variant
                && prefix(source).is_some_and(|marker| first.eq_ignore_ascii_case(marker))
                && !rest.trim_start().is_empty() =>
        {
            rest.trim_start()
        }
        _ => &title,
    };
    let display_name = match prefix(variant) {
        Some(prefix)
            if !name
                .split_whitespace()
                .next()
                .is_some_and(|word| word.eq_ignore_ascii_case(prefix)) =>
        {
            format!("{prefix} {name}")
        }
        _ => name.to_owned(),
    };
    let current_hp = max_hp;
    Ok(AddEncounterParticipant {
        level_override: None,
        spell_resources: super::spell_catalog::initial_resources(&record.source),
        participant_variant: variant,
        variant_origin,
        hp_origin: if max_hp.is_some() {
            atlas_local_state::ParticipantHpOrigin::DerivedPristine
        } else {
            atlas_local_state::ParticipantHpOrigin::Unknown
        },
        record_key: Some(record.summary.key.clone()),
        participant_kind,
        display_name,
        record_title_snapshot: Some(title),
        record_kind_snapshot: Some(
            record
                .summary
                .record_kind
                .as_value()
                .map(|k| k.as_str().to_owned())
                .unwrap_or_default(),
        ),
        side,
        initiative: None,
        max_hp,
        current_hp,
        temporary_hp: 0,
        note: None,
    })
}

/// The selected published stat block carries its own default adjustment.
pub(super) fn published_replacement(
    retrieval: &atlas_search::AtlasRetrievalService,
    record: &SourceRecordDetail,
    adjustment: Option<EncounterParticipantVariantView>,
    scenario: Option<&atlas_app_model::CreatureScenarioContext>,
) -> AppServiceResult<Option<SourceRecordDetail>> {
    if let Some(scenario) = scenario {
        let generation = retrieval.artifact_generation()?;
        if scenario.artifact_generation != generation {
            return Err(AppServiceError::artifact_generation_changed(
                &scenario.artifact_generation,
                &generation,
                atlas_app_model::AppRecoverableAction::ReloadRecord,
            ));
        }
    }
    let adjustment = match adjustment {
        Some(EncounterParticipantVariantView::Weak) => atlas_search::CreatureAdjustment::Weak,
        Some(EncounterParticipantVariantView::Elite) => atlas_search::CreatureAdjustment::Elite,
        _ => return Ok(None),
    };
    let use_id = scenario.and_then(|scenario| scenario.use_id.as_deref());
    let Some(key) = retrieval.creature_adjustment_override(&record.summary, adjustment, use_id)?
    else {
        return Ok(None);
    };
    Ok(retrieval.get_record(atlas_search::GetRecordRequest {
        record_key: &key,
        selected_content: &[],
    })?)
}

pub(super) fn prepared_choice(
    participant: &AddEncounterParticipant,
) -> Option<atlas_app_model::PreparedCreatureChoice> {
    let record_key = participant.record_key.as_ref()?.to_string();
    let initial_adjustment =
        super::projection::participant_variant_view(participant.participant_variant);
    Some(atlas_app_model::PreparedCreatureChoice {
        record_key,
        initial_adjustment,
    })
}
