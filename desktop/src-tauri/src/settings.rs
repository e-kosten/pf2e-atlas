//! Desktop Settings destinations and their native window or tab.
use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Manager, WebviewWindow};
use url::Url;

use crate::{Desktop, current_product, display, lock, opening, window_host};

pub const WINDOW: &str = "atlas-settings";

#[derive(Clone, Copy, Debug, Deserialize, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Destination {
    General,
    Updates,
    Channel,
    Data,
    Terminal,
}

impl Destination {
    fn from_url(url: &Url) -> Option<Self> {
        if url.query().is_some() || url.fragment().is_some() {
            return None;
        }
        match url.path().trim_end_matches('/') {
            "/settings" => Some(Self::General),
            "/settings/updates" => Some(Self::Updates),
            "/settings/channel" => Some(Self::Channel),
            "/settings/data" => Some(Self::Data),
            "/settings/terminal" => Some(Self::Terminal),
            _ => None,
        }
    }

    #[cfg(target_os = "macos")]
    fn path(self) -> &'static str {
        match self {
            Self::Updates | Self::General => "/settings/updates",
            Self::Channel => "/settings/channel",
            Self::Data => "/settings/data",
            Self::Terminal => "/settings/terminal",
        }
    }

    #[cfg(target_os = "macos")]
    fn pane(self) -> (&'static str, Self) {
        match self {
            Self::Data => ("Search & data", Self::Data),
            Self::Terminal => ("Terminal", Self::Terminal),
            _ => ("Updates", Self::Updates),
        }
    }
}

pub fn internal_route(base: &Url, url: &Url) -> bool {
    url.origin() == base.origin()
        && url.username().is_empty()
        && url.password().is_none()
        && Destination::from_url(url).is_some()
}

fn authorize(window: &WebviewWindow, token: &str) -> Result<(), String> {
    let app = window.app_handle();
    let (_, instance) = current_product(app)?;
    if !app
        .state::<Desktop>()
        .staged_setup
        .authorize_document(window, &instance, token)
    {
        return Err("Open Settings again in the current Atlas window.".into());
    }
    Ok(())
}

#[tauri::command]
pub async fn desktop_settings_open(
    window: WebviewWindow,
    token: String,
    destination: Destination,
) -> Result<(), String> {
    let app = window.app_handle().clone();
    let (tx, rx) = std::sync::mpsc::sync_channel(1);
    crate::run_on_main(&app, move |app| {
        let result =
            authorize(&window, &token).and_then(|()| open(app, Some(&window), destination));
        let _ = tx.send(result);
    })?;
    tauri::async_runtime::spawn_blocking(move || rx.recv().map_err(display)?)
        .await
        .map_err(display)?
}

#[tauri::command]
pub fn desktop_settings_ack(window: WebviewWindow, token: String, id: usize) -> Result<(), String> {
    authorize(&window, &token)?;
    window
        .state::<Desktop>()
        .staged_setup
        .ack_settings(window.label(), &token, id);
    Ok(())
}

pub fn open(
    app: &AppHandle,
    source: Option<&WebviewWindow>,
    destination: Destination,
) -> Result<(), String> {
    let product_ready = lock(&app.state::<Desktop>().primary).is_product();
    if let Ok((base, instance)) = current_product(app)
        && product_ready
    {
        #[cfg(target_os = "macos")]
        {
            let _ = source;
            return open_mac(app, &base, &instance, destination);
        }
        #[cfg(windows)]
        {
            if let Some(window) = source {
                return present_windows(window, &instance, destination);
            }
            return open_windows_menu(app, &base, instance, destination);
        }
    }
    if destination == Destination::Terminal {
        #[cfg(windows)]
        return window_host::show_cli_path(app);
        #[cfg(target_os = "macos")]
        app.state::<Desktop>()
            .bootstrap_terminal
            .store(true, std::sync::atomic::Ordering::Release);
    }
    opening::restore_setup_if_needed(app)
        .and_then(|()| crate::show_bootstrap(app))
        .map_err(display)?;
    #[cfg(target_os = "macos")]
    if destination == Destination::Terminal
        && let Some(window) = app.get_webview_window(crate::setup_window_label(app))
        && window.url().is_ok_and(|url| crate::bundled_origin(&url))
    {
        use tauri::Emitter;
        window.emit("atlas-terminal-access", ()).map_err(display)?;
    }
    Ok(())
}

#[cfg(target_os = "macos")]
fn preference_path(app: &AppHandle) -> Result<std::path::PathBuf, String> {
    #[cfg(debug_assertions)]
    if let Some(home) = std::env::var_os("ATLAS_PILOT_HOME") {
        return Ok(std::path::PathBuf::from(home).join("settings-pane"));
    }
    Ok(app
        .path()
        .app_config_dir()
        .map_err(display)?
        .join("settings-pane"))
}

#[cfg(target_os = "macos")]
fn restored_pane(app: &AppHandle) -> Destination {
    preference_path(app)
        .ok()
        .and_then(|path| read_pane(&path))
        .unwrap_or(Destination::Updates)
}

#[cfg(target_os = "macos")]
fn read_pane(path: &std::path::Path) -> Option<Destination> {
    let bytes = std::fs::read(path).ok()?;
    let destination = serde_json::from_slice::<Destination>(&bytes).ok()?;
    Some(destination.pane().1)
}

#[cfg(target_os = "macos")]
#[tauri::command]
pub fn desktop_settings_pane(
    window: WebviewWindow,
    token: String,
    destination: Destination,
) -> Result<(), String> {
    if window.label() != WINDOW {
        return Err("Open the Atlas Settings window.".into());
    }
    authorize(&window, &token)?;
    let (title, pane) = destination.pane();
    window.set_title(title).map_err(display)?;
    let path = preference_path(window.app_handle())?;
    let directory = path
        .parent()
        .ok_or("Atlas could not locate its settings folder.")?;
    std::fs::create_dir_all(directory).map_err(display)?;
    let mut file = tempfile::NamedTempFile::new_in(directory).map_err(display)?;
    serde_json::to_writer(&mut file, &pane).map_err(display)?;
    file.persist(path).map_err(display)?;
    Ok(())
}

#[cfg(windows)]
fn windows_recipient(app: &AppHandle, base: &Url, instance: &str) -> Option<WebviewWindow> {
    let desktop = app.state::<Desktop>();
    // A product window can still be loading its first authenticated document.
    let attached = || {
        app.webview_windows().into_values().filter(|window| {
            window.label() != WINDOW && desktop.staged_setup.attached_to(window.label(), instance)
        })
    };
    attached()
        .find(|window| window.is_focused().unwrap_or(false))
        .or_else(|| {
            crate::recent_product_views(app, base, instance)
                .into_iter()
                .next()
        })
        .or_else(|| attached().next())
}

#[cfg(windows)]
fn present_windows(
    window: &WebviewWindow,
    instance: &str,
    destination: Destination,
) -> Result<(), String> {
    window
        .state::<Desktop>()
        .staged_setup
        .request_settings(window, instance, destination)?;
    window_host::select(window)
}

#[cfg(windows)]
fn open_windows_menu(
    app: &AppHandle,
    base: &Url,
    instance: String,
    destination: Destination,
) -> Result<(), String> {
    let desktop = app.state::<Desktop>();
    if let Some((opening, latest)) = lock(&desktop.pending_settings_window).as_mut()
        && opening == &instance
    {
        *latest = destination;
        return Ok(());
    }
    if let Some(window) = windows_recipient(app, base, &instance) {
        return present_windows(&window, &instance, destination);
    }
    *lock(&desktop.pending_settings_window) = Some((instance.clone(), destination));
    let app = app.clone();
    tauri::async_runtime::spawn_blocking(move || {
        // Another opening flow can attach a window before this worker starts.
        let window = current_product(&app).and_then(|(base, current)| {
            if current != instance {
                return Err("The Settings destination is no longer available.".into());
            }
            windows_recipient(&app, &base, &instance).map_or_else(
                || crate::open_product_on_worker(&app, "/search", &instance),
                Ok,
            )
        });
        let opening = instance.clone();
        if let Err(error) = crate::run_on_main(&app, move |app| {
            let desktop = app.state::<Desktop>();
            let request =
                lock(&desktop.pending_settings_window).take_if(|(current, _)| current == &opening);
            if let Some((instance, latest)) = request
                && let Err(error) =
                    window.and_then(|window| present_windows(&window, &instance, latest))
            {
                window_host::report("Could not open Settings", &error);
            }
        }) {
            lock(&app.state::<Desktop>().pending_settings_window)
                .take_if(|(current, _)| current == &instance);
            eprintln!("Could not open Settings: {error}");
        }
    });
    Ok(())
}

#[cfg(target_os = "macos")]
fn open_mac(
    app: &AppHandle,
    base: &Url,
    instance: &str,
    destination: Destination,
) -> Result<(), String> {
    use tauri::{WebviewUrl, WebviewWindowBuilder};
    let desktop = app.state::<Desktop>();
    if let Some(window) = app.get_webview_window(WINDOW) {
        if !desktop.staged_setup.attached_to(WINDOW, instance) {
            recover_window(app, &window, base, instance)?;
        }
        if destination != Destination::General {
            desktop
                .staged_setup
                .request_settings(&window, instance, destination)?;
        }
        return window_host::select(&window);
    }
    let target = if destination == Destination::General {
        restored_pane(app)
    } else {
        destination
    };
    let url = base.join(target.path()).map_err(display)?;
    desktop
        .staged_setup
        .attach_window(app, WINDOW, base, instance)?;
    // A recovered workspace service can have a new port.
    let handle = app.clone();
    let window = WebviewWindowBuilder::new(app, WINDOW, WebviewUrl::External(url))
        .title(target.pane().0)
        .visible(false)
        .inner_size(620.0, 510.0)
        .min_inner_size(420.0, 340.0)
        .minimizable(false)
        .maximizable(false)
        .background_color(crate::appearance::canvas(crate::appearance::product_dark(
            app,
        )))
        .on_navigation(move |url| {
            current_product(&handle).is_ok_and(|(base, _)| internal_route(&base, url))
        })
        .on_new_window(|_, _| tauri::webview::NewWindowResponse::Deny)
        .on_page_load(crate::staged_setup::page_load)
        .build()
        .map_err(display)
        .inspect_err(|_| desktop.staged_setup.detach_window(WINDOW))?;
    crate::zoom::apply(&window);
    if let Err(error) =
        window_host::configure(&window, false).and_then(|()| window_host::select(&window))
    {
        let _ = window.destroy();
        return Err(error);
    }
    Ok(())
}

#[cfg(target_os = "macos")]
pub fn recover_window(
    app: &AppHandle,
    window: &WebviewWindow,
    base: &Url,
    instance: &str,
) -> Result<(), String> {
    let pane = window
        .url()
        .ok()
        .and_then(|url| Destination::from_url(&url))
        .unwrap_or_else(|| restored_pane(app));
    app.state::<Desktop>()
        .staged_setup
        .attach_window(app, WINDOW, base, instance)?;
    window
        .navigate(base.join(pane.path()).map_err(display)?)
        .map_err(display)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[cfg(target_os = "macos")]
    #[test]
    fn restores_only_current_top_level_panes() {
        let home = tempfile::tempdir().unwrap();
        let path = home.path().join("settings-pane");
        assert_eq!(read_pane(&path), None);
        for (value, expected) in [
            ("terminal", Some(Destination::Terminal)),
            ("data", Some(Destination::Data)),
            ("channel", Some(Destination::Updates)),
            ("general", Some(Destination::Updates)),
            ("obsolete", None),
        ] {
            std::fs::write(&path, serde_json::to_vec(value).unwrap()).unwrap();
            assert_eq!(read_pane(&path), expected);
        }
    }

    #[test]
    fn settings_routes_accept_only_known_local_destinations() {
        let base = Url::parse("http://127.0.0.1:4210/").unwrap();
        for path in [
            "/settings",
            "/settings/updates",
            "/settings/channel",
            "/settings/data",
            "/settings/terminal",
        ] {
            assert!(internal_route(&base, &base.join(path).unwrap()));
        }
        for path in [
            "/settings/other",
            "/settings?target=data",
            "/settings#other",
            "/search",
            "http://other.test/settings",
            "http://name@127.0.0.1:4210/settings",
        ] {
            assert!(!internal_route(&base, &base.join(path).unwrap()));
        }
    }
}
