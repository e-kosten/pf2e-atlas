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

pub fn forces_preflight_retry() -> bool {
    std::env::var_os("ATLAS_PILOT_SMOKE").is_some_and(|mode| mode == "preflight")
}

pub fn reference_mode() -> bool {
    std::env::var_os("ATLAS_PILOT_SMOKE").is_some_and(|mode| mode == "reference")
}

fn selector_mode() -> bool {
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
    let app = app.clone();
    thread::spawn(move || {
        match if selector_mode() {
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
        evaluates_true(
            &window,
            r#"(() => {
            const button = [...document.querySelectorAll('button')].find(
                item => item.getAttribute("aria-label") === "Setup and Updates");
            if (!button) return false;
            button.click();
            return true;
        })()"#,
        )
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
    })
}

fn run_reference(app: &AppHandle) -> Result<(), String> {
    let deadline = Instant::now() + Duration::from_secs(10);
    let home = process::home()?.canonicalize().map_err(display)?;
    let probe = process::probe()?;
    let initial = process::reference_status()?;
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
        click_product_button(&window, "Setup and Updates", false)
    })?;
    wait_for(deadline, "terminal access action", || {
        click_product_button(&window, "Enable terminal command", true)
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
            node => node.textContent.startsWith('Linked at') && node.getClientRects().length)"#,
        )
    })?;
    wait_for(deadline, "reference action", || {
        click_product_button(&window, "Check and repair reference data", true)
    })?;
    wait_for(deadline, "shared reference setup job", || {
        SETUP_RUNS.load(Ordering::Relaxed) == 1 && text_visible(&window, "Cancel setup")
    })?;

    crate::run_on_main(app, |app| {
        let _ = crate::open_product(app, "/search", true);
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
        || click_product_button(&second, "Setup and Updates", false),
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
    if process::reference_status()?.generation != initial.generation {
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
        || process::reference_status()?.generation == initial.generation
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
        Instant::now() + Duration::from_secs(10),
        "setup drawer entry",
        || click_product_button(&window, "Setup and Updates", false),
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
        const field = document.querySelector('[aria-label="Default search model"]');
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
        const field = document.querySelector('[aria-label="Default search model"]');
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
