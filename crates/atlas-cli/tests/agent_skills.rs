use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

use serde_json::Value;

#[test]
fn doctor_reports_supported_targets() -> Result<(), Box<dyn std::error::Error>> {
    let root = temp_dir("atlas-skill-doctor")?;
    let output = atlas(&root)
        .args(["agent", "skills", "doctor", "--json"])
        .output()?;

    assert!(output.status.success());
    let actual: Value = serde_json::from_slice(&output.stdout)?;
    let data = ok_data(&actual);
    assert_eq!(
        PathBuf::from(data["workspace_root"].as_str().unwrap()).canonicalize()?,
        root.canonicalize()?
    );
    assert!(
        data["targets"]
            .as_array()
            .unwrap()
            .iter()
            .any(|target| target["target"] == "codex" && target["scope"] == "workspace")
    );
    assert!(
        data["targets"]
            .as_array()
            .unwrap()
            .iter()
            .any(|target| target["target"] == "agents" && target["scope"] == "global")
    );

    fs::remove_dir_all(root)?;
    Ok(())
}

#[test]
fn installs_workspace_skill_and_reports_unchanged_second_run()
-> Result<(), Box<dyn std::error::Error>> {
    let root = temp_dir("atlas-skill-install")?;
    let first = atlas(&root)
        .args([
            "agent",
            "skills",
            "install",
            "--target",
            "codex",
            "--scope",
            "workspace",
            "--yes",
            "--json",
        ])
        .output()?;

    assert!(first.status.success());
    let skill_dir = root.join(".codex/skills/pf2e-atlas-cli");
    assert!(skill_dir.join("SKILL.md").is_file());
    assert!(skill_dir.join(".atlas-skill.json").is_file());
    let actual: Value = serde_json::from_slice(&first.stdout)?;
    let result = &ok_data(&actual)["results"][0];
    assert_eq!(result["action"], "install");
    assert_eq!(result["changed"], true);

    let second = atlas(&root)
        .args([
            "agent",
            "skills",
            "install",
            "--target",
            "codex",
            "--scope",
            "workspace",
            "--yes",
            "--json",
        ])
        .output()?;
    assert!(second.status.success());
    let actual: Value = serde_json::from_slice(&second.stdout)?;
    let result = &ok_data(&actual)["results"][0];
    assert_eq!(result["action"], "unchanged");
    assert_eq!(result["changed"], false);

    fs::remove_dir_all(root)?;
    Ok(())
}

#[test]
fn differing_install_requires_force() -> Result<(), Box<dyn std::error::Error>> {
    let root = temp_dir("atlas-skill-force")?;
    let skill_dir = root.join(".codex/skills/pf2e-atlas-cli");
    fs::create_dir_all(&skill_dir)?;
    fs::write(skill_dir.join("SKILL.md"), "edited")?;
    fs::write(
        skill_dir.join(".atlas-skill.json"),
        r#"{"content_hash":"not-current"}"#,
    )?;

    let output = atlas(&root)
        .args([
            "agent",
            "skills",
            "install",
            "--target",
            "codex",
            "--scope",
            "workspace",
            "--yes",
            "--json",
        ])
        .output()?;
    assert_eq!(output.status.code(), Some(3));
    let actual: Value = serde_json::from_slice(&output.stdout)?;
    assert_eq!(actual["status"], "error");
    assert_eq!(actual["error"]["code"], "install_failed");

    let forced = atlas(&root)
        .args([
            "agent",
            "skills",
            "install",
            "--target",
            "codex",
            "--scope",
            "workspace",
            "--yes",
            "--force",
            "--json",
        ])
        .output()?;
    assert!(forced.status.success());
    let installed_skill = fs::read_to_string(skill_dir.join("SKILL.md"))?;
    assert!(installed_skill.contains("# PF2e Atlas CLI"));
    assert!(installed_skill.contains("atlas graph links"));
    assert!(
        installed_skill
            .contains("explicit record identification followed by graph context retrieval")
    );
    assert!(installed_skill.contains("actionspf2e:1kGNdIIhuglAjIp9"));
    assert!(installed_skill.contains("spells-srd:4koZzrnMXhhosn0D"));
    assert!(installed_skill.contains("conditionitems:AJh5ex99aV6VTggg"));
    assert!(installed_skill.contains("bestiary-ability-glossary-srd:Tkd8sH4pwFIPzqTr"));

    fs::remove_dir_all(root)?;
    Ok(())
}

#[test]
fn force_reinstall_replaces_invalid_file_path() -> Result<(), Box<dyn std::error::Error>> {
    let root = temp_dir("atlas-skill-invalid-file")?;
    let skill_path = root.join(".codex/skills/pf2e-atlas-cli");
    fs::create_dir_all(skill_path.parent().unwrap())?;
    fs::write(&skill_path, "not a directory")?;

    let doctor = atlas(&root)
        .args([
            "agent",
            "skills",
            "doctor",
            "--target",
            "codex",
            "--scope",
            "workspace",
            "--json",
        ])
        .output()?;
    assert!(doctor.status.success());
    let actual: Value = serde_json::from_slice(&doctor.stdout)?;
    let target = &ok_data(&actual)["targets"][0];
    assert_eq!(target["status"], "installed_invalid");
    assert_eq!(target["action"], "reinstall");

    let blocked = atlas(&root)
        .args([
            "agent",
            "skills",
            "install",
            "--target",
            "codex",
            "--scope",
            "workspace",
            "--yes",
            "--json",
        ])
        .output()?;
    assert_eq!(blocked.status.code(), Some(3));

    let forced = atlas(&root)
        .args([
            "agent",
            "skills",
            "install",
            "--target",
            "codex",
            "--scope",
            "workspace",
            "--yes",
            "--force",
            "--json",
        ])
        .output()?;
    assert!(forced.status.success());
    assert!(skill_path.join("SKILL.md").is_file());

    fs::remove_dir_all(root)?;
    Ok(())
}

#[test]
fn edited_installed_file_requires_force_even_when_manifest_matches()
-> Result<(), Box<dyn std::error::Error>> {
    let root = temp_dir("atlas-skill-stale-manifest")?;
    let skill_dir = root.join(".codex/skills/pf2e-atlas-cli");
    let first = atlas(&root)
        .args([
            "agent",
            "skills",
            "install",
            "--target",
            "codex",
            "--scope",
            "workspace",
            "--yes",
            "--json",
        ])
        .output()?;
    assert!(first.status.success());
    fs::write(skill_dir.join("SKILL.md"), "locally edited")?;

    let doctor = atlas(&root)
        .args([
            "agent",
            "skills",
            "doctor",
            "--target",
            "codex",
            "--scope",
            "workspace",
            "--json",
        ])
        .output()?;
    assert!(doctor.status.success());
    let actual: Value = serde_json::from_slice(&doctor.stdout)?;
    let target = &ok_data(&actual)["targets"][0];
    assert_eq!(target["status"], "installed_different");
    assert_eq!(target["action"], "update");

    let reinstall = atlas(&root)
        .args([
            "agent",
            "skills",
            "install",
            "--target",
            "codex",
            "--scope",
            "workspace",
            "--yes",
            "--json",
        ])
        .output()?;
    assert_eq!(reinstall.status.code(), Some(3));

    fs::remove_dir_all(root)?;
    Ok(())
}

#[test]
fn doctor_reports_workspace_global_shadowing() -> Result<(), Box<dyn std::error::Error>> {
    let root = temp_dir("atlas-skill-shadowing")?;
    for scope in ["workspace", "global"] {
        let output = atlas(&root)
            .args([
                "agent", "skills", "install", "--target", "codex", "--scope", scope, "--yes",
                "--json",
            ])
            .output()?;
        assert!(output.status.success());
    }

    let output = atlas(&root)
        .args(["agent", "skills", "doctor", "--target", "codex", "--json"])
        .output()?;
    assert!(output.status.success());
    let actual: Value = serde_json::from_slice(&output.stdout)?;
    let shadowing = ok_data(&actual)["shadowing"].as_array().unwrap();
    assert_eq!(shadowing.len(), 1);
    assert_eq!(shadowing[0]["target"], "codex");

    fs::remove_dir_all(root)?;
    Ok(())
}

#[cfg(unix)]
#[test]
fn force_install_writes_through_symlinked_skill_directory() -> Result<(), Box<dyn std::error::Error>>
{
    let root = temp_dir("atlas-skill-symlink")?;
    let real_skill_dir = root.join("real-skill");
    let link_parent = root.join(".codex/skills");
    let link_skill_dir = link_parent.join("pf2e-atlas-cli");
    fs::create_dir_all(&real_skill_dir)?;
    fs::create_dir_all(&link_parent)?;
    fs::write(real_skill_dir.join("old.txt"), "old")?;
    std::os::unix::fs::symlink(&real_skill_dir, &link_skill_dir)?;

    let output = atlas(&root)
        .args([
            "agent",
            "skills",
            "install",
            "--target",
            "codex",
            "--scope",
            "workspace",
            "--yes",
            "--force",
            "--json",
        ])
        .output()?;

    assert!(output.status.success());
    assert!(link_skill_dir.is_symlink());
    assert!(real_skill_dir.join("SKILL.md").is_file());
    assert!(!real_skill_dir.join("old.txt").exists());

    fs::remove_dir_all(root)?;
    Ok(())
}

#[cfg(unix)]
#[test]
fn symlinked_parent_requires_force_and_reports_resolved_path()
-> Result<(), Box<dyn std::error::Error>> {
    let root = temp_dir("atlas-skill-symlink-parent")?;
    let real_codex_dir = root.join("real-codex");
    let link_codex_dir = root.join(".codex");
    fs::create_dir_all(&real_codex_dir)?;
    std::os::unix::fs::symlink(&real_codex_dir, &link_codex_dir)?;

    let blocked = atlas(&root)
        .args([
            "agent",
            "skills",
            "install",
            "--target",
            "codex",
            "--scope",
            "workspace",
            "--yes",
            "--json",
        ])
        .output()?;
    assert_eq!(blocked.status.code(), Some(3));

    let doctor = atlas(&root)
        .args([
            "agent",
            "skills",
            "doctor",
            "--target",
            "codex",
            "--scope",
            "workspace",
            "--json",
        ])
        .output()?;
    assert!(doctor.status.success());
    let actual: Value = serde_json::from_slice(&doctor.stdout)?;
    let target = &ok_data(&actual)["targets"][0];
    assert_eq!(target["status"], "symlinked");
    assert_eq!(target["symlinked"], true);
    assert!(
        target["resolved_skill_path"]
            .as_str()
            .unwrap()
            .contains("real-codex")
    );

    let forced = atlas(&root)
        .args([
            "agent",
            "skills",
            "install",
            "--target",
            "codex",
            "--scope",
            "workspace",
            "--yes",
            "--force",
            "--json",
        ])
        .output()?;
    assert!(forced.status.success());
    assert!(
        real_codex_dir
            .join("skills/pf2e-atlas-cli/SKILL.md")
            .is_file()
    );

    fs::remove_dir_all(root)?;
    Ok(())
}

#[cfg(unix)]
#[test]
fn symlinked_missing_workspace_install_does_not_shadow_global()
-> Result<(), Box<dyn std::error::Error>> {
    let root = temp_dir("atlas-skill-symlink-no-shadow")?;
    let global = atlas(&root)
        .args([
            "agent", "skills", "install", "--target", "codex", "--scope", "global", "--yes",
            "--json",
        ])
        .output()?;
    assert!(global.status.success());

    let real_codex_dir = root.join("real-codex");
    let link_codex_dir = root.join(".codex");
    fs::create_dir_all(&real_codex_dir)?;
    std::os::unix::fs::symlink(&real_codex_dir, &link_codex_dir)?;

    let doctor = atlas(&root)
        .args(["agent", "skills", "doctor", "--target", "codex", "--json"])
        .output()?;
    assert!(doctor.status.success());
    let actual: Value = serde_json::from_slice(&doctor.stdout)?;
    let data = ok_data(&actual);
    assert!(data["shadowing"].as_array().unwrap().is_empty());
    assert!(
        data["targets"]
            .as_array()
            .unwrap()
            .iter()
            .any(|target| target["scope"] == "workspace" && target["status"] == "symlinked")
    );

    fs::remove_dir_all(root)?;
    Ok(())
}

#[test]
fn json_errors_cover_missing_confirmation_and_invalid_target()
-> Result<(), Box<dyn std::error::Error>> {
    let root = temp_dir("atlas-skill-json-errors")?;
    let missing_yes = atlas(&root)
        .args([
            "agent",
            "skills",
            "install",
            "--target",
            "codex",
            "--scope",
            "workspace",
            "--json",
        ])
        .output()?;
    assert_eq!(missing_yes.status.code(), Some(2));
    let actual: Value = serde_json::from_slice(&missing_yes.stdout)?;
    assert_eq!(actual["error"]["code"], "invalid_input");

    let invalid_target = atlas(&root)
        .args([
            "agent",
            "skills",
            "install",
            "--target",
            "nope",
            "--scope",
            "workspace",
            "--yes",
            "--json",
        ])
        .output()?;
    assert_eq!(invalid_target.status.code(), Some(2));
    let actual: Value = serde_json::from_slice(&invalid_target.stdout)?;
    assert_eq!(actual["error"]["code"], "invalid_input");

    fs::remove_dir_all(root)?;
    Ok(())
}

fn atlas(root: &Path) -> Command {
    let mut command = Command::new(env!("CARGO_BIN_EXE_atlas"));
    command.current_dir(root);
    command.env("HOME", root.join("home"));
    command
}

fn ok_data(value: &Value) -> &Value {
    assert_eq!(value["status"], "ok");
    &value["data"]
}

fn temp_dir(name: &str) -> Result<PathBuf, Box<dyn std::error::Error>> {
    let nanos = SystemTime::now().duration_since(UNIX_EPOCH)?.as_nanos();
    let path = std::env::temp_dir().join(format!("{name}-{}-{nanos}", std::process::id()));
    fs::create_dir_all(&path)?;
    Ok(path)
}
