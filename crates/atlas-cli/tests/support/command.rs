#![allow(dead_code)]

use std::path::Path;
use std::process::{Command, Output};

pub fn atlas_command() -> Command {
    let mut command = Command::new(env!("CARGO_BIN_EXE_atlas"));
    clear_git_hook_env(&mut command);
    command
}

pub fn git_command() -> Command {
    let mut command = Command::new("git");
    clear_git_hook_env(&mut command);
    command
}

pub fn atlas_json(args: &[&str]) -> Result<Output, Box<dyn std::error::Error>> {
    let mut args = args.to_vec();
    if !args.contains(&"--json") {
        args.push("--json");
    }
    Ok(atlas_command().args(args).output()?)
}

pub fn build_index(source_root: &Path) -> Result<std::path::PathBuf, Box<dyn std::error::Error>> {
    let index_path = source_root.join("artifact.sqlite");
    let output = atlas_json(&[
        "index",
        "build",
        "--source",
        source_root.to_str().unwrap(),
        "--output",
        index_path.to_str().unwrap(),
        "--no-embeddings",
    ])?;
    assert!(output.status.success());
    Ok(index_path)
}

pub fn validate_index(path: &Path) -> Result<Output, Box<dyn std::error::Error>> {
    Ok(atlas_command()
        .args(["index", "validate", "--index"])
        .arg(path)
        .arg("--json")
        .output()?)
}

pub fn validate_base_index(path: &Path) -> Result<Output, Box<dyn std::error::Error>> {
    Ok(atlas_command()
        .args(["index", "validate", "--no-embeddings", "--index"])
        .arg(path)
        .arg("--json")
        .output()?)
}

pub fn check_base_index(path: &Path) -> Result<Output, Box<dyn std::error::Error>> {
    Ok(atlas_command()
        .args(["index", "check", "--no-embeddings", "--index"])
        .arg(path)
        .arg("--json")
        .output()?)
}

pub fn help_output(args: &[&str]) -> Result<String, Box<dyn std::error::Error>> {
    let output = atlas_command().args(args).arg("--help").output()?;
    assert!(output.status.success());
    Ok(String::from_utf8(output.stdout)?)
}

pub fn clear_git_hook_env(command: &mut Command) {
    for name in ["GIT_DIR", "GIT_WORK_TREE", "GIT_INDEX_FILE", "GIT_PREFIX"] {
        command.env_remove(name);
    }
}
