//! Shared workspace selection, guarded-member arguments and linear output.
use crate::client::{ClientOptions, connect};
use atlas_app_model::{
    AppError, CharacterAbilityFilter, CharacterContextRequest, CharacterSection, PartyMemberInput,
};
use atlas_app_service::{AppServiceResult, AppServiceRetrievalMode, AtlasAppService};
use atlas_cli_support::{CliPathMode, write_json_data};
use clap::{Args, ValueEnum};
use serde::Serialize;
use std::{
    fs::File,
    io::{self, IsTerminal, Read, Write},
    path::{Path, PathBuf},
    process::ExitCode,
    str::FromStr,
};

#[derive(Debug, Args)]
pub(crate) struct WorkspaceOptions {
    #[arg(long, global = true, help = "Emit the standard JSON envelope")]
    pub json: bool,
    #[arg(long, global = true, help = "Override the SQLite workspace")]
    pub index: Option<PathBuf>,
    #[arg(long, global = true, value_enum, default_value_t = CliPathMode::Global)]
    pub path_mode: CliPathMode,
}

impl WorkspaceOptions {
    pub fn connect(&self, catalog: bool) -> Result<AtlasAppService, AppError> {
        connect(ClientOptions {
            search_profile: None,
            path_mode: crate::cli::runtime_path_mode(self.path_mode),
            index: self.index.clone(),
            embedding_cache: None,
            retrieval_mode: if catalog {
                AppServiceRetrievalMode::OnDemandNoQueryInference
            } else {
                AppServiceRetrievalMode::OnDemandSavedWorkspace
            },
        })
    }
}

#[derive(Debug, Clone)]
pub(crate) struct ReviewedMember(pub PartyMemberInput);

impl FromStr for ReviewedMember {
    type Err = String;
    fn from_str(value: &str) -> Result<Self, Self::Err> {
        let (key, revision) = value
            .rsplit_once('=')
            .ok_or("Use CHARACTER_KEY=REVISION from a reviewed roster or character read.")?;
        let revision = revision
            .parse::<i64>()
            .map_err(|_| "Character revision must be a nonnegative integer.")?;
        if key.trim().is_empty() || revision < 0 {
            return Err("Use a nonempty character key and nonnegative revision.".into());
        }
        Ok(Self(PartyMemberInput {
            character_key: key.into(),
            expected_character_revision: revision,
        }))
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
pub(crate) enum SectionArg {
    Identity,
    Statistics,
    Proficiencies,
    CharacterOptions,
    Abilities,
    Spellcasting,
    Equipment,
    Money,
    Crafting,
    Companions,
    Source,
}
impl From<SectionArg> for CharacterSection {
    fn from(value: SectionArg) -> Self {
        match value {
            SectionArg::Identity => Self::Identity,
            SectionArg::Statistics => Self::Statistics,
            SectionArg::Proficiencies => Self::Proficiencies,
            SectionArg::CharacterOptions => Self::CharacterOptions,
            SectionArg::Abilities => Self::Abilities,
            SectionArg::Spellcasting => Self::Spellcasting,
            SectionArg::Equipment => Self::Equipment,
            SectionArg::Money => Self::Money,
            SectionArg::Crafting => Self::Crafting,
            SectionArg::Companions => Self::Companions,
            SectionArg::Source => Self::Source,
        }
    }
}

#[derive(Debug, Clone, Copy, ValueEnum)]
pub(crate) enum AbilityKindArg {
    Feat,
    Heritage,
    Special,
    Spell,
    FocusSpell,
}
impl AbilityKindArg {
    fn token(self) -> &'static str {
        match self {
            Self::Feat => "feat",
            Self::Heritage => "heritage",
            Self::Special => "special",
            Self::Spell => "spell",
            Self::FocusSpell => "focus_spell",
        }
    }
}

#[derive(Debug, Default, Args)]
pub(crate) struct SelectionOptions {
    #[arg(
        long = "section",
        value_enum,
        help = "Select a detail section; repeat to combine. Default: all interpreted sections; source is opt-in"
    )]
    pub sections: Vec<SectionArg>,
    #[arg(
        long,
        help = "Only ability links needing review; requires only --section abilities"
    )]
    pub needs_review: bool,
    #[arg(
        long,
        value_enum,
        help = "Filter ability kind; requires only --section abilities"
    )]
    pub ability_kind: Option<AbilityKindArg>,
    #[arg(
        long,
        help = "Filter ability label, choice or rule title; requires only --section abilities"
    )]
    pub query: Option<String>,
}
impl SelectionOptions {
    pub fn request(&self) -> Result<CharacterContextRequest, String> {
        let mut sections: Vec<_> = self.sections.iter().copied().map(Into::into).collect();
        sections.sort();
        sections.dedup();
        if (self.needs_review || self.ability_kind.is_some() || self.query.is_some())
            && sections != [CharacterSection::Abilities]
        {
            return Err("Ability filters require only --section abilities.".into());
        }
        Ok(CharacterContextRequest {
            sections,
            abilities: CharacterAbilityFilter {
                needs_review: self.needs_review,
                kind: self.ability_kind.map(|k| k.token().into()),
                query: self.query.clone(),
            },
        })
    }
}

pub(crate) fn respond<T: Serialize>(
    result: AppServiceResult<T>,
    json: bool,
    human: impl FnOnce(&T) -> String,
) -> Result<ExitCode, String> {
    match result {
        Ok(view) => {
            if json {
                write_json_data(&view)?;
            } else {
                write_text(&human(&view))?;
            }
            Ok(ExitCode::SUCCESS)
        }
        Err(error) => super::product::report_app_error(error.into_app_error(), json),
    }
}

pub(crate) fn write_text(text: &str) -> Result<(), String> {
    write_bytes(text.as_bytes())
}

pub(crate) fn write_bytes(bytes: &[u8]) -> Result<(), String> {
    match io::stdout().lock().write_all(bytes) {
        Ok(()) => Ok(()),
        Err(e) if e.kind() == io::ErrorKind::BrokenPipe => Ok(()),
        Err(e) => Err(e.to_string()),
    }
}

pub(crate) fn read_export(path: &Path) -> Result<String, String> {
    const LIMIT: u64 = 512 * 1024;
    let mut reader: Box<dyn Read> = if path == Path::new("-") {
        if io::stdin().is_terminal() {
            return Err("Pipe a Pathbuilder export into '-' or pass its JSON file path.".into());
        }
        Box::new(io::stdin())
    } else {
        Box::new(File::open(path).map_err(|e| format!("{}: {e}", path.display()))?)
    };
    let mut bytes = Vec::new();
    reader
        .by_ref()
        .take(LIMIT + 1)
        .read_to_end(&mut bytes)
        .map_err(|e| format!("Could not read {}: {e}", path.display()))?;
    if bytes.len() as u64 > LIMIT {
        return Err("The Pathbuilder export exceeds 512 KiB.".into());
    }
    String::from_utf8(bytes).map_err(|_| "The Pathbuilder export must be UTF-8 JSON.".into())
}

pub(crate) fn write_new_file(path: &Path, bytes: &[u8]) -> Result<(), String> {
    let parent = path
        .parent()
        .filter(|p| !p.as_os_str().is_empty())
        .unwrap_or(Path::new("."));
    let write = || -> io::Result<()> {
        let mut staged = tempfile::NamedTempFile::new_in(parent)?;
        staged.write_all(bytes)?;
        staged.as_file().sync_all()?;
        staged.persist_noclobber(path).map_err(|e| e.error)?;
        Ok(())
    };
    write().map_err(|e| format!("{}: {e}", path.display()))
}
