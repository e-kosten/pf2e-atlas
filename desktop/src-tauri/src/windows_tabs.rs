//! Windows document-tab commands and acknowledged protocol-link delivery.

use std::{
    collections::HashMap,
    sync::{Mutex, mpsc},
    time::Duration,
};

use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Manager, WebviewWindow};
use url::Url;

use crate::{
    Desktop, current_product, display, lock, product_view, recent_product_views, run_on_main,
    running_product, same_product_route, staged_setup, tab_links, window_host,
};

#[derive(Default)]
pub struct WindowsTabs(Mutex<State>);

#[derive(Default)]
struct State {
    windows: HashMap<String, AttachedTabs>,
    next_delivery: u64,
    delivery: Option<Delivery>,
    exit: Option<ExitCheck>,
    window_handoff: bool,
    opening_windows: usize,
}

pub(crate) struct OpeningWindow<'a>(&'a Mutex<State>);

impl WindowsTabs {
    pub(crate) fn reserve_opening(&self) -> Result<OpeningWindow<'_>, String> {
        let mut state = lock(&self.0);
        if state.window_handoff {
            return Err("Wait for Atlas to finish checking its windows.".into());
        }
        state.opening_windows += 1;
        Ok(OpeningWindow(&self.0))
    }

    fn begin_handoff(&self) -> Result<(), String> {
        let mut state = lock(&self.0);
        if state.opening_windows != 0 {
            return Err("A workspace window is still opening. Try again once it opens.".into());
        }
        if state.window_handoff || state.exit.is_some() {
            return Err("Atlas is already checking its windows.".into());
        }
        state.window_handoff = true;
        Ok(())
    }
}

impl Drop for OpeningWindow<'_> {
    fn drop(&mut self) {
        lock(self.0).opening_windows -= 1;
    }
}

struct ExitCheck {
    id: u64,
    waiting: HashMap<String, String>,
    dirty: bool,
    pending: bool,
    reply: mpsc::Sender<(bool, bool)>,
}

struct AttachedTabs {
    token: String,
    snapshot: Snapshot,
}

struct Delivery {
    label: String,
    token: String,
    id: u64,
    sequence: u64,
}

#[derive(Deserialize, PartialEq, Eq)]
pub struct Snapshot {
    active: String,
    activation: u64,
    sequence: u64,
    tabs: Vec<Tab>,
}

#[derive(Deserialize, PartialEq, Eq)]
struct Tab {
    id: String,
    href: String,
}

#[derive(Serialize)]
#[serde(tag = "kind", rename_all = "kebab-case")]
pub enum Command<'a> {
    Open {
        href: &'a str,
        foreground: bool,
        #[serde(skip_serializing_if = "Option::is_none")]
        delivery: Option<u64>,
    },
    Select {
        id: &'a str,
        href: &'a str,
        delivery: u64,
    },
    CloseTab,
    CloseWindow,
    CheckExit {
        id: u64,
    },
}

fn validate(window: &WebviewWindow, token: &str) -> Result<Url, String> {
    let (base, instance) = validate_document(window, token)?;
    if !product_view(window, &base, &instance) {
        return Err("Atlas tabs are still opening.".into());
    }
    Ok(base)
}

fn validate_document(window: &WebviewWindow, token: &str) -> Result<(Url, String), String> {
    let (base, instance) = current_product(window.app_handle())?;
    if !window.url().is_ok_and(|url| url.origin() == base.origin())
        || !staged_setup::owns_document(window, &instance, token)
    {
        return Err("This tab command belongs to an earlier Atlas document.".into());
    }
    Ok((base, instance))
}

pub fn forget(app: &AppHandle, label: &str) -> bool {
    let desktop = app.state::<Desktop>();
    let mut state = lock(&desktop.windows_tabs.0);
    state.windows.remove(label);
    if state
        .exit
        .as_ref()
        .is_some_and(|exit| exit.waiting.contains_key(label))
    {
        state.exit = None;
    }
    let interrupted_delivery = state
        .delivery
        .as_ref()
        .is_some_and(|delivery| delivery.label == label);
    if interrupted_delivery {
        state.delivery = None;
    }
    interrupted_delivery
}

pub fn ready(window: &WebviewWindow) -> bool {
    document_token(window).is_some()
}

pub fn send(window: &WebviewWindow, command: Command<'_>) -> Result<(), String> {
    let desktop = window.state::<Desktop>();
    let token = lock(&desktop.windows_tabs.0)
        .windows
        .get(window.label())
        .map(|entry| entry.token.clone())
        .ok_or("Atlas tabs are still opening.")?;
    validate(window, &token)?;
    let token = serde_json::to_string(&token).map_err(display)?;
    let detail = serde_json::to_string(&command).map_err(display)?;
    window.eval(format!(
        "if (window.__ATLAS_DESKTOP_SETUP__?.token === {token}) {{ window.dispatchEvent(new CustomEvent('atlas-windows-tab-command', {{detail: {detail}}})); }}"
    )).map_err(display)
}

pub fn active_route(window: &WebviewWindow, base: &Url, tab: &str, activation: u64) -> Option<Url> {
    let desktop = window.state::<Desktop>();
    let state = lock(&desktop.windows_tabs.0);
    let snapshot = &state.windows.get(window.label())?.snapshot;
    if snapshot.active != tab || snapshot.activation != activation {
        return None;
    }
    snapshot
        .tabs
        .iter()
        .find(|entry| entry.id == tab)
        .and_then(|entry| base.join(&entry.href).ok())
}

#[tauri::command]
pub async fn desktop_tabs_state(
    window: WebviewWindow,
    token: String,
    snapshot: Snapshot,
) -> Result<(), String> {
    let (base, _) = validate_document(&window, &token)?;
    let mut ids = std::collections::HashSet::new();
    if !snapshot.tabs.iter().any(|tab| tab.id == snapshot.active)
        || !snapshot.tabs.iter().all(|tab| {
            ids.insert(&tab.id)
                && tab.id.len() == 36
                && base.join(&tab.href).is_ok_and(|url| {
                    tab_links::product_link(&base, &url)
                        || crate::settings::internal_route(&base, &url)
                })
        })
    {
        return Err("Invalid Atlas tab state.".into());
    }
    let desktop = window.state::<Desktop>();
    {
        let mut state = lock(&desktop.windows_tabs.0);
        if state.windows.get(window.label()).is_some_and(|entry| {
            entry.token == token && entry.snapshot.sequence >= snapshot.sequence
        }) {
            return Ok(());
        }
        state
            .windows
            .insert(window.label().into(), AttachedTabs { token, snapshot });
    }
    run_on_main(window.app_handle(), |app| {
        crate::copy_link::refresh(app);
        if let Err(error) = crate::drain_links(app) {
            crate::report_link_error(app, &error);
        }
    })
}

pub fn deliver(app: &AppHandle, route: &str) -> Result<(), String> {
    if window_handoff_active(app) {
        return Ok(());
    }
    // A cancelled Exit can release its guard while disconnected. Keep the
    // unacknowledged link queued until Reconnect starts a new service.
    let Some((base, instance)) = running_product(app)? else {
        return Ok(());
    };
    let destination = base.join(route).map_err(display)?;
    let desktop = app.state::<Desktop>();
    let windows = recent_product_views(app, &base, &instance);
    let (window, selected, delivery) = {
        let mut state = lock(&desktop.windows_tabs.0);
        if state.delivery.as_ref().is_some_and(|delivery| {
            windows.iter().any(|window| {
                window.label() == delivery.label
                    && staged_setup::owns_document(window, &instance, &delivery.token)
            })
        }) {
            return Ok(());
        }
        // A detached or replaced document cannot acknowledge this link. Leave
        // it at the head of the queue for the current service.
        state.delivery = None;
        let candidates = windows.iter().filter_map(|window| {
            let attached = state.windows.get(window.label())?;
            if attached.snapshot.tabs.is_empty() {
                return None;
            }
            staged_setup::owns_document(window, &instance, &attached.token)
                .then_some((window, attached))
        });
        let matching = candidates.clone().find_map(|(window, attached)| {
            let snapshot = &attached.snapshot;
            let matches = |tab: &&Tab| {
                base.join(&tab.href)
                    .is_ok_and(|url| same_product_route(&destination, &url))
            };
            snapshot
                .tabs
                .iter()
                .filter(matches)
                .min_by_key(|tab| tab.id != snapshot.active)
                .map(|tab| {
                    (
                        window,
                        attached.token.clone(),
                        Some((tab.id.clone(), tab.href.clone())),
                    )
                })
        });
        let Some((window, token, selected)) = matching.or_else(|| {
            candidates
                .into_iter()
                .next()
                .map(|(window, attached)| (window, attached.token.clone(), None))
        }) else {
            drop(state);
            // The native window can exist before its URL and first React
            // snapshot are ready. Its authenticated snapshot will retry this
            // delivery and select the tab that opened at the requested route.
            let attaching = app.webview_windows().keys().any(|label| {
                label != crate::settings::WINDOW
                    && desktop.staged_setup.attached_to(label, &instance)
            });
            if !attaching && crate::setup_window_label(app) != crate::opening::PRIMARY {
                crate::open_missing_product(app, route)?;
            }
            return Ok(());
        };
        state.next_delivery += 1;
        let id = state.next_delivery;
        state.delivery = Some(Delivery {
            label: window.label().into(),
            token,
            id,
            sequence: state.windows[window.label()].snapshot.sequence,
        });
        (window, selected, id)
    };
    let result = window_host::select(window).and_then(|()| {
        send(
            window,
            match &selected {
                Some((id, href)) => Command::Select { id, href, delivery },
                None => Command::Open {
                    href: route,
                    foreground: true,
                    delivery: Some(delivery),
                },
            },
        )
    });
    if result.is_err() {
        lock(&desktop.windows_tabs.0).delivery = None;
    }
    result
}

#[tauri::command]
pub async fn desktop_tabs_ack(
    window: WebviewWindow,
    token: String,
    delivery: u64,
    accepted: Option<bool>,
) -> Result<(), String> {
    validate(&window, &token)?;
    let desktop = window.state::<Desktop>();
    {
        let mut state = lock(&desktop.windows_tabs.0);
        if !state.delivery.as_ref().is_some_and(|active| {
            active.id == delivery && active.label == window.label() && active.token == token
        }) {
            return Ok(());
        }
        if accepted.unwrap_or(true) {
            lock(&desktop.links).pop_front();
        } else if let Some(sequence) = state.delivery.as_ref().map(|entry| entry.sequence)
            && let Some(entry) = state.windows.get_mut(window.label())
            && entry.snapshot.sequence <= sequence
        {
            // Await the next React snapshot rather than dispatching the stale ID again.
            entry.snapshot.tabs.clear();
        }
        state.delivery = None;
    }
    run_on_main(window.app_handle(), |app| {
        if let Err(error) = crate::drain_links(app) {
            crate::report_link_error(app, &error);
        } else {
            crate::hide_bootstrap_after_delivery(app);
        }
    })
}

#[tauri::command]
pub async fn desktop_tabs_close(window: WebviewWindow, token: String) -> Result<(), String> {
    validate(&window, &token)?;
    let app = window.app_handle().clone();
    run_on_main(&app, move |app| {
        if !window_handoff_active(app)
            && validate(&window, &token).is_ok()
            && let Err(error) = window.destroy()
        {
            window_host::report("Could not close Atlas window", &display(error));
        }
    })
}

#[tauri::command]
pub async fn desktop_tabs_new_window(window: WebviewWindow, token: String) -> Result<(), String> {
    validate(&window, &token)?;
    let app = window.app_handle().clone();
    run_on_main(&app, move |app| {
        if validate(&window, &token).is_ok()
            && let Err(error) = crate::new_search(app, false)
        {
            window_host::report("Could not open an Atlas window", &error);
        }
    })
}

pub fn menu(app: &AppHandle, command: Command<'_>) -> Result<(), String> {
    let Ok((base, instance)) = current_product(app) else {
        return Ok(());
    };
    let views = recent_product_views(app, &base, &instance);
    if let Some(window) = views
        .iter()
        .find(|window| window.is_focused().unwrap_or(false) && ready(window))
    {
        send(window, command)?;
    }
    Ok(())
}

fn workspace_window(window: &WebviewWindow) -> bool {
    window.label() != "bootstrap"
        && (window.label() != crate::opening::PRIMARY
            || lock(&window.state::<Desktop>().primary).is_product())
}

pub fn handle_close(window: &WebviewWindow, api: &tauri::CloseRequestApi) {
    if !workspace_window(window) {
        return;
    }
    api.prevent_close();
    if window_handoff_active(window.app_handle()) {
        return;
    }
    let result = if ready(window) {
        send(window, Command::CloseWindow)
    } else if window_host::confirm_disconnected_close()
        && workspace_window(window)
        && !ready(window)
        && !window_handoff_active(window.app_handle())
    {
        window.destroy().map_err(display)
    } else {
        Ok(())
    };
    if let Err(error) = result {
        window_host::report("Could not close Atlas window", &error);
    }
}

pub fn exit(app: &AppHandle) -> Result<(), String> {
    if window_handoff_active(app) {
        return Ok(());
    }
    if !app.webview_windows().values().any(workspace_window) {
        app.exit(0);
        return Ok(());
    }
    let app = app.clone();
    tauri::async_runtime::spawn_blocking(move || {
        let check = || -> Result<(), String> {
            let (guard, dirty, pending) = WindowHandoff::inspect(&app, false)?;
            if pending {
                return Err("Wait for the save to finish before exiting.".into());
            }
            let disconnected = guard.documents.values().any(Option::is_none);
            if (dirty || disconnected) && !window_host::confirm_exit(disconnected) {
                return Ok(());
            }
            guard.validate()?;
            app.exit(0);
            Ok(())
        };
        if let Err(error) = check() {
            window_host::report("Could not exit Atlas", &error);
        }
    });
    Ok(())
}

fn check_windows(
    app: &AppHandle,
    windows: &[WebviewWindow],
    reply: mpsc::Sender<(bool, bool)>,
) -> Result<(), String> {
    let desktop = app.state::<Desktop>();
    let id = {
        let mut state = lock(&desktop.windows_tabs.0);
        let tokens = windows
            .iter()
            .map(|window| {
                let entry = state
                    .windows
                    .get(window.label())
                    .ok_or("A workspace window changed. Retry after it finishes opening.")?;
                Ok((window.label().to_string(), entry.token.clone()))
            })
            .collect::<Result<HashMap<_, _>, &'static str>>()?;
        state.next_delivery += 1;
        let id = state.next_delivery;
        state.exit = Some(ExitCheck {
            id,
            waiting: tokens,
            dirty: false,
            pending: false,
            reply,
        });
        id
    };
    for window in windows {
        if let Err(error) = send(window, Command::CheckExit { id }) {
            lock(&desktop.windows_tabs.0).exit = None;
            return Err(error);
        }
    }
    Ok(())
}

#[tauri::command]
pub async fn desktop_tabs_exit_status(
    window: WebviewWindow,
    token: String,
    id: u64,
    dirty: bool,
    pending: bool,
) -> Result<(), String> {
    validate(&window, &token)?;
    let desktop = window.state::<Desktop>();
    let ready = {
        let mut state = lock(&desktop.windows_tabs.0);
        let Some(exit) = &mut state.exit else {
            return Ok(());
        };
        if exit.id != id || exit.waiting.get(window.label()) != Some(&token) {
            return Ok(());
        }
        exit.waiting.remove(window.label());
        exit.dirty |= dirty;
        exit.pending |= pending;
        if exit.waiting.is_empty() {
            state.exit.take()
        } else {
            None
        }
    };
    let Some(exit) = ready else {
        return Ok(());
    };
    let _ = exit.reply.send((exit.dirty, exit.pending));
    Ok(())
}

pub fn window_handoff_active(app: &AppHandle) -> bool {
    lock(&app.state::<Desktop>().windows_tabs.0).window_handoff
}

/// Freeze native input while every live workspace reports its drafts. Exit can
/// confirm discarded work in a disconnected document; update requires all reports.
pub struct WindowHandoff {
    app: AppHandle,
    disabled: Vec<WebviewWindow>,
    documents: HashMap<String, Option<String>>,
}

fn document_token(window: &WebviewWindow) -> Option<String> {
    let token = lock(&window.state::<Desktop>().windows_tabs.0)
        .windows
        .get(window.label())?
        .token
        .clone();
    validate(window, &token).ok()?;
    Some(token)
}

impl WindowHandoff {
    pub fn for_update(app: &AppHandle) -> Result<Self, String> {
        let (guard, dirty, pending) = Self::inspect(app, true)?;
        if pending {
            return Err(
                "Wait for Atlas to finish saving in all windows, then retry the update.".into(),
            );
        }
        if dirty && !window_host::confirm_update() {
            return Err("Update cancelled. Your unsaved changes are still open.".into());
        }
        guard.validate()?;
        Ok(guard)
    }

    fn inspect(app: &AppHandle, updating: bool) -> Result<(Self, bool, bool), String> {
        let (prepared, prepared_rx) = mpsc::sync_channel(1);
        let (reply, replies) = mpsc::channel();
        run_on_main(app, move |app| {
            let _ = prepared.send(Self::freeze(app, reply, updating));
        })?;
        let guard = prepared_rx
            .recv_timeout(Duration::from_secs(5))
            .map_err(|_| "Atlas could not pause its windows. Try again.")??;
        let (dirty, pending) = replies.recv_timeout(Duration::from_secs(5)).map_err(|_| {
            "Atlas could not check every window. Finish or close the unresponsive window and retry."
        })?;
        guard.validate()?;
        Ok((guard, dirty, pending))
    }

    pub fn validate(&self) -> Result<(), String> {
        let current: HashMap<_, _> = self
            .app
            .webview_windows()
            .into_values()
            .filter(workspace_window)
            .map(|window| (window.label().to_owned(), document_token(&window)))
            .collect();
        if current != self.documents {
            return Err("A workspace window changed while Atlas was checking for edits. Review it and retry.".into());
        }
        Ok(())
    }

    fn freeze(
        app: &AppHandle,
        reply: mpsc::Sender<(bool, bool)>,
        updating: bool,
    ) -> Result<Self, String> {
        let desktop = app.state::<Desktop>();
        desktop.windows_tabs.begin_handoff()?;
        let mut guard = Self {
            app: app.clone(),
            disabled: Vec::new(),
            documents: HashMap::new(),
        };
        let mut live = Vec::new();
        for window in app.webview_windows().into_values().filter(workspace_window) {
            let token = document_token(&window);
            if updating && token.is_none() {
                return Err("A workspace window is disconnected or still opening. Review or close it before updating.".into());
            }
            if window.is_enabled().map_err(display)? {
                window.set_enabled(false).map_err(display)?;
                guard.disabled.push(window.clone());
            }
            if token.is_some() {
                live.push(window.clone());
            }
            guard.documents.insert(window.label().to_owned(), token);
        }
        if live.is_empty() {
            if updating {
                return Err("Open a connected workspace window before updating.".into());
            }
            let _ = reply.send((false, false));
        } else {
            check_windows(app, &live, reply)?;
        }
        Ok(guard)
    }
}

impl Drop for WindowHandoff {
    fn drop(&mut self) {
        let disabled = std::mem::take(&mut self.disabled);
        let _ = run_on_main(&self.app, move |app| {
            let desktop = app.state::<Desktop>();
            {
                let mut state = lock(&desktop.windows_tabs.0);
                state.window_handoff = false;
                state.exit = None;
            }
            for window in disabled {
                let _ = window.set_enabled(true);
            }
            if let Err(error) = crate::drain_links(app) {
                crate::report_link_error(app, &error);
            }
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn update_waits_for_every_window_and_blocks_new_opens() {
        let tabs = WindowsTabs::default();
        let first = tabs.reserve_opening().unwrap();
        let second = tabs.reserve_opening().unwrap();
        assert!(tabs.begin_handoff().is_err());
        drop(first);
        assert!(tabs.begin_handoff().is_err());
        drop(second);
        tabs.begin_handoff().unwrap();
        assert!(tabs.reserve_opening().is_err());
    }
}
