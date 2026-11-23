use super::product::{app_result, report_app_error};
use crate::client::{ClientOptions, connect};
use atlas_app_model::{
    AddEncounterCharacterRequest, AddEncounterPartyRequest, AddEncounterRecordParticipantRequest,
    CharacterRevision, CreateEncounterRequest, EncounterCharacterSource,
    EncounterDifficultyAddition, EncounterParticipantVariantView,
};
use atlas_cli_support::{CliPathMode, write_json_data};
use clap::{ArgGroup, Args, Subcommand, ValueEnum};
mod output;
mod review;

use std::{fs::File, path::PathBuf, process::ExitCode};

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
    /// Add a saved character using the reviewed encounter and character revisions.
    AddCharacter {
        encounter: String,
        #[arg(long, add = clap_complete::ArgValueCompleter::new(crate::commands::completions::character_keys))]
        character: String,
        #[arg(long)]
        expected_revision: i64,
        #[arg(long)]
        expected_character_revision: i64,
        #[arg(long)]
        request_id: String,
    },
    /// Add the entire reviewed party in its saved order; reuse the request id on identical retries.
    AddParty {
        encounter: String,
        #[arg(long, add = clap_complete::ArgValueCompleter::new(crate::commands::completions::party_keys))]
        party: String,
        #[arg(long)]
        expected_revision: i64,
        #[arg(long)]
        expected_party_revision: i64,
        #[arg(
            long = "member",
            required = true,
            help = "Reviewed CHARACTER_KEY=REVISION; repeat for every party member in order"
        )]
        members: Vec<super::saved_workspace::ReviewedMember>,
        #[arg(long)]
        request_id: String,
    },
    #[command(alias = "ls", about = "List saved encounters")]
    List,
    #[command(about = "Show the roster, difficulty, and suggested XP award")]
    Show {
        encounter: String,
        #[command(flatten)]
        party: PartyReference,
        #[arg(
            long,
            value_enum,
            default_value = "standard",
            help = "Show a compact roster or complete participant details"
        )]
        detail: Detail,
        #[arg(
            long,
            help = "Expand this participant only, using the key from the roster"
        )]
        participant: Option<String>,
    },
    /// Preview an addition and print the command to add the reviewed opponents.
    #[command(
        after_help = "Example:\n  atlas encounters preview garden-sentries --record 'Goblin Warrior' --quantity 2 --party-level 1 --party-size 4\n\nRun the returned add command to save the reviewed addition. Repeat that exact command to confirm a retry."
    )]
    Preview {
        encounter: String,
        #[command(flatten)]
        opponent: Opponent,
        #[command(flatten)]
        party: PartyReference,
        /// Require the encounter revision from a previous show or preview.
        #[arg(long)]
        expected_revision: Option<i64>,
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
        #[command(flatten)]
        opponent: Opponent,
        /// Reuse this ID and the canonical encounter key to retry one intended addition.
        #[arg(long)]
        request_id: Option<String>,
        /// Require the roster revision returned by encounters preview.
        #[arg(long)]
        expected_revision: Option<i64>,
        /// Require the reference generation returned by encounters preview.
        #[arg(long)]
        expected_artifact_generation: Option<String>,
        #[arg(long, allow_negative_numbers = true)]
        initiative: Option<i64>,
    },
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, ValueEnum)]
enum Detail {
    Standard,
    Full,
}

#[derive(Clone, Copy, Debug, Args)]
struct PartyReference {
    #[arg(long, help = "Preview difficulty at this party level")]
    party_level: Option<u32>,
    #[arg(long, help = "Preview difficulty for this number of PCs")]
    party_size: Option<u32>,
}

impl From<PartyReference> for atlas_app_model::EncounterDifficultyReferenceInput {
    fn from(reference: PartyReference) -> Self {
        Self {
            party_level: reference.party_level,
            party_size: reference.party_size,
        }
    }
}

#[derive(Debug, Args)]
struct Opponent {
    #[arg(long, add = clap_complete::ArgValueCompleter::new(crate::commands::completions::record_keys))]
    record: String,
    #[arg(long, default_value_t = 1, value_parser = clap::value_parser!(u32).range(1..=50))]
    quantity: u32,
    #[arg(long, value_enum)]
    adjustment: Option<Adjustment>,
}

impl From<Adjustment> for EncounterParticipantVariantView {
    fn from(value: Adjustment) -> Self {
        match value {
            Adjustment::Normal => Self::Normal,
            Adjustment::Elite => Self::Elite,
            Adjustment::Weak => Self::Weak,
        }
    }
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
    let service = match connect(ClientOptions {
        search_profile: None,
        path_mode: crate::cli::runtime_path_mode(args.path_mode),
        index: args.index.clone(),
        embedding_cache: None,
        retrieval_mode: atlas_app_service::AppServiceRetrievalMode::OnDemandNoQueryInference,
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
        EncounterCommand::AddCharacter {
            encounter,
            character,
            expected_revision,
            expected_character_revision,
            request_id,
        } => {
            let view = view!(service.add_encounter_character(
                &encounter,
                AddEncounterCharacterRequest {
                    request_id,
                    expected_encounter_revision: expected_revision,
                    source: EncounterCharacterSource::Existing {
                        character_key: character,
                        expected_character_revision
                    },
                    party: None,
                }
            ));
            if json {
                write_json_data(&view)?;
            } else {
                super::saved_workspace::write_text(&format!(
                    "Encounter {}\nParticipant {}\nCharacter {}\n",
                    view.encounter_key, view.participant_key, view.character_key
                ))?;
            }
        }
        EncounterCommand::AddParty {
            encounter,
            party,
            expected_revision,
            expected_party_revision,
            members,
            request_id,
        } => {
            let view = view!(
                service.add_encounter_party(
                    &encounter,
                    AddEncounterPartyRequest {
                        request_id,
                        expected_encounter_revision: expected_revision,
                        party_key: party,
                        expected_revision: expected_party_revision,
                        characters: members
                            .into_iter()
                            .map(|m| CharacterRevision {
                                character_key: m.0.character_key,
                                expected_revision: m.0.expected_character_revision
                            })
                            .collect(),
                    }
                )
            );
            if json {
                write_json_data(&view)?;
            } else {
                super::saved_workspace::write_text(&format!(
                    "Added reviewed party to {}\n",
                    view.encounter_key
                ))?;
            }
        }
        EncounterCommand::List => {
            let view = view!(service.encounters());
            if json {
                write_json_data(&view)?;
            } else {
                if view.encounters.is_empty() {
                    println!("No saved encounters.");
                }
                for encounter in view.encounters {
                    let archived = if encounter.archived {
                        " · Archived"
                    } else {
                        ""
                    };
                    let progress = output::turn_progress(&encounter)
                        .map(|value| format!(" · {value}"))
                        .unwrap_or_default();
                    println!(
                        "{} · {} ({} participants){progress}{archived}",
                        encounter.slug, encounter.name, encounter.participant_count
                    );
                }
            }
        }
        EncounterCommand::Show {
            encounter,
            party,
            detail,
            participant,
        } => {
            let view = view!(service.encounter_with_reference(&encounter, party.into()));
            if let Some(key) = &participant
                && !view.participants.iter().any(|p| &p.participant_key == key)
            {
                return report_app_error(
                    atlas_app_model::AppError::new(
                        atlas_app_model::AppErrorCode::EncounterParticipantNotFound,
                        format!("Participant {key} is not on this encounter's roster."),
                    ),
                    json,
                );
            }
            output::show(view, party, detail, participant.as_deref(), json)?;
        }
        EncounterCommand::Preview {
            encounter,
            party,
            opponent,
            expected_revision,
        } => {
            let comparison = view!(service.compare_encounter_addition(
                &encounter,
                expected_revision,
                party.into(),
                EncounterDifficultyAddition {
                    record_ref: opponent.record.clone(),
                    quantity: opponent.quantity,
                    participant_variant: opponent.adjustment.map(Into::into),
                    creature_scenario: None,
                }
            ));
            let commands = review::workspace_args(args.path_mode, args.index.as_deref(), json)
                .map(|workspace| review::Commands::new(&comparison, &opponent, party, &workspace));
            output::comparison(comparison, party, commands, json)?;
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
            if let Some(path) = output {
                if let Err(error) = super::saved_workspace::write_new_file(&path, &bytes) {
                    return super::saved_workspace::file_error(json, error);
                }
            } else {
                super::saved_workspace::write_bytes(&bytes)?;
            }
        }
        EncounterCommand::Import { input, slug } => {
            let file = match File::open(&input) {
                Ok(file) => file,
                Err(error) => {
                    return super::product::command_error(
                        json,
                        "file_access_failed",
                        format!("{}: {error}", input.display()),
                        3,
                    );
                }
            };
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
            opponent,
            request_id,
            expected_revision,
            expected_artifact_generation,
            initiative,
        } => {
            let guarded = expected_revision.is_some() || expected_artifact_generation.is_some();
            let result = service.add_reviewed_encounter_record_participant(
                &encounter,
                AddEncounterRecordParticipantRequest {
                    request_id: request_id.unwrap_or_else(review::request_id),
                    creature_scenario: None,
                    record_ref: opponent.record.clone(),
                    quantity: opponent.quantity,
                    participant_variant: opponent.adjustment.map(Into::into),
                    initiative,
                },
                expected_revision,
                expected_artifact_generation,
            );
            match result {
                Ok(view) => output::added(view, json)?,
                Err(error) => {
                    let error = error.into_app_error();
                    if guarded
                        && matches!(
                            error.code,
                            atlas_app_model::AppErrorCode::EncounterTurnConflict
                                | atlas_app_model::AppErrorCode::ArtifactGenerationChanged
                        )
                    {
                        let workspace =
                            review::workspace_args(args.path_mode, args.index.as_deref(), json);
                        return review::report_stale(error, &encounter, &opponent, workspace, json);
                    }
                    return report_app_error(error, json);
                }
            }
        }
    }
    Ok(ExitCode::SUCCESS)
}
