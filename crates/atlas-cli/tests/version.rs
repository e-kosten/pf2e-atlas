use std::process::Command;

#[test]
fn version_output_matches_package_version() {
    let output = Command::new(env!("CARGO_BIN_EXE_atlas"))
        .arg("--version")
        .output()
        .expect("atlas --version should run");

    assert!(
        output.status.success(),
        "atlas --version failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let actual = String::from_utf8(output.stdout).expect("version output should be utf-8");
    let manifest = std::fs::read_to_string(concat!(env!("CARGO_MANIFEST_DIR"), "/Cargo.toml"))
        .expect("manifest should be readable");
    let version = manifest
        .lines()
        .find_map(|line| line.strip_prefix("version = \""))
        .and_then(|line| line.strip_suffix('"'))
        .expect("manifest should declare package version");

    assert_eq!(actual.trim(), format!("atlas {version}"));
}
