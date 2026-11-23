//! Windows document-tab commands and acknowledged protocol-link delivery.

use std::{collections::HashMap, sync::Mutex};

use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Manager, WebviewWindow};
use url::Url;

use crate::{
    Desktop, current_product, display, lock, product_view, recent_product_views, run_on_main,
    same_product_route, staged_setup, tab_links, window_host,
};

#[derive(Default)]
pub struct WindowsTabs(Mutex<State>);

#[derive(Default)]
struct State {
    windows: HashMap<String, AttachedTabs>,
    next_delivery: u64,
    delivery: Option<Delivery>,
    exit: Option<ExitCheck>,
}

struct ExitCheck {
    id: u64,
    waiting: HashMap<String, String>,
    dirty: bool,
    pending: bool,
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
    let desktop = window.state::<Desktop>();
    let token = lock(&desktop.windows_tabs.0)
        .windows
        .get(window.label())
        .map(|entry| entry.token.clone());
    token.is_some_and(|token| validate(window, &token).is_ok())
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
    let (base, instance) = current_product(app)?;
    let destination = base.join(route).map_err(display)?;
    let desktop = app.state::<Desktop>();
    let windows = recent_product_views(app, &base, &instance);
    let (window, selected, delivery) = {
        let mut state = lock(&desktop.windows_tabs.0);
        if state.delivery.is_some() {
            return Ok(());
        }
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
        }
    })
}

#[tauri::command]
pub async fn desktop_tabs_close(window: WebviewWindow, token: String) -> Result<(), String> {
    validate(&window, &token)?;
    let app = window.app_handle().clone();
    run_on_main(&app, move |_| {
        if let Err(error) = window.destroy() {
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

pub fn exit(app: &AppHandle) -> Result<(), String> {
    let Ok((base, instance)) = current_product(app) else {
        app.exit(0);
        return Ok(());
    };
    let windows: Vec<_> = recent_product_views(app, &base, &instance)
        .into_iter()
        .filter(ready)
        .collect();
    if windows.is_empty() {
        app.exit(0);
        return Ok(());
    }
    let desktop = app.state::<Desktop>();
    let id = {
        let mut state = lock(&desktop.windows_tabs.0);
        let waiting = windows
            .iter()
            .filter_map(|window| {
                let token = &state.windows.get(window.label())?.token;
                Some((window.label().to_string(), token.clone()))
            })
            .collect();
        state.next_delivery += 1;
        let id = state.next_delivery;
        state.exit = Some(ExitCheck {
            id,
            waiting,
            dirty: false,
            pending: false,
        });
        id
    };
    for window in windows {
        if let Err(error) = send(&window, Command::CheckExit { id }) {
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
    run_on_main(window.app_handle(), move |app| {
        if exit.pending {
            window_host::report(
                "Atlas is saving",
                "Wait for the save to finish before exiting.",
            );
        } else if !exit.dirty || window_host::confirm_exit() {
            app.exit(0);
        }
    })
}
