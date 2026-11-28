//! Drives the packaged debug app in an isolated, model-free smoke test.

use std::{
    path::PathBuf,
    process::Command,
    sync::atomic::{AtomicUsize, Ordering},
    thread,
    time::{Duration, Instant},
};

use tauri::{AppHandle, Manager, WebviewWindow, WebviewWindowBuilder};

use crate::{
    Desktop, bundled_origin, display, lock,
    opening::{PRIMARY, evaluates_true},
    process, receive_link,
    settings::{self, Destination},
};

const FIXTURE_LINK: &str = "pf2e-atlas://v1/record/actions%3Aaaaaaaaaaaaaaaaa";
const FIXTURE_RECORD: &str = "/records/actions:aaaaaaaaaaaaaaaa";
const FIXTURE_PREVIEW: &str = "/search/records/actions:aaaaaaaaaaaaaaaa";
static SETUP_RUNS: AtomicUsize = AtomicUsize::new(0);

pub fn record_setup() {
    SETUP_RUNS.fetch_add(1, Ordering::Relaxed);
}

pub fn forces_retry() -> bool {
    std::env::var_os("ATLAS_PILOT_SMOKE").is_some_and(|mode| mode == "retry")
}

pub fn forces_preflight_retry() -> bool {
    std::env::var_os("ATLAS_PILOT_SMOKE").is_some_and(|mode| mode == "preflight")
}

pub fn reference_mode() -> bool {
    std::env::var_os("ATLAS_PILOT_SMOKE").is_some_and(|mode| mode == "reference")
}

pub fn selector_mode() -> bool {
    std::env::var_os("ATLAS_PILOT_SMOKE").is_some_and(|mode| mode == "selector")
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
    if !reference_mode() && source()?.is_none() {
        return Ok(());
    }
    if reference_mode() {
        let mut probed = false;
        crate::staged_setup::broadcast_to_documents(app, || {
            probed = true;
            Err("Unexpected selection probe during bootstrap.".into())
        });
        if probed {
            return Err(
                "A setup broadcast probed selection before a product window attached.".into(),
            );
        }
    }
    let history_mode = std::env::var_os("ATLAS_PILOT_SMOKE").is_some_and(|mode| mode == "history");
    if forces_retry() || forces_preflight_retry() || history_mode {
        receive_link(app, FIXTURE_LINK);
    }
    let links_mode = std::env::var_os("ATLAS_PILOT_SMOKE").is_some_and(|mode| mode == "links");
    if links_mode {
        std::fs::write(
            process::home()?.join("link-smoke.pid"),
            std::process::id().to_string(),
        )
        .map_err(display)?;
    }
    let app = app.clone();
    thread::spawn(move || {
        match if std::env::var_os("ATLAS_PILOT_SMOKE").is_some_and(|mode| mode == "channels") {
            run_channels(&app)
        } else if std::env::var_os("ATLAS_PILOT_SMOKE").is_some_and(|mode| mode == "same-channel") {
            run_same_channel(&app)
        } else if std::env::var_os("ATLAS_PILOT_SMOKE").is_some_and(|mode| mode == "upgrade") {
            run_upgrade(&app)
        } else if history_mode {
            run_history(&app)
        } else if links_mode {
            run_links(&app)
        } else if selector_mode() {
            run_selector(&app)
        } else if reference_mode() {
            run_reference(&app)
        } else {
            run(&app)
        } {
            Ok(()) => {
                println!("Atlas native smoke passed");
                app.exit(0);
            }
            Err(error) => {
                eprintln!("Atlas native smoke failed: {error}");
                app.exit(1);
            }
        }
    });
    Ok(())
}

fn upgrade_record(app: &AppHandle, stage: &str, error: Option<&str>) -> Result<(), String> {
    let home = process::home()?;
    let path = home.join("upgrade-smoke-result.json");
    let mut entries: Vec<serde_json::Value> = match std::fs::read(&path) {
        Ok(bytes) => serde_json::from_slice(&bytes).map_err(display)?,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Vec::new(),
        Err(error) => return Err(display(error)),
    };
    entries.push(serde_json::json!({
        "pid": std::process::id(),
        "build": crate::updates::installed_build(),
        "window_url": app.get_webview_window(PRIMARY).and_then(|window| window.url().ok()).map(|url| url.to_string()),
        "stage": stage,
        "error": error,
        "selection": crate::workspace_switch::load(app).ok(),
    }));
    // LaunchServices gives the incoming process /dev/null for stdio. Preserve
    // checkpoints in the disposable home across that real updater restart.
    let mut staged = tempfile::NamedTempFile::new_in(home).map_err(display)?;
    serde_json::to_writer_pretty(staged.as_file_mut(), &entries).map_err(display)?;
    staged.persist(path).map_err(display)?;
    Ok(())
}

fn run_upgrade(app: &AppHandle) -> Result<(), String> {
    upgrade_record(app, "started", None)?;
    let result = run_upgrade_flow(app);
    upgrade_record(
        app,
        if result.is_ok() { "passed" } else { "failed" },
        result.as_ref().err().map(String::as_str),
    )?;
    result
}

fn run_upgrade_flow(app: &AppHandle) -> Result<(), String> {
    use crate::workspace_switch::Role;

    let installed =
        crate::updates::installed_build().ok_or("Upgrade smoke requires a release identity.")?;
    let window = app
        .get_webview_window(PRIMARY)
        .ok_or("primary window unavailable")?;
    let deadline = Instant::now() + Duration::from_secs(45);
    let config = crate::workspace_switch::load(app)?;
    if config.active != Role::Original {
        return Err("Upgrade smoke requires the original workspace to remain committed.".into());
    }
    if installed.tag.starts_with("release/") {
        if config.pending.is_some() {
            return Err("Upgrade smoke requires a fresh Stable review.".into());
        }
        let window = open_settings(app, &window, Destination::Channel, deadline)?;
        for label in [
            "Switch to Nightly…",
            "Use the other copy",
            "Check Nightly compatibility",
        ] {
            wait_for(deadline, label, || click_settings_button(&window, label))?;
        }
        wait_for(deadline, "data upgrade downtime warning", || {
            text_visible(
                &window,
                "This version needs to upgrade your data after restarting. Atlas will be unavailable until the upgrade finishes.",
            )
        })?;
        reset_install_progress()?;
        upgrade_record(app, "awaiting-automatic-restart", None)?;
        wait_for(deadline, "signed Nightly install confirmation", || {
            click_settings_button(&window, "Install Nightly and restart")
        })?;
        verify_install_progress(&window, deadline)?;
        return wait_for(deadline, "automatic target restart", || false);
    }
    if !installed.tag.starts_with("nightly/")
        || !config.pending.as_ref().is_some_and(|pending| {
            pending.target == installed && pending.admission.role == Role::Separate
        })
    {
        return Err("Upgrade smoke requires the reviewed Nightly target and separate copy.".into());
    }
    require_install_progress(&installed.version)?;
    if process::probe(app)?.status != atlas_desktop_protocol::WorkspaceStatus::UpgradeRequired {
        return Err("The incoming target did not require the expected data upgrade.".into());
    }
    wait_for(
        deadline,
        "native upgrade gate after automatic restart",
        || {
            evaluates_true(
                &window,
                r#"(() => {
            const heading = document.getElementById('heading');
            const button = document.getElementById('primary');
            return heading?.getClientRects().length > 0
                && heading.textContent === 'Update your workspace'
                && button?.getClientRects().length > 0
                && button.textContent === 'Update workspace' && !button.disabled;
        })()"#,
            )
        },
    )?;
    upgrade_record(app, "automatic-restart-upgrade-gate", None)?;
    wait_for(deadline, "native workspace preparation", || {
        click_primary(&window, "Update workspace")
    })?;
    wait_for(
        deadline,
        "native data-dependent preparation refusal",
        || {
            evaluates_true(
                &window,
                r#"(() => {
            const diagnostic = document.getElementById('diagnostic');
            const details = document.getElementById('details');
            const button = document.getElementById('primary');
            if (!diagnostic || diagnostic.hidden || !details?.textContent.includes('invalid workspace migration')
                || button?.textContent !== 'Check workspace' || button.disabled) return false;
            diagnostic.open = true;
            return details.getClientRects().length > 0;
        })()"#,
            )
        },
    )?;
    if crate::workspace_switch::load(app)? != config {
        return Err("Failed preparation changed the committed or pending selection.".into());
    }
    upgrade_record(app, "native-preparation-refused", None)?;
    wait_for(deadline, "workspace recheck after refusal", || {
        click_primary(&window, "Check workspace")
    })?;
    wait_for(deadline, "original workspace recovery", || {
        evaluates_true(
            &window,
            r#"(() => {
            const button = document.getElementById('secondary');
            if (!button?.getClientRects().length || button.disabled
                || button.textContent !== 'Open my original workspace') return false;
            button.click();
            return true;
        })()"#,
        )
    })?;
    wait_for(deadline, "recovered product view", || {
        evaluates_true(
            &window,
            r#"document.querySelector('.topbar')?.getClientRects().length > 0
                && window.__ATLAS_DESKTOP_SETUP__?.channel?.active === 'original'"#,
        )
    })?;
    let recovered = crate::workspace_switch::load(app)?;
    if recovered.active != Role::Original || recovered.pending.is_some() {
        return Err("The recovered original workspace still has a pending switch.".into());
    }
    upgrade_record(app, "original-product-recovered", None)
}

fn run_channels(app: &AppHandle) -> Result<(), String> {
    let installed =
        crate::updates::installed_build().ok_or("Channel smoke requires a release identity.")?;
    let window = app
        .get_webview_window(PRIMARY)
        .ok_or("primary window unavailable")?;
    let deadline = Instant::now() + Duration::from_secs(30);
    let window = open_settings(app, &window, Destination::Channel, deadline)?;
    let stable = installed.tag.starts_with("release/");
    if stable {
        require_install_progress(&installed.version)?;
        wait_for(deadline, "Stable channel after restart", || {
            text_visible(&window, &format!("Installed version {}", installed.version))
                && text_visible(&window, "Current channel · Stable")
        })?;
        let config = crate::workspace_switch::load(app)?;
        if config.pending.is_some() {
            return Err("Stable still has a pending switch after opening.".into());
        }
        let expected =
            std::env::var("ATLAS_PILOT_EXPECTED_WORKSPACE").unwrap_or_else(|_| "original".into());
        if config.active.as_arg() != expected {
            return Err("Stable did not retain the expected workspace selection.".into());
        }
        let cli = Command::new(process::bundled_cli()?)
            .env("HOME", process::home()?)
            .arg("desktop-probe")
            .output()
            .map_err(display)?;
        let default: serde_json::Value = serde_json::from_slice(&cli.stdout).map_err(display)?;
        if !cli.status.success()
            || default["data"]["workspace"] != serde_json::json!(process::probe(app)?.workspace)
        {
            return Err(
                "The standalone command default differs from the running Stable workspace.".into(),
            );
        }
        std::fs::write(
            process::home()?.join("channel-switch-complete"),
            installed.version,
        )
        .map_err(display)?;
        return Ok(());
    }
    if crate::workspace_switch::load(app)?.returning.is_none() {
        wait_for(deadline, "start switching to Stable", || {
            click_settings_button(&window, "Switch to Stable…")
        })?;
    }
    wait_for(deadline, "Stable compatibility review", || {
        click_settings_button(&window, "Check Stable compatibility")
    })?;
    reset_install_progress()?;
    if let Err(error) = wait_for(deadline, "signed Stable install confirmation", || {
        click_settings_button(&window, "Install Stable and restart")
    }) {
        let _ = window
            .eval("location.hash = 'smoke_error=' + encodeURIComponent(document.body.innerText)");
        thread::sleep(Duration::from_millis(200));
        let detail = window
            .url()
            .ok()
            .and_then(|url| url.fragment().map(str::to_owned))
            .and_then(|fragment| {
                url::form_urlencoded::parse(fragment.as_bytes())
                    .find(|(key, _)| key == "smoke_error")
                    .map(|(_, value)| value.into_owned())
            })
            .unwrap_or_default();
        return Err(format!("{error}: {detail}"));
    }
    verify_install_progress(&window, deadline)?;
    wait_for(deadline, "installer restart", || false)
}

fn run_same_channel(app: &AppHandle) -> Result<(), String> {
    let installed = crate::updates::installed_build().ok_or("Expected a signed test build.")?;
    let expected = std::env::var("ATLAS_PILOT_EXPECTED_VERSION")
        .map_err(|_| "Expected an explicit target release for the same-channel smoke.")?;
    let origin = app
        .get_webview_window(PRIMARY)
        .ok_or("Primary window unavailable.")?;
    let deadline = Instant::now() + Duration::from_secs(45);
    let settings = open_settings(app, &origin, Destination::Channel, deadline)?;
    if installed.version == expected {
        require_install_progress(&expected)?;
        wait_for(deadline, "installed target after restart", || {
            text_visible(&settings, &format!("Installed version {expected}"))
        })?;
        std::fs::write(process::home()?.join("same-channel-complete"), expected)
            .map_err(display)?;
        return Ok(());
    }
    reset_install_progress()?;
    wait_for(
        deadline,
        "same-channel install with Release channel expanded",
        || {
            evaluates_true(
                &settings,
                "Boolean(document.querySelector('details[open]'))",
            ) && click_settings_button(&settings, "Install and restart")
        },
    )?;
    verify_install_progress(&settings, deadline)?;
    wait_for(deadline, "same-channel installer restart", || false)
}

// Capture the actual WKWebView immediately before the signed installer restarts
// Atlas. A disclosure that mounts its own presenter would duplicate this text.
fn verify_install_progress(window: &WebviewWindow, deadline: Instant) -> Result<(), String> {
    let mut evidence = None;
    wait_for(
        deadline,
        "a single install status on the expanded channel route",
        || {
            let (send, receive) = std::sync::mpsc::channel();
            let script = r#"(() => {
            const main = document.querySelector('main.desktop-settings');
            if (!main || location.pathname !== '/settings/channel') return null;
            const text = main.innerText;
            const status = 'Installing and restarting…';
            const count = text.split(status).length - 1;
            const live = [...main.querySelectorAll('[role=status]')].filter(
                el => el.getClientRects().length && el.textContent.trim() === status
            ).length;
            const version = window.__ATLAS_DESKTOP_SETUP__?.update?.version;
            return count ? { count, live, text, version, route: location.pathname } : null;
        })()"#;
            if window
                .eval_with_callback(script, move |value| {
                    let _ = send.send(value);
                })
                .is_err()
            {
                return false;
            }
            evidence = receive
                .recv_timeout(Duration::from_millis(200))
                .ok()
                .and_then(|value| serde_json::from_str::<serde_json::Value>(&value).ok())
                .filter(|value| !value.is_null());
            evidence.is_some()
        },
    )?;
    let evidence = evidence.ok_or("The install status could not be recorded.")?;
    std::fs::write(
        process::home()?.join("install-progress.json"),
        evidence.to_string(),
    )
    .map_err(display)?;
    Ok(())
}

fn reset_install_progress() -> Result<(), String> {
    std::fs::write(process::home()?.join("install-progress.json"), "null").map_err(display)
}

// The installer can restart while the old process is still waiting to observe
// Settings. The incoming process therefore owns the final assertion.
fn require_install_progress(version: &str) -> Result<(), String> {
    let bytes = std::fs::read(process::home()?.join("install-progress.json")).map_err(display)?;
    let evidence: serde_json::Value = serde_json::from_slice(&bytes).map_err(display)?;
    if evidence["count"] != 1 || evidence["live"] != 1 || evidence["version"] != version {
        return Err(format!(
            "Expected one visible install status and one announcement: {evidence}"
        ));
    }
    Ok(())
}

fn run(app: &AppHandle) -> Result<(), String> {
    let deadline = Instant::now() + Duration::from_secs(10);
    let home = process::home()?.canonicalize().map_err(display)?;
    let probe = process::probe(app)?;
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

    if forces_retry() || forces_preflight_retry() {
        wait_for(deadline, "render failure and retry action", || {
            window
                .ns_window()
                .is_ok_and(|current| current == initial_window)
                && evaluates_true(
                    &window,
                    "document.querySelector('#details')?.textContent.includes('app view did not finish loading') === true ||
                     document.querySelector('#details')?.textContent.includes('local service readiness check failed') === true",
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
        })
        .map_err(|error| {
            let location = window.url().map(|url| url.to_string()).unwrap_or_default();
            let opening_error = lock(&app.state::<Desktop>().primary)
                .error()
                .map(str::to_owned);
            format!("{error}; location {location}; opening error {opening_error:?}")
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
    let expected_path = if forces_retry() || forces_preflight_retry() {
        FIXTURE_RECORD
    } else {
        FIXTURE_PREVIEW
    };
    wait_for(deadline, "rendered fixture record", || {
        let opened = lock(&app.state::<Desktop>().primary).is_product();
        opened
            && window.is_visible().unwrap_or(false)
            && window
                .ns_window()
                .is_ok_and(|current| current == initial_window)
            && fixture_record_rendered(&window, expected_path)
    })?;
    if forces_retry() || forces_preflight_retry() {
        check_history_boundary(&window, deadline)?;
    }
    // Directly launched smoke bundles can remain behind another Atlas instance.
    // The Edit menu follows the key window, as it does during normal use.
    let focused = window.clone();
    crate::run_on_main(app, move |_| {
        let _ = crate::window_host::select(&focused);
    })?;
    wait_for(
        deadline,
        "loaded record copy control and enabled Edit menu",
        || {
            crate::copy_link::menu_item(app).is_some_and(|item| item.is_enabled().unwrap_or(false))
                && evaluates_true(
                    &window,
                    r#"Boolean(document.querySelector('button[aria-label="Copy link"]'))"#,
                )
        },
    )?;
    let check_find = std::env::var_os("ATLAS_PILOT_SMOKE").is_some_and(|mode| mode == "normal");
    if check_find {
        find_in_scrolling_pane(app, &window)?;
    }
    if SETUP_RUNS.load(Ordering::Relaxed) != 1 || app.webview_windows().len() != 1 {
        return Err("opening the record should use one setup run and one window".into());
    }
    wait_for(deadline, "current document setup attachment", || {
        evaluates_true(
            &window,
            r#"(() => {
                const attachment = window.__ATLAS_DESKTOP_SETUP__;
                if (!attachment) return false;
                sessionStorage.setItem("atlas-smoke-token", attachment.token);
                return true;
            })()"#,
        )
    })?;
    window
        .navigate(window.url().map_err(display)?)
        .map_err(display)?;
    wait_for(deadline, "new document setup attachment", || {
        evaluates_true(
            &window,
            r#"Boolean(window.__ATLAS_DESKTOP_SETUP__?.token &&
                sessionStorage.getItem("atlas-smoke-token") !== window.__ATLAS_DESKTOP_SETUP__.token)"#,
        )
    })?;
    check_history_boundary(&window, deadline)?;
    if check_find {
        find_after_reload(app, &window, deadline)?;
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
    )?;
    window.eval(
        r#"void (async () => {
            const ready = await (await fetch("/api/readiness")).json();
            const policy = (await fetch(location.href)).headers.get("content-security-policy");
            const response = await fetch("/api/lists", {
                method: "POST", headers: { "content-type": "application/json" },
                body: JSON.stringify({ name: "Staged setup smoke", slug: "staged-setup-smoke", tags: [] })
            });
            const problem = await response.json();
            const attachment = window.__ATLAS_DESKTOP_SETUP__;
            let rejected = false;
            try {
                await window.__TAURI__.core.invoke("desktop_setup_action", {
                    token: sessionStorage.getItem("atlas-smoke-token"),
                    job: attachment?.setup.job, action: "cancel"
                });
            } catch {
                rejected = true;
            }
            if (ready.saved_writes_paused === true && response.status === 409 &&
                problem.code === "setup_in_progress" && rejected && policy?.includes("frame-src 'none'")) {
                document.documentElement.dataset.atlasSmokePause = "verified";
            }
        })()"#,
    ).map_err(display)?;
    wait_for(
        deadline,
        "paused editing and rejected stale attachment",
        || {
            evaluates_true(
                &window,
                r#"document.documentElement.dataset.atlasSmokePause === "verified""#,
            )
        },
    )?;
    let settings = open_settings(app, &window, Destination::Data, deadline)?;
    wait_for(deadline, "cancel setup from Settings", || {
        click_settings_button(&settings, "Cancel setup")
    })?;
    settings.close().map_err(display)?;
    wait_for(deadline, "Settings closes", || {
        app.get_webview_window(settings::WINDOW).is_none()
    })?;
    wait_for(deadline, "cancel releases edit admission", || {
        evaluates_true(
            &window,
            r#"(() => {
            const root = document.documentElement;
            if (root.dataset.atlasSmokeRestored === "yes") return true;
            if (root.dataset.atlasSmokeChecking) return false;
            root.dataset.atlasSmokeChecking = "true";
            void (async () => {
                try {
                    const readiness = await (await fetch("/api/readiness")).json();
                    if (readiness.saved_writes_paused === true) return;
                    const result = await fetch("/api/lists", {
                        method: "POST", headers: { "content-type": "application/json" },
                        body: JSON.stringify({ name: "Staged setup smoke", slug: "staged-setup-smoke", tags: [] })
                    });
                    if (result.ok) root.dataset.atlasSmokeRestored = "yes";
                } finally {
                    delete root.dataset.atlasSmokeChecking;
                }
            })();
            return false;
        })()"#,
        )
    })?;
    if !forces_retry() && !forces_preflight_retry() {
        check_tabs_and_zoom(app, &window)?;
    }
    Ok(())
}

fn run_history(app: &AppHandle) -> Result<(), String> {
    let home = process::home()?.canonicalize().map_err(display)?;
    let probe = process::probe(app)?;
    if probe.status != atlas_desktop_protocol::WorkspaceStatus::Browsable
        || !probe.workspace.starts_with(home)
    {
        return Err("the history smoke test requires an isolated fixture workspace".into());
    }
    let window = app
        .get_webview_window(PRIMARY)
        .ok_or("Workspace window unavailable")?;
    let deadline = Instant::now() + Duration::from_secs(10);
    wait_for(deadline, "record opened in the native workspace", || {
        lock(&app.state::<Desktop>().primary).is_product()
            && fixture_record_rendered(&window, FIXTURE_RECORD)
    })?;
    check_history_boundary(&window, deadline)?;
    check_tabs_and_zoom(app, &window)
}

fn check_tabs_and_zoom(app: &AppHandle, primary: &WebviewWindow) -> Result<(), String> {
    let primary = primary.clone();
    let (sender, receiver) = std::sync::mpsc::channel();
    crate::run_on_main(app, move |app| {
        let result: Result<(), String> = (|| {
            crate::window_host::select(&primary)?;
            app.state::<Desktop>().zoom.change(app, "zoom-in")?;
            crate::new_search(app, true)?;
            let tab = app
                .get_webview_window("atlas-0")
                .ok_or("New Search tab unavailable")?;
            crate::window_host::verify_group(&primary, &tab)?;
            if !tab.url().is_ok_and(|url| url.path() == "/search") {
                return Err("New tab did not open Search".into());
            }
            let preference = process::home()?.join("desktop-zoom");
            if std::fs::read_to_string(preference).map_err(display)?.trim() != "110" {
                return Err("Zoom was not saved".into());
            }
            crate::window_host::select(&primary)?;
            crate::window_host::verify_group(&tab, &primary)?;
            app.state::<Desktop>().zoom.change(app, "zoom-reset")?;
            Ok(())
        })();
        let _ = sender.send(result);
    })?;
    receiver
        .recv_timeout(Duration::from_secs(4))
        .map_err(display)??;
    let check_titles = |stage, primary_title| {
        let mut observed = Vec::new();
        let result = wait_for(
            Instant::now() + Duration::from_secs(4),
            &format!("{stage} native tab labels {primary_title:?} and Search"),
            || {
                let (sender, receiver) = std::sync::mpsc::channel();
                let queued = crate::run_on_main(app, move |app| {
                    let tabs = [PRIMARY, "atlas-0"]
                        .into_iter()
                        .filter_map(|label| app.get_webview_window(label))
                        .map(|window| {
                            (
                                crate::window_host::tab_title(&window),
                                window
                                    .url()
                                    .map(|url| url.path().to_owned())
                                    .map_err(display),
                            )
                        })
                        .collect::<Vec<_>>();
                    let _ = sender.send(tabs);
                });
                if queued.is_ok()
                    && let Ok(tabs) = receiver.recv_timeout(Duration::from_millis(250))
                {
                    observed = tabs;
                }
                observed.len() == 2
                    && observed.iter().zip([primary_title, "Search"]).all(
                        |((title, _), expected)| {
                            title.as_ref().is_ok_and(|title| title == expected)
                        },
                    )
            },
        );
        result.map_err(|error| format!("{error}; observed {observed:?}"))
    };
    check_titles("initial", "Handoff Test Action")?;
    let primary = app
        .get_webview_window(PRIMARY)
        .ok_or("Primary record tab unavailable")?;
    if !evaluates_true(
        &primary,
        r#"(() => {
            const link = document.querySelector('a[href="/lists"]');
            if (!link) return false;
            link.click();
            return true;
        })()"#,
    ) {
        return Err("Could not open Lists inside the record tab".into());
    }
    check_titles("after navigation", "Lists")?;
    wait_for(
        Instant::now() + Duration::from_secs(5),
        "Back toolbar button enabled",
        || {
            evaluates_true(
                &primary,
                r#"document.querySelector('button[aria-label="Back"]:enabled') !== null"#,
            )
        },
    )?;
    primary
        .eval(r#"document.querySelector('button[aria-label="Back"]').click()"#)
        .map_err(display)?;
    check_titles("after Back", "Handoff Test Action")?;
    wait_for(
        Instant::now() + Duration::from_secs(5),
        "Forward toolbar button enabled at the document boundary",
        || {
            evaluates_true(
                &primary,
                r#"document.querySelector('button[aria-label="Forward"]:enabled') !== null && document.querySelector('button[aria-label="Back"]:disabled') !== null"#,
            )
        },
    )?;
    let (sent, received) = std::sync::mpsc::channel();
    crate::run_on_main(app, move |_| {
        let _ = sent.send(menu_shortcut("]", 30));
    })?;
    received
        .recv_timeout(Duration::from_secs(4))
        .map_err(display)??;
    check_titles("after Forward shortcut", "Lists")?;
    primary
        .eval(
            r#"(() => {
        let index = history.state.__TSR_index;
        for (const href of ["/encounters", "/characters"]) {
            const key = crypto.randomUUID();
            history.pushState({ __TSR_index: ++index, __TSR_key: key, key }, "", href);
        }
    })()"#,
        )
        .map_err(display)?;
    check_titles("after two pushes in one callback", "Characters")?;
    wait_for(
        Instant::now() + Duration::from_secs(3),
        "Back available after two pushes",
        || {
            evaluates_true(
                &primary,
                r#"document.querySelector('button[aria-label="Back"]:enabled') !== null"#,
            )
        },
    )?;
    primary
        .eval(r#"document.querySelector('button[aria-label="Back"]').click()"#)
        .map_err(display)?;
    check_titles("after Back to the intermediate native entry", "Encounters")
}

fn check_history_boundary(window: &WebviewWindow, deadline: Instant) -> Result<(), String> {
    wait_for(
        deadline,
        "desktop history starts at the attached document",
        || {
            evaluates_true(
                window,
                r#"
            Boolean(window.__ATLAS_DESKTOP_SETUP__?.token) &&
            document.querySelector('button[aria-label="Back"]:disabled') !== null &&
            document.querySelector('button[aria-label="Forward"]:disabled') !== null
        "#,
            )
        },
    )?;
    window
        .eval(
            r#"(() => {
        const href = location.href;
        const token = window.__ATLAS_DESKTOP_SETUP__.token;
        delete document.documentElement.dataset.atlasSmokeHistoryBoundary;
        window.history.back();
        setTimeout(() => {
            if (location.href === href && window.__ATLAS_DESKTOP_SETUP__?.token === token) {
                document.documentElement.dataset.atlasSmokeHistoryBoundary = "verified";
            }
        }, 150);
    })()"#,
        )
        .map_err(display)?;
    wait_for(
        deadline,
        "raw Back stayed inside the desktop document boundary",
        || {
            evaluates_true(
                window,
                r#"document.documentElement.dataset.atlasSmokeHistoryBoundary === "verified""#,
            )
        },
    )
}

fn fixture_record_rendered(window: &WebviewWindow, path: &str) -> bool {
    window
        .url()
        .is_ok_and(|url| url.path().replace("%3A", ":") == path)
        && evaluates_true(
            window,
            r#"[...document.querySelectorAll('#atlas-main h1, #atlas-main h2')].some(
                heading => heading.textContent.trim() === "Handoff Test Action" && heading.getClientRects().length > 0
            )"#,
        )
}

fn run_links(app: &AppHandle) -> Result<(), String> {
    let home = process::home()?.canonicalize().map_err(display)?;
    let probe = process::probe(app)?;
    if probe.status != atlas_desktop_protocol::WorkspaceStatus::Browsable
        || !probe.workspace.starts_with(&home)
    {
        return Err("the link smoke test requires a seeded disposable workspace".into());
    }
    let bundle = home.join("Atlas Link Smoke.app");
    if !std::env::current_exe().is_ok_and(|exe| exe.starts_with(&bundle)) {
        return Err("the link smoke test requires an isolated app bundle".into());
    }
    let primary = app
        .get_webview_window(PRIMARY)
        .ok_or("primary link test window unavailable")?;
    let deadline = Instant::now() + Duration::from_secs(10);
    wait_for(deadline, "standalone record from cold OS URL", || {
        lock(&app.state::<Desktop>().primary).is_product()
            && fixture_record_rendered(&primary, FIXTURE_RECORD)
    })?;
    if app.webview_windows().len() != 1 {
        return Err("the cold link should open in one window".into());
    }

    let preview = primary
        .url()
        .map_err(display)?
        .join(FIXTURE_PREVIEW)
        .map_err(display)?;
    primary.navigate(preview).map_err(display)?;
    wait_for(deadline, "Search preview before warm OS URL", || {
        fixture_record_rendered(&primary, FIXTURE_PREVIEW)
    })?;
    let send_url = || {
        let status = Command::new("/usr/bin/open")
            .arg("-a")
            .arg(&bundle)
            .args(["-u", FIXTURE_LINK])
            .status()
            .map_err(display)?;
        if status.success() {
            Ok(())
        } else {
            Err(format!("macOS did not deliver the Atlas URL: {status}"))
        }
    };

    let _settings = open_settings(app, &primary, Destination::Updates, deadline)?;
    send_url()?;
    let deadline = Instant::now() + Duration::from_secs(5);
    wait_for(deadline, "foreground record tab from warm OS URL", || {
        app.get_webview_window("atlas-0").is_some_and(|record| {
            record.is_focused().unwrap_or(false) && fixture_record_rendered(&record, FIXTURE_RECORD)
        })
    })?;
    let (sender, receiver) = std::sync::mpsc::channel();
    crate::run_on_main(app, move |_| {
        let _ = sender.send(crate::window_host::select(&primary));
    })?;
    receiver
        .recv_timeout(Duration::from_secs(2))
        .map_err(display)??;
    send_url()?;
    let deadline = Instant::now() + Duration::from_secs(5);
    wait_for(
        deadline,
        "existing standalone record tab after reopening URL",
        || {
            app.webview_windows().len() == 3
                && app.get_webview_window("atlas-0").is_some_and(|record| {
                    record.is_focused().unwrap_or(false)
                        && fixture_record_rendered(&record, FIXTURE_RECORD)
                })
        },
    )
}

fn run_reference(app: &AppHandle) -> Result<(), String> {
    let deadline = Instant::now() + Duration::from_secs(10);
    let home = process::home()?.canonicalize().map_err(display)?;
    let probe = process::probe(app)?;
    let initial = process::reference_status(app)?;
    if probe.status != atlas_desktop_protocol::WorkspaceStatus::Browsable
        || !probe.workspace.starts_with(&home)
        || initial.offer.is_none()
    {
        return Err("the reference smoke test requires a disposable managed workspace".into());
    }
    let window = app
        .get_webview_window(PRIMARY)
        .ok_or("primary window unavailable")?;
    let terminal = open_settings(app, &window, Destination::Terminal, deadline)?;
    wait_for(deadline, "terminal access action", || {
        click_settings_button(&terminal, "Install terminal command")
    })?;
    let linked = home.join(".local/bin/atlas");
    wait_for(deadline, "bundled terminal command linked", || {
        linked
            .read_link()
            .is_ok_and(|path| process::bundled_cli().is_ok_and(|cli| path == cli))
    })?;
    wait_for(deadline, "terminal link status", || {
        text_visible(&terminal, "The terminal command is installed.")
    })?;
    let settings = open_settings(app, &window, Destination::Data, deadline)?;
    wait_for(deadline, "reference action", || {
        click_settings_button(&settings, "Check and repair reference data")
    })?;
    wait_for(deadline, "shared reference setup job", || {
        SETUP_RUNS.load(Ordering::Relaxed) == 1 && text_visible(&settings, "Cancel setup")
    })?;

    crate::run_on_main(app, |app| {
        let _ = crate::open_product(app, "/search", None);
    })?;
    wait_for(deadline, "second product window", || {
        app.webview_windows().len() == 3
    })?;
    let second = app
        .webview_windows()
        .into_values()
        .find(|view| view.label() != PRIMARY && view.label() != settings::WINDOW)
        .ok_or("second product window unavailable")?;
    let reopened = open_settings(app, &second, Destination::Data, deadline)?;
    if reopened.ns_window().map_err(display)? != settings.ns_window().map_err(display)? {
        return Err("Opening Settings created another native window.".into());
    }
    wait_for(deadline, "running job survives refocusing Settings", || {
        text_visible(&settings, "Cancel setup")
    })?;
    for view in [&window, &second, &settings] {
        if !evaluates_true(
            view,
            r#"(() => {
            const before = window.__ATLAS_DESKTOP_SETUP__;
            window.addEventListener('atlas-desktop-setup', () => {
                const after = window.__ATLAS_DESKTOP_SETUP__;
                window.selectionFailureDelivered = after.token === before.token &&
                    after.setup.job === before.setup.job && after.update === null && after.channel === null;
            }, { once: true });
            return true;
        })()"#,
        ) {
            return Err("Could not observe the next setup broadcast.".into());
        }
    }
    for fail in [true, false] {
        let (send, receive) = std::sync::mpsc::channel();
        crate::run_on_main(app, move |app| {
            let mut reads = 0;
            crate::staged_setup::broadcast_to_documents(app, || {
                reads += 1;
                if fail {
                    Err("Unavailable smoke selection.".into())
                } else {
                    process::workspace_selection_snapshot()
                }
            });
            let _ = send.send(reads);
        })?;
        let reads = receive
            .recv_timeout(Duration::from_secs(2))
            .map_err(display)?;
        if reads != 1 {
            return Err(format!(
                "Shared setup broadcast read workspace selection {reads} times."
            ));
        }
        if fail {
            wait_for(deadline, "setup delivery after selection failure", || {
                [&window, &second, &settings]
                    .into_iter()
                    .all(|view| evaluates_true(view, "window.selectionFailureDelivered === true"))
            })?;
        }
    }
    wait_for(deadline, "cancel the shared job", || {
        click_settings_button(&settings, "Cancel setup")
    })?;
    wait_for(deadline, "cancelled setup state", || {
        text_visible(&settings, "Setup was cancelled.")
    })?;
    let pinned_source = std::fs::read_dir(home.join("Library/Caches/pf2e-atlas/pf2e"))
        .map_err(display)?
        .next()
        .ok_or("managed fixture source missing")?
        .map_err(display)?
        .path();
    std::fs::remove_file(pinned_source.join(".hold-reference")).map_err(display)?;
    wait_for(deadline, "retry reference setup", || {
        click_settings_button(&settings, "Check and repair reference data")
    })?;
    wait_for(deadline, "verified no-op", || {
        text_visible(&settings, "Reference data is already up to date.")
    })?;
    if process::reference_status(app)?.generation != initial.generation {
        return Err("the no-op changed the committed generation".into());
    }

    let record = pinned_source.join("packs/actions/handoff.json");
    let mut source: serde_json::Value =
        serde_json::from_slice(&std::fs::read(&record).map_err(display)?).map_err(display)?;
    source["name"] = "Updated Handoff Test Action".into();
    std::fs::write(record, source.to_string()).map_err(display)?;
    wait_for(deadline, "explicit reference refresh", || {
        click_settings_button(&settings, "Check and repair reference data")
    })?;
    wait_for(deadline, "reference publication confirmed", || {
        text_visible(&settings, "Reference data updated.")
    })?;
    if SETUP_RUNS.load(Ordering::Relaxed) != 3
        || process::reference_status(app)?.generation == initial.generation
        || app.webview_windows().len() != 3
    {
        return Err(
            "the reference refresh should publish once without replacing either window".into(),
        );
    }
    settings.close().map_err(display)?;
    wait_for(
        deadline,
        "both workspaces remain after closing Settings",
        || {
            app.webview_windows().len() == 2
                && [&window, &second].into_iter().all(|view| {
                    evaluates_true(
                        view,
                        "Boolean(document.querySelector('.topbar')?.getClientRects().length)",
                    )
                })
        },
    )?;
    let restored = open_settings(app, &second, Destination::General, deadline)?;
    wait_for(
        deadline,
        "Settings remembers the data pane and result",
        || text_visible(&restored, "Reference data updated."),
    )
}

fn run_selector(app: &AppHandle) -> Result<(), String> {
    let window = app
        .get_webview_window(PRIMARY)
        .ok_or("primary window unavailable")?;
    wait_for(
        Instant::now() + Duration::from_secs(5),
        "delayed opening indicator without setup controls for an existing workspace",
        || {
            window.url().is_ok_and(|url| bundled_origin(&url))
                && evaluates_true(
                    &window,
                    "Boolean(document.querySelector('#opening-status')?.getClientRects().length
                        && !document.getElementById('workspace'))",
                )
        },
    )?;
    let settings = open_settings(
        app,
        &window,
        Destination::Data,
        Instant::now() + Duration::from_secs(10),
    )?;
    check_model_selector(&settings)
}

fn check_model_selector(window: &WebviewWindow) -> Result<(), String> {
    let deadline = Instant::now() + Duration::from_secs(5);
    window
        .set_size(tauri::LogicalSize::new(480.0, 720.0))
        .map_err(display)?;
    // Supply model availability to this isolated WebView fixture without loading models.
    wait_for(deadline, "open model selector", || {
        evaluates_true(
            window,
            r#"(() => {
        const state = window.__ATLAS_DESKTOP_SETUP__;
        if (!document.getElementById('desktop-setup-models') || !state?.reference || state.referenceLoading)
            return false;
        if (state !== window.__atlasSmokeModels) {
            window.__atlasSmokeSetup = state;
            window.__atlasSmokeModels = {
                ...state, reference: { ...state.reference, modelsAvailable: true, canChangeModels: true,
                    preferred: 'bge-base', models: [
                        {id: 'bge-base', label: 'BGE Base', installed: true, usable: true},
                        {id: 'embeddinggemma', label: 'EmbeddingGemma', installed: true, usable: true}
                    ]}
            };
            window.__ATLAS_DESKTOP_SETUP__ = window.__atlasSmokeModels;
            window.dispatchEvent(new Event('atlas-desktop-setup'));
            return false;
        }
        const field = [...document.querySelectorAll('label')].find(
            label => label.textContent === 'Preferred search model')?.control;
        const select = field?.closest('.ant-select');
        if (!select || field.disabled || !select.textContent.includes('BGE Base')) return false;
        select.dataset.smokeShortWidth = select.offsetWidth;
        field.dispatchEvent(new MouseEvent('mousedown', {bubbles: true}));
        return true;
    })()"#,
        )
    })?;
    wait_for(deadline, "full EmbeddingGemma option in viewport", || {
        evaluates_true(
            window,
            r#"(() => {
        const option = [...document.querySelectorAll('.ant-select-item-option-content')].find(
            item => item.textContent === 'EmbeddingGemma' && item.getClientRects().length);
        if (!option || option.clientWidth < option.scrollWidth ||
            option.getBoundingClientRect().right > innerWidth) return false;
        option.click();
        return true;
    })()"#,
        )
    })?;
    wait_for(deadline, "full selected model label", || {
        evaluates_true(
            window,
            r#"(() => {
        const field = [...document.querySelectorAll('label')].find(
            label => label.textContent === 'Preferred search model')?.control;
        const select = field?.closest('.ant-select');
        const value = select?.querySelector('.ant-select-content');
        return value?.textContent === 'EmbeddingGemma' && value.clientWidth >= value.scrollWidth
            && select.offsetWidth > Number(select.dataset.smokeShortWidth);
    })()"#,
        )
    })?;
    window
        .eval(
            r#"window.__ATLAS_DESKTOP_SETUP__ = window.__atlasSmokeSetup;
        delete window.__atlasSmokeSetup;
        delete window.__atlasSmokeModels;
        window.dispatchEvent(new Event('atlas-desktop-setup'));"#,
        )
        .map_err(display)
}

fn open_settings(
    app: &AppHandle,
    origin: &WebviewWindow,
    pane: Destination,
    deadline: Instant,
) -> Result<WebviewWindow, String> {
    wait_for(
        deadline,
        "workspace attached before opening Settings",
        || {
            lock(&app.state::<Desktop>().primary).is_product()
                && evaluates_true(origin, "Boolean(window.__ATLAS_DESKTOP_SETUP__?.token)")
        },
    )?;
    let origin = origin.clone();
    let (send, receive) = std::sync::mpsc::channel();
    crate::run_on_main(app, move |app| {
        let result = if pane == Destination::General {
            menu_shortcut(",", 43)
        } else {
            settings::open(app, Some(&origin), pane)
        };
        let _ = send.send(result);
    })?;
    receive
        .recv_timeout(Duration::from_secs(3))
        .map_err(display)??;
    wait_for(deadline, "attached Settings window", || {
        app.get_webview_window(settings::WINDOW)
            .is_some_and(|view| {
                evaluates_true(
                    &view,
                    "Boolean(window.__ATLAS_DESKTOP_SETUP__?.settingsWindow)",
                )
            })
    })?;
    app.get_webview_window(settings::WINDOW)
        .ok_or_else(|| "Settings window closed.".into())
}

// Drive the application's actual AppKit menu accelerator without requiring
// Accessibility permission or synthesizing input into another application.
fn menu_shortcut(key: &str, code: u16) -> Result<(), String> {
    use objc2::MainThreadMarker;
    use objc2_app_kit::{NSApplication, NSEventModifierFlags};
    let main = MainThreadMarker::new().ok_or("Menu shortcuts need the main thread.")?;
    let app = NSApplication::sharedApplication(main);
    let menu = app.mainMenu().ok_or("The Atlas menu is unavailable.")?;
    let window = app.keyWindow().ok_or("Atlas has no key window.")?;
    let event = key_event(&window, key, code, NSEventModifierFlags::Command)?;
    if menu.performKeyEquivalent(&event) {
        Ok(())
    } else {
        Err(format!("The Atlas menu did not handle Command-{key}."))
    }
}

fn key_event(
    window: &objc2_app_kit::NSWindow,
    key: &str,
    code: u16,
    modifiers: objc2_app_kit::NSEventModifierFlags,
) -> Result<objc2::rc::Retained<objc2_app_kit::NSEvent>, String> {
    use objc2_app_kit::{NSEvent, NSEventType};
    use objc2_foundation::{NSPoint, NSString};
    let key = NSString::from_str(key);
    NSEvent::keyEventWithType_location_modifierFlags_timestamp_windowNumber_context_characters_charactersIgnoringModifiers_isARepeat_keyCode(
        NSEventType::KeyDown, NSPoint::new(0.0, 0.0), modifiers, 0.0,
        window.windowNumber(), None, &key, &key, false, code,
    ).ok_or_else(|| "Could not construct the smoke key event.".to_owned())
}

fn find_in_scrolling_pane(app: &AppHandle, window: &WebviewWindow) -> Result<(), String> {
    use objc2_app_kit::NSEventModifierFlags;

    // A deterministic offscreen match in a pane of the live product webview.
    window
        .eval(
            r#"
        const pane = document.createElement('div');
        pane.id = 'smoke-find-pane';
        sessionStorage.setItem('atlas-smoke-page-height', String(innerHeight));
        pane.style.cssText = 'position:fixed;top:0;left:0;height:80px;overflow:auto;z-index:9999';
        pane.innerHTML = '<div style="height:700px">Start</div><strong>Quetzal</strong>';
        document.body.append(pane);
    "#,
        )
        .map_err(display)?;
    wait_for(
        Instant::now() + Duration::from_secs(2),
        "offscreen Find fixture",
        || {
            evaluates_true(
                window,
                "document.querySelector('#smoke-find-pane strong')?.getBoundingClientRect().top > 600",
            )
        },
    )?;
    in_key_window(app, |native| {
        menu_shortcut("f", 3)?;
        menu_shortcut("a", 0)?;
        let event = key_event(native, "Quetzal", 12, NSEventModifierFlags::empty())?;
        native.sendEvent(&event);
        Ok(())
    })?;
    wait_for(
        Instant::now() + Duration::from_secs(3),
        "highlighted and revealed Find match",
        || {
            evaluates_true(
                window,
                r#"(() => {
            const pane = document.querySelector('#smoke-find-pane');
            const rect = pane.lastElementChild.getBoundingClientRect();
            return innerHeight < Number(sessionStorage.getItem('atlas-smoke-page-height'))
                && getSelection().toString() === 'Quetzal' && pane.scrollTop > 0
                && rect.top >= pane.getBoundingClientRect().top && rect.bottom <= pane.getBoundingClientRect().bottom;
        })()"#,
            )
        },
    )
}

fn find_after_reload(
    app: &AppHandle,
    window: &WebviewWindow,
    deadline: Instant,
) -> Result<(), String> {
    wait_for(deadline, "Find visibility after reload", || {
        evaluates_true(
            window,
            "document.documentElement.hasAttribute('data-atlas-find-open')",
        )
    })?;
    in_key_window(app, |native| {
        let escape = key_event(
            native,
            "\u{1b}",
            53,
            objc2_app_kit::NSEventModifierFlags::empty(),
        )?;
        native.sendEvent(&escape);
        Ok(())
    })?;
    wait_for(deadline, "Escape restored the full page", || {
        evaluates_true(
            window,
            "!document.documentElement.hasAttribute('data-atlas-find-open') && innerHeight === Number(sessionStorage.getItem('atlas-smoke-page-height'))",
        )
    })?;
    window
        .eval(
            r#"
        const draft = document.createElement('input');
        draft.id = 'smoke-find-draft';
        draft.value = 'saved draft';
        document.body.prepend(draft, document.createTextNode('Tanager'));
        draft.focus();
    "#,
        )
        .map_err(display)?;
    wait_for(deadline, "focused editable document field", || {
        evaluates_true(window, "document.activeElement?.id === 'smoke-find-draft'")
    })?;
    in_key_window(app, |_| menu_shortcut("f", 3))?;
    wait_for(deadline, "Find reopened after Escape", || {
        evaluates_true(
            window,
            "document.documentElement.hasAttribute('data-atlas-find-open')",
        )
    })?;
    in_key_window(app, |native| {
        menu_shortcut("a", 0)?;
        let event = key_event(
            native,
            "Tanager",
            17,
            objc2_app_kit::NSEventModifierFlags::empty(),
        )?;
        native.sendEvent(&event);
        Ok(())
    })?;
    wait_for(deadline, "reopened Find received the new query", || {
        evaluates_true(
            window,
            "getSelection().toString() === 'Tanager' && document.querySelector('#smoke-find-draft')?.value === 'saved draft'",
        )
    })
}

fn in_key_window(
    app: &AppHandle,
    action: impl FnOnce(&objc2_app_kit::NSWindow) -> Result<(), String> + Send + 'static,
) -> Result<(), String> {
    let (send, recv) = std::sync::mpsc::channel();
    crate::run_on_main(app, move |_| {
        let result = objc2::MainThreadMarker::new()
            .and_then(|main| objc2_app_kit::NSApplication::sharedApplication(main).keyWindow())
            .ok_or_else(|| "No key window for the Find smoke check.".to_owned())
            .and_then(|window| action(&window));
        let _ = send.send(result);
    })?;
    recv.recv_timeout(Duration::from_secs(3)).map_err(display)?
}

fn click_settings_button(window: &WebviewWindow, text: &str) -> bool {
    let label = serde_json::json!(text);
    evaluates_true(
        window,
        &format!(
            r#"(() => {{
            const button = [...document.querySelectorAll("button")].find(
                item => (item.textContent.trim() === {label} || item.getAttribute("aria-label") === {label})
                    && item.getClientRects().length > 0);
            if (!button || button.disabled) return false;
            button.click();
            return true;
        }})()"#
        ),
    )
}

fn text_visible(window: &WebviewWindow, text: &str) -> bool {
    let text = serde_json::json!(text);
    evaluates_true(
        window,
        &format!(
            r#"[...document.querySelectorAll('body *')].some(
            item => item.textContent.trim() === {text} && item.getClientRects().length > 0)"#
        ),
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
