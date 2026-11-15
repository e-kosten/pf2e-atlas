use std::sync::Mutex;

use atlas_desktop_protocol::EntityLink;
use tauri::{AppHandle, Manager, WebviewWindow};
use tauri_plugin_clipboard_manager::ClipboardExt;

use crate::{Desktop, current_product, display, lock, product_view, staged_setup};

#[derive(Default)]
pub struct CopyMenu(Mutex<MenuState>);

#[derive(Default)]
struct MenuState {
    ready: std::collections::HashMap<String, Document>,
    availability: std::collections::HashMap<String, (String, u64)>,
    pending: Option<(String, Document, u64)>,
    sequence: u64,
}

#[derive(Clone, PartialEq, Eq)]
struct Document {
    token: String,
    route: String,
}

fn document(
    window: &WebviewWindow,
    token: &str,
    route: &str,
    require_current_route: bool,
) -> Result<Document, String> {
    let app = window.app_handle();
    let (base, instance) = current_product(app)?;
    let url = window.url().map_err(display)?;
    if url.origin() != base.origin()
        || !staged_setup::owns_document(window, &instance, token)
        || (require_current_route && url.as_str() != route)
    {
        return Err("This link belongs to an earlier Atlas view.".into());
    }
    Ok(Document {
        token: token.into(),
        route: route.into(),
    })
}

fn focused(app: &AppHandle) -> Option<WebviewWindow> {
    app.webview_windows()
        .into_values()
        .find(|window| window.is_focused().unwrap_or(false))
}

pub fn refresh(app: &AppHandle) {
    let desktop = app.state::<Desktop>();
    let enabled = focused(app).is_some_and(|window| {
        let ready = lock(&desktop.copy_menu.0)
            .ready
            .get(window.label())
            .cloned();
        ready.is_some_and(|ready| {
            eligible(&window) && document(&window, &ready.token, &ready.route, true).is_ok()
        })
    });
    if let Some(item) = menu_item(app) {
        let _ = item.set_enabled(enabled);
    }
}

pub fn menu_item(app: &AppHandle) -> Option<tauri::menu::MenuItem<tauri::Wry>> {
    app.menu()?
        .items()
        .ok()?
        .into_iter()
        .find_map(|item| item.as_submenu()?.get("copy-link")?.as_menuitem().cloned())
}

pub fn invalidate(app: &AppHandle, label: &str, remove: bool) {
    let desktop = app.state::<Desktop>();
    let mut menu = lock(&desktop.copy_menu.0);
    menu.pending = None;
    if remove {
        menu.ready.remove(label);
        menu.availability.remove(label);
    }
    drop(menu);
    refresh(app);
}

pub fn request(app: &AppHandle) -> Result<(), String> {
    let Some(window) = focused(app) else {
        return Ok(());
    };
    let desktop = app.state::<Desktop>();
    let ready = lock(&desktop.copy_menu.0)
        .ready
        .get(window.label())
        .cloned();
    let Some(ready) = ready else {
        return Ok(());
    };
    if !eligible(&window) || document(&window, &ready.token, &ready.route, true).is_err() {
        return Ok(());
    }
    let mut menu = lock(&desktop.copy_menu.0);
    menu.sequence += 1;
    let id = menu.sequence;
    menu.pending = Some((window.label().into(), ready, id));
    drop(menu);
    window
        .eval(format!(
            "window.dispatchEvent(new CustomEvent('atlas-copy-link', {{detail: {id}}}))"
        ))
        .map_err(display)
}

#[tauri::command]
pub fn desktop_copy_availability(
    window: WebviewWindow,
    token: String,
    route: String,
    available: bool,
    sequence: u64,
) -> Result<(), String> {
    let current = document(&window, &token, &route, available)?;
    let app = window.app_handle();
    let desktop = app.state::<Desktop>();
    let mut menu = lock(&desktop.copy_menu.0);
    let last = menu.availability.entry(window.label().into()).or_default();
    if last.0 == token && sequence <= last.1 {
        return Ok(());
    }
    *last = (token, sequence);
    if available {
        menu.ready.insert(window.label().into(), current);
    } else if menu.ready.get(window.label()) == Some(&current) {
        menu.ready.remove(window.label());
    }
    if !available {
        if menu
            .pending
            .as_ref()
            .is_some_and(|(label, _, _)| label == window.label())
        {
            menu.pending = None;
        }
    }
    drop(menu);
    refresh(app);
    Ok(())
}

#[tauri::command]
pub fn desktop_copy_link(
    window: WebviewWindow,
    token: String,
    route: String,
    kind: String,
    key: String,
    title: String,
    request: Option<u64>,
) -> Result<(), String> {
    let current = document(&window, &token, &route, true)?;
    let app = window.app_handle();
    if !eligible(&window) {
        return Err("Atlas is still opening.".into());
    }
    if let Some(id) = request {
        let desktop = app.state::<Desktop>();
        let mut menu = lock(&desktop.copy_menu.0);
        if !window.is_focused().unwrap_or(false)
            || !menu
                .pending
                .as_ref()
                .is_some_and(|(label, expected, request)| {
                    label == window.label() && expected == &current && *request == id
                })
        {
            return Err("This copy request belongs to an earlier Atlas view.".into());
        }
        menu.pending = None;
    }
    let link = EntityLink::from_key(&kind, &key)?.uri();
    let html = format!(
        "<a href=\"{}\">{}</a>",
        crate::escape(&link),
        crate::escape(&title)
    );
    app.clipboard()
        .write_html(html, Some(link))
        .map_err(display)
}

fn eligible(window: &WebviewWindow) -> bool {
    current_product(window.app_handle())
        .is_ok_and(|(base, instance)| product_view(window, &base, &instance))
}

pub fn page_load(window: WebviewWindow, payload: tauri::webview::PageLoadPayload<'_>) {
    if payload.event() == tauri::webview::PageLoadEvent::Started {
        invalidate(window.app_handle(), window.label(), true);
    }
    staged_setup::page_load(window, payload);
}
