pub(crate) mod args;
pub(crate) mod output;
use super::saved_workspace::{file_error, read_export, respond, write_new_file};
use args::*;
use atlas_app_model::*;
use atlas_app_service::AtlasAppService;
use std::{collections::BTreeMap, process::ExitCode};

pub(crate) fn run(args: CharactersArgs) -> Result<ExitCode, String> {
    let json = args.workspace.json;
    let selection = match &args.command {
        CharacterCommand::Show { selection, .. } | CharacterCommand::Preview { selection, .. } => {
            match selection.request() {
                Ok(request) => Some(request),
                Err(error) => return super::product::report_app_error(error, json),
            }
        }
        _ => None,
    };
    if matches!(
        &args.command,
        CharacterCommand::Show {
            output: Some(_),
            ..
        }
    ) && selection
        .as_ref()
        .is_none_or(|r| r.sections != [CharacterSection::Source])
    {
        return super::product::report_app_error(
            AppError::new(
                AppErrorCode::InvalidRequest,
                "--output requires only --section source.",
            ),
            json,
        );
    }
    let service = match args.workspace.connect() {
        Ok(s) => s,
        Err(e) => return super::product::report_app_error(e, json),
    };
    match args.command {
        CharacterCommand::List { party } => {
            respond(service.party_characters(party.as_deref()), json, |view| {
                if view.characters.is_empty() {
                    return "No saved characters.\n".into();
                }
                view.characters
                    .iter()
                    .map(|c| {
                        format!(
                            "{} · level {}\n  {}  revision {}\n",
                            c.name,
                            c.level.map_or_else(|| "unknown".into(), |v| v.to_string()),
                            c.character_key,
                            c.revision
                        )
                    })
                    .collect()
            })
        }
        CharacterCommand::Show {
            character,
            selection: options,
            output: destination,
        } => {
            let request = selection.unwrap_or_default();
            let result = service.character_context(&character, request);
            if let Some(destination) = destination {
                let view = match super::product::app_result(result, json)? {
                    std::ops::ControlFlow::Continue(v) => v,
                    std::ops::ControlFlow::Break(exit) => return Ok(exit),
                };
                let Some(source) = view
                    .sections
                    .get(&CharacterSection::Source)
                    .and_then(|s| s.source_json.as_deref())
                else {
                    return super::product::report_app_error(
                        AppError::new(
                            AppErrorCode::InvalidRequest,
                            "This character has no original export. Re-import from Pathbuilder first.",
                        ),
                        json,
                    );
                };
                if let Err(error) = write_new_file(&destination, source.as_bytes()) {
                    return file_error(json, error);
                }
                respond(
                    Ok(
                        serde_json::json!({"character_key":view.character.character_key,"character_revision":view.character.revision,"output":destination}),
                    ),
                    json,
                    |_| format!("Wrote original export to {}\n", destination.display()),
                )
            } else if json && options.detail == super::saved_workspace::SheetDetail::Standard {
                super::saved_workspace::respond_json(
                    result.map(|view| output::compact_character(&view)),
                )
            } else {
                respond(result, json, |view| output::character(view, &options))
            }
        }
        CharacterCommand::Create(options) => {
            let party = party_target(options.party, options.expected_party_revision)?;
            respond(
                service.create_character(CreateCharacterRequest {
                    request_id: options.request_id,
                    name: options.name,
                    level: options.level,
                    party,
                }),
                json,
                mutation,
            )
        }
        CharacterCommand::Edit(options) => {
            let level = if options.clear_level {
                Some(None)
            } else {
                options.level.map(Some)
            };
            respond(
                service.edit_character_metadata(
                    &options.character,
                    options.expected_revision,
                    options.name,
                    level,
                ),
                json,
                mutation,
            )
        }
        CharacterCommand::Delete {
            character,
            expected_revision,
        } => respond(
            service.delete_character(&character, DeleteCharacterRequest { expected_revision }),
            json,
            mutation,
        ),
        CharacterCommand::Preview {
            input,
            character,
            selection: options,
        } => {
            let source = match read_export(&input) {
                Ok(source) => source,
                Err(error) => return file_error(json, error),
            };
            let result = service.preview_character_import(
                &source,
                character.as_deref(),
                selection.unwrap_or_default(),
            );
            if json && options.detail == super::saved_workspace::SheetDetail::Standard {
                super::saved_workspace::respond_json(
                    result.map(|view| output::compact_preview(&view)),
                )
            } else {
                respond(result, json, |view| output::preview(view, options.detail))
            }
        }
        CharacterCommand::Import(options) => {
            let party = party_target(options.party, options.expected_party_revision)?;
            import(
                &service,
                match read_export(&options.input) {
                    Ok(source) => source,
                    Err(error) => return file_error(json, error),
                },
                options.review,
                options.request_id,
                None,
                None,
                party,
                json,
            )
        }
        CharacterCommand::Refresh {
            character,
            input,
            expected_revision,
            review,
            request_id,
        } => import(
            &service,
            match read_export(&input) {
                Ok(source) => source,
                Err(error) => return file_error(json, error),
            },
            review,
            request_id.unwrap_or_else(|| format!("{:032x}", rand::random::<u128>())),
            Some(character),
            Some(expected_revision),
            None,
            json,
        ),
        CharacterCommand::Link(options) => {
            let selection = match (options.record, options.expected_generation) {
                (Some(record_key), Some(expected_artifact_generation)) => {
                    PcAbilitySelection::Manual {
                        record_key,
                        expected_artifact_generation,
                    }
                }
                _ if options.automatic => PcAbilitySelection::Automatic,
                _ => {
                    return super::product::command_error(
                        json,
                        "invalid_input",
                        "Choose --record with --expected-generation, or --automatic.".into(),
                        2,
                    );
                }
            };
            respond(
                service.save_pc_ability_link(
                    &options.character,
                    &options.ability,
                    SavePcAbilityLinkRequest {
                        expected_character_revision: options.expected_revision,
                        selection,
                    },
                ),
                json,
                |view| {
                    format!(
                        "Updated ability link on {}\n{}  revision {}\n",
                        view.name, view.character_key, view.character_revision
                    )
                },
            )
        }
    }
}

fn mutation(view: &CharacterMutationView) -> String {
    if view.available {
        format!(
            "Character {}\nUse atlas characters show {} for its current revision and details.\n",
            view.character_key, view.character_key
        )
    } else {
        format!("Character {} is no longer available.\n", view.character_key)
    }
}

fn party_target(
    key: Option<String>,
    revision: Option<i64>,
) -> Result<Option<CharacterPartyTarget>, String> {
    match (key, revision) {
        (Some(party_key), Some(expected_revision)) => Ok(Some(CharacterPartyTarget {
            party_key,
            expected_revision,
        })),
        (None, None) => Ok(None),
        _ => Err("Supply --party and --expected-party-revision together.".into()),
    }
}

#[expect(
    clippy::too_many_arguments,
    reason = "Mirrors the existing import request plus its reviewed source precondition"
)]
fn import(
    service: &AtlasAppService,
    source: String,
    review: ImportReviewOptions,
    request_id: String,
    character_key: Option<String>,
    expected_character_revision: Option<i64>,
    party: Option<CharacterPartyTarget>,
    json: bool,
) -> Result<ExitCode, String> {
    let mut rule_links = BTreeMap::new();
    for RuleLink(ability, record) in review.links {
        if rule_links.insert(ability, record).is_some() {
            return super::product::command_error(
                json,
                "invalid_input",
                "A rule-link ability was supplied twice.".into(),
                2,
            );
        }
    }
    respond(
        service.save_reviewed_character_import(
            SavePathbuilderPcRequest {
                request_id,
                json: source,
                rule_links,
                party,
                expected_character_revision,
                expected_artifact_generation: review.expected_generation,
                character_key,
            },
            &review.expected_source_sha256,
        ),
        json,
        mutation,
    )
}
