//! Release discovery and user-authorized installation belong to the desktop host.
use std::{
    sync::{Mutex, atomic::Ordering},
    time::Duration,
};

use semver::Version;
use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Manager, WebviewWindow};
use tauri_plugin_updater::{Update, UpdaterExt};
use url::Url;

use crate::{Desktop, current_product, display, lock, opening, process, run_on_main, staged_setup};

const RELEASES: &str = "https://github.com/e-kosten/pf2e-atlas/releases";
const MANIFEST: &str = "atlas-desktop-update.json";
#[cfg(target_os = "macos")]
const PACKAGE: &str = "atlas-desktop-aarch64-apple-darwin.app.tar.gz";
#[cfg(windows)]
const PACKAGE: &str = "atlas-desktop-x86_64-pc-windows-msvc-setup.exe";

#[derive(Clone, PartialEq, Eq)]
enum Channel {
    Stable,
    Nightly { timestamp: String, commit: String },
}

impl Channel {
    fn from_version(version: &Version) -> Option<Self> {
        if !version.build.is_empty() {
            return None;
        }
        if version.pre.is_empty() {
            return Some(Self::Stable);
        }
        let mut fields = version.pre.as_str().split('.');
        if fields.next()? != "nightly" {
            return None;
        }
        let timestamp = fields.next()?;
        let commit = fields.next()?;
        if fields.next().is_some()
            || timestamp.len() != 14
            || !timestamp.bytes().all(|b| b.is_ascii_digit())
            || commit.len() != 13
            || !commit
                .strip_prefix('g')?
                .bytes()
                .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
        {
            return None;
        }
        Some(Self::Nightly {
            timestamp: timestamp.into(),
            commit: commit.into(),
        })
    }

    fn tag(&self, version: &Version) -> String {
        match self {
            Self::Stable => format!("release/v{version}"),
            Self::Nightly { timestamp, commit } => format!("nightly/{timestamp}.{commit}"),
        }
    }
}

fn newer(installed: &Version, candidate: &Version) -> bool {
    match (
        Channel::from_version(installed),
        Channel::from_version(candidate),
    ) {
        (Some(Channel::Stable), Some(Channel::Stable)) => candidate > installed,
        (
            Some(Channel::Nightly {
                timestamp: before, ..
            }),
            Some(Channel::Nightly {
                timestamp: after, ..
            }),
        ) => after > before,
        _ => false,
    }
}

#[derive(Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
struct Compatibility {
    workspace_schema: u32,
    foundry_commit: String,
    foundry_version: String,
}

#[derive(Clone)]
struct Offer {
    update: Update,
    compatibility: Compatibility,
}

#[derive(Default)]
pub(crate) struct Updates {
    state: Mutex<State>,
}

#[derive(Default)]
struct State {
    offer: Option<Offer>,
    phase: Phase,
    error: Option<String>,
}

#[derive(Clone, Copy, Default, Serialize, PartialEq, Eq)]
#[serde(tag = "kind", rename_all = "snake_case")]
enum Phase {
    #[default]
    Available,
    Downloading {
        percent: Option<u8>,
    },
    Installing,
}

#[derive(Serialize)]
pub(crate) struct Snapshot {
    version: String,
    phase: Phase,
    error: Option<String>,
}

impl Updates {
    pub(crate) fn snapshot(&self) -> Option<Snapshot> {
        let state = lock(&self.state);
        Some(Snapshot {
            version: state.offer.as_ref()?.update.version.clone(),
            phase: state.phase,
            error: state.error.clone(),
        })
    }
}

#[derive(Clone)]
struct ReleaseSource {
    root: &'static str,
    rehearsal_ca: Option<reqwest::Certificate>,
}

impl ReleaseSource {
    fn bundled() -> Result<Self, String> {
        let Some(root) = option_env!("ATLAS_UPDATER_REHEARSAL_ROOT") else {
            return Ok(Self {
                root: RELEASES,
                rehearsal_ca: None,
            });
        };
        let url = Url::parse(root).map_err(display)?;
        if url.scheme() != "https"
            || url.host_str() != Some("127.0.0.1")
            || url.port().is_none()
            || url.path() != "/releases"
            || !url.username().is_empty()
            || url.password().is_some()
            || url.query().is_some()
            || url.fragment().is_some()
        {
            return Err("The rehearsal feed must use HTTPS on the local loopback address.".into());
        }
        let pem = option_env!("ATLAS_UPDATER_REHEARSAL_CA")
            .ok_or("The rehearsal feed requires a disposable TLS certificate.")?;
        let certificate = reqwest::Certificate::from_pem(pem.as_bytes()).map_err(display)?;
        Ok(Self {
            root,
            rehearsal_ca: Some(certificate),
        })
    }

    fn client(&self, builder: reqwest::ClientBuilder) -> reqwest::ClientBuilder {
        if let Some(certificate) = &self.rehearsal_ca {
            builder
                .tls_certs_only([certificate.clone()])
                .no_proxy()
                .redirect(reqwest::redirect::Policy::none())
        } else {
            builder.redirect(reqwest::redirect::Policy::custom(release_redirect))
        }
    }
}

pub(crate) fn start(app: &AppHandle) {
    let Some(installed) =
        option_env!("ATLAS_DESKTOP_RELEASE").and_then(|value| Version::parse(value).ok())
    else {
        return;
    };
    let Some(channel) = Channel::from_version(&installed) else {
        return;
    };
    let Some(key) = option_env!("ATLAS_UPDATER_PUBLIC_KEY").filter(|key| !key.is_empty()) else {
        return;
    };
    let source = match ReleaseSource::bundled() {
        Ok(source) => source,
        Err(error) => {
            eprintln!("Could not configure Atlas update feed: {error}");
            return;
        }
    };
    let _ = rustls::crypto::ring::default_provider().install_default();
    let app = app.clone();
    std::thread::spawn(move || {
        let mut first = true;
        loop {
            if !first {
                std::thread::sleep(Duration::from_secs(4 * 60 * 60));
            }
            first = false;
            if app.state::<Desktop>().quitting.load(Ordering::Acquire) {
                return;
            }
            match tauri::async_runtime::block_on(check(&app, &installed, &channel, key, &source)) {
                Ok(offer) => {
                    let desktop = app.state::<Desktop>();
                    let mut state = lock(&desktop.updates.state);
                    if state.phase == Phase::Available {
                        let previous = state
                            .offer
                            .as_ref()
                            .map(|entry| entry.update.version.as_str());
                        if previous != offer.as_ref().map(|entry| entry.update.version.as_str()) {
                            state.offer = offer;
                            state.error = None;
                            drop(state);
                            staged_setup::broadcast(&app);
                        }
                    }
                }
                Err(error) => eprintln!("Atlas update check failed: {error}"),
            }
        }
    });
}

fn release_redirect(attempt: reqwest::redirect::Attempt<'_>) -> reqwest::redirect::Action {
    let destination = attempt.url();
    if attempt.previous().len() < 5
        && destination.scheme() == "https"
        && destination.port().is_none()
        && destination.username().is_empty()
        && destination.password().is_none()
        && matches!(
            destination.host_str(),
            Some("github.com" | "release-assets.githubusercontent.com")
        )
    {
        attempt.follow()
    } else {
        attempt.error("Untrusted Atlas release redirect")
    }
}

async fn release_tag(channel: &Channel, source: &ReleaseSource) -> Result<Option<String>, String> {
    let root = source.root;
    let builder = source.client(reqwest::Client::builder().timeout(Duration::from_secs(10)));
    match channel {
        Channel::Stable => {
            let response = builder
                .redirect(reqwest::redirect::Policy::none())
                .build()
                .map_err(display)?
                .head(format!("{root}/latest"))
                .send()
                .await
                .map_err(display)?;
            if response.status() == reqwest::StatusCode::NOT_FOUND {
                return Ok(None);
            }
            let location = response
                .headers()
                .get(reqwest::header::LOCATION)
                .and_then(|value| value.to_str().ok())
                .ok_or("Stable release did not resolve to an Atlas tag.")?;
            if location == root {
                return Ok(None);
            }
            let tag = location
                .strip_prefix(&format!("{root}/tag/"))
                .ok_or("Stable release resolved outside Atlas releases.")?;
            let version =
                Version::parse(tag.strip_prefix("release/v").ok_or("Invalid stable tag.")?)
                    .map_err(display)?;
            if Channel::from_version(&version) != Some(Channel::Stable)
                || Channel::Stable.tag(&version) != tag
            {
                return Err("Invalid stable tag.".into());
            }
            Ok(Some(tag.into()))
        }
        Channel::Nightly { .. } => {
            let response = builder
                .build()
                .map_err(display)?
                .get(format!(
                    "{root}/download/nightly/latest/nightly-version.txt"
                ))
                .send()
                .await
                .map_err(display)?;
            if response.status() == reqwest::StatusCode::NOT_FOUND {
                return Ok(None);
            }
            let mut response = response.error_for_status().map_err(display)?;
            let mut pointer = Vec::new();
            while let Some(chunk) = response.chunk().await.map_err(display)? {
                if pointer.len() + chunk.len() > 64 {
                    return Err("Invalid nightly release pointer.".into());
                }
                pointer.extend_from_slice(&chunk);
            }
            let tag = std::str::from_utf8(&pointer).map_err(display)?.trim();
            let identity = tag
                .strip_prefix("nightly/")
                .ok_or("Invalid nightly release pointer.")?;
            let version = Version::parse(&format!("0.0.0-nightly.{identity}")).map_err(display)?;
            if !matches!(Channel::from_version(&version), Some(channel @ Channel::Nightly { .. }) if channel.tag(&version) == tag)
            {
                return Err("Invalid nightly release pointer.".into());
            }
            Ok(Some(tag.into()))
        }
    }
}

async fn check(
    app: &AppHandle,
    installed: &Version,
    channel: &Channel,
    key: &str,
    source: &ReleaseSource,
) -> Result<Option<Offer>, String> {
    let Some(tag) = release_tag(channel, source).await? else {
        return Ok(None);
    };
    if tag == channel.tag(installed) {
        return Ok(None);
    }
    let base = format!("{}/download/{tag}", source.root);
    let source = source.clone();
    let current = installed.clone();
    let updater = app
        .updater_builder()
        .pubkey(key)
        .timeout(Duration::from_secs(15))
        .endpoints(vec![
            Url::parse(&format!("{base}/{MANIFEST}")).map_err(display)?,
        ])
        .map_err(display)?
        .configure_client(move |builder| source.client(builder))
        .version_comparator(move |_, release| newer(&current, &release.version))
        .build()
        .map_err(display)?;
    let found = match updater.check().await {
        Ok(found) => found,
        Err(tauri_plugin_updater::Error::ReleaseNotFound) => return Ok(None),
        Err(error) => return Err(display(error)),
    };
    let Some(found) = found else { return Ok(None) };
    let version = Version::parse(&found.version).map_err(display)?;
    if Channel::from_version(&version).is_none_or(|channel| channel.tag(&version) != tag)
        || found.download_url.as_str() != format!("{base}/{PACKAGE}")
    {
        return Err("Update metadata does not match the Atlas release.".into());
    }
    let compatibility =
        serde_json::from_value(found.raw_json["atlasCompatibility"].clone()).map_err(display)?;
    // Restoration guarantees that install and download require a signature bound to
    // the full Atlas version, including the nightly timestamp and commit.
    let mut update = updater
        .restore_update(found.raw_json)
        .map_err(display)?
        .ok_or("The Atlas update version changed.")?;
    update.timeout = Some(Duration::from_secs(15 * 60));
    Ok(Some(Offer {
        update,
        compatibility,
    }))
}

fn setup_notice(target: &Compatibility, workspace: &std::path::Path) -> Option<String> {
    let installed_schema = option_env!("ATLAS_WORKSPACE_SCHEMA")?.parse::<u32>().ok()?;
    if target.workspace_schema > installed_schema {
        return Some(
            "Atlas will be unavailable until you finish upgrading the workspace after restart."
                .into(),
        );
    }
    let reference = process::reference_status()
        .ok()
        .filter(|report| report.workspace == workspace)?;
    if reference.managed && reference.revision.as_deref()? != target.foundry_commit {
        return Some(format!(
            "This update includes Foundry PF2e {}. The Setup panel will show whether your reference data needs to be refreshed. Some features may be unavailable during setup.",
            target.foundry_version,
        ));
    }
    None
}

#[derive(Deserialize)]
#[serde(rename_all = "snake_case")]
pub(crate) enum Action {
    Details,
    Install,
}

#[tauri::command]
pub(crate) async fn desktop_update_action(
    window: WebviewWindow,
    token: String,
    version: String,
    action: Action,
) -> Result<Option<String>, String> {
    let app = window.app_handle().clone();
    let desktop = app.state::<Desktop>();
    let (_, instance) = current_product(&app)?;
    if !desktop
        .staged_setup
        .authorize_document(&window, &instance, &token)
    {
        return Err("Open the update again in the current Atlas window.".into());
    }
    let workspace = lock(&desktop.launcher)
        .as_ref()
        .filter(|launcher| launcher.ready.instance == instance)
        .map(|launcher| launcher.ready.workspace.clone())
        .ok_or("The Atlas service stopped.")?;
    let offer = lock(&desktop.updates.state)
        .offer
        .as_ref()
        .filter(|offer| offer.update.version == version)
        .cloned()
        .ok_or("A different update is available. Open Update again.")?;
    match action {
        Action::Details => tauri::async_runtime::spawn_blocking(move || {
            setup_notice(&offer.compatibility, &workspace)
        })
        .await
        .map_err(display),
        Action::Install => {
            {
                let _operation = desktop
                    .operation
                    .try_lock()
                    .map_err(|_| "Wait for Atlas setup to finish and try again.")?;
                let mut state = lock(&desktop.updates.state);
                if state.phase != Phase::Available {
                    return Err("Atlas is already installing an update.".into());
                }
                if state
                    .offer
                    .as_ref()
                    .is_none_or(|entry| entry.update.version != version)
                {
                    return Err("A different update is available. Open Update again.".into());
                }
                if desktop.staged_setup.active() || lock(&desktop.ordinary_setup).is_some() {
                    return Err("Wait for Atlas setup to finish and try again.".into());
                }
                state.phase = Phase::Downloading { percent: None };
                state.error = None;
            }
            staged_setup::broadcast(&app);
            tauri::async_runtime::spawn(async move {
                if let Err(error) = download_and_install(&app, offer.update, instance).await {
                    eprintln!("Atlas update failed: {error}");
                    let desktop = app.state::<Desktop>();
                    let mut state = lock(&desktop.updates.state);
                    state.phase = Phase::Available;
                    state.error = Some(error);
                    drop(state);
                    staged_setup::broadcast(&app);
                }
            });
            Ok(None)
        }
    }
}

async fn download_and_install(
    app: &AppHandle,
    update: Update,
    instance: String,
) -> Result<(), String> {
    let mut downloaded = 0u64;
    let bytes = update
        .download(
            |count, total| {
                downloaded += count as u64;
                let percent = total
                    .filter(|size| *size > 0)
                    .map(|size| ((downloaded.saturating_mul(100) / size).min(100)) as u8);
                let desktop = app.state::<Desktop>();
                let mut state = lock(&desktop.updates.state);
                let phase = Phase::Downloading { percent };
                if state.phase != phase {
                    state.phase = phase;
                    drop(state);
                    staged_setup::broadcast(app);
                }
            },
            || {},
        )
        .await
        .map_err(|error| {
            eprintln!("Could not verify or download Atlas update: {error}");
            "Atlas couldn’t download or verify the update. Try again.".to_owned()
        })?;
    let app = app.clone();
    tauri::async_runtime::spawn_blocking(move || install(&app, &update, &bytes, &instance))
        .await
        .map_err(display)?
}

fn install(app: &AppHandle, update: &Update, bytes: &[u8], instance: &str) -> Result<(), String> {
    let desktop = app.state::<Desktop>();
    let _operation = desktop
        .operation
        .try_lock()
        .map_err(|_| "Wait for Atlas setup to finish and try again.")?;
    if desktop.staged_setup.active() || lock(&desktop.ordinary_setup).is_some() {
        return Err("Wait for Atlas setup to finish and try again.".into());
    }
    let (old_base, current_instance) = current_product(app)?;
    if instance != current_instance {
        return Err("The workspace connection changed. Try the update again.".into());
    }
    let launcher = lock(&desktop.launcher)
        .take()
        .ok_or("The Atlas service stopped.")?;
    lock(&desktop.updates.state).phase = Phase::Installing;
    staged_setup::broadcast(app);
    let install = launcher.release().and_then(|()| {
        update.install(bytes).map_err(|error| {
            eprintln!("Could not install Atlas update: {error}");
            if matches!(error, tauri_plugin_updater::Error::Io(ref io)
                if io.kind() == std::io::ErrorKind::PermissionDenied)
            {
                "Atlas couldn’t write to its installation folder. Check the folder permissions and try again.".to_owned()
            } else {
                "Atlas couldn’t install the update. Try again.".to_owned()
            }
        })
    });
    if let Err(error) = install {
        desktop.staged_setup.clear_documents();
        recover(app, old_base);
        return Err(error);
    }
    #[cfg(target_os = "macos")]
    app.restart();
    #[cfg(windows)]
    Ok(())
}

fn recover(app: &AppHandle, old_base: Url) {
    let recovery = crate::start_launcher(app).and_then(|()| current_product(app));
    let result = run_on_main(app, move |app| {
        let desktop = app.state::<Desktop>();
        let views = app
            .webview_windows()
            .into_values()
            .filter_map(|window| {
                let url = window.url().ok()?;
                if url.origin() != old_base.origin() {
                    return None;
                }
                let route = &url[url::Position::BeforePath..];
                Some((window, route.to_owned()))
            })
            .collect::<Vec<_>>();
        match recovery {
            Ok((base, instance)) => {
                for (window, route) in views {
                    if base.origin() == old_base.origin() || window.label() == opening::PRIMARY {
                        if let Ok(target) = base.join(&route)
                            && target.origin() == base.origin()
                            && desktop
                                .staged_setup
                                .attach_window(app, window.label(), &base, &instance)
                                .is_ok()
                        {
                            if window.label() == opening::PRIMARY {
                                *lock(&desktop.primary) =
                                    opening::PrimaryWindow::Product(base.origin());
                            }
                            let _ = window.navigate(target);
                        }
                    } else if crate::open_product(app, &route, None).is_ok() {
                        let _ = window.destroy();
                    }
                }
            }
            Err(error) => {
                desktop.recovery.store(true, Ordering::Release);
                let _ = crate::show_bootstrap(app);
                crate::window_host::report("Atlas couldn’t reconnect", &error);
            }
        }
    });
    if let Err(error) = result {
        eprintln!("Could not restore Atlas windows: {error}");
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn isolates_channels_and_orders_same_core_nightlies() {
        let stable = Version::parse("0.2.2").unwrap();
        let next = Version::parse("0.2.3").unwrap();
        let first = Version::parse("0.2.3-nightly.20261115092300.gabcdef123456").unwrap();
        let second = Version::parse("0.2.3-nightly.20261116092300.gabcdef123456").unwrap();
        let rc = Version::parse("0.2.3-rc.1").unwrap();
        assert!(newer(&stable, &next));
        assert!(newer(&first, &second));
        assert!(!newer(&second, &first));
        assert!(!newer(&stable, &first));
        assert!(!newer(&first, &next));
        assert!(!newer(&stable, &rc));
        assert!(!newer(&rc, &next));
    }
    #[cfg(target_os = "macos")]
    #[test]
    #[ignore = "requires the Tauri CLI installed with npm --prefix desktop ci"]
    fn signed_upgrade_replaces_only_the_disposable_app() {
        use std::{fs, process::Command};
        use tauri::test::{mock_builder, mock_context, noop_assets};

        let temp = tempfile::tempdir().unwrap();
        let root = temp.path();
        let bundle_path = "PF2e Atlas.app/Contents/MacOS/atlas-desktop";
        let current = root.join(bundle_path);
        let candidate = root.join("candidate");
        let replacement = candidate.join(bundle_path);
        for (executable, contents) in [(&current, "installed"), (&replacement, "updated")] {
            fs::create_dir_all(executable.parent().unwrap()).unwrap();
            fs::write(executable, contents).unwrap();
        }
        let package = root.join(PACKAGE);
        assert!(
            Command::new("tar")
                .env("COPYFILE_DISABLE", "1")
                .arg("-czf")
                .arg(&package)
                .arg("-C")
                .arg(&candidate)
                .arg("PF2e Atlas.app")
                .status()
                .unwrap()
                .success()
        );

        let signer = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../node_modules/@tauri-apps/cli/tauri.js");
        let key = root.join("ephemeral.key");
        let status = Command::new("node")
            .arg(&signer)
            .args(["signer", "generate", "--ci", "--write-keys"])
            .arg(&key)
            .output()
            .unwrap()
            .status;
        assert!(status.success());
        let version = "0.2.3-nightly.20261116092300.gabcdef123456";
        let status = Command::new("node")
            .arg(&signer)
            .args(["signer", "sign", "--app-version", version])
            .arg(&package)
            .env_remove("TAURI_SIGNING_PRIVATE_KEY")
            .env("TAURI_SIGNING_PRIVATE_KEY_PATH", key)
            .env("TAURI_SIGNING_PRIVATE_KEY_PASSWORD", "")
            .output()
            .unwrap()
            .status;
        assert!(status.success());
        let key = fs::read_to_string(root.join("ephemeral.key.pub")).unwrap();
        let signature = fs::read_to_string(root.join(format!("{PACKAGE}.sig"))).unwrap();
        let mut context = mock_context(noop_assets());
        context
            .config_mut()
            .plugins
            .0
            .insert("updater".into(), serde_json::json!({ "pubkey": "" }));
        let app = mock_builder()
            .plugin(tauri_plugin_updater::Builder::new().build())
            .build(context)
            .unwrap();
        let updater = app
            .updater_builder()
            .pubkey(key.trim())
            .executable_path(current.clone())
            .endpoints(vec![
                Url::parse("https://example.invalid/manifest.json").unwrap(),
            ])
            .unwrap()
            .build()
            .unwrap();
        let manifest = |version| {
            serde_json::json!({
                "version": version,
                "platforms": {
                    "darwin-aarch64": {
                        "url": format!("https://example.invalid/{PACKAGE}"),
                        "signature": signature.trim()
                    }
                }
            })
        };
        let bytes = fs::read(package).unwrap();
        let wrong_version = updater.restore_update(manifest("0.2.3")).unwrap().unwrap();
        assert!(wrong_version.install(&bytes).is_err());
        assert_eq!(fs::read_to_string(&current).unwrap(), "installed");

        let signed = updater.restore_update(manifest(version)).unwrap().unwrap();
        signed.install(&bytes).unwrap();
        assert_eq!(fs::read_to_string(&current).unwrap(), "updated");
    }
}
