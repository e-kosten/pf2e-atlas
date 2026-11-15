//! First-run navigation and recovery within the primary native window.

use std::{
    sync::{atomic::Ordering, mpsc},
    thread,
    time::{Duration, Instant},
};

use tauri::{AppHandle, Manager, WebviewUrl, WebviewWindow, WebviewWindowBuilder};
use url::{Origin, Url};

use crate::{
    Desktop, bundled_origin, display, lock, open_external, run_on_main, same_product_route,
    show_pending_or_default, verified_url,
};

pub const PRIMARY: &str = "atlas-primary";

#[derive(Default)]
pub enum PrimaryWindow {
    #[default]
    Setup,
    Opening(Origin),
    Retry {
        route: String,
        error: String,
        bundled: Url,
    },
    Product(Origin),
}

impl PrimaryWindow {
    pub fn is_product(&self) -> bool {
        matches!(self, Self::Product(_))
    }

    pub fn error(&self) -> Option<&str> {
        match self {
            Self::Retry { error, .. } => Some(error),
            _ => None,
        }
    }

    fn allows(&self, url: &Url) -> bool {
        match self {
            Self::Setup | Self::Retry { .. } => bundled_origin(url),
            Self::Opening(origin) | Self::Product(origin) => url.origin() == *origin,
        }
    }
}

pub fn build(app: &AppHandle) -> tauri::Result<WebviewWindow> {
    let handle = app.clone();
    let app_for_popup = app.clone();
    let builder = WebviewWindowBuilder::new(app, PRIMARY, WebviewUrl::App("index.html".into()))
        .title("PF2e Atlas")
        .initialization_script(crate::tab_links::SCRIPT)
        .inner_size(1210.0, 830.0)
        .min_inner_size(480.0, 440.0)
        .on_page_load(crate::copy_link::page_load)
        .on_navigation(move |url| {
            let desktop = handle.state::<Desktop>();
            let (allowed, product) = {
                let primary = lock(&desktop.primary);
                (primary.allows(url), primary.is_product())
            };
            if allowed {
                return true;
            }
            if product {
                open_external(url);
            }
            false
        })
        .on_new_window(move |destination, _| {
            let desktop = app_for_popup.state::<Desktop>();
            if !lock(&desktop.primary).allows(&destination) {
                open_external(&destination);
            }
            tauri::webview::NewWindowResponse::Deny
        });
    #[cfg(debug_assertions)]
    let builder = crate::smoke::configure(builder);
    builder.build()
}

pub fn open(app: &AppHandle, window: &WebviewWindow) -> Result<(), String> {
    let desktop = app.state::<Desktop>();
    let (base, instance) = {
        let launcher = lock(&desktop.launcher);
        let ready = &launcher.as_ref().ok_or("The Atlas service stopped.")?.ready;
        (verified_url(&ready.endpoint)?, ready.instance.clone())
    };
    let retry_route = match &*lock(&desktop.primary) {
        PrimaryWindow::Retry { route, .. } => Some(route.clone()),
        _ => None,
    };
    let is_retry = retry_route.is_some();
    let route = retry_route
        .or_else(|| lock(&desktop.links).front().cloned())
        .unwrap_or_else(|| "/search".into());
    let target = base.join(&route).map_err(display)?;
    desktop
        .staged_setup
        .attach_window(app, PRIMARY, &base, &instance)?;
    let origin = target.origin();
    let bundled = window.url().map_err(display)?;
    *lock(&desktop.primary) = PrimaryWindow::Opening(origin.clone());

    let result = window
        .navigate(target.clone())
        .map_err(display)
        .and_then(|()| wait_until_rendered(&desktop, window, &target, &instance, is_retry));
    if let Err(error) = result {
        // Navigation unloads the invoking document. Its replacement reads this
        // state through inspect, including when the original promise is gone.
        *lock(&desktop.primary) = PrimaryWindow::Retry {
            route,
            error: error.clone(),
            bundled: bundled.clone(),
        };
        window.navigate(bundled).map_err(display)?;
        return Err(error);
    }

    {
        let mut links = lock(&desktop.links);
        if links.front() == Some(&route) {
            links.pop_front();
        }
    }
    run_on_main(app, move |app| {
        *lock(&app.state::<Desktop>().primary) = PrimaryWindow::Product(origin);
        crate::copy_link::refresh(app);
        if let Some(window) = app.get_webview_window(PRIMARY) {
            if let Err(error) = crate::native_tabs::configure(&window, true) {
                eprintln!("Could not configure Atlas tabs: {error}");
            }
            crate::remember_product_focus(app, PRIMARY);
        }
        let has_pending_links = !lock(&app.state::<Desktop>().links).is_empty();
        if has_pending_links {
            show_pending_or_default(app);
        }
    })
}

pub fn restore_setup_if_needed(app: &AppHandle) -> tauri::Result<()> {
    let bundled = match &*lock(&app.state::<Desktop>().primary) {
        PrimaryWindow::Retry { bundled, .. } => bundled.clone(),
        _ => return Ok(()),
    };
    if let Some(window) = app.get_webview_window(PRIMARY)
        && !window.url().is_ok_and(|url| bundled_origin(&url))
    {
        window.navigate(bundled)?;
    }
    Ok(())
}

pub fn retry_before_navigation(app: &AppHandle, window: &WebviewWindow, error: &str) {
    let Ok(bundled) = window.url() else { return };
    if !bundled_origin(&bundled) {
        return;
    }
    let desktop = app.state::<Desktop>();
    let route = lock(&desktop.links)
        .front()
        .cloned()
        .unwrap_or_else(|| "/search".into());
    {
        let mut primary = lock(&desktop.primary);
        if !matches!(*primary, PrimaryWindow::Setup) {
            return;
        }
        *primary = PrimaryWindow::Retry {
            route,
            error: error.to_owned(),
            bundled: bundled.clone(),
        };
    }
    // Restart the bundled page so it can offer Retry while the original
    // prepare invocation still owns and drains the semantic child.
    let _ = window.navigate(bundled);
}

fn wait_until_rendered(
    desktop: &Desktop,
    window: &WebviewWindow,
    target: &Url,
    instance: &str,
    is_retry: bool,
) -> Result<(), String> {
    #[cfg(debug_assertions)]
    let timeout = if !is_retry && crate::smoke::forces_retry() {
        Duration::from_millis(1500)
    } else {
        Duration::from_secs(8)
    };
    #[cfg(not(debug_assertions))]
    let timeout = {
        let _ = is_retry;
        Duration::from_secs(8)
    };
    let deadline = Instant::now() + timeout;
    let mut reached_page = false;
    while Instant::now() < deadline {
        {
            let launcher = lock(&desktop.launcher);
            if launcher
                .as_ref()
                .is_none_or(|launcher| launcher.ready.instance != instance)
                || desktop.quitting.load(Ordering::Acquire)
            {
                return Err("The local Atlas service stopped while the window was opening.".into());
            }
        }
        if window
            .url()
            .is_ok_and(|url| same_product_route(target, &url))
        {
            reached_page = true;
            if evaluates_true(
                window,
                "Boolean(document.querySelector('#atlas-main')?.getClientRects().length)",
            ) && window
                .url()
                .is_ok_and(|url| same_product_route(target, &url))
            {
                let launcher = lock(&desktop.launcher);
                if launcher
                    .as_ref()
                    .is_some_and(|launcher| launcher.ready.instance == instance)
                    && !desktop.quitting.load(Ordering::Acquire)
                {
                    return Ok(());
                }
            }
        }
        thread::sleep(Duration::from_millis(50));
    }
    Err(if reached_page {
        "The Atlas service opened, but the app view did not finish loading."
    } else {
        "The window did not reach the local Atlas service."
    }
    .into())
}

pub fn evaluates_true(window: &WebviewWindow, script: &str) -> bool {
    let (sender, receiver) = mpsc::channel();
    window
        .eval_with_callback(script, move |value| {
            let _ = sender.send(value == "true");
        })
        .is_ok()
        && receiver
            .recv_timeout(Duration::from_millis(200))
            .unwrap_or(false)
}
