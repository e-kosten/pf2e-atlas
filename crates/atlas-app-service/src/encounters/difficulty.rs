//! Planned encounter budgets. Rules: GM Core 56–57 and 75–76 (AoN 2649/2653/2716).
//! Defeated/disabled state never changes the planned opposition or party.
mod preview;
use crate::{AppServiceResult, AtlasAppService};
use atlas_app_model::{
    EncounterDifficultyBudgetView, EncounterDifficultyOpponentView,
    EncounterDifficultyReferenceView, EncounterDifficultyView, EncounterMutationView,
    SetEncounterDifficultyRequest,
};
use atlas_local_state::{
    EncounterDifficultyReference, EncounterParticipant, ParticipantKind, ParticipantSide,
};

impl AtlasAppService {
    pub fn set_encounter_difficulty(
        &self,
        encounter_ref: &str,
        request: SetEncounterDifficultyRequest,
    ) -> AppServiceResult<EncounterMutationView> {
        let encounter_ref = encounter_ref.to_owned();
        self.with_write(move |session| {
            let encounter = session
                .local_state()
                .encounters()
                .set_difficulty_reference(
                    &encounter_ref,
                    request.expected_revision,
                    local_reference(request.reference),
                )?;
            Ok(EncounterMutationView {
                encounter_key: encounter.encounter_key,
            })
        })
    }
}
fn local_reference(reference: EncounterDifficultyReferenceView) -> EncounterDifficultyReference {
    EncounterDifficultyReference {
        party_level: reference.party_level,
        party_size: reference.party_size,
    }
}

pub(super) fn project<'a>(
    reference: EncounterDifficultyReference,
    participants: &[EncounterParticipant],
    record_for_key: impl Fn(&str) -> Option<&'a atlas_search::SourceRecordDetail>,
) -> EncounterDifficultyView {
    let pcs = participants
        .iter()
        .filter(|p| p.participant_kind == ParticipantKind::Pc && p.side == ParticipantSide::Pc)
        .collect::<Vec<_>>();
    let levels = pcs
        .iter()
        .map(|p| {
            p.level_override
                .and_then(|l| u32::try_from(l).ok())
                .filter(|l| (1..=20).contains(l))
        })
        .collect::<Option<Vec<_>>>();
    let (suggested, basis) = suggest_level(levels.as_deref());
    let party_level = reference.party_level.or(suggested);
    let pc_count = u32::try_from(pcs.len()).unwrap_or(u32::MAX);
    let bonus = levels
        .as_deref()
        .zip(party_level)
        .map_or(0, |(levels, level)| outlier_bonus(levels, level));
    let party_size = reference
        .party_size
        .or_else(|| (pc_count > 0).then_some(pc_count.saturating_add(bonus)));
    let budgets = party_size.map(budgets).unwrap_or_default();
    let reference_note = (reference.party_size.is_none() && bonus > 0).then(|| {
        format!(
            "{pc_count} PCs; budget counts as {} for the higher-level PC.",
            pc_count + bonus
        )
    });
    let opposition = participants
        .iter()
        .filter(|p| match p.participant_kind {
            ParticipantKind::Creature => p.side == ParticipantSide::Enemy,
            ParticipantKind::Hazard => {
                matches!(p.side, ParticipantSide::Hazard | ParticipantSide::Enemy)
            }
            ParticipantKind::Pc => false,
        })
        .map(|p| {
            let record = p.record_key.as_deref().and_then(&record_for_key);
            let source = record.map(|record| {
                atlas_record::source_record::SourceQueryView::new(
                    record.source.source(),
                    record.source.key().pack().as_str(),
                    "",
                )
            });
            let level = super::rules::effective_participant_level(
                p,
                source
                    .as_ref()
                    .and_then(|source| source.actor().level().value()),
                record
                    .is_some_and(|record| super::hydration::adjustment_applicable(&record.source)),
            )
            .as_ref()
            .and_then(super::mechanics::integer)
            .and_then(|l| i32::try_from(l).ok());
            let complexity = source.and_then(|source| source.actor().hazard_complexity().value());
            opponent(
                Some(p.participant_key.clone()),
                p.display_name.clone(),
                p.participant_kind,
                level,
                complexity,
                party_level,
                1,
            )
        })
        .collect::<Vec<_>>();
    let mut view = EncounterDifficultyView {
        rules: rule_references(),
        reference: EncounterDifficultyReferenceView {
            party_level: reference.party_level,
            party_size: reference.party_size,
        },
        party_level,
        party_size,
        pc_count,
        basis,
        reference_note,
        total_xp: None,
        known_xp: 0,
        rating: None,
        xp_per_pc: None,
        award_note: None,
        budgets,
        opposition,
    };
    recalculate(&mut view, participants);
    view
}

// A prospective row has no participant identity and is never written to saved state.
fn opponent(
    participant_key: Option<String>,
    name: String,
    kind: ParticipantKind,
    level: Option<i32>,
    complexity: Option<bool>,
    party_level: Option<u32>,
    quantity: u32,
) -> EncounterDifficultyOpponentView {
    let result = match (party_level, level) {
        (None, _) => Err("Set a party level to calculate XP."),
        (_, None) => Err("Participant level is unavailable."),
        (Some(party), Some(level)) => level
            .checked_sub(party as i32)
            .ok_or("Outside the party level −4 to +4 XP table.")
            .and_then(|relative| xp(relative, kind == ParticipantKind::Hazard, complexity))
            .and_then(|xp| xp.checked_mul(quantity).ok_or("XP total is too large.")),
    };
    let (xp, unavailable_reason) = match result {
        Ok(xp) => (Some(xp), None),
        Err(reason) => (None, Some(reason.into())),
    };
    EncounterDifficultyOpponentView {
        participant_key,
        name: if quantity == 1 {
            name
        } else {
            format!("{quantity} × {name}")
        },
        kind: match (kind, complexity) {
            (ParticipantKind::Creature, _) => "Creature",
            (_, Some(true)) => "Complex hazard",
            (_, Some(false)) => "Simple hazard",
            _ => "Hazard",
        }
        .into(),
        level,
        xp,
        unavailable_reason,
    }
}
fn recalculate(view: &mut EncounterDifficultyView, participants: &[EncounterParticipant]) {
    view.known_xp = view
        .opposition
        .iter()
        .filter_map(|o| o.xp)
        .fold(0, u32::saturating_add);
    view.total_xp = view
        .opposition
        .iter()
        .map(|o| o.xp)
        .try_fold(0u32, |sum, xp| sum.checked_add(xp?));
    view.rating = view
        .total_xp
        .zip(view.party_size)
        .map(|(xp, _)| rating(xp, &view.budgets));
    // Pinned PF2e calculateXP: src/scripts/macros/xp/index.ts. Keep normalization
    // separate from the published budget adjustments (notably Low's +20).
    view.xp_per_pc = view
        .total_xp
        .zip(view.party_size)
        .filter(|(_, size)| *size > 0)
        .filter(|_| view.party_level.is_some())
        .and_then(|(xp, size)| u32::try_from(u64::from(xp) * 4 / u64::from(size)).ok());
    view.award_note = view.party_size.map(|size| {
        let mut note = format!("Encounter XP × 4 ÷ {size}, rounded down.");
        if let Some(party) = view.party_level
            && participants.iter().any(|p| {
                p.participant_kind == ParticipantKind::Pc
                    && p.side == ParticipantSide::Pc
                    && p.level_override
                        .is_some_and(|level| level < i64::from(party))
            })
        {
            note.push_str(&format!(" PCs below level {party} earn double."));
        }
        note
    });
    if view.rating.as_deref() == Some("Trivial") {
        view.xp_per_pc = view.party_level.map(|_| 0);
        view.award_note = Some("Trivial encounters normally grant no XP.".into());
    }
}

// Pinned Foundry GM Screen pages; stable owned identities preserve native links.
// Navigation is optional reading, not an assertion that a partial corpus contains it.
fn rule_references() -> Vec<atlas_app_model::EncounterDifficultyRuleView> {
    [
        ("Encounter Budget", "WZgFEMDSP9TvYc4u"),
        ("XP Awards", "ZcddTU02joqYuuoV"),
    ]
    .into_iter()
    .map(|(title, id)| atlas_app_model::EncounterDifficultyRuleView {
        title: title.into(),
        navigation: atlas_app_model::RecordNavigationView {
            spell_rank: None,
            record_key: "journals:S55aqwWIzpQRFhcq".into(),
            owners: vec![atlas_domain::OwnedContentLocator {
                collection: "/pages".into(),
                identity: atlas_domain::OwnedContentIdentity::Stable(id.into()),
            }],
            field: None,
            passage: None,
            source_fingerprint: None,
        },
    })
    .collect()
}

fn suggest_level(levels: Option<&[u32]>) -> (Option<u32>, String) {
    let Some(levels) = levels.filter(|levels| !levels.is_empty()) else {
        return (None, "Add PC levels or choose a party reference.".into());
    };
    let highest = levels.iter().copied().max().unwrap_or(1);
    let lowest = levels.iter().copied().min().unwrap_or(1);
    if lowest == highest {
        return (Some(highest), "From encounter PC levels.".into());
    }
    if outlier_bonus(levels, lowest) > 0 {
        return (
            Some(lowest),
            "Lower group level, with extra budget for one higher-level PC.".into(),
        );
    }
    let behind = levels.iter().filter(|&&l| l < highest).count();
    if behind <= 2 && levels.len() - behind >= 2 {
        return (
            Some(highest),
            format!("Suggested level {highest} · highest PC level."),
        );
    }
    let sum = levels.iter().map(|&l| u64::from(l)).sum::<u64>();
    let count = levels.len() as u64;
    let average = ((sum + count / 2) / count) as u32;
    (
        Some(average),
        format!("Suggested level {average} · rounded party average."),
    )
}
fn outlier_bonus(levels: &[u32], reference: u32) -> u32 {
    // Apply only the unambiguous GM Core case: one outlier above a uniform group.
    if levels.len() < 2 {
        return 0;
    }
    let mut higher = levels.iter().copied().filter(|&l| l != reference);
    match (higher.next(), higher.next()) {
        (Some(level), None) => level.saturating_sub(reference) / 2,
        _ => 0,
    }
}
fn xp(relative: i32, hazard: bool, complex: Option<bool>) -> Result<u32, &'static str> {
    if hazard && complex.is_none() {
        return Err("Hazard complexity is unavailable.");
    }
    if relative < -4 {
        // Match pinned Foundry's creature XP floor below the table. Hazards keep
        // their existing below-range policy; unknown complexity was checked above.
        return Ok(if hazard { 0 } else { 10 });
    }
    let value = match relative {
        -4 => 10,
        -3 => 15,
        -2 => 20,
        -1 => 30,
        0 => 40,
        1 => 60,
        2 => 80,
        3 => 120,
        4 => 160,
        _ => return Err("Outside the party level −4 to +4 XP table."),
    };
    Ok(if hazard && complex == Some(false) {
        value / 5
    } else {
        value
    })
}
fn budgets(size: u32) -> Vec<EncounterDifficultyBudgetView> {
    // GM Core's Low adjustment is 20, not the proportional 15 used by some tools.
    [
        ("Trivial", 40, 10),
        ("Low", 60, 20),
        ("Moderate", 80, 20),
        ("Severe", 120, 30),
        ("Extreme", 160, 40),
    ]
    .into_iter()
    .map(|(label, base, adjustment)| EncounterDifficultyBudgetView {
        label: label.into(),
        xp: base + (size as i32 - 4) * adjustment,
    })
    .collect()
}
fn rating(xp: u32, budgets: &[EncounterDifficultyBudgetView]) -> String {
    budgets
        .iter()
        .find(|b| i64::from(xp) <= i64::from(b.xp))
        .map(|b| b.label.clone())
        .unwrap_or_else(|| "Above Extreme".into())
}

fn guard_revision(
    encounter: &atlas_local_state::Encounter,
    expected_revision: i64,
) -> AppServiceResult<()> {
    if encounter.revision != expected_revision {
        return Err(atlas_local_state::LocalStateError::StaleEncounterRevision {
            encounter_key: encounter.encounter_key.clone(),
            expected_revision,
            actual_revision: encounter.revision,
        }
        .into());
    }
    Ok(())
}
#[cfg(test)]
mod tests;
