use std::process::Command;

#[test]
fn completions_register_live_bash_completion_and_offer_product_commands() {
    let output = Command::new(env!("CARGO_BIN_EXE_atlas"))
        .args(["completions", "bash"])
        .output()
        .expect("atlas completions should run");
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let script = String::from_utf8_lossy(&output.stdout);
    assert!(script.contains("_atlas"));
    assert!(script.contains("ATLAS_COMPLETE"));
    let output = Command::new(env!("CARGO_BIN_EXE_atlas"))
        .env("ATLAS_COMPLETE", "zsh")
        .env("_CLAP_COMPLETE_INDEX", "1")
        .env("_CLAP_IFS", "\n")
        .args(["--", "atlas", ""])
        .output()
        .unwrap();
    assert!(output.status.success());
    let output_text = String::from_utf8_lossy(&output.stdout);
    let candidates = output_text
        .lines()
        .map(|line| line.split(':').next().unwrap())
        .collect::<Vec<_>>();
    for name in ["record", "search", "setup"] {
        assert!(candidates.contains(&name));
    }
    assert_eq!(candidates.contains(&"web"), cfg!(feature = "web"));
    assert!(candidates.contains(&"build-info"));
    for name in ["analyze", "audit-source-paths", "inspect"] {
        assert!(!candidates.contains(&name));
    }
}

#[test]
fn search_and_source_scope_completions_offer_only_supported_values() {
    for (command, flag) in [
        ("search", "--search-content"),
        ("similar", "--search-content"),
        ("similar", "--sections"),
    ] {
        let output = Command::new(env!("CARGO_BIN_EXE_atlas"))
            .env("ATLAS_COMPLETE", "zsh")
            .env("_CLAP_COMPLETE_INDEX", "3")
            .env("_CLAP_IFS", "\n")
            .args(["--", "atlas", command, flag, ""])
            .output()
            .unwrap();
        assert!(
            output.status.success(),
            "{}",
            String::from_utf8_lossy(&output.stderr)
        );
        let values = String::from_utf8_lossy(&output.stdout)
            .lines()
            .map(|line| line.split(':').next().unwrap().to_owned())
            .collect::<Vec<_>>();
        assert_eq!(values, ["all", "core"], "{command} {flag}");
    }
}
