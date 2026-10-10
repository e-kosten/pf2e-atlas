use std::process::Command;
pub fn atlas_command() -> Command {
    let mut command = Command::new(env!("CARGO_BIN_EXE_atlas"));
    for name in ["GIT_DIR", "GIT_WORK_TREE", "GIT_INDEX_FILE", "GIT_PREFIX"] {
        command.env_remove(name);
    }
    command
}
