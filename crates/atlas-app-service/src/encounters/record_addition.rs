//! Source/default preparation shared by addition and its read-only difficulty preview.
use super::mechanics::{effective_max, initial_variant};
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
    let (chosen, variant_origin) = match variant {
        Some(variant) => (
            Some(participant_variant(variant)),
            atlas_local_state::ParticipantVariantOrigin::Explicit,
        ),
        None => initial_variant(&record.source),
    };
    let variant = chosen.unwrap_or(atlas_local_state::ParticipantVariant::Normal);
    let max_hp = effective_max(&record.source, chosen);
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
        display_name: record
            .summary
            .name
            .as_value()
            .cloned()
            .unwrap_or_else(|| record.summary.key.to_string()),
        record_title_snapshot: Some(
            record
                .summary
                .name
                .as_value()
                .cloned()
                .unwrap_or_else(|| record.summary.key.to_string()),
        ),
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
