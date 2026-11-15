//! First-run navigation and recovery within the primary native window.

use std::{
    sync::{atomic::Ordering, mpsc},
    thread,
    time::{Duration, Instant, SystemTime, UNIX_EPOCH},
};

use tauri::{AppHandle, Manager, WebviewUrl, WebviewWindow, WebviewWindowBuilder};
use url::{Origin, Url};

use crate::{
    Desktop, bundled_origin, display, lock, open_external, run_on_main, same_product_route,
    show_pending_or_default, verified_url,
};

use crate::bootstrap_jobs::{Attempt, BootstrapJob, Update as BootstrapUpdate};

pub const PRIMARY: &str = "atlas-primary";

#[derive(Default)]
pub enum PrimaryWindow {
    #[default]
    Setup,
    Armed {
        target: Url,
        route: String,
        bundled: Url,
        attempt: Attempt,
        admitted: mpsc::Sender<()>,
    },
    Opening {
        origin: Origin,
        attempt: Attempt,
    },
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
            Self::Setup | Self::Retry { .. } | Self::Armed { .. } => bundled_origin(url),
            Self::Opening { origin, .. } | Self::Product(origin) => url.origin() == *origin,
        }
    }
}

pub fn build(app: &AppHandle) -> tauri::Result<WebviewWindow> {
    let handle = app.clone();
    let app_for_popup = app.clone();
    let dark = crate::window_host::system_dark();
    let builder = with_cover(
        WebviewWindowBuilder::new(app, PRIMARY, WebviewUrl::App("index.html".into()))
            .title("PF2e Atlas")
            .visible(false),
        dark,
        true,
    )
    .inner_size(1210.0, 830.0)
    .min_inner_size(480.0, 440.0)
    .on_page_load(crate::bootstrap_page_load())
    .on_navigation(move |url| {
        let desktop = handle.state::<Desktop>();
        let (allowed, product, admitted) = {
            let jobs = lock(&desktop.bootstrap_jobs);
            let mut primary = lock(&desktop.primary);
            let admitted = match &*primary {
                PrimaryWindow::Armed {
                    target,
                    attempt,
                    admitted,
                    ..
                } if target == url && jobs.owns_attempt(PRIMARY, *attempt) => {
                    Some((*attempt, admitted.clone()))
                }
                _ => None,
            };
            if let Some((attempt, _)) = &admitted {
                *primary = PrimaryWindow::Opening {
                    origin: url.origin(),
                    attempt: *attempt,
                };
            }
            (primary.allows(url), primary.is_product(), admitted)
        };
        if let Some((_, admitted)) = admitted {
            let _ = admitted.send(());
        }
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
    #[cfg(all(debug_assertions, target_os = "macos"))]
    let builder = crate::smoke::configure(builder);
    builder.build()
}

pub fn with_cover<R: tauri::Runtime, M: Manager<R>>(
    builder: WebviewWindowBuilder<'_, R, M>,
    dark: bool,
    primary: bool,
) -> WebviewWindowBuilder<'_, R, M> {
    let started = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis();
    #[cfg(debug_assertions)]
    let builder = if primary && crate::startup_trace::enabled() {
        builder.initialization_script("window.__atlasStartupTrace = true;")
    } else {
        builder
    };
    #[cfg(windows)]
    let builder = builder.initialization_script("window.__ATLAS_WINDOWS_TABS__ = true;");
    builder
        .background_color(crate::appearance::canvas(dark))
        .initialization_script(crate::tab_links::SCRIPT)
        .initialization_script(format!(
            "window.__atlasOpeningStarted = {started}; window.__atlasOpeningDark = {dark}; window.__atlasPrimaryOpening = {primary};"
        ))
        .initialization_script(include_str!("../../.generated/bootstrap/opening-handoff.js"))
}

pub fn open(app: &AppHandle, job: &BootstrapJob) -> Result<(), String> {
    let window = &job.window;
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
    let attempt = job.attempt();
    let (admitted, navigation) = mpsc::channel();
    {
        let jobs = lock(&desktop.bootstrap_jobs);
        if !jobs.owns(job) {
            return Err("The Atlas startup document was replaced.".into());
        }
        *lock(&desktop.primary) = PrimaryWindow::Armed {
            target: target.clone(),
            route: route.clone(),
            bundled: bundled.clone(),
            attempt,
            admitted,
        };
    }
    job.update(BootstrapUpdate::Navigate(target.to_string()));
    if navigation.recv_timeout(Duration::from_secs(8)).is_err() {
        // Resolve expiry under the same lock as navigation admission. Policy
        // can accept the URL just before the notification reaches this worker.
        let error = "The startup page did not open the verified Atlas view.";
        let expired = {
            let mut primary = lock(&desktop.primary);
            match &*primary {
                PrimaryWindow::Opening {
                    attempt: current, ..
                } if *current == attempt => false,
                PrimaryWindow::Armed {
                    attempt: current, ..
                } if *current == attempt => {
                    *primary = PrimaryWindow::Retry {
                        route: route.clone(),
                        error: error.into(),
                        bundled: bundled.clone(),
                    };
                    true
                }
                _ => return Err(error.into()),
            }
        };
        if expired {
            job.finish(BootstrapUpdate::Retry);
            return Err(error.into());
        }
    }
    #[cfg(debug_assertions)]
    crate::startup_trace::record(
        "native-navigation-admitted",
        serde_json::json!({ "operation": job.id }),
    );
    let result = wait_until_rendered(&desktop, window, &target, &instance, is_retry);
    if let Err(error) = result {
        eprintln!("Atlas could not open the workspace view: {error}");
        #[cfg(debug_assertions)]
        crate::startup_trace::record(
            "native-navigation-failed",
            serde_json::json!({ "error": error }),
        );
        {
            let mut primary = lock(&desktop.primary);
            if !matches!(&*primary, PrimaryWindow::Armed { attempt: current, .. } |
                PrimaryWindow::Opening { attempt: current, .. } if *current == attempt)
            {
                return Err(error);
            }
            *primary = PrimaryWindow::Retry {
                route,
                error: error.clone(),
                bundled: bundled.clone(),
            };
        }
        // Policy already admitted navigation. The committed URL may still be
        // bundled while WebKit is loading the product, so cancel that load.
        job.retire();
        return_to_bundled(window, bundled, &error);
        return Err(error);
    }

    run_on_main(app, move |app| {
        let desktop = app.state::<Desktop>();
        {
            let mut primary = lock(&desktop.primary);
            if !matches!(&*primary, PrimaryWindow::Opening { attempt: current, .. } if *current == attempt)
            {
                return;
            }
            *primary = PrimaryWindow::Product(origin);
        }
        {
            let mut links = lock(&desktop.links);
            if links.front() == Some(&route) {
                links.pop_front();
            }
        }
        #[cfg(debug_assertions)]
        crate::startup_trace::record("native-product-rendered", serde_json::Value::Null);
        crate::copy_link::refresh(app);
        if let Some(window) = app.get_webview_window(PRIMARY) {
            let _ = window.eval("window.dispatchEvent(new Event('atlas-opening-committed'))");
            #[cfg(target_os = "macos")]
            if let Err(error) = crate::window_host::configure(&window, true) {
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

pub fn bundled_document_replaced(desktop: &Desktop) {
    let mut primary = lock(&desktop.primary);
    if let PrimaryWindow::Armed { route, bundled, .. } = &*primary {
        *primary = PrimaryWindow::Retry {
            route: route.clone(),
            error: "The startup page changed before Atlas could open.".into(),
            bundled: bundled.clone(),
        };
    }
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

pub fn retry_before_navigation(app: &AppHandle, job: &BootstrapJob, error: &str) {
    let window = &job.window;
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
        let jobs = lock(&desktop.bootstrap_jobs);
        if !jobs.owns(job) {
            return;
        }
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
    job.finish(BootstrapUpdate::Retry);
}

fn return_to_bundled(window: &WebviewWindow, bundled: Url, error: &str) {
    if let Err(recovery_error) = window.navigate(bundled) {
        if window.url().is_ok_and(|url| bundled_origin(&url))
            && window
                .eval("window.dispatchEvent(new Event('atlas-opening-retry'))")
                .is_ok()
        {
            return;
        }
        let detail = format!("{error}\n\nCould not return to startup: {recovery_error}");
        let _ = run_on_main(window.app_handle(), move |_| {
            crate::window_host::report("Atlas couldn’t open", &detail);
        });
    }
}

fn wait_until_rendered(
    desktop: &Desktop,
    window: &WebviewWindow,
    target: &Url,
    instance: &str,
    is_retry: bool,
) -> Result<(), String> {
    #[cfg(all(debug_assertions, target_os = "macos"))]
    let render_timeout = if !is_retry && crate::smoke::forces_retry() {
        Duration::from_millis(1500)
    } else {
        Duration::from_secs(30)
    };
    #[cfg(not(all(debug_assertions, target_os = "macos")))]
    let render_timeout = {
        let _ = is_retry;
        Duration::from_secs(30)
    };
    let navigation_deadline = Instant::now() + Duration::from_secs(8);
    let mut render_deadline = None;
    while Instant::now() < render_deadline.unwrap_or(navigation_deadline) {
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
            render_deadline.get_or_insert_with(|| Instant::now() + render_timeout);
            if evaluates_true(
                window,
                "Boolean(!document.getElementById('atlas-opening-cover') && document.querySelector('#atlas-main')?.getClientRects().length)",
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
    #[cfg(debug_assertions)]
    if render_deadline.is_some() {
        let _ = window.eval_with_callback(
            "JSON.stringify({ documentReady: document.readyState, shellMounted: !!document.getElementById('atlas-main'), coverVisible: !!document.getElementById('atlas-opening-cover'), compatibilityStatus: document.querySelector('.backend-status')?.textContent?.trim() })",
            |state| eprintln!("Atlas startup render state: {state}"),
        );
    }
    Err(if render_deadline.is_some() {
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
