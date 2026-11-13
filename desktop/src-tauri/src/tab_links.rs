//! Product link gestures from the desktop WebView.

use tauri::{AppHandle, Manager, WebviewWindow, menu::MenuBuilder};
use url::Url;

use crate::{
    Desktop, current_product, display, lock, native_tabs, open_product, product_view, run_on_main,
};

pub const SCRIPT: &str = include_str!("tab_links.js");
pub const MENU_ID: &str = "open-link-in-new-tab";

pub struct Link {
    source: String,
    destination: Url,
}

impl Link {
    fn validate(&self, app: &AppHandle) -> Result<WebviewWindow, String> {
        let (base, instance) = current_product(app)?;
        app.get_webview_window(&self.source)
            .filter(|window| product_view(window, &base, &instance))
            .filter(|_| product_link(&base, &self.destination))
            .ok_or_else(|| "This link does not belong to the active Atlas workspace.".into())
    }

    fn open(self, app: &AppHandle) -> Result<(), String> {
        let source = self.validate(app)?;
        open_product(app, self.destination.as_str(), Some((&source, false))).map(|_| ())
    }
}

#[tauri::command]
pub async fn desktop_tab_link(
    window: WebviewWindow,
    url: Url,
    context: bool,
) -> Result<(), String> {
    let request = Link {
        source: window.label().into(),
        destination: url,
    };
    request.validate(window.app_handle())?;
    run_on_main(window.app_handle(), move |app| {
        let result = if context {
            show_menu(app, request)
        } else {
            request.open(app)
        };
        if let Err(error) = result {
            native_tabs::report("Could not open Atlas link", &error);
        }
    })
}

fn show_menu(app: &AppHandle, request: Link) -> Result<(), String> {
    let window = request.validate(app)?;
    let menu = MenuBuilder::new(app)
        .text(MENU_ID, "Open Link in New Tab")
        .build()
        .map_err(display)?;
    *lock(&app.state::<Desktop>().context_link) = Some(request);
    window.popup_menu(&menu).map_err(display)
}

pub fn open_context_link(app: &AppHandle) -> Result<(), String> {
    let request = lock(&app.state::<Desktop>().context_link).take();
    request.map_or(Ok(()), |request| request.open(app))
}

fn product_link(base: &Url, destination: &Url) -> bool {
    if destination.origin() != base.origin()
        || !destination.username().is_empty()
        || destination.password().is_some()
    {
        return false;
    }
    let path = destination.path().trim_end_matches('/');
    let parts = path
        .strip_prefix('/')
        .unwrap_or(path)
        .split('/')
        .collect::<Vec<_>>();
    matches!(
        parts.as_slice(),
        [""] | ["search" | "lists" | "encounters"]
            | [
                "records" | "definitions" | "traits" | "lists" | "encounters",
                _
            ]
            | ["search", "records", _]
            | ["lists", _, "records", _]
            | ["families", "creatures" | "equipment", _]
            | ["encounters" | "lists", _, "edit"]
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn only_current_product_routes_can_open_in_tabs() {
        let base = Url::parse("http://127.0.0.1:9988").unwrap();
        for path in [
            "/search",
            "/records/items%3Aid?rank=2#details",
            "/lists/loot/records/items%3Aid",
            "/lists/loot/edit",
        ] {
            assert!(product_link(&base, &base.join(path).unwrap()), "{path}");
        }
        for path in [
            "/api/records/items%3Aid",
            "/assets/index.js",
            "/unknown",
            "http://127.0.0.1:9989/search",
        ] {
            assert!(!product_link(&base, &base.join(path).unwrap()), "{path}");
        }
    }
}
