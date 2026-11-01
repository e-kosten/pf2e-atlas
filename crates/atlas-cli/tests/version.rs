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

#[test]
fn build_info_identifies_the_command_surface_without_setup() {
    let output = Command::new(env!("CARGO_BIN_EXE_atlas"))
        .args(["build-info", "--json"])
        .output()
        .unwrap();
    assert!(output.status.success());
    let body: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(
        body,
        serde_json::json!({
            "status": "ok",
            "data": {
                "version": env!("CARGO_PKG_VERSION"),
                "variant": if cfg!(feature = "web") { "full" } else { "cli-only" }
            }
        })
    );
    let output = Command::new(env!("CARGO_BIN_EXE_atlas"))
        .args(["web", "--help"])
        .output()
        .unwrap();
    assert_eq!(output.status.success(), cfg!(feature = "web"));
    if !cfg!(feature = "web") {
        assert_eq!(output.status.code(), Some(2));
        assert!(String::from_utf8_lossy(&output.stderr).contains("unrecognized subcommand 'web'"));
    }
    let output = Command::new(env!("CARGO_BIN_EXE_atlas"))
        .arg("--help")
        .output()
        .unwrap();
    let help = String::from_utf8(output.stdout).unwrap();
    assert_eq!(
        help.lines()
            .any(|line| line.trim_start().starts_with("web ")),
        cfg!(feature = "web")
    );
}
