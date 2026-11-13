fn main() {
    let commands = tauri_build::AppManifest::new().commands(&[
        "inspect",
        "launch",
        "prepare",
        "cancel_setup",
        "enable_cli",
        "desktop_setup_action",
        "desktop_tab_link",
        "desktop_appearance",
    ]);
    tauri_build::try_build(tauri_build::Attributes::new().app_manifest(commands))
        .expect("configure the Atlas desktop bundle");
}
