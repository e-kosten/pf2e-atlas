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
    if forces_retry() || forces_preflight_retry() {
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

fn run_channels(app: &AppHandle) -> Result<(), String> {
    let installed =
        crate::updates::installed_build().ok_or("Channel smoke requires a release identity.")?;
    let window = app
        .get_webview_window(PRIMARY)
        .ok_or("primary window unavailable")?;
    let deadline = Instant::now() + Duration::from_secs(30);
    wait_for(deadline, "desktop settings", || open_setup_panel(&window))?;
    let stable = installed.tag.starts_with("release/");
    if stable {
        wait_for(deadline, "preview build disclosure", || {
            evaluates_true(
                &window,
                r#"(() => {
                const summary = document.querySelector('.desktop-channel__preview summary');
                if (!summary?.getClientRects().length) return false;
                if (!summary.parentElement.open) summary.click();
                return true;
            })()"#,
            )
        })?;
    }
    wait_for(deadline, "release channel controls", || {
        click_product_button(
            &window,
            if stable {
                "Try Nightly…"
            } else {
                "Change release channel"
            },
            true,
        ) || (!stable && click_product_button(&window, "Stable available", true))
    })?;
    if stable {
        wait_for(deadline, "Stable channel after restart", || {
            text_visible(&window, &format!("Version {}", installed.version))
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
    wait_for(deadline, "Stable compatibility review", || {
        click_product_button(&window, "Check Stable compatibility", true)
    })?;
    if let Err(error) = wait_for(deadline, "signed Stable install confirmation", || {
        click_product_button(&window, "Install Stable and restart", true)
    }) {
        let _ = window.eval("location.hash = 'smoke_error=' + encodeURIComponent(document.querySelector('[role=dialog]')?.innerText || document.body.innerText)");
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
    wait_for(deadline, "installer restart", || false)
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
    wait_for(deadline, "in-window setup panel", || {
        open_setup_panel(&window)
    })?;
    wait_for(deadline, "cancel setup in the product", || {
        evaluates_true(
            &window,
            r#"(() => {
            const button = [...document.querySelectorAll('[role="dialog"] button')].find(
                item => item.textContent.trim() === "Cancel setup");
            if (!button || button.disabled) return false;
            button.click();
            return true;
        })()"#,
        )
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
    primary.eval("window.history.back()").map_err(display)?;
    check_titles("after Back", "Handoff Test Action")
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
            app.webview_windows().len() == 2
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
    wait_for(deadline, "permanent setup toolbar entry", || {
        open_setup_panel(&window)
    })?;
    wait_for(deadline, "terminal access action", || {
        click_product_button(&window, "Install terminal command", true)
    })?;
    let linked = home.join(".local/bin/atlas");
    wait_for(deadline, "bundled terminal command linked", || {
        linked
            .read_link()
            .is_ok_and(|path| process::bundled_cli().is_ok_and(|cli| path == cli))
    })?;
    wait_for(deadline, "terminal link status", || {
        evaluates_true(
            &window,
            r#"[...document.querySelectorAll('[role=dialog] [role=status]')].some(
            node => node.textContent === 'The terminal command is installed.' && node.getClientRects().length)"#,
        )
    })?;
    wait_for(deadline, "reference action", || {
        click_product_button(&window, "Check and repair reference data", true)
    })?;
    wait_for(deadline, "shared reference setup job", || {
        SETUP_RUNS.load(Ordering::Relaxed) == 1 && text_visible(&window, "Cancel setup")
    })?;

    crate::run_on_main(app, |app| {
        let _ = crate::open_product(app, "/search", None);
    })?;
    wait_for(deadline, "second product window", || {
        app.webview_windows().len() == 2
    })?;
    let second = app
        .webview_windows()
        .into_values()
        .find(|view| view.label() != PRIMARY)
        .ok_or("second product window unavailable")?;
    wait_for(deadline, "second window attachment", || {
        evaluates_true(&second, "Boolean(window.__ATLAS_DESKTOP_SETUP__?.token)")
    })?;
    wait_for(
        deadline,
        "setup gear opens drawer in the second window",
        || open_setup_panel(&second),
    )?;
    wait_for(deadline, "running job appears in the second drawer", || {
        text_visible(&second, "Cancel setup")
    })?;
    if app.webview_windows().len() != 2 {
        return Err("opening setup created an extra window".into());
    }
    wait_for(deadline, "cancel the job from the second window", || {
        click_product_button(&second, "Cancel setup", true)
    })?;
    wait_for(deadline, "cancelled setup state", || {
        text_visible(&window, "Setup was cancelled.")
    })?;
    let pinned_source = std::fs::read_dir(home.join("Library/Caches/pf2e-atlas/pf2e"))
        .map_err(display)?
        .next()
        .ok_or("managed fixture source missing")?
        .map_err(display)?
        .path();
    std::fs::remove_file(pinned_source.join(".hold-reference")).map_err(display)?;
    wait_for(deadline, "retry reference setup", || {
        click_product_button(&window, "Check and repair reference data", true)
    })?;
    wait_for(deadline, "verified no-op", || {
        text_visible(&window, "Reference data is already up to date.")
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
        click_product_button(&window, "Check and repair reference data", true)
    })?;
    wait_for(deadline, "publication confirmed in both drawers", || {
        text_visible(&window, "Reference data updated.")
            && text_visible(&second, "Reference data updated.")
    })?;
    if SETUP_RUNS.load(Ordering::Relaxed) != 3
        || process::reference_status(app)?.generation == initial.generation
        || app.webview_windows().len() != 2
    {
        return Err(
            "the reference refresh should publish once without replacing either window".into(),
        );
    }
    wait_for(deadline, "close the setup drawer", || {
        click_product_button(&second, "Close", true)
    })?;
    wait_for(
        deadline,
        "the product remains open after closing setup",
        || {
            evaluates_true(
                &second,
                r#"!Array.from(document.querySelectorAll('[role="dialog"]')).some(el => el.getClientRects().length)"#,
            )
        },
    )?;
    // With one product window, exercise the app-menu handler without racing
    // macOS focus changes caused by the user or another application.
    crate::run_on_main(app, move |_| {
        let _ = second.close();
    })?;
    wait_for(deadline, "one product window remains", || {
        app.webview_windows().len() == 1
    })?;
    wait_for(deadline, "close the original setup drawer", || {
        click_product_button(&window, "Close", true)
    })?;
    wait_for(deadline, "original drawer closes", || {
        evaluates_true(
            &window,
            r#"!Array.from(document.querySelectorAll('[role="dialog"]')).some(el => el.getClientRects().length)"#,
        )
    })?;
    crate::run_on_main(app, |app| {
        let _ = crate::show_workspace_setup(app, false);
    })?;
    wait_for(
        deadline,
        "native menu reopens the drawer with the last result",
        || text_visible(&window, "Reference data updated."),
    )?;
    if app.webview_windows().len() != 1 {
        return Err("setup menu opened an extra window".into());
    }
    Ok(())
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
    wait_for(
        Instant::now() + Duration::from_secs(10),
        "setup drawer entry",
        || open_setup_panel(&window),
    )?;
    check_model_selector(&window)
}

fn check_model_selector(window: &WebviewWindow) -> Result<(), String> {
    let deadline = Instant::now() + Duration::from_secs(5);
    window
        .set_size(tauri::LogicalSize::new(480.0, 720.0))
        .map_err(display)?;
    // Supply model availability to this isolated WebView fixture without loading models.
    wait_for(deadline, "fixture model selector", || {
        evaluates_true(
            window,
            r#"(() => {
        const state = window.__ATLAS_DESKTOP_SETUP__;
        if (!state?.reference || state.referenceLoading) return false;
        window.__atlasSmokeSetup = state;
        window.__ATLAS_DESKTOP_SETUP__ = {
            ...state, reference: { ...state.reference, modelsAvailable: true, canChangeModels: true,
                preferred: null, models: [
                    {id: 'bge-base', label: 'BGE Base', installed: true, usable: true},
                    {id: 'embeddinggemma', label: 'EmbeddingGemma', installed: true, usable: true}
                ]}
        };
        window.dispatchEvent(new Event('atlas-desktop-setup'));
        return true;
    })()"#,
        )
    })?;
    wait_for(deadline, "open model selector", || {
        evaluates_true(
            window,
            r#"(() => {
        const field = document.querySelector('[aria-label="Preferred search model"]');
        if (!field || field.disabled) return false;
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
        const field = document.querySelector('[aria-label="Preferred search model"]');
        const value = field?.closest('.ant-select')?.querySelector('.ant-select-content');
        return value?.textContent === 'EmbeddingGemma' && value.clientWidth >= value.scrollWidth;
    })()"#,
        )
    })?;
    window
        .eval(
            r#"window.__ATLAS_DESKTOP_SETUP__ = window.__atlasSmokeSetup;
        delete window.__atlasSmokeSetup;
        window.dispatchEvent(new Event('atlas-desktop-setup'));"#,
        )
        .map_err(display)
}

fn open_setup_panel(window: &WebviewWindow) -> bool {
    [
        "Setup and Updates",
        "Setup and Updates (setup in progress)",
        "Setup and Updates (setup incomplete)",
    ]
    .into_iter()
    .any(|label| click_product_button(window, label, false))
}

fn click_product_button(window: &WebviewWindow, text: &str, drawer: bool) -> bool {
    let label = serde_json::json!(text);
    let selector = if drawer {
        "[role=dialog] button"
    } else {
        ".topbar button"
    };
    evaluates_true(
        window,
        &format!(
            r#"(() => {{
            const button = [...document.querySelectorAll("{selector}")].find(
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
            r#"[...document.querySelectorAll('[role="dialog"] *')].some(
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
