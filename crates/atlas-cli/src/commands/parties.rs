use super::saved_workspace::{ReviewedMember, SelectionOptions, WorkspaceOptions, respond};
use atlas_app_model::{CreatePartyRequest, DeletePartyRequest, PartyView};
use clap::{ArgGroup, Args, Subcommand};
use std::process::ExitCode;

#[derive(Debug, Args)]
#[command(
    after_help = "Examples:\n  atlas parties list --json\n  atlas parties context \"Example party\" --section equipment --section money --json\n  atlas parties add PARTY_KEY --member CHARACTER_KEY=REVISION --expected-revision PARTY_REVISION\n\nRead commands accept keys or exact unique names. --member takes a character key and its reviewed revision, separated by '='."
)]
pub(crate) struct PartiesArgs {
    #[command(flatten)]
    pub workspace: WorkspaceOptions,
    #[command(subcommand)]
    command: PartyCommand,
}

#[derive(Debug, Subcommand)]
enum PartyCommand {
    #[command(alias = "ls", about = "List saved parties and their roster sizes")]
    List,
    /// Show the ordered roster and reviewed character revisions.
    Show {
        #[arg(add = clap_complete::ArgValueCompleter::new(crate::commands::completions::party_keys))]
        party: String,
    },
    /// Read coherent build and loadout context for every member.
    Context {
        #[arg(add = clap_complete::ArgValueCompleter::new(crate::commands::completions::party_keys))]
        party: String,
        #[command(flatten)]
        selection: SelectionOptions,
    },
    /// Create a party with optional reviewed existing characters.
    Create {
        name: String,
        #[arg(
            long = "member",
            help = "CHARACTER_KEY=REVISION; repeat for initial roster order"
        )]
        members: Vec<ReviewedMember>,
    },
    /// Rename a saved party.
    Edit {
        party: String,
        #[arg(long)]
        name: String,
        #[arg(long)]
        expected_revision: i64,
    },
    /// Delete the party, preserving its characters.
    Delete {
        party: String,
        #[arg(long)]
        expected_revision: i64,
    },
    /// Append reviewed saved characters to the roster.
    Add {
        party: String,
        #[arg(
            long = "member",
            required = true,
            help = "Reviewed CHARACTER_KEY=REVISION; repeat to add several"
        )]
        members: Vec<ReviewedMember>,
        #[arg(long)]
        expected_revision: i64,
    },
    /// Remove memberships, preserving saved characters.
    Remove {
        party: String,
        #[arg(long = "character", required = true)]
        characters: Vec<String>,
        #[arg(long)]
        expected_revision: i64,
    },
    /// Supply the full reviewed roster in the desired order.
    Reorder {
        party: String,
        #[arg(
            long = "member",
            required = true,
            help = "Reviewed CHARACTER_KEY=REVISION; supply every member once"
        )]
        members: Vec<ReviewedMember>,
        #[arg(long)]
        expected_revision: i64,
    },
    /// Set or clear selected character levels atomically across their rosters.
    SetLevel(LevelOptions),
}

#[derive(Debug, Args)]
#[command(group(ArgGroup::new("level_change").required(true).args(["level", "clear"])))]
struct LevelOptions {
    party: String,
    #[arg(
        long = "member",
        required = true,
        help = "Selected CHARACTER_KEY=REVISION; repeat for several PCs"
    )]
    members: Vec<ReviewedMember>,
    #[arg(long)]
    expected_revision: i64,
    #[arg(long, value_parser = clap::value_parser!(i64).range(1..=20))]
    level: Option<i64>,
    #[arg(long)]
    clear: bool,
}

pub(crate) fn run(args: PartiesArgs) -> Result<ExitCode, String> {
    let json = args.workspace.json;
    let request = if let PartyCommand::Context { selection, .. } = &args.command {
        Some(selection.request()?)
    } else {
        None
    };
    let service = match args.workspace.connect() {
        Ok(s) => s,
        Err(e) => return super::product::report_app_error(e, json),
    };
    match args.command {
        PartyCommand::List => respond(service.parties(), json, |view| {
            if view.parties.is_empty() {
                return "No saved parties.\n".into();
            }
            view.parties
                .iter()
                .map(|p| {
                    format!(
                        "{} · {} characters\n  {}  revision {}\n",
                        p.name,
                        p.members.len(),
                        p.party_key,
                        p.revision
                    )
                })
                .collect()
        }),
        PartyCommand::Show { party } => respond(service.party_roster(&party), json, roster),
        PartyCommand::Context { party, selection } => respond(
            service.party_context(&party, request.unwrap_or_default()),
            json,
            |view| {
                let mut text = format!(
                    "{} · {} characters\n{}  revision {}\n\n",
                    view.name,
                    view.members.len(),
                    view.party_key,
                    view.revision
                );
                for member in &view.members {
                    text.push_str(&super::characters::output::character(
                        member,
                        !selection.sections.is_empty(),
                    ));
                    text.push('\n');
                }
                text
            },
        ),
        PartyCommand::Create { name, members } => respond(
            service.create_party(CreatePartyRequest {
                name,
                members: members.into_iter().map(|m| m.0).collect(),
            }),
            json,
            roster,
        ),
        PartyCommand::Edit {
            party,
            name,
            expected_revision,
        } => respond(
            service.edit_party_name(&party, expected_revision, name),
            json,
            roster,
        ),
        PartyCommand::Delete {
            party,
            expected_revision,
        } => respond(
            service.delete_party(&party, DeletePartyRequest { expected_revision }),
            json,
            |view| {
                format!(
                    "Deleted party {}. Its characters are preserved.\n",
                    view.party_key
                )
            },
        ),
        PartyCommand::Add {
            party,
            members,
            expected_revision,
        } => respond(
            service.add_party_members(
                &party,
                expected_revision,
                members.into_iter().map(|m| m.0).collect(),
            ),
            json,
            roster,
        ),
        PartyCommand::Remove {
            party,
            characters,
            expected_revision,
        } => respond(
            service.remove_party_members(&party, expected_revision, characters),
            json,
            roster,
        ),
        PartyCommand::Reorder {
            party,
            members,
            expected_revision,
        } => respond(
            service.reorder_party_members(
                &party,
                expected_revision,
                members.into_iter().map(|m| m.0).collect(),
            ),
            json,
            roster,
        ),
        PartyCommand::SetLevel(options) => respond(
            service.set_party_character_levels(
                &options.party,
                options.expected_revision,
                options.members.into_iter().map(|m| m.0).collect(),
                options.level,
            ),
            json,
            roster,
        ),
    }
}

fn roster(view: &PartyView) -> String {
    let mut text = format!(
        "{} · {} characters\n{}  revision {}\n",
        view.name,
        view.members.len(),
        view.party_key,
        view.revision
    );
    for (i, member) in view.members.iter().enumerate() {
        text.push_str(&format!(
            "  {}. {} · level {}\n     {}={}\n",
            i + 1,
            member.name,
            member
                .level
                .map_or_else(|| "unknown".into(), |v| v.to_string()),
            member.character_key,
            member.character_revision
        ));
    }
    text
}
