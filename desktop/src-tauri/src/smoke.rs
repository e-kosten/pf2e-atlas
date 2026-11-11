//! Drives the packaged debug app in an isolated, model-free smoke test.

use std::{
    path::PathBuf,
    sync::atomic::{AtomicUsize, Ordering},
    thread,
    time::{Duration, Instant},
};

use tauri::{AppHandle, Manager, WebviewWindow, WebviewWindowBuilder};

use crate::{
    Desktop, bundled_origin, display, lock,
    opening::{PRIMARY, evaluates_true},
    process, receive_link,
};

const FIXTURE_LINK: &str = "pf2e-atlas://v1/record/actions%3Aaaaaaaaaaaaaaaaa";
static SETUP_RUNS: AtomicUsize = AtomicUsize::new(0);

pub fn record_setup() {
    SETUP_RUNS.fetch_add(1, Ordering::Relaxed);
}

pub fn forces_retry() -> bool {
    std::env::var_os("ATLAS_PILOT_SMOKE").is_some_and(|mode| mode == "retry")
}

pub fn configure(
    builder: WebviewWindowBuilder<'_, tauri::Wry, AppHandle>,
) -> WebviewWindowBuilder<'_, tauri::Wry, AppHandle> {
    if !forces_retry() {
        return builder;
    }
    builder.initialization_script(
        r#"if (location.protocol === "http:" && location.hostname === "127.0.0.1"
            && !sessionStorage.getItem("atlas-smoke-blank")) {
            sessionStorage.setItem("atlas-smoke-blank", "1");
            document.addEventListener("DOMContentLoaded", () => {
                document.getElementById("root").style.setProperty("display", "none", "important");
            }, { once: true });
        }"#,
    )
}

pub fn source() -> Result<Option<PathBuf>, String> {
    if std::env::var_os("ATLAS_PILOT_SMOKE").is_none() {
        return Ok(None);
    }
    let home = std::env::var_os("ATLAS_PILOT_HOME")
        .map(PathBuf::from)
        .filter(|path| path.is_absolute() && path.is_dir())
        .ok_or("The native smoke test requires a disposable Atlas home.")?;
    let source = home.join("source");
    if !source.join("module.json").is_file() {
        return Err("The native smoke test fixture is missing.".into());
    }
    Ok(Some(source))
}

pub fn start(app: &AppHandle) -> Result<(), String> {
    if source()?.is_none() {
        return Ok(());
    }
    if forces_retry() {
        receive_link(app, FIXTURE_LINK);
    }
    let app = app.clone();
    thread::spawn(move || match run(&app) {
        Ok(()) => {
            println!("Atlas native smoke passed");
            app.exit(0);
        }
        Err(error) => {
            eprintln!("Atlas native smoke failed: {error}");
            app.exit(1);
        }
    });
    Ok(())
}

fn run(app: &AppHandle) -> Result<(), String> {
    let deadline = Instant::now() + Duration::from_secs(10);
    let home = process::home()?.canonicalize().map_err(display)?;
    let probe = process::probe()?;
    if probe.status != atlas_desktop_protocol::WorkspaceStatus::NeedsSetup
        || !probe.workspace.starts_with(home)
        || probe.workspace.exists()
    {
        return Err("the smoke test requires a fresh, isolated workspace".into());
    }
    let window = app
        .get_webview_window(PRIMARY)
        .ok_or("setup window was not created")?;
    let initial_window = window.ns_window().map_err(display)?;
    wait_for(deadline, "fresh setup action", || {
        click_primary(&window, "Set up Atlas")
    })?;

    if forces_retry() {
        wait_for(deadline, "render failure and retry action", || {
            window
                .ns_window()
                .is_ok_and(|current| current == initial_window)
                && evaluates_true(
                    &window,
                    "document.querySelector('#details')?.textContent.includes('app view did not finish loading') === true",
                )
                && click_primary(&window, "Retry opening Atlas")
        })?;
    } else {
        wait_for(deadline, "rendered Search view", || {
            let opened = lock(&app.state::<Desktop>().primary).is_product();
            opened
                && window.url().is_ok_and(|url| url.path() == "/search")
                && evaluates_true(
                    &window,
                    r#"Boolean(document.querySelector('input[aria-label="Search records"]')?.getClientRects().length)"#,
                )
        })?;
        let destination = window
            .url()
            .map_err(display)?
            .join("/search?q=Handoff%20Test%20Action")
            .map_err(display)?;
        window.navigate(destination).map_err(display)?;
        wait_for(deadline, "keyword search result", || {
            evaluates_true(
                &window,
                r#"(() => {
                    const keyword = [...document.querySelectorAll('[role="status"]')]
                        .some(status => status.textContent.trim() === "Keyword search");
                    const row = document.querySelector('[data-record-key="actions:aaaaaaaaaaaaaaaa"]');
                    if (!keyword || !row?.getClientRects().length) return false;
                    row.click();
                    return true;
                })()"#,
            )
        })?;
    }
    let expected = atlas_desktop_protocol::parse_record_link(FIXTURE_LINK).map_err(display)?;
    wait_for(deadline, "rendered fixture record", || {
        let opened = lock(&app.state::<Desktop>().primary).is_product();
        opened
            && window.is_visible().unwrap_or(false)
            && window
                .ns_window()
                .is_ok_and(|current| current == initial_window)
            && window.url().is_ok_and(|url| {
                atlas_desktop_protocol::parse_record_path(url.path()).as_ref() == Some(&expected)
            })
            && evaluates_true(
                &window,
                r#"[...document.querySelectorAll('#atlas-main h1, #atlas-main h2')].some(
                heading => heading.textContent.trim() === "Handoff Test Action" && heading.getClientRects().length > 0
            )"#,
            )
    })?;
    if SETUP_RUNS.load(Ordering::Relaxed) != 1 || app.webview_windows().len() != 1 {
        return Err("opening the record should use one setup run and one window".into());
    }
    window
        .eval(
            r#"void (async () => {
                try {
                    await window.__TAURI__.core.invoke("inspect");
                    document.documentElement.dataset.atlasSmokeInspect = "allowed";
                } catch {
                    document.documentElement.dataset.atlasSmokeInspect = "denied";
                }
            })()"#,
        )
        .map_err(display)?;
    wait_for(
        deadline,
        "setup command rejection from the product origin",
        || {
            evaluates_true(
                &window,
                r#"document.documentElement.dataset.atlasSmokeInspect === "denied""#,
            )
        },
    )
}

fn click_primary(window: &WebviewWindow, label: &str) -> bool {
    if !window.url().is_ok_and(|url| bundled_origin(&url)) {
        return false;
    }
    let Ok(label) = serde_json::to_string(label) else {
        return false;
    };
    evaluates_true(
        window,
        &format!(
            r#"(() => {{
            const button = document.getElementById("primary");
            if (!button || button.hidden || button.disabled || button.textContent !== {label}) return false;
            button.click();
            return true;
        }})()"#
        ),
    )
}

fn wait_for(
    deadline: Instant,
    stage: &str,
    mut condition: impl FnMut() -> bool,
) -> Result<(), String> {
    while Instant::now() < deadline {
        if condition() {
            return Ok(());
        }
        thread::sleep(Duration::from_millis(50));
    }
    Err(format!("timed out waiting for {stage}"))
}
