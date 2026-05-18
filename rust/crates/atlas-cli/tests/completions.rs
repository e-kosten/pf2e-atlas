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
}
