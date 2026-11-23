//! Terminal and JSON encounter reads over the shared encounter projection.
use super::{Detail, PartyReference, review::Commands};
use crate::commands::followup::shell_command;
use crate::commands::saved_workspace::write_text;
use atlas_app_model::{
    EncounterAbilityStateKindView, EncounterDetailView, EncounterDifficultyComparison,
    EncounterDifficultyView, EncounterParticipantSideView, EncounterParticipantStatusView,
    EncounterParticipantView, EncounterRecordAdditionResult, EncounterSummaryView,
    MortalityStateView,
};
use atlas_cli_support::write_json_data;
use serde::Serialize;
use serde_json::{Value, json};
use std::fmt::{Display, Write};

pub(super) fn turn_progress(encounter: &EncounterSummaryView) -> Option<String> {
    let round = encounter.round_number;
    match &encounter.current_turn_participant_name {
        Some(name) => Some(format!("Round {round} · {name}’s turn")),
        None if round > 1 => Some(format!("Round {round}")),
        None => None,
    }
}

pub(super) fn added(result: EncounterRecordAdditionResult, json: bool) -> Result<(), String> {
    let outcome = match &result.participant_keys {
        Some(_) => "added",
        None => "already_committed",
    };
    if json {
        return write_json_data(json!({
            "outcome": outcome, "encounter_key": result.encounter_key, "request_id": result.request_id,
            "participant_keys": result.participant_keys,
        }));
    }
    let mut text = match result.participant_keys {
        Some(keys) => format!(
            "Added to {}\n  {}\n",
            result.encounter_key,
            keys.join("\n  ")
        ),
        None => format!(
            "Already added to {}. Original participant keys are unavailable on replay.\n",
            result.encounter_key
        ),
    };
    let _ = writeln!(text, "Request {}", result.request_id);
    write_text(&text)
}

#[derive(Serialize)]
struct Difficulty {
    #[serde(flatten)]
    view: EncounterDifficultyView,
    party_level_source: &'static str,
    party_size_source: &'static str,
    #[serde(skip_serializing_if = "Option::is_none")]
    roster_suggestion: Option<RosterSuggestion>,
}

#[derive(Serialize)]
struct RosterSuggestion {
    party_level: u32,
    basis: String,
}

fn input_source(
    explicit: Option<u32>,
    reference: Option<u32>,
    effective: Option<u32>,
) -> &'static str {
    if explicit.is_some() {
        "preview"
    } else if reference.is_some() {
        "saved"
    } else if effective.is_some() {
        "roster"
    } else {
        "unknown"
    }
}

impl Difficulty {
    fn new(mut view: EncounterDifficultyView, party: PartyReference) -> Self {
        let party_level_source = input_source(
            party.party_level,
            view.reference.party_level,
            view.party_level,
        );
        let party_size_source =
            input_source(party.party_size, view.reference.party_size, view.party_size);
        let roster_suggestion =
            if let (Some(level), Some(_)) = (view.party_level, view.reference.party_level) {
                let basis = std::mem::replace(
                    &mut view.basis,
                    format!("Using {party_level_source} party level {level}."),
                );
                view.roster_suggested_level
                    .map(|party_level| RosterSuggestion { party_level, basis })
            } else {
                None
            };
        Self {
            view,
            party_level_source,
            party_size_source,
            roster_suggestion,
        }
    }

    fn render_reference(&self, text: &mut String) {
        let view = &self.view;
        let _ = writeln!(
            text,
            "Party level {} ({}) · size {} ({})",
            display(view.party_level),
            self.party_level_source,
            display(view.party_size),
            self.party_size_source
        );
        if self.party_level_source != "preview" && self.party_level_source != "saved" {
            let _ = writeln!(text, "{}", view.basis);
        }
        if let Some(suggestion) = &self.roster_suggestion {
            let _ = writeln!(
                text,
                "Roster suggestion · level {}. {}",
                suggestion.party_level, suggestion.basis
            );
        }
        if let Some(note) = &view.reference_note {
            let _ = writeln!(text, "{note}");
        }
    }

    fn render_xp(&self, text: &mut String) {
        let view = &self.view;
        match (view.rating.as_deref(), view.total_xp) {
            (Some(rating), Some(xp)) => {
                let _ = writeln!(text, "Difficulty: {rating} · {xp} opposition XP");
            }
            _ => {
                let _ = writeln!(
                    text,
                    "Difficulty incomplete · {} known opposition XP",
                    view.known_xp
                );
            }
        }
        let _ = writeln!(
            text,
            "Suggested award: {} XP per PC",
            display(view.xp_per_pc)
        );
        if let Some(note) = &view.award_note {
            let _ = writeln!(text, "{note}");
        }
    }

    fn render(&self, text: &mut String) {
        self.render_reference(text);
        self.render_xp(text);
        let view = &self.view;
        for budget in &view.budgets {
            let _ = writeln!(text, "  {} budget: {} XP", budget.label, budget.xp);
        }
        for opponent in &view.opposition {
            write_opponent(text, opponent);
        }
    }
}

fn write_opponent(text: &mut String, opponent: &atlas_app_model::EncounterDifficultyOpponentView) {
    let result = opponent
        .xp
        .map(|xp| format!("{xp} XP"))
        .or_else(|| opponent.unavailable_reason.clone())
        .unwrap_or_else(|| "XP unknown".into());
    let _ = writeln!(text, "  {}: {result}", opponent.name);
}

fn display<T: Display>(value: Option<T>) -> String {
    value.map_or_else(|| "unknown".into(), |v| v.to_string())
}

pub(super) fn comparison(
    comparison: EncounterDifficultyComparison,
    party: PartyReference,
    commands: Result<Commands, String>,
    json: bool,
) -> Result<(), String> {
    let current = Difficulty::new(comparison.current, party);
    let projected = Difficulty::new(comparison.projected, party);
    if json {
        let mut data = json!({
            "encounter_key": comparison.encounter_key, "revision": comparison.revision,
            "artifact_generation": comparison.artifact_generation, "current": current, "projected": projected,
        });
        match commands {
            Ok(commands) => {
                data["add_args"] = json!(commands.add);
                if let Some(repeat) = commands.repeat_preview {
                    data["repeat_preview_args"] = json!(repeat);
                }
            }
            Err(reason) => data["followup_unavailable"] = json!(reason),
        }
        return write_json_data(data);
    }
    let mut text = format!(
        "{} · revision {}\n",
        comparison.encounter_key, comparison.revision
    );
    current.render_reference(&mut text);
    text.push_str("\nCurrent\n");
    current.render_xp(&mut text);
    text.push_str("\nWith proposed addition\n");
    projected.render_xp(&mut text);
    for opponent in &projected.view.opposition {
        if opponent.participant_key.is_none() || opponent.xp.is_none() {
            write_opponent(&mut text, opponent);
        }
    }
    match commands {
        Ok(commands) => {
            if let Some(repeat) = commands.repeat_preview {
                let _ = writeln!(text, "\nRepeat preview: {}", shell_command(&repeat));
            }
            let _ = writeln!(text, "\nAdd as reviewed: {}", shell_command(&commands.add));
        }
        Err(reason) => {
            let _ = writeln!(text, "{reason}");
        }
    }
    write_text(&text)
}

pub(super) fn show(
    view: EncounterDetailView,
    party: PartyReference,
    detail: Detail,
    selected: Option<&str>,
    json: bool,
) -> Result<(), String> {
    let participants = view
        .participants
        .iter()
        .filter(|p| selected.is_none_or(|key| p.participant_key == key));
    let expanded = detail == Detail::Full || selected.is_some();
    let difficulty = Difficulty::new(view.difficulty, party);
    if json {
        let rows = participants
            .map(|p| {
                if expanded {
                    serde_json::to_value(p).map_err(|e| e.to_string())
                } else {
                    Ok(compact(p))
                }
            })
            .collect::<Result<Vec<_>, _>>()?;
        return write_json_data(json!({
            "encounter": view.encounter, "revision": view.revision, "note": view.note,
            "artifact_generation": view.artifact_generation, "reference_error": view.reference_error,
            "current_turn_participant_key": view.current_turn_participant_key,
            "difficulty": difficulty, "participants": rows,
        }));
    }
    let mut text = format!(
        "{} ({}){}\n{} · revision {}\n",
        view.encounter.name,
        view.encounter.slug,
        if view.encounter.archived {
            " · Archived"
        } else {
            ""
        },
        view.encounter.encounter_key,
        view.revision
    );
    if let Some(description) = &view.encounter.description {
        let _ = writeln!(text, "{description}");
    }
    if let Some(note) = &view.note {
        let _ = writeln!(text, "GM note: {note}");
    }
    if let Some(progress) = turn_progress(&view.encounter) {
        let _ = writeln!(text, "{progress}");
    }
    if let Some(error) = &view.reference_error {
        let _ = writeln!(text, "Reference data: {}", error.message);
    }
    if view.participants.is_empty() {
        text.push_str("Empty roster.\n");
    }
    for p in participants {
        participant_text(
            &mut text,
            p,
            view.current_turn_participant_key.as_deref() == Some(&p.participant_key),
            expanded,
        )?;
    }
    text.push('\n');
    difficulty.render(&mut text);
    write_text(&text)
}

fn compact(p: &EncounterParticipantView) -> Value {
    let conditions: Vec<_> = p
        .conditions
        .iter()
        .map(|c| {
            json!({
                "condition_key": c.condition_key, "name": c.name, "value": c.value, "scope": c.scope
            })
        })
        .collect();
    let inherent = p.inherent_conditions.as_ref().map(|c| json!({
        "unavailable_reason": c.unavailable_reason,
        "entries": c.entries.iter().map(|e| json!({"name": e.name, "value": e.value, "source_name": e.source_name, "unavailable_reason": e.unavailable_reason})).collect::<Vec<_>>()
    }));
    json!({
        "participant_key": p.participant_key, "display_name": p.display_name, "revision": p.revision,
        "record_key": p.record_key, "participant_kind": p.participant_kind, "participant_variant": p.participant_variant,
        "side": p.side, "status": p.status, "initiative": p.initiative, "initiative_order": p.initiative_order,
        "current_hp": p.current_hp, "max_hp": p.max_hp, "effective_max_hp": p.effective_max_hp,
        "temporary_hp": p.temporary_hp, "conditions": conditions, "inherent_conditions": inherent,
        "mortality": p.mortality, "defeated": p.defeated, "hidden": p.hidden, "hazard_disabled": p.hazard_disabled,
        "note": p.note,
    })
}

fn condition_scope(scope: &atlas_app_model::ConditionScopeView) -> String {
    match scope {
        atlas_app_model::ConditionScopeView::Everyone => String::new(),
        atlas_app_model::ConditionScopeView::SelectedParticipants { participant_keys } => {
            format!(" against {}", participant_keys.join(", "))
        }
    }
}

fn participant_text(
    text: &mut String,
    p: &EncounterParticipantView,
    current: bool,
    expanded: bool,
) -> Result<(), String> {
    let side = match p.side {
        EncounterParticipantSideView::Pc => "PC",
        EncounterParticipantSideView::Ally => "ally",
        EncounterParticipantSideView::Enemy => "enemy",
        EncounterParticipantSideView::Neutral => "neutral",
        EncounterParticipantSideView::Hazard => "hazard",
    };
    let turn = if current { " · current turn" } else { "" };
    let _ = writeln!(
        text,
        "\n{} · {side}{turn}\n  {}",
        p.display_name, p.participant_key
    );
    let _ = writeln!(
        text,
        "  Initiative {} · HP {}/{}",
        display(p.initiative),
        display(p.current_hp),
        display(p.effective_max_hp.value.as_ref())
    );
    if p.temporary_hp != 0 {
        let _ = writeln!(text, "  Temporary HP {}", p.temporary_hp);
    }
    for (flag, label) in [
        (p.defeated, "Defeated"),
        (p.hazard_disabled, "Disabled"),
        (p.hidden, "Hidden"),
        (p.mortality == MortalityStateView::Dead, "Dead"),
    ] {
        if flag {
            let _ = writeln!(text, "  {label}");
        }
    }
    if matches!(
        p.status,
        EncounterParticipantStatusView::Unchecked | EncounterParticipantStatusView::Unresolved
    ) {
        let _ = writeln!(text, "  Reference {:?}", p.status);
    }
    for c in &p.conditions {
        let value = c.value.map(|v| format!(" {v}")).unwrap_or_default();
        let _ = writeln!(text, "  {}{value}{}", c.name, condition_scope(&c.scope));
    }
    if let Some(inherent) = &p.inherent_conditions {
        for c in &inherent.entries {
            let value = c.value.map(|v| format!(" {v}")).unwrap_or_default();
            let _ = writeln!(text, "  {}{value} · from {}", c.name, c.source_name);
        }
        if let Some(reason) = &inherent.unavailable_reason {
            let _ = writeln!(text, "  {reason}");
        }
    }
    if let Some(note) = &p.note {
        let _ = writeln!(text, "  Note: {note}");
    }
    if expanded {
        expanded_participant(text, p)?;
    }
    Ok(())
}

fn expanded_participant(text: &mut String, p: &EncounterParticipantView) -> Result<(), String> {
    let _ = writeln!(text, "  Revision {}", p.revision);
    if let Some(key) = &p.record_key {
        let _ = writeln!(text, "  Record {key}");
    }
    if let Some(presentation) = &p.presentation {
        text.push_str(&super::super::record::render_encounter_actor(presentation)?);
    }
    if let Some(resources) = &p.turn_resources {
        let _ = writeln!(
            text,
            "  Actions remaining {} · reactions remaining {}",
            display(resources.regular_remaining),
            display(resources.reactions_remaining)
        );
    }
    if let Some(abilities) = &p.ability_states {
        for ability in abilities {
            let state = match &ability.state {
                EncounterAbilityStateKindView::Uses {
                    remaining,
                    maximum,
                    interval,
                } => format!("{remaining}/{maximum} uses ({interval})"),
                EncounterAbilityStateKindView::Option {
                    active, selection, ..
                } => {
                    let status = if *active { "active" } else { "inactive" };
                    selection.as_ref().map_or_else(
                        || status.to_owned(),
                        |choice| format!("{status} · {choice}"),
                    )
                }
            };
            let _ = writeln!(text, "  {} · {state}", ability.label);
        }
    }
    if let Some(casting) = &p.spellcasting {
        for resource in &casting.resources {
            let _ = writeln!(
                text,
                "  {} · {}/{} remaining",
                resource.label, resource.remaining, resource.capacity
            );
        }
        for spell in &casting.spells {
            let ranks = spell
                .ranks
                .iter()
                .map(|r| r.rank.to_string())
                .collect::<Vec<_>>()
                .join(", ");
            let _ = writeln!(text, "  {} · ranks {ranks}", spell.title);
        }
    }
    for group in &p.persistent_damage_groups {
        for source in &group.sources {
            let _ = writeln!(
                text,
                "  Persistent {} · {} ({:?}) · recovery DC {}",
                group.damage_type,
                source.formula,
                source.multiplier,
                display(source.recovery_dc)
            );
        }
        if let Some(note) = &group.immunity_note {
            let _ = writeln!(text, "  {note}");
        }
    }
    for tracker in &p.manual_trackers {
        let _ = writeln!(text, "  {} · {}", tracker.name, tracker.timing_label);
    }
    if let Some(adjustments) = &p.gm_adjustments {
        for adjustment in &adjustments.entries {
            let _ = writeln!(
                text,
                "  {} · {} {:+}",
                adjustment.name, adjustment.label, adjustment.amount
            );
        }
    }
    Ok(())
}
