use std::fs;
use std::path::PathBuf;
use std::process::Command;

use serde_json::Value;

#[test]
fn setup_repo_path_resolution_uses_root_workspace_defaults()
-> Result<(), Box<dyn std::error::Error>> {
    let root = temp_root("setup-repo-path-defaults");
    fs::create_dir_all(root.join("crates/atlas-cli"))?;
    fs::write(root.join("Cargo.toml"), "[workspace]\n")?;
    fs::write(
        root.join("crates/atlas-cli/Cargo.toml"),
        "[package]\nname = \"atlas-cli\"\n",
    )?;

    let init = git_command()
        .args(["init", "--quiet"])
        .current_dir(&root)
        .output()?;
    assert!(init.status.success());

    let output = atlas_command()
        .current_dir(root.join("crates/atlas-cli"))
        .args([
            "setup",
            "--path-mode",
            "repo",
            "--offline",
            "--check",
            "--json",
        ])
        .output()?;

    assert_eq!(output.status.code(), Some(1));
    assert!(output.stderr.is_empty());
    let actual: Value = serde_json::from_slice(&output.stdout)?;
    let actual = ok_data(&actual);
    let expected_root = root.canonicalize()?;
    assert_eq!(actual["path_mode"], "repo");
    assert_eq!(actual["repo_root"], expected_root.display().to_string());
    assert_eq!(
        actual["paths"]["source"],
        expected_root.join("vendor/pf2e").display().to_string()
    );
    assert_eq!(
        actual["paths"]["embedding_cache"],
        expected_root.join(".cache/hf-models").display().to_string()
    );
    assert_eq!(
        actual["paths"]["index"],
        expected_root
            .join(".cache/pf2e-rust-index.sqlite")
            .display()
            .to_string()
    );

    let _ = fs::remove_dir_all(root);
    Ok(())
}

fn ok_data(value: &Value) -> &Value {
    assert_eq!(value["status"], "ok");
    value.get("data").expect("ok envelope should contain data")
}

fn temp_root(name: &str) -> PathBuf {
    let mut path = std::env::temp_dir();
    path.push(format!(
        "atlas-cli-{name}-{}-{}",
        std::process::id(),
        std::thread::current().name().unwrap_or("test")
    ));
    let _ = fs::remove_dir_all(&path);
    path
}

fn atlas_command() -> Command {
    let mut command = Command::new(env!("CARGO_BIN_EXE_atlas"));
    clear_git_hook_env(&mut command);
    command
}

fn git_command() -> Command {
    let mut command = Command::new("git");
    clear_git_hook_env(&mut command);
    command
}

fn clear_git_hook_env(command: &mut Command) {
    for name in ["GIT_DIR", "GIT_WORK_TREE", "GIT_INDEX_FILE", "GIT_PREFIX"] {
        command.env_remove(name);
    }
}
