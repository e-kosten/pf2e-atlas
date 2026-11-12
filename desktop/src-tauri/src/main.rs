#![deny(unsafe_code)]

mod opening;
mod process;
#[cfg(debug_assertions)]
mod smoke;
mod staged_setup;

use std::{
    path::{Path, PathBuf},
    sync::{
        Mutex, MutexGuard,
        atomic::{AtomicBool, AtomicUsize, Ordering},
    },
};

use atlas_desktop_protocol::{
    WorkspaceProbe, WorkspaceStatus, api_record_path, parse_record_link, parse_record_path,
    record_link, record_path,
};
use process::Launcher;
use serde::Serialize;
use tauri::{
    AppHandle, Emitter, Manager, WebviewUrl, WebviewWindow, WebviewWindowBuilder,
    menu::{MenuBuilder, MenuItem, SubmenuBuilder},
};
use tauri_plugin_clipboard_manager::ClipboardExt;
use tauri_plugin_deep_link::DeepLinkExt;
use url::Url;

#[derive(Default)]
struct Desktop {
    operation: Mutex<()>,
    staged_setup: staged_setup::StagedSetup,
    launcher: Mutex<Option<Launcher>>,
    setup_pipe: Mutex<Option<std::process::ChildStdin>>,
    cancel_requested: AtomicBool,
    links: Mutex<Vec<String>>,
    window_id: AtomicUsize,
    quitting: AtomicBool,
    recovery: AtomicBool,
    primary: Mutex<opening::PrimaryWindow>,
}

#[derive(Serialize)]
struct Inspection {
    probe: WorkspaceProbe,
    running: bool,
    recovery: bool,
    opening_error: Option<String>,
}

fn lock<T>(mutex: &Mutex<T>) -> MutexGuard<'_, T> {
    mutex
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
}

fn guard(window: &WebviewWindow) -> Result<(), String> {
    if !matches!(window.label(), "bootstrap" | opening::PRIMARY)
        || !window.url().is_ok_and(|url| bundled_origin(&url))
    {
        return Err("This action is available in the Atlas setup window.".into());
    }
    Ok(())
}

fn bundled_origin(url: &Url) -> bool {
    url.scheme() == "tauri" && url.host_str() == Some("localhost")
}

fn show_bootstrap(app: &AppHandle) -> tauri::Result<()> {
    let label = setup_window_label(app);
    let window = if let Some(window) = app.get_webview_window(label) {
        window
    } else if label == opening::PRIMARY {
        opening::build(app)?
    } else {
        WebviewWindowBuilder::new(app, label, WebviewUrl::App("index.html".into()))
            .title("Atlas Setup")
            .inner_size(600.0, 520.0)
            .min_inner_size(420.0, 420.0)
            .on_navigation(bundled_origin)
            .build()?
    };
    window.show()?;
    window.set_focus()?;
    if window.url().is_ok_and(|url| bundled_origin(&url)) {
        let _ = window.emit("atlas-inspect", ());
    }
    Ok(())
}

fn setup_window_label(app: &AppHandle) -> &'static str {
    if lock(&app.state::<Desktop>().primary).is_product() {
        "bootstrap"
    } else {
        opening::PRIMARY
    }
}

fn run_on_main(
    app: &AppHandle,
    action: impl FnOnce(&AppHandle) + Send + 'static,
) -> Result<(), String> {
    let handle = app.clone();
    app.run_on_main_thread(move || action(&handle))
        .map_err(display)
}

fn show_error(app: &AppHandle, error: String) {
    if let Some(window) = app.get_webview_window(setup_window_label(app)) {
        let _ = window.show();
        let _ = window.emit("atlas-error", error);
    }
}

#[tauri::command]
async fn inspect(window: WebviewWindow, app: AppHandle) -> Result<Inspection, String> {
    guard(&window)?;
    let initial = window.label() == opening::PRIMARY;
    tauri::async_runtime::spawn_blocking(move || {
        let desktop = app.state::<Desktop>();
        Ok(Inspection {
            probe: process::probe()?,
            running: lock(&desktop.launcher).is_some(),
            recovery: desktop.recovery.load(Ordering::Acquire),
            opening_error: if initial {
                lock(&desktop.primary).error().map(str::to_owned)
            } else {
                None
            },
        })
    })
    .await
    .map_err(display)?
}

#[tauri::command]
async fn launch(window: WebviewWindow, app: AppHandle) -> Result<(), String> {
    guard(&window)?;
    tauri::async_runtime::spawn_blocking(move || {
        let desktop = app.state::<Desktop>();
        let _operation = desktop
            .operation
            .try_lock()
            .map_err(|_| "Atlas is already working.")?;
        launch_ready(&app, &window)
    })
    .await
    .map_err(display)?
}

fn start_launcher(app: &AppHandle) -> Result<(), String> {
    let desktop = app.state::<Desktop>();
    if lock(&desktop.launcher).is_none() {
        let probe = process::probe()?;
        if !matches!(
            probe.status,
            WorkspaceStatus::Ready | WorkspaceStatus::Browsable
        ) {
            return Err(probe.message);
        }
        let handle = app.clone();
        let launcher = process::start(move |pid| on_launcher_exit(&handle, pid))?;
        if launcher.ready.workspace != probe.workspace
            || verified_url(&launcher.ready.endpoint).is_err()
        {
            return Err("The local service is using a different workspace or address.".into());
        }
        let mut current = lock(&desktop.launcher);
        if desktop.quitting.load(Ordering::Acquire) || launcher.has_stopped() {
            return Err("The Atlas launcher closed while opening the workspace.".into());
        }
        *current = Some(launcher);
        if probe.stage == Some(atlas_desktop_protocol::SetupStage::Resume) {
            desktop.staged_setup.offer_resume();
        }
    }
    desktop.recovery.store(false, Ordering::Release);
    Ok(())
}

fn launch_ready(app: &AppHandle, window: &WebviewWindow) -> Result<(), String> {
    start_launcher(app)?;
    if window.label() == opening::PRIMARY {
        return opening::open(app, window);
    }
    run_on_main(app, show_pending_or_default)
}

fn verified_url(endpoint: &str) -> Result<Url, String> {
    let url = Url::parse(endpoint).map_err(display)?;
    if url.scheme() != "http"
        || url.host_str() != Some("127.0.0.1")
        || url.port().is_none()
        || url.path() != "/"
        || !url.username().is_empty()
        || url.password().is_some()
        || url.query().is_some()
        || url.fragment().is_some()
    {
        return Err("Atlas returned an invalid local service address.".into());
    }
    Ok(url)
}

fn local_http() -> ureq::Agent {
    ureq::Agent::config_builder()
        .proxy(None)
        .max_redirects(0)
        .timeout_global(Some(std::time::Duration::from_secs(3)))
        .build()
        .into()
}

fn on_launcher_exit(app: &AppHandle, pid: u32) {
    let desktop = app.state::<Desktop>();
    let mut launcher = lock(&desktop.launcher);
    if launcher.as_ref().is_none_or(|launcher| launcher.pid != pid) {
        return;
    }
    launcher.take();
    desktop.staged_setup.clear_documents();
    desktop.recovery.store(true, Ordering::Release);
    drop(launcher);
    if !desktop.quitting.load(Ordering::Acquire) {
        let _ = run_on_main(app, |app| {
            let _ = show_bootstrap(app);
        });
    }
}

#[tauri::command]
async fn prepare(
    window: WebviewWindow,
    app: AppHandle,
    build_embeddings_locally: Option<bool>,
) -> Result<String, serde_json::Value> {
    guard(&window)?;
    tauri::async_runtime::spawn_blocking(move || {
        let staged = process::probe()?.stage;
        if let Some(stage) = staged {
            return staged_setup::from_bootstrap(
                app,
                window,
                stage,
                build_embeddings_locally.unwrap_or_default(),
            );
        }
        let desktop = app.state::<Desktop>();
        let _operation = desktop
            .operation
            .try_lock()
            .map_err(|_| "Atlas is already working.")?;
        if desktop.staged_setup.active() {
            return Err("Atlas setup is already running.".into());
        }
        desktop.cancel_requested.store(false, Ordering::Release);
        let probe = process::probe()?;
        let upgrade = probe.status == WorkspaceStatus::UpgradeRequired;
        let running = lock(&desktop.launcher).is_some();
        if upgrade {
            let launcher = lock(&desktop.launcher).take();
            if let Some(launcher) = launcher {
                launcher.release()?;
            }
            process::wait_for_upgrade()?;
        }
        let progress_window = window.clone();
        let result = process::setup(
            None,
            build_embeddings_locally.unwrap_or_default(),
            move |event| {
                let _ = progress_window.emit("atlas-progress", event);
            },
            |_| {},
            |pipe| {
                let mut held = lock(&desktop.setup_pipe);
                if !desktop.cancel_requested.load(Ordering::Acquire)
                    && !desktop.quitting.load(Ordering::Acquire)
                {
                    *held = Some(pipe);
                }
            },
        );
        lock(&desktop.setup_pipe).take();
        result?;
        if upgrade || !running {
            let _ = window.emit("atlas-opening", ());
            launch_ready(&app, &window)?;
            return Ok("Your workspace is ready.".into());
        }
        run_on_main(&app, |app| refresh_product(app, false))?;
        Ok("Reference data is up to date.".into())
    })
    .await
    .map_err(display)?
}

#[tauri::command]
fn cancel_setup(window: WebviewWindow, app: AppHandle) -> Result<(), String> {
    guard(&window)?;
    let desktop = app.state::<Desktop>();
    if desktop.staged_setup.active() {
        return staged_setup::cancel(&app, None);
    }
    desktop.cancel_requested.store(true, Ordering::Release);
    lock(&desktop.setup_pipe).take();
    Ok(())
}

fn refresh_product(app: &AppHandle, semantic_only: bool) {
    let desktop = app.state::<Desktop>();
    let origin = lock(&desktop.launcher)
        .as_ref()
        .and_then(|launcher| verified_url(&launcher.ready.endpoint).ok())
        .map(|endpoint| endpoint.origin());
    for window in app.webview_windows().into_values() {
        if window.label().starts_with("atlas-")
            && window.url().is_ok_and(|url| Some(url.origin()) == origin)
        {
            let detail = if semantic_only { "'semantic'" } else { "null" };
            let _ = window.eval(format!(
                "window.dispatchEvent(new CustomEvent('atlas-reference-updated', {{ detail: {detail} }}))"
            ));
        }
    }
}

fn show_pending_or_default(app: &AppHandle) {
    let desktop = app.state::<Desktop>();
    let mut routes = std::mem::take(&mut *lock(&desktop.links));
    if routes.is_empty() {
        let service = lock(&desktop.launcher).as_ref().and_then(|launcher| {
            let ready = &launcher.ready;
            Some((
                verified_url(&ready.endpoint).ok()?.origin(),
                ready.instance.clone(),
            ))
        });
        let existing = app.webview_windows().into_values().find(|window| {
            service.as_ref().is_some_and(|(origin, instance)| {
                desktop.staged_setup.attached_to(window.label(), instance)
                    && window.url().is_ok_and(|url| &url.origin() == origin)
            })
        });
        if let Some(window) = existing {
            let _ = window.show();
            let _ = window.set_focus();
        } else {
            routes.push("/search".into());
        }
    }
    let mut routes = routes.into_iter();
    while let Some(route) = routes.next() {
        if let Err(error) = open_product(app, &route, false) {
            lock(&desktop.links).splice(0..0, std::iter::once(route).chain(routes));
            show_error(app, error);
            return;
        }
    }
    if let Some(window) = app.get_webview_window("bootstrap") {
        let _ = window.hide();
    }
}

fn open_product(app: &AppHandle, route: &str, new_window: bool) -> Result<(), String> {
    let desktop = app.state::<Desktop>();
    let (base, instance) = {
        let current = lock(&desktop.launcher);
        let ready = &current
            .as_ref()
            .ok_or("Start the Atlas workspace first.")?
            .ready;
        (verified_url(&ready.endpoint)?, ready.instance.clone())
    };
    let url = base.join(route).map_err(display)?;
    let origin = base.origin();
    if !new_window {
        for window in app
            .webview_windows()
            .into_values()
            .filter(|window| window.label().starts_with("atlas-"))
        {
            if desktop.staged_setup.attached_to(window.label(), &instance)
                && window
                    .url()
                    .is_ok_and(|open| same_product_route(&url, &open))
            {
                window.show().map_err(display)?;
                return window.set_focus().map_err(display);
            }
        }
    }
    let id = desktop.window_id.fetch_add(1, Ordering::Relaxed);
    let label = format!("atlas-{id}");
    desktop
        .staged_setup
        .attach_window(app, &label, &base, &instance)?;
    WebviewWindowBuilder::new(app, label, WebviewUrl::External(url))
        .on_page_load(staged_setup::page_load)
        .title("PF2e Atlas")
        .inner_size(1210.0, 830.0)
        .min_inner_size(480.0, 440.0)
        .on_navigation(move |destination| {
            if destination.origin() == origin {
                return true;
            }
            open_external(destination);
            false
        })
        .on_new_window(|destination, _| {
            open_external(&destination);
            tauri::webview::NewWindowResponse::Deny
        })
        .build()
        .map_err(display)?;
    Ok(())
}

fn open_external(destination: &Url) {
    if matches!(destination.scheme(), "https" | "http" | "mailto") {
        let _ = std::process::Command::new("/usr/bin/open")
            .arg(destination.as_str())
            .spawn();
    }
}

fn same_product_route(expected: &Url, current: &Url) -> bool {
    expected.origin() == current.origin()
        && (expected.path() == current.path()
            || parse_record_path(expected.path())
                .is_some_and(|key| parse_record_path(current.path()).as_ref() == Some(&key)))
}

fn receive_link(app: &AppHandle, link: &str) {
    let key = match parse_record_link(link) {
        Ok(key) => key,
        Err(error) => {
            let _ = show_bootstrap(app);
            show_error(app, error.into());
            return;
        }
    };
    let route = record_path(&key);
    let desktop = app.state::<Desktop>();
    if setup_window_label(app) != opening::PRIMARY && lock(&desktop.launcher).is_some() {
        if let Err(error) = open_product(app, &route, false) {
            show_error(app, error);
        }
    } else {
        lock(&desktop.links).push(route);
        let _ = show_bootstrap(app);
    }
}

fn copy_record(app: &AppHandle) -> Result<(), String> {
    let desktop = app.state::<Desktop>();
    let ready = lock(&desktop.launcher)
        .as_ref()
        .ok_or("Start the Atlas workspace first.")?
        .ready
        .clone();
    let base = verified_url(&ready.endpoint)?;
    let window = app
        .webview_windows()
        .into_values()
        .find(|window| window.label().starts_with("atlas-") && window.is_focused().unwrap_or(false))
        .ok_or("Open an Atlas record first.")?;
    let url = window.url().map_err(display)?;
    let key = (url.origin() == base.origin())
        .then(|| parse_record_path(url.path()))
        .flatten()
        .ok_or("Open an Atlas record first.")?;
    let link = record_link(&key);
    let api = base.join(&api_record_path(&key)).map_err(display)?;
    let handle = app.clone();
    std::thread::spawn(move || {
        let result = (|| -> Result<(), String> {
            let record: serde_json::Value = local_http()
                .get(api.as_str())
                .header("x-atlas-api-contract", &ready.api_contract)
                .call()
                .map_err(display)?
                .body_mut()
                .read_json()
                .map_err(display)?;
            let title = record["record"]["title"]
                .as_str()
                .ok_or("The record name is unavailable.")?;
            let html = format!("<a href=\"{}\">{}</a>", escape(&link), escape(title));
            handle
                .clipboard()
                .write_html(html, Some(link))
                .map_err(display)
        })();
        if let Err(error) = result {
            let _ = run_on_main(&handle, move |app| {
                let _ = show_bootstrap(app);
                show_error(app, error);
            });
        }
    });
    Ok(())
}

fn escape(value: &str) -> String {
    value
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}

#[tauri::command]
async fn enable_cli(window: WebviewWindow, directory: String) -> Result<String, String> {
    guard(&window)?;
    let home = process::home()?;
    let path = directory
        .strip_prefix("~/")
        .map_or_else(|| PathBuf::from(&directory), |relative| home.join(relative));
    link_cli(&home, &path, &process::bundled_cli()?)
}

fn link_cli(home: &Path, directory: &Path, executable: &Path) -> Result<String, String> {
    let home = home.canonicalize().map_err(display)?;
    let candidate = if directory == home.join(".local/bin") && !directory.exists() {
        let parent = home.join(".local");
        if !parent.exists() {
            std::fs::create_dir(&parent).map_err(display)?;
        }
        if parent.canonicalize().map_err(display)? != parent {
            return Err("Choose a bin folder inside your home folder.".into());
        }
        std::fs::create_dir(directory).map_err(display)?;
        directory.to_owned()
    } else {
        directory.canonicalize().map_err(display)?
    };
    if !candidate.starts_with(&home) || !candidate.is_dir() {
        return Err("Choose an existing bin folder inside your home folder.".into());
    }
    let destination = candidate.join("atlas");
    match std::os::unix::fs::symlink(executable, &destination) {
        Ok(()) => Ok(destination.display().to_string()),
        Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => {
            if std::fs::read_link(&destination).is_ok_and(|path| path == executable) {
                Ok(destination.display().to_string())
            } else {
                Err(format!(
                    "An atlas command already exists at {}. Choose another folder to preserve it.",
                    destination.display()
                ))
            }
        }
        Err(error) => Err(display(error)),
    }
}

fn display(error: impl std::fmt::Display) -> String {
    error.to_string()
}

fn menu(app: &AppHandle) -> tauri::Result<tauri::menu::Menu<tauri::Wry>> {
    let settings = MenuItem::with_id(
        app,
        "setup",
        "Setup and Updates…",
        true,
        Some("CmdOrCtrl+,"),
    )?;
    let window = MenuItem::with_id(app, "new", "New Window", true, Some("CmdOrCtrl+N"))?;
    let copy = MenuItem::with_id(app, "copy-link", "Copy Atlas Link", true, None::<&str>)?;
    let atlas = SubmenuBuilder::new(app, "PF2e Atlas")
        .item(&settings)
        .separator()
        .quit()
        .build()?;
    let file = SubmenuBuilder::new(app, "File")
        .item(&window)
        .close_window()
        .build()?;
    let edit = SubmenuBuilder::new(app, "Edit")
        .undo()
        .redo()
        .separator()
        .cut()
        .copy()
        .paste()
        .select_all()
        .separator()
        .item(&copy)
        .build()?;
    let window = SubmenuBuilder::new(app, "Window")
        .minimize()
        .maximize()
        .build()?;
    MenuBuilder::new(app)
        .items(&[&atlas, &file, &edit, &window])
        .build()
}

fn main() {
    let result = tauri::Builder::default()
        .plugin(tauri_plugin_deep_link::init())
        .plugin(tauri_plugin_clipboard_manager::init())
        .manage(Desktop::default())
        .invoke_handler(tauri::generate_handler![
            inspect,
            launch,
            prepare,
            cancel_setup,
            enable_cli,
            staged_setup::desktop_setup_action
        ])
        .menu(menu)
        .on_window_event(|window, event| {
            if window.label() == setup_window_label(window.app_handle())
                && let tauri::WindowEvent::CloseRequested { api, .. } = event
            {
                api.prevent_close();
                let _ = window.hide();
            }
        })
        .on_menu_event(|app, event| {
            let result = match event.id().as_ref() {
                "setup" => opening::restore_setup_if_needed(app)
                    .and_then(|()| show_bootstrap(app))
                    .map_err(display),
                "new" => {
                    let running = lock(&app.state::<Desktop>().launcher).is_some();
                    if running && setup_window_label(app) != opening::PRIMARY {
                        open_product(app, "/search", true)
                    } else {
                        show_bootstrap(app).map_err(display)
                    }
                }
                "copy-link" => copy_record(app),
                _ => Ok(()),
            };
            if let Err(error) = result {
                let _ = show_bootstrap(app);
                show_error(app, error);
            }
        })
        .setup(|app| {
            let handle = app.handle().clone();
            app.deep_link().on_open_url(move |event| {
                let urls = event.urls();
                let _ = run_on_main(&handle, move |app| {
                    for link in urls {
                        receive_link(app, link.as_str());
                    }
                });
            });
            if let Some(urls) = app.deep_link().get_current()? {
                for link in urls {
                    receive_link(app.handle(), link.as_str());
                }
            }
            show_bootstrap(app.handle())?;
            #[cfg(debug_assertions)]
            smoke::start(app.handle())?;
            Ok(())
        })
        .build(tauri::generate_context!());
    match result {
        Ok(app) => app.run(|app, event| match event {
            tauri::RunEvent::ExitRequested {
                code: None, api, ..
            } => api.prevent_exit(),
            tauri::RunEvent::Exit => {
                let desktop = app.state::<Desktop>();
                desktop.quitting.store(true, Ordering::Release);
                lock(&desktop.launcher).take();
                lock(&desktop.setup_pipe).take();
            }
            tauri::RunEvent::Reopen {
                has_visible_windows: false,
                ..
            } => {
                if setup_window_label(app) != opening::PRIMARY
                    && lock(&app.state::<Desktop>().launcher).is_some()
                {
                    show_pending_or_default(app);
                } else {
                    let _ = show_bootstrap(app);
                }
            }
            _ => {}
        }),
        Err(error) => {
            eprintln!("Could not start PF2e Atlas: {error}");
            std::process::exit(1);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn opening_keeps_record_identity_across_router_normalization() {
        let requested = Url::parse("http://127.0.0.1:4732/search/records/actions%3Aone").unwrap();
        let normalized = Url::parse("http://127.0.0.1:4732/records/actions:one?mode=read").unwrap();
        let other = Url::parse("http://127.0.0.1:4732/records/actions:two").unwrap();
        let foreign = Url::parse("http://127.0.0.1:4733/records/actions:one").unwrap();
        assert!(same_product_route(&requested, &normalized));
        assert!(!same_product_route(&requested, &other));
        assert!(!same_product_route(&requested, &foreign));
    }

    #[test]
    fn native_context_and_service_origins_are_distinct() {
        assert!(bundled_origin(&Url::parse("tauri://localhost/").unwrap()));
        assert!(!bundled_origin(
            &Url::parse("http://127.0.0.1:3000/").unwrap()
        ));
        assert!(verified_url("http://127.0.0.1:3000/").is_ok());
        for address in [
            "http://localhost:3000/",
            "http://127.0.0.1/",
            "http://127.0.0.1:3000/?other",
            "http://user@127.0.0.1:3000/",
            "https://127.0.0.1:3000/",
        ] {
            assert!(verified_url(address).is_err(), "{address}");
        }
    }

    #[test]
    fn terminal_exposure_preserves_an_existing_command() {
        let root = tempfile::tempdir().unwrap();
        let bin = root.path().join("bin");
        let command = bin.join("atlas");
        let bundled = root.path().join("bundled");
        std::fs::create_dir(&bin).unwrap();
        std::fs::write(&command, "existing CLI").unwrap();
        assert!(link_cli(root.path(), &bin, &bundled).is_err());
        assert_eq!(std::fs::read_to_string(&command).unwrap(), "existing CLI");
        std::fs::remove_file(&command).unwrap();
        assert!(link_cli(root.path(), &bin, &bundled).is_ok());
        assert!(link_cli(root.path(), &bin, &bundled).is_ok());
        assert_eq!(std::fs::read_link(command).unwrap(), bundled);
    }
}
