use std::process::Command;

#[test]
fn completions_generate_bash_script() {
    let output = Command::new(env!("CARGO_BIN_EXE_atlas"))
        .args(["completions", "bash"])
        .output()
        .expect("atlas completions should run");

    assert!(
        output.status.success(),
        "expected success, stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("_atlas"));
    assert!(stdout.contains("record"));
    assert!(stdout.contains("search"));
    assert!(stdout.contains("setup"));
    assert!(!stdout.contains("audit-source-paths"));
    assert!(!stdout.contains("source-values"));

    let probe = r#"
COMP_WORDS=(atlas source "")
COMP_CWORD=2
_atlas atlas "" source
printf '%s\n' "${COMPREPLY[@]}"
"#;
    let completion = Command::new("bash")
        .args(["-c", &format!("{stdout}\n{probe}")])
        .output()
        .expect("execute generated completion");
    assert!(completion.status.success());
    let suggestions = String::from_utf8_lossy(&completion.stdout);
    for command in ["schema", "values", "analyze"] {
        assert!(
            suggestions.lines().any(|line| line == command),
            "missing completion: {command}"
        );
    }
}
