use super::product::{app_result, report_app_error};
use crate::client::{ClientOptions, connect};
use atlas_app_model::{
    AddEncounterRecordParticipantRequest, CreateEncounterRequest, EncounterDetailView,
    EncounterParticipantVariantView, PreviewEncounterDifficultyRequest,
};
use atlas_cli_support::{CliPathMode, write_json_data};
use clap::{ArgGroup, Args, Subcommand, ValueEnum};
use std::{
    fs::File,
    io::{self, Write},
    path::{Path, PathBuf},
    process::ExitCode,
};

#[derive(Debug, Args)]
pub(crate) struct EncountersArgs {
    #[arg(long, global = true)]
    pub(crate) json: bool,
    #[arg(long, global = true, help = "Override the SQLite workspace")]
    index: Option<PathBuf>,
    #[arg(long, global = true, value_enum, default_value_t = CliPathMode::Global)]
    path_mode: CliPathMode,
    #[command(subcommand)]
    command: EncounterCommand,
}

#[derive(Debug, Subcommand)]
enum EncounterCommand {
    #[command(alias = "ls", about = "List saved encounters")]
    List,
    #[command(about = "Show the roster, difficulty, and suggested XP award")]
    Show {
        encounter: String,
        #[arg(long, help = "Preview difficulty at this party level")]
        party_level: Option<u32>,
        #[arg(long, help = "Preview difficulty for this number of PCs")]
        party_size: Option<u32>,
    },
    #[command(about = "Create a saved encounter")]
    Create {
        name: String,
        #[arg(long)]
        description: Option<String>,
        #[arg(long)]
        note: Option<String>,
    },
    /// Edit saved encounter metadata.
    Edit(EditOptions),
    /// Remove a participant by the key shown in encounters show.
    Remove {
        encounter: String,
        #[arg(long)]
        participant: String,
    },
    /// Delete a saved encounter and its participants.
    Delete { encounter: String },
    /// Export a complete saved encounter as a versioned JSON document.
    Export {
        encounter: String,
        #[arg(long, help = "Write a new file instead of stdout")]
        output: Option<PathBuf>,
    },
    /// Import a portable JSON document as a new encounter.
    Import {
        input: PathBuf,
        #[arg(long, help = "Choose a different destination slug")]
        slug: Option<String>,
    },
    #[command(about = "Add a creature or hazard by record key or exact name")]
    Add {
        encounter: String,
        #[arg(long, add = clap_complete::ArgValueCompleter::new(crate::commands::completions::record_keys))]
        record: String,
        #[arg(long, default_value_t = 1, value_parser = clap::value_parser!(u32).range(1..=50))]
        quantity: u32,
        #[arg(long, value_enum)]
        adjustment: Option<Adjustment>,
        #[arg(long, allow_negative_numbers = true)]
        initiative: Option<i64>,
    },
}

#[derive(Debug, Args)]
#[command(group(ArgGroup::new("changes").required(true).multiple(true)
    .args(["name", "description", "clear_description", "note", "clear_note"])))]
struct EditOptions {
    encounter: String,
    #[arg(long)]
    name: Option<String>,
    #[arg(long)]
    description: Option<String>,
    #[arg(long, conflicts_with = "description")]
    clear_description: bool,
    #[arg(long)]
    note: Option<String>,
    #[arg(long, conflicts_with = "note")]
    clear_note: bool,
}

#[derive(Clone, Copy, Debug, ValueEnum)]
enum Adjustment {
    Normal,
    Elite,
    Weak,
}

pub(crate) fn run(args: EncountersArgs) -> Result<ExitCode, String> {
    let json = args.json;
    let retrieval_mode = if matches!(
        &args.command,
        EncounterCommand::Export { .. } | EncounterCommand::Import { .. }
    ) {
        atlas_app_service::AppServiceRetrievalMode::OnDemandSavedWorkspace
    } else {
        atlas_app_service::AppServiceRetrievalMode::OnDemandNoQueryInference
    };
    let service = match connect(ClientOptions {
        search_profile: None,
        path_mode: crate::cli::runtime_path_mode(args.path_mode),
        index: args.index,
        embedding_cache: None,
        retrieval_mode,
    }) {
        Ok(service) => service,
        Err(error) => return report_app_error(error, json),
    };
    macro_rules! view {
        ($result:expr) => {
            match app_result($result, json)? {
                std::ops::ControlFlow::Continue(view) => view,
                std::ops::ControlFlow::Break(exit) => return Ok(exit),
            }
        };
    }
    match args.command {
        EncounterCommand::List => {
            let view = view!(service.encounters());
            if json {
                write_json_data(&view)?;
            } else {
                for encounter in view.encounters {
                    println!(
                        "{} · {} ({} participants)",
                        encounter.slug, encounter.name, encounter.participant_count
                    );
                }
            }
        }
        EncounterCommand::Show {
            encounter,
            party_level,
            party_size,
        } => {
            let mut view = view!(service.encounter(&encounter));
            if party_level.is_some() || party_size.is_some() {
                let reference = view.difficulty.reference;
                view.difficulty = view!(service.preview_encounter_difficulty(
                    &view.encounter.encounter_key,
                    PreviewEncounterDifficultyRequest {
                        expected_revision: view.revision,
                        reference: atlas_app_model::EncounterDifficultyReferenceInput {
                            party_level: party_level.or(reference.party_level),
                            party_size: party_size.or(reference.party_size),
                        },
                        addition: None,
                    },
                ));
            }
            if json {
                write_json_data(&view)?;
            } else {
                print_detail(&view);
            }
        }
        EncounterCommand::Create {
            name,
            description,
            note,
        } => {
            let view = view!(service.create_encounter(CreateEncounterRequest {
                name,
                description,
                note
            }));
            if json {
                write_json_data(&view)?;
            } else {
                println!(
                    "Created {} ({}, {})",
                    view.encounter.name, view.encounter.slug, view.encounter.encounter_key
                );
            }
        }
        EncounterCommand::Edit(edit) => {
            let description = if edit.clear_description {
                Some(None)
            } else {
                edit.description.map(Some)
            };
            let note = if edit.clear_note {
                Some(None)
            } else {
                edit.note.map(Some)
            };
            let view = view!(service.edit_encounter_metadata(
                &edit.encounter,
                edit.name,
                description,
                note
            ));
            if json {
                write_json_data(&view)?;
            } else {
                println!("Updated {}", view.encounter_key);
            }
        }
        EncounterCommand::Remove {
            encounter,
            participant,
        } => {
            let view = view!(service.remove_encounter_participant(&encounter, &participant));
            if json {
                write_json_data(&view)?;
            } else {
                println!("Removed {participant} from {}", view.encounter_key);
            }
        }
        EncounterCommand::Delete { encounter } => {
            let view = view!(service.delete_encounter(&encounter));
            if json {
                write_json_data(&view)?;
            } else {
                println!("Deleted {}", view.encounter_key);
            }
        }
        EncounterCommand::Export { encounter, output } => {
            let bytes = view!(service.export_encounter(&encounter));
            write_document(output.as_deref(), &bytes)?;
        }
        EncounterCommand::Import { input, slug } => {
            let file = File::open(&input).map_err(|err| format!("{}: {err}", input.display()))?;
            let report = view!(service.import_encounter(file, slug));
            if json {
                write_json_data(&report)?;
            } else {
                println!("Imported {} ({})", report.slug, report.encounter_key);
                if !report.reference_checked {
                    eprintln!(
                        "Reference records could not be checked in the destination workspace."
                    );
                }
                if report.source_changed == Some(true) {
                    eprintln!("Reference content differs from the exported workspace.");
                }
                for key in report.missing_records {
                    eprintln!("Reference unavailable: {key}");
                }
                for key in report.changed_records {
                    eprintln!("Reference changed: {key}");
                }
            }
        }
        EncounterCommand::Add {
            encounter,
            record,
            quantity,
            adjustment,
            initiative,
        } => {
            let view = view!(service.add_encounter_record_participant(
                &encounter,
                AddEncounterRecordParticipantRequest {
                    creature_scenario: None,
                    record_ref: record,
                    quantity,
                    participant_variant: adjustment.map(|value| match value {
                        Adjustment::Normal => EncounterParticipantVariantView::Normal,
                        Adjustment::Elite => EncounterParticipantVariantView::Elite,
                        Adjustment::Weak => EncounterParticipantVariantView::Weak,
                    }),
                    initiative,
                }
            ));
            if json {
                write_json_data(&view)?;
            } else {
                println!("Added {quantity} participant(s) to {}", view.encounter_key);
            }
        }
    }
    Ok(ExitCode::SUCCESS)
}

fn print_detail(view: &EncounterDetailView) {
    println!("{} ({})", view.encounter.name, view.encounter.slug);
    for participant in &view.participants {
        println!(
            "  {} ({})",
            participant.display_name, participant.participant_key
        );
    }
    let difficulty = &view.difficulty;
    match (difficulty.rating.as_deref(), difficulty.total_xp) {
        (Some(rating), Some(xp)) => println!("Difficulty: {rating} · {xp} opposition XP"),
        _ => println!(
            "Difficulty incomplete · {} known opposition XP",
            difficulty.known_xp
        ),
    }
    if let Some(note) = &difficulty.reference_note {
        println!("  {note}");
    }
    if let Some(award) = difficulty.xp_per_pc {
        println!("Suggested award: {award} XP per PC");
        if let Some(note) = &difficulty.award_note {
            println!("  {note}");
        }
    }
    for budget in &difficulty.budgets {
        println!("  {} budget: {} XP", budget.label, budget.xp);
    }
    for opponent in &difficulty.opposition {
        if let Some(xp) = opponent.xp {
            println!("  {}: {xp} XP", opponent.name);
        } else if let Some(reason) = &opponent.unavailable_reason {
            println!("  {}: {reason}", opponent.name);
        }
    }
}

fn write_document(output: Option<&Path>, bytes: &[u8]) -> Result<(), String> {
    let Some(output) = output else {
        return io::stdout()
            .lock()
            .write_all(bytes)
            .map_err(|err| err.to_string());
    };
    let parent = output
        .parent()
        .filter(|p| !p.as_os_str().is_empty())
        .unwrap_or(Path::new("."));
    let write = || -> io::Result<()> {
        let mut staged = tempfile::NamedTempFile::new_in(parent)?;
        staged.write_all(bytes)?;
        staged.as_file().sync_all()?;
        staged.persist_noclobber(output).map_err(|err| err.error)?;
        Ok(())
    };
    write().map_err(|err| format!("{}: {err}", output.display()))
}
