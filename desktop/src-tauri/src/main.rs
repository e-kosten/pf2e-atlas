#![deny(unsafe_code)]

mod native_tabs;
mod opening;
mod process;
#[cfg(debug_assertions)]
mod smoke;
mod staged_setup;
mod tab_links;
mod terminal;
mod zoom;

use std::{
    collections::VecDeque,
    path::PathBuf,
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
    links: Mutex<VecDeque<String>>,
    focus_order: Mutex<Vec<String>>,
    context_link: Mutex<Option<tab_links::Link>>,
    draining_links: AtomicBool,
    zoom: zoom::Zoom,
    window_id: AtomicUsize,
    quitting: AtomicBool,
    recovery: AtomicBool,
    primary: Mutex<opening::PrimaryWindow>,
    terminal_operation: Mutex<()>,
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
    if let Err(error) = native_tabs::configure(&window, false) {
        eprintln!("Could not configure Atlas setup window: {error}");
    }
    zoom::apply(&window);
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
            if upgrade {
                process::SetupMode::UpgradeWorkspace
            } else {
                process::SetupMode::Ordinary
            },
            !upgrade && build_embeddings_locally.unwrap_or_default(),
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
        if upgrade
            && !matches!(
                process::probe()?.status,
                WorkspaceStatus::Ready | WorkspaceStatus::Browsable
            )
        {
            return Ok("Workspace format upgraded.".into());
        }
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
    let service = lock(&desktop.launcher).as_ref().and_then(|launcher| {
        let ready = &launcher.ready;
        Some((
            verified_url(&ready.endpoint).ok()?.origin(),
            ready.instance.clone(),
        ))
    });
    let Some((origin, instance)) = service else {
        return;
    };
    for window in app.webview_windows().into_values() {
        if desktop.staged_setup.has_document(window.label(), &instance)
            && window.url().is_ok_and(|url| url.origin() == origin)
        {
            let detail = if semantic_only { "'semantic'" } else { "null" };
            let _ = window.eval(format!(
                "window.dispatchEvent(new CustomEvent('atlas-reference-updated', {{ detail: {detail} }}))"
            ));
        }
    }
}

fn show_pending_or_default(app: &AppHandle) {
    let pending = !lock(&app.state::<Desktop>().links).is_empty();
    let result = if !pending {
        current_product(app).and_then(|(base, instance)| {
            if let Some(window) = recent_product_views(app, &base, &instance).first() {
                native_tabs::select(window)
            } else {
                open_product(app, "/search", None).map(|_| ())
            }
        })
    } else {
        drain_links(app)
    };
    if let Err(error) = result {
        report_link_error(app, &error);
    } else if let Some(window) = app.get_webview_window("bootstrap") {
        let _ = window.hide();
    }
}

fn current_product(app: &AppHandle) -> Result<(Url, String), String> {
    let desktop = app.state::<Desktop>();
    let launcher = lock(&desktop.launcher);
    let ready = &launcher
        .as_ref()
        .ok_or("Start the Atlas workspace first.")?
        .ready;
    Ok((verified_url(&ready.endpoint)?, ready.instance.clone()))
}

fn product_view(window: &WebviewWindow, base: &Url, instance: &str) -> bool {
    let desktop = window.state::<Desktop>();
    (window.label() != opening::PRIMARY || lock(&desktop.primary).is_product())
        && desktop.staged_setup.attached_to(window.label(), instance)
        && window.url().is_ok_and(|url| url.origin() == base.origin())
}

fn remember_product_focus(app: &AppHandle, label: &str) {
    if let Ok((base, instance)) = current_product(app)
        && app
            .get_webview_window(label)
            .is_some_and(|window| product_view(&window, &base, &instance))
    {
        let desktop = app.state::<Desktop>();
        let mut order = lock(&desktop.focus_order);
        order.retain(|previous| previous != label);
        order.insert(0, label.into());
    }
}

fn recent_product_views(app: &AppHandle, base: &Url, instance: &str) -> Vec<WebviewWindow> {
    let desktop = app.state::<Desktop>();
    let order = lock(&desktop.focus_order).clone();
    let mut windows = app
        .webview_windows()
        .into_values()
        .filter(|window| product_view(window, base, instance))
        .collect::<Vec<_>>();
    windows.sort_by_key(|window| {
        (
            !window.is_focused().unwrap_or(false),
            order
                .iter()
                .position(|label| label == window.label())
                .unwrap_or(usize::MAX),
            window.label().to_owned(),
        )
    });
    windows
}

fn open_product(
    app: &AppHandle,
    route: &str,
    tab: Option<(&WebviewWindow, bool)>,
) -> Result<WebviewWindow, String> {
    let desktop = app.state::<Desktop>();
    let (base, instance) = current_product(app)?;
    let url = base.join(route).map_err(display)?;
    let origin = base.origin();
    if url.origin() != origin {
        return Err("The destination is outside the current Atlas workspace.".into());
    }
    let id = desktop.window_id.fetch_add(1, Ordering::Relaxed);
    let label = format!("atlas-{id}");
    let (width, height) = if let Some((source, _)) = tab {
        let size = source.inner_size().map_err(display)?;
        let size = size.to_logical::<f64>(source.scale_factor().map_err(display)?);
        (size.width, size.height)
    } else {
        (1210.0, 830.0)
    };
    desktop
        .staged_setup
        .attach_window(app, &label, &base, &instance)?;
    let window = WebviewWindowBuilder::new(app, &label, WebviewUrl::External(url))
        .initialization_script(tab_links::SCRIPT)
        .on_page_load(staged_setup::page_load)
        .title("PF2e Atlas")
        .visible(false)
        .inner_size(width, height)
        .min_inner_size(480.0, 440.0)
        .on_navigation(move |destination| {
            if destination.origin() == origin {
                return true;
            }
            open_external(destination);
            false
        })
        .on_new_window(move |destination, _| {
            if destination.origin() != base.origin() {
                open_external(&destination);
            }
            tauri::webview::NewWindowResponse::Deny
        })
        .build()
        .map_err(display)
        .inspect_err(|_| desktop.staged_setup.detach_window(&label))?;
    zoom::apply(&window);
    let placement = native_tabs::configure(&window, true).and_then(|()| {
        if let Some((source, foreground)) = tab {
            native_tabs::attach(source, &window, foreground)
        } else {
            native_tabs::select(&window)
        }
    });
    if let Err(error) = placement {
        let _ = window.destroy();
        desktop.staged_setup.detach_window(&label);
        return Err(error);
    }
    Ok(window)
}

fn new_search(app: &AppHandle, as_tab: bool) -> Result<(), String> {
    if setup_window_label(app) == opening::PRIMARY || current_product(app).is_err() {
        return show_bootstrap(app).map_err(display);
    }
    let (base, instance) = current_product(app)?;
    let source = as_tab
        .then(|| recent_product_views(app, &base, &instance))
        .and_then(|views| views.into_iter().next());
    open_product(app, "/search", source.as_ref().map(|window| (window, true))).map(|_| ())
}

fn show_workspace_setup(app: &AppHandle, terminal: bool) -> Result<(), String> {
    let desktop = app.state::<Desktop>();
    if let Ok((base, instance)) = current_product(app) {
        for window in recent_product_views(app, &base, &instance) {
            if desktop
                .staged_setup
                .request_open(&window, &instance, false, terminal)
            {
                return native_tabs::select(&window);
            }
        }
        if lock(&desktop.primary).is_product() {
            let window = open_product(app, "/search", None)?;
            if desktop
                .staged_setup
                .request_open(&window, &instance, true, terminal)
            {
                return Ok(());
            }
        }
    }
    opening::restore_setup_if_needed(app)
        .and_then(|()| show_bootstrap(app))
        .map_err(display)
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
    lock(&desktop.links).push_back(route);
    if setup_window_label(app) == opening::PRIMARY || current_product(app).is_err() {
        let _ = show_bootstrap(app);
    } else if let Err(error) = drain_links(app) {
        report_link_error(app, &error);
    }
}

fn drain_links(app: &AppHandle) -> Result<(), String> {
    let desktop = app.state::<Desktop>();
    if desktop.draining_links.swap(true, Ordering::Relaxed) {
        return Ok(());
    }
    let result = deliver_links(app, &desktop);
    desktop.draining_links.store(false, Ordering::Relaxed);
    result
}

fn deliver_links(app: &AppHandle, desktop: &Desktop) -> Result<(), String> {
    loop {
        let Some(route) = lock(&desktop.links).front().cloned() else {
            return Ok(());
        };
        let (base, instance) = current_product(app)?;
        let target = base.join(&route).map_err(display)?;
        let windows = recent_product_views(app, &base, &instance);
        if let Some(window) = windows.iter().find(|window| {
            window
                .url()
                .is_ok_and(|url| same_product_route(&target, &url))
        }) {
            native_tabs::select(window)?;
        } else {
            open_product(app, &route, windows.first().map(|window| (window, true)))?;
        }
        lock(&desktop.links).pop_front();
    }
}

fn report_link_error(app: &AppHandle, error: &str) {
    if native_tabs::offer_retry("Could not open Atlas record link", error)
        && let Err(error) = drain_links(app)
    {
        native_tabs::report("Could not open Atlas record link", &error);
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
    let window = recent_product_views(app, &base, &ready.instance)
        .into_iter()
        .find(|window| window.is_focused().unwrap_or(false))
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
    tauri::async_runtime::spawn_blocking(move || {
        let desktop = window.state::<Desktop>();
        let _operation = desktop
            .terminal_operation
            .try_lock()
            .map_err(|_| "A terminal command action is already in progress.")?;
        let home = process::home()?.canonicalize().map_err(display)?;
        let path = directory.trim().strip_prefix("~/").map_or_else(
            || PathBuf::from(directory.trim()),
            |relative| home.join(relative),
        );
        let bundled = terminal::Bundled::inspect()?;
        guard(&window)?;
        terminal::link_from_bootstrap(&home, &path, &bundled)
    })
    .await
    .map_err(display)?
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
    let terminal = MenuItem::with_id(
        app,
        "terminal-access",
        "Terminal and Agent Access…",
        true,
        None::<&str>,
    )?;
    let copy = MenuItem::with_id(app, "copy-link", "Copy Atlas Link", true, None::<&str>)?;
    let atlas = SubmenuBuilder::new(app, "PF2e Atlas")
        .item(&settings)
        .item(&terminal)
        .separator()
        .quit()
        .build()?;
    let tab = MenuItem::with_id(app, "new-tab", "New Tab", true, Some("CmdOrCtrl+T"))?;
    let file = SubmenuBuilder::new(app, "File")
        .item(&window)
        .item(&tab)
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
    let zoom_in = MenuItem::with_id(app, "zoom-in", "Zoom In", true, Some("CmdOrCtrl+Equal"))?;
    let zoom_plus = MenuItem::with_id(app, "zoom-plus", "Zoom In (+)", true, None::<&str>)?;
    let zoom_out = MenuItem::with_id(app, "zoom-out", "Zoom Out", true, Some("CmdOrCtrl+Minus"))?;
    let zoom_reset =
        MenuItem::with_id(app, "zoom-reset", "Actual Size", true, Some("CmdOrCtrl+0"))?;
    let view = SubmenuBuilder::new(app, "View")
        .items(&[&zoom_in, &zoom_plus, &zoom_out, &zoom_reset])
        .build()?;
    let window = SubmenuBuilder::new(app, "Window")
        .minimize()
        .maximize()
        .build()?;
    MenuBuilder::new(app)
        .items(&[&atlas, &file, &edit, &view, &window])
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
            staged_setup::desktop_setup_action,
            tab_links::desktop_tab_link,
            native_tabs::desktop_appearance
        ])
        .menu(menu)
        .on_window_event(|window, event| {
            let app = window.app_handle();
            if matches!(event, tauri::WindowEvent::Focused(true)) {
                remember_product_focus(app, window.label());
            }
            if matches!(event, tauri::WindowEvent::Destroyed) {
                let desktop = app.state::<Desktop>();
                desktop.staged_setup.detach_window(window.label());
                lock(&desktop.focus_order).retain(|label| label != window.label());
            }
            if window.label() == setup_window_label(window.app_handle())
                && let tauri::WindowEvent::CloseRequested { api, .. } = event
            {
                api.prevent_close();
                let _ = window.hide();
            }
        })
        .on_menu_event(|app, event| {
            let result = match event.id().as_ref() {
                "setup" => show_workspace_setup(app, false),
                "terminal-access" => show_workspace_setup(app, true),
                "new" => new_search(app, false),
                "new-tab" => new_search(app, true),
                tab_links::MENU_ID => tab_links::open_context_link(app),
                action @ ("zoom-in" | "zoom-plus" | "zoom-out" | "zoom-reset") => {
                    if let Err(error) = app.state::<Desktop>().zoom.change(app, action) {
                        native_tabs::report("Could not update Atlas zoom", &error);
                    }
                    Ok(())
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
            native_tabs::disable_automatic_grouping().map_err(std::io::Error::other)?;
            native_tabs::configure_zoom_shortcut();
            app.state::<Desktop>().zoom.load(app.handle());
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
                has_visible_windows,
                ..
            } => {
                let pending = !lock(&app.state::<Desktop>().links).is_empty();
                if has_visible_windows && !pending {
                    return;
                }
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
}
