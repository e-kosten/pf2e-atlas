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
    for name in ["analyze", "audit-source-paths", "inspect"] {
        assert!(!candidates.contains(&name));
    }
}
