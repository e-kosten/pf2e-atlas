//! CLI-specific section rediscovery; shared workflow error identities stay intact.
use super::args::SimilarOptions;
use atlas_app_model::{AppError, AppErrorCode};
use atlas_cli_support::CliPathMode;
use atlas_domain::SimilaritySeed;

pub(super) fn missing_generation(options: &SimilarOptions) -> Option<AppError> {
    (!options.parts.is_empty() && options.artifact_generation.is_none()).then(|| {
        let mut error = AppError::new(
            AppErrorCode::InvalidRequest,
            "--part requires --artifact-generation from the section inventory",
        );
        append_command(&mut error, &command(options, None));
        error
    })
}

pub(super) fn command(options: &SimilarOptions, seed: Option<&SimilaritySeed>) -> String {
    let identity = match seed {
        Some(SimilaritySeed::Record { record_key }) => record_key.to_string(),
        Some(SimilaritySeed::Trait { trait_key }) => trait_key.to_string(),
        None => options.record_ref.clone(),
    };
    let mut words = vec![
        "atlas".into(),
        "similar".into(),
        identity,
        "--list-parts".into(),
    ];
    if let Some(profile) = &options.search_profile {
        words.extend(["--search-profile".into(), profile.to_string()]);
    }
    words.extend([
        "--path-mode".into(),
        match options.path_mode {
            CliPathMode::Repo => "repo",
            CliPathMode::Global => "global",
        }
        .into(),
    ]);
    if let Some(index) = &options.index {
        words.extend(["--index".into(), index.to_string_lossy().into_owned()]);
    }
    if options.json {
        words.push("--json".into());
    }
    words
        .iter()
        .map(|word| shell_literal(word))
        .collect::<Vec<_>>()
        .join(" ")
}

pub(super) fn enrich(mut error: AppError, rediscovery: &str, explicit_parts: bool) -> AppError {
    // InvalidRequest has no selector-specific subtype. For an explicit selection,
    // offer rediscovery as context without replacing the actual failure reason.
    if error.code == AppErrorCode::ArtifactGenerationChanged
        || (explicit_parts && error.code == AppErrorCode::InvalidRequest)
    {
        append_command(&mut error, rediscovery);
    }
    error
}

fn append_command(error: &mut AppError, command: &str) {
    error.message.push_str("\nRediscover sections: ");
    error.message.push_str(command);
}

// PowerShell on Windows; POSIX shells on other supported hosts.
fn shell_literal(value: &str) -> String {
    if !value.is_empty()
        && value
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || b"_./:-".contains(&byte))
    {
        return value.to_owned();
    }
    #[cfg(windows)]
    let escaped = value.replace('\'', "''");
    #[cfg(not(windows))]
    let escaped = value.replace('\'', "'\"'\"'");
    format!("'{escaped}'")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cli::{Cli, Command};
    use clap::Parser;
    use serde_json::json;

    #[test]
    fn rediscovery_preserves_exact_seed_profile_paths_and_shell_metacharacters() {
        let Command::Similar(options) = Cli::try_parse_from([
            "atlas",
            "similar",
            "Brown's $(touch /tmp/nope)",
            "--search-profile",
            "embeddinggemma",
            "--index",
            "/tmp/Atlas's data/artifact.sqlite",
            "--path-mode",
            "repo",
            "--json",
        ])
        .unwrap()
        .command
        else {
            panic!("expected Similar");
        };
        let seed = SimilaritySeed::Record {
            record_key: "hazards:aaaaaaaaaaaaaaaa".parse().unwrap(),
        };
        let actual = command(&options, Some(&seed));
        assert!(actual.starts_with("atlas similar hazards:aaaaaaaaaaaaaaaa --list-parts --search-profile embeddinggemma --path-mode repo --index "));
        assert!(actual.ends_with(" --json"));
        #[cfg(not(windows))]
        assert!(actual.contains("'/tmp/Atlas'\"'\"'s data/artifact.sqlite'"));
        #[cfg(windows)]
        assert!(actual.contains("'/tmp/Atlas''s data/artifact.sqlite'"));
        assert!(!actual.contains("touch"));
        let raw = command(&options, None);
        #[cfg(not(windows))]
        assert!(raw.contains("'Brown'\"'\"'s $(touch /tmp/nope)'"));
        #[cfg(windows)]
        assert!(raw.contains("'Brown''s $(touch /tmp/nope)'"));
    }

    #[test]
    fn stale_generation_and_unavailable_parts_keep_error_identity_and_json_envelope_data() {
        for (code, explicit) in [
            (AppErrorCode::ArtifactGenerationChanged, false),
            (AppErrorCode::InvalidRequest, true),
        ] {
            let mut original = AppError::new(code, "Refresh required").with_details(
                json!({"expected_artifact_generation":"old","actual_artifact_generation":"new"}),
            );
            original.retryable = Some(false);
            original.recoverable_action =
                Some(atlas_app_model::AppRecoverableAction::RefreshResults);
            let updated = enrich(
                original.clone(),
                "atlas similar trait:fire --list-parts --path-mode repo",
                explicit,
            );
            assert_eq!(updated.code, original.code);
            assert_eq!(updated.details, original.details);
            assert_eq!(updated.retryable, original.retryable);
            assert_eq!(updated.recoverable_action, original.recoverable_action);
            let encoded = serde_json::to_value(&updated).unwrap();
            assert!(
                encoded["message"]
                    .as_str()
                    .unwrap()
                    .contains("atlas similar trait:fire --list-parts")
            );
        }
        let untouched = AppError::new(AppErrorCode::ServiceBusy, "busy");
        assert_eq!(enrich(untouched.clone(), "unused", true), untouched);
        let unrelated = AppError::new(AppErrorCode::InvalidRequest, "invalid page");
        assert_eq!(enrich(unrelated.clone(), "unused", false), unrelated);
        let selected_request = enrich(unrelated, "atlas similar trait:fire --list-parts", true);
        assert!(selected_request.message.starts_with("invalid page\n"));
        assert!(selected_request.message.contains("Rediscover sections:"));
    }
}
