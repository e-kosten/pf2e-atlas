//! Drives the packaged debug app in an isolated, model-free smoke test.

use std::{
    path::PathBuf,
    sync::mpsc,
    thread,
    time::{Duration, Instant},
};

use tauri::{AppHandle, Manager, WebviewWindow};

use crate::{bundled_origin, display, process, verified_url};

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
    let bootstrap = app
        .get_webview_window("bootstrap")
        .ok_or("setup window was not created")?;
    wait_for(deadline, "fresh setup action", || {
        bootstrap.url().is_ok_and(|url| bundled_origin(&url))
            && evaluate(
                &bootstrap,
                r#"(() => {
                    const button = document.getElementById("primary");
                    if (!button || button.hidden || button.disabled || button.textContent !== "Set up Atlas") return false;
                    button.click();
                    return true;
                })()"#,
            )
    })?;

    let product = wait_for_value(deadline, "rendered Search view", || {
        app.webview_windows().into_values().find(|window| {
            window.label().starts_with("atlas-")
                && window.is_visible().unwrap_or(false)
                && window.url().is_ok_and(|url| {
                    url.path() == "/search" && url.join("/").is_ok_and(|base| verified_url(base.as_str()).is_ok())
                })
                && evaluate(
                    window,
                    r#"Boolean(document.querySelector('input[aria-label="Search records"]')?.getClientRects().length)"#,
                )
        })
    })?;

    let destination = product
        .url()
        .map_err(display)?
        .join("/records/actions:aaaaaaaaaaaaaaaa")
        .map_err(display)?;
    product.navigate(destination).map_err(display)?;
    wait_for(deadline, "rendered fixture record", || {
        evaluate(
            &product,
            r#"[...document.querySelectorAll('#atlas-main h1, #atlas-main h2')].some(
                heading => heading.textContent.trim() === "Handoff Test Action" && heading.getClientRects().length > 0
            )"#,
        )
    })
}

fn evaluate(window: &WebviewWindow, script: &str) -> bool {
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

fn wait_for(
    deadline: Instant,
    stage: &str,
    mut condition: impl FnMut() -> bool,
) -> Result<(), String> {
    wait_for_value(deadline, stage, || condition().then_some(()))
}

fn wait_for_value<T>(
    deadline: Instant,
    stage: &str,
    mut check: impl FnMut() -> Option<T>,
) -> Result<T, String> {
    while Instant::now() < deadline {
        if let Some(value) = check() {
            return Ok(value);
        }
        thread::sleep(Duration::from_millis(50));
    }
    Err(format!("timed out waiting for {stage}"))
}
