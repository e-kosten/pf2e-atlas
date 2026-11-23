fn main() {
    for name in [
        "ATLAS_DESKTOP_RELEASE",
        "ATLAS_DESKTOP_SOURCE_COMMIT",
        "ATLAS_UPDATER_PUBLIC_KEY",
        "ATLAS_WORKSPACE_SCHEMA",
        "ATLAS_UPDATER_REHEARSAL_ROOT",
        "ATLAS_UPDATER_REHEARSAL_CA",
    ] {
        println!("cargo:rerun-if-env-changed={name}");
    }
    let commands = tauri_build::AppManifest::new().commands(&[
        "inspect",
        "recover_workspace",
        "launch",
        "prepare",
        "cancel_setup",
        "enable_cli",
        "desktop_setup_action",
        "desktop_settings_open",
        "desktop_settings_ack",
        "desktop_settings_pane",
        "desktop_update_action",
        "desktop_channel_action",
        "desktop_startup_trace",
        "desktop_tab_link",
        "desktop_native_tab_title",
        "desktop_appearance",
        "desktop_copy_link",
        "desktop_copy_availability",
        "desktop_tabs_state",
        "desktop_tabs_ack",
        "desktop_tabs_close",
        "desktop_tabs_new_window",
        "desktop_tabs_exit_status",
    ]);
    tauri_build::try_build(tauri_build::Attributes::new().app_manifest(commands))
        .expect("configure the Atlas desktop bundle");
}
