fn main() {
    let commands = tauri_build::AppManifest::new().commands(&[
        "inspect",
        "launch",
        "prepare",
        "cancel_setup",
        "enable_cli",
        "desktop_setup_action",
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
