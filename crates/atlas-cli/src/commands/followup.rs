//! Re-executable CLI arguments for continuing a read in the same workspace.
use atlas_app_model::RecordNavigationView;
use atlas_cli_support::CliPathMode;
use clap::ValueEnum;
use std::path::Path;

use super::record::navigation::Locator;

pub(crate) fn workspace_args(
    mode: CliPathMode,
    index: Option<&Path>,
) -> Result<Vec<String>, String> {
    let mut args = Vec::new();
    if let Some(path) = index {
        args.push(format!(
            "--index={}",
            path.to_str()
                .ok_or("Cannot encode the workspace path in a command argument for display.")?
        ));
    }
    if matches!(mode, CliPathMode::Repo) {
        args.push("--path-mode=repo".into());
    }
    Ok(args)
}

pub(crate) fn shell_command(args: &[String]) -> String {
    args.iter()
        .map(|arg| {
            if !arg.is_empty()
                && arg
                    .bytes()
                    .all(|b| b.is_ascii_alphanumeric() || b"_.,/:=-".contains(&b))
            {
                return arg.clone();
            }
            let escaped = if cfg!(windows) {
                // PowerShell accepts these four typographic marks as single-quote delimiters too.
                let mut literal = String::with_capacity(arg.len());
                for character in arg.chars() {
                    if matches!(character, '\'' | '‘' | '’' | '‚' | '‛') {
                        literal.push('\'');
                    }
                    literal.push(character);
                }
                literal
            } else {
                arg.replace('\'', "'\"'\"'")
            };
            format!("'{escaped}'")
        })
        .collect::<Vec<_>>()
        .join(" ")
}

pub(crate) fn open_record(
    navigation: &RecordNavigationView,
    generation: &str,
    workspace: &[String],
    json: bool,
) -> Vec<String> {
    let locator = Locator::new(generation, navigation);
    let mut args = vec![
        "atlas".into(),
        "record".into(),
        "get".into(),
        "--navigation".into(),
        locator.compact().to_string(),
    ];
    args.extend_from_slice(workspace);
    if json {
        args.push("--json".into());
    }
    args
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
pub(crate) enum EvidenceDetail {
    Standard,
    Full,
}

pub(crate) fn filter_args(filter: &crate::cli::args::FilterOptions) -> Vec<String> {
    let mut args = Vec::new();
    if let Some(expression) = &filter.where_expression {
        args.push(format!("--where={expression}"));
    }
    for (flag, values) in [
        ("kind", &filter.kinds),
        ("pack-name", &filter.pack_names),
        ("rarity", &filter.rarities),
        ("trait", &filter.traits),
        ("links-to", &filter.links_to),
        ("linked-from", &filter.linked_from),
        ("links-to-main-entry", &filter.links_to_main_entry),
        ("linked-from-main-entry", &filter.linked_from_main_entry),
    ] {
        args.extend(values.iter().map(|value| format!("--{flag}={value}")));
    }
    args
}

pub(crate) struct OccurrenceArgs<'a> {
    pub filter: &'a [String],
    pub workspace: Option<&'a [String]>,
    pub generation: &'a str,
    pub json: bool,
    pub detail: EvidenceDetail,
}

impl OccurrenceArgs<'_> {
    pub fn next(
        &self,
        record: &str,
        clause: &str,
        page: &atlas_app_model::SearchPageView,
    ) -> Option<Vec<String>> {
        let next = page.next_page?;
        let workspace = self.workspace?;
        let mut args = vec![
            "atlas".into(),
            "graph".into(),
            "occurrences".into(),
            record.into(),
            format!("--clause-id={clause}"),
            format!("--artifact-generation={}", self.generation),
            format!("--page={next}"),
            format!("--limit={}", page.size),
        ];
        args.extend_from_slice(self.filter);
        args.extend_from_slice(workspace);
        if self.json {
            args.push("--json".into());
        }
        if self.detail == EvidenceDetail::Full {
            args.push("--detail=full".into());
        }
        Some(args)
    }
}
