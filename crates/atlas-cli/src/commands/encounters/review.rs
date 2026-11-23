//! Follow-up invocations scoped to the encounter and reference snapshot a GM reviewed.
use super::{Opponent, PartyReference};
use crate::commands::{followup, product::report_app_error};
use atlas_app_model::{AppError, EncounterDifficultyComparison};
use atlas_cli_support::CliPathMode;
use clap::ValueEnum;
use std::{path::Path, process::ExitCode};

pub(super) fn request_id() -> String {
    uuid::Builder::from_random_bytes(rand::random())
        .into_uuid()
        .to_string()
}

pub(super) fn workspace_args(
    mode: CliPathMode,
    index: Option<&Path>,
    json: bool,
) -> Result<Vec<String>, String> {
    let mut args = followup::workspace_args(mode, index)?;
    if json {
        args.push("--json".into());
    }
    Ok(args)
}

impl Opponent {
    fn command(
        &self,
        action: &str,
        encounter: &str,
        record: &str,
        workspace: &[String],
        flags: &[String],
    ) -> Vec<String> {
        let mut args = vec![
            "atlas".into(),
            "encounters".into(),
            action.into(),
            format!("--record={record}"),
            format!("--quantity={}", self.quantity),
        ];
        if let Some(value) = self
            .adjustment
            .and_then(|adjustment| adjustment.to_possible_value())
        {
            args.push(format!("--adjustment={}", value.get_name()));
        }
        args.extend_from_slice(workspace);
        args.extend_from_slice(flags);
        args.extend(["--".into(), encounter.into()]);
        args
    }
}

pub(super) struct Commands {
    pub add: Vec<String>,
    pub repeat_preview: Option<Vec<String>>,
}

impl Commands {
    pub fn new(
        comparison: &EncounterDifficultyComparison,
        opponent: &Opponent,
        party: PartyReference,
        workspace: &[String],
    ) -> Self {
        let command = |action, flags: &[String]| {
            opponent.command(
                action,
                &comparison.encounter_key,
                &comparison.record_key,
                workspace,
                flags,
            )
        };
        let add = command(
            "add",
            &[
                format!("--expected-revision={}", comparison.revision),
                format!(
                    "--expected-artifact-generation={}",
                    comparison.artifact_generation
                ),
                format!("--request-id={}", request_id()),
            ],
        );
        let mut overrides = Vec::new();
        if let Some(level) = party.party_level {
            overrides.push(format!("--party-level={level}"));
        }
        if let Some(size) = party.party_size {
            overrides.push(format!("--party-size={size}"));
        }
        let repeat_preview = (!overrides.is_empty()).then(|| command("preview", &overrides));
        Self {
            add,
            repeat_preview,
        }
    }
}

pub(super) fn report_stale(
    mut error: AppError,
    encounter: &str,
    opponent: &Opponent,
    workspace: Result<Vec<String>, String>,
    json: bool,
) -> Result<ExitCode, String> {
    let preview =
        workspace.map(|args| opponent.command("preview", encounter, &opponent.record, &args, &[]));
    if json {
        let details = error.details.get_or_insert_with(|| serde_json::json!({}));
        match preview {
            Ok(args) => details["preview_args"] = serde_json::json!(args),
            Err(reason) => details["followup_unavailable"] = serde_json::json!(reason),
        }
        return report_app_error(error, true);
    }
    let exit = report_app_error(error, false)?;
    match preview {
        Ok(args) => eprintln!("Preview again: {}", followup::shell_command(&args)),
        Err(reason) => eprintln!("{reason}"),
    }
    Ok(exit)
}
